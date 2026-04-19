//! Validation engine: walks an HWPX document, applies rule checkers, and
//! emits DVC-compatible violation records.
//!
//! Scope for the initial engine:
//! - CharShape checks: font (Hangul face), fontsize (height, 1/100 pt),
//!   bold, italic, underline, strikeout.
//! - ParaShape checks: align, line spacing value.
//!
//! Page/line tracking is a port of upstream `OWPMLReader::FindPageInfo`:
//! page breaks open when a section-level paragraph's first lineseg has
//! `vert_pos == 0` or wraps back above the previous paragraph's tail.
//! `line_no` accumulates by the paragraph's lineseg count after each
//! paragraph.
//!
//! Further JID categories (table borders, permissions, shapes, hyperlinks,
//! macros) land in follow-up commits, each with its own checker module.

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
    /// Current page number (0 before the first paragraph; increments to 1
    /// when the first paragraph's first lineseg fires the opening page
    /// break, matching upstream `FindPageInfo`).
    page_no: u32,
    /// Current line number within the current page (1-based).
    line_no: u32,
    before_vert_pos: i64,
    before_vert_size: i64,
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
        page_no: 0,
        line_no: 1,
        before_vert_pos: 0,
        before_vert_size: 0,
    };

    // Document-scope checks run before the section walk so a forbidden
    // macro reports before any per-run violation. These fire at most
    // once per document.
    if let Some(m) = spec.macro_.as_ref() {
        if m.permission == Some(false) && doc.header.has_macro {
            let v = ViolationRecord {
                page_no: 1,
                line_no: 1,
                error_code: jid::MACRO_PERMISSION,
                error_string: "macro asset found in manifest".to_string(),
                use_style: false,
                use_hyperlink: false,
                ..ViolationRecord::new(jid::MACRO_PERMISSION)
            };
            if !ctx.push(v) {
                return ctx.report;
            }
        }
    }

    'sections: for section in &doc.sections {
        // Per upstream `GetPageInfo`, vertical-position trackers reset per
        // section. Page counter is cumulative across sections.
        ctx.before_vert_pos = 0;
        ctx.before_vert_size = 0;
        for paragraph in &section.paragraphs {
            // Opening page break for a paragraph that sits directly under the
            // section: either the first lineseg has vert_pos == 0, or its
            // vert_pos wrapped back above the previous paragraph's tail
            // (a new column/page flow). Tables / shapes are out of scope
            // today, so every paragraph we see is section-level.
            let first_vp = paragraph.line_segs.first().map(|s| s.vert_pos).unwrap_or(0);
            let page_break_open =
                first_vp == 0 || first_vp < ctx.before_vert_pos + ctx.before_vert_size;
            if page_break_open {
                ctx.page_no = ctx.page_no.saturating_add(1);
                ctx.line_no = 1;
            }

            if !check_paragraph(&mut ctx, paragraph, spec) {
                break 'sections;
            }

            let n_seg = paragraph.line_segs.len() as u32;
            // Advance line counter by this paragraph's line count. Upstream
            // gates this on "not in table and not in object"; once we model
            // those categories we'll guard it similarly.
            if n_seg > 0 {
                ctx.line_no = ctx.line_no.saturating_add(n_seg);
                let last = paragraph.line_segs.last().unwrap();
                ctx.before_vert_pos = last.vert_pos;
                ctx.before_vert_size = last.vert_size;
            }

            // Intra-paragraph page break: if any non-first lineseg sits at
            // vert_pos == 0, the paragraph itself crossed a page boundary.
            if paragraph.line_segs.len() > 1 {
                for (idx, seg) in paragraph.line_segs.iter().enumerate().skip(1).rev() {
                    if seg.vert_pos == 0 {
                        ctx.page_no = ctx.page_no.saturating_add(1);
                        ctx.line_no = (paragraph.line_segs.len() - idx + 1) as u32;
                        break;
                    }
                }
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

    // Style-permission: upstream DVCErrorInfo marks runs with `use_style`
    // when the paragraph references a non-zero style id. When the spec
    // forbids style usage, that's a violation at the run level.
    let style_forbidden = spec
        .style
        .as_ref()
        .and_then(|p| p.permission)
        .map(|allowed| !allowed)
        .unwrap_or(false);
    let hyperlink_forbidden = spec
        .hyperlink
        .as_ref()
        .and_then(|p| p.permission)
        .map(|allowed| !allowed)
        .unwrap_or(false);

    for run in &paragraph.runs {
        if let Some(char_spec) = spec.charshape.as_ref() {
            if let Some(char_pr) = ctx.doc.header.char_shape(run.char_pr_id_ref) {
                if !check_char_shape(ctx, paragraph, run, char_pr, char_spec) {
                    return false;
                }
            }
        }

        if style_forbidden && paragraph.style_id_ref != 0 {
            let mut v = violation_for(
                ctx,
                paragraph,
                run,
                jid::STYLE_PERMISSION,
                format!("style id {} used but not permitted", paragraph.style_id_ref),
            );
            v.use_style = true;
            if !ctx.push(v) {
                return false;
            }
        }

        if hyperlink_forbidden && run.is_hyperlink {
            let mut v = violation_for(
                ctx,
                paragraph,
                run,
                jid::HYPERLINK_PERMISSION,
                "hyperlink run found but not permitted".to_string(),
            );
            v.use_hyperlink = true;
            if !ctx.push(v) {
                return false;
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
    // Font check — the Hangul face registered for this CharPr must appear
    // in the allowlist. Upstream `charshape.font` is an array of allowed
    // family names; a document value outside the list is a violation.
    if let Some(allow) = spec.font.as_ref() {
        if !allow.is_empty() {
            let actual = ctx
                .doc
                .header
                .face_name(char_pr.font_ref.hangul, "HANGUL")
                .map(|f| f.face.as_str());
            let ok = actual.map(|a| allow.contains(a)).unwrap_or(false);
            if !ok {
                let v = violation_for(
                    ctx,
                    paragraph,
                    run,
                    jid::CHAR_SHAPE_FONT,
                    format!(
                        "font '{}' not in allowlist {:?}",
                        actual.unwrap_or("<unknown>"),
                        allow.0
                    ),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
    }

    // Fontsize: spec is in points (e.g., 10). CharPr.height is 1/100 pt.
    if let Some(expected_pt) = spec.fontsize {
        let expected_height = (expected_pt * 100.0).round() as u32;
        if char_pr.height != expected_height {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_FONTSIZE,
                format!(
                    "expected {} pt ({}), got {}",
                    expected_pt, expected_height, char_pr.height
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.bold {
        if char_pr.bold != expected {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_BOLD,
                format!("expected bold={}, got {}", expected, char_pr.bold),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.italic {
        if char_pr.italic != expected {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_ITALIC,
                format!("expected italic={}, got {}", expected, char_pr.italic),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.underline {
        let actual = char_pr.underline.is_some();
        if actual != expected {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_UNDERLINE,
                format!("expected underline={}, got {}", expected, actual),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.strikeout {
        let actual = char_pr.strikeout.is_some();
        if actual != expected {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_STRIKEOUT,
                format!("expected strikeout={}, got {}", expected, actual),
            );
            if !ctx.push(v) {
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
            let page_no = ctx.page_no;
            let line_no = ctx.line_no;
            let v = ViolationRecord {
                para_pr_id_ref: paragraph.para_pr_id_ref,
                page_no,
                line_no,
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
            let page_no = ctx.page_no;
            let line_no = ctx.line_no;
            let v = ViolationRecord {
                para_pr_id_ref: paragraph.para_pr_id_ref,
                page_no,
                line_no,
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
    ctx: &Ctx,
    paragraph: &Paragraph,
    run: &Run,
    code: ErrorCode,
    diagnostic: String,
) -> ViolationRecord {
    ViolationRecord {
        char_pr_id_ref: run.char_pr_id_ref,
        para_pr_id_ref: paragraph.para_pr_id_ref,
        text: run.text.clone(),
        page_no: ctx.page_no,
        line_no: ctx.line_no,
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
            is_hyperlink: false,
        };
        let paragraph = polaris_hwpx::Paragraph {
            id: 0,
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![run],
            line_segs: Vec::new(),
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
    fn page_line_accumulates_across_paragraphs() {
        use polaris_hwpx::{LineSeg, Paragraph, Run, Section};

        let mut header = Header::default();
        header.face_names.push(FaceName {
            id: 0,
            lang: "HANGUL".into(),
            face: "바탕".into(),
        });
        header.char_shapes.push(CharPr {
            id: 0,
            height: 1000,
            font_ref: FontRef::default(),
            ..CharPr::default()
        });
        header.char_shapes.push(CharPr {
            id: 1,
            height: 1200,
            font_ref: FontRef::default(),
            ..CharPr::default()
        });
        header.para_shapes.push(ParaPr {
            id: 0,
            ..ParaPr::default()
        });

        // Three paragraphs stacking downward at vert_pos 0, 1600, 3200.
        let make_para = |char_ref: u32, text: &str, vp: i64| Paragraph {
            id: 0,
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![Run {
                char_pr_id_ref: char_ref,
                text: text.into(),
                is_hyperlink: false,
            }],
            line_segs: vec![LineSeg {
                vert_pos: vp,
                vert_size: 1000,
                ..LineSeg::default()
            }],
        };

        let doc = HwpxDocument {
            mimetype: "application/hwp+zip".into(),
            header,
            sections: vec![Section {
                paragraphs: vec![
                    make_para(0, "clean1", 0),
                    make_para(1, "bad", 1600),
                    make_para(0, "clean2", 3200),
                ],
            }],
        };

        let spec: RuleSpec = serde_json::from_str(r#"{"charshape":{"fontsize":10}}"#).unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 1);
        // First paragraph's opening lineseg (vert_pos=0) opens page 1 with
        // line 1. After it, line_no advances to 2. The second paragraph's
        // vert_pos=1600 >= 0 + 1000 so no new page — runs get PageNo=1,
        // LineNo=2.
        assert_eq!(r.violations[0].page_no, 1);
        assert_eq!(r.violations[0].line_no, 2);
    }

    #[test]
    fn page_break_when_vertpos_resets_to_zero() {
        use polaris_hwpx::{LineSeg, Paragraph, Run, Section};

        let mut header = Header::default();
        header.face_names.push(FaceName {
            id: 0,
            lang: "HANGUL".into(),
            face: "바탕".into(),
        });
        header.char_shapes.push(CharPr {
            id: 0,
            height: 1200,
            font_ref: FontRef::default(),
            ..CharPr::default()
        });
        header.para_shapes.push(ParaPr::default());

        let make_para = |text: &str, vp: i64| Paragraph {
            id: 0,
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![Run {
                char_pr_id_ref: 0,
                text: text.into(),
                is_hyperlink: false,
            }],
            line_segs: vec![LineSeg {
                vert_pos: vp,
                vert_size: 1000,
                ..LineSeg::default()
            }],
        };

        let doc = HwpxDocument {
            mimetype: "application/hwp+zip".into(),
            header,
            sections: vec![Section {
                paragraphs: vec![
                    make_para("p1", 0),
                    make_para("p2", 1600),
                    // Third paragraph wraps back to vert_pos=0 → new page.
                    make_para("p3", 0),
                ],
            }],
        };

        let spec: RuleSpec = serde_json::from_str(r#"{"charshape":{"fontsize":10}}"#).unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 3);
        assert_eq!((r.violations[0].page_no, r.violations[0].line_no), (1, 1));
        assert_eq!((r.violations[1].page_no, r.violations[1].line_no), (1, 2));
        assert_eq!((r.violations[2].page_no, r.violations[2].line_no), (2, 1));
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
