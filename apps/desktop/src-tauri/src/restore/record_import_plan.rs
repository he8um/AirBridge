use serde::{Deserialize, Serialize};

use crate::restore::plan::RestoreTargetMode;

/// Status of the record import plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreRecordImportPlanStatus {
    Ready,
    ReadyWithWarnings,
    Blocked,
}

/// Phase of a record import batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreRecordBatchPhase {
    /// First pass: create records without linked record field values.
    CreateRecords,
    /// Second pass: update linked record references after ID mapping exists.
    UpdateLinkedRecords,
    /// Skipped fields that cannot be written via the API.
    SkippedFields,
    /// Validation pass at end of import.
    Validation,
}

/// Strategy for mapping old record IDs to new record IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreRecordMappingStrategy {
    /// Each created record's new ID is collected and mapped from the source ID.
    MapSourceRecordIdToCreatedRecordId,
    /// Source record ID is preserved in a metadata field for reference.
    PreserveSourceIdInMetadata,
    /// Mapping is an output of execution — not available until records are created.
    UnavailableUntilExecution,
}

/// Policy for restoring attachment fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreAttachmentRestorePolicy {
    /// Attachment metadata is captured; file bytes are not re-uploaded.
    MetadataOnly,
    /// Downloading attachment files from backup is not supported in this version.
    DownloadNotSupported,
    /// Uploading attachment files to Airtable is not supported in this version.
    UploadNotSupported,
    /// Manual action is required to re-attach files after restore.
    ManualActionRequired,
}

/// How a field will be handled during record import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreRecordFieldImportPolicy {
    /// Field value is included in the record create payload.
    Include,
    /// Field is deferred to the linked record update pass.
    DeferToLinkedRecordPass,
    /// Field is skipped — computed, read-only, or unsupported by the API.
    Skip,
    /// Field value is included as metadata reference only.
    MetadataOnly,
}

/// A plan for a single batch of record operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordBatchPlan {
    pub batch_index: usize,
    pub phase: RestoreRecordBatchPhase,
    /// Number of records in this batch (Airtable write batch size = 10).
    pub record_count: usize,
    pub note: String,
}

/// How record ID mapping will be maintained for a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordMappingPlan {
    pub table_id: String,
    pub table_name: String,
    pub strategy: RestoreRecordMappingStrategy,
    /// True if this table has linked record fields that require remapping.
    pub remapping_required: bool,
    pub note: String,
}

/// A planned second-pass update for linked record fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreLinkedRecordUpdatePlan {
    pub table_id: String,
    pub table_name: String,
    pub field_id: String,
    pub field_name: String,
    pub linked_table_id: String,
    pub linked_table_name: String,
    /// Number of update batches planned (Airtable write batch size = 10).
    pub update_batch_count: Option<usize>,
    pub note: String,
}

/// Policy for handling attachment fields during import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAttachmentImportPolicy {
    pub table_id: String,
    pub table_name: String,
    pub field_id: String,
    pub field_name: String,
    pub policy: RestoreAttachmentRestorePolicy,
    pub note: String,
}

/// Policy for a single field during record import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordFieldPolicy {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    pub policy: RestoreRecordFieldImportPolicy,
    pub note: String,
}

/// Checkpoint/resume plan for a table's import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordImportCheckpointPlan {
    pub table_id: String,
    pub table_name: String,
    /// Batch index at which a checkpoint would be saved.
    pub checkpoint_batch_index: usize,
    /// Placeholder for the source record ID offset at checkpoint time.
    pub source_record_id_offset_placeholder: String,
    /// The phase that would be marked complete at this checkpoint.
    pub completed_phase: RestoreRecordBatchPhase,
    pub note: String,
}

/// Import plan for a single table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreTableImportPlan {
    pub table_id: String,
    pub table_name: String,
    /// Import order index — tables are imported in dependency order.
    pub import_order: usize,
    /// Known record count from the package manifest (None if not available).
    pub record_count: Option<usize>,
    pub record_count_known: bool,
    /// Batch size used for planning. Airtable write batch limit is 10.
    pub batch_size: usize,
    /// Number of first-pass create batches (None if record count unknown).
    pub create_batch_count: Option<usize>,
    /// Number of second-pass update batches (None if record count unknown).
    pub update_batch_count: Option<usize>,
    pub first_pass_batches: Vec<RestoreRecordBatchPlan>,
    pub second_pass_batches: Vec<RestoreRecordBatchPlan>,
    pub field_policies: Vec<RestoreRecordFieldPolicy>,
    pub attachment_policies: Vec<RestoreAttachmentImportPolicy>,
    pub mapping_plan: RestoreRecordMappingPlan,
    pub checkpoint_plan: RestoreRecordImportCheckpointPlan,
    pub linked_record_updates: Vec<RestoreLinkedRecordUpdatePlan>,
}

/// Retry and backoff assumptions for record import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRetryPolicy {
    /// Maximum number of retries per batch on rate-limit (429) responses.
    pub max_retries_on_rate_limit: usize,
    /// Initial backoff in milliseconds before retry.
    pub initial_backoff_ms: usize,
    /// Backoff multiplier for exponential backoff.
    pub backoff_multiplier: f32,
    pub note: String,
}

/// A warning generated during record import planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordImportWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
}

/// An error that blocks record import planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordImportError {
    pub code: String,
    pub message: String,
}

/// Input for the record import planning command.
///
/// - No token — record import planning requires no Airtable access.
/// - package_filename is filename only; never a full path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordImportPlanRequest {
    /// Filename from the most recent package inspection. Never a path.
    pub package_filename: String,
    /// Dry-run plan status for gate check.
    pub dry_run_status: String,
    /// Schema plan status for gate check.
    pub schema_plan_status: String,
    pub target_mode: RestoreTargetMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_name: Option<String>,
    /// Tables with record counts derived from dry-run plan / package summary.
    #[serde(default)]
    pub tables: Vec<RecordImportTableInput>,
}

/// Per-table input for record import planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordImportTableInput {
    pub table_id: String,
    pub table_name: String,
    /// Record count from the package manifest. None if unknown.
    pub record_count: Option<usize>,
    /// Fields in this table, used to compute per-field policies.
    #[serde(default)]
    pub fields: Vec<RecordImportFieldInput>,
}

/// Per-field input for record import planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordImportFieldInput {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    #[serde(default)]
    pub linked_table_id: Option<String>,
}

/// Full record import plan.
///
/// - No Airtable calls.
/// - No writes.
/// - No token.
/// - no_changes_made is always true.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordImportPlan {
    /// Filename only — never the full path.
    pub filename: String,
    pub status: RestoreRecordImportPlanStatus,
    pub target_mode: RestoreTargetMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_name: Option<String>,
    /// Ordered table import plans.
    pub table_plans: Vec<RestoreTableImportPlan>,
    /// Global linked record update plans (all tables combined).
    pub linked_record_update_plans: Vec<RestoreLinkedRecordUpdatePlan>,
    /// Retry and backoff policy assumptions.
    pub retry_policy: RestoreRetryPolicy,
    pub warnings: Vec<RestoreRecordImportWarning>,
    pub errors: Vec<RestoreRecordImportError>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
}
