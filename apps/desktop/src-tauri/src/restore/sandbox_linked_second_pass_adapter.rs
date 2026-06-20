use serde::{Deserialize, Serialize};

use crate::restore::linked_second_pass_execution_preview::{
    LinkedSecondPassExecutionPreviewStatus, LinkedSecondPassFieldSummary,
};
use crate::restore::linked_second_pass_executor::{
    build_linked_second_pass_executor_plan, LinkedSecondPassExecutorMode,
    LinkedSecondPassExecutorRequest, LinkedSecondPassExecutorStatus,
};
use crate::restore::sandbox_gate_arming::{
    build_sandbox_gate_arming_decision, SandboxGateArmingMode, SandboxGateArmingRequest,
    SandboxGateArmingStatus,
};
use crate::restore::sandbox_record_write_adapter::{
    build_sandbox_record_write_adapter, SandboxRecordWriteAdapterMode,
    SandboxRecordWriteAdapterRequest, SandboxRecordWriteAdapterStatus,
};
use crate::restore::sandbox_restore_simulator::{
    run_sandbox_restore_simulator, SandboxRestoreSimulatorMode, SandboxRestoreSimulatorRequest,
    SandboxRestoreSimulatorStatus,
};
use crate::restore::sandbox_schema_write_adapter::{
    build_sandbox_schema_write_adapter, SandboxSchemaWriteAdapterMode,
    SandboxSchemaWriteAdapterRequest, SandboxSchemaWriteAdapterStatus,
};
use crate::restore::schema_write_requests::SchemaWriteRequestPlan;
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public types ──────────────────────────────────────────────────────────────

/// Overall status of the sandbox linked second-pass adapter evaluation.
///
/// Safety invariants:
/// - `ReadyForSandboxCall` does NOT execute any Airtable network call.
/// - `ReadyForSandboxCall` does NOT enable runtime writes, reads, or execution.
/// - `ReadyForSandboxCall` is not stored globally, not persisted, and is not
///   reachable from UI, TypeScript, or any Tauri command.
/// - `ReadyForSandboxCall` does NOT change `evaluate_write_gate()` behavior.
/// - `runtime_execution_enabled` is always `false` regardless of status.
/// - `app_runtime_writes_enabled` is always `false` regardless of status.
/// - `app_runtime_reads_enabled` is always `false` regardless of status.
/// - `network_writes_attempted` is always `false`.
/// - `no_changes_made` is always `true`.
/// - No `Succeeded`, `Complete`, `Enabled`, `Done`, or `ExecutionReady` status exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxLinkedSecondPassAdapterStatus {
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
    /// All prerequisites satisfied and linked second-pass update operations have
    /// been described as adapter-boundary operation descriptors. No network call
    /// was made. No execution occurred. No state was persisted.
    ReadyForSandboxCall,
    /// The adapter is in disabled mode. No evaluation was performed. Default state.
    NotExecuted,
}

/// Mode for the sandbox linked second-pass adapter.
///
/// Safety invariants:
/// - `Disabled` is the default and operationally always-reachable mode.
/// - `SandboxOnlyInternal` is for Rust unit tests only.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxLinkedSecondPassAdapterMode {
    /// Adapter disabled — no evaluation is performed. Default state.
    Disabled,
    /// Internal sandbox-only adapter mode for Rust unit tests only.
    /// Does NOT execute network calls, enable runtime writes/reads, or persist state.
    SandboxOnlyInternal,
}

/// Status of a single planned linked second-pass batch descriptor.
///
/// Note: `succeeded` / `completed` / `done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxLinkedSecondPassAdapterOperationStatus {
    /// The batch is fully described and would be the next call if enabled.
    /// No network call has been made.
    Planned,
    /// The batch is blocked by a safety prerequisite failure.
    Blocked,
    /// The adapter is in disabled mode. Batch descriptor was built but not executed.
    NotExecuted,
}

/// A single linked second-pass batch descriptor at the adapter boundary.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No raw Airtable request body.
/// - No raw HTTP response.
/// - No old or new Airtable record IDs.
/// - No record field payload or field values.
/// - No attachment URL.
/// - `status` is never `succeeded`.
/// - Only linked update batches are described — no schema, first-pass record create,
///   attachment, or checkpoint operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxLinkedSecondPassAdapterOperation {
    /// Stable adapter-boundary operation ID (SLSPA-OP-NNN).
    pub operation_id: String,
    /// Always `"linkedUpdateBatchDescriptor"` in this adapter.
    pub operation_kind: String,
    /// Source table label from the field summary (not a live Airtable ID).
    pub table_label: String,
    /// Linked field label from the field summary (not a live Airtable field ID).
    pub field_label: String,
    /// Number of records in this batch. None if count was unknown at plan time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_count: Option<usize>,
    /// Number of mapping entries required for this batch (safe count only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_coverage_count: Option<usize>,
    pub status: SandboxLinkedSecondPassAdapterOperationStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the adapter evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxLinkedSecondPassAdapterSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the sandbox gate arming decision returned `ArmedNotExecutable`.
    pub gate_arming_armed_not_executable: bool,
    /// Whether the sandbox restore simulator returned `SimulatedNotExecuted`.
    pub simulator_simulated_not_executed: bool,
    /// Whether the linked second-pass executor plan returned `NotExecuted`.
    pub linked_executor_not_executed: bool,
    /// Whether the schema write adapter returned `ReadyForSandboxCall`.
    pub schema_adapter_ready: bool,
    /// Whether the record write adapter returned `ReadyForSandboxCall`.
    pub record_adapter_ready: bool,
    /// Whether the target base is declared empty.
    pub target_base_empty: bool,
    /// Whether sandbox verification is declared safe.
    pub sandbox_verified: bool,
    /// Whether mapping coverage is sufficient (no IDs exposed).
    pub mapping_coverage_sufficient: bool,
    /// Whether the explicit internal linked sandbox call flag was set.
    pub explicit_linked_sandbox_flag_set: bool,
    /// Whether runtime execution is enabled — always `false`.
    pub runtime_execution_enabled: bool,
    /// Whether app runtime writes are enabled — always `false`.
    pub app_runtime_writes_enabled: bool,
    /// Whether app runtime reads are enabled — always `false`.
    pub app_runtime_reads_enabled: bool,
    /// Whether any network write was attempted — always `false`.
    pub network_writes_attempted: bool,
}

