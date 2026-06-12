use crate::restore::plan::RestoreTargetMode;
use serde::{Deserialize, Serialize};

/// The exact confirmation phrase the user must supply to attempt restore execution.
pub const RESTORE_CONFIRMATION_PHRASE: &str = "RESTORE BACKUP";

/// Status of the restore execution attempt.
/// Note: `Succeeded` is intentionally absent — the write engine is not enabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreExecutionStatus {
    Blocked,
    ReadyButDisabled,
    Failed,
}

/// The reason a restore execution was blocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreExecutionBlockReason {
    MissingPackageInspection,
    InvalidPackage,
    MissingDryRunPlan,
    DryRunBlocked,
    MissingTargetMode,
    MissingToken,
    MissingConfirmation,
    RestoreWriteEngineNotEnabled,
}

/// Input for the restore execution command.
/// - token is consumed; never stored or echoed.
/// - package_path is never echoed in the result.
/// - confirmation must equal RESTORE_CONFIRMATION_PHRASE exactly.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreExecutionRequest {
    /// Filename-only identifier of the inspected package (not the full path).
    pub package_filename: String,
    /// Full path used to locate the package — never echoed in the result.
    pub package_path: String,
    /// Validation status from the most recent inspection ("valid" | "warning").
    pub package_validation_status: String,
    /// Status from the most recent dry-run plan ("ready" | "readyWithWarnings").
    pub dry_run_status: String,
    /// Target restore mode.
    pub target_mode: RestoreTargetMode,
    /// Optional target base name.
    #[serde(default)]
    pub target_base_name: Option<String>,
    /// Airtable personal access token. Consumed; never stored.
    pub token: String,
    /// Must equal RESTORE_CONFIRMATION_PHRASE exactly.
    pub confirmation: String,
}

/// A non-blocking warning attached to the execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreExecutionWarning {
    pub code: String,
    pub message: String,
}

/// A blocking error attached to the execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreExecutionError {
    pub code: String,
    pub message: String,
}

/// Result of the restore execution command.
/// - No token.
/// - No absolute path — filename only.
/// - no_changes_made is always true.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreExecutionResult {
    /// Filename only — never the full path.
    pub filename: String,
    pub status: RestoreExecutionStatus,
    pub block_reason: Option<RestoreExecutionBlockReason>,
    pub message: String,
    pub warnings: Vec<RestoreExecutionWarning>,
    pub errors: Vec<RestoreExecutionError>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
}
