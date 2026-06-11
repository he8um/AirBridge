use serde::{Deserialize, Serialize};

/// Whether the dry-run plan is ready, ready with warnings, or blocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestorePlanStatus {
    Ready,
    ReadyWithWarnings,
    Blocked,
}

/// Target mode for restore.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreTargetMode {
    NewBase,
    EmptyExistingBase,
}

/// Field-level restore compatibility classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreFieldCompatibility {
    Supported,
    PartiallySupported,
    MetadataOnly,
    Unsupported,
    ManualActionRequired,
}

/// Package-level summary extracted from the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePackageSummary {
    pub filename: String,
    pub format: String,
    pub format_version: String,
    pub app_version: String,
    pub created_at: String,
    pub provider: String,
    pub base_id: String,
    pub base_name: String,
    pub table_count: usize,
    pub field_count: usize,
    pub record_count: usize,
    pub contains_record_data: bool,
    pub contains_attachment_urls: bool,
    pub encrypted: bool,
}

/// Plan for restoring a single field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFieldPlan {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    pub compatibility: RestoreFieldCompatibility,
    pub note: String,
}

/// Plan for handling linked record references across tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreLinkedRecordPlan {
    pub field_id: String,
    pub field_name: String,
    pub linked_table_id: String,
    pub remapping_required: bool,
    pub note: String,
}

/// Plan for handling attachment fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAttachmentPlan {
    pub field_id: String,
    pub field_name: String,
    /// Always true in V0.1 — file content is not re-uploaded.
    pub metadata_only: bool,
    pub note: String,
}

/// Plan for restoring a single table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreTablePlan {
    pub table_id: String,
    pub table_name: String,
    pub field_count: usize,
    pub record_count: usize,
    pub fields: Vec<RestoreFieldPlan>,
    pub linked_record_plans: Vec<RestoreLinkedRecordPlan>,
    pub attachment_plans: Vec<RestoreAttachmentPlan>,
    pub restorable_field_count: usize,
    pub partial_field_count: usize,
    pub unsupported_field_count: usize,
}

/// Describes the order in which restore operations would be applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecordOrderingPlan {
    /// Step 1: create table schemas.
    pub create_tables_first: bool,
    /// Step 2: create fields within tables.
    pub create_fields_after_tables: bool,
    /// Step 3: import records without linked-record references.
    pub import_records_without_links: bool,
    /// Step 4: apply linked-record references after all records exist and ID remapping is done.
    pub apply_links_after_records: bool,
    pub note: String,
}

/// A warning generated during dry-run planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDryRunWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
}

/// An error that blocks dry-run planning from completing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDryRunError {
    pub code: String,
    pub message: String,
}

/// Input for creating a restore dry-run plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDryRunRequest {
    /// Absolute path to the `.airbridge` package. Never echoed in the result.
    pub path: String,
    pub target_mode: RestoreTargetMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_name: Option<String>,
}

/// Full dry-run restore plan. No Airtable calls. No writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDryRunPlan {
    /// Filename only — never the full path.
    pub filename: String,
    pub status: RestorePlanStatus,
    pub target_mode: RestoreTargetMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_name: Option<String>,
    pub package_summary: Option<RestorePackageSummary>,
    pub tables: Vec<RestoreTablePlan>,
    pub ordering: Option<RestoreRecordOrderingPlan>,
    pub warnings: Vec<RestoreDryRunWarning>,
    pub errors: Vec<RestoreDryRunError>,
    /// Always present — states explicitly that no Airtable changes were made.
    pub no_changes_made: bool,
}
