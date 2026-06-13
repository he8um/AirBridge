use crate::errors::AirBridgeResult;
use crate::models::restore::{
    RestoreCompatibilityWarning, RestoreMode, RestorePlanStatus, RestorePlanSummary,
    WarningSeverity,
};
use crate::restore::dry_run::create_dry_run_plan;
use crate::restore::execution::{RestoreExecutionRequest, RestoreExecutionResult};
use crate::restore::execution_gate::validate_restore_execution_gate;
use crate::restore::plan::{RestoreDryRunPlan, RestoreDryRunRequest};
use crate::restore::record_import_plan::{RestoreRecordImportPlan, RestoreRecordImportPlanRequest};
use crate::restore::record_import_planner::create_record_import_plan;
use crate::restore::schema_plan::{RestoreSchemaPlan, RestoreSchemaPlanRequest};
use crate::restore::schema_planner::create_schema_plan;
use crate::restore::write_engine::{preview_write_engine, RestoreWriteEngineRequest};
use crate::restore::write_result::RestoreWriteEngineResult;

#[tauri::command]
pub fn list_restore_plans() -> AirBridgeResult<Vec<RestorePlanSummary>> {
    Ok(vec![RestorePlanSummary {
        id: "plan-001".to_string(),
        package_id: "pkg-001".to_string(),
        connection_id: "conn-002".to_string(),
        target_base_id: None,
        mode: RestoreMode::NewBase,
        status: RestorePlanStatus::Ready,
        warnings: vec![
            RestoreCompatibilityWarning {
                field_id: "fldProjFormula".to_string(),
                field_name: "Formula Result".to_string(),
                field_type: "formula".to_string(),
                message: "Formula fields cannot be restored via the API. The field must be recreated manually.".to_string(),
                severity: WarningSeverity::Warning,
            },
            RestoreCompatibilityWarning {
                field_id: "fldTaskRollup".to_string(),
                field_name: "Rollup Count".to_string(),
                field_type: "rollup".to_string(),
                message: "Rollup configuration is captured in the schema backup but computed values will not be restored.".to_string(),
                severity: WarningSeverity::Info,
            },
        ],
        created_at: "2025-01-14T15:00:00Z".to_string(),
    }])
}

/// Creates a restore dry-run plan from an existing `.airbridge` package.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files extracted to disk.
/// - No write operations of any kind.
/// - Returns filename only — the full path is never included in the result.
#[tauri::command]
pub fn create_restore_dry_run_plan(request: RestoreDryRunRequest) -> RestoreDryRunPlan {
    create_dry_run_plan(&request)
}

/// Creates a schema creation plan from a dry-run result.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files written or extracted.
/// - Filename in the result is never a full path.
/// - no_changes_made is always true.
#[tauri::command]
pub fn create_restore_schema_plan(request: RestoreSchemaPlanRequest) -> RestoreSchemaPlan {
    create_schema_plan(&request)
}

/// Creates a record import plan from a dry-run result and schema plan.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files written or extracted.
/// - Filename in the result is never a full path.
/// - no_changes_made is always true.
#[tauri::command]
pub fn create_restore_record_import_plan(
    request: RestoreRecordImportPlanRequest,
) -> RestoreRecordImportPlan {
    create_record_import_plan(&request)
}

/// Validates the restore execution safety gate and returns a blocked/disabled result.
///
/// In this version the write engine is not enabled. The command:
/// - Validates all preconditions (inspection, dry-run, target mode, token, confirmation).
/// - Never calls the Airtable API.
/// - Never writes any file.
/// - Never stores or echoes the token.
/// - Never includes the full package path in the result.
/// - Always sets no_changes_made: true.
#[tauri::command]
pub fn run_restore_execution(request: RestoreExecutionRequest) -> RestoreExecutionResult {
    // Token is forwarded to the gate for presence check only; it is not stored or logged.
    validate_restore_execution_gate(&request)
}

