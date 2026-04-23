# Contributing to polaris_dvc

Thanks for considering a contribution. This project re-implements
[hancom-io/dvc](https://github.com/hancom-io/dvc) in pure Rust, with
byte-level output compatibility goals. Because the upstream reference
is authoritative for behavior, the contribution flow is more
structured than a typical Rust port — read this file before opening
a PR.

For internal AI-agent guidance, see `CLAUDE.md`. This document is
the human-contributor counterpart.

## Before you start

- **Issue first for anything non-trivial.** Bug reports, feature
  proposals, and "I found a parity gap" discoveries should open an
  issue before code lands. This lets us flag whether the behavior
  you're about to change is a deliberate upstream-compatibility
  decision.
- **Do not edit `third_party/dvc-upstream/`.** It's a read-only
  snapshot retained for reference and license compliance. See
  `third_party/dvc-upstream/PROVENANCE.md`.
- **Upstream is authoritative for behavior.** If our engine diverges
  from `Source/Checker.cpp`, the engine is wrong — even if our
  behavior "looks cleaner". Parity is the point.

## Development setup

Requires Rust 1.82 or newer (pinned in `rust-toolchain.toml`).

```sh
git clone https://github.com/PolarisOffice/polaris_dvc
cd polaris_dvc
cargo build --workspace
cargo test  --workspace --exclude polaris-dvc-wasm
```

For the WASM target:

```sh
# Install wasm-pack once:
cargo install wasm-pack

wasm-pack build crates/polaris-dvc-wasm --target web
```

## Checks your PR must pass

CI runs the following on Ubuntu + macOS; run them locally before
pushing to avoid round-trips:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --exclude polaris-dvc-wasm --all-targets -- -D warnings
cargo test  --workspace --exclude polaris-dvc-wasm
wasm-pack build crates/polaris-dvc-wasm --target web
```

The clippy run is strict (`-D warnings`). If you need to suppress a
lint, add a narrowly-scoped `#[allow(...)]` with a one-line comment
explaining why.

## Golden tests

The `testdata/golden/<nn>_...` directories are the authoritative
regression corpus. Each case is a triple `(doc.hwpx, spec.json,
expected.json)` (plus `expected.xml` as of Phase 8). The
`doc.hwpx` bytes are *reproducible from an in-Rust fixture template*
at `crates/polaris-dvc-core/tests/support/mod.rs` — `cargo test`
rebuilds the zip and asserts it matches the committed bytes exactly.

If you change the fixture template or an engine checker, regenerate:

```sh
POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden
```

**Review the diff before committing.** A regenerated `expected.json`
is how we catch silent behavior regressions — if you can't explain
each line change, the engine change is probably wrong.

Adding a new golden case: see `docs/golden-tests.md` for the full
walkthrough. Short version:

1. Add a `Case { name: "NN_descr", build, spec }` entry to
   `crates/polaris-dvc-core/tests/golden.rs`.
2. `mkdir testdata/golden/NN_descr`.
3. `POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden`.
4. Commit the four generated files plus your source changes.

## Adding a new JID (error code)

Follow the template in `CLAUDE.md` §"How to wire a new JID into the
engine". The short version:

1. Find the constant in `crates/polaris-dvc-core/src/jid_registry.rs`
   (generated from upstream; do not hand-edit).
2. Add a short-name alias in the `jid` module of `src/error_codes.rs`.
3. Add an arm to `ErrorCode::text(self)` with a human-readable message
   (match upstream wording when applicable).
4. Extend `rules/schema.rs` if the rule introduces new JSON fields.
5. Add a `check_<thing>()` function in `engine.rs`. Always go through
   `ctx.push(v)`; never push directly onto `ctx.records`.
6. Add a golden case exercising the new code.

## Commit & PR conventions

- **Commit messages**: imperative mood, subject under 72 chars,
  explain *why* in the body. See recent `git log` for style.
- **One concern per PR.** Refactors and functional changes don't mix
  — reviewers can't tell which byte-diff came from which intent.
- **Never amend or force-push** once a PR is open. Stack new commits;
  the repo squash-merges on the GitHub side. This matches the
  repository-wide rule in `CLAUDE.md`.
- **Sign off acceptable; not required.** Apache-2.0 project — no CLA.

## Parity with upstream DVC.exe

We target byte-exact output parity with upstream `DVC.exe` under
the `--dvc-strict` profile. The `docs/parity-roadmap.md` tracks what
JIDs are covered. The `docs/windows-parity-howto.md` walks a Windows
contributor through producing fresh `expected.json` bytes from the
real upstream binary. You do not need Windows to contribute — the
in-repo golden tests run on Linux/macOS — but if you do have Windows
access and can help close parity gaps, see that doc.

## Code of conduct

We follow the
[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).
Report violations via the channel in `SECURITY.md` — reports are
read privately.

## License

By contributing, you agree that your contributions will be licensed
under the Apache License 2.0 (matching this project and upstream
hancom-io/dvc). See `LICENSE` and `NOTICE`.
