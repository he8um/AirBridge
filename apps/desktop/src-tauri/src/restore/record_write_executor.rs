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

// ── Richer executor foundation ────────────────────────────────────────────────

/// Execution status for the record write executor foundation.
///
/// Safety invariants:
/// - `DryRunOnly` does NOT enable live record writes.
/// - `NotExecuted` is the expected state when the write gate is disabled.
/// - `Blocked` indicates a safety prerequisite is missing.
/// - No status named `succeeded`, `complete`, or `done` exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteExecutorStatus {
    /// All prerequisites satisfied but the write gate is disabled.
    /// This is the current expected state — no execution occurs.
    NotExecuted,
    /// Dry-run plan built; execution would be sandbox-only.
    /// Write gate must be explicitly enabled before this transitions to execution.
    DryRunOnly,
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Execution mode for the record write executor.
///
/// Safety invariants:
/// - `Disabled` is the only reachable mode in the current implementation.
/// - `SandboxOnly` is defined for future use but is unreachable while
///   `evaluate_write_gate()` returns `Disabled`.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteExecutorMode {
    /// Write gate is disabled — no execution is possible. Default state.
    Disabled,
    /// Sandbox-only mode — execution is restricted to verified sandbox targets.
    /// Unreachable in the current implementation.
    SandboxOnly,
}

/// Status of a single batch in the executor's internal plan.
///
/// Note: `succeeded` / `completed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteExecutorBatchStatus {
    /// The batch would be executed if the gate were enabled. Not executed.
    Pending,
    /// The batch is blocked by a safety prerequisite.
    Blocked,
    /// The batch is skipped (e.g. empty table, skipped phase).
    Skipped,
}

/// A single ordered batch in the executor's internal plan.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No raw record payload or field values.
/// - No raw HTTP body.
/// - No old or new Airtable record IDs.
/// - No attachment URL.
/// - `status` is never `succeeded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteExecutorBatch {
    pub batch_index: usize,
    pub batch_id: String,
    /// Class of operation: "first-pass-create" or "second-pass-linked-update".
    pub operation_class: String,
    /// Safe table label — not a live Airtable ID.
    pub table_label: String,
    /// Number of records in this batch. Always <= batch_size.
    pub record_count: usize,
    pub status: RecordWriteExecutorBatchStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the record write executor foundation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteExecutorSafetySnapshot {
    /// Write gate result — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the mode is sandbox-only (always `false` in the current build).
    pub sandbox_mode_active: bool,
    /// Whether the explicit internal record write flag was set.
    pub explicit_internal_write_requested: bool,
    /// Whether sandbox verification passed.
    pub sandbox_verified: bool,
    /// Whether target empty verification passed.
    pub target_empty_verified: bool,
    /// Whether the schema write executor foundation completed safely (NotExecuted or safe).
    pub schema_executor_safe: bool,
    /// Whether the rate-limit/backoff policy is compliant or warning-safe.
    pub rate_limit_backoff_safe: bool,
    /// Whether the checkpoint metadata store prerequisite is satisfied.
    pub checkpoint_store_safe: bool,
    /// Whether live-write readiness is ready or warning-safe.
    pub live_write_readiness_satisfied: bool,
}

/// Request to the record write executor foundation.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_record_write_requested` must be `true` for the executor
/// to proceed past the gate check. It is an internal-only guard — there is no
/// UI control that sets it, and the write gate must also allow record writes
/// (which it currently never does).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteExecutorRequest {
    /// Must be `sandboxOnly` for execution to be considered.
    /// `disabled` (the default) always results in `Blocked`.
    pub mode: RecordWriteExecutorMode,
    /// Internal-only flag. Must be explicitly `true` to proceed past the gate.
    /// No UI control sets this; it is an internal safety guard.
    pub explicit_internal_record_write_requested: bool,
    /// Whether the sandbox environment check has passed.
    pub sandbox_verified: bool,
    /// Whether the target empty verification has passed.
    pub target_empty_verified: bool,
    /// Whether the schema write executor foundation completed safely.
    pub schema_executor_safe: bool,
    /// Whether the rate-limit/backoff policy is compliant or warning-safe.
    pub rate_limit_backoff_safe: bool,
    /// Whether the checkpoint metadata store prerequisite is satisfied.
    pub checkpoint_store_safe: bool,
    /// Whether live-write readiness is ready or warning-safe.
    pub live_write_readiness_satisfied: bool,
}

