use serde::{Deserialize, Serialize};

use crate::backup::package::{PackageInput, TableRecords};

/// Exported records and extracted metadata for a single table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableExportResult {
    pub table_id: String,
    pub table_name: String,
    /// JSONL lines — one per record.
    pub jsonl_lines: Vec<String>,
    pub record_count: usize,
    pub pages_fetched: usize,
}

/// Full result returned by the record export engine for one base.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordExportEngineResult {
    pub base_id: String,
    pub base_name: String,
    pub tables: Vec<TableExportResult>,
    /// Serialised linked-record JSONL bytes (all tables merged).
    pub linked_records_jsonl: Vec<u8>,
    /// Serialised attachment-metadata JSONL bytes (all tables merged).
    pub attachment_metadata_jsonl: Vec<u8>,
}

impl RecordExportEngineResult {
    /// Total records across all tables.
    pub fn total_records(&self) -> usize {
        self.tables.iter().map(|t| t.record_count).sum()
    }
}

/// Build a minimal `PackageInput` from an engine result.
///
/// Callers must supply pre-serialised `manifest_json`, `base_json`,
/// `schema_json`, and `backup_report_json` — the engine result fills in
/// the record-data fields only.
pub fn build_package_input(
    result: &RecordExportEngineResult,
    manifest_json: Vec<u8>,
    base_json: Vec<u8>,
    schema_json: Vec<u8>,
    backup_report_json: Vec<u8>,
    compatibility_report_json: Vec<u8>,
) -> PackageInput {
    let tables: Vec<TableRecords> = result
        .tables
        .iter()
        .map(|t| TableRecords {
            table_id: t.table_id.clone(),
            lines: t.jsonl_lines.clone(),
        })
        .collect();

    PackageInput {
        manifest_json,
        base_json,
        schema_json,
        tables,
        attachment_metadata_jsonl: result.attachment_metadata_jsonl.clone(),
        linked_records_jsonl: result.linked_records_jsonl.clone(),
        backup_report_json,
        compatibility_report_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_result(table_id: &str, records: usize) -> TableExportResult {
        TableExportResult {
            table_id: table_id.to_string(),
            table_name: "Test".to_string(),
            jsonl_lines: (0..records)
                .map(|i| format!(r#"{{"id":"rec{i:03}","fields":{{}}}}"#))
                .collect(),
            record_count: records,
            pages_fetched: 1,
        }
    }

    fn engine_result(tables: Vec<TableExportResult>) -> RecordExportEngineResult {
        RecordExportEngineResult {
            base_id: "appSyn01".to_string(),
            base_name: "Synthetic".to_string(),
            tables,
            linked_records_jsonl: vec![],
            attachment_metadata_jsonl: vec![],
        }
    }

    #[test]
    fn total_records_sums_across_tables() {
        let r = engine_result(vec![table_result("tbl01", 10), table_result("tbl02", 5)]);
        assert_eq!(r.total_records(), 15);
    }

    #[test]
    fn total_records_zero_for_empty() {
        let r = engine_result(vec![]);
        assert_eq!(r.total_records(), 0);
    }

    #[test]
    fn build_package_input_maps_table_ids() {
        let r = engine_result(vec![table_result("tbl01", 2)]);
        let input = build_package_input(
            &r,
            b"{\"format\":\"airbridge\"}".to_vec(),
            b"{\"baseId\":\"appSyn01\"}".to_vec(),
            b"{\"tables\":[]}".to_vec(),
            b"{\"status\":\"ok\"}".to_vec(),
            b"{}".to_vec(),
        );
        assert_eq!(input.tables.len(), 1);
        assert_eq!(input.tables[0].table_id, "tbl01");
        assert_eq!(input.tables[0].lines.len(), 2);
    }

    #[test]
    fn build_package_input_preserves_jsonl_bytes() {
        let mut r = engine_result(vec![]);
        r.linked_records_jsonl = b"linked\n".to_vec();
        r.attachment_metadata_jsonl = b"attach\n".to_vec();
        let input = build_package_input(
            &r,
            b"{}".to_vec(),
            b"{}".to_vec(),
            b"{}".to_vec(),
            b"{}".to_vec(),
            b"{}".to_vec(),
        );
        assert_eq!(input.linked_records_jsonl, b"linked\n");
        assert_eq!(input.attachment_metadata_jsonl, b"attach\n");
    }

    #[test]
    fn build_package_input_is_complete_with_required_fields() {
        let r = engine_result(vec![]);
        let input = build_package_input(
            &r,
            b"{\"format\":\"airbridge\"}".to_vec(),
            b"{\"baseId\":\"appSyn01\"}".to_vec(),
            b"{\"tables\":[]}".to_vec(),
            b"{\"status\":\"ok\"}".to_vec(),
            b"{}".to_vec(),
        );
        assert!(input.is_complete());
    }

    #[test]
    fn engine_result_serializes_to_json() {
        let r = engine_result(vec![table_result("tbl01", 1)]);
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(json.contains("appSyn01"));
        assert!(json.contains("tbl01"));
    }

    #[test]
    fn table_export_result_pages_fetched() {
        let t = TableExportResult {
            table_id: "tbl01".to_string(),
            table_name: "T".to_string(),
            jsonl_lines: vec![],
            record_count: 0,
            pages_fetched: 3,
        };
        assert_eq!(t.pages_fetched, 3);
    }
}
