use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldSummary {
    pub id: String,
    pub name: String,
    pub field_type: String,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSummary {
    pub id: String,
    pub name: String,
    pub field_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u32>,
    pub fields: Vec<FieldSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseSummary {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub table_count: u32,
    pub tables: Vec<TableSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub id: String,
    pub name: String,
    pub base_count: u32,
}
