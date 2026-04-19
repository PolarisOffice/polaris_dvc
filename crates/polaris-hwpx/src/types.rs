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
    pub styles: Vec<Style>,
    pub has_macro: bool,
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
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Paragraph {
    pub id: u32,
    pub para_pr_id_ref: u32,
    pub style_id_ref: u32,
    pub runs: Vec<Run>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Run {
    pub char_pr_id_ref: u32,
    /// Concatenated `<hp:t>` text from all run segments. Control objects
    /// (`<hp:ctrl>`) are skipped at this layer — they're routed to their
    /// own validators in higher layers.
    pub text: String,
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
}
