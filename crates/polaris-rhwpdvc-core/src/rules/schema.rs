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

/// Numeric spec value that may be either a scalar (equality check) or a
/// `{ "min": X, "max": Y }` / `{ "min": X }` / `{ "max": Y }` object
/// (inclusive range check). Mirrors the shape in upstream's schema
/// documentation (`sample/jsonFullSpec.json`): `"fontsize": { "min":
/// number, "max": number }`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Range64 {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub exact: Option<f64>,
}

impl Range64 {
    pub fn from_exact(v: f64) -> Self {
        Self {
            exact: Some(v),
            ..Self::default()
        }
    }

    /// `true` iff `v` satisfies whichever form the spec used. Empty
    /// ranges (no exact, no min, no max) match everything — callers
    /// should gate with `.is_constrained()` before running the check.
    pub fn matches(&self, v: f64) -> bool {
        if let Some(e) = self.exact {
            return (v - e).abs() <= f64::EPSILON;
        }
        if let Some(lo) = self.min {
            if v < lo {
                return false;
            }
        }
        if let Some(hi) = self.max {
            if v > hi {
                return false;
            }
        }
        true
    }

    pub fn is_constrained(&self) -> bool {
        self.exact.is_some() || self.min.is_some() || self.max.is_some()
    }

    /// Human-readable representation, used in diagnostic strings so
    /// error messages show what the spec required.
    pub fn describe(&self) -> String {
        if let Some(e) = self.exact {
            return format!("{}", e);
        }
        match (self.min, self.max) {
            (Some(lo), Some(hi)) => format!("{}..={}", lo, hi),
            (Some(lo), None) => format!(">= {}", lo),
            (None, Some(hi)) => format!("<= {}", hi),
            (None, None) => "*".into(),
        }
    }
}

impl<'de> Deserialize<'de> for Range64 {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{Error, MapAccess, Visitor};
        use std::fmt;

        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Range64;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("number or {min, max} object")
            }
            fn visit_i64<E: Error>(self, v: i64) -> Result<Range64, E> {
                Ok(Range64::from_exact(v as f64))
            }
            fn visit_u64<E: Error>(self, v: u64) -> Result<Range64, E> {
                Ok(Range64::from_exact(v as f64))
            }
            fn visit_f64<E: Error>(self, v: f64) -> Result<Range64, E> {
                Ok(Range64::from_exact(v))
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Range64, M::Error> {
                let mut r = Range64::default();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "min" => r.min = Some(map.next_value()?),
                        "max" => r.max = Some(map.next_value()?),
                        // Tolerate upstream's `"type": "number"` shape
                        // from jsonFullSpec.json — it's a schema marker,
                        // not a constraint. Skip.
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(r)
            }
        }
        d.deserialize_any(V)
    }
}

