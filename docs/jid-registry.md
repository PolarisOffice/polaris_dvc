# JID registry

## What it is

`crates/polaris-dvc-core/src/jid_registry.rs` holds a `pub const JID_*:
ErrorCode` declaration for every `#define JID_*` in upstream
`third_party/dvc-upstream/Source/JsonModel.h` — 218 entries at the time
this was written.

The curated `jid` submodule in `crates/polaris-dvc-core/src/error_codes.rs`
is a short-name alias layer over that registry (e.g., `jid::CHAR_SHAPE_
FONTSIZE` → `jid_registry::JID_CHAR_SHAPE_FONTSIZE`). Values live in the
registry so they can never drift from upstream.

## Regenerating

After the vendored snapshot in `third_party/dvc-upstream/` is refreshed:

```sh
cargo run --manifest-path tools/gen-jids/Cargo.toml
```

The tool rewrites `jid_registry.rs` in full. A safety net in
`crates/polaris-dvc-core/tests/jid_registry_drift.rs` runs on every
`cargo test` and fails if the committed registry's numeric values don't
match what `JsonModel.h` implies. If you're mid-edit and need to bypass
that check temporarily, set `POLARIS_ALLOW_JID_DRIFT=1`.

## How to wire a new JID into the engine

1. Find the upstream constant in the generated file, e.g.
   `JID_TABLE_SIZEWIDTH` (`crates/polaris-dvc-core/src/jid_registry.rs`).
2. Add a short-name alias in the `jid` module of
   `crates/polaris-dvc-core/src/error_codes.rs`:
   ```rust
   pub const TABLE_SIZE_WIDTH: ErrorCode = r::JID_TABLE_SIZEWIDTH;
   ```
3. Add an `ErrorCode::text(self)` arm in the same file if you want a
   human-readable message for the code.
4. Reference the alias from the engine's checker (e.g.,
   `crates/polaris-dvc-core/src/engine.rs::check_table`).
5. If the value appears in a committed golden case, regenerate with
   `POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden`.

## Naming convention

- The generated file keeps upstream names verbatim with the `JID_`
  prefix (`JID_PARA_SHAPE_HORIZONTAL`).
- The curated alias layer uses shorter, idiomatic names sometimes
  renamed for clarity (`PARA_SHAPE_ALIGN` rather than `HORIZONTAL`).
  Any rename is called out with a one-line comment in
  `error_codes.rs::jid`.

## Coverage status

The full 218-entry registry is available as Rust constants. The engine
currently references ~30 of them through the curated aliases; the rest
are declared so downstream work (additional table fields, border-fill
rules on CharPr, outline-shape specifics, etc.) can reference them
without guessing numeric values.
