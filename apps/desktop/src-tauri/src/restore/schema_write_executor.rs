use serde::{Deserialize, Serialize};

use crate::restore::schema_write_requests::{
    SchemaWriteBlockedReason, SchemaWriteOperationKind, SchemaWriteOperationStatus,
    SchemaWriteRequestPlan,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::{RestoreWriteDisabledReason, RestoreWriteEngineStatus};

// ── Legacy dry-run result (preserved for backward compatibility) ──────────────

/// Result of the schema write executor dry-run skeleton.
///
/// Safety properties:
/// - No token field.
/// - No absolute paths — filename only.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - status is never `succeeded`.
/// - No Airtable client is constructed or called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteDryRunResult {
    /// Filename only — never the full package path.
    pub filename: String,
    pub status: RestoreWriteEngineStatus,
    pub disabled_reason: RestoreWriteDisabledReason,
    pub message: String,
    pub operations_planned: usize,
    pub operations_executed: usize,
    pub table_ops_planned: usize,
    pub field_ops_planned: usize,
    pub deferred_ops_planned: usize,
    pub manual_action_count: usize,
    pub warnings: Vec<String>,
    /// Always true — no Airtable writes were attempted.
    pub no_changes_made: bool,
    /// Always false — the executor skeleton does not call the network.
    pub network_writes_attempted: bool,
}

/// Executes the schema write request plan in dry-run mode (legacy skeleton).
///
/// Always consults the write gate first. Always returns `Disabled` or `Blocked`.
/// Never calls the Airtable API. Never creates a base, table, or field.
/// No token is required or accepted.
pub fn execute_schema_write_dry_run(
    request_plan: &SchemaWriteRequestPlan,
) -> SchemaWriteDryRunResult {
    let filename = request_plan.filename.clone();

    // Gate check — always disabled
    let gate = evaluate_write_gate();

    // If the request plan itself is blocked (bad input), surface that
    if request_plan.status == SchemaWriteOperationStatus::Blocked {
        let (disabled_reason, message) = match &request_plan.blocked_reason {
            Some(SchemaWriteBlockedReason::SchemaPlanNotReady) => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "Schema plan is not ready — cannot build write request plan.".to_string(),
            ),
            Some(SchemaWriteBlockedReason::NoTablesInPlan) => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "No tables in schema plan — nothing to write.".to_string(),
            ),
            _ => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "Schema write request plan is blocked.".to_string(),
            ),
        };
        return SchemaWriteDryRunResult {
            filename,
            status: RestoreWriteEngineStatus::Blocked,
            disabled_reason,
            message,
            operations_planned: 0,
            operations_executed: 0,
            table_ops_planned: 0,
            field_ops_planned: 0,
            deferred_ops_planned: 0,
            manual_action_count: 0,
            warnings: request_plan.warnings.clone(),
            no_changes_made: true,
            network_writes_attempted: false,
        };
    }

    // Normal disabled path — write gate is enforced
    SchemaWriteDryRunResult {
        filename,
        status: gate.status,
        disabled_reason: gate.reason,
        message: gate.message,
        operations_planned: request_plan.total_op_count,
        operations_executed: 0, // Always 0 — nothing is executed
        table_ops_planned: request_plan.table_op_count,
        field_ops_planned: request_plan.field_op_count,
        deferred_ops_planned: request_plan.deferred_op_count,
        manual_action_count: request_plan.manual_action_count,
        warnings: request_plan.warnings.clone(),
        no_changes_made: true,
        network_writes_attempted: false,
    }
}

// ── Richer executor foundation ────────────────────────────────────────────────

/// Execution status for the schema write executor foundation.
///
/// Safety invariants:
/// - `DryRunOnly` does NOT enable live writes.
/// - `NotExecuted` is the expected state when the write gate is disabled.
/// - `Blocked` indicates a safety prerequisite is missing.
/// - No status named `succeeded`, `complete`, or `done` exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteExecutorStatus {
    /// All prerequisites satisfied but the write gate is disabled.
    /// This is the current expected state — no execution occurs.
    NotExecuted,
    /// Dry-run plan built; execution would be sandbox-only.
    /// Write gate must be explicitly enabled before this transitions to execution.
    DryRunOnly,
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Execution mode for the schema write executor.
///
/// Safety invariants:
/// - `Disabled` is the only reachable mode in the current implementation.
/// - `SandboxOnly` is defined for future use but is unreachable while
///   `evaluate_write_gate()` returns `Disabled`.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteExecutorMode {
    /// Write gate is disabled — no execution is possible. Default state.
    Disabled,
    /// Sandbox-only mode — execution is restricted to sandbox targets.
    /// Unreachable in the current implementation.
    SandboxOnly,
}

