use serde::{Deserialize, Serialize};

use crate::restore::schema_write_requests::{SchemaWriteBlockedReason, SchemaWriteOperationStatus};
use crate::restore::write_result::RestoreWriteDisabledReason;

/// Request for the schema write request plan preview command.
///
/// No token field — schema write planning does not require Airtable access.
/// The package_filename is used for display only. The full package path is
/// not accepted and is never echoed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteRequestPlanRequest {
    /// Filename from the most recent schema plan. Never echoed as a path.
    pub package_filename: String,
    /// Serialised schema plan status ("ready" | "readyWithWarnings" | "blocked").
    pub schema_plan_status: String,
    /// Number of tables in the existing schema plan.
    #[serde(default)]
    pub table_count: usize,
    /// Number of directly-creatable fields.
    #[serde(default)]
    pub direct_field_count: usize,
    /// Number of deferred linked fields.
    #[serde(default)]
    pub deferred_field_count: usize,
    /// Number of fields requiring manual action.
    #[serde(default)]
    pub manual_action_count: usize,
}

/// Result returned by the `preview_schema_write_request_plan` command.
///
/// Safety properties:
/// - No token field.
/// - No absolute paths — filename only.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - status is never `succeeded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteRequestPlanResult {
    /// Filename only — never the full package path.
    pub filename: String,
    pub status: SchemaWriteOperationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<SchemaWriteBlockedReason>,
    pub disabled_reason: Option<RestoreWriteDisabledReason>,
    pub message: String,
    pub table_op_count: usize,
    pub field_op_count: usize,
    pub deferred_op_count: usize,
    pub manual_action_count: usize,
    pub total_op_count: usize,
    pub warnings: Vec<String>,
    /// Always true — no Airtable writes were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
}

impl SchemaWriteRequestPlanResult {
    /// Constructs a disabled result indicating the write gate prevents execution.
    pub fn disabled(filename: String, total_op_count: usize, message: String) -> Self {
        SchemaWriteRequestPlanResult {
            filename,
            status: SchemaWriteOperationStatus::Disabled,
            blocked_reason: None,
            disabled_reason: Some(RestoreWriteDisabledReason::DisabledByProductPolicy),
            message,
            table_op_count: 0,
            field_op_count: 0,
            deferred_op_count: 0,
            manual_action_count: 0,
            total_op_count,
            warnings: vec![],
            no_changes_made: true,
            network_writes_attempted: false,
        }
    }

    /// Constructs a blocked result for an invalid or missing plan.
    pub fn blocked(filename: String, reason: SchemaWriteBlockedReason, message: String) -> Self {
        SchemaWriteRequestPlanResult {
            filename,
            status: SchemaWriteOperationStatus::Blocked,
            blocked_reason: Some(reason),
            disabled_reason: None,
            message,
            table_op_count: 0,
            field_op_count: 0,
            deferred_op_count: 0,
            manual_action_count: 0,
            total_op_count: 0,
            warnings: vec![],
            no_changes_made: true,
            network_writes_attempted: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_result_no_changes_made_true() {
        let r = SchemaWriteRequestPlanResult::disabled(
            "backup.airbridge".to_string(),
            5,
            "disabled".to_string(),
        );
        assert!(r.no_changes_made);
        assert!(!r.network_writes_attempted);
        assert_eq!(r.status, SchemaWriteOperationStatus::Disabled);
    }

    #[test]
    fn blocked_result_no_changes_made_true() {
        let r = SchemaWriteRequestPlanResult::blocked(
            "backup.airbridge".to_string(),
            SchemaWriteBlockedReason::SchemaPlanNotReady,
            "blocked".to_string(),
        );
        assert!(r.no_changes_made);
        assert!(!r.network_writes_attempted);
        assert_eq!(r.status, SchemaWriteOperationStatus::Blocked);
    }

    #[test]
    fn result_has_no_token_field() {
        let r = SchemaWriteRequestPlanResult::disabled(
            "backup.airbridge".to_string(),
            0,
            "disabled".to_string(),
        );
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn result_has_no_succeeded_status() {
        let r = SchemaWriteRequestPlanResult::disabled(
            "backup.airbridge".to_string(),
            0,
            "disabled".to_string(),
        );
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn request_struct_has_no_token_field() {
        let req = SchemaWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            schema_plan_status: "ready".to_string(),
            table_count: 2,
            direct_field_count: 5,
            deferred_field_count: 1,
            manual_action_count: 0,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }
}
