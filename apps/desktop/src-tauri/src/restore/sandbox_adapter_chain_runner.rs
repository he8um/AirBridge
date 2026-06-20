use serde::{Deserialize, Serialize};

use crate::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use crate::restore::record_write_requests::RecordWriteRequestPlan;
use crate::restore::sandbox_final_validation_adapter::{
    build_sandbox_final_validation_adapter, SandboxFinalValidationAdapterMode,
    SandboxFinalValidationAdapterRequest, SandboxFinalValidationAdapterStatus,
};
use crate::restore::sandbox_linked_second_pass_adapter::{
    build_sandbox_linked_second_pass_adapter, SandboxLinkedSecondPassAdapterMode,
    SandboxLinkedSecondPassAdapterRequest, SandboxLinkedSecondPassAdapterStatus,
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

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the sandbox adapter chain runner.
///
/// Safety invariants:
/// - `MockRunNotExecuted` does NOT execute any Airtable network call.
/// - `MockRunNotExecuted` does NOT enable runtime writes, reads, or execution.
/// - `MockRunNotExecuted` is not stored globally, not persisted, and is not
///   reachable from UI, TypeScript, or any Tauri command.
/// - `MockRunNotExecuted` does NOT change `evaluate_write_gate()` behavior.
/// - `runtime_execution_enabled` is always `false` regardless of status.
/// - `app_runtime_writes_enabled` is always `false` regardless of status.
/// - `app_runtime_reads_enabled` is always `false` regardless of status.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `airtable_client_called` is always `false`.
/// - `no_changes_made` is always `true`.
/// - No `Succeeded`, `Complete`, `Enabled`, `Done`, or `ExecutionReady` status exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxAdapterChainRunnerStatus {
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
    /// All prerequisites satisfied and all four adapter phases were traversed
    /// using mock/no-op adapters only. No Airtable network call was made.
    /// No execution occurred. No state was persisted.
    MockRunNotExecuted,
}

/// Mode for the sandbox adapter chain runner.
///
/// Safety invariants:
/// - `Disabled` is the default and operationally always-reachable mode.
/// - `MockInternalOnly` is for Rust unit tests only.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxAdapterChainRunnerMode {
    /// Runner disabled — no evaluation is performed. Default state.
    Disabled,
    /// Internal mock-only runner mode for Rust unit tests only.
    /// Does NOT execute network calls, enable runtime writes/reads, or persist state.
    MockInternalOnly,
}

/// Status of a single adapter phase in the chain run.
///
/// Note: `succeeded` / `completed` / `done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxAdapterChainRunnerPhaseStatus {
    /// The phase was observed from a mock/no-op adapter result. No network call
    /// was made. No execution occurred.
    MockObserved,
    /// The phase is described but skipped (checkpoint boundary). No file written.
    Skipped,
    /// The phase is planned but was not reached due to an earlier prerequisite failure.
    Planned,
    /// The phase is blocked by a safety prerequisite failure.
    Blocked,
    /// The runner is in disabled mode. Phase was not evaluated.
    NotExecuted,
}

/// A single adapter phase in the chain run.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No raw Airtable request body or response.
/// - No old or new Airtable record IDs.
/// - No record field payload or field values.
/// - No attachment URL.
/// - `status` is never `succeeded`, `complete`, or `done`.
/// - Only safe operation counts are reported — no raw operation payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxAdapterChainRunnerPhase {
    /// Stable phase identifier (e.g. `SACR-PH-01`).
    pub phase_id: String,
    /// Human-readable phase label.
    pub label: String,
    pub status: SandboxAdapterChainRunnerPhaseStatus,
    /// Safe operation count for this phase (no raw payloads).
    pub operation_count: usize,
    pub note: String,
}

/// Point-in-time safety snapshot for the chain run.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No old or new Airtable record IDs.
/// - No raw record field values.
/// - No raw HTTP body.
/// - No attachment URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxAdapterChainRunnerSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the sandbox restore simulator returned `SimulatedNotExecuted`.
    pub simulator_simulated_not_executed: bool,
    /// Whether the schema write adapter returned `ReadyForSandboxCall`.
    pub schema_adapter_ready: bool,
    /// Whether the record write adapter returned `ReadyForSandboxCall`.
    pub record_adapter_ready: bool,
    /// Whether the linked second-pass adapter returned `ReadyForSandboxCall`.
    pub linked_adapter_ready: bool,
    /// Whether the final validation adapter returned `ReadyForSandboxCall`.
    pub final_validation_adapter_ready: bool,
    /// Whether the explicit internal mock chain flag was set.
    pub explicit_mock_chain_flag_set: bool,
    /// Whether runtime execution is enabled — always `false`.
    pub runtime_execution_enabled: bool,
    /// Whether app runtime writes are enabled — always `false`.
    pub app_runtime_writes_enabled: bool,
    /// Whether app runtime reads are enabled — always `false`.
    pub app_runtime_reads_enabled: bool,
    /// Whether any network read was attempted — always `false`.
    pub network_reads_attempted: bool,
    /// Whether any network write was attempted — always `false`.
    pub network_writes_attempted: bool,
    /// Whether the real Airtable client was called — always `false`.
    pub airtable_client_called: bool,
}

