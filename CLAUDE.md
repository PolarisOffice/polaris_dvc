# CLAUDE.md — AI Agent working notes for polaris_dvc

Read this before doing any non-trivial work in this repo. It describes
the invariants that catch you out if you don't know them, the commands
that actually run the test matrix, and the exact places to edit when
you're wiring a new rule from upstream DVC.

This file is for AI agents (and humans doing AI-style drive-by edits).
It is intentionally denser than the user-facing `README.md`.

## What this project is, in one paragraph

polaris_dvc is a pure-Rust reimplementation of [hancom-io/dvc](https://github.com/hancom-io/dvc),
a Windows-only C++ DLL that validates HWPX (OWPML) documents against
JSON rule specs. The port keeps byte-level compatibility goals with
upstream — same rule-file schema (`sample/jsonFullSpec.json`), same
numeric error codes (`JID_*` from `Source/JsonModel.h`), same output
JSON field names and ordering — while running on macOS, Linux, and
`wasm32-unknown-unknown`. Upstream sources are vendored read-only at
`third_party/dvc-upstream/`; never edit them.

## Workspace layout

```
polaris_dvc/
├── crates/
│   ├── polaris-dvc-core/    rule engine, error codes, output model, Report
│   ├── polaris-dvc-hwpx/    pure-Rust HWPX (OWPML) parser (zip + quick-xml)
│   ├── polaris-dvc-format/  sniff() + parse() dispatch; reserves HWP5 slot
│   ├── polaris-dvc-cli/     `polaris-dvc` binary (DVC-compatible flags)
│   └── polaris-dvc-wasm/    wasm-bindgen shim (single `validate` entry)
├── tools/gen-jids/              regenerates jid_registry.rs from JsonModel.h
├── schemas/jsonFullSpec.json    upstream rule schema sample (unchanged copy)
├── testdata/golden/<nn>_.../    (doc.hwpx, spec.json, expected.json) triples
├── third_party/dvc-upstream/    READ-ONLY upstream snapshot, pinned commit
├── scripts/                     parity-windows.ps1, diff-dvc-outputs.sh, push.sh
└── docs/                        parity-roadmap, jid-registry, golden-tests, …
```

All five crates share dependencies via `[workspace.dependencies]` in the
root `Cargo.toml`. MSRV is pinned to 1.82 in `rust-toolchain.toml`.

## The three invariants you must not violate

### 1. JID values come from `JsonModel.h`, never from you

The 217 error codes live in `crates/polaris-dvc-core/src/jid_registry.rs`.
That file is **generated**, not hand-edited. It mirrors every
`#define JID_* N` in `third_party/dvc-upstream/Source/JsonModel.h`.

A drift test (`crates/polaris-dvc-core/tests/jid_registry_drift.rs`)
re-parses the upstream header on every `cargo test` and fails if the
committed registry's numeric values don't match. Bypass with
`POLARIS_ALLOW_JID_DRIFT=1` only while mid-edit.

The curated `jid` submodule in `crates/polaris-dvc-core/src/error_codes.rs`
exposes short-name aliases (`jid::TABLE_BGFILL_TYPE` →
`jid_registry::JID_TABLE_BGFILL_TYPE`). **Engine code uses the alias; the
alias refers to the registry.** Never hardcode an integer for an
ErrorCode — always go through `jid::`.

To regenerate the registry (only after `third_party/dvc-upstream/`
changes):

```sh
cargo run --manifest-path tools/gen-jids/Cargo.toml
```

### 2. `testdata/golden/` is authoritative for engine output

Every case directory holds `doc.hwpx`, `spec.json`, `expected.json`. The
`doc.hwpx` bytes are *reproducible from the in-Rust fixture template* in
`crates/polaris-dvc-core/tests/support/mod.rs`. On every `cargo
test` run, `tests/golden.rs` rebuilds each fixture and asserts:

- the freshly built `doc.hwpx` bytes equal the committed file (byte-exact),
- the engine's output JSON equals the committed `expected.json`.

If you change the fixture template or an engine checker, regenerate:

```sh
POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden
```

Review the diff before committing. Orphan directories (case dir with no
matching `Case` entry in `golden.rs`) fail a separate test — when
renaming a case, delete the old directory.

As of the XML-output change (Phase 8), each case directory also holds
`expected.xml`. The harness regenerates both JSON and XML expected
files from the same engine run — they stay semantically identical.

### 3. Output JSON shape matches upstream DVCOutputJson

The `ViolationRecord` struct in `crates/polaris-dvc-core/src/output.rs`
field-maps 1:1 to upstream `DVCOutputJson.cpp` writes. Field **order
matters** (serde_json uses `preserve_order`): upstream emits
`CharIDRef, ParaPrIDRef, errorText, PageNo, LineNo, ErrorCode, TableID,
IsInTable, IsInTableInTable, TableRow, TableCol, UseStyle, IsInShape,
UseHyperlink`. Conditional fields (`TableID`, `UseStyle`, …) are
included or omitted based on `OutputOption` — golden tests render under
`AllOption` so every field is present.

If you add a field, it must be because upstream emits it. Don't add
convenience fields.

## Daily commands

```sh
# Format + lint + test everything except WASM (which has a separate build):
cargo fmt --all
cargo clippy --workspace --exclude polaris-dvc-wasm --all-targets -- -D warnings
cargo test --workspace --exclude polaris-dvc-wasm

# WASM build (matches CI):
wasm-pack build crates/polaris-dvc-wasm --target web

# Regenerate golden fixtures after intentional engine/template change:
POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden

# Run the CLI directly:
cargo run -p polaris-dvc-cli -- -j -t spec.json doc.hwpx

# Push to origin (wrapper that handles the PAT from .env.local):
./scripts/push.sh
```

CI runs `cargo fmt --check`, `cargo clippy -D warnings`, and
`cargo test` on Ubuntu + macOS, plus `wasm-pack build` on Ubuntu.

## How to wire a new JID into the engine

End-to-end template for "upstream has rule X, we don't":

1. **Find the constant.** Look in
   `crates/polaris-dvc-core/src/jid_registry.rs`. It's already
   there — the registry is complete. E.g. `JID_TABLE_BGFILL_TYPE = 3037`.

2. **Add a short-name alias** in the `jid` module of
   `crates/polaris-dvc-core/src/error_codes.rs`:
   ```rust
   pub const TABLE_BGFILL_TYPE: ErrorCode = r::JID_TABLE_BGFILL_TYPE;
   ```

3. **Add a text message** in the same file's `ErrorCode::text(self)`
   match arm, if you want a human-readable string. Match upstream
   wording where possible.

4. **Extend the rule schema.** In
   `crates/polaris-dvc-core/src/rules/schema.rs`, add/extend the
   relevant struct (e.g. `TableSpec`, `BgFillSpec`). Use
   `#[serde(default)]` liberally — upstream specs are permissive about
   which fields are present. `#[serde(flatten)] extra:
   serde_json::Map<String, serde_json::Value>` catches unknown keys.
   If you add a new struct, re-export it from
   `crates/polaris-dvc-core/src/rules/mod.rs`.

5. **Parse the HWPX side if needed.** If the rule compares against a
   new OWPML attribute/element, extend `polaris-dvc-hwpx` — usually
   `src/header.rs` (doc-wide tables like borderFill, charShape,
   paraShape) or `src/section.rs` (body elements). Add public types to
   `src/types.rs` and re-export from `src/lib.rs`. Write a unit test in
   the same file against a minimal XML snippet.

6. **Write the checker.** In `crates/polaris-dvc-core/src/engine.rs`,
   add a `check_<thing>(ctx, node, spec) -> bool` function. The
   `bool` is "continue checking" — return `false` only when
   `ctx.push(v)` says the sink is full (typically `--simple` mode
   after the first violation). Always go through `ctx.push`; never
   push directly onto `ctx.records`.

7. **Add a golden case.** In
   `crates/polaris-dvc-core/tests/golden.rs`, append a `Case { name:
   "<nn>_<descr>", build, spec }` entry. `mkdir
   testdata/golden/<nn>_<descr>`. Run `POLARIS_REGEN_FIXTURES=1 cargo
   test -p polaris-dvc-core --test golden`. Commit all four files
   (source edits + three fixture files per case).

8. **Verify.**
   ```sh
   cargo fmt --all && \
   cargo clippy --workspace --exclude polaris-dvc-wasm --all-targets -- -D warnings && \
   cargo test --workspace --exclude polaris-dvc-wasm
   ```

## Conventions that catch out new contributors

### `Range64` = scalar-or-`{min,max}`

Spec values like `fontsize`, `margin.left`, etc. accept either a single
number or `{ "min": N, "max": M }`. The type is
`crates/polaris-dvc-core/src/rules/schema.rs::Range64`, with a
custom `Deserialize`. Scalar values become `Range64::Exact(n)`; objects
become `Range64::Bounds { min, max }`. Engine checkers use
`range.matches(actual)` — don't open-code the comparison.

### `ColorValue` accepts int or `#RRGGBB`

Same schema file, `ColorValue(pub u32)`. Users may write `"#FF0000"`,
`"FF0000"`, `16711680`, or a decimal string. Engine compares against the
HWPX-emitted `#RRGGBB` via `decode_hex_color`.

### Raw-string gotcha in Rust golden specs

The `spec: r#"..."#` literal in `tests/golden.rs` closes on `"#` — so
an inner `"#FFFFFF"` terminates the raw string early. Bump to
`r##"..."##` when your spec contains a `#` hex color. This bites
everyone at least once.

### BorderFill carries both borders AND fill

In upstream OWPML, `<hh:borderFill>` is a mixed node: it holds the
four line specs AND the background fill (`<hh:fillBrush>`,
`<hh:gradation>`, `<hh:imgBrush>`). A table references one borderFill
by `id`, and DVC's `table.border` and `table.bgfill` rules both look
up the same record. The parser reflects this: `BorderFill { left,
right, top, bottom, fill: Fill }`. The `Fill` enum's `.ordinal()`
matches upstream `BGFillType`: NONE=0, SOLID=1, PATTERN=2,
GRADATION=3, IMAGE=4. SOLID vs PATTERN is disambiguated by winBrush's
`hatchStyle != "NONE"`.

### Page/line tracking port

`engine.rs` contains a port of upstream `OWPMLReader::FindPageInfo`.
Page breaks open when a section-level paragraph's first lineseg has
`vert_pos == 0` or wraps back above the previous tail; `line_no`
accumulates by lineseg count. Don't "improve" this — it has to match
upstream bit-for-bit.

### `#![allow(clippy::collapsible_if)]` in engine

Intentional. Engine code uses the pattern
```rust
if mismatched {
    if !ctx.push(v) { return false; }
}
```
because `push` can itself signal stop-early, and collapsing to `&&`
across multi-line `push` calls hurts diff hygiene. Leave it.

### Never amend — always new commits

`CLAUDE.md` doesn't override the global rule: don't `git commit
--amend`, don't `git push --force`. Stack new commits. Cleanup
happens via regular rebase-on-merge on GitHub.

## Where to find things quickly

| looking for… | file |
|---|---|
| How a rule category is validated | `crates/polaris-dvc-core/src/engine.rs` — one `check_<cat>` fn per JID group |
| How to express a rule in JSON | `crates/polaris-dvc-core/src/rules/schema.rs` |
| Error-code numeric table | `crates/polaris-dvc-core/src/jid_registry.rs` (generated) |
| Short-name aliases + messages | `crates/polaris-dvc-core/src/error_codes.rs` |
| HWPX element parsing | `crates/polaris-dvc-hwpx/src/header.rs`, `section.rs` |
| HWPX types exposed to engine | `crates/polaris-dvc-hwpx/src/types.rs` |
| Output JSON shape | `crates/polaris-dvc-core/src/output.rs` |
| CLI flag surface | `crates/polaris-dvc-cli/src/main.rs` |
| Test fixture builder | `crates/polaris-dvc-core/tests/support/mod.rs` |
| Golden case list | `crates/polaris-dvc-core/tests/golden.rs` |
| Upstream references | `third_party/dvc-upstream/Source/{Checker,CheckList,DVCOutputJson,OWPMLReader,JsonModel}.*` |

## Structural-integrity checks (JID 11000-11999)

Polaris-original check category that catches cross-reference and
ZIP-container defects the DVC rule system doesn't address. Common in
hand-crafted or LLM-generated HWPX.

Currently implemented (all emitted from `engine::check_integrity`):

| JID | Condition |
|---|---|
| 11001 | `charPrIDRef` has no matching `<hh:charPr>` in header |
| 11002 | `paraPrIDRef` has no matching `<hh:paraPr>` |
| 11003 | `styleIDRef` has no matching `<hh:style>` |
| 11004 | paragraph has text but empty `<hp:linesegarray>` |
| 11010 | `mimetype` isn't the first ZIP entry |
| 11011 | `mimetype` entry is compressed (spec requires STORED) |
| 11012 | `mimetype` content isn't `application/hwp+zip` |
| 11020 | section's `binaryItemIDRef` not in the manifest |
| 11021 | manifest lists BinData href but the ZIP doesn't have the file |
| 11022 | ZIP has a `BinData/*` entry no manifest item points at |

Strict-mode gate filters 11000-11999 automatically — upstream DVC
never emits these, so DvcStrict output stays byte-compatible.

Adding a new integrity check:
1. Reserve a JID constant in `error_codes::jid` (11000-11999 range).
2. Add a text() arm (ideally through a `const X: u32 = jid::Y.value()`
   binding — match other integrity arms).
3. Add a function `check_integrity_<thing>(&mut Ctx) -> bool` in
   `engine.rs`. Use `integrity_violation(ctx, jid, msg)` to shape the
   record.
4. Wire it into `check_integrity`. No strict-gate edits needed.
5. Add a golden case with a minimal fixture that triggers the new
   JID; `spec.json` is usually `"{}"` (pure integrity, no DVC rules).

If the new check needs data the parser doesn't currently capture,
extend `StructuralFacts` in `polaris-dvc-hwpx/src/types.rs` and
populate it from `open_bytes` / `section::parse_section`.

## Extended vs DvcStrict profiles

The engine has two check profiles (`EngineOptions::profile`):

- **Extended** (default) — everything our engine can check. Includes
  rules that upstream `Checker.cpp` leaves as `break;` no-ops (e.g.
  `table.margin-*`, `table.bgfill-*`, `caption-*`, `parashape.horizontal`).
  Think of this as "stricter OWPML validator".
- **DvcStrict** — only emit JIDs upstream DVC.exe also validates. A
  single gate in `Ctx::push` drops violations whose `ErrorCode` is on
  the `dvc_strict_allows` block-list (engine.rs).

User-facing:
- CLI: `--dvc-strict`
- WASM: `validate(hwpx, spec, { dvcStrict: true })`
- Web demo: checkbox next to the Validate button
- XML output (`-x` / `--format=xml`) is an Extended-only extension.
  Upstream never implemented it; under `--dvc-strict` our CLI mirrors
  that by returning the same "NotYet" error and exit code 2. Under
  Extended (default), polaris emits its own attribute-per-field
  `<violation>` elements — see `Report::to_xml_string`.

When adding a new checker:
- If upstream also implements it → no strict-mode action needed.
- If upstream has it as `break;` → add the JID to the `dvc_strict_allows`
  block-list so strict output stays clean. Verify by running a fresh
  golden case with `profile: CheckProfile::DvcStrict` and expected `[]`.

## Roadmap snapshot

Current prioritized status lives in `docs/parity-roadmap.md`. As of this
file's last update:

- **P0-1**: DVC.exe byte-exact parity — scripted (`scripts/parity-windows.ps1`)
  but not yet run on a Windows host. Deferred until a Windows PC is
  available.
- **P0-2**: Table category expansion — in progress. Currently covered:
  `border`, `size`, `margin`, `treatAsChar`, `table-in-table`, `bgfill`
  (type/facecolor/pattoncolor/pattontype). Still to do: `caption`,
  `outside`, gradation-specific fields, effects.
- **P0-3**: Range-based `{min,max}` spec values — **done** via
  `Range64`.
- **P1+**: CharShape advanced fields, Shape/Footnote/Hyperlink details,
  XML output format, DVC bug-compat option.

`docs/parity-roadmap.md` has the detailed breakdown with file pointers.
`docs/dvc-parity-status.md` tracks the Windows-side build attempts.

## When in doubt

- The upstream behavior is authoritative. If our engine diverges from
  what `Source/Checker.cpp` does, the engine is wrong.
- A test that "passes but I don't know why" is a regression waiting to
  happen. Add a golden case that exercises the path.
- Never commit expected.json changes without understanding why each
  line changed. The file is how we notice silent regressions.