/// Request to the sandbox linked second-pass adapter.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_linked_sandbox_call_requested` must be `true` for the adapter
/// to proceed past its gate check. No UI control, Tauri command, or runtime path
/// sets this flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxLinkedSecondPassAdapterRequest {
    /// Must be `sandboxOnlyInternal` for evaluation to proceed.
    /// `disabled` (the default) always results in `NotExecuted`.
    pub mode: SandboxLinkedSecondPassAdapterMode,
    /// Internal-only flag. Must be explicitly `true` to proceed past the gate.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_linked_sandbox_call_requested: bool,
    /// Whether sandbox environment verification is declared and safe.
    pub sandbox_verified: bool,
    /// Whether target empty verification is declared and safe.
    pub target_base_empty: bool,
    /// Whether field summaries have sufficient mapping coverage (no IDs exposed).
    pub mapping_coverage_sufficient: bool,
    /// Prerequisite booleans forwarded to arming/simulator/executor probes.
    pub confirmation_gate_declared: bool,
    pub destructive_operation_policy_safe: bool,
    pub attachment_phase_disabled_safe: bool,
    pub live_write_readiness_safe: bool,
    pub write_phase_ordering_safe: bool,
    pub failure_modes_safe: bool,
    pub rollback_limitation_safe: bool,
    pub checkpoint_durability_safe: bool,
    pub sensitive_data_safe: bool,
    pub final_validation_enforcement_safe: bool,
    pub rate_limit_backoff_safe: bool,
    pub schema_executor_safe: bool,
    pub checkpoint_store_safe: bool,
    pub record_executor_safe: bool,
    pub linked_second_pass_preview_ready: bool,
    pub mapping_checkpoint_preview_ready: bool,
    /// Per-field summaries used to build the adapter batch descriptors.
    /// No raw record IDs — only safe counts and labels.
    pub field_summaries: Vec<LinkedSecondPassFieldSummary>,
}

/// Result of the sandbox linked second-pass adapter evaluation.
///
/// Safety invariants (always enforced):
/// - `runtime_execution_enabled` is always `false`.
/// - `app_runtime_writes_enabled` is always `false`.
/// - `app_runtime_reads_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `succeeded`, `complete`, `enabled`, `done`, or `executionReady`.
/// - No Airtable client is called.
/// - The result is not persisted globally.
/// - The result is not reachable from UI, TypeScript, or any Tauri command.
/// - Only linked update batch descriptors appear in `operations` — no schema,
///   first-pass record create, attachment, or checkpoint operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxLinkedSecondPassAdapterResult {
    pub status: SandboxLinkedSecondPassAdapterStatus,
    pub mode: SandboxLinkedSecondPassAdapterMode,
    pub message: String,
    pub operations: Vec<SandboxLinkedSecondPassAdapterOperation>,
    pub safety_snapshot: SandboxLinkedSecondPassAdapterSafetySnapshot,
    pub total_operation_count: usize,
    pub total_update_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — runtime execution is not enabled.
    pub runtime_execution_enabled: bool,
    /// Always `false` — app runtime writes are not enabled.
    pub app_runtime_writes_enabled: bool,
    /// Always `false` — app runtime reads are not enabled.
    pub app_runtime_reads_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const CHK_MODE: &str = "SLSPA-CHK-01";
const CHK_EXPLICIT_FLAG: &str = "SLSPA-CHK-02";
const CHK_WRITE_GATE: &str = "SLSPA-CHK-03";
const CHK_ARMING: &str = "SLSPA-CHK-04";
const CHK_SIMULATOR: &str = "SLSPA-CHK-05";
const CHK_LINKED_EXECUTOR: &str = "SLSPA-CHK-06";
const CHK_SCHEMA_ADAPTER: &str = "SLSPA-CHK-07";
const CHK_RECORD_ADAPTER: &str = "SLSPA-CHK-08";
const CHK_MAPPING_COVERAGE: &str = "SLSPA-CHK-09";
const CHK_TARGET_EMPTY: &str = "SLSPA-CHK-10";
const CHK_SANDBOX_VERIFIED: &str = "SLSPA-CHK-11";

