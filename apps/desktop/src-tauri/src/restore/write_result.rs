use serde::{Deserialize, Serialize};

/// Status values for a restore write engine preview.
///
/// Note: `Succeeded` is intentionally absent — the write engine is not enabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreWriteEngineStatus {
    Disabled,
    Blocked,
    NotStarted,
}

/// Phase identifiers for the write engine pipeline.
///
/// These phases describe what a write engine *would* execute.
/// None of these phases are active — all return disabled status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreWritePhase {
    ValidateInputs,
    SchemaCreation,
    RecordCreation,
    LinkedRecordUpdates,
    AttachmentHandling,
    FinalValidation,
}

/// Why restore writes are disabled or blocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreWriteDisabledReason {
    DisabledByProductPolicy,
    BlockedByInvalidPlan,
    BlockedByMissingConfirmation,
    BlockedByTargetSafety,
    NotAvailable,
}

/// An event emitted during the write engine skeleton evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreWriteEvent {
    pub phase: RestoreWritePhase,
    pub code: String,
    pub message: String,
}

/// Per-phase status summary for one write engine phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreWritePhaseSummary {
    pub phase: RestoreWritePhase,
    pub status: RestoreWriteEngineStatus,
    /// Always true — no changes made.
    pub no_changes_made: bool,
    pub note: String,
}

/// Result of the restore write engine skeleton preview.
///
/// - No token.
/// - No absolute path — filename only.
/// - no_changes_made is always true.
/// - status is always disabled or blocked — never succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreWriteEngineResult {
    /// Filename only — never the full path.
    pub filename: String,
    pub status: RestoreWriteEngineStatus,
    pub disabled_reason: RestoreWriteDisabledReason,
    pub message: String,
    pub phase_summaries: Vec<RestoreWritePhaseSummary>,
    pub events: Vec<RestoreWriteEvent>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
}
