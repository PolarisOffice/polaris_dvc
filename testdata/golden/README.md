# testdata/golden

Committed fixtures for the DVC-parity regression harness. See
[docs/golden-tests.md](../../docs/golden-tests.md) for how to add,
update, or verify cases.

**Do not edit these files by hand.** Regenerate via:

```sh
POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-core --test golden
```

Each case directory has:

- `doc.hwpx` — the HWPX document under test (binary; unzip to inspect)
- `spec.json` — the rule spec applied
- `expected.json` — the DVC-shaped JSON array the engine should emit

Current cases:

| case | what it exercises |
|---|---|
| `01_clean` | all rules pass → empty array |
| `02_fontsize_mismatch` | CharPr.height 1200 vs spec 10pt → `ErrorCode: 1001` |
| `03_bold_mismatch` | document has bold vs spec `bold:false` → `ErrorCode: 1009` |
| `04_font_allowlist_miss` | Hangul face outside spec allowlist → `ErrorCode: 1004` |
| `05_font_allowlist_hit` | Hangul face inside allowlist → empty array |
| `06_linespacing_mismatch` | para 180 vs spec 160 → `ErrorCode: 2050` |
| `07_mixed_paragraphs` | only the dirty paragraph reports (LineNo 2); clean one skipped |
| `08_style_forbidden` | paragraph has styleIDRef=7 vs spec `style.permission:false` → `ErrorCode: 3502` (`UseStyle: true`) |
| `09_hyperlink_forbidden` | run wrapped in `<hp:fieldBegin type="HYPERLINK">` vs spec `hyperlink.permission:false` → `ErrorCode: 6901` (`UseHyperlink: true`) |
| `10_macro_forbidden` | manifest has `Scripts/macros.js` vs spec `macro.permission:false` → `ErrorCode: 7001` (document-level, empty errorText) |
