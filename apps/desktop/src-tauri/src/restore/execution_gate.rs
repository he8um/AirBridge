use crate::restore::execution::{
    RestoreExecutionBlockReason, RestoreExecutionError, RestoreExecutionRequest,
    RestoreExecutionResult, RestoreExecutionStatus, RestoreExecutionWarning,
    RESTORE_CONFIRMATION_PHRASE,
};
use std::path::Path;

/// Validates all preconditions for restore execution.
///
/// Even when all gates pass, returns `ReadyButDisabled` with
/// `RestoreWriteEngineNotEnabled` — the write engine is not enabled in this version.
///
/// - No Airtable API calls.
/// - No file writes.
/// - Token is checked for presence only; it is not stored or echoed.
pub fn validate_restore_execution_gate(
    request: &RestoreExecutionRequest,
) -> RestoreExecutionResult {
    let filename = Path::new(&request.package_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| request.package_filename.clone());

    // 1. Package filename must be non-empty
    if request.package_filename.is_empty() {
        return blocked(
            filename,
            RestoreExecutionBlockReason::MissingPackageInspection,
            "No package has been inspected. Select and inspect a backup package first.",
        );
    }

    // 2. Package validation status must be valid or warning
    let valid_statuses = ["valid", "warning"];
    if !valid_statuses.contains(&request.package_validation_status.as_str()) {
        return blocked(filename, RestoreExecutionBlockReason::InvalidPackage,
            "The selected package is invalid or has not been inspected. Inspect the package before proceeding.");
    }

    // 3. Dry-run plan must exist (non-empty dry_run_status)
    if request.dry_run_status.is_empty() {
        return blocked(
            filename,
            RestoreExecutionBlockReason::MissingDryRunPlan,
            "A restore plan preview must be generated before restore execution can be attempted.",
        );
    }

    // 4. Dry-run status must be ready or readyWithWarnings
    let ready_statuses = ["ready", "readyWithWarnings"];
    if !ready_statuses.contains(&request.dry_run_status.as_str()) {
        return blocked(
            filename,
            RestoreExecutionBlockReason::DryRunBlocked,
            "The restore plan is blocked. Resolve all errors in the plan before proceeding.",
        );
    }

    // 5. Target mode is always present via the enum, but package path must be non-empty
    if request.package_path.is_empty() {
        return blocked(
            filename,
            RestoreExecutionBlockReason::MissingTargetMode,
            "No package path is set. Select a package first.",
        );
    }

    // 6. Token must be non-empty
    // Note: token is checked for presence only; it is not stored or echoed.
    if request.token.is_empty() {
        return blocked(filename, RestoreExecutionBlockReason::MissingToken,
            "An Airtable personal access token is required. The token is used only for this operation and is not stored.");
    }

    // 7. Confirmation must match exactly
    if request.confirmation != RESTORE_CONFIRMATION_PHRASE {
        return blocked(
            filename,
            RestoreExecutionBlockReason::MissingConfirmation,
            &format!("Confirmation text does not match. Type the exact phrase to proceed."),
        );
    }

    // All gates passed — but write engine is not enabled.
    RestoreExecutionResult {
        filename,
        status: RestoreExecutionStatus::ReadyButDisabled,
        block_reason: Some(RestoreExecutionBlockReason::RestoreWriteEngineNotEnabled),
        message: "Restore execution contract is ready, but the write engine is not enabled in this version. No Airtable changes were made.".to_string(),
        warnings: vec![
            RestoreExecutionWarning {
                code: "WRITE_ENGINE_DISABLED".to_string(),
                message: "The restore write engine is not enabled. Schema creation and record import will be available in a future version.".to_string(),
            },
        ],
        errors: vec![],
        no_changes_made: true,
    }
}

fn blocked(
    filename: String,
    reason: RestoreExecutionBlockReason,
    message: &str,
) -> RestoreExecutionResult {
    RestoreExecutionResult {
        filename,
        status: RestoreExecutionStatus::Blocked,
        block_reason: Some(reason),
        message: message.to_string(),
        warnings: vec![],
        errors: vec![RestoreExecutionError {
            code: "GATE_BLOCKED".to_string(),
            message: message.to_string(),
        }],
        no_changes_made: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;

    fn base_request() -> RestoreExecutionRequest {
        RestoreExecutionRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            package_validation_status: "valid".to_string(),
            dry_run_status: "readyWithWarnings".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            token: "test-token-value".to_string(),
            confirmation: RESTORE_CONFIRMATION_PHRASE.to_string(),
        }
    }

    #[test]
    fn missing_package_filename_blocks() {
        let mut req = base_request();
        req.package_filename = "".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::MissingPackageInspection)
        );
        assert!(result.no_changes_made);
    }

    #[test]
    fn invalid_package_status_blocks() {
        let mut req = base_request();
        req.package_validation_status = "invalid".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::InvalidPackage)
        );
    }

    #[test]
    fn missing_dry_run_blocks() {
        let mut req = base_request();
        req.dry_run_status = "".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::MissingDryRunPlan)
        );
    }

    #[test]
    fn blocked_dry_run_blocks() {
        let mut req = base_request();
        req.dry_run_status = "blocked".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::DryRunBlocked)
        );
    }

    #[test]
    fn missing_token_blocks() {
        let mut req = base_request();
        req.token = "".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::MissingToken)
        );
    }

    #[test]
    fn wrong_confirmation_blocks() {
        let mut req = base_request();
        req.confirmation = "restore backup".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::MissingConfirmation)
        );
    }

    #[test]
    fn empty_confirmation_blocks() {
        let mut req = base_request();
        req.confirmation = "".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::MissingConfirmation)
        );
    }

    #[test]
    fn valid_gate_returns_ready_but_disabled() {
        let req = base_request();
        let result = validate_restore_execution_gate(&req);
        assert_eq!(result.status, RestoreExecutionStatus::ReadyButDisabled);
        assert_eq!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::RestoreWriteEngineNotEnabled)
        );
        assert!(result.no_changes_made);
    }

    #[test]
    fn result_never_contains_token() {
        let req = base_request();
        let result = validate_restore_execution_gate(&req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("test-token-value"));
    }

    #[test]
    fn result_never_contains_absolute_path() {
        let req = base_request();
        let result = validate_restore_execution_gate(&req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_changes_made_always_true() {
        // Both blocked and ready-but-disabled must have no_changes_made: true
        let mut req = base_request();
        req.token = "".to_string();
        let blocked_result = validate_restore_execution_gate(&req);
        assert!(blocked_result.no_changes_made);

        req.token = "test-token-value".to_string();
        let ready_result = validate_restore_execution_gate(&req);
        assert!(ready_result.no_changes_made);
    }

    #[test]
    fn warning_status_package_is_accepted() {
        let mut req = base_request();
        req.package_validation_status = "warning".to_string();
        let result = validate_restore_execution_gate(&req);
        // Should not block on package validation — warning is acceptable
        assert_ne!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::InvalidPackage)
        );
    }

    #[test]
    fn ready_dry_run_is_accepted() {
        let mut req = base_request();
        req.dry_run_status = "ready".to_string();
        let result = validate_restore_execution_gate(&req);
        assert_ne!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::DryRunBlocked)
        );
        assert_ne!(
            result.block_reason,
            Some(RestoreExecutionBlockReason::MissingDryRunPlan)
        );
    }
}
