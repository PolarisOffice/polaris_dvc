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

use polaris_dvc_hwpx::{Border, BorderFill, CharPr, HwpxDocument, ParaPr, Paragraph, Run, Table};

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
    /// Enable KS X 6101 XSD conformance checks (JID 13000-13999). Off
    /// by default. The bundled `generated_owpml.rs` is produced by
    /// `tools/gen-owpml/` from the official KS X 6101 XSDs, so it's
    /// materially complete (~292 element decls, enum/type/cardinality
    /// coverage). But real-world HWPX drifts from the formal spec in
    /// several places: `<hp:linesegarray>` isn't declared in the XSD
    /// at all, `<hh:borderFill>` carries diagonal-line attrs (`slash`,
    /// `backSlash`, …) that the XSD omits, and the
    /// `compatibleDocument@targetProgram` enum is out of date (misses
    /// `HWP2018`+). Enabling this pass on a typical document emits
    /// 30-40 findings that are XSD-correct but document-legal; until we
    /// augment the generated schema with a drift-patch layer, the
    /// default stays off so the 11000/12000 axes aren't drowned out.
    /// Explicit opt-in (`--enable-schema` on the CLI, `enableSchema:
    /// true` in the WASM API) is the supported way to run this axis.
    pub enable_schema: bool,
}

/// JIDs the `DvcStrict` gate drops so our output stays byte-compatible
/// with `DVC.exe`. Two sources feed this list:
///
/// 1. **Upstream `Checker.cpp` no-ops** — the dispatch switch has
///    `case JID_X: break;` with no real comparison, so DVC accepts
///    those spec entries but emits no violations. Our engine
///    over-implements several (margin, bgfill, caption, horizontal);
///    strict mode drops them.
/// 2. **polaris-original integrity JIDs (11000-11999)** — not in
///    upstream at all. Under strict mode we filter them out so the
///    output remains a pure DVC-parity run; otherwise they surface
///    alongside normal rule violations.
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
        // polaris-original categories — not in upstream DVC at all.
        | 11000..=11999   // Integrity   (cross-ref / manifest / lineseg)
        | 12000..=12999   // Container   (ZIP well-formedness)
        | 13000..=13999   // Schema      (KS X 6101 XSD conformance)
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
    /// Which section we're currently walking. `None` outside the
    /// section loop (e.g. during the Container / Integrity /
    /// document-scope prologue). Rule checkers push with `None`
    /// set to skip auto-anchoring when the context isn't a single
    /// section — document-scope violations (macro permission,
    /// bullet permission, etc.) don't point at one specific section.
    current_section: Option<usize>,
    /// Which paragraph we're currently checking within
    /// `current_section`. Only set during the per-paragraph walk.
    current_para_id: Option<u32>,
}

