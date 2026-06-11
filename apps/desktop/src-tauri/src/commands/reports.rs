use crate::errors::AirBridgeResult;
use crate::models::report::{ReportItem, ReportSeverity, ReportSummary, ReportType};

#[tauri::command]
pub fn list_reports() -> AirBridgeResult<Vec<ReportSummary>> {
    Ok(vec![
        ReportSummary {
            id: "report-001".to_string(),
            report_type: ReportType::Backup,
            title: "Backup Report: Example Projects & Tasks".to_string(),
            created_at: "2025-01-14T14:22:12Z".to_string(),
            severity: ReportSeverity::Info,
            item_count: 1,
            items: vec![ReportItem {
                id: "ritem-001".to_string(),
                severity: ReportSeverity::Info,
                title: "Backup completed successfully".to_string(),
                detail: Some("2 tables and 47 records written".to_string()),
                field_name: None,
                table_name: None,
            }],
            related_job_id: Some("job-001".to_string()),
            related_base_id: Some("appExampleBase01".to_string()),
            related_base_name: Some("Example Projects & Tasks".to_string()),
        },
        ReportSummary {
            id: "report-002".to_string(),
            report_type: ReportType::Compatibility,
            title: "Compatibility Report: pkg-001".to_string(),
            created_at: "2025-01-14T14:22:14Z".to_string(),
            severity: ReportSeverity::Warning,
            item_count: 2,
            items: vec![
                ReportItem {
                    id: "ritem-002".to_string(),
                    severity: ReportSeverity::Warning,
                    title: "Formula field is unsupported for restore".to_string(),
                    detail: None,
                    field_name: Some("Formula Result".to_string()),
                    table_name: Some("Projects".to_string()),
                },
                ReportItem {
                    id: "ritem-003".to_string(),
                    severity: ReportSeverity::Info,
                    title: "Rollup field backed up as metadata only".to_string(),
                    detail: None,
                    field_name: Some("Rollup Count".to_string()),
                    table_name: Some("Tasks".to_string()),
                },
            ],
            related_job_id: None,
            related_base_id: Some("appExampleBase01".to_string()),
            related_base_name: None,
        },
        ReportSummary {
            id: "report-003".to_string(),
            report_type: ReportType::Restore,
            title: "Dry-Run Report: plan-001".to_string(),
            created_at: "2025-01-14T15:01:20Z".to_string(),
            severity: ReportSeverity::Warning,
            item_count: 1,
            items: vec![ReportItem {
                id: "ritem-004".to_string(),
                severity: ReportSeverity::Warning,
                title: "2 fields skipped during dry run".to_string(),
                detail: Some("Fields skipped: Formula Result, Rollup Count".to_string()),
                field_name: None,
                table_name: None,
            }],
            related_job_id: Some("rjob-001".to_string()),
            related_base_id: None,
            related_base_name: None,
        },
    ])
}
