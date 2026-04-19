//! Golden regression harness.
//!
//! Each case under `testdata/golden/<case>/` holds three files:
//!
//! - `doc.hwpx` — realistic HWPX produced from a Rust template. Committed
//!   as a binary so a future Windows contributor can feed it to DVC.exe
//!   for bit-exact parity verification.
//! - `spec.json` — the DVC rule file applied to the document.
//! - `expected.json` — the DVC-compatible JSON array the engine must emit
//!   (using `OutputOption::AllOption`).
//!
//! The default `golden_cases` test enforces the committed expectations.
//! Setting `POLARIS_REGEN_FIXTURES=1` before running tests rewrites every
//! `doc.hwpx` / `expected.json` from the in-Rust templates below — useful
//! when a template changes intentionally.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use polaris_rhwpdvc_core::engine::{validate, CheckProfile, EngineOptions};
use polaris_rhwpdvc_core::output::OutputOption;
use polaris_rhwpdvc_core::rules::schema::RuleSpec;
use serde_json::Value;

use support::{
    FixBorderFill, FixBullet, FixCharPr, FixFillBrush, FixNumbering, FixParaHead, FixParaPr,
    FixParagraph, FixRun, FixRunScope, FixTable, Fixture,
};

struct Case {
    name: &'static str,
    build: fn() -> Fixture,
    spec: &'static str,
    /// Defaults to `CheckProfile::Extended`. Set to `DvcStrict` on cases
    /// that specifically verify the strict filter drops over-implemented
    /// JIDs (margin-*, bgfill-*, etc.).
    profile: CheckProfile,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "01_clean",
            build: || Fixture::baseline(),
            spec: r#"{
  "charshape": { "font": ["바탕"], "fontsize": 10, "bold": false },
  "parashape": { "linespacingvalue": 160 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "02_fontsize_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].height = 1200;
                f
            },
            spec: r#"{
  "charshape": { "fontsize": 10 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "03_bold_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].bold = true;
                f
            },
            spec: r#"{
  "charshape": { "bold": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "04_font_allowlist_miss",
            build: || {
                let mut f = Fixture::baseline();
                f.hangul_face = "임의글꼴".into();
                f
            },
            spec: r#"{
  "charshape": { "font": ["바탕", "돋움", "굴림"] }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "05_font_allowlist_hit",
            build: || {
                let mut f = Fixture::baseline();
                f.hangul_face = "돋움".into();
                f
            },
            spec: r#"{
  "charshape": { "font": ["바탕", "돋움", "굴림"] }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "06_linespacing_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.para_prs[0].line_spacing_value = 180.0;
                f
            },
            spec: r#"{
  "parashape": { "linespacingvalue": 160 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "07_mixed_paragraphs",
            build: || {
                // Two paragraphs, charPr 0 is clean, charPr 1 is bold+large.
                Fixture {
                    hangul_face: "바탕".into(),
                    char_prs: vec![
                        FixCharPr {
                            id: 0,
                            height: 1000,
                            bold: false,
                            italic: false,
                            ..FixCharPr::default()
                        },
                        FixCharPr {
                            id: 1,
                            height: 1400,
                            bold: true,
                            italic: false,
                            ..FixCharPr::default()
                        },
                    ],
                    para_prs: vec![FixParaPr {
                        id: 0,
                        align: "JUSTIFY".into(),
                        line_spacing_value: 160.0,
                        ..FixParaPr::default()
                    }],
                    border_fills: vec![FixBorderFill::solid_default(1)],
                    numberings: Vec::new(),
                    bullets: Vec::new(),
                    paragraphs: vec![
                        FixParagraph {
                            para_pr_id_ref: 0,
                            style_id_ref: 0,
                            runs: vec![FixRun {
                                char_pr_id_ref: 0,
                                text: "ok".into(),
                                hyperlink: false,
                                scope: support::FixRunScope::None,
                            }],
                            table: None,
                        },
                        FixParagraph {
                            para_pr_id_ref: 0,
                            style_id_ref: 0,
                            runs: vec![FixRun {
                                char_pr_id_ref: 1,
                                text: "bad".into(),
                                hyperlink: false,
                                scope: support::FixRunScope::None,
                            }],
                            table: None,
                        },
                    ],
                    has_macro: false,
                }
            },
            spec: r#"{
  "charshape": { "fontsize": 10, "bold": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "08_style_forbidden",
            build: || {
                let mut f = Fixture::baseline();
                // Paragraph references a custom style — permission rule fires.
                f.paragraphs[0].style_id_ref = 7;
                f
            },
            spec: r#"{
  "style": { "permission": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "09_hyperlink_forbidden",
            build: || {
                let mut f = Fixture::baseline();
                f.paragraphs[0].runs[0].hyperlink = true;
                f
            },
            spec: r#"{
  "hyperlink": { "permission": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "10_macro_forbidden",
            build: || {
                let mut f = Fixture::baseline();
                f.has_macro = true;
                f
            },
            spec: r#"{
  "macro": { "permission": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "11_table_border_type_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                // borderFill id=2 has DASH top — spec requires SOLID (=1).
                let mut bf = FixBorderFill::solid_default(2);
                bf.top_kind = "DASH".into();
                f.border_fills.push(bf);
                f.paragraphs[0].table = Some(FixTable {
                    id: 100,
                    border_fill_id_ref: 2,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": {
    "border": [
      { "position": 1, "bordertype": 1 }
    ]
  }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "16_bullet_char_not_allowed",
            build: || {
                let mut f = Fixture::baseline();
                f.bullets.push(FixBullet {
                    id: 1,
                    char_: "★".into(),
                });
                f
            },
            // Allowed set excludes the star.
            spec: r#"{
  "bullet": { "bulletshapes": "□○-•*" }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "17_outlineshape_numtype_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.numberings.push(FixNumbering {
                    id: 3,
                    start: 1,
                    heads: vec![FixParaHead {
                        level: 1,
                        start: 1,
                        num_format: "^1)".into(), // spec wants "^1."
                        number_shape: 0,
                    }],
                });
                f
            },
            spec: r#"{
  "outlineshape": {
    "leveltype": [
      { "level": 1, "numbertype": "^1.", "numbershape": 0 }
    ]
  }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "18_paranumbullet_numshape_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.numberings.push(FixNumbering {
                    id: 4,
                    start: 1,
                    heads: vec![FixParaHead {
                        level: 2,
                        start: 1,
                        num_format: "^2.".into(),
                        number_shape: 3, // spec wants 8
                    }],
                });
                f
            },
            spec: r#"{
  "paranumbullet": {
    "leveltype": [
      { "level": 2, "numbertype": "^2.", "numbershape": 8 }
    ]
  }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "21_table_size_width_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                // Fixture default width is 42520. Spec requires exactly 30000.
                f.paragraphs[0].table = Some(FixTable {
                    id: 500,
                    border_fill_id_ref: 1,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "size-width": 30000 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "22_table_margin_range_ok",
            build: || {
                let mut f = Fixture::baseline();
                // Fixture inMargin is 141 on every side. Range includes it.
                f.paragraphs[0].table = Some(FixTable {
                    id: 501,
                    border_fill_id_ref: 1,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": {
    "margin-left":   { "min": 100, "max": 200 },
    "margin-right":  { "min": 100, "max": 200 },
    "margin-top":    { "min": 100, "max": 200 },
    "margin-bottom": { "min": 100, "max": 200 }
  }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "23_table_treat_as_char_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                // Fixture emits treatAsChar="0" (false). Spec demands true.
                f.paragraphs[0].table = Some(FixTable {
                    id: 502,
                    border_fill_id_ref: 1,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "treatAsChar": true }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "24_table_bgfill_type_mismatch",
            build: || {
                // Baseline borderFill has no <hh:fillBrush>, so Fill::None
                // (ordinal 0). Spec demands SOLID (1) → JID 3037 fires.
                let mut f = Fixture::baseline();
                f.paragraphs[0].table = Some(FixTable {
                    id: 600,
                    border_fill_id_ref: 1,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "bgfill-type": 1 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "25_table_bgfill_facecolor_mismatch",
            build: || {
                // borderFill id=3 has a winBrush with a specific faceColor.
                // Spec demands a different faceColor → JID 3038 fires.
                let mut f = Fixture::baseline();
                let mut bf = FixBorderFill::solid_default(3);
                bf.fill_brush = Some(FixFillBrush {
                    face_color: "#FF0000".into(),
                    hatch_color: "#000000".into(),
                    hatch_style: "NONE".into(),
                    alpha: 0,
                });
                f.border_fills.push(bf);
                f.paragraphs[0].table = Some(FixTable {
                    id: 601,
                    border_fill_id_ref: 3,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r##"{
  "table": { "bgfill-facecolor": "#FFFFFF" }
}
"##,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "19_fontsize_range_ok",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].height = 1100; // 11pt, within {min:10, max:12}
                f
            },
            spec: r#"{
  "charshape": { "fontsize": { "min": 10, "max": 12 } }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "20_fontsize_range_above_max",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].height = 1400; // 14pt, above max:12
                f
            },
            spec: r#"{
  "charshape": { "fontsize": { "min": 10, "max": 12 } }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "15_specialcharacter_below_minimum",
            build: || {
                let mut f = Fixture::baseline();
                // Replace the run text with one containing a TAB (U+0009),
                // which is below the spec minimum of U+0020.
                f.paragraphs[0].runs[0].text = "A\tB".into();
                f
            },
            spec: r#"{
  "specialcharacter": { "minimum": 32, "maximum": 1048575 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "13_charshape_ratio_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].ratio = 90.0; // spec wants 100
                f
            },
            spec: r#"{
  "charshape": { "ratio": 100 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "14_parashape_indent_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.para_prs[0].margin_intent = 500.0; // spec wants 0
                f
            },
            spec: r#"{
  "parashape": { "indent": 0 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            name: "12_lone_table_with_table_in_table_rule",
            build: || {
                // Regression anchor: a document with a single top-level
                // table should NOT fire `table-in-table:false` since there
                // is no nested table. Nested-table fixtures need richer
                // XML escape support — tracked separately.
                let mut f = Fixture::baseline();
                f.paragraphs[0].table = Some(FixTable {
                    id: 200,
                    border_fill_id_ref: 1,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "table-in-table": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // CharShape decoration parity: fixture emboss=true, spec
            // emboss=false → JID 1014 fires. Mirrors upstream's
            // `charshape->getEmboss() != charPr->charPrInfo.emboss`.
            name: "27_charshape_emboss_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].emboss = true;
                f
            },
            spec: r#"{
  "charshape": { "emboss": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Shadow decoration: fixture shadow=true (type="CONTINUOUS"),
            // spec shadow=false → JID 1016 fires.
            name: "28_charshape_shadow_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].shadow = true;
                f
            },
            spec: r#"{
  "charshape": { "shadow": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Shadow detail: fixture shadow=true with kind="CONTINUOUS"
            // (ord 2); spec demands shadowtype=1 (비연속) → 1019 fires.
            name: "29_charshape_shadowtype_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].shadow = true;
                f.char_prs[0].shadow_kind = "CONTINUOUS".into();
                f
            },
            spec: r#"{
  "charshape": { "shadowtype": "비연속" }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Shadow X offset: fixture offset_x=25; spec range {0..=20} → 1020.
            name: "30_charshape_shadow_x_outside_range",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].shadow = true;
                f.char_prs[0].shadow_offset_x = 25;
                f
            },
            spec: r#"{
  "charshape": { "shadow-x": { "min": 0, "max": 20 } }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Shadow color: fixture color=#FF0000; spec demands #000000 → 1022.
            name: "31_charshape_shadow_color_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].shadow = true;
                f.char_prs[0].shadow_color = "#FF0000".into();
                f
            },
            spec: r##"{
  "charshape": { "shadow-color": "#000000" }
}
"##,
            profile: CheckProfile::Extended,
        },
        Case {
            // Relative size: fixture rel_sz=120 vs spec r-size=100 → 1005.
            name: "32_charshape_rsize_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].rel_sz = 120.0;
                f
            },
            spec: r#"{
  "charshape": { "r-size": 100 }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Kerning: fixture useKerning=1 vs spec kerning=false → 1031.
            name: "33_charshape_kerning_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].kerning = true;
                f
            },
            spec: r#"{
  "charshape": { "kerning": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Table pos (textWrap) mismatch. Fixture emits default
            // TOP_AND_BOTTOM; spec demands SQUARE → JID 3005 fires.
            name: "34_table_pos_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.paragraphs[0].table = Some(FixTable {
                    id: 700,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "pos": "SQUARE" }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Table textpos (textFlow). Fixture default BOTH_SIDES,
            // spec LEFT_ONLY → JID 3006.
            name: "35_table_textpos_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.paragraphs[0].table = Some(FixTable {
                    id: 701,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "textpos": "LEFT_ONLY" }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Table fixed (lock). Fixture lock=false; spec demands true → 3003.
            name: "36_table_size_fixed_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.paragraphs[0].table = Some(FixTable {
                    id: 702,
                    lock: false,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "fixed": true }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // use_style propagation: a charshape violation on a run
            // that belongs to a paragraph with a non-zero styleIDRef
            // should carry `UseStyle: true` — regardless of whether
            // the spec has a `style.permission` rule. Mirrors
            // upstream OWPMLReader.cpp:304-305 (`isStyle = pPType->
            // GetStyleIDRef() != 0`) getting threaded through every
            // DVCErrorInfo.
            name: "41_charshape_violation_in_styled_paragraph",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].bold = true;
                f.paragraphs[0].style_id_ref = 5; // non-zero → isStyle
                f
            },
            spec: r#"{
  "charshape": { "bold": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Footnote scope tracking. Run inside <hp:footnote>; fixture
            // bold=true vs spec bold=false fires 1009. Run gets
            // `is_in_footnote=true` internally, but upstream has no JID
            // surfacing that flag, so expected.json carries the same
            // violation as a body-text run would — the check still fires
            // (footnote text is validated), which is the parity contract.
            name: "39_footnote_scope_bold_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].bold = true;
                f.paragraphs[0].runs[0].scope = FixRunScope::InFootnote;
                f
            },
            spec: r#"{
  "charshape": { "bold": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Endnote scope — same shape as the footnote case. Confirms
            // <hp:endnote> is also recognized by the parser's scope stack
            // and the run still participates in rule checking.
            name: "40_endnote_scope_bold_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].bold = true;
                f.paragraphs[0].runs[0].scope = FixRunScope::InEndnote;
                f
            },
            spec: r#"{
  "charshape": { "bold": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Shape scope tracking. Run is wrapped in <hp:shapeObject>
            // so the parser sets `Run.is_in_shape=true`, which the
            // engine propagates to `ViolationRecord.is_in_shape`.
            // The run also triggers a bold mismatch (fixture bold=true
            // vs spec bold=false) so we have a non-empty violation to
            // inspect. Expected.json should carry `IsInShape: true`.
            name: "38_shape_scope_bold_mismatch",
            build: || {
                let mut f = Fixture::baseline();
                f.char_prs[0].bold = true;
                f.paragraphs[0].runs[0].scope = FixRunScope::InShape;
                f
            },
            spec: r#"{
  "charshape": { "bold": false }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // ParaShape linespacing mode: fixture type=PERCENT, spec
            // demands "FIXED" → JID 2007 fires.
            name: "37_parashape_linespacing_mode_mismatch",
            build: || Fixture::baseline(),
            spec: r#"{
  "parashape": { "linespacing": "FIXED" }
}
"#,
            profile: CheckProfile::Extended,
        },
        Case {
            // Same fixture + spec as 24_table_bgfill_type_mismatch, but
            // the DvcStrict profile filters out JIDs upstream leaves as
            // no-op. bgfill-type is one of those, so expected.json is [].
            // This case pins the strict gate's behavior.
            name: "26_table_bgfill_type_strict_filters_out",
            build: || {
                let mut f = Fixture::baseline();
                f.paragraphs[0].table = Some(FixTable {
                    id: 600,
                    border_fill_id_ref: 1,
                    row_cnt: 1,
                    col_cnt: 1,
                    ..FixTable::default()
                });
                f
            },
            spec: r#"{
  "table": { "bgfill-type": 1 }
}
"#,
            profile: CheckProfile::DvcStrict,
        },
    ]
}

