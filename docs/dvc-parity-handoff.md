# DVC parity — current state

`polaris_dvc` targets **output-shape parity** with upstream
`hancom-io/dvc` when run under `--dvc-strict`: given the same
`(hwpx, spec.json)` pair, both tools emit the same JID set in the
same JSON/XML field layout.

**We don't go further than shape parity.** Byte-exact cross-
verification against a running `DVC.exe` is out of scope for this
repo, and we don't build, package, or distribute any upstream DVC
binary. See [Scope & policy](#scope--policy) below.

## Scope & policy

The upstream project lives at `hancom-io/dvc` and is Apache-2.0
licensed. We retain a read-only snapshot of its source at
`third_party/dvc-upstream/` strictly for reference: the `JsonModel.h`
constant table drives our generated `jid_registry.rs`, and
`Source/Checker.cpp` is how we know which JIDs upstream actually
implements (vs. the many that are `break;` no-ops — see
[`parity-roadmap.md`](parity-roadmap.md)).

What we don't do:

- **No upstream build in CI.** There's no workflow in this repo that
  clones `hancom-io/dvc`, runs MSBuild on `DVCModel.sln`, or produces
  any upstream `.exe` / `.dll`.
- **No redistribution of compiled upstream code.** Release artifacts
  ship only polaris-authored crates (`polaris-dvc-*`) and the WASM
  bundle. The repo itself contains no checked-in upstream binaries.
- **No scripts that automate upstream building.** A previous iteration
  had `scripts/parity-windows.ps1` and friends plus a
  `dvc-parity.yml` workflow; both were removed when we committed to
  the policy above.

What shape parity still gives us:

| Piece | Status |
|---|---|
| Rust-side strict-mode output (JID gating, field order) | **working** — matches the shape upstream emits |
| Golden fixtures (44 cases, auto-generated) | **working** — checked into `testdata/golden/` |
| `--dvc-strict` CLI / WASM flag | **working** — polaris-only hint fields cleared at push time, upstream-compatible JIDs only |

If someone outside this repo later wants to prove byte-exact
agreement with `DVC.exe` on a Windows host, they're free to run
polaris under `--dvc-strict` and diff the output against a locally-
built upstream binary. The infrastructure for that is their
problem, not ours.

## Reference: why byte-exact parity is hard regardless

This section is historical context — preserved for anyone who'd want
to resurrect binary-level parity work later. It doesn't imply we'll
do that work.

When we did run `DVC.exe` on a Windows 11 VM against every
Hancom-Docs-produced HWPX we had (empty documents, plain text,
3 MB press releases), the same crash reproduced:

```
DVCModel!OWPMLReaderModule::OWPMLReader::FindPageInfo+0xb6
DVCModel!Checker::Initialize
DVCModel!DVCModule::doValidationCheck
```

At the crash frame, a `pSegArray` pointer is `NULL` and standard
error carries `"Unimplemented error for refID"` from the OWPML
library just before the dereference. The pattern — library logs an
unsupported refID, returns with a NULL array, caller iterates
blindly — reproduces across every Hancom-Docs HWPX we tested.
`!analyze -v` mis-attributes the crash to a shallower
`CheckCharShapeToCheckList` frame; `.ecxr; k` on the crash thread
surfaces the real `FindPageInfo` site.

The crash is **not** triggered by our rule JSON (upstream's own
`sample/test.json` reproduces it identically) and **not** triggered
by our HWPX containers being malformed (they open cleanly in Hangul
Office). The combination that does not crash is `{}` spec + any
HWPX, which early-returns from `CheckList::parsing` before
`FindPageInfo` is reached.

Working theory: the shipped `hwpx-owpml-model` only fully supports
the subset of OWPML emitted by desktop Hangul Office. Files
produced by the Hancom Docs cloud / mobile editors carry refIDs
that the library recognizes syntactically but doesn't implement, so
it bails out leaving a NULL segment array that `FindPageInfo`
dereferences.

Paths that could have unblocked this (none taken):

1. Test with a desktop Hangul Office HWPX. If the crash
   disappears, the fixtures are the proximate cause and we could
   regenerate from desktop output.
2. Patch or fork `hwpx-owpml-model`. Would have to live upstream
   or in a fork — out of scope for this repo.
3. What we settled on: skip byte-exact parity entirely, verify at
   the output-shape level via `--dvc-strict`. This is what this
   document now describes.

## Related reading

- [`cli-compat.md`](cli-compat.md) — flag-level compatibility with
  upstream `CommandParser.cpp`
- [`parity-roadmap.md`](parity-roadmap.md) — prioritized roadmap
  for closing remaining feature-parity gaps in the engine
