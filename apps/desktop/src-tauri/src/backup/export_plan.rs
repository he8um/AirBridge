use serde::{Deserialize, Serialize};

use crate::backup::estimates::DEFAULT_PAGE_SIZE;
use crate::backup::export_paths::{fields_json_path, records_jsonl_path, table_json_path};
use crate::models::backup_plan::{
    AttachmentPolicy, BackupPlan, BackupPlanWarning, LinkedRecordPolicy, RecordReadEstimate,
    WarningSeverity,
};

// ── Record count state ────────────────────────────────────────────────────────

/// Whether the record count for a table is known ahead of export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RecordCountState {
    /// Record count is known; value is the count.
    Known { count: usize },
    /// Record count is not yet known — requires a live fetch.
    Unknown,
}

// ── Request estimate ──────────────────────────────────────────────────────────

/// Estimated number of API requests for a single table's records export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RequestEstimate {
    Known { pages: usize },
    Unknown,
}

impl RequestEstimate {
    fn from_record_count_state(state: &RecordCountState, page_size: usize) -> Self {
        match state {
            RecordCountState::Known { count } => {
                let pages = if *count == 0 {
                    1
                } else {
                    (count + page_size - 1) / page_size
                };
                RequestEstimate::Known { pages }
            }
            RecordCountState::Unknown => RequestEstimate::Unknown,
        }
    }
}

// ── JSONL output plan ─────────────────────────────────────────────────────────

/// Planned JSONL output entry for a table's records.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonlOutputPlan {
    /// Package entry path (e.g. `tables/tblAbc01/records.jsonl`).
    pub entry_path: String,
    /// Whether this entry is planned but not yet written.
    pub planned_only: bool,
}

// ── Linked record extraction plan ────────────────────────────────────────────

/// Plan for extracting linked record references from a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedRecordExtractionPlan {
    pub field_id: String,
    pub field_name: String,
    pub policy: LinkedRecordPolicy,
    /// Restore will require ID remapping — captured as a notice.
    pub restore_note: String,
}

// ── Attachment metadata extraction plan ──────────────────────────────────────

/// Plan for extracting attachment metadata from a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMetadataExtractionPlan {
    pub field_id: String,
    pub field_name: String,
    pub policy: AttachmentPolicy,
    /// Attachment file content is not included — only metadata is captured.
    pub content_note: String,
}

// ── Field extraction plan ─────────────────────────────────────────────────────

/// Per-field extraction entry in a table export plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldExtractionPlan {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    pub compatibility: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_record_plan: Option<LinkedRecordExtractionPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_plan: Option<AttachmentMetadataExtractionPlan>,
}

// ── Per-table export plan ─────────────────────────────────────────────────────

/// Export plan for a single table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableExportPlan {
    pub table_id: String,
    pub table_name: String,
    pub record_count: RecordCountState,
    pub request_estimate: RequestEstimate,
    pub page_size: usize,
    pub jsonl_output: JsonlOutputPlan,
    pub table_metadata_path: String,
    pub fields_metadata_path: String,
    pub fields: Vec<FieldExtractionPlan>,
    pub linked_record_plans: Vec<LinkedRecordExtractionPlan>,
    pub attachment_plans: Vec<AttachmentMetadataExtractionPlan>,
    pub warnings: Vec<BackupPlanWarning>,
}

// ── Top-level export plan ─────────────────────────────────────────────────────

/// Request input for generating a records export plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordsExportPlanRequest {
    pub base_id: String,
    pub base_name: String,
    pub backup_plan: BackupPlan,
}

