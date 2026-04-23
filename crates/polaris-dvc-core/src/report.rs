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

    /// Render the report as an XML document.
    ///
    /// **polaris extension** — upstream DVC's `-x` / `--format=xml` is
    /// unimplemented (`CommandParser.cpp` returns "NotYet"). This format
    /// therefore has no parity contract; it's offered as a convenience
    /// in `CheckProfile::Extended` and gated out of the strict mode at
    /// the CLI layer (`docs/parity-roadmap.md`).
    ///
    /// Schema: an outer `<violations>` wrapper with one
    /// `<violation …/>` per record. Attribute names / order / conditional
    /// inclusion match the JSON output exactly so the two formats are
    /// diff-equivalent when compared.
    pub fn to_xml_string(&self, option: OutputOption) -> String {
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str("<violations>\n");
        for v in &self.violations {
            v.append_xml(option, &mut out);
        }
        out.push_str("</violations>\n");
        out
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

    #[test]
    fn xml_empty_report_has_wrapper_only() {
        let r = Report::empty();
        let xml = r.to_xml_string(OutputOption::AllOption);
        assert_eq!(
            xml,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<violations>\n</violations>\n"
        );
    }

    #[test]
    fn xml_emits_all_fields_under_all_option() {
        let mut r = Report::empty();
        r.push(ViolationRecord {
            char_pr_id_ref: 0,
            para_pr_id_ref: 0,
            text: "안녕".into(),
            page_no: 1,
            line_no: 1,
            error_code: ErrorCode::new(1001),
            table_id: 0,
            is_in_table: false,
            is_in_table_in_table: false,
            table_row: 0,
            table_col: 0,
            use_style: false,
            use_hyperlink: false,
            is_in_shape: false,
            error_string: String::new(),
            file_label: String::new(),
            byte_offset: 0,
        });
        let xml = r.to_xml_string(OutputOption::AllOption);
        assert!(xml.contains("CharIDRef=\"0\""));
        assert!(xml.contains("errorText=\"안녕\""));
        assert!(xml.contains("ErrorCode=\"1001\""));
        assert!(xml.contains("TableID=\"0\""));
        assert!(xml.contains("UseStyle=\"false\""));
        assert!(xml.contains("IsInShape=\"false\""));
        assert!(xml.contains("UseHyperlink=\"false\""));
    }

    #[test]
    fn xml_escapes_special_chars_in_error_text() {
        let mut r = Report::empty();
        r.push(ViolationRecord {
            text: "<A&B\"C>".into(),
            page_no: 1,
            line_no: 1,
            error_code: ErrorCode::new(1001),
            ..ViolationRecord::default()
        });
        let xml = r.to_xml_string(OutputOption::Default);
        // Content escaped, no raw reserved chars inside the attribute.
        assert!(xml.contains("errorText=\"&lt;A&amp;B&quot;C&gt;\""));
    }

    #[test]
    fn xml_drops_empty_text_records_in_default_mode() {
        // Same drop rule as JSON — `to_xml_string` must not emit a
        // violation whose text is empty in non-table-family options.
        let mut r = Report::empty();
        r.push(ViolationRecord::new(ErrorCode::new(1001))); // text empty
        let xml = r.to_xml_string(OutputOption::Default);
        assert!(!xml.contains("<violation "));
    }

    #[test]
    fn xml_preserves_empty_text_records_under_table_option() {
        let mut r = Report::empty();
        r.push(ViolationRecord {
            text: String::new(),
            table_id: 42,
            is_in_table: true,
            page_no: 1,
            line_no: 1,
            error_code: ErrorCode::new(3001),
            ..ViolationRecord::default()
        });
        let xml = r.to_xml_string(OutputOption::Table);
        assert!(xml.contains("TableID=\"42\""));
        assert!(xml.contains("IsInTable=\"true\""));
    }
}
