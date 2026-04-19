//! Validation engine: walks an HWPX document, applies rule checkers, and
//! emits DVC-compatible violation records.
//!
//! Scope for the initial engine:
//! - CharShape checks: font (Hangul face), fontsize (height, 1/100 pt),
//!   bold, italic.
//! - ParaShape checks: line spacing value.
//!
//! Further JID categories land in follow-up commits, each with its own
//! checker module. Page/line tracking is stubbed to 1/1 until the HWPX
//! parser exposes `<hp:linesegarray>` data.

// Each check is written as `if mismatched { if !ctx.push(...) { return false; } }`
// because the inner `push` can also tell us to stop early. Collapsing these
// into single `&&` expressions hurts readability when the push call spans
// several lines, so we silence the lint locally.
#![allow(clippy::collapsible_if)]

use crate::error_codes::{jid, ErrorCode};
use crate::output::ViolationRecord;
use crate::report::Report;
use crate::rules::schema::{CharShape, ParaShape, RuleSpec};

use polaris_hwpx::{CharPr, HwpxDocument, ParaPr, Paragraph, Run};

#[derive(Default)]
pub struct EngineOptions {
    pub stop_on_first: bool,
}

struct Ctx<'a> {
    doc: &'a HwpxDocument,
    opts: &'a EngineOptions,
    report: Report,
}

impl<'a> Ctx<'a> {
    fn push(&mut self, v: ViolationRecord) -> bool {
        self.report.push(v);
        if self.opts.stop_on_first {
            self.report.stopped_early = true;
            return false;
        }
        true
    }
}

pub fn validate(doc: &HwpxDocument, spec: &RuleSpec, opts: &EngineOptions) -> Report {
    let mut ctx = Ctx {
        doc,
        opts,
        report: Report::empty(),
    };

    'sections: for section in &doc.sections {
        for paragraph in &section.paragraphs {
            if !check_paragraph(&mut ctx, paragraph, spec) {
                break 'sections;
            }
        }
    }

    ctx.report
}

fn check_paragraph(ctx: &mut Ctx, paragraph: &Paragraph, spec: &RuleSpec) -> bool {
    let para_pr = ctx.doc.header.para_shape(paragraph.para_pr_id_ref);

    if let (Some(para_spec), Some(para_pr)) = (spec.parashape.as_ref(), para_pr) {
        if !check_para_shape(ctx, paragraph, para_pr, para_spec) {
            return false;
        }
    }

    for run in &paragraph.runs {
        if let Some(char_spec) = spec.charshape.as_ref() {
            if let Some(char_pr) = ctx.doc.header.char_shape(run.char_pr_id_ref) {
                if !check_char_shape(ctx, paragraph, run, char_pr, char_spec) {
                    return false;
                }
            }
        }
    }
    true
}

