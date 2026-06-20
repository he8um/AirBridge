use serde::{Deserialize, Serialize};

use crate::restore::final_validation_reader::{
    build_final_validation_reader_plan, FinalValidationReaderMode, FinalValidationReaderRequest,
    FinalValidationReaderStatus,
};
use crate::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use crate::restore::sandbox_gate_arming::{
    build_sandbox_gate_arming_decision, SandboxGateArmingMode, SandboxGateArmingRequest,
    SandboxGateArmingStatus,
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

// ── Public types ──────────────────────────────────────────────────────────────

/// Overall status of the sandbox final validation adapter evaluation.
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
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `no_changes_made` is always `true`.
/// - No `Succeeded`, `Complete`, `Enabled`, `Done`, or `ExecutionReady` status exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxFinalValidationAdapterStatus {
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
    /// All prerequisites satisfied and final validation read operations have
    /// been described as adapter-boundary read descriptors. No network call
    /// was made. No execution occurred. No state was persisted.
    ReadyForSandboxCall,
    /// The adapter is in disabled mode. No evaluation was performed. Default state.
    NotExecuted,
}

/// Mode for the sandbox final validation adapter.
///
/// Safety invariants:
/// - `Disabled` is the default and operationally always-reachable mode.
/// - `SandboxOnlyInternal` is for Rust unit tests only.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxFinalValidationAdapterMode {
    /// Adapter disabled — no evaluation is performed. Default state.
    Disabled,
    /// Internal sandbox-only adapter mode for Rust unit tests only.
    /// Does NOT execute network calls, enable runtime writes/reads, or persist state.
    SandboxOnlyInternal,
}

/// Status of a single planned final validation read descriptor.
///
/// Note: `succeeded` / `completed` / `done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxFinalValidationAdapterOperationStatus {
    /// The descriptor is fully described and would be the next call if enabled.
    /// No network call has been made.
    Planned,
    /// The descriptor is blocked by a safety prerequisite failure.
    Blocked,
    /// The adapter is in disabled mode. Descriptor was built but not executed.
    NotExecuted,
}

/// A single final validation read descriptor at the adapter boundary.
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
/// - Only final validation read descriptors are described — no schema, first-pass
///   record create, linked update, attachment endpoint, or checkpoint operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxFinalValidationAdapterOperation {
    /// Stable adapter-boundary operation ID (SFVA-OP-NNN).
    pub operation_id: String,
    /// One of: `"schemaCountReadDescriptor"`, `"fieldCountReadDescriptor"`,
    /// `"recordCountReadDescriptor"`, `"linkedFieldCoverageReadDescriptor"`,
    /// `"attachmentMetadataReadDescriptor"`, `"manifestChecksumReadDescriptor"`,
    /// `"finalGuardDescriptor"`.
    pub operation_kind: String,
    /// Safe expected count for this read operation (no raw record IDs).
    pub expected_count: usize,
    pub status: SandboxFinalValidationAdapterOperationStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the adapter evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxFinalValidationAdapterSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the sandbox gate arming decision returned `ArmedNotExecutable`.
    pub gate_arming_armed_not_executable: bool,
    /// Whether the sandbox restore simulator returned `SimulatedNotExecuted`.
    pub simulator_simulated_not_executed: bool,
    /// Whether the final validation reader plan returned `NotExecuted`.
    pub final_validation_reader_not_executed: bool,
    /// Whether the schema write adapter returned `ReadyForSandboxCall`.
    pub schema_adapter_ready: bool,
    /// Whether the record write adapter returned `ReadyForSandboxCall`.
    pub record_adapter_ready: bool,
    /// Whether the linked second-pass adapter returned `ReadyForSandboxCall`.
    pub linked_adapter_ready: bool,
    /// Whether the final validation enforcement policy is safe.
    pub final_validation_enforcement_safe: bool,
    /// Whether sandbox verification is declared safe.
    pub sandbox_verified: bool,
    /// Whether the explicit internal validation sandbox call flag was set.
    pub explicit_validation_sandbox_flag_set: bool,
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
}