/// Result of the record write executor foundation.
///
/// Safety invariants (enforced):
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `succeeded`, `complete`, or `done`.
/// - `NotExecuted` / `DryRunOnly` do NOT enable live record writes.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteExecutorResult {
    pub status: RecordWriteExecutorStatus,
    pub mode: RecordWriteExecutorMode,
    pub message: String,
    pub batches: Vec<RecordWriteExecutorBatch>,
    pub safety_snapshot: RecordWriteExecutorSafetySnapshot,
    pub first_pass_batch_count: usize,
    pub second_pass_batch_count: usize,
    pub total_batch_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — live record writes are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const RWEX_PREREQ_WRITE_GATE: &str = "RWEX-PRE-01";
const RWEX_PREREQ_MODE: &str = "RWEX-PRE-02";
const RWEX_PREREQ_EXPLICIT_FLAG: &str = "RWEX-PRE-03";
const RWEX_PREREQ_SANDBOX: &str = "RWEX-PRE-04";
const RWEX_PREREQ_TARGET_EMPTY: &str = "RWEX-PRE-05";
const RWEX_PREREQ_SCHEMA_EXECUTOR: &str = "RWEX-PRE-06";
const RWEX_PREREQ_RATE_LIMIT: &str = "RWEX-PRE-07";
const RWEX_PREREQ_CHECKPOINT_STORE: &str = "RWEX-PRE-08";
const RWEX_PREREQ_LWR: &str = "RWEX-PRE-09";