fn check_char_shape(
    ctx: &mut Ctx,
    paragraph: &Paragraph,
    run: &Run,
    char_pr: &CharPr,
    spec: &CharShape,
) -> bool {
    // Font check — compare against the Hangul face registered for this CharPr.
    if let Some(expected) = spec.font.as_deref() {
        let actual = ctx
            .doc
            .header
            .face_name(char_pr.font_ref.hangul, "HANGUL")
            .map(|f| f.face.as_str());
        if actual != Some(expected) {
            if !ctx.push(violation_for(
                paragraph,
                run,
                jid::CHAR_SHAPE_FONT,
                format!(
                    "expected font '{}', got '{}'",
                    expected,
                    actual.unwrap_or("<unknown>")
                ),
            )) {
                return false;
            }
        }
    }

    // Fontsize: spec is in points (e.g., 10). CharPr.height is 1/100 pt.
    if let Some(expected_pt) = spec.fontsize {
        let expected_height = (expected_pt * 100.0).round() as u32;
        if char_pr.height != expected_height {
            if !ctx.push(violation_for(
                paragraph,
                run,
                jid::CHAR_SHAPE_FONTSIZE,
                format!(
                    "expected {} pt ({}), got {}",
                    expected_pt, expected_height, char_pr.height
                ),
            )) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.bold {
        if char_pr.bold != expected {
            if !ctx.push(violation_for(
                paragraph,
                run,
                jid::CHAR_SHAPE_BOLD,
                format!("expected bold={}, got {}", expected, char_pr.bold),
            )) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.italic {
        if char_pr.italic != expected {
            if !ctx.push(violation_for(
                paragraph,
                run,
                jid::CHAR_SHAPE_ITALIC,
                format!("expected italic={}, got {}", expected, char_pr.italic),
            )) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.underline {
        let actual = char_pr.underline.is_some();
        if actual != expected {
            if !ctx.push(violation_for(
                paragraph,
                run,
                jid::CHAR_SHAPE_UNDERLINE,
                format!("expected underline={}, got {}", expected, actual),
            )) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.strikeout {
        let actual = char_pr.strikeout.is_some();
        if actual != expected {
            if !ctx.push(violation_for(
                paragraph,
                run,
                jid::CHAR_SHAPE_STRIKEOUT,
                format!("expected strikeout={}, got {}", expected, actual),
            )) {
                return false;
            }
        }
    }

    true
}

fn check_para_shape(
    ctx: &mut Ctx,
    paragraph: &Paragraph,
    para_pr: &ParaPr,
    spec: &ParaShape,
) -> bool {
    if let Some(expected) = spec.linespacingvalue {
        if (para_pr.line_spacing_value - expected).abs() > f64::EPSILON {
            let v = ViolationRecord {
                para_pr_id_ref: paragraph.para_pr_id_ref,
                page_no: 1,
                line_no: 1,
                error_code: jid::PARA_SHAPE_LINESPACING,
                error_string: format!(
                    "expected line spacing {}, got {}",
                    expected, para_pr.line_spacing_value
                ),
                ..ViolationRecord::new(jid::PARA_SHAPE_LINESPACING)
            };
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.align.as_deref() {
        if !para_pr.align_horizontal.eq_ignore_ascii_case(expected) {
            let v = ViolationRecord {
                para_pr_id_ref: paragraph.para_pr_id_ref,
                page_no: 1,
                line_no: 1,
                error_code: jid::PARA_SHAPE_ALIGN,
                error_string: format!(
                    "expected align {}, got {}",
                    expected, para_pr.align_horizontal
                ),
                ..ViolationRecord::new(jid::PARA_SHAPE_ALIGN)
            };
            if !ctx.push(v) {
                return false;
            }
        }
    }

    true
}

fn violation_for(
    paragraph: &Paragraph,
    run: &Run,
    code: ErrorCode,
    diagnostic: String,
) -> ViolationRecord {
    ViolationRecord {
        char_pr_id_ref: run.char_pr_id_ref,
        para_pr_id_ref: paragraph.para_pr_id_ref,
        text: run.text.clone(),
        page_no: 1,
        line_no: 1,
        error_code: code,
        error_string: diagnostic,
        ..ViolationRecord::new(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polaris_hwpx::{CharPr, FaceName, FontRef, Header, HwpxDocument, ParaPr};

    fn make_doc(char_pr: CharPr, para_pr: ParaPr, text: &str) -> HwpxDocument {
        let mut header = Header::default();
        header.face_names.push(FaceName {
            id: 0,
            lang: "HANGUL".into(),
            face: "바탕".into(),
        });
        header.char_shapes.push(char_pr);
        header.para_shapes.push(para_pr);
        let run = polaris_hwpx::Run {
            char_pr_id_ref: 0,
            text: text.into(),
        };
        let paragraph = polaris_hwpx::Paragraph {
            id: 0,
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![run],
        };
        let section = polaris_hwpx::Section {
            paragraphs: vec![paragraph],
        };
        HwpxDocument {
            mimetype: "application/hwp+zip".into(),
            header,
            sections: vec![section],
        }
    }

    fn bata_char_pr(height: u32) -> CharPr {
        CharPr {
            id: 0,
            height,
            font_ref: FontRef::default(),
            ..CharPr::default()
        }
    }

    #[test]
    fn clean_document_yields_no_violations() {
        let doc = make_doc(
            bata_char_pr(1000),
            ParaPr {
                id: 0,
                line_spacing_value: 160.0,
                ..ParaPr::default()
            },
            "hello",
        );
        let spec: RuleSpec = serde_json::from_str(
            r#"{"charshape":{"font":"바탕","fontsize":10,"bold":false},
                 "parashape":{"linespacingvalue":160}}"#,
        )
        .unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert!(r.violations.is_empty(), "violations: {:?}", r.violations);
    }

    #[test]
    fn wrong_fontsize_emits_code_1001() {
        let doc = make_doc(bata_char_pr(1200), ParaPr::default(), "hello");
        let spec: RuleSpec = serde_json::from_str(r#"{"charshape":{"fontsize":10}}"#).unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].error_code.value(), 1001);
        assert_eq!(r.violations[0].text, "hello");
    }

    #[test]
    fn wrong_font_emits_code_1004() {
        let mut cp = bata_char_pr(1000);
        cp.font_ref.hangul = 0;
        // Face at id 0 is "바탕" per helper; spec wants "다른글꼴" → violation.
        let doc = make_doc(cp, ParaPr::default(), "hello");
        let spec: RuleSpec = serde_json::from_str(r#"{"charshape":{"font":"다른글꼴"}}"#).unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].error_code.value(), 1004);
    }

    #[test]
    fn bold_mismatch_emits_1009() {
        let mut cp = bata_char_pr(1000);
        cp.bold = true;
        let doc = make_doc(cp, ParaPr::default(), "hello");
        let spec: RuleSpec = serde_json::from_str(r#"{"charshape":{"bold":false}}"#).unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].error_code.value(), 1009);
    }

    #[test]
    fn linespacing_mismatch_emits_2050() {
        let doc = make_doc(
            bata_char_pr(1000),
            ParaPr {
                id: 0,
                line_spacing_value: 180.0,
                ..ParaPr::default()
            },
            "hello",
        );
        let spec: RuleSpec =
            serde_json::from_str(r#"{"parashape":{"linespacingvalue":160}}"#).unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].error_code.value(), 2050);
    }

    #[test]
    fn stop_on_first_halts_after_single_violation() {
        let mut cp = bata_char_pr(1200); // wrong size AND bold
        cp.bold = true;
        let doc = make_doc(cp, ParaPr::default(), "hello");
        let spec: RuleSpec =
            serde_json::from_str(r#"{"charshape":{"fontsize":10,"bold":false}}"#).unwrap();
        let r = validate(
            &doc,
            &spec,
            &EngineOptions {
                stop_on_first: true,
            },
        );
        assert_eq!(r.violations.len(), 1);
        assert!(r.stopped_early);
    }
}
