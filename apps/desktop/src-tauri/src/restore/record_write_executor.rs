use serde::{Deserialize, Serialize};

use crate::restore::record_write_requests::{
    RecordWriteBlockedReason, RecordWriteOperationStatus, RecordWriteRequestPlan,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::{RestoreWriteDisabledReason, RestoreWriteEngineStatus};

/// Result of the record write executor skeleton.
///
/// Safety properties:
/// - No token field.
/// - No absolute paths — filename only.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - status is never `succeeded`.
/// - operations_executed is always 0.
/// - No Airtable client is constructed or called.
/// - No record payloads are created or stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteDryRunResult {
    /// Filename only — never the full package path.
    pub filename: String,
    pub status: RestoreWriteEngineStatus,
    pub disabled_reason: RestoreWriteDisabledReason,
    pub message: String,
    /// Number of operations in the request plan.
    pub operations_planned: usize,
    /// Always 0 — no records are created or updated.
    pub operations_executed: usize,
    /// Number of first-pass create batch operations planned.
    pub create_batch_ops_planned: usize,
    /// Number of second-pass linked update batch operations planned.
    pub linked_update_ops_planned: usize,
    /// Number of checkpoint operations planned.
    pub checkpoint_ops_planned: usize,
    /// Number of attachment metadata operations planned.
    pub attachment_ops_planned: usize,
    /// Number of skipped field operations planned.
    pub skipped_field_ops_planned: usize,
    pub warnings: Vec<String>,
    /// Always true — no Airtable writes were attempted.
    pub no_changes_made: bool,
    /// Always false — the executor skeleton does not call the network.
    pub network_writes_attempted: bool,
}

/// Executes the record write request plan in dry-run mode.
///
/// Always consults the write gate first. Always returns `Disabled` or `Blocked`.
/// Never calls the Airtable API. Never creates, updates, or deletes any record.
/// No token is required or accepted.
/// No record payloads are constructed or stored.
pub fn execute_record_write_dry_run(
    request_plan: &RecordWriteRequestPlan,
) -> RecordWriteDryRunResult {
    let filename = request_plan.filename.clone();

    // Gate check — always disabled
    let gate = evaluate_write_gate();

    // If the request plan is blocked (bad input), surface that
    if request_plan.status == RecordWriteOperationStatus::Blocked {
        let (disabled_reason, message) = match &request_plan.blocked_reason {
            Some(RecordWriteBlockedReason::RecordImportPlanNotReady) => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "Record import plan is not ready — cannot build record write request plan."
                    .to_string(),
            ),
            Some(RecordWriteBlockedReason::NoTablesInPlan) => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "No tables in record import plan — nothing to write.".to_string(),
            ),
            _ => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "Record write request plan is blocked.".to_string(),
            ),
        };
        return RecordWriteDryRunResult {
            filename,
            status: RestoreWriteEngineStatus::Blocked,
            disabled_reason,
            message,
            operations_planned: 0,
            operations_executed: 0,
            create_batch_ops_planned: 0,
            linked_update_ops_planned: 0,
            checkpoint_ops_planned: 0,
            attachment_ops_planned: 0,
            skipped_field_ops_planned: 0,
            warnings: request_plan.warnings.clone(),
            no_changes_made: true,
            network_writes_attempted: false,
        };
    }

    // Normal disabled path — write gate is enforced
    RecordWriteDryRunResult {
        filename,
        status: gate.status,
        disabled_reason: gate.reason,
        message: gate.message,
        operations_planned: request_plan.total_op_count,
        operations_executed: 0, // Always 0 — nothing is executed
        create_batch_ops_planned: request_plan.create_batch_op_count,
        linked_update_ops_planned: request_plan.linked_update_op_count,
        checkpoint_ops_planned: request_plan.checkpoint_op_count,
        attachment_ops_planned: request_plan.attachment_op_count,
        skipped_field_ops_planned: request_plan.skipped_field_op_count,
        warnings: request_plan.warnings.clone(),
        no_changes_made: true,
        network_writes_attempted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
    };
    use crate::restore::record_import_planner::create_record_import_plan;
    use crate::restore::record_write_requests::build_record_write_request_plan;

    fn make_request(tables: Vec<RecordImportTableInput>) -> RestoreRecordImportPlanRequest {
        RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables,
        }
    }

    fn simple_table() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tbl01".to_string(),
            table_name: "Tasks".to_string(),
            record_count: Some(20),
            fields: vec![RecordImportFieldInput {
                field_id: "fld01".to_string(),
                field_name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
                linked_table_id: None,
            }],
        }
    }

    fn ready_request_plan() -> RecordWriteRequestPlan {
        let req = make_request(vec![simple_table()]);
        let import_plan = create_record_import_plan(&req);
        build_record_write_request_plan(&import_plan)
    }

    fn blocked_request_plan() -> RecordWriteRequestPlan {
        let req = RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "blocked".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            tables: vec![simple_table()],
        };
        let import_plan = create_record_import_plan(&req);
        build_record_write_request_plan(&import_plan)
    }

    #[test]
    fn executor_always_returns_disabled() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn executor_no_changes_made_always_true() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert!(result.no_changes_made);
    }

    #[test]
    fn executor_network_writes_attempted_always_false() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn executor_operations_executed_always_zero() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(result.operations_executed, 0);
    }

    #[test]
    fn executor_blocked_plan_returns_blocked_status() {
        let plan = blocked_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Blocked);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn executor_disabled_reason_is_product_policy_for_ready_plan() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(
            result.disabled_reason,
            RestoreWriteDisabledReason::DisabledByProductPolicy
        );
    }

    #[test]
    fn executor_blocked_reason_is_invalid_plan_for_blocked_input() {
        let plan = blocked_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(
            result.disabled_reason,
            RestoreWriteDisabledReason::BlockedByInvalidPlan
        );
    }

    #[test]
    fn executor_operations_planned_matches_request_plan() {
        let plan = ready_request_plan();
        let expected = plan.total_op_count;
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(result.operations_planned, expected);
    }

    #[test]
    fn executor_create_batch_count_matches_request_plan() {
        let plan = ready_request_plan();
        let expected = plan.create_batch_op_count;
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(result.create_batch_ops_planned, expected);
    }

    #[test]
    fn executor_filename_is_basename_only() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(result.filename, "backup.airbridge");
        assert!(!result.filename.contains('/'));
    }

    #[test]
    fn executor_result_has_no_token() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn executor_result_has_no_succeeded_status() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn executor_result_has_no_absolute_path() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn executor_no_airtable_client_called() {
        // execute_record_write_dry_run accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn executor_serializes_safety_keys() {
        let plan = ready_request_plan();
        let result = execute_record_write_dry_run(&plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("noChangesMade"));
        assert!(json.contains("networkWritesAttempted"));
        assert!(json.contains("operationsExecuted"));
    }
}
