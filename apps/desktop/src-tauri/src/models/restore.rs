use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreMode {
    NewBase,
    EmptyExistingBase,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestorePlanStatus {
    Draft,
    Validated,
    Incompatible,
    Ready,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreJobStatus {
    Pending,
    Running,
    DryRunComplete,
    Succeeded,
    Failed,
    Cancelled,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCompatibilityWarning {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePlanSummary {
    pub id: String,
    pub package_id: String,
    pub connection_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_id: Option<String>,
    pub mode: RestoreMode,
    pub status: RestorePlanStatus,
    pub warnings: Vec<RestoreCompatibilityWarning>,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreJobSummary {
    pub id: String,
    pub plan_id: String,
    pub connection_id: String,
    pub is_dry_run: bool,
    pub status: RestoreJobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub tables_restored: u32,
    pub total_tables: u32,
    pub records_restored: u32,
    pub skipped_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}
