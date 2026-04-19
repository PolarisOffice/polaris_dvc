//! Top-level validation report. The DVC-compatible wire format is a bare
//! JSON array; [`Report::to_json_value`] produces that. The struct itself
//! carries some additional diagnostic context for our own consumers.

use serde_json::Value;

use crate::output::{OutputOption, ViolationRecord};

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub source: Option<String>,
    pub spec: Option<String>,
    pub violations: Vec<ViolationRecord>,
    pub stopped_early: bool,
}

impl Report {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn push(&mut self, v: ViolationRecord) {
        self.violations.push(v);
    }

    /// Render the DVC-compatible JSON-array payload for the given option.
    /// Matches upstream `DVCOutputJson::makeJsonBuffer`: records are emitted
    /// in insertion order; empty-text records are dropped in non-table modes.
    pub fn to_json_value(&self, option: OutputOption) -> Value {
        let items: Vec<Value> = self
            .violations
            .iter()
            .map(|v| v.to_json_value(option))
            .filter(|v| !v.is_null())
            .collect();
        Value::Array(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_codes::ErrorCode;

    #[test]
    fn renders_as_array() {
        let mut r = Report::empty();
        r.push(ViolationRecord {
            char_pr_id_ref: 1,
            text: "t".into(),
            page_no: 1,
            line_no: 1,
            error_code: ErrorCode::new(1001),
            ..ViolationRecord::default()
        });
        let v = r.to_json_value(OutputOption::Default);
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 1);
    }

    #[test]
    fn empty_text_records_dropped_in_default_mode() {
        let mut r = Report::empty();
        r.push(ViolationRecord::new(ErrorCode::new(1001))); // text empty
        let v = r.to_json_value(OutputOption::Default);
        assert_eq!(v.as_array().unwrap().len(), 0);
    }
}
