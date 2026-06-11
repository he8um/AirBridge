use crate::errors::AirBridgeResult;
use crate::models::restore::{
    RestoreCompatibilityWarning, RestoreMode, RestorePlanStatus, RestorePlanSummary,
    WarningSeverity,
};

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
