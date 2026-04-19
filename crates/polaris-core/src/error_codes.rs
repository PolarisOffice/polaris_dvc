//! DVC-compatible error code registry.
//!
//! Codes mirror the `JID_*` constants in upstream `Source/JsonModel.h`
//! (blocks 1000–10999). This module ships with a seed set; full extraction
//! will be driven by `tools/gen_jids.rs` against the vendored upstream.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        // Seed messages; expand via tools/gen_jids.rs from upstream JsonModel.h.
        match self.0 {
            1001 => "Font size does not match specification",
            1002 => "Font name does not match specification",
            1010 => "Bold setting does not match specification",
            1011 => "Italic setting does not match specification",
            2001 => "Paragraph alignment does not match specification",
            2050 => "Line spacing out of specification range",
            3001 => "Table width does not match specification",
            3002 => "Table border does not match specification",
            _ => "Rule violation",
        }
    }
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
