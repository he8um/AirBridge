use serde::{Deserialize, Serialize};

use crate::restore::plan::RestoreTargetMode;

/// Status of the schema creation plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreSchemaPlanStatus {
    Ready,
    ReadyWithWarnings,
    Blocked,
}

/// How a field will be handled during schema creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreFieldCreateClassification {
    /// Field can be created via the Airtable API immediately.
    CreateDirectly,
    /// Field can be created but some properties need adjustment.
    CreateWithAdjustment,
    /// Field must be deferred until all tables and their linked targets exist.
    DeferUntilTablesExist,
    /// Field schema is captured but values cannot be restored.
    MetadataOnly,
    /// Field requires a manual step outside the automated restore path.
    ManualActionRequired,
    /// Field type is not supported via the Airtable API.
    Unsupported,
}

/// A planned step for creating one table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreTableCreationStep {
    pub table_id: String,
    pub table_name: String,
    pub step_index: usize,
    pub field_count: usize,
    pub direct_field_count: usize,
    pub deferred_field_count: usize,
    pub manual_action_count: usize,
    pub unsupported_count: usize,
    pub note: String,
}

/// A planned step for creating one field within a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFieldCreationStep {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    pub table_id: String,
    pub table_name: String,
    pub classification: RestoreFieldCreateClassification,
    pub note: String,
}

/// A planned step for a field that must be deferred (e.g. linked records).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDeferredFieldStep {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    pub table_id: String,
    pub table_name: String,
    pub reason: String,
    pub linked_table_id: Option<String>,
}

/// A field that requires manual action outside the restore pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreManualActionField {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    pub table_id: String,
    pub table_name: String,
    pub action_description: String,
}

/// A dependency edge in the linked record dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreLinkedDependencyStep {
    pub field_id: String,
    pub field_name: String,
    pub source_table_id: String,
    pub source_table_name: String,
    pub target_table_id: String,
    pub target_table_name: String,
    pub remapping_required: bool,
    pub note: String,
}

/// The full dependency graph summary for linked record fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSchemaDependencyGraph {
    pub edges: Vec<RestoreLinkedDependencyStep>,
    pub has_circular_dependency: bool,
    pub resolution_note: String,
}

/// A warning generated during schema creation planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSchemaWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
}

/// An error that prevents a schema creation plan from being generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSchemaError {
    pub code: String,
    pub message: String,
}

/// Input for the schema creation plan command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSchemaPlanRequest {
    /// Filename from the most recent package inspection or dry-run. Never echoed as a path.
    pub package_filename: String,
    /// Serialised dry-run plan status for gate-check ("ready" | "readyWithWarnings" | "blocked").
    pub dry_run_status: String,
    /// Target mode.
    pub target_mode: RestoreTargetMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_name: Option<String>,
    /// Tables extracted from the dry-run plan for planning purposes.
    #[serde(default)]
    pub tables: Vec<SchemaPlanTableInput>,
}

/// Table data derived from a dry-run plan, used as input to the schema planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaPlanTableInput {
    pub table_id: String,
    pub table_name: String,
    #[serde(default)]
    pub fields: Vec<SchemaPlanFieldInput>,
}

/// Field data derived from a dry-run plan, used as input to the schema planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaPlanFieldInput {
    pub field_id: String,
    pub field_name: String,
    pub field_type: String,
    #[serde(default)]
    pub linked_table_id: Option<String>,
}

/// Full schema creation plan. No Airtable calls. No writes. No token required.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSchemaPlan {
    /// Filename only — never the full path.
    pub filename: String,
    pub status: RestoreSchemaPlanStatus,
    pub target_mode: RestoreTargetMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_name: Option<String>,
    /// Ordered steps for table creation (tables planned before fields).
    pub table_steps: Vec<RestoreTableCreationStep>,
    /// Ordered steps for field creation (only directly-creatable fields).
    pub field_steps: Vec<RestoreFieldCreationStep>,
    /// Fields deferred until tables and records exist.
    pub deferred_steps: Vec<RestoreDeferredFieldStep>,
    /// Fields that require manual action outside the restore pipeline.
    pub manual_action_fields: Vec<RestoreManualActionField>,
    /// Linked record dependency graph.
    pub dependency_graph: RestoreSchemaDependencyGraph,
    pub warnings: Vec<RestoreSchemaWarning>,
    pub errors: Vec<RestoreSchemaError>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
}
