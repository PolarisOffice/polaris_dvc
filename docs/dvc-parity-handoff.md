# DVC parity — current state

`polaris_dvc` targets **byte-exact output parity** with upstream
`hancom-io/dvc` when run under `--dvc-strict`. The parity contract
is: given the same `(hwpx, spec.json)` pair, `polaris-dvc` and
`DVC.exe` produce identical `expected.json` output.

This document records where we stand on achieving that contract,
why the verification pipeline is partially blocked, and what would
unblock it.

## Summary

| Piece | Status |
|---|---|
| Rust-side strict-mode output (JID gating, field order) | **working** — matches the shape upstream emits |
| Golden fixtures (44 cases, auto-generated) | **working** — checked into `testdata/golden/` |
| `DVC.exe` running against the same fixtures on Windows | **blocked** — see below |
| CI workflow (`.github/workflows/dvc-parity.yml`) | **scripted but not run green** — waits on the block |
| Cross-verified "bit-for-bit matches upstream" evidence | **pending** |

We can't sign off on *provable* byte-exact parity until the Windows
side runs successfully against our fixtures. The engine is built to
the parity contract, but the third-party loop is currently open.

## What blocks Windows-side verification

The upstream `DVC.exe` binary links against `hwpx-owpml-model`, a
separate Hancom library that parses HWPX into the OWPML object tree
before `DVC.exe`'s rule engine walks it. Running `DVC.exe` under
`cdb` on a Windows 11 VM against every real HWPX we have produces
the same crash:

```
DVCModel!OWPMLReaderModule::OWPMLReader::FindPageInfo+0xb6
DVCModel!Checker::Initialize
DVCModel!DVCModule::doValidationCheck
```

At the crash frame, a `pSegArray` pointer is `NULL` and standard
error carries `"Unimplemented error for refID"` from the OWPML
library just before the dereference. The pattern — library logs an
unsupported refID, returns with a NULL array, caller iterates
blindly — reproduces on every Hancom-Docs-produced HWPX we have:
empty documents, plain text, and a 3 MB press release all hit the
same frame. The `!analyze -v` automatic analyzer mis-attributed the
crash to a shallower `CheckCharShapeToCheckList` frame; `.ecxr; k`
on the crash thread surfaces the real `FindPageInfo` site.

The crash is not triggered by our rule JSON (upstream's own
`sample/test.json` reproduces it identically) and not triggered by
our HWPX containers being malformed (they open cleanly in Hangul
Office). The combination that does **not** crash is `{}` spec + any
HWPX, which early-returns from `CheckList::parsing` before
`FindPageInfo` is reached.

Working theory: the shipped `hwpx-owpml-model` only fully supports
the subset of OWPML emitted by desktop Hangul Office. Files
produced by the Hancom Docs cloud / mobile editors carry refIDs
that the library recognizes syntactically but doesn't implement, so
it bails out leaving a NULL segment array that `FindPageInfo`
dereferences.

## Paths forward

1. **Test with a desktop Hangul Office HWPX.** If the crash
   disappears, our fixtures are the proximate cause and we can
   regenerate them from desktop output. 30-day trial is available
   from Hancom.

2. **Patch or fork `hwpx-owpml-model`.** Add a guard that returns
   an empty segment array safely when an unsupported refID is
   encountered. Out-of-scope for the polaris repo proper; would
   have to live upstream or in a fork.

3. **Skip byte-exact parity; verify at the output-structure
   level.** If bit-for-bit against `DVC.exe` turns out to be
   unreachable, we can still guarantee that `--dvc-strict` emits
   the same JID set and field shape as upstream for the documents
   DVC.exe *can* process. This is the pragmatic fallback and would
   retitle the goal from "parity" to "DVC-compatible output."

Path 1 is the cheapest test. Path 3 is what we fall back to if
the blocker proves intractable.

## Verification pipeline (when unblocked)

The workflow at `.github/workflows/dvc-parity.yml` builds the
upstream DVC from source on a Windows runner, runs it against each
`testdata/golden/<nn>/doc.hwpx`, and diffs the output against the
committed `expected.json`. It is parameterized to run on
`workflow_dispatch` so we don't spend CI minutes on it routinely.
When we have a working Windows-side setup, a single manual trigger
produces the parity evidence report.

For developers who want to run the same loop locally, see
[`windows-parity-howto.md`](windows-parity-howto.md) (clean-room
procedure) and [`utm-windows-setup.md`](utm-windows-setup.md)
(Apple Silicon via UTM).

## Related reading

- [`cli-compat.md`](cli-compat.md) — flag-level compatibility with
  upstream `CommandParser.cpp`
- [`parity-roadmap.md`](parity-roadmap.md) — prioritized roadmap
  for closing the remaining feature-parity gaps in the engine
- [`dvc-parity-status.md`](dvc-parity-status.md) — build-attempt
  history of the upstream DVC on Windows