/// Produces a write engine skeleton preview from existing planning outputs.
///
/// - No token field — no Airtable access required.
/// - No Airtable API calls.
/// - No file writes.
/// - Never echoes the package path.
/// - Always sets no_changes_made: true.
/// - Status is always disabled — never succeeded.
#[tauri::command]
pub fn preview_restore_write_engine(
    request: RestoreWriteEngineRequest,
) -> RestoreWriteEngineResult {
    preview_write_engine(&request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::execution::{RestoreExecutionStatus, RESTORE_CONFIRMATION_PHRASE};
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::schema_plan::{
        RestoreSchemaPlanStatus, SchemaPlanFieldInput, SchemaPlanTableInput,
    };

    // ── schema plan command tests ──────────────────────────────────────────

    fn schema_plan_request(dry_run_status: &str) -> RestoreSchemaPlanRequest {
        RestoreSchemaPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: dry_run_status.to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables: vec![SchemaPlanTableInput {
                table_id: "tbl01".to_string(),
                table_name: "Projects".to_string(),
                fields: vec![
                    SchemaPlanFieldInput {
                        field_id: "fld01".to_string(),
                        field_name: "Name".to_string(),
                        field_type: "singleLineText".to_string(),
                        linked_table_id: None,
                    },
                    SchemaPlanFieldInput {
                        field_id: "fld02".to_string(),
                        field_name: "Status".to_string(),
                        field_type: "singleSelect".to_string(),
                        linked_table_id: None,
                    },
                ],
            }],
        }
    }

    #[test]
    fn schema_plan_command_does_not_require_token() {
        let req = schema_plan_request("ready");
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn schema_plan_command_ready_status_produces_plan() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(
            plan.status == RestoreSchemaPlanStatus::Ready
                || plan.status == RestoreSchemaPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn schema_plan_command_blocked_status_produces_blocked_plan() {
        let req = schema_plan_request("blocked");
        let plan = create_restore_schema_plan(req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Blocked);
        assert!(!plan.errors.is_empty());
    }

    #[test]
    fn schema_plan_result_no_changes_made_always_true() {
        for status in &["ready", "readyWithWarnings", "blocked"] {
            let req = schema_plan_request(status);
            let plan = create_restore_schema_plan(req);
            assert!(plan.no_changes_made);
        }
    }

    #[test]
    fn schema_plan_result_does_not_contain_absolute_path() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn schema_plan_result_has_no_token() {
        const SENTINEL: &str = "pat_command_test_sentinel_9999999999";
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn schema_plan_filename_is_not_a_path() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
        assert_eq!(plan.filename, "backup.airbridge");
    }

    #[test]
    fn schema_plan_result_has_table_steps() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(!plan.table_steps.is_empty());
        assert_eq!(plan.table_steps[0].table_name, "Projects");
    }

    #[test]
    fn schema_plan_result_has_field_steps() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(!plan.field_steps.is_empty());
    }

    #[test]
    fn schema_plan_ready_with_warnings_dry_run_status_accepted() {
        let req = schema_plan_request("readyWithWarnings");
        let plan = create_restore_schema_plan(req);
        assert!(
            plan.status == RestoreSchemaPlanStatus::Ready
                || plan.status == RestoreSchemaPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn schema_plan_empty_dry_run_status_produces_blocked() {
        let req = schema_plan_request("");
        let plan = create_restore_schema_plan(req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Blocked);
    }

    #[test]
    fn schema_plan_serializes_with_no_changes_made_key() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
    }

    #[test]
    fn schema_plan_has_no_succeeded_status() {
        for status in &["ready", "blocked"] {
            let req = schema_plan_request(status);
            let plan = create_restore_schema_plan(req);
            let json = serde_json::to_string(&plan).expect("serialize");
            assert!(!json.contains("succeeded"));
            assert!(!json.contains("Succeeded"));
        }
    }

    // ── dry-run command tests ──────────────────────────────────────────────

    #[test]
    fn command_does_not_require_token() {
        let req = RestoreDryRunRequest {
            path: "/nonexistent.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("token"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn nonexistent_path_returns_blocked_plan() {
        let req = RestoreDryRunRequest {
            path: "/tmp/does_not_exist_command_test.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let plan = create_restore_dry_run_plan(req);
        assert_eq!(
            plan.status,
            crate::restore::plan::RestorePlanStatus::Blocked
        );
        assert!(!plan.errors.is_empty());
    }

    #[test]
    fn blocked_plan_always_has_no_changes_made() {
        let req = RestoreDryRunRequest {
            path: "/tmp/does_not_exist_no_changes.airbridge".to_string(),
            target_mode: RestoreTargetMode::EmptyExistingBase,
            target_base_name: None,
        };
        let plan = create_restore_dry_run_plan(req);
        assert!(plan.no_changes_made);
    }

    #[test]
    fn result_does_not_contain_absolute_path() {
        let req = RestoreDryRunRequest {
            path: "/Users/testuser/backups/test.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let plan = create_restore_dry_run_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/testuser/"));
        assert!(!json.contains("/backups/"));
    }

    // ── execution command tests ────────────────────────────────────────────

    fn exec_request_missing_confirmation() -> RestoreExecutionRequest {
        RestoreExecutionRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            package_validation_status: "valid".to_string(),
            dry_run_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            token: "tok-test-value".to_string(),
            confirmation: "".to_string(),
        }
    }

    fn exec_request_valid() -> RestoreExecutionRequest {
        RestoreExecutionRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            package_validation_status: "valid".to_string(),
            dry_run_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            token: "tok-test-value".to_string(),
            confirmation: RESTORE_CONFIRMATION_PHRASE.to_string(),
        }
    }

    #[test]
    fn execution_command_missing_confirmation_returns_blocked() {
        let req = exec_request_missing_confirmation();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert!(result.no_changes_made);
    }

    #[test]
    fn execution_command_valid_gate_returns_ready_but_disabled() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::ReadyButDisabled);
        assert!(result.no_changes_made);
    }

    #[test]
    fn execution_result_does_not_contain_token() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("tok-test-value"));
    }

    #[test]
    fn execution_result_does_not_contain_absolute_path() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn execution_result_has_no_succeeded_status() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("succeeded"));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn execution_missing_token_returns_blocked() {
        let mut req = exec_request_valid();
        req.token = "".to_string();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
    }

    #[test]
    fn execution_invalid_package_returns_blocked() {
        let mut req = exec_request_valid();
        req.package_validation_status = "invalid".to_string();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
    }

    #[test]
    fn execution_no_changes_made_always_true() {
        // blocked case
        let req = exec_request_missing_confirmation();
        let result = run_restore_execution(req);
        assert!(result.no_changes_made);
        // ready-but-disabled case
        let req2 = exec_request_valid();
        let result2 = run_restore_execution(req2);
        assert!(result2.no_changes_made);
    }

    // ── record import plan command tests ───────────────────────────────────

    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
        RestoreRecordImportPlanStatus,
    };

    fn record_import_plan_request(
        dry_run_status: &str,
        schema_plan_status: &str,
    ) -> RestoreRecordImportPlanRequest {
        RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: dry_run_status.to_string(),
            schema_plan_status: schema_plan_status.to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables: vec![RecordImportTableInput {
                table_id: "tbl01".to_string(),
                table_name: "Projects".to_string(),
                record_count: Some(20),
                fields: vec![
                    RecordImportFieldInput {
                        field_id: "fld01".to_string(),
                        field_name: "Name".to_string(),
                        field_type: "singleLineText".to_string(),
                        linked_table_id: None,
                    },
                    RecordImportFieldInput {
                        field_id: "fld02".to_string(),
                        field_name: "Status".to_string(),
                        field_type: "singleSelect".to_string(),
                        linked_table_id: None,
                    },
                ],
            }],
        }
    }

    #[test]
    fn record_import_plan_command_does_not_require_token() {
        let req = record_import_plan_request("ready", "ready");
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn record_import_plan_command_ready_status_produces_plan() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(
            plan.status == RestoreRecordImportPlanStatus::Ready
                || plan.status == RestoreRecordImportPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn record_import_plan_command_blocked_dry_run_produces_blocked() {
        let req = record_import_plan_request("blocked", "ready");
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert!(!plan.errors.is_empty());
    }

    #[test]
    fn record_import_plan_command_blocked_schema_plan_produces_blocked() {
        let req = record_import_plan_request("ready", "blocked");
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert_eq!(plan.errors[0].code, "SCHEMA_PLAN_BLOCKED");
    }

    #[test]
    fn record_import_plan_command_no_changes_made_always_true() {
        for (dry, schema) in &[
            ("ready", "ready"),
            ("readyWithWarnings", "readyWithWarnings"),
            ("blocked", "ready"),
            ("ready", "blocked"),
        ] {
            let req = record_import_plan_request(dry, schema);
            let plan = create_restore_record_import_plan(req);
            assert!(
                plan.no_changes_made,
                "no_changes_made must be true for dry={dry}, schema={schema}"
            );
        }
    }

    #[test]
    fn record_import_plan_result_does_not_contain_absolute_path() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn record_import_plan_result_does_not_contain_token() {
        const SENTINEL: &str = "pat_command_test_import_sentinel_9999999999";
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn record_import_plan_filename_is_not_a_path() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
        assert_eq!(plan.filename, "backup.airbridge");
    }

    #[test]
    fn record_import_plan_result_has_table_plans() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(!plan.table_plans.is_empty());
        assert_eq!(plan.table_plans[0].table_name, "Projects");
    }

    #[test]
    fn record_import_plan_result_has_batch_size() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.table_plans[0].batch_size, 10);
    }

    #[test]
    fn record_import_plan_result_has_retry_policy() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(plan.retry_policy.max_retries_on_rate_limit > 0);
        assert!(plan.retry_policy.initial_backoff_ms > 0);
    }

    #[test]
    fn record_import_plan_ready_with_warnings_dry_run_status_accepted() {
        let req = record_import_plan_request("readyWithWarnings", "readyWithWarnings");
        let plan = create_restore_record_import_plan(req);
        assert_ne!(plan.status, RestoreRecordImportPlanStatus::Blocked);
    }

    #[test]
    fn record_import_plan_empty_tables_produces_blocked() {
        let req = RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            tables: vec![],
        };
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert_eq!(plan.errors[0].code, "NO_TABLES");
    }

    #[test]
    fn record_import_plan_has_no_succeeded_status() {
        for (dry, schema) in &[("ready", "ready"), ("blocked", "ready")] {
            let req = record_import_plan_request(dry, schema);
            let plan = create_restore_record_import_plan(req);
            let json = serde_json::to_string(&plan).expect("serialize");
            assert!(!json.contains("succeeded"), "dry={dry}, schema={schema}");
        }
    }

    #[test]
    fn record_import_plan_serializes_no_changes_made_key() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
    }

    // ── write engine command tests ─────────────────────────────────────────

    use crate::restore::write_result::RestoreWriteEngineStatus;

    fn write_engine_request() -> RestoreWriteEngineRequest {
        RestoreWriteEngineRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            schema_table_count: 2,
            schema_direct_field_count: 8,
            schema_deferred_field_count: 1,
            schema_manual_action_count: 0,
            schema_unsupported_count: 0,
            estimated_first_pass_batches: 3,
            estimated_second_pass_batches: 1,
            linked_record_update_count: 2,
        }
    }

    #[test]
    fn write_engine_command_returns_disabled() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn write_engine_command_no_changes_made_is_true() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn write_engine_command_does_not_require_token() {
        let req = write_engine_request();
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn write_engine_command_result_has_no_token() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn write_engine_command_result_has_no_absolute_path() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn write_engine_command_result_has_no_succeeded_status() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn write_engine_command_frontend_contract_is_safe() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        // status must be disabled or blocked — never succeeded
        assert!(
            result.status == RestoreWriteEngineStatus::Disabled
                || result.status == RestoreWriteEngineStatus::Blocked
        );
        assert!(result.no_changes_made);
        assert!(!result.filename.contains('/'));
    }
}