impl Serialize for Range64 {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        if let Some(e) = self.exact {
            return s.serialize_f64(e);
        }
        let len = self.min.is_some() as usize + self.max.is_some() as usize;
        let mut m = s.serialize_map(Some(len))?;
        if let Some(lo) = self.min {
            m.serialize_entry("min", &lo)?;
        }
        if let Some(hi) = self.max {
            m.serialize_entry("max", &hi)?;
        }
        m.end()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct RuleSpec {
    pub charshape: Option<CharShape>,
    pub parashape: Option<ParaShape>,
    pub table: Option<TableSpec>,
    pub specialcharacter: Option<SpecialCharacter>,
    pub outlineshape: Option<OutlineShape>,
    pub bullet: Option<BulletSpec>,
    pub paranumbullet: Option<ParaNumBullet>,
    pub style: Option<Permission>,
    pub hyperlink: Option<Permission>,
    #[serde(rename = "macro")]
    pub macro_: Option<Permission>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// `outlineshape` / `paranumbullet` level table — same schema, two
/// separately-applied rule categories (outline uses section secPr's
/// outlineShapeIDRef-referenced numbering; paranumbullet applies to the
/// others).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct OutlineShape {
    pub leveltype: Option<Vec<LevelType>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct ParaNumBullet {
    pub leveltype: Option<Vec<LevelType>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct LevelType {
    pub level: Option<u32>,
    pub numbertype: Option<String>,
    pub numbershape: Option<u32>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// `bullet.bulletshapes` — the rule value is a string of permitted bullet
/// characters; any document bullet whose `char` is not contained fires
/// `JID_BULLET_SHAPES` (3304).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct BulletSpec {
    pub bulletshapes: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// `specialcharacter` rule — run text code points must fall within
/// `[minimum, maximum]` inclusive. Characters below `minimum` fire
/// `JID_SPECIALCHARACTER_MINIMUM` (3101); above `maximum` fires
/// `JID_SPECIALCHARACTER_MAXIMUM` (3102).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct SpecialCharacter {
    pub minimum: Option<u32>,
    pub maximum: Option<u32>,
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
    pub fontsize: Option<Range64>,
    pub ratio: Option<Range64>,
    pub spacing: Option<Range64>,
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
    pub linespacing: Option<Range64>,
    pub linespacingvalue: Option<Range64>,
    #[serde(rename = "spacing-paraup")]
    pub spacing_paraup: Option<Range64>,
    #[serde(rename = "spacing-parabottom")]
    pub spacing_parabottom: Option<Range64>,
    pub indent: Option<Range64>,
    pub outdent: Option<Range64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct TableSpec {
    /// Per-side border rules. Positions 1..=4 conventionally map to
    /// top/bottom/left/right in upstream samples.
    pub border: Option<Vec<BorderRule>>,
    pub size: Option<TableSizeSpec>,
    pub margin: Option<TableMarginSpec>,
    pub outside: Option<TableMarginSpec>,
    pub bgfill: Option<BgFillSpec>,
    pub caption: Option<TableCaptionSpec>,
    #[serde(rename = "treatAsChar")]
    pub treat_as_char: Option<bool>,
    #[serde(rename = "table-in-table")]
    pub table_in_table: Option<bool>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Background-fill rule. `type` is the upstream `BGFillType` ordinal
/// (0=NONE, 1=SOLID, 2=PATTERN, 3=GRADATION, 4=IMAGE). `facecolor`
/// and `pattoncolor` accept either a decimal/hex integer or a
/// `#RRGGBB` string for user convenience.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct BgFillSpec {
    #[serde(rename = "type")]
    pub kind: Option<u32>,
    pub facecolor: Option<ColorValue>,
    pub pattoncolor: Option<ColorValue>,
    pub pattontype: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Caption positioning/sizing rule for tables. Upstream
/// `JID_TABLE_CAPTION_*` (3026–3030). Caption text position is typically
/// one of `LEFT_TOP`, `TOP_CENTER`, `RIGHT_TOP`, `LEFT_MIDDLE`, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct TableCaptionSpec {
    pub position: Option<String>,
    pub size: Option<Range64>,
    pub spacing: Option<Range64>,
    #[serde(rename = "socapfullsize")]
    pub so_cap_full_size: Option<bool>,
    pub linewrap: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A color value — accepts plain integer (decimal), `#RRGGBB` /
/// `RRGGBB` string. Internal representation is `u32` (packed RGB), so
/// the engine can compare numerically against HWPX's `#RRGGBB` strings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColorValue(pub u32);

impl<'de> Deserialize<'de> for ColorValue {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{Error, Visitor};
        use std::fmt;

        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = ColorValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("integer color or #RRGGBB string")
            }
            fn visit_u64<E: Error>(self, v: u64) -> Result<ColorValue, E> {
                Ok(ColorValue(v as u32))
            }
            fn visit_i64<E: Error>(self, v: i64) -> Result<ColorValue, E> {
                Ok(ColorValue(v as u32))
            }
            fn visit_str<E: Error>(self, s: &str) -> Result<ColorValue, E> {
                let t = s.trim_start_matches('#');
                if let Ok(v) = u32::from_str_radix(t, 16) {
                    return Ok(ColorValue(v));
                }
                s.parse::<u32>()
                    .map(ColorValue)
                    .map_err(|_| E::custom(format!("color: cannot parse {s:?}")))
            }
        }
        d.deserialize_any(V)
    }
}

impl Serialize for ColorValue {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(self.0)
    }
}

/// Table width/height (in HWPUNIT). Both support ranges.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct TableSizeSpec {
    pub width: Option<Range64>,
    pub height: Option<Range64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Per-side table margin spec. Used for both `margin` (inner/cell
/// margin → `<hp:inMargin>`) and `outside` (outer margin →
/// `<hp:outside>`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "lowercase")]
pub struct TableMarginSpec {
    pub left: Option<Range64>,
    pub right: Option<Range64>,
    pub top: Option<Range64>,
    pub bottom: Option<Range64>,
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
        assert_eq!(
            spec.charshape
                .as_ref()
                .unwrap()
                .fontsize
                .as_ref()
                .unwrap()
                .exact,
            Some(10.0)
        );
        assert_eq!(
            spec.parashape
                .as_ref()
                .unwrap()
                .linespacingvalue
                .as_ref()
                .unwrap()
                .exact,
            Some(160.0)
        );
    }

    #[test]
    fn range_spec_min_max() {
        let src = r#"{
            "charshape": { "fontsize": { "min": 10, "max": 12 } },
            "parashape": { "linespacingvalue": { "min": 160 } }
        }"#;
        let spec: RuleSpec = serde_json::from_str(src).unwrap();
        let fs = spec.charshape.as_ref().unwrap().fontsize.as_ref().unwrap();
        assert_eq!(fs.min, Some(10.0));
        assert_eq!(fs.max, Some(12.0));
        assert!(fs.exact.is_none());
        assert!(fs.matches(10.0));
        assert!(fs.matches(11.5));
        assert!(fs.matches(12.0));
        assert!(!fs.matches(9.9));
        assert!(!fs.matches(12.1));

        let ls = spec
            .parashape
            .as_ref()
            .unwrap()
            .linespacingvalue
            .as_ref()
            .unwrap();
        assert_eq!(ls.min, Some(160.0));
        assert!(ls.max.is_none());
        assert!(ls.matches(160.0));
        assert!(ls.matches(200.0));
        assert!(!ls.matches(140.0));
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
            spec.parashape
                .as_ref()
                .unwrap()
                .linespacingvalue
                .as_ref()
                .unwrap()
                .exact,
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
