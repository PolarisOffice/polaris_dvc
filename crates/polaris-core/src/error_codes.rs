//! DVC-compatible error code registry.
//!
//! Codes mirror the `JID_*` constants in upstream `Source/JsonModel.h`
//! (blocks 1000–10999). This module ships with a seed set; full extraction
//! will be driven by `tools/gen_jids.rs` against the vendored upstream.

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub const fn new(code: u32) -> Self {
        Self(code)
    }

    pub const fn value(self) -> u32 {
        self.0
    }

    pub fn category(self) -> Category {
        match self.0 {
            1000..=1999 => Category::CharShape,
            2000..=2999 => Category::ParaShape,
            3000..=3099 => Category::Table,
            3100..=3999 => Category::Element,
            4000..=4999 => Category::Style,
            5000..=5999 => Category::Page,
            6000..=6999 => Category::Reference,
            7000..=10999 => Category::Extended,
            _ => Category::Unknown,
        }
    }

    pub fn text(self) -> &'static str {
        // Seed messages keyed by JID values from upstream `Source/JsonModel.h`.
        // Expand via `tools/gen_jids.rs` against the vendored snapshot.
        match self.0 {
            1001 => "Font size does not match specification",
            1004 => "Font name does not match specification",
            1009 => "Bold setting does not match specification",
            1010 => "Italic setting does not match specification",
            1011 => "Underline setting does not match specification",
            1012 => "Strikeout setting does not match specification",
            2001 => "Paragraph alignment does not match specification",
            2007 => "Line spacing type does not match specification",
            2008 => "Line spacing value does not match specification",
            2009 => "Paragraph top spacing does not match specification",
            2010 => "Paragraph bottom spacing does not match specification",
            3101 => "Character code point below specification minimum",
            3102 => "Character code point above specification maximum",
            3206 => "Outline numbering type does not match specification",
            3207 => "Outline numbering shape does not match specification",
            3304 => "Bullet character not in allowed set",
            3406 => "Paragraph numbering type does not match specification",
            3407 => "Paragraph numbering shape does not match specification",
            3502 => "Style usage is not permitted",
            6901 => "Hyperlink usage is not permitted",
            7001 => "Macro usage is not permitted",
            3001 => "Table width does not match specification",
            3002 => "Table border does not match specification",
            _ => "Rule violation",
        }
    }
}

/// Canonical errorCode constants covered by the initial engine. Names and
/// integer values mirror `JID_*` in upstream `Source/JsonModel.h`.
pub mod jid {
    use super::ErrorCode;
    pub const CHAR_SHAPE_FONTSIZE: ErrorCode = ErrorCode::new(1001);
    pub const CHAR_SHAPE_LANGSET: ErrorCode = ErrorCode::new(1002);
    pub const CHAR_SHAPE_FONT: ErrorCode = ErrorCode::new(1004);
    pub const CHAR_SHAPE_RATIO: ErrorCode = ErrorCode::new(1007);
    pub const CHAR_SHAPE_SPACING: ErrorCode = ErrorCode::new(1008);
    pub const CHAR_SHAPE_BOLD: ErrorCode = ErrorCode::new(1009);
    pub const CHAR_SHAPE_ITALIC: ErrorCode = ErrorCode::new(1010);
    pub const CHAR_SHAPE_UNDERLINE: ErrorCode = ErrorCode::new(1011);
    pub const CHAR_SHAPE_STRIKEOUT: ErrorCode = ErrorCode::new(1012);
    pub const PARA_SHAPE_ALIGN: ErrorCode = ErrorCode::new(2001);
    pub const PARA_SHAPE_INDENT: ErrorCode = ErrorCode::new(2005);
    pub const PARA_SHAPE_OUTDENT: ErrorCode = ErrorCode::new(2006);
    pub const PARA_SHAPE_LINESPACING_TYPE: ErrorCode = ErrorCode::new(2007);
    pub const PARA_SHAPE_LINESPACINGVALUE: ErrorCode = ErrorCode::new(2008);
    pub const PARA_SHAPE_SPACING_PARAUP: ErrorCode = ErrorCode::new(2009);
    pub const PARA_SHAPE_SPACING_PARABOTTOM: ErrorCode = ErrorCode::new(2010);
    pub const STYLE_PERMISSION: ErrorCode = ErrorCode::new(3502);
    pub const HYPERLINK_PERMISSION: ErrorCode = ErrorCode::new(6901);
    pub const MACRO_PERMISSION: ErrorCode = ErrorCode::new(7001);
    pub const TABLE_BORDER: ErrorCode = ErrorCode::new(3031);
    pub const TABLE_BORDER_TYPE: ErrorCode = ErrorCode::new(3033);
    pub const TABLE_BORDER_SIZE: ErrorCode = ErrorCode::new(3034);
    pub const TABLE_BORDER_COLOR: ErrorCode = ErrorCode::new(3035);
    pub const TABLE_TREAT_AS_CHAR: ErrorCode = ErrorCode::new(3004);
    pub const TABLE_IN_TABLE: ErrorCode = ErrorCode::new(3056);
    pub const SPECIAL_CHAR_MINIMUM: ErrorCode = ErrorCode::new(3101);
    pub const SPECIAL_CHAR_MAXIMUM: ErrorCode = ErrorCode::new(3102);
    pub const OUTLINESHAPE_NUMBERTYPE: ErrorCode = ErrorCode::new(3206);
    pub const OUTLINESHAPE_NUMBERSHAPE: ErrorCode = ErrorCode::new(3207);
    pub const BULLET_SHAPES: ErrorCode = ErrorCode::new(3304);
    pub const PARANUMBULLET_NUMBERTYPE: ErrorCode = ErrorCode::new(3406);
    pub const PARANUMBULLET_NUMBERSHAPE: ErrorCode = ErrorCode::new(3407);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    CharShape,
    ParaShape,
    Table,
    Element,
    Style,
    Page,
    Reference,
    Extended,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_boundaries() {
        assert_eq!(ErrorCode::new(1000).category(), Category::CharShape);
        assert_eq!(ErrorCode::new(2500).category(), Category::ParaShape);
        assert_eq!(ErrorCode::new(3050).category(), Category::Table);
        assert_eq!(ErrorCode::new(3500).category(), Category::Element);
        assert_eq!(ErrorCode::new(10999).category(), Category::Extended);
        assert_eq!(ErrorCode::new(999).category(), Category::Unknown);
    }

    #[test]
    fn serde_is_transparent_u32() {
        let code = ErrorCode::new(1001);
        let s = serde_json::to_string(&code).unwrap();
        assert_eq!(s, "1001");
        let back: ErrorCode = serde_json::from_str("1001").unwrap();
        assert_eq!(back, code);
    }
}