fn testdata_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/golden")
        .canonicalize()
        .expect("testdata/golden exists (see tests/support and testdata/golden/README.md)")
}

fn case_dir(name: &str) -> PathBuf {
    testdata_root().join(name)
}

fn regenerate() -> bool {
    std::env::var_os("POLARIS_REGEN_FIXTURES").is_some()
}

#[test]
fn golden_cases() {
    let root = testdata_root();
    for case in cases() {
        let dir = root.join(case.name);
        let doc_path = dir.join("doc.hwpx");
        let spec_path = dir.join("spec.json");
        let expected_path = dir.join("expected.json");
        let expected_xml_path = dir.join("expected.xml");

        let fixture = (case.build)();
        let hwpx_bytes = fixture.to_hwpx_bytes();
        let spec: RuleSpec = serde_json::from_str(case.spec).expect("case spec parses");

        let doc = polaris_rhwpdvc_hwpx::open_bytes(&hwpx_bytes).expect("fixture parses");
        let opts = EngineOptions {
            profile: case.profile,
            ..EngineOptions::default()
        };
        let report = validate(&doc, &spec, &opts);
        let actual = report.to_json_value(OutputOption::AllOption);
        let actual_xml = report.to_xml_string(OutputOption::AllOption);

        if regenerate() {
            fs::create_dir_all(&dir).unwrap();
            fs::write(&doc_path, &hwpx_bytes).unwrap();
            fs::write(&spec_path, case.spec).unwrap();
            let pretty = serde_json::to_string_pretty(&actual).unwrap() + "\n";
            fs::write(&expected_path, pretty).unwrap();
            fs::write(&expected_xml_path, &actual_xml).unwrap();
            continue;
        }

        let committed_doc = fs::read(&doc_path)
            .unwrap_or_else(|_| panic!("missing {}; run POLARIS_REGEN_FIXTURES=1", case.name));
        assert_eq!(
            committed_doc, hwpx_bytes,
            "doc.hwpx drift for {} — regenerate with POLARIS_REGEN_FIXTURES=1",
            case.name
        );

        let expected: Value = serde_json::from_slice(
            &fs::read(&expected_path)
                .unwrap_or_else(|_| panic!("missing expected for {}", case.name)),
        )
        .expect("expected.json parses");

        assert_eq!(
            actual, expected,
            "golden output mismatch for {} — regenerate with POLARIS_REGEN_FIXTURES=1",
            case.name
        );

        let committed_xml = fs::read_to_string(&expected_xml_path).unwrap_or_else(|_| {
            panic!(
                "missing expected.xml for {}; run POLARIS_REGEN_FIXTURES=1",
                case.name
            )
        });
        assert_eq!(
            actual_xml, committed_xml,
            "golden XML output mismatch for {} — regenerate with POLARIS_REGEN_FIXTURES=1",
            case.name
        );
    }

    if regenerate() {
        write_manifest(&root);
    } else {
        // Drift check: the committed manifest must match what we'd
        // regenerate right now. Catches "added a golden case but forgot
        // POLARIS_REGEN_FIXTURES" errors.
        let committed = fs::read_to_string(root.join("manifest.json"))
            .unwrap_or_else(|_| panic!("missing manifest.json — run POLARIS_REGEN_FIXTURES=1"));
        let expected = build_manifest();
        assert_eq!(
            committed, expected,
            "manifest.json drift — regenerate with POLARIS_REGEN_FIXTURES=1"
        );
    }
}

