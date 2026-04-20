# DVC Parity Investigation — Handoff

**Status as of 2026-04-20**: unresolved. Upstream `DVC.exe` crashes
with access violation (exit code `-1073741819` / `0xC0000005`) for
every non-trivial rule spec on our CI. Root cause narrowed but not
identified. A new local workflow revision now builds an instrumented
`DvcProbeHarness.exe` and enables crash-dump upload. Run #34 proved the
crash happens inside `doValidationCheck()` for a non-empty spec, after
`createDVC()` and `setCommand()` succeed. Run #35 captured the stack:
`OWPMLReaderModule::OWPMLReader::FindPageInfo` →
`Checker::Initialize` → `DVCModule::doValidationCheck`. A minidump was
uploaded as artifact `dvc-crashdumps-24655412858`. This document lets a
fresh agent (AI or human) pick up without reading the full conversation
history.

## 1. What we're trying to do

`polaris_rhwpdvc` is a pure-Rust reimplementation of
[`hancom-io/dvc`](https://github.com/hancom-io/dvc) — a Windows-only
C++ DLL that validates HWPX documents against a JSON rule spec.
Our engine targets **byte-exact output parity** with upstream when
run under `--dvc-strict` mode, so golden tests in
`testdata/golden/*/expected.json` are meant to be
reproducible via `DVC.exe` on a Windows machine.

**The parity workflow** (`.github/workflows/dvc-parity.yml`) does:

1. `build` job (Windows runner):
   - Clone `hancom-io/dvc@main` + OSS deps (`hwpx-owpml-model`,
     `rapidjson`, `jsoncpp`).
   - Build `jsoncpp` from source as a shared DLL (`Win32/Release/v143`).
   - Patch `DVCModel.vcxproj` to point at our locally-built jsoncpp
     (upstream's paths reference a private vcpkg layout CI can't
     populate).
   - Disable OWPML sub-projects' `stdafx.h` PCH (v143 compat).
   - Add `DVC_EXPORTS` to `DVCModel.vcxproj`'s preprocessor
     definitions (upstream only defines `DVCMODEL_EXPORTS`, but
     `export/export.h` gates `__declspec(dllexport)` on `DVC_EXPORTS`).
   - `msbuild DVCModel.sln /p:Configuration=Release /p:Platform=x86 /p:PlatformToolset=v143`.
   - Upload `ExampleWindows.exe` + `DVCModel.dll` + `jsoncpp.dll` as
     artifact `dvc-windows-x86-<run_id>`.

2. `parity` job (Windows runner):
   - Download artifact (fresh build OR from a prior run via
     `workflow_dispatch.inputs.reuse_artifact_from_run=<run_id>`).
   - Run `scripts/parity-probes.ps1` against the binary to
     systematically characterize behavior.
   - Run DVC.exe against every fixture in `testdata/golden/*/`.
   - Byte-diff output vs committed `expected.json`.

### Iteration mechanics (important)

- **`scripts/parity-probes.ps1`** is NOT in `push.paths` filter.
  Editing only this file + pushing does NOT trigger a CI run.
  Iterating on probes uses **manual `workflow_dispatch`** with
  `reuse_artifact_from_run=<last-good-run-id>` → `build` job is
  skipped (via `if:` condition), parity runs in ~2 min against
  the already-built binary.
- Editing `.github/workflows/dvc-parity.yml` or the workflow's
  inlined patches triggers a full ~7-min rebuild.
- Last known-good build artifact: **run #30, id `24653580323`**
  (binary = unpatched upstream, all source-patch experiments
  reverted).

Dispatch from CLI:

```bash
curl -X POST -H "Authorization: Bearer $GITHUB_PAT" \
     -H "Accept: application/vnd.github+json" \
     "https://api.github.com/repos/miles-hs-lee/polaris_rhwpdvc/actions/workflows/dvc-parity.yml/dispatches" \
     -d '{"ref":"main","inputs":{"reuse_artifact_from_run":"24653580323"}}'
```

## 2. The crash

```
D:\...\ExampleWindows.exe -j -o --file=OUT spec.json doc.hwpx
  > Hello World!            # ← sometimes printed, sometimes not
                            # (stdout buffering-dependent)
exit_code=-1073741819       # = 0xC0000005 (access violation)
                            # no --file=OUT produced
```

Reproduces on every windows-latest runner, every build,
regardless of DVC build config or source patches tried.

## 3. Variables ruled out (empirical, not speculation)

All of these were tested via `scripts/parity-probes.ps1` across CI
runs #23-#33. Every one produced **no change in crash behavior**
versus baseline.

| Variable | Test | Verdict |
|---|---|---|
| HWPX doc content | `empty.hwpx` (18 KB), `테스트.hwpx` (19 KB, Korean filename), `260210 국가AI전략위…hwpx` (3.1 MB complex) | irrelevant |
| HWPX path encoding | ASCII / Korean / copied-to-ASCII | irrelevant |
| Spec file size | 2 bytes (`{}`), 16 bytes, 27 KB | irrelevant |
| Spec file encoding | `Out-File -Encoding ascii`, `Set-Content`, `[System.IO.File]::WriteAllBytes`, explicit UTF-8 BOM | irrelevant |
| Spec filename | ASCII, with spaces, with Korean | irrelevant |
| Spec `[Json schema]\n` prelude | with / without | irrelevant |
| Spec key name | registered in `checkListMapData` (`charshape`) / unregistered (`nonexistent`, `objproperty`) | irrelevant |
| Spec key position | alphabetically early (`a`) / late (`z`) / empty string (`""`) | irrelevant |
| Spec value type | `{}` object / `1` int / `"x"` str / `[]` array / `null` | irrelevant |
| Spec key count | 1 key / 2 keys | irrelevant (both crash) |
| CLI flags | `-j`, `-j -o`, `-j -d`, `-j -s`, `-j -a`, `-j -c`, no `--file=` | irrelevant |
| Upstream source patches attempted | null-terminator off-by-one (#26), missing `end()` check (#27) | both failed, reverted |
| Build config | v141 (wouldn't install) / v143 (used) / x86 / x64 | v143+Win32 builds; x64 incomplete per upstream vcxproj |
| jsoncpp version | HEAD / pinned `8190e061` (upstream's intended pin, blocked by typo) | same behavior |

## 4. The one variable that matters

**`root.getMemberNames().size() > 0`** — the instant DVC.exe parses
any spec with at least one top-level member, it crashes.

- `{}` → exit 0, graceful "초기화 진행에 오류가 있습니다" message.
- `{}` + 27 KB of whitespace padding → exit 0, graceful.
- `{"anything":anyvalue}` → `0xC0000005`.

## 5. Where the crash is (code pointer)

`third_party/dvc-upstream/Source/CheckList.cpp::parsing()` at the
for-loop body starting at line 103:

```cpp
std::vector<std::string> members = root.getMemberNames();

if (members.size() <= 0)
    return false;                    // <-- `{}` path; early return,
                                     //     no crash

for (int i = 0; i < members.size(); i++)
{
    int nID = -1;
    mapIter = checkListMapData.find(members[i]);            
    switch (mapIter->second)         // <-- UB if iter == end()
    {
        case JID_CHAR_SHAPE: {
            CheckListModule::CCharShape* charshape = 
                new CheckListModule::CCharShape(
                    root[mapIter->first].toStyledString().c_str());
            ...
```

`checkListMapData` is a static global `std::map<std::string, int>` in
the same file, initialized with 21 JIN→JID bindings at lines 21-43.
`sample/jsonFullSpec.json` has 22 top-level keys (`objproperty` is
missing from the map).

## 6. Hypotheses tried and disproved

### A. Null-terminator off-by-one at line 90-91

Theory: `new char[size+1]; memset(..., size); fread(..., size, ...)`
leaves `buffer[size]` indeterminate. `reader.parse(buffer, root)`
constructs `std::string(buffer)` → `strlen(buffer)` reads past the
allocation.

**Status**: theoretically correct UB, empirically NOT the trigger.
Run #26 applied the one-byte fix (`size` → `size + 1`) via sed and
the crash survived.

### B. Missing `end()` check before `mapIter->second` dereference

Theory: for a spec key not in `checkListMapData`, `find()` returns
`end()`; `switch (mapIter->second)` dereferences end iterator → UB.

**Status**: also theoretically correct UB, but disproved by
probe 6b — `{"charshape":{}}` (key IS registered, `find()` returns
valid iterator) ALSO crashes. Run #27 applied sed-injected
`if (mapIter == checkListMapData.end()) continue;` before the
switch. Crash survived.

## 7. Remaining candidates (not yet tested)

- **Static init order fiasco**: `checkListMapData` is a static
  global in the DLL with complex (std::map<std::string, int>)
  initialization. If its init hasn't completed by the time
  `parsing()` runs (e.g., from a thread that loaded the DLL
  through a non-standard path), the map could be empty at
  runtime. That would fit: `{}` never enters the loop → OK; any
  non-empty spec enters the loop, `find()` against an empty map
  always returns `end()`, `mapIter->second` is UB → crash.
  **But probe 6b with a registered key also crashes** — if the
  map were empty that path wouldn't exercise anything further.
  Unless the crash is specifically `end()` deref and all our
  observed crashes are in that same spot regardless of key.

- **CRT / STL ABI mismatch**: `DVCModel.dll` built with
  `TreatWChar_tAsBuiltInType=false` (wchar_t = unsigned short),
  `ConformanceMode=false`. `ExampleWindows.exe` built with
  defaults (wchar_t distinct, `/permissive-`). `std::string`,
  `std::map`, `std::vector` could differ in layout across the
  DLL boundary. Upstream uses v141, we use v143 (v141 can't be
  installed on windows-latest). But `createDVC`/`deleteDVC` are
  `extern "C"` so direct name-mangling isn't the issue — the
  issue would be STL containers returned/passed through the
  interface. Looking at `IDVC::setCommand(int, DVC_CHAR* argv[])`
  it only takes plain pointers, no STL types. So this shouldn't
  matter *for the probe-level call*. But `checkListMapData` is
  inside the DLL — its behavior shouldn't depend on the caller's
  STL. So this theory probably doesn't hold.

- **Compiler / runtime bug specific to v143 + Release + Win32**:
  unlikely but possible.

- **Debug build with MiniDump**: would give an actual crash
  address + stack. Not yet attempted because Debug requires
  different vcpkg-ish paths that aren't populated. Might be
  feasible with further vcxproj patching.

## 8. What a fresh agent should try next

### Option A: crash-dump collection

Modify the workflow to enable Windows Error Reporting minidump
collection and upload the dump as an artifact. With symbols from a
Debug build, the exact crash site becomes visible.

Sketch:

```powershell
reg add 'HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps' `
    /v DumpFolder /t REG_EXPAND_SZ /d 'C:\crashdumps' /f
reg add 'HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps' `
    /v DumpType /t REG_DWORD /d 2 /f
# run DVC.exe - crash produces dump in C:\crashdumps
# upload as artifact
```

Then `cdb.exe -z <dump>` locally to get the stack.

**Implemented locally in `.github/workflows/dvc-parity.yml` on
2026-04-20.** The parity job now configures
`HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps`
to write full dumps under `dvc-crashdumps/` and uploads them as
artifact `dvc-crashdumps-<run_id>`. A reused old artifact can still
produce dumps for `ExampleWindows.exe`; a fresh build is needed for
the new harness below.

### Option B: Debug-build pdb publication

Build DVCModel with `/p:Configuration=Debug /p:Platform=x86` and
run the parity probes against the Debug binary. If the Debug CRT
fails-fast with a specific assertion instead of bare 0xC0000005,
we learn exactly which assertion.

The upstream's Debug|Win32 config references source-built jsoncpp
at `./opensource/jsoncpp/json_git/lib/Release` (which we already
produce in the Release build). Just change `Configuration=Release`
to `Configuration=Debug` and adjust the lib path for Debug.

### Option C: Replace ExampleWindows with a minimal harness

Write a tiny C++ program that:
1. Calls `DVC::createDVC()`.
2. Calls `setCommand` with `-j -o spec.json doc.hwpx`.
3. Calls `doValidationCheck()`.
4. Prints before and after each C++ call via `std::fflush(stdout)`.

Compile against the DLL. Run it. If every print before the crash
shows, we know which interface call is the culprit. Right now
ExampleWindows prints "Hello World!" BEFORE `createDVC`, but
because stdout is pipe-buffered in our runner, the line can get
lost if the crash happens right after. Using `fflush` after every
print disambiguates.

**Implemented locally in `.github/workflows/dvc-parity.yml` on
2026-04-20.** The build job compiles `DvcProbeHarness.exe` as Win32
next to `ExampleWindows.exe`, loads `DVCModel.dll` with
`LoadLibraryW`, resolves `createDVC`/`deleteDVC` with `GetProcAddress`,
and flushes after each boundary:

```
start
LoadLibraryW ok
GetProcAddress ok
before createDVC
after createDVC
before setCommand
after setCommand
before doValidationCheck
after doValidationCheck
before output
after output
before deleteDVC
after deleteDVC
```

The parity job runs the harness once with `{}` and once with
`{"charshape":{}}`. If it crashes, the last printed `before ...` line
is the failing call boundary. If `DvcProbeHarness.exe` is missing, the
job is using an old reused artifact; dispatch a fresh build instead of
`reuse_artifact_from_run`.

Run #34 (`24654899545`) result:

- `{}`: returned from `doValidationCheck()` with result `0`, then
  `output()` wrote `null`.
- `{"charshape":{}}`: printed through `before doValidationCheck`, then
  exited `-1073741819`.
- No WER dump files were uploaded, so the next local revision adds
  `SetUnhandledExceptionFilter`, `MiniDumpWriteDump`, and `StackWalk64`
  directly inside `DvcProbeHarness.exe`.

Run #35 (`24655412858`) result:

- The harness-level exception filter worked and uploaded
  `dvc-crashdumps-24655412858` (artifact ID `6526962915`).
- Exception: `0xc0000005` at `7460D6A6`.
- Symbolized stack from the job log:
  ```text
  OWPMLReaderModule::OWPMLReader::FindPageInfo+0xb6
  Checker::Initialize+0x254
  DVCModule::doValidationCheck+0x2c7
  ```
- This rules out `createDVC`, export-table resolution, and
  `CommandParser::setCommand` as the immediate crash point. The next
  source-level investigation should inspect `Source/OWPMLReader.cpp`
  `FindPageInfo` and the document/page-info state that
  `Checker::Initialize` passes into it.

### Option D: Skip CI-based parity, rely on Windows PC run

The parity gate exists to guarantee byte-exact output matching
with upstream's reference binary. If CI consistently can't build
a working upstream binary, the fallback is:

1. Someone with Windows access runs `scripts/parity-windows.ps1`
   on their own machine (documented in `docs/windows-parity-howto.md`).
2. The resulting `expected.json` per-fixture gets committed.
3. CI's parity job is scaled back to "our engine output equals
   committed expected.json" without needing DVC.exe at runtime.

This is the **pragmatic path forward** given how much time has
gone into the CI build.

## 9. Important files to read

- `.github/workflows/dvc-parity.yml` — full CI definition + probe
  invocation.
- `scripts/parity-probes.ps1` — the probe matrix. Run it locally
  on Windows against a pre-built DVC.exe to reproduce.
- `scripts/parity-windows.ps1` — "run DVC.exe against every
  fixture" script for hand-run on Windows PCs.
- `scripts/diff-dvc-outputs.sh` — JSON diff between `expected.json`
  and runtime output.
- `docs/windows-parity-howto.md` — manual walkthrough for Windows
  contributors.
- `third_party/dvc-upstream/Source/CheckList.cpp` — the file
  containing the crash site (lines 66-130).
- `third_party/dvc-upstream/Source/DVCModule.cpp` — entry into
  `doValidationCheck`.
- `testdata/real-samples/` — three real Hancom Docs HWPX files
  (empty, with-text, complex 3.1 MB press release).
- `schemas/jsonFullSpec.json` — the canonical 22-key upstream spec
  that triggers the crash.

## 10. Build-history summary (abbreviated)

| Run | What it tested | Outcome |
|---|---|---|
| #13-#22 | Various build recipes (v141/v143, x86/x64, vcpkg vs source jsoncpp, PCH handling) | Build eventually succeeded (#22) but binary crashed with 0xC0000005 on every args-bearing invocation |
| #23 | Added probe ladder (`-h` / `-j` / bogus paths) | probes 0-3 all exit 0 cleanly; crash only in Checker |
| #24-#25 | Real Hancom HWPX + `{}` spec, then + full jsonFullSpec | `{}`: exit 0 graceful. jsonFullSpec: crash |
| #26 | null-terminator fix applied | crash survived — theory A disproved |
| #27 | `end()` check patch applied | crash survived — theory B disproved |
| #28 | All patches reverted + orthogonal probe matrix (size/key/prelude/content) setup crashed due to pwsh ConvertFrom-Json | no data |
| #29 | Cancelled (probe-only change triggered rebuild — refactor needed) | — |
| #30 | Final mandatory rebuild after externalizing probe matrix to `scripts/parity-probes.ps1` | matrix ran; ruled out size, key identity, prelude as triggers |
| #31-#32 | Extended matrix (encoding / flag variations), `reuse_artifact_from_run` flow | ruled out encoding, BOM, CLI flags, output-file writing |
| #33 | JSON-shape matrix (key position, value types, multi-key) | all 8 variants crash identically; every external variable now eliminated |

## 11. Key conclusions so far

1. The DVC Windows binary itself builds cleanly and loads correctly.
   Basic lifecycle (createDVC → setCommand with trivial args →
   doValidationCheck graceful early-exit → deleteDVC) works.
2. The failure is specifically in `CheckList::parsing()` loop body
   upon processing the first top-level spec member. Input encoding,
   path format, CLI flags, output configuration, HWPX content, and
   upstream source-level null-terminator/end() fixes all fail to
   change this.
3. The `{}`-spec path is the only reliable "Initialize returns false
   gracefully" path. Anything non-empty hits the crash regardless of
   what the non-empty content looks like.
4. Given the exhaustive external-variable elimination, the next
   productive step is **internal observation**: symbols, debug build,
   or crash-dump collection. No more external-parameter sweeping
   will yield new information.

## 12. Contact / context

- Repo: <https://github.com/miles-hs-lee/polaris_rhwpdvc>
- Branch protection on main: force-push and deletion blocked; no
  other gates.
- PAT is in `.env.local` (gitignored); `scripts/push.sh` wraps it.
- Workflow runs: <https://github.com/miles-hs-lee/polaris_rhwpdvc/actions>
- Hand-off date: 2026-04-20.
