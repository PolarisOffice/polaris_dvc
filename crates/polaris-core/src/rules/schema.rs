//! Rule spec types. Shape mirrors upstream `third_party/dvc-upstream/sample/
//! test.json` — the canonical DVC rule file. Notable upstream conventions:
//!
//! - `charshape.font` is an **allowlist array** (e.g., `["바탕", "돋움"]`),
//!   not a single value. Empty/missing means "any font ok".
//! - `style.permission`, `hyperlink.permission`, `macro.permission` are
//!   `true` to allow the feature; `false` means flag any usage.
//! - Scalars like `fontsize` / `linespacingvalue` are "must equal" checks.
//!
//! All fields use `#[serde(default)]` and an `extra` catch-all so specs
//! with unknown keys don't break loading.

use serde::{Deserialize, Serialize};

/// Field-like JSON value that may be a single value or an array of values.
/// Upstream uses this for `font` at least. Custom deserializer accepts both.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StringList(pub Vec<String>);

impl StringList {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn contains(&self, needle: &str) -> bool {
        self.0.iter().any(|s| s == needle)
    }
}

impl<'de> Deserialize<'de> for StringList {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{SeqAccess, Visitor};
        use std::fmt;

        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = StringList;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("string or array of strings")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<StringList, E> {
                Ok(StringList(vec![s.to_string()]))
            }
            fn visit_string<E: serde::de::Error>(self, s: String) -> Result<StringList, E> {
                Ok(StringList(vec![s]))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<StringList, A::Error> {
                let mut v = Vec::new();
                while let Some(s) = seq.next_element::<String>()? {
                    v.push(s);
                }
                Ok(StringList(v))
            }
        }
        d.deserialize_any(V)
    }
}

impl Serialize for StringList {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct RuleSpec {
    pub charshape: Option<CharShape>,
    pub parashape: Option<ParaShape>,
    pub table: Option<TableSpec>,
    pub style: Option<Permission>,
    pub hyperlink: Option<Permission>,
    #[serde(rename = "macro")]
    pub macro_: Option<Permission>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct CharShape {
    /// Language scope for per-language attributes. Upstream values include
    /// "대표", "한글", "영문", etc.
    pub langtype: Option<String>,
    /// Allowed fonts. Accepts either `"바탕"` or `["바탕", "돋움"]`.
    pub font: Option<StringList>,
    pub fontsize: Option<f64>,
    pub ratio: Option<f64>,
    pub spacing: Option<f64>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikeout: Option<bool>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct ParaShape {
    pub align: Option<String>,
    pub linespacing: Option<f64>,
    pub linespacingvalue: Option<f64>,
    #[serde(rename = "spacing-paraup")]
    pub spacing_paraup: Option<f64>,
    #[serde(rename = "spacing-parabottom")]
    pub spacing_parabottom: Option<f64>,
    pub indent: Option<f64>,
    pub outdent: Option<f64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct TableSpec {
    /// Per-side border rules. Positions 1..=4 conventionally map to
    /// top/bottom/left/right in upstream samples.
    pub border: Option<Vec<BorderRule>>,
    #[serde(rename = "treatAsChar")]
    pub treat_as_char: Option<bool>,
    #[serde(rename = "table-in-table")]
    pub table_in_table: Option<bool>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct BorderRule {
    pub position: Option<u32>,
    pub bordertype: Option<u32>,
    pub size: Option<f64>,
    pub color: Option<u32>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Feature-permission rule (`style`, `hyperlink`, `macro` in upstream).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct Permission {
    /// `true` → feature is permitted. `false` → flag any usage as a violation.
    pub permission: Option<bool>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_minimal_spec() {
        let src = r#"{
            "charshape": { "font": "바탕", "fontsize": 10, "bold": false },
            "parashape": { "linespacingvalue": 160 }
        }"#;
        let spec: RuleSpec = serde_json::from_str(src).unwrap();
        // Single-string font is accepted and normalized to a single-element list.
        assert_eq!(
            spec.charshape.as_ref().unwrap().font.as_ref().unwrap().0,
            vec!["바탕".to_string()]
        );
        assert_eq!(spec.charshape.as_ref().unwrap().fontsize, Some(10.0));
        assert_eq!(
            spec.parashape.as_ref().unwrap().linespacingvalue,
            Some(160.0)
        );
    }

    #[test]
    fn font_accepts_array_form_from_upstream_sample() {
        let src = r#"{
            "charshape": { "font": ["바탕", "돋움", "굴림"] }
        }"#;
        let spec: RuleSpec = serde_json::from_str(src).unwrap();
        let fonts = &spec.charshape.as_ref().unwrap().font.as_ref().unwrap().0;
        assert_eq!(fonts.len(), 3);
        assert!(fonts.iter().any(|s| s == "돋움"));
    }

    #[test]
    fn permission_sections_parse() {
        let src = r#"{
            "style":     { "permission": false },
            "hyperlink": { "permission": false },
            "macro":     { "permission": true }
        }"#;
        let spec: RuleSpec = serde_json::from_str(src).unwrap();
        assert_eq!(spec.style.as_ref().unwrap().permission, Some(false));
        assert_eq!(spec.hyperlink.as_ref().unwrap().permission, Some(false));
        assert_eq!(spec.macro_.as_ref().unwrap().permission, Some(true));
    }

    #[test]
    fn upstream_test_json_loads() {
        // Smoke test against the real upstream fixture.
        let bytes = include_bytes!("../../../../third_party/dvc-upstream/sample/test.json");
        let spec: RuleSpec = serde_json::from_slice(bytes).unwrap();
        let cs = spec.charshape.as_ref().unwrap();
        assert_eq!(cs.langtype.as_deref(), Some("대표"));
        let fonts = cs.font.as_ref().unwrap();
        assert!(fonts.contains("바탕"));
        assert!(fonts.contains("맑은 고딕"));
        assert_eq!(
            spec.parashape.as_ref().unwrap().linespacingvalue,
            Some(160.0)
        );
        let borders = spec.table.as_ref().unwrap().border.as_ref().unwrap();
        assert_eq!(borders.len(), 4);
        assert_eq!(spec.style.as_ref().unwrap().permission, Some(false));
    }

    #[test]
    fn unknown_keys_preserved_under_extra() {
        let src = r#"{
            "charshape": { "font": "바탕", "newprop": 42 },
            "future_category": { "k": "v" }
        }"#;
        let spec: RuleSpec = serde_json::from_str(src).unwrap();
        assert!(spec
            .charshape
            .as_ref()
            .unwrap()
            .extra
            .contains_key("newprop"));
        assert!(spec.extra.contains_key("future_category"));
    }
}
