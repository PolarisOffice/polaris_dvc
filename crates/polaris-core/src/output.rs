//! DVC-compatible per-violation output record.
//!
//! Field names and conditional inclusion mirror upstream
//! `Source/DVCOutputJson.cpp::makeJsonBuffer`. Reference types come from
//! `Source/DVCErrorInfo.h`: CharIDRef / ParaPrIDRef / TableID are UINT,
//! and `errorText` carries the document text at the violation site — not
//! a human-readable error message.

use serde::{Deserialize, Serialize};

use crate::error_codes::ErrorCode;

/// Output option controlling which conditional fields are emitted.
/// Mirrors upstream `DVCOutputOption`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputOption {
    #[default]
    Default,
    Table,
    TableDetail,
    Style,
    Shape,
    Hyperlink,
    AllOption,
}

impl OutputOption {
    fn include_table(self) -> bool {
        matches!(self, Self::Table | Self::TableDetail | Self::AllOption)
    }
    fn include_style(self) -> bool {
        matches!(self, Self::Style | Self::AllOption)
    }
    fn include_shape(self) -> bool {
        matches!(self, Self::Shape | Self::AllOption)
    }
    fn include_hyperlink(self) -> bool {
        matches!(self, Self::Hyperlink | Self::AllOption)
    }
}

/// A single rule-violation record. Field *storage* is unconditional; the
/// output option controls which fields are *emitted* in serialized JSON via
/// [`Self::to_json_value`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ViolationRecord {
    pub char_pr_id_ref: u32,
    pub para_pr_id_ref: u32,
    /// Document text at the violation site. Serialized as `errorText`.
    pub text: String,
    pub page_no: u32,
    pub line_no: u32,
    pub error_code: ErrorCode,

    pub table_id: u32,
    pub is_in_table: bool,
    pub is_in_table_in_table: bool,
    pub table_row: u32,
    pub table_col: u32,
    pub use_style: bool,
    pub use_hyperlink: bool,
    pub is_in_shape: bool,

    /// Developer-oriented diagnostic string. Not included in DVC-compatible
    /// output (upstream `getErrorString` is internal-only).
    #[serde(default)]
    pub error_string: String,
}

impl ViolationRecord {
    pub fn new(error_code: ErrorCode) -> Self {
        Self {
            error_code,
            ..Self::default()
        }
    }

    /// Serialize into a `serde_json::Value` that matches the layout produced
    /// by upstream `DVCOutputJson::makeJsonBuffer` for the given option.
    ///
    /// Mirrors these upstream behaviors:
    /// - Always-emitted fields: `CharIDRef`, `ParaPrIDRef`, `errorText`,
    ///   `PageNo`, `LineNo`, `ErrorCode`.
    /// - Conditional fields per `OutputOption`.
    /// - Empty-text records are dropped (returns `Value::Null`) unless a
    ///   table-family option is active.
    pub fn to_json_value(&self, opt: OutputOption) -> serde_json::Value {
        use serde_json::json;

        if self.text.is_empty() && !opt.include_table() {
            return serde_json::Value::Null;
        }

        let mut v = json!({
            "CharIDRef": self.char_pr_id_ref,
            "ParaPrIDRef": self.para_pr_id_ref,
            "errorText": self.text,
            "PageNo": self.page_no,
            "LineNo": self.line_no,
            "ErrorCode": self.error_code.value(),
        });
        let obj = v.as_object_mut().unwrap();

        if opt.include_table() {
            obj.insert("TableID".into(), json!(self.table_id));
            obj.insert("IsInTable".into(), json!(self.is_in_table));
            obj.insert("IsInTableInTable".into(), json!(self.is_in_table_in_table));
            obj.insert("TableRow".into(), json!(self.table_row));
            obj.insert("TableCol".into(), json!(self.table_col));
        }
        if opt.include_style() {
            obj.insert("UseStyle".into(), json!(self.use_style));
        }
        if opt.include_shape() {
            obj.insert("IsInShape".into(), json!(self.is_in_shape));
        }
        if opt.include_hyperlink() {
            obj.insert("UseHyperlink".into(), json!(self.use_hyperlink));
        }

        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_with_text() -> ViolationRecord {
        ViolationRecord {
            char_pr_id_ref: 7,
            para_pr_id_ref: 3,
            text: "hello".into(),
            page_no: 1,
            line_no: 5,
            error_code: ErrorCode::new(1001),
            table_id: 11,
            is_in_table: true,
            is_in_table_in_table: false,
            table_row: 2,
            table_col: 4,
            use_style: true,
            use_hyperlink: true,
            is_in_shape: true,
            error_string: String::new(),
        }
    }

    #[test]
    fn default_option_emits_base_fields_only() {
        let v = sample_with_text().to_json_value(OutputOption::Default);
        assert_eq!(v["CharIDRef"], 7);
        assert_eq!(v["ParaPrIDRef"], 3);
        assert_eq!(v["errorText"], "hello");
        assert_eq!(v["PageNo"], 1);
        assert_eq!(v["LineNo"], 5);
        assert_eq!(v["ErrorCode"], 1001);
        assert!(v.get("IsInTable").is_none());
        assert!(v.get("UseStyle").is_none());
        assert!(v.get("IsInShape").is_none());
        assert!(v.get("UseHyperlink").is_none());
    }

    #[test]
    fn table_option_adds_table_fields() {
        let v = sample_with_text().to_json_value(OutputOption::Table);
        assert_eq!(v["TableID"], 11);
        assert_eq!(v["IsInTable"], true);
        assert_eq!(v["IsInTableInTable"], false);
        assert_eq!(v["TableRow"], 2);
        assert_eq!(v["TableCol"], 4);
    }

    #[test]
    fn all_option_emits_everything() {
        let v = sample_with_text().to_json_value(OutputOption::AllOption);
        for key in [
            "TableID",
            "IsInTable",
            "IsInTableInTable",
            "TableRow",
            "TableCol",
            "UseStyle",
            "IsInShape",
            "UseHyperlink",
        ] {
            assert!(v.get(key).is_some(), "missing key: {key}");
        }
    }

    #[test]
    fn empty_text_drops_record_in_default_option() {
        let mut r = sample_with_text();
        r.text.clear();
        assert!(r.to_json_value(OutputOption::Default).is_null());
    }

    #[test]
    fn empty_text_kept_when_table_option_active() {
        let mut r = sample_with_text();
        r.text.clear();
        let v = r.to_json_value(OutputOption::Table);
        assert!(v.is_object());
        assert_eq!(v["errorText"], "");
    }
}
