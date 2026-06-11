/// Synthetic table records in JSONL format (one JSON object per line).
#[derive(Debug, Clone)]
pub struct TableRecords {
    pub table_id: String,
    /// Each string is one JSONL line (one record).
    pub lines: Vec<String>,
}

/// Input for writing a complete backup package.
#[derive(Debug, Default)]
pub struct PackageInput {
    /// Serialized manifest JSON bytes.
    pub manifest_json: Vec<u8>,
    /// Serialized base metadata JSON bytes.
    pub base_json: Vec<u8>,
    /// Serialized schema JSON bytes.
    pub schema_json: Vec<u8>,
    /// Per-table record sets (JSONL).
    pub tables: Vec<TableRecords>,
    /// Attachment metadata JSONL bytes.
    pub attachment_metadata_jsonl: Vec<u8>,
    /// Linked-records JSONL bytes.
    pub linked_records_jsonl: Vec<u8>,
    /// Serialized backup report JSON bytes.
    pub backup_report_json: Vec<u8>,
    /// Serialized compatibility report JSON bytes.
    pub compatibility_report_json: Vec<u8>,
}

impl PackageInput {
    /// Returns true if all required non-empty fields are present.
    pub fn is_complete(&self) -> bool {
        !self.manifest_json.is_empty()
            && !self.base_json.is_empty()
            && !self.schema_json.is_empty()
            && !self.backup_report_json.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_input_incomplete_when_empty() {
        let input = PackageInput::default();
        assert!(!input.is_complete());
    }

    #[test]
    fn package_input_complete_with_required_fields() {
        let input = PackageInput {
            manifest_json: b"{\"format\":\"airbridge\"}".to_vec(),
            base_json: b"{\"baseId\":\"appSyn01\"}".to_vec(),
            schema_json: b"{\"tables\":[]}".to_vec(),
            backup_report_json: b"{\"status\":\"ok\"}".to_vec(),
            ..Default::default()
        };
        assert!(input.is_complete());
    }

    #[test]
    fn table_records_preserves_lines() {
        let tr = TableRecords {
            table_id: "tblSyn01".to_string(),
            lines: vec![
                r#"{"id":"rec001","fields":{"Name":"Alpha"}}"#.to_string(),
                r#"{"id":"rec002","fields":{"Name":"Beta"}}"#.to_string(),
            ],
        };
        assert_eq!(tr.lines.len(), 2);
        assert!(tr.lines[0].contains("Alpha"));
        assert!(tr.lines[1].contains("Beta"));
    }
}
