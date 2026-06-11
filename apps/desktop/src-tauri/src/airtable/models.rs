use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ── Identifier wrappers ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AirtableBaseId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AirtableTableId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AirtableRecordId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AirtableFieldId(pub String);

impl AirtableBaseId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AirtableTableId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AirtableRecordId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ── Schema models ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableBase {
    pub id: AirtableBaseId,
    pub name: String,
    pub permission_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableTable {
    pub id: AirtableTableId,
    pub name: String,
    pub primary_field_id: Option<AirtableFieldId>,
    pub fields: Vec<AirtableField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableField {
    pub id: AirtableFieldId,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    /// Raw options blob — shape varies by field type.
    pub options: Option<Value>,
}

// ── Record models ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableRecord {
    pub id: AirtableRecordId,
    pub fields: HashMap<String, Value>,
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableListRecordsResponse {
    pub records: Vec<AirtableRecord>,
    /// Present when more pages are available.
    pub offset: Option<String>,
}

// ── Write request models ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableRecordFields {
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableCreateRecordsRequest {
    pub records: Vec<AirtableRecordFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableRecordUpdate {
    pub id: AirtableRecordId,
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableUpdateRecordsRequest {
    pub records: Vec<AirtableRecordUpdate>,
}

// ── API error response ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirtableErrorResponse {
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub message: Option<String>,
}

// ── Connection check result ────────────────────────────────────────────────

/// A base entry returned in a connection check result.
/// Contains only public, non-sensitive metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibleBase {
    pub id: AirtableBaseId,
    pub name: String,
}

/// Structured result from a connection check via list-bases endpoint.
/// Never contains the token or any secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCheckOutcome {
    /// Bases visible to the token, from list-bases response.
    pub accessible_bases: Vec<AccessibleBase>,
}

/// Partial list-bases API response shape, used to parse the connection check.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBasesResponse {
    pub bases: Vec<ListBasesEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBasesEntry {
    pub id: String,
    pub name: String,
}
