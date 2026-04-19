# DVC parity verification — status

Tracks progress on running upstream `hancom-io/dvc`'s `ExampleWindows.exe`
against our committed fixtures to verify bit-exact parity.

## Summary

**Not yet working.** Upstream DVC has several build-configuration quirks
that make out-of-the-box CI compilation difficult. We've peeled through
several layers; remaining issue is an unexplained `Source\pch.h`
reference the compiler emits for DVCModel's own sources.

## Environment

- CI workflow: [.github/workflows/dvc-parity.yml](../.github/workflows/dvc-parity.yml)
- Diff script: [scripts/diff-dvc-outputs.sh](../scripts/diff-dvc-outputs.sh)
- Trigger: `workflow_dispatch` + push on workflow/script/fixture changes
- Runner: `windows-latest` (VS2022, v143 toolset)

## Issues resolved so far

| # | Problem | Fix |
|---|---|---|
| 1 | VS2017 v141 toolset absent on runner | Retarget to `/p:PlatformToolset=v143` |
| 2 | `WindowsTargetPlatformVersion 10.0.17763.0` missing | Detect installed SDK dynamically via pwsh |
| 3 | v141's `stdafx.h`/`stdafx.cpp` PCH convention broken on v143 | Sed-patch `<PrecompiledHeader>` → `NotUsing` only under `opensource/hwpx-owpml-model/` |
| 4 | x64 `<ItemDefinitionGroup>`s in OWPML vcxprojs lack `<AdditionalIncludeDirectories>` | Build `Platform=x86` instead; solution name is `x86` (not `Win32`) |
| 5 | `jsoncpp` missing (upstream vcxproj points at `./opensource/vcpkg/packages/jsoncpp_x86-windows/`) | `vcpkg install jsoncpp:x86-windows` then patch DVCModel.vcxproj to absolute package path |
| 6 | PowerShell `-replace` backslash escaping — produced `C:\\vcpkg\\...` in vcxproj | Pass paths verbatim; PS regex-replacement doesn't interpret `\` |

## Open issue

Build fails with:

```
D:\a\.../dvc\Source\pch.h(11,10): error C1083: Cannot open include file:
  'json/json.h': No such file or directory [.../DVCModel.vcxproj]
```

…but **no `Source\pch.h` exists in the upstream tree.** The real pch.h
is at the repo root (`dvc/pch.h`), contains only `#include "framework.h"`,
and has no json includes. Yet the compiler reports an error from a
non-existent `Source\pch.h` at line 11 column 10.

Hypotheses to chase:

- MSBuild may auto-synthesize a `pch.h` next to ClCompile sources when
  `PrecompiledHeaderFile=pch.h` is set and the file isn't found at the
  expected location.
- Upstream may have a pre-build step we're not running that copies /
  generates `Source/pch.h`.
- Our sed patch may have a side effect on DVCModel.vcxproj we're not
  seeing (though it only touches `opensource/hwpx-owpml-model/`).

## Recommended next step: local UTM/Windows VM iteration

GitHub Actions Windows runners cost ~10 min per iteration and the
Actions logs API is rate-limited heavily (often returns
`BlobNotFound` for several minutes after a run). Iterating on MSBuild
issues is faster on a local Windows VM:

1. **UTM (Apple Silicon, free)**: install Windows 11 ARM64, then
   install VS2022 Community with "Desktop development with C++".
2. Clone this repo + `hancom-io/dvc` side by side, run the steps from
   `dvc-parity.yml` manually. Each iteration is ~2 minutes instead of
   ~10, and the MSBuild output is immediately available.
3. Once DVC.exe runs cleanly on fixtures locally, port the exact
   winning combination back into `dvc-parity.yml`.

## What we have regardless

Even without DVC.exe parity proof, polaris already mirrors upstream's
published rule-file shape (`third_party/dvc-upstream/sample/test.json`)
and every JID value comes from the generated `jid_registry.rs`. Our
`expected.json` files are our engine's own output pinned as regression
anchors. Swapping those for true DVC.exe output is the last-mile
verification step.