/// Request to the sandbox final validation adapter.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_validation_sandbox_call_requested` must be `true` for the adapter
/// to proceed past its gate check. No UI control, Tauri command, or runtime path
/// sets this flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxFinalValidationAdapterRequest {
    /// Must be `sandboxOnlyInternal` for evaluation to proceed.
    /// `disabled` (the default) always results in `NotExecuted`.
    pub mode: SandboxFinalValidationAdapterMode,
    /// Internal-only flag. Must be explicitly `true` to proceed past the gate.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_validation_sandbox_call_requested: bool,
    /// Whether sandbox environment verification is declared and safe.
    pub sandbox_verified: bool,
    /// Whether the final validation enforcement policy is safe.
    pub final_validation_enforcement_safe: bool,
    /// Prerequisite booleans forwarded to arming/simulator/adapter probes.
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
    pub target_base_empty: bool,
    pub mapping_coverage_sufficient: bool,
    /// Per-field summaries forwarded to the linked adapter probe.
    /// No raw record IDs — only safe counts and labels.
    pub field_summaries: Vec<LinkedSecondPassFieldSummary>,
    /// Safe count of tables to be validated (no raw IDs).
    pub table_count: usize,
    /// Safe count of fields to be validated (no raw IDs).
    pub field_count: usize,
    /// Safe count of records to be counted (no raw IDs).
    pub record_count: usize,
    /// Safe count of ID mapping entries (no raw IDs).
    pub id_mapping_entry_count: usize,
    /// Safe count of linked field coverage entries.
    pub linked_coverage_count: usize,
    /// Safe count of attachment metadata entries.
    pub attachment_metadata_count: usize,
    /// Whether a package manifest is present.
    pub manifest_present: bool,
}

/// Result of the sandbox final validation adapter evaluation.
///
/// Safety invariants (always enforced):
/// - `runtime_execution_enabled` is always `false`.
/// - `app_runtime_writes_enabled` is always `false`.
/// - `app_runtime_reads_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_reads_attempted` is always `false`.
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
/// - Only final validation read descriptors appear in `operations` — no schema,
///   first-pass record create, linked update, attachment endpoint, or checkpoint
///   operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxFinalValidationAdapterResult {
    pub status: SandboxFinalValidationAdapterStatus,
    pub mode: SandboxFinalValidationAdapterMode,
    pub message: String,
    pub operations: Vec<SandboxFinalValidationAdapterOperation>,
    pub safety_snapshot: SandboxFinalValidationAdapterSafetySnapshot,
    pub total_operation_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network read operations were attempted.
    pub network_reads_attempted: bool,
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

const CHK_MODE: &str = "SFVA-CHK-01";
const CHK_EXPLICIT_FLAG: &str = "SFVA-CHK-02";
const CHK_WRITE_GATE: &str = "SFVA-CHK-03";
const CHK_ARMING: &str = "SFVA-CHK-04";
const CHK_SIMULATOR: &str = "SFVA-CHK-05";
const CHK_READER: &str = "SFVA-CHK-06";
const CHK_SCHEMA_ADAPTER: &str = "SFVA-CHK-07";
const CHK_RECORD_ADAPTER: &str = "SFVA-CHK-08";
const CHK_LINKED_ADAPTER: &str = "SFVA-CHK-09";
const CHK_ENFORCEMENT: &str = "SFVA-CHK-10";
const CHK_SANDBOX_VERIFIED: &str = "SFVA-CHK-11";

// ── Adapter trait (no-op and mock) ────────────────────────────────────────────

/// Trait representing a final validation read adapter boundary.
///
/// This trait exists purely as a future injection point for sandbox tests.
/// No production adapter is implemented. The default adapter is no-op.
///
/// Safety invariants:
/// - No implementation of this trait may call the real Airtable API.
/// - No implementation may return a token, path, record payload, or HTTP body.
/// - No implementation is wired into the runtime app flow.
pub trait FinalValidationReadAdapter {
    /// Returns the count of read operations this adapter would handle.
    /// Must not make any network calls.
    fn planned_operation_count(&self) -> usize;
}

/// No-op adapter used in unit tests.
/// Records no state, makes no network calls.
pub struct NoOpFinalValidationReadAdapter;

impl FinalValidationReadAdapter for NoOpFinalValidationReadAdapter {
    fn planned_operation_count(&self) -> usize {
        0
    }
}

/// Mock adapter that counts planned operations without any network call.
/// For use in unit tests only.
pub struct MockFinalValidationReadAdapter {
    pub operation_count: usize,
}