/// Build the golden manifest JSON the web UI and Pages deploy read.
/// Keep the output shape identical to what `.github/workflows/pages.yml`
/// generates (sorted case list, `{name,label}` entries) so the site
/// stays consistent whether local-dev loads it or Pages does.
fn build_manifest() -> String {
    let mut entries: Vec<(String, String)> = cases()
        .iter()
        .map(|c| (c.name.to_string(), humanize_case_name(c.name)))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut s = String::from("[\n");
    for (i, (name, label)) in entries.iter().enumerate() {
        s.push_str(&format!(
            "  {{\n    \"name\": \"{}\",\n    \"label\": \"{}\"\n  }}",
            name, label
        ));
        if i + 1 < entries.len() {
            s.push(',');
        }
        s.push('\n');
    }
    s.push_str("]\n");
    s
}

fn humanize_case_name(name: &str) -> String {
    // "24_table_bgfill_type_mismatch" -> "24 · table bgfill type mismatch"
    match name.split_once('_') {
        Some((num, rest)) if num.chars().all(|c| c.is_ascii_digit()) => {
            format!("{} · {}", num, rest.replace('_', " "))
        }
        _ => name.to_string(),
    }
}

fn write_manifest(root: &Path) {
    fs::write(root.join("manifest.json"), build_manifest()).unwrap();
}

/// Sanity: every golden case directory on disk must correspond to a
/// declared case. Prevents orphans after renames.
#[test]
fn no_orphan_case_directories() {
    let root = testdata_root();
    let declared: std::collections::HashSet<&str> = cases().iter().map(|c| c.name).collect();
    for entry in fs::read_dir(&root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        assert!(
            declared.contains(name.as_str()),
            "orphan golden case: {name} — delete the directory or add a Case entry"
        );
    }
    // Also verify each declared case has a populated directory (unless we're
    // about to regenerate).
    if !regenerate() {
        for c in cases() {
            let d = case_dir(c.name);
            assert!(d.exists(), "missing dir for {}", c.name);
            for f in ["doc.hwpx", "spec.json", "expected.json", "expected.xml"] {
                assert!(
                    d.join(f).exists(),
                    "missing {}/{} — run POLARIS_REGEN_FIXTURES=1",
                    c.name,
                    f
                );
            }
        }
    }
}
