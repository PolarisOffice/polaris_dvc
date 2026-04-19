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

use polaris_rhwpdvc_core::engine::{validate, EngineOptions};
use polaris_rhwpdvc_core::output::OutputOption;
use polaris_rhwpdvc_core::rules::schema::RuleSpec;
use serde_json::Value;

use support::{
    FixBorderFill, FixBullet, FixCharPr, FixFillBrush, FixNumbering, FixParaHead, FixParaPr,
    FixParagraph, FixRun, FixTable, Fixture,
};

struct Case {
    name: &'static str,
    build: fn() -> Fixture,
    spec: &'static str,
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
                });
                f
            },
            spec: r#"{
  "table": { "size-width": 30000 }
}
"#,
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
                });
                f
            },
            spec: r#"{
  "table": { "treatAsChar": true }
}
"#,
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
                });
                f
            },
            spec: r#"{
  "table": { "bgfill-type": 1 }
}
"#,
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
                });
                f
            },
            spec: r##"{
  "table": { "bgfill-facecolor": "#FFFFFF" }
}
"##,
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
                });
                f
            },
            spec: r#"{
  "table": { "table-in-table": false }
}
"#,
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

        let fixture = (case.build)();
        let hwpx_bytes = fixture.to_hwpx_bytes();
        let spec: RuleSpec = serde_json::from_str(case.spec).expect("case spec parses");

        let doc = polaris_rhwpdvc_hwpx::open_bytes(&hwpx_bytes).expect("fixture parses");
        let report = validate(&doc, &spec, &EngineOptions::default());
        let actual = report.to_json_value(OutputOption::AllOption);

        if regenerate() {
            fs::create_dir_all(&dir).unwrap();
            fs::write(&doc_path, &hwpx_bytes).unwrap();
            fs::write(&spec_path, case.spec).unwrap();
            let pretty = serde_json::to_string_pretty(&actual).unwrap() + "\n";
            fs::write(&expected_path, pretty).unwrap();
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
    }
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
            for f in ["doc.hwpx", "spec.json", "expected.json"] {
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
