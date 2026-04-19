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
| `11_table_border_type_mismatch` | table borderFill has DASH top vs spec `{position:1, bordertype:1}` (SOLID) → `ErrorCode: 3033` (`IsInTable: true`) |
| `12_lone_table_with_table_in_table_rule` | single top-level table + spec `table-in-table:false` → empty array (regression anchor) |
| `13_charshape_ratio_mismatch` | charPr ratio 90 vs spec `ratio:100` → `ErrorCode: 1007` |
| `14_parashape_indent_mismatch` | paraPr margin-intent 500 vs spec `indent:0` → `ErrorCode: 2005` |
| `15_specialcharacter_below_minimum` | run text contains TAB (U+0009) vs spec `minimum:32` → `ErrorCode: 3101` |
| `16_bullet_char_not_allowed` | header bullet char `★` vs spec `bulletshapes:"□○-•*"` → `ErrorCode: 3304` |
| `17_outlineshape_numtype_mismatch` | numbering level 1 numFormat `"^1)"` vs spec `numbertype:"^1."` → `ErrorCode: 3206` |
| `18_paranumbullet_numshape_mismatch` | numbering level 2 numberShape 3 vs spec `numbershape:8` → `ErrorCode: 3407` |
