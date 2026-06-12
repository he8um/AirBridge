use crate::errors::AirBridgeResult;
use crate::models::restore::{
    RestoreCompatibilityWarning, RestoreMode, RestorePlanStatus, RestorePlanSummary,
    WarningSeverity,
};
use crate::restore::dry_run::create_dry_run_plan;
use crate::restore::execution::{RestoreExecutionRequest, RestoreExecutionResult};
use crate::restore::execution_gate::validate_restore_execution_gate;
use crate::restore::plan::{RestoreDryRunPlan, RestoreDryRunRequest};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::execution::{RestoreExecutionStatus, RESTORE_CONFIRMATION_PHRASE};
    use crate::restore::plan::RestoreTargetMode;

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
}
