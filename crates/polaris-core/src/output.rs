//! DVC-compatible per-violation output record.
//!
//! Field names and order mirror upstream `Source/DVCOutputJson.cpp` so that
//! consumers of DVC's JSON output can read ours without changes.

use serde::{Deserialize, Serialize};

use crate::error_codes::ErrorCode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ViolationRecord {
    #[serde(rename = "CharIDRef")]
    pub char_id_ref: Option<String>,
    #[serde(rename = "ParaPrIDRef")]
    pub para_pr_id_ref: Option<String>,
    pub page_no: u32,
    pub line_no: u32,
    pub error_code: ErrorCode,
    #[serde(rename = "errorText")]
    pub error_text: String,

    #[serde(rename = "TableID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_id: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub is_in_table: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub is_in_table_in_table: bool,
    #[serde(skip_serializing_if = "is_zero_u32")]
    #[serde(default)]
    pub table_row: u32,
    #[serde(skip_serializing_if = "is_zero_u32")]
    #[serde(default)]
    pub table_col: u32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub use_style: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub use_hyperlink: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub is_in_shape: bool,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

impl ViolationRecord {
    pub fn new(error_code: ErrorCode, page_no: u32, line_no: u32) -> Self {
        Self {
            char_id_ref: None,
            para_pr_id_ref: None,
            page_no,
            line_no,
            error_code,
            error_text: error_code.text().to_string(),
            table_id: None,
            is_in_table: false,
            is_in_table_in_table: false,
            table_row: 0,
            table_col: 0,
            use_style: false,
            use_hyperlink: false,
            is_in_shape: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_serialization_shape() {
        let rec = ViolationRecord {
            char_id_ref: Some("charId123".into()),
            para_pr_id_ref: Some("paraId456".into()),
            page_no: 1,
            line_no: 5,
            error_code: ErrorCode::new(1001),
            error_text: "Font size does not match specification".into(),
            table_id: None,
            is_in_table: false,
            is_in_table_in_table: false,
            table_row: 0,
            table_col: 0,
            use_style: false,
            use_hyperlink: false,
            is_in_shape: false,
        };
        let v: serde_json::Value = serde_json::to_value(&rec).unwrap();
        assert_eq!(v["CharIDRef"], "charId123");
        assert_eq!(v["ParaPrIDRef"], "paraId456");
        assert_eq!(v["PageNo"], 1);
        assert_eq!(v["LineNo"], 5);
        assert_eq!(v["ErrorCode"], 1001);
        assert_eq!(v["errorText"], "Font size does not match specification");
        // Default-false / default-zero fields are omitted, matching upstream.
        assert!(v.get("IsInTable").is_none());
        assert!(v.get("TableRow").is_none());
        assert!(v.get("UseStyle").is_none());
    }

    #[test]
    fn table_fields_emitted_when_set() {
        let mut rec = ViolationRecord::new(ErrorCode::new(3001), 2, 7);
        rec.is_in_table = true;
        rec.table_row = 3;
        rec.table_col = 4;
        rec.table_id = Some("tbl-7".into());
        let v: serde_json::Value = serde_json::to_value(&rec).unwrap();
        assert_eq!(v["IsInTable"], true);
        assert_eq!(v["TableRow"], 3);
        assert_eq!(v["TableCol"], 4);
        assert_eq!(v["TableID"], "tbl-7");
    }
}
