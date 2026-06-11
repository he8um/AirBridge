use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReportType {
    Backup,
    Restore,
    Validation,
    Compatibility,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReportSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportItem {
    pub id: String,
    pub severity: ReportSeverity,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSummary {
    pub id: String,
    pub report_type: ReportType,
    pub title: String,
    pub created_at: String,
    pub severity: ReportSeverity,
    pub item_count: u32,
    pub items: Vec<ReportItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_base_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_base_name: Option<String>,
}