impl<'a> Ctx<'a> {
    fn push(&mut self, mut v: ViolationRecord) -> bool {
        // Strict-mode gate: drop violations whose JID upstream leaves as
        // a no-op. This keeps our output byte-compatible with DVC.exe
        // while leaving the engine's over-implementations intact for the
        // default `Extended` profile.
        if self.opts.profile == CheckProfile::DvcStrict {
            if !dvc_strict_allows(v.error_code) {
                return true;
            }
            // Clear polaris-only hint fields on the surviving records
            // so JSON / XML output stays byte-identical to upstream
            // DVC. (The DVC-compat fields — CharIDRef, ParaPrIDRef,
            // errorText, PageNo, LineNo, ErrorCode, conditional
            // table/style/shape/hyperlink — are the only thing
            // upstream emits.)
            v.error_string.clear();
            v.file_label.clear();
            v.byte_offset = 0;
        } else if v.file_label.is_empty() {
            // Extended profile: auto-anchor the violation to the
            // section / paragraph the walker is currently inside, if
            // the checker didn't set a more specific location. Only
            // Rule-axis JIDs (1000-7999) go through this path with
            // empty `file_label` — polaris-original axes
            // (integrity / schema / container) set their own labels
            // explicitly before push.
            if let Some(si) = self.current_section {
                v.file_label = format!("section{si}");
                if v.byte_offset == 0 {
                    if let Some(para_id) = self.current_para_id {
                        let bytes = self
                            .doc
                            .structural
                            .section_xml_bytes
                            .get(si)
                            .map(Vec::as_slice)
                            .unwrap_or(&[]);
                        v.byte_offset = find_para_byte_offset(bytes, para_id);
                    }
                }
            }
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
        current_section: None,
        current_para_id: None,
    };

    // Container well-formedness (polaris-original, JID 12000-12999).
    // ZIP-level defects are most likely to invalidate everything
    // downstream; report them first. Strict-mode filters them out.
    if !check_container(&mut ctx) {
        return ctx.report;
    }

    // Structural-integrity checks (polaris-original, JID 11000-11999).
    // Cross-ref / manifest consistency sits between ZIP and content.
    if !check_integrity(&mut ctx) {
        return ctx.report;
    }

    // KS X 6101 XSD conformance (polaris-original, JID 13000-13999).
    // Opt-in via EngineOptions.enable_schema — the bootstrap schema is
    // a top-20% subset and produces many unexpected-child findings on
    // elements it doesn't cover yet. Full coverage via tools/gen-owpml.
    if ctx.opts.enable_schema && !check_schema(&mut ctx) {
        return ctx.report;
    }

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
        for (si, section) in doc.sections.iter().enumerate() {
            ctx.current_section = Some(si);
            for table in &section.tables {
                if !check_table(&mut ctx, table, table_spec) {
                    return ctx.report;
                }
            }
        }
        ctx.current_section = None;
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

    'sections: for (si, section) in doc.sections.iter().enumerate() {
        // Per upstream `GetPageInfo`, vertical-position trackers reset per
        // section. Page counter is cumulative across sections.
        ctx.before_vert_pos = 0;
        ctx.before_vert_size = 0;
        ctx.current_section = Some(si);
        for paragraph in &section.paragraphs {
            ctx.current_para_id = Some(paragraph.id);
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
        ctx.current_para_id = None;
    }
    ctx.current_section = None;

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
            // `use_style` is populated by `violation_for` from
            // `paragraph.style_id_ref != 0`; no manual override needed.
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::STYLE_PERMISSION,
                format!("style id {} used but not permitted", paragraph.style_id_ref),
            );
            if !ctx.push(v) {
                return false;
            }
        }

        if hyperlink_forbidden && run.is_hyperlink {
            // `use_hyperlink` is populated by `violation_for` from
            // `run.is_hyperlink`.
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::HYPERLINK_PERMISSION,
                "hyperlink run found but not permitted".to_string(),
            );
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

    // r-size (<hh:relSz hangul=…>). Upstream JID_CHAR_SHAPE_RSIZE.
    if let Some(r) = spec.r_size.as_ref() {
        if r.is_constrained() && !r.matches(char_pr.rel_sz_hangul) {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_RSIZE,
                format!(
                    "r-size {} outside spec {}",
                    char_pr.rel_sz_hangul,
                    r.describe()
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    // kerning (useKerning="0|1"). Upstream JID_CHAR_SHAPE_KERNING.
    if let Some(expected) = spec.kerning {
        if char_pr.use_kerning != expected {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::CHAR_SHAPE_KERNING,
                format!("expected kerning={}, got {}", expected, char_pr.use_kerning),
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
    // linespacing mode (type enum). Upstream JID_PARA_SHAPE_LINESPACING
    // compares `paraShape->getLinespacing() != paraPr->lineSpacing.type`.
    if let Some(expected) = spec.linespacing.as_deref() {
        if !para_pr.line_spacing_type.eq_ignore_ascii_case(expected) {
            let v = para_violation(
                ctx,
                paragraph,
                jid::PARA_SHAPE_LINESPACING_TYPE,
                format!(
                    "line spacing type \"{}\" != spec \"{}\"",
                    para_pr.line_spacing_type, expected
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

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

    // size-fixed (`lock` attribute on <hp:tbl>).
    // Upstream JID_TABLE_SIZEFIXED compares `iTable->getSizeFixed() !=
    // table->getLock()`.
    if let Some(expected) = spec.size_fixed {
        if table.lock != expected {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_SIZE_FIXED,
                format!(
                    "table {} size-fixed (lock) {} != spec {}",
                    table.id, table.lock, expected
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    // pos / textpos — string compare against the OWPML enum attribute.
    // Upstream compares enum ordinals and has a known mismatch between
    // its PosType (4 values) and OWPML ASOTEXTWRAPTYPE (6 values); our
    // string-compare is a pragmatic stand-in until that's disambiguated.
    if let Some(expected) = spec.pos.as_deref() {
        if !table.text_wrap.eq_ignore_ascii_case(expected) {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_POS,
                format!(
                    "table {} textWrap \"{}\" != spec \"{}\"",
                    table.id, table.text_wrap, expected
                ),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }
    if let Some(expected) = spec.textpos.as_deref() {
        if !table.text_flow.eq_ignore_ascii_case(expected) {
            let v = table_violation(
                ctx,
                table,
                jid::TABLE_TEXTPOS,
                format!(
                    "table {} textFlow \"{}\" != spec \"{}\"",
                    table.id, table.text_flow, expected
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
    fill: &polaris_dvc_hwpx::Fill,
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
        let Some(first) = bullet.char_.chars().next() else {
            continue;
        };
        // Upstream `CheckBulletToCheckList` compares `bullet->bulletChar[0]`
        // against the allowed set — first character only. Prior versions
        // of this engine compared every char of `bullet.char_`, which
        // diverged from DVC.exe for multi-character bullet strings.
        if !allowed.contains(first) {
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
    // NOTE (parity): upstream only checks bullets **referenced** from
    // `ParaBullet`-heading paragraphs, while we iterate every declared
    // bullet in the header regardless of usage. For typical HWPX
    // documents these yield identical output (unused bullets almost
    // never have disallowed chars), but for strict byte parity this
    // should be narrowed to paragraph-referenced bullets only. Tracked
    // in docs/parity-roadmap.md.
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
    // Matches upstream `CheckSpacialCharacterToCheckList`: one
    // `JID_SPECIALCHARACTER` (3100) push per offending code point,
    // regardless of whether the violation is min- or max-sided.
    // `errorText` is the run's full text (populated by the common
    // `violation_for` helper), so multiple violations for the same run
    // share the same `errorText` — identical to upstream's behavior.
    for ch in run.text.chars() {
        let cp = ch as u32;
        let below = spec.minimum.is_some_and(|m| cp < m);
        let above = spec.maximum.is_some_and(|m| cp > m);
        if below || above {
            let v = violation_for(
                ctx,
                paragraph,
                run,
                jid::SPECIAL_CHARACTER,
                format!(
                    "code point U+{:04X} outside spec [{:?}, {:?}]",
                    cp, spec.minimum, spec.maximum
                ),
            );
            if !ctx.push(v) {
                return false;
            }
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
        // Propagate scope flags so the output's `IsInShape` /
        // `UseHyperlink` / `UseStyle` fields reflect what upstream
        // would have recorded from the run's RunTypeInfo:
        //   - isInShape:  set by parser per run (shape scope stack)
        //   - isHyperlink: set by parser per run (field-scope stack)
        //   - isStyle:    true iff the paragraph carries a non-zero
        //                 styleIDRef (upstream
        //                 `OWPMLReader.cpp:304-305`).
        is_in_shape: run.is_in_shape,
        use_hyperlink: run.is_hyperlink,
        use_style: paragraph.style_id_ref != 0,
        error_string: diagnostic,
        ..ViolationRecord::new(code)
    }
}

// ──────────────────────────────────────────────────────────────────
// Structural integrity (polaris-original, JID 11000-11999)
//
// Catches cross-reference and container-shape defects that DVC's rule
// system doesn't address. Common in hand-crafted or LLM-generated
// HWPX. See `error_codes::jid::INTEGRITY_*` for the JID catalogue.
//
// Add a new check by:
//   1. Reserving a JID constant in `error_codes::jid` (11000-11999).
//   2. Adding a text() arm.
//   3. Adding one function `check_integrity_<thing>(&mut Ctx)` that
//      emits through `ctx.push(integrity_violation(ctx, jid, msg))`.
//   4. Calling it from `check_integrity`.
// The strict-mode gate auto-filters everything in this range — no
// allow-list edits needed when adding a new integrity JID.
// ──────────────────────────────────────────────────────────────────
fn check_integrity(ctx: &mut Ctx) -> bool {
    if !check_integrity_id_refs(ctx) {
        return false;
    }
    if !check_integrity_empty_lineseg(ctx) {
        return false;
    }
    if !check_integrity_structural_facts(ctx) {
        return false;
    }
    if !check_integrity_duplicate_ids(ctx) {
        return false;
    }
    if !check_integrity_border_fill_refs(ctx) {
        return false;
    }
    if !check_integrity_font_refs(ctx) {
        return false;
    }
    true
}

fn integrity_violation(
    ctx: &Ctx,
    code: ErrorCode,
    diagnostic: impl Into<String>,
) -> ViolationRecord {
    // Integrity issues aren't tied to a specific run; we anchor them
    // at page 1 / line 1 with empty text. The diagnostic string goes
    // to `error_string` (internal) so the generic-shaped record still
    // carries actionable information for downstream consumers.
    ViolationRecord {
        page_no: ctx.page_no.max(1),
        line_no: ctx.line_no.max(1),
        error_code: code,
        error_string: diagnostic.into(),
        ..ViolationRecord::new(code)
    }
}

/// #2 `charPrIDRef` / `paraPrIDRef` / `styleIDRef` → missing in header.
fn check_integrity_id_refs(ctx: &mut Ctx) -> bool {
    let doc = ctx.doc;
    // Snapshot every orphan with enough location context to anchor it
    // back to a specific section XML + paragraph offset. The web demo's
    // click-to-locate uses `file_label` + `byte_offset` to scroll the
    // viewer to the exact element that holds the dangling IDRef.
    struct Orphan {
        code: ErrorCode,
        msg: String,
        section_idx: usize,
        para_id: u32,
    }
    let mut orphans: Vec<Orphan> = Vec::new();

    for (si, section) in doc.sections.iter().enumerate() {
        for para in &section.paragraphs {
            if doc.header.para_shape(para.para_pr_id_ref).is_none() {
                orphans.push(Orphan {
                    code: jid::INTEGRITY_ORPHAN_PARA_PR_IDREF,
                    msg: format!(
                        "paragraph {} references paraPrIDRef={} with no matching <hh:paraPr>",
                        para.id, para.para_pr_id_ref
                    ),
                    section_idx: si,
                    para_id: para.id,
                });
            }
            if para.style_id_ref != 0
                && !doc.header.styles.iter().any(|s| s.id == para.style_id_ref)
            {
                orphans.push(Orphan {
                    code: jid::INTEGRITY_ORPHAN_STYLE_IDREF,
                    msg: format!(
                        "paragraph {} references styleIDRef={} with no matching <hh:style>",
                        para.id, para.style_id_ref
                    ),
                    section_idx: si,
                    para_id: para.id,
                });
            }
            for run in &para.runs {
                if doc.header.char_shape(run.char_pr_id_ref).is_none() {
                    orphans.push(Orphan {
                        code: jid::INTEGRITY_ORPHAN_CHAR_PR_IDREF,
                        msg: format!(
                            "run in paragraph {} references charPrIDRef={} with no matching <hh:charPr>",
                            para.id, run.char_pr_id_ref
                        ),
                        section_idx: si,
                        para_id: para.id,
                    });
                }
            }
        }
    }

    for o in orphans {
        let bytes = ctx
            .doc
            .structural
            .section_xml_bytes
            .get(o.section_idx)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let byte_offset = find_para_byte_offset(bytes, o.para_id);
        let mut v = integrity_violation(ctx, o.code, o.msg);
        v.file_label = format!("section{}", o.section_idx);
        v.byte_offset = byte_offset;
        if !ctx.push(v) {
            return false;
        }
    }
    true
}

/// Best-effort: locate the byte offset of a `<hp:p id="{para_id}" …>`
/// opening tag in section XML. Returns 0 when no match is found (the
/// caller treats 0 as "no precise location," which makes click-to-
/// locate open the file without scrolling to a specific line).
///
/// Not exhaustive — same para_id can legitimately appear many times
/// in the file (table cells carry `id="0"` repeatedly). This hits the
/// first occurrence, which is still close to the real problem area
/// in practice.
fn find_para_byte_offset(xml: &[u8], para_id: u32) -> u32 {
    let needle = format!("id=\"{para_id}\"");
    let needle_bytes = needle.as_bytes();
    let open_tag = b"<hp:p ";
    let mut i = 0;
    while i + open_tag.len() < xml.len() {
        if xml[i..].starts_with(open_tag) {
            let scan_end = (i + 500).min(xml.len());
            let tag_end = xml[i..scan_end]
                .iter()
                .position(|&b| b == b'>')
                .map(|p| i + p)
                .unwrap_or(scan_end);
            let tag = &xml[i..tag_end];
            if tag.windows(needle_bytes.len()).any(|w| w == needle_bytes) {
                return i as u32;
            }
            i = tag_end;
        } else {
            i += 1;
        }
    }
    0
}

/// #3 Paragraph with runs that carry text but no `<hp:linesegarray>`
/// entries.
fn check_integrity_empty_lineseg(ctx: &mut Ctx) -> bool {
    struct Hit {
        section_idx: usize,
        para_id: u32,
    }
    let mut hits: Vec<Hit> = Vec::new();

    for (si, section) in ctx.doc.sections.iter().enumerate() {
        for para in &section.paragraphs {
            if !para.line_segs.is_empty() {
                continue;
            }
            let has_text = para.runs.iter().any(|r| !r.text.is_empty());
            if has_text {
                hits.push(Hit {
                    section_idx: si,
                    para_id: para.id,
                });
            }
        }
    }

    for h in hits {
        let bytes = ctx
            .doc
            .structural
            .section_xml_bytes
            .get(h.section_idx)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let byte_offset = find_para_byte_offset(bytes, h.para_id);
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_EMPTY_LINESEG,
            format!(
                "section {} paragraph {} has text but empty lineSegArray",
                h.section_idx, h.para_id
            ),
        );
        v.file_label = format!("section{}", h.section_idx);
        v.byte_offset = byte_offset;
        if !ctx.push(v) {
            return false;
        }
    }
    true
}

/// #4 mimetype ZIP invariants (11010-11012) and
/// #1/#5 BinData 3-way cross-references (11020-11022).
/// Reads the parse-time `StructuralFacts` + section's collected
/// `binary_item_id_refs`.
fn check_integrity_structural_facts(ctx: &mut Ctx) -> bool {
    let s = &ctx.doc.structural;

    // #4a — mimetype must be ZIP entry #0.
    if !s.mimetype_is_first {
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_MIMETYPE_POSITION,
            "mimetype is not the first ZIP entry (HWPX / OCF spec requires position 0)",
        );
        v.file_label = "mimetype".to_string();
        if !ctx.push(v) {
            return false;
        }
    }
    // #4b — mimetype must be STORED (uncompressed).
    if !s.mimetype_stored {
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_MIMETYPE_COMPRESSED,
            "mimetype ZIP entry is compressed; spec requires STORED",
        );
        v.file_label = "mimetype".to_string();
        if !ctx.push(v) {
            return false;
        }
    }
    // #4c — content must be exactly `application/hwp+zip`.
    if ctx.doc.mimetype != "application/hwp+zip" {
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_MIMETYPE_CONTENT,
            format!(
                "mimetype content is {:?}, expected \"application/hwp+zip\"",
                ctx.doc.mimetype
            ),
        );
        v.file_label = "mimetype".to_string();
        if !ctx.push(v) {
            return false;
        }
    }

    // BinData 3-way sync: build the three sets once, diff them.
    //
    //   section `binaryItemIDRef` → must match a manifest item's `id`
    //   manifest item href → must match a ZIP entry path
    //   ZIP BinData entry → should match a manifest item href (orphan)
    use std::collections::HashSet;
    let section_refs: HashSet<&str> = ctx
        .doc
        .sections
        .iter()
        .flat_map(|s| s.binary_item_id_refs.iter().map(String::as_str))
        .collect();
    let manifest_ids: HashSet<&str> = s
        .manifest_bindata_items
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    let manifest_hrefs: HashSet<&str> = s
        .manifest_bindata_items
        .iter()
        .map(|(_, href)| href.as_str())
        .collect();
    let zip_paths: HashSet<&str> = s.zip_bindata_paths.iter().map(String::as_str).collect();

    // #1 — section references a binaryItemIDRef that the manifest
    // doesn't define.
    let mut orphan_refs: Vec<&str> = section_refs
        .iter()
        .copied()
        .filter(|r| !manifest_ids.contains(r))
        .collect();
    orphan_refs.sort();
    for r in orphan_refs {
        // The dangling IDRef lives in section XML; the manifest check
        // is about the content.hpf side. We anchor at content.hpf so
        // clicking navigates to where the id catalog should have been.
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_BINDATA_REF_MISSING_MANIFEST,
            format!("section references binaryItemIDRef={r:?} with no matching manifest item"),
        );
        v.file_label = "content.hpf".to_string();
        if !ctx.push(v) {
            return false;
        }
    }

    // #5a — manifest references a BinData file that isn't in the ZIP.
    let mut missing_files: Vec<&str> = manifest_hrefs
        .iter()
        .copied()
        .filter(|h| !zip_paths.contains(h))
        .collect();
    missing_files.sort();
    for href in missing_files {
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_BINDATA_MANIFEST_MISSING_FILE,
            format!("manifest lists BinData href={href:?} but the ZIP has no such entry"),
        );
        v.file_label = "content.hpf".to_string();
        if !ctx.push(v) {
            return false;
        }
    }

    // #5b — ZIP has a BinData entry no manifest item points at.
    let mut orphan_files: Vec<&str> = zip_paths
        .iter()
        .copied()
        .filter(|p| !manifest_hrefs.contains(p))
        .collect();
    orphan_files.sort();
    for path in orphan_files {
        let v = integrity_violation(
            ctx,
            jid::INTEGRITY_BINDATA_ORPHAN_FILE,
            format!("ZIP BinData entry {path:?} is not referenced by any manifest item"),
        );
        if !ctx.push(v) {
            return false;
        }
    }

    true
}

/// Phase 1 — duplicate-id checks across every header table.
/// Two header entries with the same id make any downstream IDRef
/// ambiguous. Each JID emits one violation per duplicated id value
/// (not per pair) to keep the list compact on bulk-duplicate specs.
fn check_integrity_duplicate_ids(ctx: &mut Ctx) -> bool {
    use std::collections::BTreeMap;

    /// Group ids and collect the duplicates (ids that appear > once).
    fn dups<I: IntoIterator<Item = u32>>(ids: I) -> Vec<u32> {
        let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
        for id in ids {
            *counts.entry(id).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .filter_map(|(id, n)| if n > 1 { Some(id) } else { None })
            .collect()
    }

    let h = &ctx.doc.header;

    // Emit one violation per (JID, duplicate id). Each category gets
    // its own JID so downstream tooling can filter by header table.
    let checks: [(ErrorCode, Vec<u32>, &str); 7] = [
        (
            jid::INTEGRITY_DUPLICATE_CHAR_PR_ID,
            dups(h.char_shapes.iter().map(|c| c.id)),
            "<hh:charPr>",
        ),
        (
            jid::INTEGRITY_DUPLICATE_PARA_PR_ID,
            dups(h.para_shapes.iter().map(|p| p.id)),
            "<hh:paraPr>",
        ),
        (
            jid::INTEGRITY_DUPLICATE_BORDER_FILL_ID,
            dups(h.border_fills.iter().map(|b| b.id)),
            "<hh:borderFill>",
        ),
        (
            jid::INTEGRITY_DUPLICATE_STYLE_ID,
            dups(h.styles.iter().map(|s| s.id)),
            "<hh:style>",
        ),
        (
            // FaceName ids restart per language block in the HWPX spec;
            // our parser flattens all blocks into one Vec. A duplicate
            // here means two fonts share id WITHIN the same (lang, id)
            // pair, which IS a bug. Keying by (lang, id) to avoid false
            // positives from two languages both numbering from 0.
            jid::INTEGRITY_DUPLICATE_FACE_NAME_ID,
            {
                let mut counts: BTreeMap<(String, u32), usize> = BTreeMap::new();
                for f in &h.face_names {
                    *counts.entry((f.lang.clone(), f.id)).or_insert(0) += 1;
                }
                counts
                    .into_iter()
                    .filter_map(|((_lang, id), n)| if n > 1 { Some(id) } else { None })
                    .collect()
            },
            "<hh:font> (within a fontface language block)",
        ),
        (
            jid::INTEGRITY_DUPLICATE_NUMBERING_ID,
            dups(h.numberings.iter().map(|n| n.id)),
            "<hh:numbering>",
        ),
        (
            jid::INTEGRITY_DUPLICATE_BULLET_ID,
            dups(h.bullets.iter().map(|b| b.id)),
            "<hh:bullet>",
        ),
    ];

    for (code, dup_ids, label) in checks {
        for id in dup_ids {
            // All header tables (charPr / paraPr / borderFill / style /
            // numbering / bullet / face) live inside `<hh:head>` — so
            // click-to-locate lands the user on header.xml.
            let mut v = integrity_violation(ctx, code, format!("duplicate {label} id={id}"));
            v.file_label = "header.xml".to_string();
            if !ctx.push(v) {
                return false;
            }
        }
    }
    true
}

/// Phase 1 — body elements that carry a `borderFillIDRef` must point
/// at a declared `<hh:borderFill>`. The HWPX schema lets paraPr, charPr,
/// table, tableCell, and page all reference borderFills; our parser
/// currently captures this only on `<hp:tbl>` (Table), so we check
/// tables now and keep the paraPr/charPr JIDs (11041/11042) reserved
/// for when the parser surfaces those fields.
fn check_integrity_border_fill_refs(ctx: &mut Ctx) -> bool {
    let h = &ctx.doc.header;
    use std::collections::HashSet;
    let known: HashSet<u32> = h.border_fills.iter().map(|b| b.id).collect();

    let mut orphans: Vec<(usize, u32, u32)> = Vec::new(); // (section_idx, table_id, bf_idref)
    for (si, section) in ctx.doc.sections.iter().enumerate() {
        for table in &section.tables {
            if table.border_fill_id_ref != 0 && !known.contains(&table.border_fill_id_ref) {
                orphans.push((si, table.id, table.border_fill_id_ref));
            }
        }
    }

    for (si, tid, bf) in orphans {
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_ORPHAN_BORDER_FILL_IDREF,
            format!(
                "table id={tid} references borderFillIDRef={bf} with no matching <hh:borderFill>"
            ),
        );
        // The dangling IDRef is on the `<hp:tbl>` element in the
        // section XML; the target record would be in header.xml.
        // Navigate to the section side so the user sees the bad ref
        // in context.
        v.file_label = format!("section{si}");
        if !ctx.push(v) {
            return false;
        }
    }
    true
}

/// Phase 3 — OWPML (KS X 6101) XSD conformance. Walks each raw XML
/// buffer captured by `open_bytes` against the corresponding schema
/// model in `polaris_dvc_schema`. Emits JID 13001-13007.
///
/// Scope: bootstrap subset of the OWPML schemas (top-20 % of element
/// names by occurrence frequency). Unknown elements pass through
/// without a violation — this matches the pragmatic "80 % coverage"
/// design goal. Schema completeness lands when `tools/gen-owpml/`
/// replaces the hand-curated `generated_owpml.rs`.
///
/// If the caller built `HwpxDocument` directly (no ZIP bytes), all
/// three raw-XML slices are empty and this pass is a no-op — same
/// graceful degradation pattern as `check_container`.
fn check_schema(ctx: &mut Ctx) -> bool {
    use polaris_dvc_schema::{validate_xml, OwpmlRoot, ViolationCode};

    let s = &ctx.doc.structural;
    let passes: [(OwpmlRoot, &[u8], &'static str); 3] = [
        (OwpmlRoot::ContentHpf, &s.content_hpf_bytes, "content.hpf"),
        (OwpmlRoot::Head, &s.header_xml_bytes, "header.xml"),
        // Sections handled in a loop below.
        (OwpmlRoot::Section, &[], ""),
    ];

    // Iterate the first two static passes.
    for (root, bytes, label) in passes.iter().take(2) {
        if bytes.is_empty() {
            continue;
        }
        let violations = validate_xml(bytes, *root);
        for v in violations {
            let code = map_schema_violation(v.code);
            let mut rec = integrity_violation(ctx, code, v.message);
            rec.file_label = (*label).to_string();
            rec.byte_offset = v.byte_offset as u32;
            if !ctx.push(rec) {
                return false;
            }
        }
    }

    // Section XML passes — one per section in manifest order.
    for (idx, bytes) in s.section_xml_bytes.iter().enumerate() {
        if bytes.is_empty() {
            continue;
        }
        let violations = validate_xml(bytes, OwpmlRoot::Section);
        for v in violations {
            let code = map_schema_violation(v.code);
            let mut rec = integrity_violation(ctx, code, v.message);
            rec.file_label = format!("section{idx}");
            rec.byte_offset = v.byte_offset as u32;
            if !ctx.push(rec) {
                return false;
            }
        }
    }

    // `ViolationCode → ErrorCode` mapping.
    fn map_schema_violation(c: ViolationCode) -> ErrorCode {
        match c {
            ViolationCode::UnexpectedChild => jid::SCHEMA_UNEXPECTED_CHILD,
            ViolationCode::MissingRequiredChild => jid::SCHEMA_MISSING_REQUIRED_CHILD,
            ViolationCode::TooManyOccurrences => jid::SCHEMA_TOO_MANY_OCCURRENCES,
            ViolationCode::MissingRequiredAttribute => jid::SCHEMA_MISSING_REQUIRED_ATTR,
            ViolationCode::UnknownAttribute => jid::SCHEMA_UNKNOWN_ATTR,
            ViolationCode::AttributeTypeMismatch => jid::SCHEMA_ATTR_TYPE_MISMATCH,
            ViolationCode::UnexpectedText => jid::SCHEMA_UNEXPECTED_TEXT,
        }
    }

    true
}

/// Phase 2 — ZIP container well-formedness. Runs before the Integrity
/// pass because a broken container can invalidate everything above it.
/// Emits JID 12001-12030 (Container category). All JIDs strict-gated.
///
/// If `StructuralFacts.zip_all_paths` is empty, the caller constructed
/// a synthetic `HwpxDocument` without going through `open_bytes` — we
/// have no ZIP-level observations to check. This path matters for unit
/// tests that build `HwpxDocument` directly; they're not testing the
/// container layer.
fn check_container(ctx: &mut Ctx) -> bool {
    use std::collections::HashSet;
    let s = &ctx.doc.structural;
    if s.zip_all_paths.is_empty() {
        return true;
    }
    let paths: HashSet<&str> = s.zip_all_paths.iter().map(String::as_str).collect();

    // 12001 — required HWPX entries. Three are load-bearing per spec.
    // Missing `mimetype` is already signalled via JID 11010-11012 (the
    // integrity pass treats it as a content-level defect); here we only
    // fire on the two genuinely-required manifest / container entries
    // whose absence blocks any subsequent XML work.
    for required in ["META-INF/container.xml", "Contents/content.hpf"] {
        if !paths.contains(required) {
            let v = integrity_violation(
                ctx,
                jid::CONTAINER_REQUIRED_ENTRY_MISSING,
                format!("required HWPX ZIP entry missing: {required}"),
            );
            if !ctx.push(v) {
                return false;
            }
        }
    }

    // 12010 — path-traversal / zip-slip defenses.
    for path in &s.zip_path_traversal {
        let v = integrity_violation(
            ctx,
            jid::CONTAINER_PATH_TRAVERSAL,
            format!("zip entry {path:?} contains path-traversal segment"),
        );
        if !ctx.push(v) {
            return false;
        }
    }

    // 12020 — cruft entries. Soft warning, one per entry.
    for path in &s.zip_cruft_entries {
        let v = integrity_violation(
            ctx,
            jid::CONTAINER_CRUFT_ENTRY,
            format!("zip contains editor/OS cruft entry: {path}"),
        );
        if !ctx.push(v) {
            return false;
        }
    }

    // 12030 — duplicate entries.
    for path in &s.zip_duplicate_entries {
        let v = integrity_violation(
            ctx,
            jid::CONTAINER_DUPLICATE_ENTRY,
            format!("zip contains duplicate entry name: {path}"),
        );
        if !ctx.push(v) {
            return false;
        }
    }

    true
}

/// Phase 1 — every `<hh:charPr><hh:fontRef>` sub-attribute must point
/// at a `<hh:font id=…>` that exists in the corresponding language
/// fontface block. We match by (lang, id) so "HANGUL id 0" and
/// "LATIN id 0" resolve independently.
fn check_integrity_font_refs(ctx: &mut Ctx) -> bool {
    let h = &ctx.doc.header;
    use std::collections::HashSet;

    // Build lookup: set of (lang, id) pairs present.
    let by_lang: HashSet<(String, u32)> = h
        .face_names
        .iter()
        .map(|f| (f.lang.clone(), f.id))
        .collect();

    let lang_of = |slot: &str| match slot {
        "hangul" => "HANGUL",
        "latin" => "LATIN",
        "hanja" => "HANJA",
        "japanese" => "JAPANESE",
        "other" => "OTHER",
        "symbol" => "SYMBOL",
        "user" => "USER",
        _ => "",
    };

    struct Miss {
        char_pr_id: u32,
        slot: &'static str,
        id: u32,
    }
    let mut misses: Vec<Miss> = Vec::new();

    for c in &h.char_shapes {
        let slots: [(&'static str, u32); 7] = [
            ("hangul", c.font_ref.hangul),
            ("latin", c.font_ref.latin),
            ("hanja", c.font_ref.hanja),
            ("japanese", c.font_ref.japanese),
            ("other", c.font_ref.other),
            ("symbol", c.font_ref.symbol),
            ("user", c.font_ref.user),
        ];
        for (slot, id) in slots {
            let lang = lang_of(slot);
            // If the document has no fontface entries for this language
            // at all, the IDRef has nothing to match; treat as orphan
            // only if the language block exists but the id is absent.
            let lang_has_any = h.face_names.iter().any(|f| f.lang == lang);
            if lang_has_any && !by_lang.contains(&(lang.to_string(), id)) {
                misses.push(Miss {
                    char_pr_id: c.id,
                    slot,
                    id,
                });
            }
        }
    }

    for m in misses {
        // Both sides of this ref live in header.xml (`<hh:charPr>`
        // and the `<hh:fontface>` blocks), so that's where the user
        // wants to land.
        let mut v = integrity_violation(
            ctx,
            jid::INTEGRITY_ORPHAN_FONT_REF,
            format!(
                "charPr id={} has fontRef {}={} with no matching <hh:font> in the {} block",
                m.char_pr_id,
                m.slot,
                m.id,
                lang_of(m.slot)
            ),
        );
        v.file_label = "header.xml".to_string();
        if !ctx.push(v) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use polaris_dvc_hwpx::{CharPr, FaceName, FontRef, Header, HwpxDocument, ParaPr};

    fn make_doc(char_pr: CharPr, para_pr: ParaPr, text: &str) -> HwpxDocument {
        let mut header = Header::default();
        header.face_names.push(FaceName {
            id: 0,
            lang: "HANGUL".into(),
            face: "바탕".into(),
        });
        header.char_shapes.push(char_pr);
        header.para_shapes.push(para_pr);
        let run = polaris_dvc_hwpx::Run {
            char_pr_id_ref: 0,
            text: text.into(),
            is_hyperlink: false,
            ..polaris_dvc_hwpx::Run::default()
        };
        let paragraph = polaris_dvc_hwpx::Paragraph {
            id: 0,
            para_pr_id_ref: 0,
            style_id_ref: 0,
            runs: vec![run],
            // One dummy lineseg keeps the integrity gate happy — these
            // synthetic test docs aren't exercising page/line tracking.
            line_segs: vec![polaris_dvc_hwpx::LineSeg::default()],
        };
        let section = polaris_dvc_hwpx::Section {
            paragraphs: vec![paragraph],
            tables: Vec::new(),
            binary_item_id_refs: Vec::new(),
        };
        HwpxDocument {
            mimetype: "application/hwp+zip".into(),
            header,
            sections: vec![section],
            structural: mimetype_ok_facts(),
        }
    }

    /// Minimal `StructuralFacts` that satisfies all mimetype integrity
    /// checks (11010/11011/11012). Used by synthetic test docs that
    /// aren't built from a real ZIP.
    fn mimetype_ok_facts() -> polaris_dvc_hwpx::StructuralFacts {
        polaris_dvc_hwpx::StructuralFacts {
            mimetype_is_first: true,
            mimetype_stored: true,
            ..Default::default()
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
        use polaris_dvc_hwpx::{LineSeg, Paragraph, Run, Section};

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
                ..Run::default()
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
                binary_item_id_refs: Vec::new(),
            }],
            structural: mimetype_ok_facts(),
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
        use polaris_dvc_hwpx::{LineSeg, Paragraph, Run, Section};

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
                ..Run::default()
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
                binary_item_id_refs: Vec::new(),
            }],
            structural: mimetype_ok_facts(),
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
        use polaris_dvc_hwpx::{Border, BorderFill, Section, Table};

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
            fill: polaris_dvc_hwpx::Fill::None,
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
                binary_item_id_refs: Vec::new(),
            }],
            structural: mimetype_ok_facts(),
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
        use polaris_dvc_hwpx::{Section, Table};

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
                binary_item_id_refs: Vec::new(),
            }],
            structural: mimetype_ok_facts(),
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
