use serde::{Deserialize, Serialize};

use crate::backup::export_plan::{RecordsExportPlan, RequestEstimate};

/// Discrete unit of work in an export job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressUnit {
    Schema,
    TableRecords,
    LinkedReferences,
    AttachmentMetadata,
    PackageWrite,
    Validation,
}

/// Status of a progress unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressUnitStatus {
    NotStarted,
    /// Planned but awaiting a future phase (e.g. package write is future at planning time).
    Future,
    InProgress,
    Complete,
    Skipped,
}

/// A single progress entry describing one unit of export work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEntry {
    pub unit: ProgressUnit,
    pub status: ProgressUnitStatus,
    /// Human-readable label for the progress unit.
    pub label: String,
    /// Estimated work items (e.g. pages), if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_items: Option<usize>,
}

/// The overall export progress plan for an export job.
///
/// At planning time all units are `NotStarted` or `Future`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgressPlan {
    pub entries: Vec<ProgressEntry>,
    /// Total known progress items (pages) across all tables, if all counts are known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_known_items: Option<usize>,
}

/// Builds an `ExportProgressPlan` from a `RecordsExportPlan`.
///
/// At planning time:
/// - Schema, TableRecords, LinkedReferences, AttachmentMetadata → `NotStarted`
/// - PackageWrite, Validation → `Future` (require export to complete first)
pub fn build_progress_plan(export_plan: &RecordsExportPlan) -> ExportProgressPlan {
    let has_linked = export_plan
        .tables
        .iter()
        .any(|t| !t.linked_record_plans.is_empty());
    let has_attachments = export_plan
        .tables
        .iter()
        .any(|t| !t.attachment_plans.is_empty());

    // Sum known pages across all tables; if any table is unknown, total is unknown.
    let mut total_known: Option<usize> = Some(0);
    for table in &export_plan.tables {
        match &table.request_estimate {
            RequestEstimate::Known { pages } => {
                if let Some(ref mut t) = total_known {
                    *t += pages;
                }
            }
            RequestEstimate::Unknown => {
                total_known = None;
                break;
            }
        }
    }

    let mut entries = vec![ProgressEntry {
        unit: ProgressUnit::Schema,
        status: ProgressUnitStatus::NotStarted,
        label: "Read base schema".to_string(),
        estimated_items: Some(1),
    }];

    // One entry per table for records
    for table in &export_plan.tables {
        let items = match &table.request_estimate {
            RequestEstimate::Known { pages } => Some(*pages),
            RequestEstimate::Unknown => None,
        };
        entries.push(ProgressEntry {
            unit: ProgressUnit::TableRecords,
            status: ProgressUnitStatus::NotStarted,
            label: format!("Export records — {}", table.table_name),
            estimated_items: items,
        });
    }

    if has_linked {
        entries.push(ProgressEntry {
            unit: ProgressUnit::LinkedReferences,
            status: ProgressUnitStatus::NotStarted,
            label: "Extract linked record references".to_string(),
            estimated_items: None,
        });
    }

    if has_attachments {
        entries.push(ProgressEntry {
            unit: ProgressUnit::AttachmentMetadata,
            status: ProgressUnitStatus::NotStarted,
            label: "Extract attachment metadata".to_string(),
            estimated_items: None,
        });
    }

    entries.push(ProgressEntry {
        unit: ProgressUnit::PackageWrite,
        status: ProgressUnitStatus::Future,
        label: "Write .airbridge package".to_string(),
        estimated_items: None,
    });

    entries.push(ProgressEntry {
        unit: ProgressUnit::Validation,
        status: ProgressUnitStatus::Future,
        label: "Validate package checksums".to_string(),
        estimated_items: None,
    });

    ExportProgressPlan {
        entries,
        total_known_items: total_known,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::{AirtableField, AirtableFieldId, AirtableTable, AirtableTableId};
    use crate::backup::estimates::DEFAULT_PAGE_SIZE;
    use crate::backup::export_plan::create_export_plan;
    use crate::backup::planner::create_plan;
    use crate::models::backup_plan::BackupScope;

    fn field(id: &str, name: &str, t: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId(id.to_string()),
            name: name.to_string(),
            field_type: t.to_string(),
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

    fn make_export_plan(tables: &[AirtableTable], counts: &[Option<usize>]) -> RecordsExportPlan {
        let bp = create_plan("appSyn01", "S", tables, &[], BackupScope::Full);
        create_export_plan("appSyn01", "S", &bp, counts, DEFAULT_PAGE_SIZE)
    }

    #[test]
    fn progress_plan_includes_schema_unit() {
        let ep = make_export_plan(&[], &[]);
        let pp = build_progress_plan(&ep);
        assert!(pp.entries.iter().any(|e| e.unit == ProgressUnit::Schema));
    }

    #[test]
    fn progress_plan_package_write_is_future() {
        let ep = make_export_plan(&[], &[]);
        let pp = build_progress_plan(&ep);
        let pw = pp
            .entries
            .iter()
            .find(|e| e.unit == ProgressUnit::PackageWrite);
        assert!(pw.is_some());
        assert_eq!(pw.unwrap().status, ProgressUnitStatus::Future);
    }

    #[test]
    fn progress_plan_validation_is_future() {
        let ep = make_export_plan(&[], &[]);
        let pp = build_progress_plan(&ep);
        let v = pp
            .entries
            .iter()
            .find(|e| e.unit == ProgressUnit::Validation);
        assert!(v.is_some());
        assert_eq!(v.unwrap().status, ProgressUnitStatus::Future);
    }

    #[test]
    fn total_known_items_set_when_all_counts_provided() {
        let t = table("tbl01", "T", vec![field("f01", "N", "singleLineText")]);
        let ep = make_export_plan(&[t], &[Some(100)]);
        let pp = build_progress_plan(&ep);
        // 100 records → 1 page
        assert_eq!(pp.total_known_items, Some(1));
    }

    #[test]
    fn total_known_items_none_when_count_unknown() {
        let t = table("tbl01", "T", vec![field("f01", "N", "singleLineText")]);
        let ep = make_export_plan(&[t], &[]);
        let pp = build_progress_plan(&ep);
        assert!(pp.total_known_items.is_none());
    }

    #[test]
    fn linked_record_unit_appears_when_linked_fields_present() {
        let t = table("tbl01", "T", vec![field("f01", "L", "multipleRecordLinks")]);
        let ep = make_export_plan(&[t], &[]);
        let pp = build_progress_plan(&ep);
        assert!(pp
            .entries
            .iter()
            .any(|e| e.unit == ProgressUnit::LinkedReferences));
    }

    #[test]
    fn attachment_unit_appears_when_attachment_fields_present() {
        let t = table("tbl01", "T", vec![field("f01", "F", "multipleAttachments")]);
        let ep = make_export_plan(&[t], &[]);
        let pp = build_progress_plan(&ep);
        assert!(pp
            .entries
            .iter()
            .any(|e| e.unit == ProgressUnit::AttachmentMetadata));
    }

    #[test]
    fn progress_plan_serializes_to_json() {
        let ep = make_export_plan(&[], &[]);
        let pp = build_progress_plan(&ep);
        let json = serde_json::to_string(&pp).expect("serialize");
        assert!(json.contains("entries"));
    }
}