/// Status of a single step in the executor step list.
///
/// Note: `succeeded` / `completed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteExecutorStepStatus {
    /// The step would be executed if the gate were enabled. Not executed.
    Pending,
    /// The step is blocked by a safety prerequisite.
    Blocked,
    /// The step is skipped (e.g. no linked fields to defer).
    Skipped,
}

/// A single ordered step in the executor's internal plan.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No record payload.
/// - No raw HTTP body.
/// - No old or new Airtable record IDs.
/// - No attachment URL.
/// - `status` is never `succeeded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteExecutorStep {
    pub step_index: usize,
    pub step_id: String,
    /// Stable operation kind label for diagnostics.
    pub operation_kind: String,
    pub table_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
    pub status: SchemaWriteExecutorStepStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the executor foundation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteExecutorSafetySnapshot {
    /// Write gate result — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the mode is sandbox-only (always `false` in the current build).
    pub sandbox_mode_active: bool,
    /// Whether the explicit internal schema write flag was set.
    pub explicit_internal_write_requested: bool,
    /// Whether sandbox verification passed.
    pub sandbox_verified: bool,
    /// Whether target empty verification passed.
    pub target_empty_verified: bool,
    /// Whether live-write readiness is ready or warning-safe.
    pub live_write_readiness_satisfied: bool,
}

/// Request to the schema write executor foundation.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_schema_write_requested` must be `true` for the executor
/// to proceed past the gate check. It is an internal-only guard — there is no
/// UI control that sets it, and the write gate must also allow schema writes
/// (which it currently never does).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteExecutorRequest {
    /// Must be `sandboxOnly` for execution to be considered.
    /// `disabled` (the default) always results in `NotExecuted`.
    pub mode: SchemaWriteExecutorMode,
    /// Internal-only flag. Must be explicitly `true` to proceed past the gate.
    /// No UI control sets this; it is an internal safety guard.
    pub explicit_internal_schema_write_requested: bool,
    /// Whether the sandbox environment check has passed.
    pub sandbox_verified: bool,
    /// Whether the target empty verification has passed.
    pub target_empty_verified: bool,
    /// Whether live-write readiness is ready or warning-safe.
    pub live_write_readiness_satisfied: bool,
}

/// Result of the schema write executor foundation.
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
/// - `NotExecuted` / `DryRunOnly` do NOT enable live schema writes.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteExecutorResult {
    pub status: SchemaWriteExecutorStatus,
    pub mode: SchemaWriteExecutorMode,
    pub message: String,
    pub steps: Vec<SchemaWriteExecutorStep>,
    pub safety_snapshot: SchemaWriteExecutorSafetySnapshot,
    pub table_step_count: usize,
    pub field_step_count: usize,
    pub deferred_step_count: usize,
    pub manual_step_count: usize,
    pub total_step_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — live writes are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const PREREQ_WRITE_GATE: &str = "SWEX-PRE-01";
