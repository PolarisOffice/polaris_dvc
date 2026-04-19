//! Rule spec types. Scope expands as more JID categories are implemented.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct RuleSpec {
    pub charshape: Option<CharShape>,
    pub parashape: Option<ParaShape>,
    pub table: Option<TableSpec>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct CharShape {
    pub font: Option<String>,
    pub fontsize: Option<f64>,
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
    pub linespacingvalue: Option<f64>,
    pub align: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct TableSpec {
    pub width: Option<f64>,
    pub height: Option<f64>,
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
        assert_eq!(
            spec.charshape.as_ref().unwrap().font.as_deref(),
            Some("바탕")
        );
        assert_eq!(spec.charshape.as_ref().unwrap().fontsize, Some(10.0));
        assert_eq!(
            spec.parashape.as_ref().unwrap().linespacingvalue,
            Some(160.0)
        );
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
