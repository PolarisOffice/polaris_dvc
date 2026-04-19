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
use crate::rules::schema::{
    BorderRule, BulletSpec, CharShape, LevelType, ParaShape, RuleSpec, SpecialCharacter, TableSpec,
};

use polaris_rhwpdvc_hwpx::{
    Border, BorderFill, CharPr, HwpxDocument, ParaPr, Paragraph, Run, Table,
};

/// Which ruleset to apply.
///
/// - `Extended` (default): everything our engine can check. Superset of
///   DVC. Rules upstream leaves as no-op (e.g. table `margin-*`,
///   `bgfill-*`, `bggradation-*`, paragraph `horizontal`) still fire
///   here. Useful as a stricter OWPML-spec validator.
/// - `DvcStrict`: only JIDs upstream `Checker.cpp` actually validates.
///   Violations whose JID is a known upstream no-op are silently dropped.
///   This is the profile to use when your goal is byte-compatible output
///   with the upstream `DVC.exe`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CheckProfile {
    #[default]
    Extended,
    DvcStrict,
}

#[derive(Default)]
pub struct EngineOptions {
    pub stop_on_first: bool,
    pub profile: CheckProfile,
}

/// JIDs that upstream `Checker.cpp` lists only as `break;` with no real
/// comparison. In `DvcStrict` mode we drop violations carrying these
/// codes so our output stays byte-compatible with DVC.exe. The list is
/// derived from an audit of upstream `Checker.cpp` (search for
/// `case JID_X:\n    break;` patterns) cross-referenced with the JIDs
/// our own engine actually emits.
fn dvc_strict_allows(code: ErrorCode) -> bool {
    !matches!(
        code.value(),
        // JID_PARA_SHAPE_HORIZONTAL (aliased as PARA_SHAPE_ALIGN) — noop.
        2001
        // JID_TABLE_MARGIN_{LEFT,RIGHT,TOP,BOTTOM} — noop.
        | 3022..=3025
        // JID_TABLE_BGFILL_{TYPE,FACECOLOR,PATTONCOLOR,PATTONTYPE} — noop.
        | 3037..=3040
        // JID_TABLE_BGGRADATION_* — noop (also not yet emitted by us).
        | 3041..=3048
        // JID_TABLE_CAPTION_* — noop (also not yet emitted by us).
        | 3026..=3030
    )
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
        // Strict-mode gate: drop violations whose JID upstream leaves as
        // a no-op. This keeps our output byte-compatible with DVC.exe
        // while leaving the engine's over-implementations intact for the
        // default `Extended` profile.
        if self.opts.profile == CheckProfile::DvcStrict && !dvc_strict_allows(v.error_code) {
            return true;
        }
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

    // Document-scope table checks (border / treat-as-char / table-in-table).
    // Runs before the section walk so table violations report before any
    // run-level violations downstream of the table.
    if let Some(table_spec) = spec.table.as_ref() {
        for section in &doc.sections {
            for table in &section.tables {
                if !check_table(&mut ctx, table, table_spec) {
                    return ctx.report;
                }
            }
        }
    }

    // Document-scope bullet / outline / paranumbullet checks.
    if let Some(bullet_spec) = spec.bullet.as_ref() {
        if !check_bullets(&mut ctx, bullet_spec) {
            return ctx.report;
        }
    }
    if let Some(ol_spec) = spec.outlineshape.as_ref() {
        if !check_outline_levels(
            &mut ctx,
            ol_spec.leveltype.as_deref(),
            jid::OUTLINESHAPE_NUMBERTYPE,
            jid::OUTLINESHAPE_NUMBERSHAPE,
        ) {
            return ctx.report;
        }
    }
    if let Some(pn_spec) = spec.paranumbullet.as_ref() {
        if !check_outline_levels(
            &mut ctx,
            pn_spec.leveltype.as_deref(),
            jid::PARANUMBULLET_NUMBERTYPE,
            jid::PARANUMBULLET_NUMBERSHAPE,
        ) {
            return ctx.report;
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

        if let Some(sc) = spec.specialcharacter.as_ref() {
            if !check_special_character(ctx, paragraph, run, sc) {
                return false;
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

    // Fontsize: spec value is in points (e.g., 10 or {min:9, max:12}).
    // CharPr.height is in 1/100 pt, so we compare the spec in points
    // against (height / 100.0).
    if let Some(r) = spec.fontsize.as_ref() {
        if r.is_constrained() {
            let actual_pt = char_pr.height as f64 / 100.0;
            if !r.matches(actual_pt) {
                let v = violation_for(
                    ctx,
                    paragraph,
                    run,
                    jid::CHAR_SHAPE_FONTSIZE,
                    format!("fontsize {} pt outside spec {}", actual_pt, r.describe()),
                );
                if !ctx.push(v) {
                    return false;
                }
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

    // Presence-of-element decorations (OUTLINE/EMBOSS/ENGRAVE/SHADOW/
    // SUPSCRIPT/SUBSCRIPT). Upstream `Checker.cpp` compares
    // `charshape->getX() != charPr->charPrInfo.x`, which we mirror here
    // via boolean-expected vs boolean-actual.
    for (spec_field, actual, code, label) in [
        (
            spec.outline,
            char_pr.outline,
            jid::CHAR_SHAPE_OUTLINE,
            "outline",
        ),
        (
            spec.emboss,
            char_pr.emboss,
            jid::CHAR_SHAPE_EMBOSS,
            "emboss",
        ),
        (
            spec.engrave,
            char_pr.engrave,
            jid::CHAR_SHAPE_ENGRAVE,
            "engrave",
        ),
        (
            spec.shadow,
            char_pr.shadow.is_some(),
            jid::CHAR_SHAPE_SHADOW,
            "shadow",
        ),
        (
            spec.supscript,
            char_pr.supscript,
            jid::CHAR_SHAPE_SUPSCRIPT,
            "supscript",
        ),
        (
            spec.subscript,
            char_pr.subscript,
            jid::CHAR_SHAPE_SUBSCRIPT,
            "subscript",
        ),
    ] {
        if let Some(expected) = spec_field {
            if actual != expected {
                let v = violation_for(
                    ctx,
                    paragraph,
                    run,
                    code,
                    format!("expected {}={}, got {}", label, expected, actual),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
    }

    // Shadow detail (shadowtype / shadow-x / shadow-y / shadow-color).
    // Only compare when the doc has an active shadow — if the doc has no
    // shadow, the top-level `shadow` boolean (Pt.1 above) already covers
    // the "present/absent" mismatch.
    if let Some(shadow) = char_pr.shadow.as_ref() {
        if let Some(expected) = spec.shadowtype.as_deref() {
            let actual_ord = shadow_type_ordinal(&shadow.kind);
            let expected_ord = shadow_type_ordinal(expected);
            if actual_ord != expected_ord {
                let v = violation_for(
                    ctx,
                    paragraph,
                    run,
                    jid::CHAR_SHAPE_SHADOWTYPE,
                    format!(
                        "shadowtype {:?} (ord={}) != spec {:?} (ord={})",
                        shadow.kind, actual_ord, expected, expected_ord
                    ),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
        if let Some(r) = spec.shadow_x.as_ref() {
            if r.is_constrained() && !r.matches(shadow.offset_x as f64) {
                let v = violation_for(
                    ctx,
                    paragraph,
                    run,
                    jid::CHAR_SHAPE_SHADOW_X,
                    format!("shadow-x {} outside spec {}", shadow.offset_x, r.describe()),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
        if let Some(r) = spec.shadow_y.as_ref() {
            if r.is_constrained() && !r.matches(shadow.offset_y as f64) {
                let v = violation_for(
                    ctx,
                    paragraph,
                    run,
                    jid::CHAR_SHAPE_SHADOW_Y,
                    format!("shadow-y {} outside spec {}", shadow.offset_y, r.describe()),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
        if let Some(expected) = spec.shadow_color.as_ref() {
            if let Some(actual) = decode_hex_color(&shadow.color) {
                if actual != expected.0 {
                    let v = violation_for(
                        ctx,
                        paragraph,
                        run,
                        jid::CHAR_SHAPE_SHADOW_COLOR,
                        format!("shadow-color {:#x} != spec {:#x}", actual, expected.0),
                    );
                    if !ctx.push(v) {
                        return false;
                    }
                }
            }
        }
    }

    if let Some(r) = spec.ratio.as_ref() {
        if r.is_constrained() && !r.matches(char_pr.ratio_hangul) {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_RATIO,
                format!(
                    "ratio {} outside spec {}",
                    char_pr.ratio_hangul,
                    r.describe()
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(r) = spec.spacing.as_ref() {
        if r.is_constrained() && !r.matches(char_pr.spacing_hangul) {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_SPACING,
                format!(
                    "spacing {} outside spec {}",
                    char_pr.spacing_hangul,
                    r.describe()
                ),
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
    if let Some(r) = spec.linespacingvalue.as_ref() {
        if r.is_constrained() && !r.matches(para_pr.line_spacing_value) {
            let v = para_violation(
                ctx,
                paragraph,
                jid::PARA_SHAPE_LINESPACINGVALUE,
                format!(
                    "line spacing {} outside spec {}",
                    para_pr.line_spacing_value,
                    r.describe()
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(r) = spec.spacing_paraup.as_ref() {
        if r.is_constrained() && !r.matches(para_pr.margin_prev) {
            let v = para_violation(
                ctx,
                paragraph,
                jid::PARA_SHAPE_SPACING_PARAUP,
                format!(
                    "spacing-paraup {} outside spec {}",
                    para_pr.margin_prev,
                    r.describe()
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(r) = spec.spacing_parabottom.as_ref() {
        if r.is_constrained() && !r.matches(para_pr.margin_next) {
            let v = para_violation(
                ctx,
                paragraph,
                jid::PARA_SHAPE_SPACING_PARABOTTOM,
                format!(
                    "spacing-parabottom {} outside spec {}",
                    para_pr.margin_next,
                    r.describe()
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(r) = spec.indent.as_ref() {
        // `indent` checks the first-line positive indent (margin_intent > 0).
        let actual = para_pr.margin_intent.max(0.0);
        if r.is_constrained() && !r.matches(actual) {
            let v = para_violation(
                ctx,
                paragraph,
                jid::PARA_SHAPE_INDENT,
                format!("indent {} outside spec {}", actual, r.describe()),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(r) = spec.outdent.as_ref() {
        // `outdent` mirrors `indent` with a sign flip: negative margin_intent
        // counts as outdent magnitude.
        let actual = (-para_pr.margin_intent).max(0.0);
        if r.is_constrained() && !r.matches(actual) {
            let v = para_violation(
                ctx,
                paragraph,
                jid::PARA_SHAPE_OUTDENT,
                format!("outdent {} outside spec {}", actual, r.describe()),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.align.as_deref() {
        if !para_pr.align_horizontal.eq_ignore_ascii_case(expected) {
            let v = para_violation(
                ctx,
                paragraph,
                jid::PARA_SHAPE_ALIGN,
                format!(
                    "expected align {}, got {}",
                    expected, para_pr.align_horizontal
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    true
}

/// LinePosition mapping from upstream `DVCInterface.h`:
/// `1 = Top, 2 = Bottom, 3 = Left, 4 = Right`.
fn side_from_position(bf: &BorderFill, position: u32) -> Option<&Border> {
    match position {
        1 => Some(&bf.top),
        2 => Some(&bf.bottom),
        3 => Some(&bf.left),
        4 => Some(&bf.right),
        _ => None,
    }
}

/// LineShape mapping from upstream `DVCInterface.h` — the integers used
/// in rule specs line up 1:1 with the enum order. We decode to an integer
/// so the spec's numeric `bordertype` can be compared directly.
fn line_shape_ordinal(kind: &str) -> Option<u32> {
    Some(match kind.to_ascii_uppercase().as_str() {
        "NONE" | "" => 0,
        "SOLID" => 1,
        "DOT" => 2,
        "DASH" => 3,
        "DASH_DOT" => 4,
        "DASH_DOT_DOT" => 5,
        "LONGDASH" | "LONG_DASH" => 6,
        "CIRCLE" => 7,
        "DOUBLESLIM" => 8,
        _ => return None,
    })
}

/// HWPX stores color as `#RRGGBB`; DVC rule specs store it as an integer
/// (0 for black, otherwise a packed RGB or palette index — the exact
/// convention varies by org). We decode the hex to a `u32` so equality
/// against numeric spec values works for the common "black = 0" case.
fn decode_hex_color(s: &str) -> Option<u32> {
    let t = s.trim_start_matches('#');
    if t.len() == 6 {
        u32::from_str_radix(t, 16).ok()
    } else {
        None
    }
}

/// Map a shadow-type identifier (from OWPML XML or from the spec JSON)
/// to upstream's `ShadowType` ordinal: 0=None, 1=Discontinuous, 2=Continuous.
///
/// Accepts:
///   - integer-as-string ("0", "1", "2")
///   - Korean (jsonFullSpec.json) "없음"/"비연속"/"연속"
///   - OWPML upper-case "NONE"/"DISCONTINUOUS"/"CONTINUOUS"
///   - empty string → 0
///
/// Anything unrecognized falls through to a sentinel (`u32::MAX`) so a
/// known↔unknown comparison is never a false match.
fn shadow_type_ordinal(s: &str) -> u32 {
    let t = s.trim();
    if let Ok(n) = t.parse::<u32>() {
        return n;
    }
    match t {
        "" | "없음" | "NONE" | "None" | "none" => 0,
        "비연속" | "DISCONTINUOUS" | "Discontinuous" | "UnContinue" | "uncontinue" => 1,
        "연속" | "CONTINUOUS" | "Continuous" | "Continue" | "continue" => 2,
        _ => u32::MAX,
    }
}

fn check_table(ctx: &mut Ctx, table: &Table, spec: &TableSpec) -> bool {
    if let Some(false) = spec.table_in_table {
        if table.nesting_depth >= 1 {
            let v = ViolationRecord {
                table_id: table.id,
                is_in_table: true,
                is_in_table_in_table: true,
                page_no: ctx.page_no.max(1),
                line_no: ctx.line_no,
                error_code: jid::TABLE_IN_TABLE,
                error_string: "nested table disallowed by spec".to_string(),
                ..ViolationRecord::new(jid::TABLE_IN_TABLE)
            };
            if !ctx.push(v) {
                return false;
            }
        }
    }

    // size-width / size-height
    if let Some(r) = spec.size_width.as_ref() {
        if r.is_constrained() && !r.matches(table.sz.width as f64) {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_SIZE_WIDTH,
                format!(
                    "table {} width {} outside spec {}",
                    table.id,
                    table.sz.width,
                    r.describe()
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }
    if let Some(r) = spec.size_height.as_ref() {
        if r.is_constrained() && !r.matches(table.sz.height as f64) {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_SIZE_HEIGHT,
                format!(
                    "table {} height {} outside spec {}",
                    table.id,
                    table.sz.height,
                    r.describe()
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    if let Some(expected) = spec.treat_as_char {
        if table.pos.treat_as_char != expected {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_TREAT_AS_CHAR,
                format!(
                    "table {} treatAsChar {} != spec {}",
                    table.id, table.pos.treat_as_char, expected
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    // margin-left/right/top/bottom — <hp:inMargin> (cellMargin).
    if !check_margin_side(
        ctx,
        table,
        jid::TABLE_MARGIN_LEFT,
        "margin-left",
        spec.margin_left.as_ref(),
        table.in_margin.left,
    ) {
        return false;
    }
    if !check_margin_side(
        ctx,
        table,
        jid::TABLE_MARGIN_RIGHT,
        "margin-right",
        spec.margin_right.as_ref(),
        table.in_margin.right,
    ) {
        return false;
    }
    if !check_margin_side(
        ctx,
        table,
        jid::TABLE_MARGIN_TOP,
        "margin-top",
        spec.margin_top.as_ref(),
        table.in_margin.top,
    ) {
        return false;
    }
    if !check_margin_side(
        ctx,
        table,
        jid::TABLE_MARGIN_BOTTOM,
        "margin-bottom",
        spec.margin_bottom.as_ref(),
        table.in_margin.bottom,
    ) {
        return false;
    }

    // bgfill-* and bggradation-* live on the referenced <hh:borderFill>.
    // Same lookup as borders — if the borderFill is missing we silently
    // skip (the document is malformed, but there's nothing to compare).
    if let Some(bf) = ctx.doc.header.border_fill(table.border_fill_id_ref) {
        if !check_bgfill(ctx, table, &bf.fill, spec) {
            return false;
        }
    }

    if let Some(borders) = spec.border.as_ref() {
        let Some(bf) = ctx.doc.header.border_fill(table.border_fill_id_ref) else {
            return true; // Missing borderFill — nothing to compare.
        };
        for rule in borders {
            let Some(position) = rule.position else {
                continue;
            };
            let Some(side) = side_from_position(bf, position) else {
                continue;
            };
            if !check_border_side(ctx, table, position, side, rule) {
                return false;
            }
        }
    }
    true
}

/// `bgfill-*` checkers. Reads flat `spec.bgfill_*` fields and compares
/// against the referenced borderFill's `Fill`. Skips silently when the
/// doc has no fill information.
fn check_bgfill(
    ctx: &mut Ctx,
    table: &Table,
    fill: &polaris_rhwpdvc_hwpx::Fill,
    spec: &TableSpec,
) -> bool {
    if let Some(expected) = spec.bgfill_type {
        let actual = fill.ordinal();
        if actual != expected {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_BGFILL_TYPE,
                format!(
                    "table {} bgfill-type {} != spec {}",
                    table.id, actual, expected
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }
    if let Some(expected) = spec.bgfill_facecolor.as_ref() {
        if let Some(actual_hex) = fill.face_color_hex() {
            if let Some(actual) = decode_hex_color(actual_hex) {
                if actual != expected.0 {
                    let v = table_violation(
                        ctx,
                        table,
                        jid::TABLE_BGFILL_FACECOLOR,
                        format!(
                            "table {} bgfill-facecolor {:#x} != spec {:#x}",
                            table.id, actual, expected.0
                        ),
                    );
                    if !ctx.push(v) {
                        return false;
                    }
                }
            }
        }
    }
    if let Some(expected) = spec.bgfill_pattoncolor.as_ref() {
        if let Some(actual_hex) = fill.patton_color_hex() {
            if let Some(actual) = decode_hex_color(actual_hex) {
                if actual != expected.0 {
                    let v = table_violation(
                        ctx,
                        table,
                        jid::TABLE_BGFILL_PATTONCOLOR,
                        format!(
                            "table {} bgfill-pattoncolor {:#x} != spec {:#x}",
                            table.id, actual, expected.0
                        ),
                    );
                    if !ctx.push(v) {
                        return false;
                    }
                }
            }
        }
    }
    if let Some(expected) = spec.bgfill_pattontype.as_deref() {
        if let Some(actual) = fill.patton_type() {
            if !actual.eq_ignore_ascii_case(expected) {
                let v = table_violation(
                    ctx,
                    table,
                    jid::TABLE_BGFILL_PATTONTYPE,
                    format!(
                        "table {} bgfill-pattontype \"{}\" != spec \"{}\"",
                        table.id, actual, expected
                    ),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
    }
    true
}

fn check_margin_side(
    ctx: &mut Ctx,
    table: &Table,
    code: ErrorCode,
    label: &str,
    rule: Option<&crate::rules::schema::Range64>,
    actual: i64,
) -> bool {
    let Some(r) = rule else { return true };
    if !r.is_constrained() {
        return true;
    }
    if !r.matches(actual as f64) {
        let v = table_violation(
            ctx,
            table,
            code,
            format!(
                "table {} {} = {} outside spec {}",
                table.id,
                label,
                actual,
                r.describe()
            ),
        );
        if !ctx.push(v) {
            return false;
        }
    }
    true
}

fn check_border_side(
    ctx: &mut Ctx,
    table: &Table,
    position: u32,
    side: &Border,
    rule: &BorderRule,
) -> bool {
    if let Some(expected) = rule.bordertype {
        if let Some(actual) = line_shape_ordinal(&side.kind) {
            if actual != expected {
                let v = table_violation(
                    ctx,
                    table,
                    jid::TABLE_BORDER_TYPE,
                    format!(
                        "table {} position {} bordertype {} (\"{}\") != spec {}",
                        table.id, position, actual, side.kind, expected
                    ),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
    }
    if let Some(expected) = rule.size {
        if (side.width_mm - expected).abs() > 1e-6 {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_BORDER_SIZE,
                format!(
                    "table {} position {} border width {} mm != spec {}",
                    table.id, position, side.width_mm, expected
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }
    if let Some(expected) = rule.color {
        if let Some(actual) = decode_hex_color(&side.color) {
            if actual != expected {
                let v = table_violation(
                    ctx,
                    table,
                    jid::TABLE_BORDER_COLOR,
                    format!(
                        "table {} position {} border color {:#x} != spec {:#x}",
                        table.id, position, actual, expected
                    ),
                );
                if !ctx.push(v) {
                    return false;
                }
            }
        }
    }
    true
}

fn table_violation(
    ctx: &Ctx,
    table: &Table,
    code: ErrorCode,
    diagnostic: String,
) -> ViolationRecord {
    ViolationRecord {
        table_id: table.id,
        is_in_table: true,
        is_in_table_in_table: table.nesting_depth >= 1,
        page_no: ctx.page_no.max(1),
        line_no: ctx.line_no,
        error_code: code,
        error_string: diagnostic,
        ..ViolationRecord::new(code)
    }
}

fn check_bullets(ctx: &mut Ctx, spec: &BulletSpec) -> bool {
    let Some(allowed) = spec.bulletshapes.as_deref() else {
        return true;
    };
    for bullet in &ctx.doc.header.bullets {
        // A bullet's `char` may be an empty string (no-op bullet). Skip it.
        if bullet.char_.is_empty() {
            continue;
        }
        let ok = bullet.char_.chars().all(|ch| allowed.contains(ch));
        if !ok {
            let v = ViolationRecord {
                page_no: ctx.page_no.max(1),
                line_no: ctx.line_no,
                error_code: jid::BULLET_SHAPES,
                error_string: format!(
                    "bullet id {} char '{}' not in allowed set \"{}\"",
                    bullet.id, bullet.char_, allowed
                ),
                ..ViolationRecord::new(jid::BULLET_SHAPES)
            };
            if !ctx.push(v) {
                return false;
            }
        }
    }
    true
}

/// Shared logic for outlineshape and paranumbullet. Walks each numbering
/// in the header, and for every level entry whose number matches a spec
/// `leveltype[*].level`, compares `num_format` (→ numbertype) and
/// `number_shape` (→ numbershape). Emits the caller-supplied error codes.
fn check_outline_levels(
    ctx: &mut Ctx,
    leveltype: Option<&[LevelType]>,
    num_type_code: ErrorCode,
    num_shape_code: ErrorCode,
) -> bool {
    let Some(levels) = leveltype else { return true };
    if levels.is_empty() {
        return true;
    }
    for numbering in &ctx.doc.header.numberings {
        for head in &numbering.para_heads {
            // Find the spec entry for this level. Silently skip unmatched
            // levels — the spec may only constrain a subset.
            let Some(rule) = levels.iter().find(|l| l.level == Some(head.level)) else {
                continue;
            };

            if let Some(expected) = rule.numbertype.as_deref() {
                if head.num_format != expected {
                    let v = ViolationRecord {
                        page_no: ctx.page_no.max(1),
                        line_no: ctx.line_no,
                        error_code: num_type_code,
                        error_string: format!(
                            "numbering id {} level {}: numFormat \"{}\" != spec \"{}\"",
                            numbering.id, head.level, head.num_format, expected
                        ),
                        ..ViolationRecord::new(num_type_code)
                    };
                    if !ctx.push(v) {
                        return false;
                    }
                }
            }
            if let Some(expected) = rule.numbershape {
                if head.number_shape != expected {
                    let v = ViolationRecord {
                        page_no: ctx.page_no.max(1),
                        line_no: ctx.line_no,
                        error_code: num_shape_code,
                        error_string: format!(
                            "numbering id {} level {}: numberShape {} != spec {}",
                            numbering.id, head.level, head.number_shape, expected
                        ),
                        ..ViolationRecord::new(num_shape_code)
                    };
                    if !ctx.push(v) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn check_special_character(
    ctx: &mut Ctx,
    paragraph: &Paragraph,
    run: &Run,
    spec: &SpecialCharacter,
) -> bool {
    // Scan each Unicode scalar in the run's text. Reports at most one
    // violation per bound per run — the first offending code point wins.
    // This matches upstream behavior where `errorText` is the offending
    // run's text rather than the individual character.
    let mut below: Option<u32> = None;
    let mut above: Option<u32> = None;
    for ch in run.text.chars() {
        let cp = ch as u32;
        if let Some(min) = spec.minimum {
            if cp < min && below.is_none() {
                below = Some(cp);
            }
        }
        if let Some(max) = spec.maximum {
            if cp > max && above.is_none() {
                above = Some(cp);
            }
        }
        if below.is_some() && above.is_some() {
            break;
        }
    }
    if let Some(cp) = below {
        let v = violation_for(
            ctx,
            paragraph,
            run,
            jid::SPECIAL_CHAR_MINIMUM,
            format!(
                "code point U+{:04X} below minimum U+{:04X}",
                cp,
                spec.minimum.unwrap()
            ),
        );
        if !ctx.push(v) {
            return false;
        }
    }
    if let Some(cp) = above {
        let v = violation_for(
            ctx,
            paragraph,
            run,
            jid::SPECIAL_CHAR_MAXIMUM,
            format!(
                "code point U+{:04X} above maximum U+{:04X}",
                cp,
                spec.maximum.unwrap()
            ),
        );
        if !ctx.push(v) {
            return false;
        }
    }
    true
}

/// Paragraph-level violation (no specific run). Stamps the current
/// page/line counter so downstream tooling can locate the paragraph.
fn para_violation(
    ctx: &Ctx,
    paragraph: &Paragraph,
    code: ErrorCode,
    diagnostic: String,
) -> ViolationRecord {
    ViolationRecord {
        para_pr_id_ref: paragraph.para_pr_id_ref,
        page_no: ctx.page_no,
        line_no: ctx.line_no,
        error_code: code,
        error_string: diagnostic,
        ..ViolationRecord::new(code)
    }
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
    use polaris_rhwpdvc_hwpx::{CharPr, FaceName, FontRef, Header, HwpxDocument, ParaPr};

    fn make_doc(char_pr: CharPr, para_pr: ParaPr, text: &str) -> HwpxDocument {
        let mut header = Header::default();
        header.face_names.push(FaceName {
            id: 0,
            lang: "HANGUL".into(),
            face: "바탕".into(),
        });
        header.char_shapes.push(char_pr);
        header.para_shapes.push(para_pr);
        let run = polaris_rhwpdvc_hwpx::Run {
            char_pr_id_ref: 0,
            text: text.into(),
            is_hyperlink: false,
        };
        let paragraph = polaris_rhwpdvc_hwpx::Paragraph {
            id: 0,
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![run],
            line_segs: Vec::new(),
        };
        let section = polaris_rhwpdvc_hwpx::Section {
            paragraphs: vec![paragraph],
            tables: Vec::new(),
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
    fn linespacing_mismatch_emits_2008() {
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
        assert_eq!(r.violations[0].error_code.value(), 2008);
    }

    #[test]
    fn page_line_accumulates_across_paragraphs() {
        use polaris_rhwpdvc_hwpx::{LineSeg, Paragraph, Run, Section};

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
                tables: Vec::new(),
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
        use polaris_rhwpdvc_hwpx::{LineSeg, Paragraph, Run, Section};

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
                tables: Vec::new(),
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
    fn table_border_type_mismatch_emits_3033() {
        use polaris_rhwpdvc_hwpx::{Border, BorderFill, Section, Table};

        let mut header = Header::default();
        header.border_fills.push(BorderFill {
            id: 1,
            top: Border {
                kind: "DASH".into(),
                width_mm: 0.12,
                color: "#000000".into(),
            },
            bottom: Border {
                kind: "SOLID".into(),
                width_mm: 0.12,
                color: "#000000".into(),
            },
            left: Border {
                kind: "SOLID".into(),
                width_mm: 0.12,
                color: "#000000".into(),
            },
            right: Border {
                kind: "SOLID".into(),
                width_mm: 0.12,
                color: "#000000".into(),
            },
            fill: polaris_rhwpdvc_hwpx::Fill::None,
        });

        let doc = HwpxDocument {
            mimetype: "application/hwp+zip".into(),
            header,
            sections: vec![Section {
                paragraphs: Vec::new(),
                tables: vec![Table {
                    id: 5,
                    border_fill_id_ref: 1,
                    row_cnt: 1,
                    col_cnt: 1,
                    nesting_depth: 0,
                    ..Table::default()
                }],
            }],
        };

        // Spec: Top (position 1) must be SOLID (bordertype 1).
        let spec: RuleSpec =
            serde_json::from_str(r#"{"table":{"border":[{"position":1,"bordertype":1}]}}"#)
                .unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].error_code.value(), 3033);
        assert_eq!(r.violations[0].table_id, 5);
        assert!(r.violations[0].is_in_table);
    }

    #[test]
    fn table_in_table_forbidden_emits_3056() {
        use polaris_rhwpdvc_hwpx::{Section, Table};

        let doc = HwpxDocument {
            mimetype: "application/hwp+zip".into(),
            header: Header::default(),
            sections: vec![Section {
                paragraphs: Vec::new(),
                tables: vec![
                    Table {
                        id: 1,
                        nesting_depth: 0,
                        ..Table::default()
                    },
                    Table {
                        id: 2,
                        nesting_depth: 1,
                        ..Table::default()
                    },
                ],
            }],
        };
        let spec: RuleSpec = serde_json::from_str(r#"{"table":{"table-in-table":false}}"#).unwrap();
        let r = validate(&doc, &spec, &EngineOptions::default());
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].error_code.value(), 3056);
        assert_eq!(r.violations[0].table_id, 2);
        assert!(r.violations[0].is_in_table_in_table);
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
                ..EngineOptions::default()
            },
        );
        assert_eq!(r.violations.len(), 1);
        assert!(r.stopped_early);
    }
}
