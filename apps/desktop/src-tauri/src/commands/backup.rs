use crate::airtable::models::{AirtableField, AirtableFieldId, AirtableTable, AirtableTableId};
use crate::backup::estimates::DEFAULT_PAGE_SIZE;
use crate::backup::export_plan::{create_export_plan, RecordsExportPlan, RecordsExportPlanRequest};
use crate::backup::planner::create_plan;
use crate::errors::AirBridgeResult;
use crate::models::backup::{BackupPackageSummary, BackupScope, BackupStatus};
use crate::models::backup_plan::{BackupPlan, BackupPlanRequest, BackupScope as PlanScope};

#[tauri::command]
pub fn list_backup_packages() -> AirBridgeResult<Vec<BackupPackageSummary>> {
    Ok(vec![
        BackupPackageSummary {
            id: "pkg-001".to_string(),
            connection_id: "conn-002".to_string(),
            base_id: "appExampleBase01".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            base_name: "Example Projects & Tasks".to_string(),
            scope: BackupScope::Full,
            status: BackupStatus::Succeeded,
            table_count: 2,
            record_count: 47,
            file_size_bytes: 18432,
            created_at: "2025-01-14T14:22:10Z".to_string(),
            output_path: "".to_string(),
        },
        BackupPackageSummary {
            id: "pkg-002".to_string(),
            connection_id: "conn-002".to_string(),
            base_id: "appExampleBase02".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            base_name: "Example Contacts".to_string(),
            scope: BackupScope::SchemaOnly,
            status: BackupStatus::Succeeded,
            table_count: 1,
            record_count: 0,
            file_size_bytes: 3072,
            created_at: "2025-01-13T11:05:44Z".to_string(),
            output_path: "".to_string(),
        },
        BackupPackageSummary {
            id: "pkg-003".to_string(),
            connection_id: "conn-002".to_string(),
            base_id: "appExampleBase01".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            base_name: "Example Projects & Tasks".to_string(),
            scope: BackupScope::Full,
            status: BackupStatus::Failed,
            table_count: 0,
            record_count: 0,
            file_size_bytes: 0,
            created_at: "2025-01-12T08:47:30Z".to_string(),
            output_path: "".to_string(),
        },
    ])
}

/// Generates a backup plan (dry-run) from the provided request.
///
/// No token required — the request carries the schema data already fetched
/// by the frontend. No records are read. No files are written.
#[tauri::command]
pub fn create_backup_plan(request: BackupPlanRequest) -> AirBridgeResult<BackupPlan> {
    let tables: Vec<AirtableTable> = request
        .tables
        .iter()
        .map(|t| AirtableTable {
            id: AirtableTableId(t.id.clone()),
            name: t.name.clone(),
            primary_field_id: None,
            fields: t
                .fields
                .iter()
                .map(|f| AirtableField {
                    id: AirtableFieldId(f.id.clone()),
                    name: f.name.clone(),
                    field_type: f.field_type.clone(),
                    options: None,
                })
                .collect(),
        })
        .collect();

    let record_counts: Vec<Option<usize>> = request.tables.iter().map(|t| t.record_count).collect();

    let scope = match request.scope {
        PlanScope::Full => PlanScope::Full,
        PlanScope::SchemaOnly => PlanScope::SchemaOnly,
        PlanScope::RecordsOnly => PlanScope::RecordsOnly,
    };

    let plan = create_plan(
        &request.base_id,
        &request.base_name,
        &tables,
        &record_counts,
        scope,
    );

    Ok(plan)
}

/// Generates a records export plan from a backup plan.
///
/// No token required. No live Airtable calls. No file writes.
/// The plan describes how records would be exported — it does not export them.
#[tauri::command]
pub fn create_records_export_plan(
    request: RecordsExportPlanRequest,
) -> AirBridgeResult<RecordsExportPlan> {
    let record_counts: Vec<Option<usize>> = request
        .backup_plan
        .tables
        .iter()
        .map(|t| t.record_count)
        .collect();

    let plan = create_export_plan(
        &request.base_id,
        &request.base_name,
        &request.backup_plan,
        &record_counts,
        DEFAULT_PAGE_SIZE,
    );

    Ok(plan)
}

// ── Unit-testable helpers ──────────────────────────────────────────────────

/// For testing: build a backup plan directly without Tauri IPC overhead.
#[cfg(test)]
pub fn create_backup_plan_direct(request: BackupPlanRequest) -> AirBridgeResult<BackupPlan> {
    create_backup_plan(request)
}