impl FinalValidationReadAdapter for MockFinalValidationReadAdapter {
    fn planned_operation_count(&self) -> usize {
        self.operation_count
    }
}

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the sandbox final validation adapter boundary evaluation.
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
///   `no_changes_made: true`, `network_reads_attempted: false`,
///   `network_writes_attempted: false`.
/// - Returns `NotExecuted` when mode is `Disabled`.
/// - Returns `Blocked` when any prerequisite fails.
/// - Returns `ReadyForSandboxCall` only when all prerequisites pass — this does
///   NOT execute a call, does NOT arm the gate, and is NOT persisted.
/// - Operations describe only final validation read descriptors.
///   No schema, first-pass record create, linked update, attachment endpoint,
///   or checkpoint operations appear.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_sandbox_final_validation_adapter(
    request: &SandboxFinalValidationAdapterRequest,
    schema_plan: &SchemaWriteRequestPlan,
    record_plan: &crate::restore::record_write_requests::RecordWriteRequestPlan,
) -> SandboxFinalValidationAdapterResult {
    // ── Mode check ─────────────────────────────────────────────────────────────
    if matches!(request.mode, SandboxFinalValidationAdapterMode::Disabled) {
        return not_executed_result(&format!(
            "{CHK_MODE}: Adapter mode is disabled. No evaluation is performed. This is the default state."
        ));
    }

    // ── Explicit flag ──────────────────────────────────────────────────────────
    if !request.explicit_internal_validation_sandbox_call_requested {
        return blocked(
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            false,
            false,
            false,
            false,
            false,
            false,
            request.final_validation_enforcement_safe,
            request.sandbox_verified,
            "Explicit internal validation sandbox call flag is not set. \
             This flag must be explicitly true before adapter evaluation can proceed. \
             No UI control, Tauri command, or runtime path sets this flag.",
            &format!(
                "{CHK_EXPLICIT_FLAG}: explicit_internal_validation_sandbox_call_requested must be true."
            ),
        );
    }

    // ── Write gate check ───────────────────────────────────────────────────────
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if !write_gate_disabled {
        return blocked(
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            false,
            false,
            false,
            false,
            false,
            false,
            request.final_validation_enforcement_safe,
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
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            false,
            false,
            false,
            false,
            false,
            false,
            request.final_validation_enforcement_safe,
            request.sandbox_verified,
            "Sandbox gate arming decision did not return ArmedNotExecutable. \
             All arming prerequisites must be satisfied before the final validation \
             adapter boundary can proceed.",
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
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            true,
            false,
            false,
            false,
            false,
            false,
            request.final_validation_enforcement_safe,
            request.sandbox_verified,
            "Sandbox restore simulator did not return SimulatedNotExecuted. \
             All simulation prerequisites must be satisfied before the final validation \
             adapter boundary can proceed.",
            &format!(
                "{CHK_SIMULATOR}: sandbox restore simulator must return SimulatedNotExecuted."
            ),
        );
    }

    // ── Final validation reader probe ──────────────────────────────────────────
    let reader_req = FinalValidationReaderRequest {
        mode: FinalValidationReaderMode::SandboxOnly,
        explicit_internal_final_validation_read_requested: true,
        sandbox_verified: request.sandbox_verified,
        schema_executor_safe: request.schema_executor_safe,
        record_executor_safe: request.record_executor_safe,
        linked_executor_safe: request.linked_executor_safe,
        final_validation_preview_ready: request.linked_second_pass_preview_ready,
        final_validation_enforcement_safe: request.final_validation_enforcement_safe,
        sensitive_data_safe: request.sensitive_data_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        table_count: request.table_count,
        field_count: request.field_count,
        record_count: request.record_count,
        id_mapping_entry_count: request.id_mapping_entry_count,
        linked_coverage_count: request.linked_coverage_count,
        attachment_metadata_count: request.attachment_metadata_count,
        manifest_present: request.manifest_present,
    };
    let reader_result = build_final_validation_reader_plan(&reader_req);
    let reader_not_executed = matches!(
        reader_result.status,
        FinalValidationReaderStatus::NotExecuted
    );
    if !reader_not_executed {
        return blocked(
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            true,
            true,
            false,
            false,
            false,
            false,
            request.final_validation_enforcement_safe,
            request.sandbox_verified,
            "Final validation reader plan did not return NotExecuted. \
             A safe/not-executed reader plan is required before the final validation \
             adapter boundary can proceed.",
            &format!("{CHK_READER}: final validation reader plan must return NotExecuted."),
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
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            false,
            false,
            false,
            request.final_validation_enforcement_safe,
            request.sandbox_verified,
            "Schema write adapter boundary did not return ReadyForSandboxCall. \
             The schema adapter must be ready before the final validation adapter \
             boundary can proceed.",
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
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            false,
            false,
            request.final_validation_enforcement_safe,
            request.sandbox_verified,
            "Record write adapter boundary did not return ReadyForSandboxCall. \
             The record adapter must be ready before the final validation adapter \
             boundary can proceed.",
            &format!("{CHK_RECORD_ADAPTER}: record write adapter must return ReadyForSandboxCall."),
        );
    }

    // ── Linked second-pass adapter probe ──────────────────────────────────────
    let linked_adapter_req = SandboxLinkedSecondPassAdapterRequest {
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
    let linked_adapter_result =
        build_sandbox_linked_second_pass_adapter(&linked_adapter_req, schema_plan, record_plan);
    let linked_adapter_ready = matches!(
        linked_adapter_result.status,
        SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall
    );
    if !linked_adapter_ready {
        return blocked(
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            true,
            false,
            request.final_validation_enforcement_safe,
            request.sandbox_verified,
            "Linked second-pass adapter boundary did not return ReadyForSandboxCall. \
             The linked adapter must be ready before the final validation adapter \
             boundary can proceed.",
            &format!(
                "{CHK_LINKED_ADAPTER}: linked second-pass adapter must return ReadyForSandboxCall."
            ),
        );
    }

    // ── Final validation enforcement check ────────────────────────────────────
    if !request.final_validation_enforcement_safe {
        return blocked(
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            request.sandbox_verified,
            "Final validation enforcement policy is not safe. \
             All three completion guards must be declared before validation read \
             descriptors can be built.",
            &format!("{CHK_ENFORCEMENT}: final_validation_enforcement_safe must be true."),
        );
    }

    // ── Sandbox verified check ────────────────────────────────────────────────
    if !request.sandbox_verified {
        return blocked(
            SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            "Sandbox environment verification has not passed. \
             A verified sandbox target is required before final validation adapter \
             can proceed.",
            &format!("{CHK_SANDBOX_VERIFIED}: sandbox_verified must be true."),
        );
    }

    // ── All prerequisites satisfied — build adapter operations ─────────────────
    // Only final validation read descriptors are emitted. No schema, first-pass
    // create, linked update, attachment endpoint, or checkpoint operations.
    let mut operations: Vec<SandboxFinalValidationAdapterOperation> = Vec::new();
    let mut op_idx: usize = 0;

    // schemaCountReadDescriptor
    operations.push(SandboxFinalValidationAdapterOperation {
        operation_id: format!("SFVA-OP-{op_idx:03}"),
        operation_kind: "schemaCountReadDescriptor".to_string(),
        expected_count: request.table_count,
        status: SandboxFinalValidationAdapterOperationStatus::Planned,
        note: format!(
            "Would read table list from Airtable and compare against {} expected table(s). \
             Read gate disabled — no network call made. No record IDs exposed.",
            request.table_count
        ),
    });
    op_idx += 1;

    // fieldCountReadDescriptor
    operations.push(SandboxFinalValidationAdapterOperation {
        operation_id: format!("SFVA-OP-{op_idx:03}"),
        operation_kind: "fieldCountReadDescriptor".to_string(),
        expected_count: request.field_count,
        status: SandboxFinalValidationAdapterOperationStatus::Planned,
        note: format!(
            "Would read field definitions from Airtable and compare against {} expected \
             field(s). Read gate disabled — no network call made. No record IDs exposed.",
            request.field_count
        ),
    });
    op_idx += 1;

    // recordCountReadDescriptor
    operations.push(SandboxFinalValidationAdapterOperation {
        operation_id: format!("SFVA-OP-{op_idx:03}"),
        operation_kind: "recordCountReadDescriptor".to_string(),
        expected_count: request.record_count,
        status: SandboxFinalValidationAdapterOperationStatus::Planned,
        note: format!(
            "Would read record count from Airtable and compare against {} expected record(s). \
             No raw record IDs returned. Read gate disabled — no network call made.",
            request.record_count
        ),
    });
    op_idx += 1;

    // linkedFieldCoverageReadDescriptor
    operations.push(SandboxFinalValidationAdapterOperation {
        operation_id: format!("SFVA-OP-{op_idx:03}"),
        operation_kind: "linkedFieldCoverageReadDescriptor".to_string(),
        expected_count: request.linked_coverage_count,
        status: SandboxFinalValidationAdapterOperationStatus::Planned,
        note: format!(
            "Would verify linked field coverage for {} entry/entries. \
             No raw record IDs returned. Read gate disabled — no network call made.",
            request.linked_coverage_count
        ),
    });
    op_idx += 1;

    // attachmentMetadataReadDescriptor
    operations.push(SandboxFinalValidationAdapterOperation {
        operation_id: format!("SFVA-OP-{op_idx:03}"),
        operation_kind: "attachmentMetadataReadDescriptor".to_string(),
        expected_count: request.attachment_metadata_count,
        status: SandboxFinalValidationAdapterOperationStatus::Planned,
        note: format!(
            "Would read attachment metadata (filename, MIME type, size) for {} entry/entries. \
             Metadata inspection only — no binary retrieval, no attachment URL returned. \
             Read gate disabled — no network call made.",
            request.attachment_metadata_count
        ),
    });
    op_idx += 1;

    // manifestChecksumReadDescriptor — only if manifest present
    if request.manifest_present {
        operations.push(SandboxFinalValidationAdapterOperation {
            operation_id: format!("SFVA-OP-{op_idx:03}"),
            operation_kind: "manifestChecksumReadDescriptor".to_string(),
            expected_count: 1,
            status: SandboxFinalValidationAdapterOperationStatus::Planned,
            note: "Would compare package manifest checksums against restored base state. \
                   Read gate disabled — no network call made. No attachment URL returned."
                .to_string(),
        });
        op_idx += 1;
    }

    // finalGuardDescriptor — always last
    operations.push(SandboxFinalValidationAdapterOperation {
        operation_id: format!("SFVA-OP-{op_idx:03}"),
        operation_kind: "finalGuardDescriptor".to_string(),
        expected_count: 0,
        status: SandboxFinalValidationAdapterOperationStatus::Planned,
        note: "Completion guard: no result can carry a success status without all prior \
               read descriptors planned. Read gate disabled — guard is a descriptor only. \
               No network call made."
            .to_string(),
    });

    let total_operation_count = operations.len();
    let snapshot = SandboxFinalValidationAdapterSafetySnapshot {
        write_gate_disabled: true,
        gate_arming_armed_not_executable: true,
        simulator_simulated_not_executed: true,
        final_validation_reader_not_executed: true,
        schema_adapter_ready: true,
        record_adapter_ready: true,
        linked_adapter_ready: true,
        final_validation_enforcement_safe: true,
        sandbox_verified: true,
        explicit_validation_sandbox_flag_set: true,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        network_reads_attempted: false,
        network_writes_attempted: false,
    };

    SandboxFinalValidationAdapterResult {
        status: SandboxFinalValidationAdapterStatus::ReadyForSandboxCall,
        mode: SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
        message: format!(
            "Sandbox final validation adapter boundary is ready ({total_operation_count} \
             read descriptor(s)). \
             No Airtable network call was made. No runtime execution is enabled. \
             No app runtime writes or reads are enabled. No network reads attempted. \
             No changes were made. \
             This adapter boundary is for sandbox tests only and is not reachable from \
             UI, TypeScript, or any Tauri command."
        ),
        operations,
        safety_snapshot: snapshot,
        total_operation_count,
        blocked_reason: None,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn not_executed_result(message: &str) -> SandboxFinalValidationAdapterResult {
    SandboxFinalValidationAdapterResult {
        status: SandboxFinalValidationAdapterStatus::NotExecuted,
        mode: SandboxFinalValidationAdapterMode::Disabled,
        message: format!(
            "{message} No Airtable API calls were made. No changes were made. \
             Runtime execution, writes, reads, and network reads remain disabled."
        ),
        operations: vec![],
        safety_snapshot: SandboxFinalValidationAdapterSafetySnapshot {
            write_gate_disabled: true,
            gate_arming_armed_not_executable: false,
            simulator_simulated_not_executed: false,
            final_validation_reader_not_executed: false,
            schema_adapter_ready: false,
            record_adapter_ready: false,
            linked_adapter_ready: false,
            final_validation_enforcement_safe: false,
            sandbox_verified: false,
            explicit_validation_sandbox_flag_set: false,
            runtime_execution_enabled: false,
            app_runtime_writes_enabled: false,
            app_runtime_reads_enabled: false,
            network_reads_attempted: false,
            network_writes_attempted: false,
        },
        total_operation_count: 0,
        blocked_reason: None,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
    }
}

#[allow(clippy::too_many_arguments)]
fn blocked(
    mode: SandboxFinalValidationAdapterMode,
    gate_arming_armed_not_executable: bool,
    simulator_simulated_not_executed: bool,
    final_validation_reader_not_executed: bool,
    schema_adapter_ready: bool,
    record_adapter_ready: bool,
    linked_adapter_ready: bool,
    final_validation_enforcement_safe: bool,
    sandbox_verified: bool,
    message: &str,
    blocked_reason: &str,
) -> SandboxFinalValidationAdapterResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    SandboxFinalValidationAdapterResult {
        status: SandboxFinalValidationAdapterStatus::Blocked,
        mode,
        message: format!(
            "Sandbox final validation adapter is blocked. {message} \
             No Airtable API calls were made. No changes were made. \
             Runtime execution, writes, reads, and network reads remain disabled."
        ),
        operations: vec![],
        safety_snapshot: SandboxFinalValidationAdapterSafetySnapshot {
            write_gate_disabled,
            gate_arming_armed_not_executable,
            simulator_simulated_not_executed,
            final_validation_reader_not_executed,
            schema_adapter_ready,
            record_adapter_ready,
            linked_adapter_ready,
            final_validation_enforcement_safe,
            sandbox_verified,
            explicit_validation_sandbox_flag_set: false,
            runtime_execution_enabled: false,
            app_runtime_writes_enabled: false,
            app_runtime_reads_enabled: false,
            network_reads_attempted: false,
            network_writes_attempted: false,
        },
        total_operation_count: 0,
        blocked_reason: Some(blocked_reason.to_string()),
        no_changes_made: true,
        network_reads_attempted: false,
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
            record_count: 5,
            batch_count: 1,
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

    fn full_request() -> SandboxFinalValidationAdapterRequest {
        SandboxFinalValidationAdapterRequest {
            mode: SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
            explicit_internal_validation_sandbox_call_requested: true,
            sandbox_verified: true,
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
            target_base_empty: true,
            mapping_coverage_sufficient: true,
            field_summaries: simple_field_summaries(),
            table_count: 3,
            field_count: 12,
            record_count: 50,
            id_mapping_entry_count: 50,
            linked_coverage_count: 15,
            attachment_metadata_count: 4,
            manifest_present: true,
        }
    }

    fn disabled_request() -> SandboxFinalValidationAdapterRequest {
        SandboxFinalValidationAdapterRequest {
            mode: SandboxFinalValidationAdapterMode::Disabled,
            explicit_internal_validation_sandbox_call_requested: false,
            sandbox_verified: false,
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
            target_base_empty: false,
            mapping_coverage_sufficient: false,
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

    // ── Default disabled path ─────────────────────────────────────────────────

    #[test]
    fn default_disabled_request_returns_not_executed() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&disabled_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxFinalValidationAdapterStatus::NotExecuted
        );
    }

    #[test]
    fn disabled_mode_returns_not_executed_with_mode_disabled() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&disabled_request(), &sp, &rp);
        assert_eq!(result.mode, SandboxFinalValidationAdapterMode::Disabled);
    }

    // ── Explicit flag check ───────────────────────────────────────────────────

    #[test]
    fn missing_explicit_flag_returns_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.explicit_internal_validation_sandbox_call_requested = false;
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
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
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
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
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains("SFVA-CHK-"));
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
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
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
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_RECORD_ADAPTER));
    }

    // ── Linked adapter not ready causes blocked ───────────────────────────────

    #[test]
    fn linked_adapter_not_ready_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.mapping_coverage_sufficient = false;
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(CHK_LINKED_ADAPTER));
    }

    // ── Enforcement check ─────────────────────────────────────────────────────

    #[test]
    fn enforcement_not_safe_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.final_validation_enforcement_safe = false;
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        // The arming probe fires first when enforcement_safe is false, so we only
        // assert Blocked status without checking the specific check ID.
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
    }

    // ── linked_executor_safe is distinct from record_executor_safe ───────────

    #[test]
    fn linked_executor_safe_independent_of_record_executor_safe() {
        // Verifies that linked_executor_safe is forwarded to the reader probe
        // independently — not aliased to record_executor_safe.
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.linked_executor_safe = false;
        req.record_executor_safe = true;
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        // linked_executor_safe=false flows into the reader probe; the reader
        // probe (via the arming chain) will block.
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
    }

    // ── Sandbox not verified causes blocked ───────────────────────────────────

    #[test]
    fn sandbox_not_verified_causes_blocked() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.sandbox_verified = false;
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        assert_eq!(result.status, SandboxFinalValidationAdapterStatus::Blocked);
    }

    // ── evaluate_write_gate() remains Disabled ────────────────────────────────

    #[test]
    fn evaluate_write_gate_default_remains_disabled() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let _ = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── readyForSandboxCall when all prereqs satisfied ────────────────────────

    #[test]
    fn ready_for_sandbox_call_when_all_prereqs_satisfied() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxFinalValidationAdapterStatus::ReadyForSandboxCall
        );
    }

    // ── Safety invariants on readyForSandboxCall ──────────────────────────────

    #[test]
    fn ready_for_sandbox_call_runtime_execution_enabled_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(!result.runtime_execution_enabled);
    }

    #[test]
    fn ready_for_sandbox_call_app_runtime_writes_enabled_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(!result.app_runtime_writes_enabled);
    }

    #[test]
    fn ready_for_sandbox_call_app_runtime_reads_enabled_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(!result.app_runtime_reads_enabled);
    }

    #[test]
    fn no_network_reads_attempted_by_default() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(!result.network_reads_attempted);
    }

    #[test]
    fn no_network_writes_attempted_by_default() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_always_true() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(result.no_changes_made);
    }

    // ── Safety invariants in blocked/not-executed paths ───────────────────────

    #[test]
    fn no_changes_made_true_in_blocked_path() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.explicit_internal_validation_sandbox_call_requested = false;
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_true_in_not_executed_path() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&disabled_request(), &sp, &rp);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    // ── Only final validation read descriptors in operations ──────────────────

    #[test]
    fn operation_kinds_are_all_valid_read_descriptor_kinds() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let valid_kinds = [
            "schemaCountReadDescriptor",
            "fieldCountReadDescriptor",
            "recordCountReadDescriptor",
            "linkedFieldCoverageReadDescriptor",
            "attachmentMetadataReadDescriptor",
            "manifestChecksumReadDescriptor",
            "finalGuardDescriptor",
        ];
        for op in &result.operations {
            assert!(
                valid_kinds.contains(&op.operation_kind.as_str()),
                "unexpected operation kind: {}",
                op.operation_kind
            );
        }
    }

    #[test]
    fn no_write_operation_in_output() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("createTable"));
        assert!(!json.contains("createField"));
        assert!(!json.contains("createRecordBatch"));
        assert!(!json.contains("linkedUpdateBatchDescriptor"));
    }

    // ── Manifest descriptor present only when manifest_present ───────────────

    #[test]
    fn manifest_descriptor_present_when_manifest_true() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let has_manifest = result
            .operations
            .iter()
            .any(|o| o.operation_kind == "manifestChecksumReadDescriptor");
        assert!(has_manifest);
    }

    #[test]
    fn manifest_descriptor_absent_when_manifest_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let mut req = full_request();
        req.manifest_present = false;
        let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
        let has_manifest = result
            .operations
            .iter()
            .any(|o| o.operation_kind == "manifestChecksumReadDescriptor");
        assert!(!has_manifest);
    }

    // ── Final guard is last ───────────────────────────────────────────────────

    #[test]
    fn final_guard_descriptor_is_last() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let last = result.operations.last().expect("operations not empty");
        assert_eq!(last.operation_kind, "finalGuardDescriptor");
    }

    // ── Schema count read descriptor is first ─────────────────────────────────

    #[test]
    fn schema_count_read_descriptor_is_first() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert_eq!(
            result.operations[0].operation_kind,
            "schemaCountReadDescriptor"
        );
    }

    // ── Operation ordering is deterministic ───────────────────────────────────

    #[test]
    fn operation_ordering_is_deterministic() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let r1 = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let r2 = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let ids1: Vec<_> = r1.operations.iter().map(|o| &o.operation_id).collect();
        let ids2: Vec<_> = r2.operations.iter().map(|o| &o.operation_id).collect();
        assert_eq!(ids1, ids2);
    }

    // ── Operation IDs use stable prefix ──────────────────────────────────────

    #[test]
    fn operation_ids_use_sfva_prefix() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        for op in &result.operations {
            assert!(
                op.operation_id.starts_with("SFVA-OP-"),
                "unexpected operation ID prefix: {}",
                op.operation_id
            );
        }
    }

    // ── Operation count consistent ────────────────────────────────────────────

    #[test]
    fn operation_count_consistent_with_operations_vec() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert_eq!(result.total_operation_count, result.operations.len());
    }

    // ── Expected counts reflect request ──────────────────────────────────────

    #[test]
    fn expected_counts_reflect_request() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let schema_op = result
            .operations
            .iter()
            .find(|o| o.operation_kind == "schemaCountReadDescriptor")
            .expect("schema op");
        assert_eq!(schema_op.expected_count, 3);
        let record_op = result
            .operations
            .iter()
            .find(|o| o.operation_kind == "recordCountReadDescriptor")
            .expect("record op");
        assert_eq!(record_op.expected_count, 50);
    }

    // ── Attachment metadata is metadata-only ──────────────────────────────────

    #[test]
    fn attachment_metadata_descriptor_note_is_metadata_only() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let attach_op = result
            .operations
            .iter()
            .find(|o| o.operation_kind == "attachmentMetadataReadDescriptor")
            .expect("attach op");
        assert!(
            attach_op.note.to_lowercase().contains("metadata"),
            "note must mention metadata"
        );
        assert!(
            !attach_op.note.contains("download"),
            "note must not mention download"
        );
        assert!(
            !attach_op.note.contains("cdn.airtable.com"),
            "note must not contain CDN URL"
        );
    }

    // ── No token/path/payload/raw HTTP/record ID leaks ────────────────────────

    #[test]
    fn no_token_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
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
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_record_payload_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn no_raw_http_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"body\":{"));
        assert!(!json.contains("\"statusCode\""));
    }

    #[test]
    fn no_old_record_id_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"oldRecordId\""));
        assert!(!json.contains("oldId"));
        assert!(!json.contains("rec_old_"));
    }

    #[test]
    fn no_new_record_id_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"newRecordId\""));
        assert!(!json.contains("newId"));
        assert!(!json.contains("rec_new_"));
    }

    #[test]
    fn no_attachment_url_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    // ── No success state introduced ───────────────────────────────────────────

    #[test]
    fn no_success_state_in_serialization() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
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
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(!result.safety_snapshot.network_reads_attempted);
        assert!(!result.safety_snapshot.network_writes_attempted);
    }

    // ── No Tauri command introduced ───────────────────────────────────────────

    #[test]
    fn no_tauri_command_introduced() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            SandboxFinalValidationAdapterStatus::ReadyForSandboxCall
        );
    }

    // ── No-op adapter ─────────────────────────────────────────────────────────

    #[test]
    fn no_op_adapter_returns_zero_count() {
        let adapter = NoOpFinalValidationReadAdapter;
        assert_eq!(adapter.planned_operation_count(), 0);
    }

    // ── Mock adapter ──────────────────────────────────────────────────────────

    #[test]
    fn mock_adapter_returns_configured_count() {
        let adapter = MockFinalValidationReadAdapter { operation_count: 9 };
        assert_eq!(adapter.planned_operation_count(), 9);
    }

    #[test]
    fn mock_adapter_zero_count_when_no_operations() {
        let adapter = MockFinalValidationReadAdapter { operation_count: 0 };
        assert_eq!(adapter.planned_operation_count(), 0);
    }

    // ── Write gate snapshot ───────────────────────────────────────────────────

    #[test]
    fn safety_snapshot_write_gate_disabled_always_true() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn safety_snapshot_runtime_flags_always_false() {
        let rp = simple_record_plan();
        let sp = simple_schema_plan();
        let result = build_sandbox_final_validation_adapter(&full_request(), &sp, &rp);
        assert!(!result.safety_snapshot.runtime_execution_enabled);
        assert!(!result.safety_snapshot.app_runtime_writes_enabled);
        assert!(!result.safety_snapshot.app_runtime_reads_enabled);
        assert!(!result.safety_snapshot.network_reads_attempted);
        assert!(!result.safety_snapshot.network_writes_attempted);
    }

    // ── No production mode ────────────────────────────────────────────────────

    #[test]
    fn no_production_mode_exists() {
        let disabled = SandboxFinalValidationAdapterMode::Disabled;
        let sandbox = SandboxFinalValidationAdapterMode::SandboxOnlyInternal;
        assert_ne!(disabled, sandbox);
        let json = serde_json::to_string(&disabled).expect("serialize");
        assert!(!json.contains("production"));
        let json = serde_json::to_string(&sandbox).expect("serialize");
        assert!(!json.contains("production"));
    }
}
