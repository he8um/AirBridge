use serde::{Deserialize, Serialize};

/// Scope of the backup plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupScope {
    Full,
    SchemaOnly,
    RecordsOnly,
}

/// Severity of a backup plan warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// Policy for attachment fields in V0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentPolicy {
    MetadataOnly,
}

/// Policy for linked record fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedRecordPolicy {
    ReferencesCaptured,
    RemappingRequiredForRestore,
}

/// Estimated number of record-read API pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum RecordReadEstimate {
    Known(usize),
    Unknown,
}

/// Per-field entry in a backup plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanField {
    pub id: String,
    pub name: String,
    pub field_type: String,
    /// One of: "restorable", "metadataOnly", "unknown"
    pub compatibility: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_policy: Option<AttachmentPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_record_policy: Option<LinkedRecordPolicy>,
}

/// A single warning or note associated with the backup plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanWarning {
    pub severity: WarningSeverity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
}

/// Compatibility summary counts for a table or whole base.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanCompatibilitySummary {
    pub restorable_count: usize,
    pub metadata_only_count: usize,
    pub unknown_count: usize,
    pub total_count: usize,
}

/// API request estimates for executing this backup plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanEstimate {
    pub schema_requests: usize,
    pub record_read_pages: RecordReadEstimate,
    pub note: String,
}

/// Per-table entry in a backup plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanTable {
    pub id: String,
    pub name: String,
    pub field_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<usize>,
    pub fields: Vec<BackupPlanField>,
    pub warnings: Vec<BackupPlanWarning>,
    pub compatibility: BackupPlanCompatibilitySummary,
}

/// Input request for creating a backup plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanRequest {
    pub base_id: String,
    pub base_name: String,
    pub scope: BackupScope,
    /// Serialised table list; produced by the schema read flow.
    pub tables: Vec<BackupPlanTableInput>,
}

/// Per-table input for planning; mirrors the schema summary shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanTableInput {
    pub id: String,
    pub name: String,
    pub fields: Vec<BackupPlanFieldInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<usize>,
}

/// Per-field input for planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlanFieldInput {
    pub id: String,
    pub name: String,
    pub field_type: String,
}

/// A complete backup plan — dry-run only.
///
/// `output_package_path` is always `None` at this stage: no file is written.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlan {
    pub base_id: String,
    pub base_name: String,
    pub scope: BackupScope,
    pub table_count: usize,
    pub total_field_count: usize,
    pub tables: Vec<BackupPlanTable>,
    pub compatibility: BackupPlanCompatibilitySummary,
    pub warnings: Vec<BackupPlanWarning>,
    pub estimate: BackupPlanEstimate,
    /// Always `true` in this phase — no backup file is created.
    pub dry_run: bool,
    /// Always `None` in this phase — no output file is written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_package_path: Option<String>,
}
