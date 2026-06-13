use serde::{Deserialize, Serialize};

use crate::restore::record_write_requests::{RecordWriteBlockedReason, RecordWriteOperationStatus};
use crate::restore::write_result::RestoreWriteDisabledReason;

/// Request for the record write request plan preview command.
///
/// No token field — record write planning does not require Airtable access.
/// The package_filename is used for display only. The full package path is
/// not accepted and is never echoed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteRequestPlanRequest {
    /// Filename from the most recent record import plan. Never echoed as a path.
    pub package_filename: String,
    /// Serialised record import plan status ("ready" | "readyWithWarnings" | "blocked").
    pub record_import_plan_status: String,
    /// Number of tables in the record import plan.
    #[serde(default)]
    pub table_count: usize,
    /// Total planned first-pass create batches across all tables.
    #[serde(default)]
    pub total_first_pass_batches: usize,
    /// Total planned second-pass linked-record update batches across all tables.
    #[serde(default)]
    pub total_second_pass_batches: usize,
    /// Number of attachment field entries.
    #[serde(default)]
    pub attachment_field_count: usize,
    /// Number of skipped (computed/read-only) field entries.
    #[serde(default)]
    pub skipped_field_count: usize,
}

/// Result returned by the `preview_record_write_request_plan` command.
///
/// Safety properties:
/// - No token field.
/// - No absolute paths — filename only.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - status is never `succeeded`.
/// - No raw record payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteRequestPlanResult {
    /// Filename only — never the full package path.
    pub filename: String,
    pub status: RecordWriteOperationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<RecordWriteBlockedReason>,
    pub disabled_reason: Option<RestoreWriteDisabledReason>,
    pub message: String,
    pub create_batch_op_count: usize,
    pub linked_update_op_count: usize,
    pub checkpoint_op_count: usize,
    pub attachment_op_count: usize,
    pub skipped_field_op_count: usize,
    pub total_op_count: usize,
    pub total_first_pass_batches: usize,
    pub total_second_pass_batches: usize,
    pub warnings: Vec<String>,
    /// Always true — no Airtable writes were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
}

impl RecordWriteRequestPlanResult {
    /// Constructs a disabled result indicating the write gate prevents execution.
    pub fn disabled(filename: String, total_op_count: usize, message: String) -> Self {
        RecordWriteRequestPlanResult {
            filename,
            status: RecordWriteOperationStatus::Disabled,
            blocked_reason: None,
            disabled_reason: Some(RestoreWriteDisabledReason::DisabledByProductPolicy),
            message,
            create_batch_op_count: 0,
            linked_update_op_count: 0,
            checkpoint_op_count: 0,
            attachment_op_count: 0,
            skipped_field_op_count: 0,
            total_op_count,
            total_first_pass_batches: 0,
            total_second_pass_batches: 0,
            warnings: vec![],
            no_changes_made: true,
            network_writes_attempted: false,
        }
    }

    /// Constructs a blocked result for an invalid or missing plan.
    pub fn blocked(filename: String, reason: RecordWriteBlockedReason, message: String) -> Self {
        RecordWriteRequestPlanResult {
            filename,
            status: RecordWriteOperationStatus::Blocked,
            blocked_reason: Some(reason),
            disabled_reason: None,
            message,
            create_batch_op_count: 0,
            linked_update_op_count: 0,
            checkpoint_op_count: 0,
            attachment_op_count: 0,
            skipped_field_op_count: 0,
            total_op_count: 0,
            total_first_pass_batches: 0,
            total_second_pass_batches: 0,
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
        let r = RecordWriteRequestPlanResult::disabled(
            "backup.airbridge".to_string(),
            5,
            "disabled".to_string(),
        );
        assert!(r.no_changes_made);
        assert!(!r.network_writes_attempted);
        assert_eq!(r.status, RecordWriteOperationStatus::Disabled);
    }

    #[test]
    fn blocked_result_no_changes_made_true() {
        let r = RecordWriteRequestPlanResult::blocked(
            "backup.airbridge".to_string(),
            RecordWriteBlockedReason::RecordImportPlanNotReady,
            "blocked".to_string(),
        );
        assert!(r.no_changes_made);
        assert!(!r.network_writes_attempted);
        assert_eq!(r.status, RecordWriteOperationStatus::Blocked);
    }

    #[test]
    fn result_has_no_token_field() {
        let r = RecordWriteRequestPlanResult::disabled(
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
        let r = RecordWriteRequestPlanResult::disabled(
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
        let req = RecordWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            record_import_plan_status: "ready".to_string(),
            table_count: 2,
            total_first_pass_batches: 4,
            total_second_pass_batches: 1,
            attachment_field_count: 1,
            skipped_field_count: 2,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn result_has_no_absolute_path() {
        let r = RecordWriteRequestPlanResult::disabled(
            "backup.airbridge".to_string(),
            0,
            "disabled".to_string(),
        );
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn disabled_result_operations_executed_implicit_zero() {
        let r = RecordWriteRequestPlanResult::disabled(
            "backup.airbridge".to_string(),
            10,
            "disabled".to_string(),
        );
        // total_op_count is set, but create/linked counts are 0 — no ops executed
        assert_eq!(r.create_batch_op_count, 0);
        assert_eq!(r.linked_update_op_count, 0);
    }
}
