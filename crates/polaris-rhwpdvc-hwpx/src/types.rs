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
    /// Structural facts captured at parse time for integrity checks
    /// that aren't expressible through the normal rule spec — mimetype
    /// ZIP-level invariants, BinData cross-references, etc. See
    /// [`StructuralFacts`].
    pub structural: StructuralFacts,
}

/// ZIP-container and manifest-level observations the parser collects
/// on the way through `open_bytes`. Populated once per document.
/// Consumed by `engine::check_integrity` (JID 11010+ / 11020+).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct StructuralFacts {
    /// True when `mimetype` is the literal first entry in the ZIP
    /// central directory (HWPX / OCF spec requires this).
    pub mimetype_is_first: bool,
    /// True when the mimetype entry uses `Stored` compression (must
    /// be uncompressed per spec). False for any other method.
    pub mimetype_stored: bool,
    /// All ZIP paths under `BinData/` — every binary asset actually
    /// present in the archive.
    pub zip_bindata_paths: Vec<String>,
    /// Manifest items that reference binaries. Each entry is
    /// `(opf:item@id, opf:item@href)`. Populated from
    /// `Contents/content.hpf`.
    pub manifest_bindata_items: Vec<(String, String)>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Header {
    pub face_names: Vec<FaceName>,
    pub char_shapes: Vec<CharPr>,
    pub para_shapes: Vec<ParaPr>,
    pub border_fills: Vec<BorderFill>,
    pub styles: Vec<Style>,
    pub numberings: Vec<Numbering>,
    pub bullets: Vec<Bullet>,
    /// True when the document contains any macro asset (populated from the
    /// OPF manifest during `open_bytes` — the header XML itself doesn't
    /// carry this signal). See `container::Manifest::has_macro`.
    pub has_macro: bool,
}

/// `<hh:numbering>` entry — drives both outlineshape and paranumbullet
/// rule categories. A `Numbering` holds per-level formatting (`paraHead`
/// entries) that the engine compares against the spec's `leveltype`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Numbering {
    pub id: u32,
    pub start: u32,
    pub para_heads: Vec<ParaHead>,
}

/// `<hh:paraHead>` — one row in the level table. `num_format` carries the
/// HWPX numFormat string (e.g., "^1.", "(^5)"); `number_shape` corresponds
/// to upstream `GetNumShape` (enum value).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParaHead {
    pub level: u32,
    pub start: u32,
    pub num_format: String,
    pub number_shape: u32,
}

/// `<hh:bullet>` entry.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Bullet {
    pub id: u32,
    /// The bullet character, as stored in the `char` attribute of
    /// `<hh:bullet>`. Typically a single Unicode scalar.
    pub char_: String,
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
    /// Populated from `<hh:fillBrush>`/`<hh:winBrush>`/`<hh:gradation>`
    /// sub-elements of `<hh:borderFill>`. Drives DVC's `table.bgfill`
    /// rule — tables reference a borderFill by id, and the borderFill
    /// carries both the line info and the fill info.
    pub fill: Fill,
}

/// Background fill variant. Upstream `BGFillType` enum: NONE=0, SOLID=1,
/// PATTERN=2, GRADATION=3, IMAGE=4. We distinguish SOLID vs PATTERN by
/// looking at `<hh:winBrush>`'s `hatchStyle` attribute — non-NONE means
/// PATTERN.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Fill {
    #[default]
    None,
    /// Plain or pattern fill (`<hh:winBrush>`).
    Brush(FillBrush),
    /// Linear/radial gradient (`<hh:gradation>`).
    Gradation(FillGradation),
    /// Image/picture fill (`<hh:imgBrush>`).
    Image,
}

impl Fill {
    /// Upstream `BGFillType` ordinal. Pattern vs solid is distinguished
    /// by `hatchStyle != "NONE"`.
    pub fn ordinal(&self) -> u32 {
        match self {
            Fill::None => 0,
            Fill::Brush(b) => {
                if b.hatch_style.is_empty() || b.hatch_style.eq_ignore_ascii_case("NONE") {
                    1 // SOLID
                } else {
                    2 // PATTERN
                }
            }
            Fill::Gradation(_) => 3,
            Fill::Image => 4,
        }
    }

    pub fn face_color_hex(&self) -> Option<&str> {
        match self {
            Fill::Brush(b) => Some(b.face_color.as_str()),
            _ => None,
        }
    }

    pub fn patton_color_hex(&self) -> Option<&str> {
        match self {
            Fill::Brush(b) => Some(b.hatch_color.as_str()),
            _ => None,
        }
    }