const RWEX_MAX_BATCH_SIZE: usize = 10;

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the record write executor foundation plan.
///
/// This function:
/// - Never calls the Airtable API.
/// - Never creates, updates, or deletes any record.
/// - Always enforces the write gate (`evaluate_write_gate()` always returns Disabled).
/// - Always returns `writes_enabled: false`, `no_changes_made: true`,
///   `network_writes_attempted: false`.
/// - Returns `Blocked` when any prerequisite is missing.
/// - Returns `NotExecuted` when all prerequisites are met but the gate is Disabled.
/// - Returns `DryRunOnly` only when mode is `SandboxOnly`, the explicit flag is set,
///   all safety prerequisites pass, AND the write gate permits record writes.
///   Since `evaluate_write_gate()` currently always returns `Disabled`, `DryRunOnly`
///   is currently unreachable.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_record_write_executor_plan(
    request: &RecordWriteExecutorRequest,
    request_plan: &RecordWriteRequestPlan,
) -> RecordWriteExecutorResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let safety_snapshot = RecordWriteExecutorSafetySnapshot {
        write_gate_disabled,
        sandbox_mode_active: matches!(request.mode, RecordWriteExecutorMode::SandboxOnly),
        explicit_internal_write_requested: request.explicit_internal_record_write_requested,
        sandbox_verified: request.sandbox_verified,
        target_empty_verified: request.target_empty_verified,
        schema_executor_safe: request.schema_executor_safe,
        rate_limit_backoff_safe: request.rate_limit_backoff_safe,
        checkpoint_store_safe: request.checkpoint_store_safe,
        live_write_readiness_satisfied: request.live_write_readiness_satisfied,
    };

    // Check prerequisites in order; first failure blocks.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        // Defense-in-depth: unreachable given evaluate_write_gate() always returns Disabled.
        Some(format!(
            "{RWEX_PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Record write executor must not proceed while write gate could be enabled."
        ))
    } else if !matches!(request.mode, RecordWriteExecutorMode::SandboxOnly) {
        Some(format!(
            "{RWEX_PREREQ_MODE}: Executor mode must be sandboxOnly. \
             Mode 'disabled' does not permit execution. \
             No record writes will be attempted."
        ))
    } else if !request.explicit_internal_record_write_requested {
        Some(format!(
            "{RWEX_PREREQ_EXPLICIT_FLAG}: Explicit internal record write flag is not set. \
             The internal flag must be explicitly true before execution is considered. \
             No UI control sets this flag."
        ))
    } else if !request.sandbox_verified {
        Some(format!(
            "{RWEX_PREREQ_SANDBOX}: Sandbox environment verification has not passed. \
             A verified sandbox target is required before record writes are considered."
        ))
    } else if !request.target_empty_verified {
        Some(format!(
            "{RWEX_PREREQ_TARGET_EMPTY}: Target empty verification has not passed. \
             The target base must be confirmed empty before record writes are considered."
        ))
    } else if !request.schema_executor_safe {
        Some(format!(
            "{RWEX_PREREQ_SCHEMA_EXECUTOR}: Schema write executor foundation has not completed \
             safely. Schema writes must be confirmed safe or notExecuted before record writes."
        ))
    } else if !request.rate_limit_backoff_safe {
        Some(format!(
            "{RWEX_PREREQ_RATE_LIMIT}: Rate-limit/backoff policy is not compliant or warning-safe. \
             Throttle and backoff settings must be declared before record writes are considered."
        ))
    } else if !request.checkpoint_store_safe {
        Some(format!(
            "{RWEX_PREREQ_CHECKPOINT_STORE}: Checkpoint metadata store prerequisite is not \
             satisfied. Checkpoint safety must be confirmed before record writes are considered."
        ))
    } else if !request.live_write_readiness_satisfied {
        Some(format!(
            "{RWEX_PREREQ_LWR}: Live-write readiness policy is not satisfied. \
             All upstream safety gates must be ready or warning-safe."
        ))
    } else if request_plan.status == RecordWriteOperationStatus::Blocked {
        Some(format!(
            "Record write request plan is blocked ({}). \
             Cannot build executor batches from a blocked plan.",
            request_plan
                .blocked_reason
                .as_ref()
                .map(|r| format!("{r:?}"))
                .unwrap_or_else(|| "unknown".to_string())
        ))
    } else {
        // Validate batch sizes from the request plan
        let oversized: Vec<_> = request_plan
            .operations
            .iter()
            .filter_map(|op| op.planned_record_count)
            .filter(|&c| c > RWEX_MAX_BATCH_SIZE)
            .collect();
        if !oversized.is_empty() {
            Some(format!(
                "RWEX-BATCH-SIZE: One or more planned batches exceed the safe maximum of \
                 {RWEX_MAX_BATCH_SIZE} records per batch. Oversized batch count: {}.",
                oversized.len()
            ))
        } else {
            None
        }
    };

    if let Some(ref reason) = blocked_reason {
        return RecordWriteExecutorResult {
            status: RecordWriteExecutorStatus::Blocked,
            mode: RecordWriteExecutorMode::Disabled,
            message: format!(
                "Record write executor is blocked. {reason} \
                 No record writes will be attempted."
            ),
            batches: vec![blocked_executor_batch()],
            safety_snapshot,
            first_pass_batch_count: 0,
            second_pass_batch_count: 0,
            total_batch_count: 0,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the internal batch plan.
    let batches = build_executor_batches(request_plan);
    let first_pass = batches
        .iter()
        .filter(|b| b.operation_class == "first-pass-create")
        .count();
    let second_pass = batches
        .iter()
        .filter(|b| b.operation_class == "second-pass-linked-update")
        .count();
    let total = batches.len();

    // Write gate is disabled — result is NotExecuted (not DryRunOnly).
    RecordWriteExecutorResult {
        status: RecordWriteExecutorStatus::NotExecuted,
        mode: RecordWriteExecutorMode::Disabled,
        message: format!(
            "Record write executor plan built ({first_pass} first-pass create batch(es), \
             {second_pass} second-pass linked-update batch(es), {total} total). \
             Write gate is disabled — no record writes are attempted. \
             No Airtable changes made."
        ),
        batches,
        safety_snapshot,
        first_pass_batch_count: first_pass,
        second_pass_batch_count: second_pass,
        total_batch_count: total,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn blocked_executor_batch() -> RecordWriteExecutorBatch {
    RecordWriteExecutorBatch {
        batch_index: 0,
        batch_id: "RWEX-BATCH-BLOCKED".to_string(),
        operation_class: "blocked".to_string(),
        table_label: "—".to_string(),
        record_count: 0,
        status: RecordWriteExecutorBatchStatus::Blocked,
        note: "Safety prerequisites not satisfied. No batches can be planned.".to_string(),
    }
}

fn build_executor_batches(request_plan: &RecordWriteRequestPlan) -> Vec<RecordWriteExecutorBatch> {
    use crate::restore::record_write_requests::RecordWriteOperationKind;

    let mut batches = Vec::new();
    let mut idx = 0usize;

    // Phase 1 — first-pass create batches (preserve ordering from request plan)
    for op in request_plan
        .operations
        .iter()
        .filter(|o| o.kind == RecordWriteOperationKind::CreateRecordBatch)
    {
        batches.push(RecordWriteExecutorBatch {
            batch_index: idx,
            batch_id: format!(
                "RWEX-FP-{}-B{:02}",
                op.table_id,
                op.batch_index.unwrap_or(idx)
            ),
            operation_class: "first-pass-create".to_string(),
            table_label: op.table_name.clone(),
            record_count: op.planned_record_count.unwrap_or(0),
            status: RecordWriteExecutorBatchStatus::Pending,
            note: format!(
                "Would call Airtable create-records endpoint for '{}'. \
                 Write gate disabled — no network call made.",
                op.table_name
            ),
        });
        idx += 1;
    }

    // Phase 2 — second-pass linked update batches
    for op in request_plan
        .operations
        .iter()
        .filter(|o| o.kind == RecordWriteOperationKind::UpdateLinkedRecordBatch)
    {
        batches.push(RecordWriteExecutorBatch {
            batch_index: idx,
            batch_id: format!(
                "RWEX-SP-{}-B{:02}",
                op.table_id,
                op.batch_index.unwrap_or(idx)
            ),
            operation_class: "second-pass-linked-update".to_string(),
            table_label: op.table_name.clone(),
            record_count: op.planned_record_count.unwrap_or(0),
            status: RecordWriteExecutorBatchStatus::Pending,
            note: format!(
                "Would call Airtable update-records endpoint for linked fields in '{}'. \
                 ID mapping unavailable until execution. \
                 Write gate disabled — no network call made.",
                op.table_name
            ),
        });
        idx += 1;
    }

    if batches.is_empty() {
        batches.push(RecordWriteExecutorBatch {
            batch_index: 0,
            batch_id: "RWEX-BATCH-EMPTY".to_string(),
            operation_class: "no-operations".to_string(),
            table_label: "—".to_string(),
            record_count: 0,
            status: RecordWriteExecutorBatchStatus::Skipped,
            note: "No record create or linked update operations in plan.".to_string(),
        });
    }

    batches
}

// ── Foundation tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod foundation_tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
    };
    use crate::restore::record_import_planner::create_record_import_plan;
    use crate::restore::record_write_requests::build_record_write_request_plan;

    fn make_import_request(tables: Vec<RecordImportTableInput>) -> RestoreRecordImportPlanRequest {
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

    fn linked_table() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblA".to_string(),
            table_name: "Projects".to_string(),
            record_count: Some(15),
            fields: vec![
                RecordImportFieldInput {
                    field_id: "fld01".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld02".to_string(),
                    field_name: "Tasks".to_string(),
                    field_type: "multipleRecordLinks".to_string(),
                    linked_table_id: Some("tbl01".to_string()),
                },
            ],
        }
    }

    fn ready_plan() -> RecordWriteRequestPlan {
        let req = make_import_request(vec![simple_table()]);
        let import_plan = create_record_import_plan(&req);
        build_record_write_request_plan(&import_plan)
    }

    fn linked_plan() -> RecordWriteRequestPlan {
        let req = make_import_request(vec![simple_table(), linked_table()]);
        let import_plan = create_record_import_plan(&req);
        build_record_write_request_plan(&import_plan)
    }

    fn blocked_plan() -> RecordWriteRequestPlan {
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

    fn all_prereqs_request() -> RecordWriteExecutorRequest {
        RecordWriteExecutorRequest {
            mode: RecordWriteExecutorMode::SandboxOnly,
            explicit_internal_record_write_requested: true,
            sandbox_verified: true,
            target_empty_verified: true,
            schema_executor_safe: true,
            rate_limit_backoff_safe: true,
            checkpoint_store_safe: true,
            live_write_readiness_satisfied: true,
        }
    }

    // ── Blocked when prerequisites missing ────────────────────────────────────

    #[test]
    fn foundation_blocked_when_mode_disabled() {
        let mut req = all_prereqs_request();
        req.mode = RecordWriteExecutorMode::Disabled;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-02"));
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_blocked_when_explicit_flag_not_set() {
        let mut req = all_prereqs_request();
        req.explicit_internal_record_write_requested = false;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-03"));
    }

    #[test]
    fn foundation_blocked_when_sandbox_not_verified() {
        let mut req = all_prereqs_request();
        req.sandbox_verified = false;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-04"));
    }

    #[test]
    fn foundation_blocked_when_target_not_empty() {
        let mut req = all_prereqs_request();
        req.target_empty_verified = false;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-05"));
    }

    #[test]
    fn foundation_blocked_when_schema_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.schema_executor_safe = false;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-06"));
    }

    #[test]
    fn foundation_blocked_when_rate_limit_not_safe() {
        let mut req = all_prereqs_request();
        req.rate_limit_backoff_safe = false;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-07"));
    }

    #[test]
    fn foundation_blocked_when_checkpoint_store_not_safe() {
        let mut req = all_prereqs_request();
        req.checkpoint_store_safe = false;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-08"));
    }

    #[test]
    fn foundation_blocked_when_live_write_readiness_not_satisfied() {
        let mut req = all_prereqs_request();
        req.live_write_readiness_satisfied = false;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("RWEX-PRE-09"));
    }

    #[test]
    fn foundation_blocked_when_request_plan_blocked() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &blocked_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::Blocked);
        assert!(result.blocked_reason.is_some());
    }

    // ── NotExecuted when all prerequisites met ────────────────────────────────

    #[test]
    fn foundation_not_executed_when_all_prereqs_met() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::NotExecuted);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn foundation_write_gate_still_disabled_after_plan() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn foundation_safety_snapshot_write_gate_disabled_always_true() {
        // Even in blocked state, write_gate_disabled must be true
        let mut req = all_prereqs_request();
        req.mode = RecordWriteExecutorMode::Disabled;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── No production mode ────────────────────────────────────────────────────

    #[test]
    fn foundation_no_production_mode_exists() {
        // Confirm RecordWriteExecutorMode only has Disabled and SandboxOnly
        let disabled = RecordWriteExecutorMode::Disabled;
        let sandbox = RecordWriteExecutorMode::SandboxOnly;
        assert_ne!(disabled, sandbox);
        // Serialization must not contain "production"
        let json = serde_json::to_string(&disabled).expect("serialize");
        assert!(!json.contains("production"));
        let json = serde_json::to_string(&sandbox).expect("serialize");
        assert!(!json.contains("production"));
    }

    // ── Batch ordering ────────────────────────────────────────────────────────

    #[test]
    fn foundation_batches_built_in_not_executed_result() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::NotExecuted);
        assert!(!result.batches.is_empty());
        assert!(result.first_pass_batch_count > 0);
    }

    #[test]
    fn foundation_batches_ordered_first_pass_before_second_pass() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &linked_plan());
        let fp_last = result
            .batches
            .iter()
            .filter(|b| b.operation_class == "first-pass-create")
            .map(|b| b.batch_index)
            .max();
        let sp_first = result
            .batches
            .iter()
            .find(|b| b.operation_class == "second-pass-linked-update")
            .map(|b| b.batch_index);
        if let (Some(last_fp), Some(first_sp)) = (fp_last, sp_first) {
            assert!(
                last_fp < first_sp,
                "first-pass batches must precede second-pass batches"
            );
        }
    }

    #[test]
    fn foundation_batch_indices_are_sequential() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &linked_plan());
        for (i, batch) in result.batches.iter().enumerate() {
            assert_eq!(batch.batch_index, i, "batch_index must be sequential");
        }
    }

    #[test]
    fn foundation_batch_ordering_is_deterministic() {
        let r1 = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        let r2 = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        let ids1: Vec<_> = r1.batches.iter().map(|b| &b.batch_id).collect();
        let ids2: Vec<_> = r2.batches.iter().map(|b| &b.batch_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn foundation_batch_size_never_exceeds_max() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        for batch in &result.batches {
            assert!(
                batch.record_count <= RWEX_MAX_BATCH_SIZE || batch.record_count == 0,
                "batch {} record_count {} exceeds max {}",
                batch.batch_id,
                batch.record_count,
                RWEX_MAX_BATCH_SIZE
            );
        }
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn foundation_no_token_in_result() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn foundation_no_absolute_path_in_result() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn foundation_no_record_payload_in_result() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn foundation_no_succeeded_in_serialization() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn foundation_no_attachment_url_in_result() {
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn foundation_no_airtable_client_called() {
        // build_record_write_executor_plan accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let result = build_record_write_executor_plan(&all_prereqs_request(), &ready_plan());
        assert_eq!(result.status, RecordWriteExecutorStatus::NotExecuted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_no_network_writes_in_blocked_state() {
        let mut req = all_prereqs_request();
        req.mode = RecordWriteExecutorMode::Disabled;
        let result = build_record_write_executor_plan(&req, &ready_plan());
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }
}
