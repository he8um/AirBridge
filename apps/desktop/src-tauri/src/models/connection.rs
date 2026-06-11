use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionStatus {
    Disconnected,
    Checking,
    Connected,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionCheckStatus {
    Unknown,
    Checking,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionCheck {
    pub key: String,
    pub label: String,
    pub status: PermissionCheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProfile {
    pub id: String,
    pub label: String,
    pub status: ConnectionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
    pub permissions: Vec<PermissionCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCheckResult {
    pub connection_id: String,
    pub status: ConnectionStatus,
    pub permissions: Vec<PermissionCheck>,
    /// Bases visible to the token. Populated on successful check. Never contains the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessible_bases: Option<Vec<AccessibleBaseSummaryInResult>>,
}

/// Minimal base entry returned inside a `ConnectionCheckResult`.
/// Intentionally separate from `AccessibleBaseSummary` to keep the connection
/// result shape stable even if the catalog model evolves.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibleBaseSummaryInResult {
    pub id: String,
    pub name: String,
}