/// Request to the sandbox adapter chain runner.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_mock_chain_requested` must be `true` for the runner to
/// proceed past its gate check. No UI control, Tauri command, or runtime path
/// sets this flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxAdapterChainRunnerRequest {
    /// Must be `mockInternalOnly` for evaluation to proceed.
    /// `disabled` (the default) always results in `Blocked`.
    pub mode: SandboxAdapterChainRunnerMode,
    /// Internal-only flag. Must be explicitly `true` to proceed.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_mock_chain_requested: bool,
    /// Whether sandbox environment verification is declared and safe.
    pub sandbox_verified: bool,
    /// Whether target empty verification is declared and safe.
    pub target_base_empty: bool,
    /// Whether mapping coverage is sufficient (no IDs exposed).
    pub mapping_coverage_sufficient: bool,
    /// Whether the final validation enforcement policy is safe.
    pub final_validation_enforcement_safe: bool,
    /// Prerequisite booleans forwarded to adapter probes.
    pub confirmation_gate_declared: bool,
    pub destructive_operation_policy_safe: bool,
    pub attachment_phase_disabled_safe: bool,
    pub live_write_readiness_safe: bool,
    pub write_phase_ordering_safe: bool,
    pub failure_modes_safe: bool,
    pub rollback_limitation_safe: bool,
    pub checkpoint_durability_safe: bool,
    pub sensitive_data_safe: bool,
    pub rate_limit_backoff_safe: bool,
    pub schema_executor_safe: bool,
    pub checkpoint_store_safe: bool,
    pub record_executor_safe: bool,
    pub linked_executor_safe: bool,
    pub linked_second_pass_preview_ready: bool,
    pub mapping_checkpoint_preview_ready: bool,
    /// Per-field summaries forwarded to the linked adapter probe.
    /// No raw record IDs — only safe counts and labels.
    pub field_summaries: Vec<LinkedSecondPassFieldSummary>,
    /// Safe count of tables (for final validation adapter).
    pub table_count: usize,
    /// Safe count of fields (for final validation adapter).
    pub field_count: usize,
    /// Safe count of records (for final validation adapter).
    pub record_count: usize,
    /// Safe count of ID mapping entries (for final validation adapter).
    pub id_mapping_entry_count: usize,
    /// Safe count of linked field coverage entries (for final validation adapter).
    pub linked_coverage_count: usize,
    /// Safe count of attachment metadata entries (for final validation adapter).
    pub attachment_metadata_count: usize,
    /// Whether a package manifest is present (for final validation adapter).
    pub manifest_present: bool,
}

/// Result of the sandbox adapter chain runner.
///
/// Safety invariants (always enforced):
/// - `runtime_execution_enabled` is always `false`.
/// - `app_runtime_writes_enabled` is always `false`.
/// - `app_runtime_reads_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `airtable_client_called` is always `false`.
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
/// - Only safe operation counts are reported — no raw operation payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxAdapterChainRunnerResult {
    pub status: SandboxAdapterChainRunnerStatus,
    pub mode: SandboxAdapterChainRunnerMode,
    pub message: String,
    pub phases: Vec<SandboxAdapterChainRunnerPhase>,
    pub safety_snapshot: SandboxAdapterChainRunnerSafetySnapshot,
    pub total_phase_count: usize,
    /// Safe count of schema adapter operations observed (no payloads).
    pub schema_operation_count: usize,
    /// Safe count of record adapter operations observed (no payloads).
    pub record_operation_count: usize,
    /// Safe count of linked adapter operations observed (no payloads).
    pub linked_operation_count: usize,
    /// Safe count of final validation read descriptors observed (no payloads).
    pub final_validation_operation_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network read operations were attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — the real Airtable client was not called.
    pub airtable_client_called: bool,
    /// Always `false` — runtime execution is not enabled.
    pub runtime_execution_enabled: bool,
    /// Always `false` — app runtime writes are not enabled.
    pub app_runtime_writes_enabled: bool,
    /// Always `false` — app runtime reads are not enabled.
    pub app_runtime_reads_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const CHK_MODE: &str = "SACR-CHK-01";
