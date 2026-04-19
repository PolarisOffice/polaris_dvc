//! Data types for a parsed HWPX document.
//!
//! Scope is pragmatic: we model the subset that the DVC rule engine needs
//! (CharPr / ParaPr tables + paragraph/run structure with IDRefs). Fields
//! are added incrementally as more rule categories come online.

#[derive(Debug, Default, Clone, PartialEq)]
pub struct HwpxDocument {
    pub mimetype: String,
    pub header: Header,
    pub sections: Vec<Section>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Header {
    pub face_names: Vec<FaceName>,
    pub char_shapes: Vec<CharPr>,
    pub para_shapes: Vec<ParaPr>,
    pub border_fills: Vec<BorderFill>,
    pub styles: Vec<Style>,
    /// True when the document contains any macro asset (populated from the
    /// OPF manifest during `open_bytes` — the header XML itself doesn't
    /// carry this signal). See `container::Manifest::has_macro`.
    pub has_macro: bool,
}

/// `<hh:borderFill>` entry. The `id` is referenced by tables (and by
/// CharPr's `borderFillIDRef`); positional sub-borders drive DVC's
/// `table.border` rule via the LinePosition mapping (1=Top, 2=Bottom,
/// 3=Left, 4=Right) that upstream defines in `DVCInterface.h`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BorderFill {
    pub id: u32,
    pub left: Border,
    pub right: Border,
    pub top: Border,
    pub bottom: Border,
}

/// A single side of a `BorderFill`. `kind` is the upstream `LineShape`
/// enum name as a string (e.g., "SOLID", "DASH_DOT"). `width_mm` carries
/// the numeric part of HWPX's "0.12 mm"-style attribute. `color` is the
/// HWPX `#RRGGBB` string left verbatim so the engine can decode either to
/// a packed integer or to a CSS-style hex as needed.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Border {
    pub kind: String,
    pub width_mm: f64,
    pub color: String,
}

/// Font face registration (`<hh:font>`).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FaceName {
    pub id: u32,
    pub lang: String,
    pub face: String,
}

/// Character properties (`<hh:charPr>`). Measurements are in the native
/// HWPX units: `height` is 1/100 pt (so 10 pt → 1000).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CharPr {
    pub id: u32,
    pub height: u32,
    pub text_color: String,
    pub font_ref: FontRef,
    pub bold: bool,
    pub italic: bool,
    pub underline: Option<Underline>,
    pub strikeout: Option<Strikeout>,
    pub outline: bool,
    pub emboss: bool,
    pub engrave: bool,
    pub shadow: Option<Shadow>,
    pub supscript: bool,
    pub subscript: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FontRef {
    pub hangul: u32,
    pub latin: u32,
    pub hanja: u32,
    pub japanese: u32,
    pub other: u32,
    pub symbol: u32,
    pub user: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Underline {
    pub kind: String,
    pub shape: String,
    pub color: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Strikeout {
    pub shape: String,
    pub color: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Shadow {
    pub kind: String,
    pub color: String,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParaPr {
    pub id: u32,
    pub align_horizontal: String,
    pub line_spacing_type: String,
    pub line_spacing_value: f64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Style {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub para_pr_id_ref: u32,
    pub char_pr_id_ref: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Section {
    pub paragraphs: Vec<Paragraph>,
    /// Tables encountered anywhere under this section, flattened. Order is
    /// document order. Nested tables (table-in-table) expand inline.
    pub tables: Vec<Table>,
}

/// A `<hp:tbl>` occurrence. We collect just enough for DVC's table rules
/// today — the `border_fill_id_ref` drives border validation, and
/// `row_cnt` / `col_cnt` are recorded for future size checks.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Table {
    pub id: u32,
    pub border_fill_id_ref: u32,
    pub row_cnt: u32,
    pub col_cnt: u32,
    /// Nesting depth. 0 = top-level table, 1+ = table-in-table. Upstream
    /// `isInTableInTable` fires on depth ≥ 1, which drives the
    /// `table-in-table: false` spec rule.
    pub nesting_depth: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Paragraph {
    pub id: u32,
    pub para_pr_id_ref: u32,
    pub style_id_ref: u32,
    pub runs: Vec<Run>,
    /// `<hp:lineseg>` entries for this paragraph, in document order. The
    /// engine's page/line tracker (port of upstream `FindPageInfo`) reads
    /// `vert_pos`/`vert_size` to detect page breaks.
    pub line_segs: Vec<LineSeg>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LineSeg {
    pub text_pos: u32,
    pub vert_pos: i64,
    pub vert_size: i64,
    pub horz_pos: i64,
    pub horz_size: i64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Run {
    pub char_pr_id_ref: u32,
    /// Concatenated `<hp:t>` text from all run segments. Control objects
    /// (`<hp:ctrl>`) are skipped at this layer — they're routed to their
    /// own validators in higher layers.
    pub text: String,
    /// True when this run sits between a `<hp:fieldBegin type="HYPERLINK">`
    /// and the matching `<hp:fieldEnd>`. Mirrors upstream `RunTypeInfo::
    /// isHyperlink` which drives the hyperlink-permission rule.
    pub is_hyperlink: bool,
}

impl Header {
    pub fn char_shape(&self, id: u32) -> Option<&CharPr> {
        self.char_shapes.iter().find(|c| c.id == id)
    }
    pub fn para_shape(&self, id: u32) -> Option<&ParaPr> {
        self.para_shapes.iter().find(|p| p.id == id)
    }
    pub fn face_name(&self, id: u32, lang: &str) -> Option<&FaceName> {
        self.face_names
            .iter()
            .find(|f| f.id == id && f.lang == lang)
    }
    pub fn border_fill(&self, id: u32) -> Option<&BorderFill> {
        self.border_fills.iter().find(|b| b.id == id)
    }
}