    pub fn patton_type(&self) -> Option<&str> {
        match self {
            Fill::Brush(b) => Some(b.hatch_style.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FillBrush {
    pub face_color: String,
    pub hatch_color: String,
    pub hatch_style: String,
    pub alpha: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FillGradation {
    pub kind: String,
    pub angle: i32,
    pub center_x: i32,
    pub center_y: i32,
    pub colors: Vec<String>,
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
    /// `<hh:ratio hangul="…">` — percentage width scaling per language.
    /// Only the Hangul component is stored; DVC's `charshape.ratio` rule
    /// uses the representative language's value.
    pub ratio_hangul: f64,
    /// `<hh:spacing hangul="…">` — extra inter-character spacing.
    pub spacing_hangul: f64,
    /// `<hh:relSz hangul="…">` — relative size (percent). Upstream compares
    /// this against `charshape.r-size` (`JID_CHAR_SHAPE_RSIZE`).
    pub rel_sz_hangul: f64,
    /// `useKerning="0|1"` attribute on `<hh:charPr>`. Upstream compares
    /// this against `charshape.kerning` (`JID_CHAR_SHAPE_KERNING`).
    pub use_kerning: bool,
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
    /// `<hh:margin><hc:prev value=…></hh:margin>` — spacing above the
    /// paragraph (`spacing-paraup` in DVC rule specs).
    pub margin_prev: f64,
    /// `<hh:margin><hc:next value=…></hh:margin>` —
    /// spacing below (`spacing-parabottom`).
    pub margin_next: f64,
    /// `<hh:margin><hc:intent value=…></hh:margin>` — first-line indent
    /// when positive, outdent when negative. Drives `indent` / `outdent`.
    pub margin_intent: f64,
    /// `<hh:margin><hc:left value=…></hh:margin>` — body indent.
    pub margin_left: f64,
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
    /// Every `binaryItemIDRef` value seen on picture/image/shape objects
    /// inside this section. Feeds integrity check JID 11020.
    pub binary_item_id_refs: Vec<String>,
}

/// A `<hp:tbl>` occurrence. We collect every sub-element the DVC rule
/// categories care about.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Table {
    pub id: u32,
    pub border_fill_id_ref: u32,
    pub row_cnt: u32,
    pub col_cnt: u32,
    pub cell_spacing: i64,
    /// Nesting depth. 0 = top-level table, 1+ = table-in-table. Upstream
    /// `isInTableInTable` fires on depth ≥ 1, which drives the
    /// `table-in-table: false` spec rule.
    pub nesting_depth: u32,
    /// Populated from `<hp:sz width="..." height="...">`.
    pub sz: TableSz,
    /// Populated from `<hp:pos treatAsChar="..." ...>`.
    pub pos: TablePos,
    /// Populated from `<hp:outside left="..." right="..." top="..." bottom="...">`.
    pub outside: TableEdges,
    /// Populated from `<hp:inMargin left="..." right="..." top="..." bottom="...">`
    /// — the per-table inner margin ("margin" in DVC spec lingo).
    pub in_margin: TableEdges,
    /// `textWrap` attribute on `<hp:tbl>` (OWPML enum string like
    /// `"SQUARE"` / `"TIGHT"` / `"TOP_AND_BOTTOM"` / `"BEHIND_TEXT"` /
    /// `"IN_FRONT_OF_TEXT"`). Drives DVC's `table.pos` rule
    /// (`JID_TABLE_POS`).
    pub text_wrap: String,
    /// `textFlow` attribute on `<hp:tbl>` (OWPML enum string like
    /// `"BOTH_SIDES"` / `"LEFT_ONLY"` / `"RIGHT_ONLY"` / `"LARGEST_ONLY"`).
    /// Drives DVC's `table.textpos` rule (`JID_TABLE_TEXTPOS`).
    pub text_flow: String,
    /// `lock="0|1"` attribute on `<hp:tbl>` — fixed-size flag. Drives
    /// DVC's `table.fixed` rule (`JID_TABLE_SIZEFIXED`).
    pub lock: bool,
}

/// `<hp:sz>` (table size). `width_rel_to` / `height_rel_to` are upstream
/// enum strings like `ABSOLUTE` / `PAGE` / `PARA`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TableSz {
    pub width: i64,
    pub width_rel_to: String,
    pub height: i64,
    pub height_rel_to: String,
    pub protect: bool,
}

/// `<hp:pos>` (table positioning). `treat_as_char` drives the DVC
/// `table.treatAsChar` rule.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TablePos {
    pub treat_as_char: bool,
    pub affect_l_spacing: bool,
    pub flow_with_text: bool,
    pub allow_overlap: bool,
    pub hold_anchor_and_so: bool,
    pub vert_rel_to: String,
    pub horz_rel_to: String,
    pub vert_align: String,
    pub horz_align: String,
    pub vert_offset: i64,
    pub horz_offset: i64,
}

/// Shared four-side record for `<hp:outside>` / `<hp:inMargin>`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TableEdges {
    pub left: i64,
    pub right: i64,
    pub top: i64,
    pub bottom: i64,
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
    /// True when this run is nested inside a `<hp:shapeObject>` / `<hp:pic>`
    /// / `<hp:drawing>` or any container that upstream classifies as a
    /// "shape" scope. Surfaces to the output as the `IsInShape` field on
    /// each violation.
    pub is_in_shape: bool,
    /// True when the run is inside a `<hp:footnote>` sub-list.
    pub is_in_footnote: bool,
    /// True when the run is inside a `<hp:endnote>` sub-list.
    pub is_in_endnote: bool,
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
