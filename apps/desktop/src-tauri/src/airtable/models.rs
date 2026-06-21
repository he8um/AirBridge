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

// ── Catalog summary models (Session 11) ────────────────────────────────────

/// Summary of a single accessible base, safe to return to the frontend.
/// Never contains the token or any secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibleBaseSummary {
    pub id: String,
    pub name: String,
}

/// Counts of fields grouped by compatibility classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldTypeCount {
    pub field_type: String,
    pub count: usize,
}

/// Compatibility summary counts across all fields in a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaCompatibilitySummary {
    pub restorable_count: usize,
    pub metadata_only_count: usize,
    pub unknown_count: usize,
    pub total_count: usize,
}

/// Summary of a single table within a base schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSchemaSummary {
    pub id: String,
    pub name: String,
    pub field_count: usize,
    pub field_type_counts: Vec<FieldTypeCount>,
    pub compatibility: SchemaCompatibilitySummary,
}

/// Full schema summary for a base, safe to return to the frontend.
/// Contains structural metadata only — no record values, no token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseSchemaSummary {
    pub base_id: String,
    pub table_count: usize,
    pub tables: Vec<TableSchemaSummary>,
    pub compatibility: SchemaCompatibilitySummary,
}

// ── Sandbox schema write models ────────────────────────────────────────────

/// Minimal request body for creating a table via the Airtable Metadata API.
///
/// Used only in the sandbox schema write integration test.
/// Never contains a token, record payload, attachment URL, or raw HTTP body.
/// Never called from app runtime or Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTableRequest {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<CreateTableFieldSpec>,
}

/// Minimal field spec within a `CreateTableRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTableFieldSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
}

/// Outcome of a `create_table` call.
///
/// Safety properties:
/// - No token field.
/// - No raw HTTP response body.
/// - No record IDs.
/// - No attachment URLs.
/// - Does not expose the new table's full field list (only name and id).
/// - Never returned to UI or Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTableOutcome {
    /// The new table's Airtable-assigned ID.
    pub table_id: String,
    /// The name provided in the request.
    pub table_name: String,
}

// ── Sandbox record write models ────────────────────────────────────────────

/// Minimal request body for creating a single record via the Airtable Records API.
///
/// Used only in the sandbox record write integration test.
/// Never contains a token, attachment URL, or raw HTTP body.
/// Never called from app runtime or Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSandboxRecordRequest {
    /// Fields map — only safe minimal string values. No linked fields. No attachments.
    pub fields: std::collections::HashMap<String, serde_json::Value>,
}

/// Sanitized outcome of a single sandbox record create call.
///
/// Safety properties:
/// - No token field.
/// - No raw HTTP response body.
/// - No old or new Airtable record IDs exposed in public interface.
/// - No attachment URLs.
/// - No linked field values.
/// - Never returned to UI or Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSandboxRecordOutcome {
    /// Whether the record create call returned a non-empty ID (sanitized boolean).
    pub record_created: bool,
    /// Count of records created (always 1 for a single create). No ID exposed.
    pub record_count: usize,
    /// Table name the record was created in (not a live Airtable table ID).
    pub table_name: String,
}

// ── Sandbox linked update models ────────────────────────────────────────────

/// Request for performing a single minimal linked record update via the Airtable
/// Records API (PATCH).
///
/// Used only in the sandbox linked update integration test.
/// Never called from app runtime or Tauri commands.
///
/// Safety properties:
/// - No token field.
/// - No raw HTTP body.
/// - No attachment URL.
/// - The `source_record_id` is an opaque internal handle for the test — it is
///   never included in serialized outcomes or assertion messages.
/// - Only the linked field is updated. No other field updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLinkedSandboxRecordRequest {
    /// Opaque source record ID — used only as the PATCH target.
    /// Never printed, never serialized into outcome structs.
    pub source_record_id: String,
    /// Name of the linked field in the source table.
    /// Not sensitive — it is a field name, not a token or record ID.
    pub linked_field_name: String,
    /// List of target record IDs to set in the linked field.
    /// These are opaque handles for the live call only.
    /// Never included in sanitized outcomes.
    pub target_record_ids: Vec<String>,
}

/// Sanitized outcome of a single sandbox linked record update call.
///
/// Safety properties:
/// - No token field.
/// - No raw HTTP response body.
/// - No old or new Airtable record IDs in the public interface.
/// - No attachment URLs.
/// - Never returned to UI or Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLinkedSandboxRecordOutcome {
    /// Whether the update call returned a non-empty record response (sanitized boolean).
    pub record_updated: bool,
    /// Count of records returned in the update response (always 1 for a single update).
    pub record_count: usize,
    /// Source table name for the linked field (not a live record ID).
    pub source_table_name: String,
    /// Name of the linked field that was updated.
    pub linked_field_name: String,
    /// Count of target IDs that were linked. No raw IDs.
    pub linked_target_count: usize,
}