const PREREQ_MODE: &str = "SWEX-PRE-02";
const PREREQ_EXPLICIT_FLAG: &str = "SWEX-PRE-03";
const PREREQ_SANDBOX: &str = "SWEX-PRE-04";
const PREREQ_TARGET_EMPTY: &str = "SWEX-PRE-05";
const PREREQ_LWR: &str = "SWEX-PRE-06";
const PREREQ_REQUEST_PLAN: &str = "SWEX-PRE-07";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the schema write executor foundation plan.
///
/// This function:
/// - Never calls the Airtable API.
/// - Never creates a base, table, or field.
/// - Always enforces the write gate (`evaluate_write_gate()` always returns Disabled).
/// - Always returns `writes_enabled: false`, `no_changes_made: true`,
///   `network_writes_attempted: false`.
/// - Returns `Blocked` when any prerequisite is missing.
/// - Returns `NotExecuted` when all prerequisites are met but the gate is Disabled.
/// - Returns `DryRunOnly` only when mode is `SandboxOnly`, the explicit flag is set,
///   all safety prerequisites pass, AND the write gate permits schema writes.
///   Since `evaluate_write_gate()` currently always returns `Disabled`, `DryRunOnly`
///   is currently unreachable.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_schema_write_executor_plan(
    request: &SchemaWriteExecutorRequest,
    request_plan: &SchemaWriteRequestPlan,
) -> SchemaWriteExecutorResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let safety_snapshot = SchemaWriteExecutorSafetySnapshot {
        write_gate_disabled,
        sandbox_mode_active: matches!(request.mode, SchemaWriteExecutorMode::SandboxOnly),
        explicit_internal_write_requested: request.explicit_internal_schema_write_requested,
        sandbox_verified: request.sandbox_verified,
        target_empty_verified: request.target_empty_verified,
        live_write_readiness_satisfied: request.live_write_readiness_satisfied,
    };

    // Check prerequisites in order; first failure blocks.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        // Defense-in-depth: this branch is unreachable given evaluate_write_gate()
        // always returns Disabled, but retained as a hard guard.
        Some(format!(
            "{PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Schema write executor must not proceed while write gate could be enabled."
        ))
    } else if !matches!(request.mode, SchemaWriteExecutorMode::SandboxOnly) {
        Some(format!(
            "{PREREQ_MODE}: Executor mode must be sandboxOnly. \
             Mode 'disabled' does not permit execution. \
             No schema writes will be attempted."
        ))
    } else if !request.explicit_internal_schema_write_requested {
        Some(format!(
            "{PREREQ_EXPLICIT_FLAG}: Explicit internal schema write flag is not set. \
             The internal flag must be explicitly true before execution is considered. \
             No UI control sets this flag."
        ))
    } else if !request.sandbox_verified {
        Some(format!(
            "{PREREQ_SANDBOX}: Sandbox environment verification has not passed. \
             A verified sandbox target is required before schema writes are considered."
        ))
    } else if !request.target_empty_verified {
        Some(format!(
            "{PREREQ_TARGET_EMPTY}: Target empty verification has not passed. \
             The target base must be confirmed empty before schema writes are considered."
        ))
    } else if !request.live_write_readiness_satisfied {
        Some(format!(
            "{PREREQ_LWR}: Live-write readiness policy is not satisfied. \
             All upstream safety gates must be ready or warning-safe."
        ))
    } else if request_plan.status == SchemaWriteOperationStatus::Blocked {
        Some(format!(
            "{PREREQ_REQUEST_PLAN}: Schema write request plan is blocked. \
             A ready plan is required before schema writes are considered."
        ))
    } else {
        None
    };

    if let Some(ref reason) = blocked_reason {
        return SchemaWriteExecutorResult {
            status: SchemaWriteExecutorStatus::Blocked,
            mode: SchemaWriteExecutorMode::Disabled,
            message: format!(
                "Schema write executor is blocked. {reason} \
                 No Airtable API calls were made. \
                 No changes were made. \
                 Live schema writes remain unavailable."
            ),
            steps: vec![],
            safety_snapshot,
            table_step_count: 0,
            field_step_count: 0,
            deferred_step_count: 0,
            manual_step_count: 0,
            total_step_count: 0,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the ordered step list from the request plan.
    // Since the write gate currently always returns Disabled, execution is still not
    // performed. Steps are built as `Pending` descriptors only — no network calls made.
    let mut steps: Vec<SchemaWriteExecutorStep> = Vec::new();
    let mut step_index = 0usize;
    let mut table_step_count = 0usize;
    let mut field_step_count = 0usize;
    let mut deferred_step_count = 0usize;
    let mut manual_step_count = 0usize;

    for op in &request_plan.operations {
        let (step_id, operation_kind, field_name, field_type, note) = match op.kind {
            SchemaWriteOperationKind::CreateTable => (
                format!("SWEX-TBL-{step_index:03}"),
                "createTable".to_string(),
                None,
                None,
                format!(
                    "Create table '{}' — pending (write gate disabled).",
                    op.table_name
                ),
            ),
            SchemaWriteOperationKind::CreateField => (
                format!("SWEX-FLD-{step_index:03}"),
                "createField".to_string(),
                op.field_name.clone(),
                op.field_type.clone(),
                format!(
                    "Create field '{}' ({}) in '{}' — pending (write gate disabled).",
                    op.field_name.as_deref().unwrap_or("?"),
                    op.field_type.as_deref().unwrap_or("?"),
                    op.table_name
                ),
            ),
            SchemaWriteOperationKind::DeferLinkedField => (
                format!("SWEX-DEF-{step_index:03}"),
                "deferLinkedField".to_string(),
                op.field_name.clone(),
                op.field_type.clone(),
                format!(
                    "Linked field '{}' in '{}' deferred — all tables must exist first.",
                    op.field_name.as_deref().unwrap_or("?"),
                    op.table_name
                ),
            ),
            SchemaWriteOperationKind::ManualAction => (
                format!("SWEX-MAN-{step_index:03}"),
                "manualAction".to_string(),
                op.field_name.clone(),
                op.field_type.clone(),
                format!(
                    "Field '{}' ({}) requires manual action in '{}'.",
                    op.field_name.as_deref().unwrap_or("?"),
                    op.field_type.as_deref().unwrap_or("?"),
                    op.table_name
                ),
            ),
            SchemaWriteOperationKind::CreateBase => (
                format!("SWEX-BASE-{step_index:03}"),
                "createBase".to_string(),
                None,
                None,
                "Create base operation — pending (write gate disabled).".to_string(),
            ),
        };

        match op.kind {
            SchemaWriteOperationKind::CreateTable | SchemaWriteOperationKind::CreateBase => {
                table_step_count += 1;
            }
            SchemaWriteOperationKind::CreateField => {
                field_step_count += 1;
            }
            SchemaWriteOperationKind::DeferLinkedField => {
                deferred_step_count += 1;
            }
            SchemaWriteOperationKind::ManualAction => {
                manual_step_count += 1;
            }
        }

        steps.push(SchemaWriteExecutorStep {
            step_index,
            step_id,
            operation_kind,
            table_name: op.table_name.clone(),
            field_name,
            field_type,
            status: SchemaWriteExecutorStepStatus::Pending,
            note,
        });
        step_index += 1;
    }

    let total_step_count = steps.len();

    // Since evaluate_write_gate() currently always returns Disabled, we return
    // NotExecuted even when all prerequisites are met. DryRunOnly would be
    // returned if a future gate enabled sandbox writes.
    SchemaWriteExecutorResult {
        status: SchemaWriteExecutorStatus::NotExecuted,
        mode: SchemaWriteExecutorMode::Disabled,
        message: format!(
            "Schema write executor plan built ({total_step_count} step(s): \
             {table_step_count} table(s), {field_step_count} direct field(s), \
             {deferred_step_count} deferred field(s), {manual_step_count} manual action(s)). \
             Execution is not performed: write gate is disabled by product policy. \
             No Airtable API calls were made. No changes were made. \
             Live schema writes remain unavailable. \
             Sandbox-only execution is not yet reachable."
        ),
        steps,
        safety_snapshot,
        table_step_count,
        field_step_count,
        deferred_step_count,
        manual_step_count,
        total_step_count,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::schema_plan::RestoreSchemaPlan;
    use crate::restore::schema_plan::{
        RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreManualActionField,
        RestoreSchemaDependencyGraph, RestoreSchemaPlanStatus, RestoreTableCreationStep,
    };
    use crate::restore::schema_write_requests::build_schema_write_request_plan;

    fn simple_schema_plan() -> RestoreSchemaPlan {
        RestoreSchemaPlan {
            filename: "backup.airbridge".to_string(),
            status: RestoreSchemaPlanStatus::Ready,
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            table_steps: vec![RestoreTableCreationStep {
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
                step_index: 0,
                field_count: 2,
                direct_field_count: 2,
                deferred_field_count: 0,
                manual_action_count: 0,
                unsupported_count: 0,
                note: "Create table 'Tasks'.".to_string(),
            }],
            field_steps: vec![RestoreFieldCreationStep {
                field_id: "fld001".to_string(),
                field_name: "Title".to_string(),
                field_type: "singleLineText".to_string(),
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
                classification: RestoreFieldCreateClassification::CreateDirectly,
                note: "Direct field.".to_string(),
            }],
            deferred_steps: vec![],
            manual_action_fields: vec![RestoreManualActionField {
                field_id: "fld002".to_string(),
                field_name: "Formula".to_string(),
                field_type: "formula".to_string(),
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
                action_description: "Recreate manually.".to_string(),
            }],
            dependency_graph: RestoreSchemaDependencyGraph {
                edges: vec![],
                has_circular_dependency: false,
                resolution_note: String::new(),
            },
            warnings: vec![],
            errors: vec![],
            no_changes_made: true,
        }
    }

    fn sandbox_request() -> SchemaWriteExecutorRequest {
        SchemaWriteExecutorRequest {
            mode: SchemaWriteExecutorMode::SandboxOnly,
            explicit_internal_schema_write_requested: true,
            sandbox_verified: true,
            target_empty_verified: true,
            live_write_readiness_satisfied: true,
        }
    }

    fn disabled_request() -> SchemaWriteExecutorRequest {
        SchemaWriteExecutorRequest {
            mode: SchemaWriteExecutorMode::Disabled,
            explicit_internal_schema_write_requested: false,
            sandbox_verified: false,
            target_empty_verified: false,
            live_write_readiness_satisfied: false,
        }
    }

    // ── Legacy dry-run skeleton tests (preserved) ─────────────────────────────

    #[test]
    fn executor_always_returns_disabled() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn executor_no_changes_made_always_true() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert!(result.no_changes_made);
    }

    #[test]
    fn executor_network_writes_attempted_always_false() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn executor_operations_executed_always_zero() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.operations_executed, 0);
    }

    #[test]
    fn executor_result_has_no_token() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn executor_result_has_no_succeeded_status() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn executor_result_has_no_absolute_path() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn executor_disabled_reason_is_product_policy() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(
            result.disabled_reason,
            RestoreWriteDisabledReason::DisabledByProductPolicy
        );
    }

    #[test]
    fn executor_filename_is_basename_only() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.filename, "backup.airbridge");
        assert!(!result.filename.contains('/'));
    }

    #[test]
    fn executor_blocked_plan_returns_blocked_status() {
        let mut blocked_plan = build_schema_write_request_plan(&simple_schema_plan());
        blocked_plan.status = SchemaWriteOperationStatus::Blocked;
        blocked_plan.blocked_reason = Some(SchemaWriteBlockedReason::SchemaPlanNotReady);
        let result = execute_schema_write_dry_run(&blocked_plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Blocked);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn executor_no_airtable_client_called() {
        // execute_schema_write_dry_run accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn executor_operations_planned_matches_request_plan() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let expected_total = request_plan.total_op_count;
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.operations_planned, expected_total);
    }

    #[test]
    fn executor_serializes_no_changes_made_key() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("noChangesMade"));
        assert!(json.contains("networkWritesAttempted"));
    }

    // ── Foundation: safety invariants ─────────────────────────────────────────

    #[test]
    fn foundation_writes_enabled_always_false_blocked() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&disabled_request(), &plan);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn foundation_network_writes_attempted_always_false_blocked() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&disabled_request(), &plan);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_no_changes_made_always_true_blocked() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&disabled_request(), &plan);
        assert!(result.no_changes_made);
    }

    #[test]
    fn foundation_write_gate_still_disabled_after_plan() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let _ = build_schema_write_executor_plan(&sandbox_request(), &plan);
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── Foundation: blocked because mode is disabled ──────────────────────────

    #[test]
    fn foundation_blocked_when_mode_disabled() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&disabled_request(), &plan);
        assert_eq!(result.status, SchemaWriteExecutorStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_MODE));
    }

    #[test]
    fn foundation_blocked_when_explicit_flag_not_set() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let req = SchemaWriteExecutorRequest {
            mode: SchemaWriteExecutorMode::SandboxOnly,
            explicit_internal_schema_write_requested: false,
            sandbox_verified: true,
            target_empty_verified: true,
            live_write_readiness_satisfied: true,
        };
        let result = build_schema_write_executor_plan(&req, &plan);
        assert_eq!(result.status, SchemaWriteExecutorStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_EXPLICIT_FLAG));
    }

    #[test]
    fn foundation_blocked_when_sandbox_not_verified() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let req = SchemaWriteExecutorRequest {
            mode: SchemaWriteExecutorMode::SandboxOnly,
            explicit_internal_schema_write_requested: true,
            sandbox_verified: false,
            target_empty_verified: true,
            live_write_readiness_satisfied: true,
        };
        let result = build_schema_write_executor_plan(&req, &plan);
        assert_eq!(result.status, SchemaWriteExecutorStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_SANDBOX));
    }

    #[test]
    fn foundation_blocked_when_target_not_empty() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let req = SchemaWriteExecutorRequest {
            mode: SchemaWriteExecutorMode::SandboxOnly,
            explicit_internal_schema_write_requested: true,
            sandbox_verified: true,
            target_empty_verified: false,
            live_write_readiness_satisfied: true,
        };
        let result = build_schema_write_executor_plan(&req, &plan);
        assert_eq!(result.status, SchemaWriteExecutorStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_TARGET_EMPTY));
    }

    #[test]
    fn foundation_blocked_when_live_write_readiness_not_satisfied() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let req = SchemaWriteExecutorRequest {
            mode: SchemaWriteExecutorMode::SandboxOnly,
            explicit_internal_schema_write_requested: true,
            sandbox_verified: true,
            target_empty_verified: true,
            live_write_readiness_satisfied: false,
        };
        let result = build_schema_write_executor_plan(&req, &plan);
        assert_eq!(result.status, SchemaWriteExecutorStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_LWR));
    }

    #[test]
    fn foundation_blocked_when_request_plan_blocked() {
        let mut plan = build_schema_write_request_plan(&simple_schema_plan());
        plan.status = SchemaWriteOperationStatus::Blocked;
        plan.blocked_reason = Some(SchemaWriteBlockedReason::SchemaPlanNotReady);
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert_eq!(result.status, SchemaWriteExecutorStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_REQUEST_PLAN));
    }

    // ── Foundation: not-executed when prerequisites satisfied (gate disabled) ──

    #[test]
    fn foundation_not_executed_when_all_prereqs_met() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert_eq!(result.status, SchemaWriteExecutorStatus::NotExecuted);
    }

    #[test]
    fn foundation_mode_disabled_in_not_executed_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert_eq!(result.mode, SchemaWriteExecutorMode::Disabled);
    }

    #[test]
    fn foundation_writes_enabled_false_in_not_executed_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn foundation_no_changes_made_in_not_executed_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert!(result.no_changes_made);
    }

    #[test]
    fn foundation_network_writes_not_attempted_in_not_executed_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_steps_built_in_not_executed_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        // simple_schema_plan produces: 1 table, 1 direct field, 0 deferred, 1 manual
        assert_eq!(result.table_step_count, 1);
        assert_eq!(result.field_step_count, 1);
        assert_eq!(result.manual_step_count, 1);
        assert_eq!(result.total_step_count, 3);
    }

    #[test]
    fn foundation_steps_all_pending_in_not_executed_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        for step in &result.steps {
            assert_eq!(step.status, SchemaWriteExecutorStepStatus::Pending);
        }
    }

    #[test]
    fn foundation_steps_ordered_tables_first() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        // First step must be a table creation.
        assert_eq!(result.steps[0].operation_kind, "createTable");
    }

    #[test]
    fn foundation_step_ids_use_stable_prefixes() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        for step in &result.steps {
            assert!(
                step.step_id.starts_with("SWEX-TBL-")
                    || step.step_id.starts_with("SWEX-FLD-")
                    || step.step_id.starts_with("SWEX-DEF-")
                    || step.step_id.starts_with("SWEX-MAN-")
                    || step.step_id.starts_with("SWEX-BASE-"),
                "unexpected step ID prefix: {}",
                step.step_id
            );
        }
    }

    #[test]
    fn foundation_no_token_in_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn foundation_no_absolute_path_in_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn foundation_no_succeeded_state_in_result() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    #[test]
    fn foundation_safety_snapshot_write_gate_disabled_always_true() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn foundation_safety_snapshot_reflects_request() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let req = sandbox_request();
        let result = build_schema_write_executor_plan(&req, &plan);
        assert!(result.safety_snapshot.explicit_internal_write_requested);
        assert!(result.safety_snapshot.sandbox_verified);
        assert!(result.safety_snapshot.target_empty_verified);
        assert!(result.safety_snapshot.live_write_readiness_satisfied);
    }

    #[test]
    fn foundation_no_production_mode_exists() {
        // SchemaWriteExecutorMode has only Disabled and SandboxOnly.
        // Verify the json representation for a blocked (disabled-mode) result
        // does not contain the word "production".
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&disabled_request(), &plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.to_lowercase().contains("production"));
    }

    #[test]
    fn foundation_message_mentions_no_changes_made() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert!(result.message.contains("No changes were made"));
    }

    #[test]
    fn foundation_message_mentions_gate_disabled() {
        let plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = build_schema_write_executor_plan(&sandbox_request(), &plan);
        assert!(result.message.to_lowercase().contains("disabled"));
    }
}