/// The complete records export plan — planning-only, no live data fetched.
///
/// `output_package_path` is always absent: no file is written at planning time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordsExportPlan {
    pub base_id: String,
    pub base_name: String,
    pub table_count: usize,
    pub page_size: usize,
    pub tables: Vec<TableExportPlan>,
    pub warnings: Vec<BackupPlanWarning>,
    /// Always `true` — no records have been fetched and no file has been written.
    pub planned_only: bool,
    /// Always `None` — no output file is written at planning time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_package_path: Option<String>,
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Builds a `RecordsExportPlan` from a `BackupPlan` and optional per-table record counts.
///
/// `record_counts` is indexed parallel to `backup_plan.tables`. Pass an empty slice
/// to mark all table record counts as unknown.
pub fn create_export_plan(
    base_id: &str,
    base_name: &str,
    backup_plan: &BackupPlan,
    record_counts: &[Option<usize>],
    page_size: usize,
) -> RecordsExportPlan {
    let effective_page_size = if page_size == 0 {
        DEFAULT_PAGE_SIZE
    } else {
        page_size
    };

    let mut plan_tables: Vec<TableExportPlan> = Vec::new();
    let mut plan_warnings: Vec<BackupPlanWarning> = Vec::new();

    for (i, table) in backup_plan.tables.iter().enumerate() {
        let opt_count = record_counts.get(i).copied().flatten();

        let record_count_state = match opt_count {
            Some(n) => RecordCountState::Known { count: n },
            None => RecordCountState::Unknown,
        };

        let request_estimate =
            RequestEstimate::from_record_count_state(&record_count_state, effective_page_size);

        let jsonl_output = JsonlOutputPlan {
            entry_path: records_jsonl_path(&table.id),
            planned_only: true,
        };

        // Build per-field extraction plans
        let mut fields: Vec<FieldExtractionPlan> = Vec::new();
        let mut linked_plans: Vec<LinkedRecordExtractionPlan> = Vec::new();
        let mut attachment_plans: Vec<AttachmentMetadataExtractionPlan> = Vec::new();

        for f in &table.fields {
            let linked = if f.field_type == "multipleRecordLinks" {
                let lrp = LinkedRecordExtractionPlan {
                    field_id: f.id.clone(),
                    field_name: f.name.clone(),
                    policy: LinkedRecordPolicy::RemappingRequiredForRestore,
                    restore_note:
                        "Record ID references are captured. Restore requires ID remapping."
                            .to_string(),
                };
                linked_plans.push(lrp.clone());
                Some(lrp)
            } else {
                None
            };

            let attachment = if f.field_type == "multipleAttachments" {
                let amp = AttachmentMetadataExtractionPlan {
                    field_id: f.id.clone(),
                    field_name: f.name.clone(),
                    policy: AttachmentPolicy::MetadataOnly,
                    content_note: "Attachment file content is not exported. Only metadata (filename, URL, size) is captured.".to_string(),
                };
                attachment_plans.push(amp.clone());
                Some(amp)
            } else {
                None
            };

            fields.push(FieldExtractionPlan {
                field_id: f.id.clone(),
                field_name: f.name.clone(),
                field_type: f.field_type.clone(),
                compatibility: f.compatibility.clone(),
                linked_record_plan: linked,
                attachment_plan: attachment,
            });
        }

        // Per-table warnings
        let mut table_warnings: Vec<BackupPlanWarning> = Vec::new();

        if record_count_state == RecordCountState::Unknown {
            table_warnings.push(BackupPlanWarning {
                severity: WarningSeverity::Warning,
                code: "UNKNOWN_RECORD_COUNT".to_string(),
                message: "Record count is unknown. Actual pages will be determined at export time."
                    .to_string(),
                table_name: Some(table.name.clone()),
                field_name: None,
            });
        }

        if !attachment_plans.is_empty() {
            table_warnings.push(BackupPlanWarning {
                severity: WarningSeverity::Warning,
                code: "ATTACHMENT_METADATA_ONLY".to_string(),
                message:
                    "Attachment fields detected — only metadata will be exported, not file content."
                        .to_string(),
                table_name: Some(table.name.clone()),
                field_name: None,
            });
        }

        if !linked_plans.is_empty() {
            table_warnings.push(BackupPlanWarning {
                severity: WarningSeverity::Warning,
                code: "LINKED_RECORD_REMAPPING".to_string(),
                message:
                    "Linked record references are captured. Restore will require ID remapping."
                        .to_string(),
                table_name: Some(table.name.clone()),
                field_name: None,
            });
        }

        plan_warnings.extend(table_warnings.clone());

        plan_tables.push(TableExportPlan {
            table_id: table.id.clone(),
            table_name: table.name.clone(),
            record_count: record_count_state,
            request_estimate,
            page_size: effective_page_size,
            jsonl_output,
            table_metadata_path: table_json_path(&table.id),
            fields_metadata_path: fields_json_path(&table.id),
            fields,
            linked_record_plans: linked_plans,
            attachment_plans,
            warnings: table_warnings,
        });
    }

    RecordsExportPlan {
        base_id: base_id.to_string(),
        base_name: base_name.to_string(),
        table_count: plan_tables.len(),
        page_size: effective_page_size,
        tables: plan_tables,
        warnings: plan_warnings,
        planned_only: true,
        output_package_path: None,
    }
}