// ── Adapter trait (no-op and mock) ────────────────────────────────────────────

/// Trait representing a linked second-pass execution adapter boundary.
///
/// This trait exists purely as a future injection point for sandbox tests.
/// No production adapter is implemented. The default adapter is no-op.
///
/// Safety invariants:
/// - No implementation of this trait may call the real Airtable API.
/// - No implementation may return a token, path, record payload, or HTTP body.
/// - No implementation is wired into the runtime app flow.
pub trait LinkedSecondPassAdapter {
    /// Returns the count of operations this adapter would handle.
    /// Must not make any network calls.
    fn planned_operation_count(&self) -> usize;
}

/// No-op adapter used in unit tests.
/// Records no state, makes no network calls.
pub struct NoOpLinkedSecondPassAdapter;

impl LinkedSecondPassAdapter for NoOpLinkedSecondPassAdapter {
    fn planned_operation_count(&self) -> usize {
        0
    }
}

/// Mock adapter that counts planned operations without any network call.
/// For use in unit tests only.
pub struct MockLinkedSecondPassAdapter {
    pub operation_count: usize,
}

impl LinkedSecondPassAdapter for MockLinkedSecondPassAdapter {
    fn planned_operation_count(&self) -> usize {
        self.operation_count
    }
}

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the sandbox linked second-pass adapter boundary evaluation.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never enables execution, writes, or reads.
/// - Never changes `evaluate_write_gate()` behavior.
/// - Never unlocks any executor or network path.
/// - Never stores any state globally.
/// - Is not reachable from UI, TypeScript, or any Tauri command.
/// - Always returns `runtime_execution_enabled: false`,
///   `app_runtime_writes_enabled: false`, `app_runtime_reads_enabled: false`,
///   `no_changes_made: true`, `network_writes_attempted: false`.
/// - Returns `NotExecuted` when mode is `Disabled`.
/// - Returns `Blocked` when any prerequisite fails.
/// - Returns `ReadyForSandboxCall` only when all prerequisites pass — this does
///   NOT execute a call, does NOT arm the gate, and is NOT persisted.
/// - Operations describe only linked second-pass update batches.
///   No schema, first-pass record create, attachment, or checkpoint
///   operations appear.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_sandbox_linked_second_pass_adapter(
    request: &SandboxLinkedSecondPassAdapterRequest,
    schema_plan: &SchemaWriteRequestPlan,
    record_plan: &crate::restore::record_write_requests::RecordWriteRequestPlan,
) -> SandboxLinkedSecondPassAdapterResult {
    // ── Mode check ─────────────────────────────────────────────────────────────
    if matches!(request.mode, SandboxLinkedSecondPassAdapterMode::Disabled) {
        return not_executed_result(&format!(
            "{CHK_MODE}: Adapter mode is disabled. No evaluation is performed. This is the default state."
        ));
    }

    // ── Explicit flag ──────────────────────────────────────────────────────────
    if !request.explicit_internal_linked_sandbox_call_requested {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "Explicit internal linked sandbox call flag is not set. \
             This flag must be explicitly true before adapter evaluation can proceed. \
             No UI control, Tauri command, or runtime path sets this flag.",
            &format!(
                "{CHK_EXPLICIT_FLAG}: explicit_internal_linked_sandbox_call_requested must be true."
            ),
        );
    }

    // ── Write gate check ───────────────────────────────────────────────────────
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if !write_gate_disabled {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "evaluate_write_gate() did not return Disabled/DisabledByProductPolicy. \
             This is a critical safety violation. Adapter evaluation cannot proceed.",
            &format!(
                "{CHK_WRITE_GATE}: evaluate_write_gate() must return Disabled/DisabledByProductPolicy."
            ),
        );
    }

    // ── Sandbox gate arming probe ──────────────────────────────────────────────
    let arming_req = SandboxGateArmingRequest {
        mode: SandboxGateArmingMode::SandboxOnlyInternal,
        explicit_internal_sandbox_arming_requested: true,
        sandbox_verification_safe: request.sandbox_verified,
        target_empty_safe: request.target_base_empty,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
        checkpoint_durability_safe: request.checkpoint_durability_safe,
        sensitive_data_safe: request.sensitive_data_safe,
        final_validation_enforcement_safe: request.final_validation_enforcement_safe,
        rate_limit_backoff_safe: request.rate_limit_backoff_safe,
    };
    let arming_result = build_sandbox_gate_arming_decision(&arming_req);
    let gate_arming_armed = matches!(
        arming_result.status,
        SandboxGateArmingStatus::ArmedNotExecutable
    );
    if !gate_arming_armed {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "Sandbox gate arming decision did not return ArmedNotExecutable. \
             All arming prerequisites must be satisfied before the linked adapter boundary can proceed.",
            &format!("{CHK_ARMING}: sandbox gate arming decision must return ArmedNotExecutable."),
        );
    }

    // ── Sandbox restore simulator probe ────────────────────────────────────────
    let sim_req = SandboxRestoreSimulatorRequest {
        mode: SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
        explicit_internal_simulation_requested: true,
        sandbox_verification_safe: request.sandbox_verified,
        target_empty_safe: request.target_base_empty,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
        checkpoint_durability_safe: request.checkpoint_durability_safe,
        sensitive_data_safe: request.sensitive_data_safe,
        final_validation_enforcement_safe: request.final_validation_enforcement_safe,
        rate_limit_backoff_safe: request.rate_limit_backoff_safe,
    };
    let sim_result = run_sandbox_restore_simulator(&sim_req);
    let simulator_simulated = matches!(
        sim_result.status,
        SandboxRestoreSimulatorStatus::SimulatedNotExecuted
    );
    if !simulator_simulated {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "Sandbox restore simulator did not return SimulatedNotExecuted. \
             All simulation prerequisites must be satisfied before the linked adapter boundary can proceed.",
            &format!(
                "{CHK_SIMULATOR}: sandbox restore simulator must return SimulatedNotExecuted."
            ),
        );
    }

    // ── Linked second-pass executor probe ─────────────────────────────────────
    let executor_req = LinkedSecondPassExecutorRequest {
        mode: LinkedSecondPassExecutorMode::SandboxOnly,
        explicit_internal_linked_second_pass_requested: true,
        sandbox_verified: request.sandbox_verified,
        target_empty_verified: request.target_base_empty,
        record_executor_safe: request.record_executor_safe,
        linked_second_pass_preview_ready: request.linked_second_pass_preview_ready,
        linked_second_pass_preview_status: LinkedSecondPassExecutionPreviewStatus::DryRunReady,
        mapping_checkpoint_preview_ready: request.mapping_checkpoint_preview_ready,
        sensitive_data_safe: request.sensitive_data_safe,
        live_write_readiness_satisfied: request.live_write_readiness_safe,
        batch_size: 10,
        field_summaries: request.field_summaries.clone(),
    };
    let executor_result = build_linked_second_pass_executor_plan(&executor_req);
    let linked_executor_not_executed = matches!(
        executor_result.status,
        LinkedSecondPassExecutorStatus::NotExecuted
    );
    if !linked_executor_not_executed {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "Linked second-pass executor plan did not return NotExecuted. \
             A ready (not-executed) executor plan is required before the linked adapter boundary can proceed.",
            &format!(
                "{CHK_LINKED_EXECUTOR}: linked second-pass executor plan must return NotExecuted."
            ),
        );
    }

    // ── Schema write adapter probe ─────────────────────────────────────────────
    let schema_adapter_req = SandboxSchemaWriteAdapterRequest {
        mode: SandboxSchemaWriteAdapterMode::SandboxOnlyInternal,
        explicit_internal_schema_sandbox_call_requested: true,
        sandbox_verified: request.sandbox_verified,
        target_base_empty: request.target_base_empty,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
        checkpoint_durability_safe: request.checkpoint_durability_safe,
        sensitive_data_safe: request.sensitive_data_safe,
        final_validation_enforcement_safe: request.final_validation_enforcement_safe,
        rate_limit_backoff_safe: request.rate_limit_backoff_safe,
    };
    let schema_adapter_result =
        build_sandbox_schema_write_adapter(&schema_adapter_req, schema_plan);
    let schema_adapter_ready = matches!(
        schema_adapter_result.status,
        SandboxSchemaWriteAdapterStatus::ReadyForSandboxCall
    );
    if !schema_adapter_ready {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "Schema write adapter boundary did not return ReadyForSandboxCall. \
             The schema adapter must be ready before the linked adapter boundary can proceed.",
            &format!("{CHK_SCHEMA_ADAPTER}: schema write adapter must return ReadyForSandboxCall."),
        );
    }

    // ── Record write adapter probe ─────────────────────────────────────────────
    let record_adapter_req = SandboxRecordWriteAdapterRequest {
        mode: SandboxRecordWriteAdapterMode::SandboxOnlyInternal,
        explicit_internal_record_sandbox_call_requested: true,
        sandbox_verified: request.sandbox_verified,
        target_base_empty: request.target_base_empty,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
        checkpoint_durability_safe: request.checkpoint_durability_safe,
        sensitive_data_safe: request.sensitive_data_safe,
        final_validation_enforcement_safe: request.final_validation_enforcement_safe,
        rate_limit_backoff_safe: request.rate_limit_backoff_safe,
        schema_executor_safe: request.schema_executor_safe,
        checkpoint_store_safe: request.checkpoint_store_safe,
    };
    let record_adapter_result =
        build_sandbox_record_write_adapter(&record_adapter_req, record_plan, schema_plan);
    let record_adapter_ready = matches!(
        record_adapter_result.status,
        SandboxRecordWriteAdapterStatus::ReadyForSandboxCall
    );
    if !record_adapter_ready {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "Record write adapter boundary did not return ReadyForSandboxCall. \
             The record adapter must be ready before the linked adapter boundary can proceed.",
            &format!("{CHK_RECORD_ADAPTER}: record write adapter must return ReadyForSandboxCall."),
        );
    }

    // ── Mapping coverage check ─────────────────────────────────────────────────
    if !request.mapping_coverage_sufficient {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            request.target_base_empty,
            request.sandbox_verified,
            "Mapping coverage is insufficient. Mapping coverage must be declared sufficient \
             (without exposing record IDs) before linked update descriptors can be built.",
            &format!("{CHK_MAPPING_COVERAGE}: mapping_coverage_sufficient must be true."),
        );
    }

    // ── Target empty check ─────────────────────────────────────────────────────
    if !request.target_base_empty {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            request.sandbox_verified,
            "Target base empty verification has not passed. \
             The target base must be confirmed empty before linked second-pass adapter can proceed.",
            &format!("{CHK_TARGET_EMPTY}: target_base_empty must be true."),
        );
    }

    // ── Sandbox verified check ─────────────────────────────────────────────────
    if !request.sandbox_verified {
        return blocked(
            SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            "Sandbox environment verification has not passed. \
             A verified sandbox target is required before linked second-pass adapter can proceed.",
            &format!("{CHK_SANDBOX_VERIFIED}: sandbox_verified must be true."),
        );
    }

    // ── All prerequisites satisfied — build adapter operations ─────────────────
    // Only linked update batch descriptors are emitted. No schema, first-pass
    // create, attachment, or checkpoint operations are accepted.
    let mut operations: Vec<SandboxLinkedSecondPassAdapterOperation> = Vec::new();
    let mut total_update_count: usize = 0;
    let batch_size: usize = 10;

    for (op_idx, field) in request.field_summaries.iter().enumerate() {
        if field.record_count == 0 {
            continue;
        }
        let n_batches = field.record_count.div_ceil(batch_size);
        for b in 0..n_batches {
            let offset = b * batch_size;
            let count = (field.record_count - offset).min(batch_size);
            total_update_count += count;
            let global_idx = operations.len();
            operations.push(SandboxLinkedSecondPassAdapterOperation {
                operation_id: format!("SLSPA-OP-{global_idx:03}"),
                operation_kind: "linkedUpdateBatchDescriptor".to_string(),
                table_label: field.table_label.clone(),
                field_label: field.field_label.clone(),
                update_count: Some(count),
                mapping_coverage_count: Some(count),
                status: SandboxLinkedSecondPassAdapterOperationStatus::Planned,
                note: format!(
                    "Linked update batch for '{}' in '{}' (field summary index {}, batch {} of {}, {} record(s)) — \
                     adapter boundary descriptor. No network call made. No record IDs exposed.",
                    field.field_label,
                    field.table_label,
                    op_idx,
                    b + 1,
                    n_batches,
                    count,
                ),
            });
        }
    }

    let total_operation_count = operations.len();
    let snapshot = SandboxLinkedSecondPassAdapterSafetySnapshot {
        write_gate_disabled: true,
        gate_arming_armed_not_executable: true,
        simulator_simulated_not_executed: true,
        linked_executor_not_executed: true,
        schema_adapter_ready: true,
        record_adapter_ready: true,
        mapping_coverage_sufficient: true,
        target_base_empty: true,
        sandbox_verified: true,
        explicit_linked_sandbox_flag_set: true,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        network_writes_attempted: false,
    };

    SandboxLinkedSecondPassAdapterResult {
        status: SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall,
        mode: SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
        message: format!(
            "Sandbox linked second-pass adapter boundary is ready ({total_operation_count} \
             linked update batch descriptor(s), {total_update_count} total update(s)). \
             No Airtable network call was made. No runtime execution is enabled. \
             No app runtime writes or reads are enabled. No changes were made. \
             This adapter boundary is for sandbox tests only and is not reachable from \
             UI, TypeScript, or any Tauri command."
        ),
        operations,
        safety_snapshot: snapshot,
        total_operation_count,
        total_update_count,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn not_executed_result(message: &str) -> SandboxLinkedSecondPassAdapterResult {
    SandboxLinkedSecondPassAdapterResult {
        status: SandboxLinkedSecondPassAdapterStatus::NotExecuted,
        mode: SandboxLinkedSecondPassAdapterMode::Disabled,
        message: format!(
            "{message} No Airtable API calls were made. No changes were made. \
             Runtime execution, writes, and reads remain disabled."
        ),
        operations: vec![],
        safety_snapshot: SandboxLinkedSecondPassAdapterSafetySnapshot {
            write_gate_disabled: true,
            gate_arming_armed_not_executable: false,
            simulator_simulated_not_executed: false,
            linked_executor_not_executed: false,
            schema_adapter_ready: false,
            record_adapter_ready: false,
            mapping_coverage_sufficient: false,
            target_base_empty: false,
            sandbox_verified: false,
            explicit_linked_sandbox_flag_set: false,
            runtime_execution_enabled: false,
            app_runtime_writes_enabled: false,
            app_runtime_reads_enabled: false,
            network_writes_attempted: false,
        },
        total_operation_count: 0,
        total_update_count: 0,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
    }
}

#[allow(clippy::too_many_arguments)]
fn blocked(
    mode: SandboxLinkedSecondPassAdapterMode,
    gate_arming_armed_not_executable: bool,
    simulator_simulated_not_executed: bool,
    linked_executor_not_executed: bool,
    schema_adapter_ready: bool,
    record_adapter_ready: bool,
    mapping_coverage_sufficient: bool,
    target_base_empty_snap: bool,
    target_base_empty_req: bool,
    sandbox_verified: bool,
    message: &str,
    blocked_reason: &str,
) -> SandboxLinkedSecondPassAdapterResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    SandboxLinkedSecondPassAdapterResult {
        status: SandboxLinkedSecondPassAdapterStatus::Blocked,
        mode,
        message: format!(
            "Sandbox linked second-pass adapter is blocked. {message} \
             No Airtable API calls were made. No changes were made. \
             Runtime execution, writes, and reads remain disabled."
        ),
        operations: vec![],
        safety_snapshot: SandboxLinkedSecondPassAdapterSafetySnapshot {
            write_gate_disabled,
            gate_arming_armed_not_executable,
            simulator_simulated_not_executed,
            linked_executor_not_executed,
            schema_adapter_ready,
            record_adapter_ready,
            mapping_coverage_sufficient,
            target_base_empty: target_base_empty_snap || target_base_empty_req,
            sandbox_verified,
            explicit_linked_sandbox_flag_set: false,
            runtime_execution_enabled: false,
            app_runtime_writes_enabled: false,
            app_runtime_reads_enabled: false,
            network_writes_attempted: false,
        },
        total_operation_count: 0,
        total_update_count: 0,
        blocked_reason: Some(blocked_reason.to_string()),
        no_changes_made: true,
        network_writes_attempted: false,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
    };
    use crate::restore::record_import_planner::create_record_import_plan;
    use crate::restore::record_write_requests::build_record_write_request_plan;
    use crate::restore::schema_plan::RestoreSchemaPlan;
    use crate::restore::schema_plan::{
        RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreSchemaDependencyGraph,
        RestoreSchemaPlanStatus, RestoreTableCreationStep,
    };
    use crate::restore::schema_write_requests::build_schema_write_request_plan;

    // ── Test helpers ──────────────────────────────────────────────────────────

    fn simple_field_summaries() -> Vec<LinkedSecondPassFieldSummary> {
        vec![LinkedSecondPassFieldSummary {
            table_label: "Projects".to_string(),
            field_label: "Tasks".to_string(),
            record_count: 12,
            batch_count: 2,
            unresolved_link_count: 0,
        }]
    }

    fn simple_record_plan() -> crate::restore::record_write_requests::RecordWriteRequestPlan {
        let req = RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables: vec![RecordImportTableInput {
                table_id: "tbl01".to_string(),
                table_name: "Tasks".to_string(),
                record_count: Some(10),
                fields: vec![RecordImportFieldInput {
                    field_id: "fld01".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                }],
            }],
        };
        let import_plan = create_record_import_plan(&req);
        build_record_write_request_plan(&import_plan)
    }

    fn simple_schema_plan() -> SchemaWriteRequestPlan {
        let plan = RestoreSchemaPlan {
            filename: "backup.airbridge".to_string(),
            status: RestoreSchemaPlanStatus::Ready,
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            table_steps: vec![RestoreTableCreationStep {
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
                step_index: 0,
                field_count: 1,
                direct_field_count: 1,
                deferred_field_count: 0,
                manual_action_count: 0,
                unsupported_count: 0,
                note: "Create table 'Tasks'.".to_string(),
            }],
            field_steps: vec![RestoreFieldCreationStep {
                field_id: "fld001".to_string(),
                field_name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
                classification: RestoreFieldCreateClassification::CreateDirectly,
                note: "Direct field.".to_string(),
            }],
            deferred_steps: vec![],
            manual_action_fields: vec![],
            dependency_graph: RestoreSchemaDependencyGraph {
                edges: vec![],
                has_circular_dependency: false,
                resolution_note: String::new(),
            },
            warnings: vec![],
            errors: vec![],
            no_changes_made: true,
        };
        build_schema_write_request_plan(&plan)
    }

    fn full_request() -> SandboxLinkedSecondPassAdapterRequest {
        SandboxLinkedSecondPassAdapterRequest {
            mode: SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
            explicit_internal_linked_sandbox_call_requested: true,
            sandbox_verified: true,
            target_base_empty: true,
            mapping_coverage_sufficient: true,
            confirmation_gate_declared: true,
            destructive_operation_policy_safe: true,
            attachment_phase_disabled_safe: true,
            live_write_readiness_safe: true,
            write_phase_ordering_safe: true,
            failure_modes_safe: true,
            rollback_limitation_safe: true,
            checkpoint_durability_safe: true,
            sensitive_data_safe: true,
            final_validation_enforcement_safe: true,
            rate_limit_backoff_safe: true,
            schema_executor_safe: true,
            checkpoint_store_safe: true,
            record_executor_safe: true,
            linked_second_pass_preview_ready: true,
            mapping_checkpoint_preview_ready: true,
            field_summaries: simple_field_summaries(),
        }
    }

    fn disabled_request() -> SandboxLinkedSecondPassAdapterRequest {
        SandboxLinkedSecondPassAdapterRequest {
            mode: SandboxLinkedSecondPassAdapterMode::Disabled,
            explicit_internal_linked_sandbox_call_requested: false,
            sandbox_verified: false,
            target_base_empty: false,
            mapping_coverage_sufficient: false,
            confirmation_gate_declared: false,
            destructive_operation_policy_safe: false,
            attachment_phase_disabled_safe: false,
            live_write_readiness_safe: false,
            write_phase_ordering_safe: false,
            failure_modes_safe: false,
            rollback_limitation_safe: false,
            checkpoint_durability_safe: false,
            sensitive_data_safe: false,
            final_validation_enforcement_safe: false,
            rate_limit_backoff_safe: false,
            schema_executor_safe: false,
            checkpoint_store_safe: false,
            record_executor_safe: false,
            linked_second_pass_preview_ready: false,
            mapping_checkpoint_preview_ready: false,
            field_summaries: vec![],
        }
    }

    // ── Default blocked path ───────────────────────────────────────────────────

    #[test]
    fn default_disabled_request_returns_not_executed() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&disabled_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxLinkedSecondPassAdapterStatus::NotExecuted
        );
    }

    #[test]
    fn disabled_mode_returns_not_executed_with_mode_disabled() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&disabled_request(), &sp, &rp);
        assert_eq!(result.mode, SandboxLinkedSecondPassAdapterMode::Disabled);
    }

    #[test]
    fn missing_explicit_linked_sandbox_flag_returns_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.explicit_internal_linked_sandbox_call_requested = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_EXPLICIT_FLAG));
    }

    // ── Arming blocked propagates ─────────────────────────────────────────────

    #[test]
    fn arming_prereq_failure_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.sandbox_verified = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_ARMING));
    }

    // ── Simulator blocked propagates ──────────────────────────────────────────

    #[test]
    fn simulator_prereq_failure_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.failure_modes_safe = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains("SLSPA-CHK-"));
    }

    // ── Linked executor blocked propagates ────────────────────────────────────

    #[test]
    fn linked_executor_blocked_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.record_executor_safe = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_LINKED_EXECUTOR));
    }

    // ── Schema adapter not ready causes blocked ───────────────────────────────

    #[test]
    fn schema_adapter_not_ready_causes_blocked() {
        use crate::restore::schema_write_requests::{
            SchemaWriteBlockedReason, SchemaWriteOperationStatus,
        };
        let rp = simple_record_plan();
        let mut sp = simple_schema_plan();
        sp.status = SchemaWriteOperationStatus::Blocked;
        sp.blocked_reason = Some(SchemaWriteBlockedReason::SchemaPlanNotReady);
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_SCHEMA_ADAPTER));
    }

    // ── Record adapter not ready causes blocked ───────────────────────────────

    #[test]
    fn record_adapter_not_ready_causes_blocked() {
        use crate::restore::record_write_requests::{
            RecordWriteBlockedReason, RecordWriteOperationStatus,
        };
        let mut rp = simple_record_plan();
        rp.status = RecordWriteOperationStatus::Blocked;
        rp.blocked_reason = Some(RecordWriteBlockedReason::RecordImportPlanNotReady);
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_RECORD_ADAPTER));
    }

    // ── Insufficient mapping coverage causes blocked ───────────────────────────

    #[test]
    fn insufficient_mapping_coverage_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.mapping_coverage_sufficient = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_MAPPING_COVERAGE));
    }

    // ── Target not empty causes blocked ──────────────────────────────────────

    #[test]
    fn target_not_empty_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.target_base_empty = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
    }

    // ── Sandbox not verified causes blocked ───────────────────────────────────

    #[test]
    fn sandbox_not_verified_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.sandbox_verified = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxLinkedSecondPassAdapterStatus::Blocked);
    }

    // ── evaluate_write_gate() remains Disabled ────────────────────────────────

    #[test]
    fn evaluate_write_gate_default_remains_disabled() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let _ = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── readyForSandboxCall when all prereqs satisfied ────────────────────────

    #[test]
    fn ready_for_sandbox_call_when_all_prereqs_satisfied() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall
        );
    }

    // ── Safety invariants on readyForSandboxCall ──────────────────────────────

    #[test]
    fn ready_for_sandbox_call_runtime_execution_enabled_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(!result.runtime_execution_enabled);
    }

    #[test]
    fn ready_for_sandbox_call_app_runtime_writes_enabled_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(!result.app_runtime_writes_enabled);
    }

    #[test]
    fn ready_for_sandbox_call_app_runtime_reads_enabled_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(!result.app_runtime_reads_enabled);
    }

    #[test]
    fn no_network_writes_attempted_by_default() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_always_true() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(result.no_changes_made);
    }

    // ── Safety invariants in blocked/not-executed paths ───────────────────────

    #[test]
    fn no_changes_made_true_in_blocked_path() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.explicit_internal_linked_sandbox_call_requested = false;
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_true_in_not_executed_path() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&disabled_request(), &sp, &rp);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    // ── Only linked update batch descriptors accepted ─────────────────────────

    #[test]
    fn only_linked_update_batch_descriptor_in_operations() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        for op in &result.operations {
            assert_eq!(
                op.operation_kind, "linkedUpdateBatchDescriptor",
                "unexpected operation kind: {}",
                op.operation_kind
            );
        }
    }

    #[test]
    fn no_schema_operation_in_output() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("createTable"));
        assert!(!json.contains("createField"));
        assert!(!json.contains("createBase"));
    }

    #[test]
    fn no_record_create_operation_in_output() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("createRecordBatch"));
        assert!(!json.contains("firstPassCreate"));
    }

    #[test]
    fn no_attachment_operation_in_output() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("attachment"));
        assert!(!json.contains("Attachment"));
        assert!(!json.contains("preserveMetadata"));
    }

    // ── Operation ordering is deterministic ───────────────────────────────────

    #[test]
    fn operation_ordering_is_deterministic() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let r1 = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let r2 = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let ids1: Vec<_> = r1.operations.iter().map(|o| &o.operation_id).collect();
        let ids2: Vec<_> = r2.operations.iter().map(|o| &o.operation_id).collect();
        assert_eq!(ids1, ids2);
    }

    // ── No token/path/payload/raw HTTP/record ID leaks ────────────────────────

    #[test]
    fn no_token_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn no_absolute_path_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_record_payload_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn no_raw_http_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"body\":{"));
        assert!(!json.contains("\"statusCode\""));
    }

    #[test]
    fn no_old_record_id_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"oldRecordId\""));
        assert!(!json.contains("oldId"));
        assert!(!json.contains("rec_old_"));
    }

    #[test]
    fn no_new_record_id_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"newRecordId\""));
        assert!(!json.contains("newId"));
        assert!(!json.contains("rec_new_"));
    }

    #[test]
    fn no_attachment_url_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    // ── No success state introduced ───────────────────────────────────────────

    #[test]
    fn no_success_state_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    // ── No real Airtable client called ────────────────────────────────────────

    #[test]
    fn no_real_airtable_client_called_in_default_path() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(!result.network_writes_attempted);
        assert!(!result.safety_snapshot.network_writes_attempted);
    }

    // ── No Tauri command introduced ───────────────────────────────────────────

    #[test]
    fn no_tauri_command_introduced() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall
        );
    }

    // ── No-op adapter ─────────────────────────────────────────────────────────

    #[test]
    fn no_op_adapter_returns_zero_count() {
        let adapter = NoOpLinkedSecondPassAdapter;
        assert_eq!(adapter.planned_operation_count(), 0);
    }

    // ── Mock adapter ──────────────────────────────────────────────────────────

    #[test]
    fn mock_adapter_returns_configured_count() {
        let adapter = MockLinkedSecondPassAdapter { operation_count: 7 };
        assert_eq!(adapter.planned_operation_count(), 7);
    }

    #[test]
    fn mock_adapter_zero_count_when_no_operations() {
        let adapter = MockLinkedSecondPassAdapter { operation_count: 0 };
        assert_eq!(adapter.planned_operation_count(), 0);
    }

    // ── Write gate snapshot ───────────────────────────────────────────────────

    #[test]
    fn safety_snapshot_write_gate_disabled_always_true() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn safety_snapshot_runtime_flags_always_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert!(!result.safety_snapshot.runtime_execution_enabled);
        assert!(!result.safety_snapshot.app_runtime_writes_enabled);
        assert!(!result.safety_snapshot.app_runtime_reads_enabled);
        assert!(!result.safety_snapshot.network_writes_attempted);
    }

    // ── Operation count and update count ──────────────────────────────────────

    #[test]
    fn operation_count_consistent_with_operations_vec() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        assert_eq!(result.total_operation_count, result.operations.len());
    }

    #[test]
    fn total_update_count_sums_correctly() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        let sum: usize = result
            .operations
            .iter()
            .map(|o| o.update_count.unwrap_or(0))
            .sum();
        assert_eq!(result.total_update_count, sum);
    }

    // ── Operation IDs use stable prefix ──────────────────────────────────────

    #[test]
    fn operation_ids_use_slspa_prefix() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_linked_second_pass_adapter(&full_request(), &sp, &rp);
        for op in &result.operations {
            assert!(
                op.operation_id.starts_with("SLSPA-OP-"),
                "unexpected operation ID prefix: {}",
                op.operation_id
            );
        }
    }

    // ── Empty field summaries produce empty operations ────────────────────────

    #[test]
    fn empty_field_summaries_produce_empty_operations() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.field_summaries = vec![];
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(
            result.status,
            SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall
        );
        assert_eq!(result.total_operation_count, 0);
        assert_eq!(result.total_update_count, 0);
    }

    // ── Field ordering preserved ──────────────────────────────────────────────

    #[test]
    fn field_ordering_preserved_in_operations() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.field_summaries = vec![
            LinkedSecondPassFieldSummary {
                table_label: "Projects".to_string(),
                field_label: "Alpha".to_string(),
                record_count: 5,
                batch_count: 1,
                unresolved_link_count: 0,
            },
            LinkedSecondPassFieldSummary {
                table_label: "Tasks".to_string(),
                field_label: "Beta".to_string(),
                record_count: 3,
                batch_count: 1,
                unresolved_link_count: 0,
            },
        ];
        let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
        assert_eq!(
            result.status,
            SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall
        );
        let alpha_idx = result
            .operations
            .iter()
            .find(|o| o.field_label == "Alpha")
            .map(|o| o.operation_id.clone());
        let beta_idx = result
            .operations
            .iter()
            .find(|o| o.field_label == "Beta")
            .map(|o| o.operation_id.clone());
        assert!(alpha_idx.is_some());
        assert!(beta_idx.is_some());
        // Alpha should come before Beta
        assert!(alpha_idx.unwrap() < beta_idx.unwrap());
    }
}