/// For testing: build a records export plan directly without Tauri IPC overhead.
#[cfg(test)]
pub fn create_records_export_plan_direct(
    request: RecordsExportPlanRequest,
) -> AirBridgeResult<RecordsExportPlan> {
    create_records_export_plan(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::backup_plan::{BackupPlanFieldInput, BackupPlanTableInput};

    fn make_request(tables: Vec<BackupPlanTableInput>) -> BackupPlanRequest {
        BackupPlanRequest {
            base_id: "appExampleBase01".to_string(),
            base_name: "Example Base".to_string(),
            scope: PlanScope::Full,
            tables,
        }
    }

    fn simple_table(
        id: &str,
        name: &str,
        fields: Vec<BackupPlanFieldInput>,
    ) -> BackupPlanTableInput {
        BackupPlanTableInput {
            id: id.to_string(),
            name: name.to_string(),
            fields,
            record_count: None,
        }
    }

    fn field_input(id: &str, name: &str, type_str: &str) -> BackupPlanFieldInput {
        BackupPlanFieldInput {
            id: id.to_string(),
            name: name.to_string(),
            field_type: type_str.to_string(),
        }
    }

    #[test]
    fn create_plan_returns_all_tables() {
        let req = make_request(vec![
            simple_table(
                "tbl01",
                "Projects",
                vec![field_input("f01", "Name", "singleLineText")],
            ),
            simple_table(
                "tbl02",
                "Tasks",
                vec![field_input("f02", "Title", "singleLineText")],
            ),
        ]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        assert_eq!(plan.table_count, 2);
    }

    #[test]
    fn create_plan_returns_no_file_path() {
        let req = make_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        )]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        assert!(plan.output_package_path.is_none());
        assert!(plan.dry_run);
    }

    #[test]
    fn create_plan_generates_attachment_warning() {
        let req = make_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Files", "multipleAttachments")],
        )]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.code == "ATTACHMENT_METADATA_ONLY"));
    }

    #[test]
    fn create_plan_generates_linked_record_warning() {
        let req = make_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Link", "multipleRecordLinks")],
        )]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.code == "LINKED_RECORD_REMAPPING"));
    }

    #[test]
    fn create_plan_counts_fields_correctly() {
        let req = make_request(vec![simple_table(
            "tbl01",
            "T",
            vec![
                field_input("f01", "A", "singleLineText"),
                field_input("f02", "B", "formula"),
                field_input("f03", "C", "multipleAttachments"),
            ],
        )]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        assert_eq!(plan.total_field_count, 3);
    }

    #[test]
    fn create_plan_result_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_example_command_sentinel_0123456789";
        let req = make_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        )]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn create_plan_unknown_record_count_propagates() {
        use crate::models::backup_plan::RecordReadEstimate;
        let req = make_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        )]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        assert_eq!(plan.estimate.record_read_pages, RecordReadEstimate::Unknown);
    }

    #[test]
    fn create_plan_known_record_count_gives_correct_pages() {
        use crate::models::backup_plan::RecordReadEstimate;
        let mut table = simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        );
        table.record_count = Some(250);
        let req = make_request(vec![table]);
        let plan = create_backup_plan_direct(req).expect("should succeed");
        // 250 records → ceil(250/100) = 3 pages
        assert_eq!(
            plan.estimate.record_read_pages,
            RecordReadEstimate::Known(3)
        );
    }

    // ── Records export plan command tests ──────────────────────────────────

    fn make_export_request(tables: Vec<BackupPlanTableInput>) -> RecordsExportPlanRequest {
        let backup_plan = create_backup_plan_direct(make_request(tables)).expect("backup plan");
        RecordsExportPlanRequest {
            base_id: "appExampleBase01".to_string(),
            base_name: "Example Base".to_string(),
            backup_plan,
        }
    }

    #[test]
    fn export_plan_command_returns_planned_only() {
        let req = make_export_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        )]);
        let ep = create_records_export_plan_direct(req).expect("should succeed");
        assert!(ep.planned_only);
    }

    #[test]
    fn export_plan_command_returns_no_output_path() {
        let req = make_export_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        )]);
        let ep = create_records_export_plan_direct(req).expect("should succeed");
        assert!(ep.output_package_path.is_none());
    }

    #[test]
    fn export_plan_command_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_export_cmd_test_sentinel_0123456789";
        let req = make_export_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        )]);
        let ep = create_records_export_plan_direct(req).expect("should succeed");
        let json = serde_json::to_string(&ep).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn export_plan_command_includes_all_tables() {
        let req = make_export_request(vec![
            simple_table(
                "tbl01",
                "Projects",
                vec![field_input("f01", "Name", "singleLineText")],
            ),
            simple_table(
                "tbl02",
                "Tasks",
                vec![field_input("f02", "Title", "singleLineText")],
            ),
        ]);
        let ep = create_records_export_plan_direct(req).expect("should succeed");
        assert_eq!(ep.table_count, 2);
    }

    #[test]
    fn export_plan_command_unknown_count_when_not_provided() {
        use crate::backup::export_plan::RequestEstimate;
        let req = make_export_request(vec![simple_table(
            "tbl01",
            "T",
            vec![field_input("f01", "Name", "singleLineText")],
        )]);
        let ep = create_records_export_plan_direct(req).expect("should succeed");
        assert_eq!(ep.tables[0].request_estimate, RequestEstimate::Unknown);
    }
}
