use crate::errors::AirBridgeResult;
use crate::models::restore::{
    RestoreCompatibilityWarning, RestoreMode, RestorePlanStatus, RestorePlanSummary,
    WarningSeverity,
};
use crate::restore::dry_run::create_dry_run_plan;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;

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
}