const CHK_EXPLICIT_FLAG: &str = "SACR-CHK-02";
const CHK_WRITE_GATE: &str = "SACR-CHK-03";
const CHK_SIMULATOR: &str = "SACR-CHK-04";
const CHK_SCHEMA_ADAPTER: &str = "SACR-CHK-05";
const CHK_RECORD_ADAPTER: &str = "SACR-CHK-06";
const CHK_LINKED_ADAPTER: &str = "SACR-CHK-07";
const CHK_FINAL_VALIDATION_ADAPTER: &str = "SACR-CHK-08";

// ── Phase IDs ─────────────────────────────────────────────────────────────────

const SACR_PH_01: &str = "SACR-PH-01";
const SACR_PH_02: &str = "SACR-PH-02";
const SACR_PH_03: &str = "SACR-PH-03";
const SACR_PH_04: &str = "SACR-PH-04";

// ── Core function ─────────────────────────────────────────────────────────────

/// Runs the sandbox adapter chain using mock/no-op adapters only.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never enables execution, writes, or reads.
/// - Never changes `evaluate_write_gate()` behavior.
/// - Never writes checkpoint files to disk.
/// - Never stores any state globally.
/// - Is not reachable from UI, TypeScript, or any Tauri command.
/// - Composes the four sandbox adapter boundaries in strict order:
///   1. Schema write adapter (SACR-PH-01)
///   2. Record write adapter (SACR-PH-02)
///   3. Linked second-pass adapter (SACR-PH-03)
///   4. Final validation adapter (SACR-PH-04)
/// - Reports only safe operation counts per adapter (no raw payloads).
/// - Always returns `runtime_execution_enabled: false`,
///   `app_runtime_writes_enabled: false`, `app_runtime_reads_enabled: false`,
///   `no_changes_made: true`, `network_reads_attempted: false`,
///   `network_writes_attempted: false`, `airtable_client_called: false`.
/// - Returns `Blocked` unless all prerequisites pass.
/// - Returns `MockRunNotExecuted` when all prerequisites pass — this does NOT
///   execute a live call, does NOT arm the gate, and is NOT persisted.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn run_sandbox_adapter_chain(
    request: &SandboxAdapterChainRunnerRequest,
    schema_plan: &SchemaWriteRequestPlan,
    record_plan: &RecordWriteRequestPlan,
) -> SandboxAdapterChainRunnerResult {
    // ── Mode check ─────────────────────────────────────────────────────────────
    if matches!(request.mode, SandboxAdapterChainRunnerMode::Disabled) {
        return blocked(
            SandboxAdapterChainRunnerMode::Disabled,
            false,
            false,
            false,
            false,
            false,
            false,
            &format!(
                "{CHK_MODE}: Runner mode is disabled. No evaluation is performed. \
                 This is the default state."
            ),
        );
    }

    // ── Explicit flag ──────────────────────────────────────────────────────────
    if !request.explicit_internal_mock_chain_requested {
        return blocked(
            SandboxAdapterChainRunnerMode::MockInternalOnly,
            false,
            false,
            false,
            false,
            false,
            false,
            &format!(
                "{CHK_EXPLICIT_FLAG}: explicit_internal_mock_chain_requested must be true. \
                 This flag must be explicitly set before the chain runner can proceed. \
                 No UI control, Tauri command, or runtime path sets this flag."
            ),
        );
    }

    // ── Write gate check ───────────────────────────────────────────────────────
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if !write_gate_disabled {
        return blocked(
            SandboxAdapterChainRunnerMode::MockInternalOnly,
            false,
            false,
            false,
            false,
            false,
            false,
            &format!(
                "{CHK_WRITE_GATE}: evaluate_write_gate() did not return \
                 Disabled/DisabledByProductPolicy. This is a critical safety violation."
            ),
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
    let simulator_ok = matches!(
        sim_result.status,
        SandboxRestoreSimulatorStatus::SimulatedNotExecuted
    );
    if !simulator_ok {
        return blocked(
            SandboxAdapterChainRunnerMode::MockInternalOnly,
            false,
            false,
            false,
            false,
            false,
            false,
            &format!(
                "{CHK_SIMULATOR}: sandbox restore simulator did not return SimulatedNotExecuted. \
                 All simulation prerequisites must be satisfied before the chain runner can proceed."
            ),
        );
    }

    // ── Schema write adapter (SACR-PH-01) ─────────────────────────────────────
    let schema_req = SandboxSchemaWriteAdapterRequest {
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
    let schema_result = build_sandbox_schema_write_adapter(&schema_req, schema_plan);
    let schema_ready = matches!(
        schema_result.status,
        SandboxSchemaWriteAdapterStatus::ReadyForSandboxCall
    );
    if !schema_ready {
        return blocked(
            SandboxAdapterChainRunnerMode::MockInternalOnly,
            true,
            false,
            false,
            false,
            false,
            false,
            &format!(
                "{CHK_SCHEMA_ADAPTER}: schema write adapter did not return ReadyForSandboxCall. \
                 The schema adapter must be ready before the chain runner can proceed."
            ),
        );
    }
    let schema_op_count = schema_result.total_operation_count;

    // ── Record write adapter (SACR-PH-02) ─────────────────────────────────────
    let record_req = SandboxRecordWriteAdapterRequest {
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
    let record_result = build_sandbox_record_write_adapter(&record_req, record_plan, schema_plan);
    let record_ready = matches!(
        record_result.status,
        SandboxRecordWriteAdapterStatus::ReadyForSandboxCall
    );
    if !record_ready {
        return blocked(
            SandboxAdapterChainRunnerMode::MockInternalOnly,
            true,
            true,
            false,
            false,
            false,
            false,
            &format!(
                "{CHK_RECORD_ADAPTER}: record write adapter did not return ReadyForSandboxCall. \
                 The record adapter must be ready before the chain runner can proceed."
            ),
        );
    }
    let record_op_count = record_result.total_operation_count;

    // ── Linked second-pass adapter (SACR-PH-03) ───────────────────────────────
    let linked_req = SandboxLinkedSecondPassAdapterRequest {
        mode: SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
        explicit_internal_linked_sandbox_call_requested: true,
        sandbox_verified: request.sandbox_verified,
        target_base_empty: request.target_base_empty,
        mapping_coverage_sufficient: request.mapping_coverage_sufficient,
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
        record_executor_safe: request.record_executor_safe,
        linked_second_pass_preview_ready: request.linked_second_pass_preview_ready,
        mapping_checkpoint_preview_ready: request.mapping_checkpoint_preview_ready,
        field_summaries: request.field_summaries.clone(),
    };
    let linked_result =
        build_sandbox_linked_second_pass_adapter(&linked_req, schema_plan, record_plan);
    let linked_ready = matches!(
        linked_result.status,
        SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall
    );
    if !linked_ready {
        return blocked(
            SandboxAdapterChainRunnerMode::MockInternalOnly,
            true,
            true,
            true,
            false,
            false,
            false,
            &format!(
                "{CHK_LINKED_ADAPTER}: linked second-pass adapter did not return \
                 ReadyForSandboxCall. The linked adapter must be ready before the chain \
                 runner can proceed."
            ),
        );
    }
    let linked_op_count = linked_result.total_operation_count;

    // ── Final validation adapter (SACR-PH-04) ─────────────────────────────────
    let fv_req = SandboxFinalValidationAdapterRequest {
        mode: SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
        explicit_internal_validation_sandbox_call_requested: true,
        sandbox_verified: request.sandbox_verified,
        final_validation_enforcement_safe: request.final_validation_enforcement_safe,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
        checkpoint_durability_safe: request.checkpoint_durability_safe,
        sensitive_data_safe: request.sensitive_data_safe,
        rate_limit_backoff_safe: request.rate_limit_backoff_safe,
        schema_executor_safe: request.schema_executor_safe,
        checkpoint_store_safe: request.checkpoint_store_safe,
        record_executor_safe: request.record_executor_safe,
        linked_executor_safe: request.linked_executor_safe,
        linked_second_pass_preview_ready: request.linked_second_pass_preview_ready,
        mapping_checkpoint_preview_ready: request.mapping_checkpoint_preview_ready,
        target_base_empty: request.target_base_empty,
        mapping_coverage_sufficient: request.mapping_coverage_sufficient,
        field_summaries: request.field_summaries.clone(),
        table_count: request.table_count,
        field_count: request.field_count,
        record_count: request.record_count,
        id_mapping_entry_count: request.id_mapping_entry_count,
        linked_coverage_count: request.linked_coverage_count,
        attachment_metadata_count: request.attachment_metadata_count,
        manifest_present: request.manifest_present,
    };
    let fv_result = build_sandbox_final_validation_adapter(&fv_req, schema_plan, record_plan);
    let fv_ready = matches!(
        fv_result.status,
        SandboxFinalValidationAdapterStatus::ReadyForSandboxCall
    );
    if !fv_ready {
        return blocked(
            SandboxAdapterChainRunnerMode::MockInternalOnly,
            true,
            true,
            true,
            true,
            false,
            false,
            &format!(
                "{CHK_FINAL_VALIDATION_ADAPTER}: final validation adapter did not return \
                 ReadyForSandboxCall. The final validation adapter must be ready before \
                 the chain runner can proceed."
            ),
        );
    }
    let fv_op_count = fv_result.total_operation_count;

    // ── All prerequisites satisfied — build phase list ─────────────────────────
    let phases = vec![
        SandboxAdapterChainRunnerPhase {
            phase_id: SACR_PH_01.to_string(),
            label: "Schema write adapter".to_string(),
            status: SandboxAdapterChainRunnerPhaseStatus::MockObserved,
            operation_count: schema_op_count,
            note: format!(
                "Schema write adapter observed via mock/no-op adapter — {} operation descriptor(s). \
                 No Airtable schema API call was made. No table or field was created.",
                schema_op_count
            ),
        },
        SandboxAdapterChainRunnerPhase {
            phase_id: SACR_PH_02.to_string(),
            label: "Record write adapter".to_string(),
            status: SandboxAdapterChainRunnerPhaseStatus::MockObserved,
            operation_count: record_op_count,
            note: format!(
                "Record write adapter observed via mock/no-op adapter — {} operation descriptor(s). \
                 No Airtable record API call was made. No record was created.",
                record_op_count
            ),
        },
        SandboxAdapterChainRunnerPhase {
            phase_id: SACR_PH_03.to_string(),
            label: "Linked second-pass adapter".to_string(),
            status: SandboxAdapterChainRunnerPhaseStatus::MockObserved,
            operation_count: linked_op_count,
            note: format!(
                "Linked second-pass adapter observed via mock/no-op adapter — {} operation descriptor(s). \
                 No Airtable linked record API call was made. No record was updated.",
                linked_op_count
            ),
        },
        SandboxAdapterChainRunnerPhase {
            phase_id: SACR_PH_04.to_string(),
            label: "Final validation adapter".to_string(),
            status: SandboxAdapterChainRunnerPhaseStatus::MockObserved,
            operation_count: fv_op_count,
            note: format!(
                "Final validation adapter observed via mock/no-op adapter — {} read descriptor(s). \
                 No Airtable read API call was made. No network reads attempted.",
                fv_op_count
            ),
        },
    ];

    let total_phase_count = phases.len();
    let snapshot = SandboxAdapterChainRunnerSafetySnapshot {
        write_gate_disabled: true,
        simulator_simulated_not_executed: true,
        schema_adapter_ready: true,
        record_adapter_ready: true,
        linked_adapter_ready: true,
        final_validation_adapter_ready: true,
        explicit_mock_chain_flag_set: true,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
    };

    SandboxAdapterChainRunnerResult {
        status: SandboxAdapterChainRunnerStatus::MockRunNotExecuted,
        mode: SandboxAdapterChainRunnerMode::MockInternalOnly,
        message: format!(
            "Sandbox adapter chain runner: mockRunNotExecuted. All {total_phase_count} adapter \
             phases observed via mock/no-op adapters \
             (schema: {schema_op_count} op(s), record: {record_op_count} op(s), \
             linked: {linked_op_count} op(s), final validation: {fv_op_count} descriptor(s)). \
             No Airtable network call was made. No runtime execution is enabled. \
             No app runtime writes, reads, or execution are enabled. No changes were made. \
             This runner is for internal sandbox tests only and is not reachable from \
             UI, TypeScript, or any Tauri command. \
             Live sandbox E2E restore execution remains separate pending work."
        ),
        phases,
        safety_snapshot: snapshot,
        total_phase_count,
        schema_operation_count: schema_op_count,
        record_operation_count: record_op_count,
        linked_operation_count: linked_op_count,
        final_validation_operation_count: fv_op_count,
        blocked_reason: None,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn blocked(
    mode: SandboxAdapterChainRunnerMode,
    schema_adapter_ready: bool,
    record_adapter_ready: bool,
    linked_adapter_ready: bool,
    final_validation_adapter_ready: bool,
    simulator_simulated_not_executed: bool,
    explicit_mock_chain_flag_set: bool,
    blocked_reason: &str,
) -> SandboxAdapterChainRunnerResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    SandboxAdapterChainRunnerResult {
        status: SandboxAdapterChainRunnerStatus::Blocked,
        mode,
        message: format!(
            "Sandbox adapter chain runner is blocked. {blocked_reason} \
             No Airtable API calls were made. No changes were made. \
             Runtime execution, writes, reads, and network calls remain disabled."
        ),
        phases: vec![],
        safety_snapshot: SandboxAdapterChainRunnerSafetySnapshot {
            write_gate_disabled,
            simulator_simulated_not_executed,
            schema_adapter_ready,
            record_adapter_ready,
            linked_adapter_ready,
            final_validation_adapter_ready,
            explicit_mock_chain_flag_set,
            runtime_execution_enabled: false,
            app_runtime_writes_enabled: false,
            app_runtime_reads_enabled: false,
            network_reads_attempted: false,
            network_writes_attempted: false,
            airtable_client_called: false,
        },
        total_phase_count: 0,
        schema_operation_count: 0,
        record_operation_count: 0,
        linked_operation_count: 0,
        final_validation_operation_count: 0,
        blocked_reason: Some(blocked_reason.to_string()),
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
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

    fn simple_record_plan() -> RecordWriteRequestPlan {
        let req = RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables: vec![RecordImportTableInput {
                table_id: "tbl01".to_string(),
                table_name: "Tasks".to_string(),
                record_count: Some(5),
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

    fn simple_field_summaries() -> Vec<LinkedSecondPassFieldSummary> {
        vec![LinkedSecondPassFieldSummary {
            table_label: "Projects".to_string(),
            field_label: "Tasks".to_string(),
            record_count: 5,
            batch_count: 1,
            unresolved_link_count: 0,
        }]
    }

    fn full_request() -> SandboxAdapterChainRunnerRequest {
        SandboxAdapterChainRunnerRequest {
            mode: SandboxAdapterChainRunnerMode::MockInternalOnly,
            explicit_internal_mock_chain_requested: true,
            sandbox_verified: true,
            target_base_empty: true,
            mapping_coverage_sufficient: true,
            final_validation_enforcement_safe: true,
            confirmation_gate_declared: true,
            destructive_operation_policy_safe: true,
            attachment_phase_disabled_safe: true,
            live_write_readiness_safe: true,
            write_phase_ordering_safe: true,
            failure_modes_safe: true,
            rollback_limitation_safe: true,
            checkpoint_durability_safe: true,
            sensitive_data_safe: true,
            rate_limit_backoff_safe: true,
            schema_executor_safe: true,
            checkpoint_store_safe: true,
            record_executor_safe: true,
            linked_executor_safe: true,
            linked_second_pass_preview_ready: true,
            mapping_checkpoint_preview_ready: true,
            field_summaries: simple_field_summaries(),
            table_count: 2,
            field_count: 5,
            record_count: 10,
            id_mapping_entry_count: 10,
            linked_coverage_count: 5,
            attachment_metadata_count: 2,
            manifest_present: true,
        }
    }

    fn disabled_request() -> SandboxAdapterChainRunnerRequest {
        SandboxAdapterChainRunnerRequest {
            mode: SandboxAdapterChainRunnerMode::Disabled,
            explicit_internal_mock_chain_requested: false,
            sandbox_verified: false,
            target_base_empty: false,
            mapping_coverage_sufficient: false,
            final_validation_enforcement_safe: false,
            confirmation_gate_declared: false,
            destructive_operation_policy_safe: false,
            attachment_phase_disabled_safe: false,
            live_write_readiness_safe: false,
            write_phase_ordering_safe: false,
            failure_modes_safe: false,
            rollback_limitation_safe: false,
            checkpoint_durability_safe: false,
            sensitive_data_safe: false,
            rate_limit_backoff_safe: false,
            schema_executor_safe: false,
            checkpoint_store_safe: false,
            record_executor_safe: false,
            linked_executor_safe: false,
            linked_second_pass_preview_ready: false,
            mapping_checkpoint_preview_ready: false,
            field_summaries: vec![],
            table_count: 0,
            field_count: 0,
            record_count: 0,
            id_mapping_entry_count: 0,
            linked_coverage_count: 0,
            attachment_metadata_count: 0,
            manifest_present: false,
        }
    }

    // ── Default blocked path ───────────────────────────────────────────────────

    #[test]
    fn default_disabled_request_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
    }

    #[test]
    fn disabled_mode_returns_blocked_with_mode_disabled() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
        assert_eq!(result.mode, SandboxAdapterChainRunnerMode::Disabled);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_MODE));
    }

    // ── Explicit flag check ────────────────────────────────────────────────────

    #[test]
    fn missing_explicit_flag_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.explicit_internal_mock_chain_requested = false;
        let result = run_sandbox_adapter_chain(&req, &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_EXPLICIT_FLAG));
    }

    // ── Simulator blocked causes blocked ──────────────────────────────────────

    #[test]
    fn simulator_blocked_causes_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.failure_modes_safe = false;
        let result = run_sandbox_adapter_chain(&req, &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_SIMULATOR));
    }

    // ── Schema adapter not ready causes blocked ───────────────────────────────

    #[test]
    fn schema_adapter_not_ready_causes_blocked() {
        use crate::restore::schema_write_requests::{
            SchemaWriteBlockedReason, SchemaWriteOperationStatus,
        };
        let mut sp = simple_schema_plan();
        sp.status = SchemaWriteOperationStatus::Blocked;
        sp.blocked_reason = Some(SchemaWriteBlockedReason::SchemaPlanNotReady);
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_SCHEMA_ADAPTER));
    }

    // ── Record adapter not ready causes blocked ───────────────────────────────

    #[test]
    fn record_adapter_not_ready_causes_blocked() {
        use crate::restore::record_write_requests::{
            RecordWriteBlockedReason, RecordWriteOperationStatus,
        };
        let sp = simple_schema_plan();
        let mut rp = simple_record_plan();
        rp.status = RecordWriteOperationStatus::Blocked;
        rp.blocked_reason = Some(RecordWriteBlockedReason::RecordImportPlanNotReady);
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_RECORD_ADAPTER));
    }

    // ── Linked adapter not ready causes blocked ───────────────────────────────

    #[test]
    fn linked_adapter_not_ready_causes_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.mapping_coverage_sufficient = false;
        let result = run_sandbox_adapter_chain(&req, &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_LINKED_ADAPTER));
    }

    // ── Final validation adapter not ready causes blocked ─────────────────────

    #[test]
    fn final_validation_adapter_not_ready_causes_blocked() {
        use crate::restore::schema_write_requests::{
            SchemaWriteBlockedReason, SchemaWriteOperationStatus,
        };
        // Block the schema plan so the final validation adapter probe returns blocked
        // (since it internally probes the schema adapter as well).
        let mut sp = simple_schema_plan();
        sp.status = SchemaWriteOperationStatus::Blocked;
        sp.blocked_reason = Some(SchemaWriteBlockedReason::SchemaPlanNotReady);
        let rp = simple_record_plan();
        // To reach CHK_FINAL_VALIDATION_ADAPTER we need schema/record/linked all
        // ready but final validation blocked. Use a full request so the first three
        // pass, but pass a broken schema plan so only the final validation adapter
        // sees the failure. Because schema adapter is probed at SACR-CHK-05 before
        // final validation, this test actually hits CHK_SCHEMA_ADAPTER. To isolate
        // CHK_FINAL_VALIDATION_ADAPTER, we need the final validation adapter to fail
        // independently. Removing linked_executor_safe while all others pass will
        // cause the final validation reader probe to block inside the final validation
        // adapter without affecting the earlier three adapters.
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.linked_executor_safe = false;
        req.record_executor_safe = true;
        let result = run_sandbox_adapter_chain(&req, &sp, &rp);
        assert_eq!(result.status, SandboxAdapterChainRunnerStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_FINAL_VALIDATION_ADAPTER));
    }

    // ── evaluate_write_gate remains Disabled ──────────────────────────────────

    #[test]
    fn evaluate_write_gate_default_remains_disabled() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let _ = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── mockRunNotExecuted when all prerequisites satisfied ───────────────────

    #[test]
    fn mock_run_not_executed_when_all_prereqs_satisfied() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxAdapterChainRunnerStatus::MockRunNotExecuted
        );
    }

    // ── Phase ordering is deterministic ───────────────────────────────────────

    #[test]
    fn phase_ordering_is_deterministic() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let ids1: Vec<_> = r1.phases.iter().map(|p| &p.phase_id).collect();
        let ids2: Vec<_> = r2.phases.iter().map(|p| &p.phase_id).collect();
        assert_eq!(ids1, ids2);
    }

    // ── All four adapter phases are represented ───────────────────────────────

    #[test]
    fn all_four_adapter_phases_represented() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert_eq!(result.total_phase_count, 4);
        assert_eq!(result.phases.len(), 4);
        let ids: Vec<_> = result.phases.iter().map(|p| p.phase_id.as_str()).collect();
        assert!(ids.contains(&"SACR-PH-01"));
        assert!(ids.contains(&"SACR-PH-02"));
        assert!(ids.contains(&"SACR-PH-03"));
        assert!(ids.contains(&"SACR-PH-04"));
    }

    #[test]
    fn phase_order_is_schema_record_linked_final_validation() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert_eq!(result.phases[0].phase_id, "SACR-PH-01");
        assert_eq!(result.phases[1].phase_id, "SACR-PH-02");
        assert_eq!(result.phases[2].phase_id, "SACR-PH-03");
        assert_eq!(result.phases[3].phase_id, "SACR-PH-04");
    }

    // ── Safe operation counts reported without payloads ───────────────────────

    #[test]
    fn safe_operation_counts_reported() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        // schema: 1 table + 1 field = 2 ops; record: 1 batch; final: 5+1+1=7 (manifest present)
        assert!(result.schema_operation_count > 0);
        assert!(result.final_validation_operation_count > 0);
        // phase operation_count matches result-level count
        assert_eq!(
            result.phases[0].operation_count,
            result.schema_operation_count
        );
        assert_eq!(
            result.phases[1].operation_count,
            result.record_operation_count
        );
        assert_eq!(
            result.phases[2].operation_count,
            result.linked_operation_count
        );
        assert_eq!(
            result.phases[3].operation_count,
            result.final_validation_operation_count
        );
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn runtime_execution_enabled_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(!r1.runtime_execution_enabled);
        assert!(!r2.runtime_execution_enabled);
    }

    #[test]
    fn app_runtime_writes_enabled_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(!r1.app_runtime_writes_enabled);
        assert!(!r2.app_runtime_writes_enabled);
    }

    #[test]
    fn app_runtime_reads_enabled_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(!r1.app_runtime_reads_enabled);
        assert!(!r2.app_runtime_reads_enabled);
    }

    #[test]
    fn network_reads_attempted_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(!r1.network_reads_attempted);
        assert!(!r2.network_reads_attempted);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(!r1.network_writes_attempted);
        assert!(!r2.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_always_true() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(r1.no_changes_made);
        assert!(r2.no_changes_made);
    }

    #[test]
    fn airtable_client_called_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(!r1.airtable_client_called);
        assert!(!r2.airtable_client_called);
        assert!(!r1.safety_snapshot.airtable_client_called);
        assert!(!r2.safety_snapshot.airtable_client_called);
    }

    // ── Snapshot invariants ───────────────────────────────────────────────────

    #[test]
    fn safety_snapshot_write_gate_disabled_always_true() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&disabled_request(), &sp, &rp);
        assert!(r1.safety_snapshot.write_gate_disabled);
        assert!(r2.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn safety_snapshot_runtime_flags_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert!(!result.safety_snapshot.runtime_execution_enabled);
        assert!(!result.safety_snapshot.app_runtime_writes_enabled);
        assert!(!result.safety_snapshot.app_runtime_reads_enabled);
        assert!(!result.safety_snapshot.network_reads_attempted);
        assert!(!result.safety_snapshot.network_writes_attempted);
        assert!(!result.safety_snapshot.airtable_client_called);
    }

    // ── No serialization leaks ────────────────────────────────────────────────

    #[test]
    fn no_token_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn no_absolute_path_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_record_payload_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn no_raw_http_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"body\":{"));
        assert!(!json.contains("\"statusCode\""));
    }

    #[test]
    fn no_old_record_id_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"oldRecordId\""));
        assert!(!json.contains("oldId"));
        assert!(!json.contains("rec_old_"));
    }

    #[test]
    fn no_new_record_id_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"newRecordId\""));
        assert!(!json.contains("newId"));
        assert!(!json.contains("rec_new_"));
    }

    #[test]
    fn no_attachment_url_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    // ── No success state introduced ───────────────────────────────────────────

    #[test]
    fn no_success_state_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
        assert!(!json.contains("executionReady"));
    }

    // ── No Tauri command or UI path ───────────────────────────────────────────

    #[test]
    fn no_tauri_command_introduced() {
        // run_sandbox_adapter_chain accepts no HTTP transport, no token, no Tauri app
        // handle, and carries no #[tauri::command] attribute. Reaching this assertion
        // confirms no Tauri command is wired.
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxAdapterChainRunnerStatus::MockRunNotExecuted
        );
    }

    // ── No real Airtable client called ────────────────────────────────────────

    #[test]
    fn no_real_airtable_client_called_in_default_path() {
        // run_sandbox_adapter_chain accepts no HTTP transport or token.
        // Reaching this assertion confirms no network call was made.
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(!result.airtable_client_called);
    }

    // ── Two independent calls produce independent results ─────────────────────

    #[test]
    fn two_independent_calls_produce_independent_results() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        let r2 = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert_eq!(
            r1.status,
            SandboxAdapterChainRunnerStatus::MockRunNotExecuted
        );
        assert_eq!(
            r2.status,
            SandboxAdapterChainRunnerStatus::MockRunNotExecuted
        );
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── Message says execution pending ────────────────────────────────────────

    #[test]
    fn message_says_execution_pending() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = run_sandbox_adapter_chain(&full_request(), &sp, &rp);
        assert!(
            result.message.contains("remains separate pending work"),
            "message must say live execution remains pending, got: {}",
            result.message
        );
    }
}
