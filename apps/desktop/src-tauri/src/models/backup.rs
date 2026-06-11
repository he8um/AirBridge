use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupScope {
    Full,
    SchemaOnly,
    RecordsOnly,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPackageSummary {
    pub id: String,
    pub connection_id: String,
    pub base_id: String,
    pub workspace_id: String,
    pub base_name: String,
    pub scope: BackupScope,
    pub status: BackupStatus,
    pub table_count: u32,
    pub record_count: u32,
    pub file_size_bytes: u64,
    pub created_at: String,
    pub output_path: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobSummary {
    pub id: String,
    pub connection_id: String,
    pub base_id: String,
    pub base_name: String,
    pub scope: BackupScope,
    pub status: BackupStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub tables_processed: u32,
    pub total_tables: u32,
    pub records_processed: u32,
}