// ── Helpers (re-exported for command layer) ───────────────────────────────────

/// Returns `RecordReadEstimate` for use in estimate summaries.
pub fn record_read_estimate_for_plan(plan: &RecordsExportPlan) -> RecordReadEstimate {
    let mut total: Option<usize> = Some(0);
    for table in &plan.tables {
        match &table.request_estimate {
            RequestEstimate::Known { pages } => {
                if let Some(ref mut t) = total {
                    *t += pages;
                }
            }
            RequestEstimate::Unknown => {
                total = None;
                break;
            }
        }
    }
    total
        .map(RecordReadEstimate::Known)
        .unwrap_or(RecordReadEstimate::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::{AirtableField, AirtableFieldId, AirtableTable, AirtableTableId};
    use crate::backup::planner::create_plan;
    use crate::models::backup_plan::BackupScope;

    fn field(id: &str, name: &str, type_str: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId(id.to_string()),
            name: name.to_string(),
            field_type: type_str.to_string(),
            options: None,
        }
    }

    fn table(id: &str, name: &str, fields: Vec<AirtableField>) -> AirtableTable {
        AirtableTable {
            id: AirtableTableId(id.to_string()),
            name: name.to_string(),
            primary_field_id: None,
            fields,
        }
    }

    fn plan_with_tables(tables: &[AirtableTable]) -> BackupPlan {
        create_plan("appSyn01", "Synthetic", tables, &[], BackupScope::Full)
    }

    #[test]
    fn export_plan_has_planned_only_true() {
        let plan = plan_with_tables(&[]);
        let ep = create_export_plan("appSyn01", "Synthetic", &plan, &[], DEFAULT_PAGE_SIZE);
        assert!(ep.planned_only);
    }

    #[test]
    fn export_plan_output_package_path_is_none() {
        let plan = plan_with_tables(&[]);
        let ep = create_export_plan("appSyn01", "Synthetic", &plan, &[], DEFAULT_PAGE_SIZE);
        assert!(ep.output_package_path.is_none());
    }

    #[test]
    fn known_record_count_estimates_pages_correctly() {
        let t = table(
            "tbl01",
            "Items",
            vec![field("f01", "Name", "singleLineText")],
        );
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[Some(250)], 100);
        assert_eq!(
            ep.tables[0].request_estimate,
            RequestEstimate::Known { pages: 3 }
        );
    }

    #[test]
    fn unknown_record_count_remains_unknown() {
        let t = table(
            "tbl01",
            "Items",
            vec![field("f01", "Name", "singleLineText")],
        );
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        assert_eq!(ep.tables[0].request_estimate, RequestEstimate::Unknown);
    }

    #[test]
    fn page_size_default_is_100() {
        let plan = plan_with_tables(&[]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        assert_eq!(ep.page_size, 100);
    }

    #[test]
    fn zero_records_estimates_one_page() {
        let t = table(
            "tbl01",
            "Items",
            vec![field("f01", "Name", "singleLineText")],
        );
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[Some(0)], 100);
        assert_eq!(
            ep.tables[0].request_estimate,
            RequestEstimate::Known { pages: 1 }
        );
    }

    #[test]
    fn jsonl_output_entry_path_contains_no_absolute_path() {
        let t = table(
            "tbl01",
            "Items",
            vec![field("f01", "Name", "singleLineText")],
        );
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        let path = &ep.tables[0].jsonl_output.entry_path;
        assert!(!path.starts_with('/'));
        assert!(!path.contains("Users/"));
    }

    #[test]
    fn linked_field_creates_linked_extraction_plan() {
        let t = table(
            "tbl01",
            "T",
            vec![field("f01", "Links", "multipleRecordLinks")],
        );
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        assert_eq!(ep.tables[0].linked_record_plans.len(), 1);
        assert_eq!(
            ep.tables[0].linked_record_plans[0].policy,
            LinkedRecordPolicy::RemappingRequiredForRestore
        );
    }

    #[test]
    fn attachment_field_creates_metadata_extraction_plan() {
        let t = table(
            "tbl01",
            "T",
            vec![field("f01", "Files", "multipleAttachments")],
        );
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        assert_eq!(ep.tables[0].attachment_plans.len(), 1);
        assert_eq!(
            ep.tables[0].attachment_plans[0].policy,
            AttachmentPolicy::MetadataOnly
        );
    }

    #[test]
    fn warnings_generated_for_unknown_count() {
        let t = table("tbl01", "T", vec![field("f01", "Name", "singleLineText")]);
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        assert!(ep.warnings.iter().any(|w| w.code == "UNKNOWN_RECORD_COUNT"));
    }

    #[test]
    fn warnings_generated_for_attachment_fields() {
        let t = table(
            "tbl01",
            "T",
            vec![field("f01", "Files", "multipleAttachments")],
        );
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        assert!(ep
            .warnings
            .iter()
            .any(|w| w.code == "ATTACHMENT_METADATA_ONLY"));
    }

    #[test]
    fn no_warning_for_unknown_count_when_count_provided() {
        let t = table("tbl01", "T", vec![field("f01", "Name", "singleLineText")]);
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[Some(10)], 100);
        assert!(!ep.warnings.iter().any(|w| w.code == "UNKNOWN_RECORD_COUNT"));
    }

    #[test]
    fn export_plan_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_export_plan_test_sentinel_0123456789";
        let t = table("tbl01", "T", vec![field("f01", "Name", "singleLineText")]);
        let plan = plan_with_tables(&[t]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        let json = serde_json::to_string(&ep).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn export_plan_serializes_to_json() {
        let plan = plan_with_tables(&[]);
        let ep = create_export_plan("appSyn01", "S", &plan, &[], DEFAULT_PAGE_SIZE);
        let json = serde_json::to_string(&ep).expect("serialize");
        assert!(json.contains("plannedOnly"));
    }

    #[test]
    fn record_read_estimate_unknown_when_any_table_unknown() {
        let tables = vec![
            table("tbl01", "A", vec![field("f01", "N", "singleLineText")]),
            table("tbl02", "B", vec![field("f02", "N", "singleLineText")]),
        ];
        let plan = plan_with_tables(&tables);
        // First known, second unknown
        let ep = create_export_plan("appSyn01", "S", &plan, &[Some(100)], 100);
        let est = record_read_estimate_for_plan(&ep);
        assert_eq!(est, RecordReadEstimate::Unknown);
    }

    #[test]
    fn record_read_estimate_known_when_all_counts_provided() {
        let tables = vec![
            table("tbl01", "A", vec![field("f01", "N", "singleLineText")]),
            table("tbl02", "B", vec![field("f02", "N", "singleLineText")]),
        ];
        let plan = plan_with_tables(&tables);
        let ep = create_export_plan("appSyn01", "S", &plan, &[Some(100), Some(200)], 100);
        let est = record_read_estimate_for_plan(&ep);
        // 100→1, 200→2 = 3 pages total
        assert_eq!(est, RecordReadEstimate::Known(3));
    }

    // Cross-check: verify estimate_record_pages math is consistent with export plan estimates.
    #[test]
    fn estimate_record_pages_used_consistently() {
        use crate::backup::estimates::estimate_record_pages;
        assert_eq!(
            estimate_record_pages(Some(150)),
            RecordReadEstimate::Known(2)
        );
    }
}
