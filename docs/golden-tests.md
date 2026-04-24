# Golden regression tests

The `testdata/golden/` tree is the engine's DVC-parity anchor. Each case
directory contains three committed files:

| file | role |
|---|---|
| `doc.hwpx` | realistic HWPX document, produced from a Rust template |
| `spec.json` | the DVC rule file applied during the run |
| `expected.json` | the DVC-shaped JSON array the engine must emit under `OutputOption::AllOption` |

`crates/polaris-dvc-core/tests/golden.rs` iterates every case on every test
run. A case fails if either the freshly-built `doc.hwpx` bytes drift from
the committed file, or if the engine output diverges from
`expected.json`.

## Running

```sh
# Regular check — part of `cargo test`:
cargo test -p polaris-dvc-core --test golden
```

## Adding or updating a case

1. Edit `crates/polaris-dvc-core/tests/support/mod.rs` if the template needs
   new knobs (e.g., a new CharPr field).
2. Edit `crates/polaris-dvc-core/tests/golden.rs` — add a `Case { name,
   build, spec }` entry or change an existing one. Use a snake-case
   `<nn>_<description>` name.
3. Create the directory: `mkdir testdata/golden/<nn>_<description>`.
4. Regenerate:
   ```sh
   POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden
   ```
   This rewrites `doc.hwpx`, `spec.json`, and `expected.json` for every
   case. Review the diff before committing.
5. Commit the updated test code and the three files for each affected
   case.

## Parity verification against upstream DVC.exe

Because DVC.exe is Windows-only, CI runs only the Rust side. To verify
that polaris matches DVC bit-for-bit on a given fixture, use a Windows
(or Wine) environment:

```cmd
ExampleWindows.exe -j --file=actual.json -t testdata\golden\02_fontsize_mismatch\spec.json testdata\golden\02_fontsize_mismatch\doc.hwpx
diff actual.json testdata\golden\02_fontsize_mismatch\expected.json
```

If the diff is non-empty, file an issue with both outputs — our current
`expected.json` is the engine's own output and may not yet exactly
match DVC.exe for every conditional-field case.
