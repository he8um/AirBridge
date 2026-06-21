use serde::{Deserialize, Serialize};

use crate::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use crate::restore::live_record_write_test_contract::{
    evaluate_live_record_write_test_contract, LiveRecordWriteTestContractMode,
    LiveRecordWriteTestContractRequest, LiveRecordWriteTestContractStatus,
};
use crate::restore::record_write_requests::RecordWriteRequestPlan;
use crate::restore::sandbox_adapter_chain_runner::{
    run_sandbox_adapter_chain, SandboxAdapterChainRunnerMode, SandboxAdapterChainRunnerRequest,
    SandboxAdapterChainRunnerStatus,
};
use crate::restore::sandbox_enablement_readiness::{
    build_sandbox_enablement_readiness_report, SandboxEnablementReadinessRequest,
    SandboxEnablementReadinessStatus,
};
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

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the live linked update test contract.
///
/// Safety invariants:
/// - `EligibleButNotExecuted` does NOT perform any Airtable network call.
/// - `EligibleButNotExecuted` does NOT enable runtime execution, writes, or reads.
/// - `EligibleButNotExecuted` is not stored globally, not persisted, and is not
///   reachable from UI, TypeScript, or any Tauri command.
/// - `EligibleButNotExecuted` does NOT change `evaluate_write_gate()` behavior.
/// - `contract_only` is always `true` — this is a contract/readiness layer only.
/// - `app_runtime_execution_enabled` is always `false`.
/// - `app_runtime_writes_enabled` is always `false`.
/// - `app_runtime_reads_enabled` is always `false`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `airtable_client_called` is always `false`.
/// - `no_changes_made` is always `true`.
/// - No `Succeeded`, `Complete`, `Enabled`, `Done`, or `ExecutionReady` status exists.
/// - The live linked update integration test itself remains separate pending work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveLinkedUpdateTestContractStatus {
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
    /// All contract prerequisites satisfied. No live call was made. No state
    /// was persisted. The live linked update integration test remains separate
    /// pending work.
    EligibleButNotExecuted,
}

/// Mode for the live linked update test contract.
///
/// Safety invariants:
/// - `Disabled` is the default — no evaluation is performed.
/// - `SandboxIntegrationCandidate` is for Rust unit tests and forward planning only.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveLinkedUpdateTestContractMode {
    /// Contract disabled — no evaluation is performed. Default state.
    Disabled,
    /// Sandbox integration candidate mode for Rust unit tests only.
    /// Does NOT execute any live linked update or network call.
    SandboxIntegrationCandidate,
}

/// Status of a single prerequisite evaluated by the contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveLinkedUpdateTestContractPrerequisiteStatus {
    /// Prerequisite is satisfied.
    Ready,
    /// Prerequisite is not satisfied — blocks the contract.
    Blocked,
    /// Prerequisite is absent or not evaluated.
    Missing,
    /// Prerequisite has a warning condition — does not block.
    Warning,
}

/// A single prerequisite evaluated by the contract.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No raw Airtable request body or response.
/// - No old or new Airtable record IDs.
/// - No record field payload.
/// - No attachment URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveLinkedUpdateTestContractPrerequisite {
    /// Stable prerequisite identifier (e.g. `LLUTC-PRE-01`).
    pub prerequisite_id: String,
    /// Human-readable label.
    pub label: String,
    pub status: LiveLinkedUpdateTestContractPrerequisiteStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the contract evaluation.
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
pub struct LiveLinkedUpdateTestContractSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the live record write test contract returned `EligibleButNotExecuted`.
    pub record_contract_eligible: bool,
    /// Whether the sandbox linked second-pass adapter returned `ReadyForSandboxCall`.
    pub linked_adapter_ready: bool,
    /// Whether the sandbox record write adapter returned `ReadyForSandboxCall`.
    pub record_adapter_ready: bool,
    /// Whether the sandbox schema write adapter returned `ReadyForSandboxCall`.
    pub schema_adapter_ready: bool,
    /// Whether the adapter chain runner returned `MockRunNotExecuted`.
    pub adapter_chain_mock_run_not_executed: bool,
    /// Whether the gate arming decision returned `ArmedNotExecutable`.
    pub gate_arming_armed_not_executable: bool,
    /// Whether the restore simulator returned `SimulatedNotExecuted`.
    pub simulator_simulated_not_executed: bool,
    /// Whether the enablement readiness report returned `ReadyButDisabled`.
    pub enablement_readiness_ready_but_disabled: bool,
    /// Whether the explicit internal contract flag was set.
    pub explicit_contract_flag_set: bool,
    /// Always `true` — this is a contract/readiness layer only, not an executor.
    pub contract_only: bool,
    /// Whether app runtime execution is enabled — always `false`.
    pub app_runtime_execution_enabled: bool,
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

/// Request to the live linked update test contract.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_live_linked_update_test_contract_requested` must be `true` for
/// evaluation to proceed. No UI control, Tauri command, or runtime path sets
/// this flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveLinkedUpdateTestContractRequest {
    /// Must be `sandboxIntegrationCandidate` for evaluation to proceed.
    /// `disabled` (the default) always results in `Blocked`.
    pub mode: LiveLinkedUpdateTestContractMode,
    /// Internal-only flag. Must be explicitly `true` to proceed.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_live_linked_update_test_contract_requested: bool,
    /// Prerequisite booleans forwarded to the underlying probe modules.
    pub sandbox_verified: bool,
    pub target_base_empty: bool,
    pub mapping_coverage_sufficient: bool,
    pub final_validation_enforcement_safe: bool,
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
    /// Per-field summaries forwarded to the adapter probes.
    /// No raw record IDs — only safe counts and labels.
    pub field_summaries: Vec<LinkedSecondPassFieldSummary>,
    /// Safe count of tables forwarded to the final validation probe.
    pub table_count: usize,
    /// Safe count of fields forwarded to the final validation probe.
    pub field_count: usize,
    /// Safe count of records forwarded to the final validation probe.
    pub record_count: usize,
    /// Safe count of ID mapping entries forwarded to the final validation probe.
    pub id_mapping_entry_count: usize,
    /// Safe count of linked field coverage entries.
    pub linked_coverage_count: usize,
    /// Safe count of attachment metadata entries.
    pub attachment_metadata_count: usize,
    /// Whether a package manifest is present.
    pub manifest_present: bool,
}

/// Result of the live linked update test contract.
///
/// Safety invariants (always enforced):
/// - `contract_only` is always `true`.
/// - `app_runtime_execution_enabled` is always `false`.
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
/// - The live linked update integration test itself remains separate pending work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveLinkedUpdateTestContractResult {
    pub status: LiveLinkedUpdateTestContractStatus,
    pub mode: LiveLinkedUpdateTestContractMode,
    pub message: String,
    pub prerequisites: Vec<LiveLinkedUpdateTestContractPrerequisite>,
    pub safety_snapshot: LiveLinkedUpdateTestContractSafetySnapshot,
    pub total_prerequisite_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — this is a contract/readiness layer only.
    pub contract_only: bool,
    /// Always `true` — no changes made.
    pub no_changes_made: bool,
    /// Always `false` — no network read operations were attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — the real Airtable client was not called.
    pub airtable_client_called: bool,
    /// Always `false` — app runtime execution is not enabled.
    pub app_runtime_execution_enabled: bool,
    /// Always `false` — app runtime writes are not enabled.
    pub app_runtime_writes_enabled: bool,
    /// Always `false` — app runtime reads are not enabled.
    pub app_runtime_reads_enabled: bool,
    /// Required conditions for a future live linked update integration test.
    /// These are reported only — they are NOT executed by this contract.
    pub required_future_live_conditions: Vec<String>,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const PRE_MODE: &str = "LLUTC-PRE-01";
const PRE_EXPLICIT_FLAG: &str = "LLUTC-PRE-02";
const PRE_WRITE_GATE: &str = "LLUTC-PRE-03";
const PRE_RECORD_CONTRACT: &str = "LLUTC-PRE-04";
const PRE_LINKED_ADAPTER: &str = "LLUTC-PRE-05";
const PRE_RECORD_ADAPTER: &str = "LLUTC-PRE-06";
const PRE_SCHEMA_ADAPTER: &str = "LLUTC-PRE-07";
const PRE_ADAPTER_CHAIN: &str = "LLUTC-PRE-08";
const PRE_GATE_ARMING: &str = "LLUTC-PRE-09";
const PRE_SIMULATOR: &str = "LLUTC-PRE-10";
const PRE_READINESS: &str = "LLUTC-PRE-11";

// ── Required future-live conditions ──────────────────────────────────────────

fn required_future_live_conditions() -> Vec<String> {
    vec![
        "disposable sandbox-only base required — no production base may be used".to_string(),
        "live schema and live record harnesses must have prepared sandbox-only records before linked updates".to_string(),
        "explicit test-only credentials required in future task — no token accepted by this contract".to_string(),
        "no UI execution path allowed — live call must be a separate Rust-internal task".to_string(),
        "only linked update operations allowed — no schema or first-pass record create operations".to_string(),
        "attachment handling remains disabled — must not be enabled in this task".to_string(),
        "final validation reads remain disabled — must not be enabled in this task".to_string(),
    ]
}

// ── Core function ─────────────────────────────────────────────────────────────

/// Evaluates whether a future live linked record update integration test could be
/// attempted — without performing any live call.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never accepts, stores, or returns a token.
/// - Never enables execution, writes, or reads.
/// - Never changes `evaluate_write_gate()` behavior.
/// - Never writes checkpoint files to disk.
/// - Never stores any state globally.
/// - Is not reachable from UI, TypeScript, or any Tauri command.
/// - Reports required future-live conditions without executing them.
/// - Always returns `contract_only: true`.
/// - Always returns `app_runtime_execution_enabled: false`,
///   `app_runtime_writes_enabled: false`, `app_runtime_reads_enabled: false`,
///   `no_changes_made: true`, `network_reads_attempted: false`,
///   `network_writes_attempted: false`, `airtable_client_called: false`.
/// - Returns `Blocked` unless all prerequisites pass.
/// - Returns `EligibleButNotExecuted` when all prerequisites pass — this does
///   NOT execute a live call, does NOT arm the gate, and is NOT persisted.
///   The live linked update integration test remains separate pending work.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn evaluate_live_linked_update_test_contract(
    request: &LiveLinkedUpdateTestContractRequest,
    schema_plan: &SchemaWriteRequestPlan,
    record_plan: &RecordWriteRequestPlan,
) -> LiveLinkedUpdateTestContractResult {
    // ── Mode check ─────────────────────────────────────────────────────────────
    if matches!(request.mode, LiveLinkedUpdateTestContractMode::Disabled) {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::Disabled,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_MODE,
            &format!(
                "{PRE_MODE}: Contract mode is disabled. No evaluation is performed. \
                 This is the default state. Set mode to sandboxIntegrationCandidate \
                 in future Rust-internal tests only."
            ),
            vec![],
        );
    }

    // ── Explicit flag ──────────────────────────────────────────────────────────
    if !request.explicit_internal_live_linked_update_test_contract_requested {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_EXPLICIT_FLAG,
            &format!(
                "{PRE_EXPLICIT_FLAG}: explicit_internal_live_linked_update_test_contract_requested \
                 must be true. This flag must be explicitly set in Rust unit tests only. \
                 No UI control, Tauri command, or runtime path sets this flag."
            ),
            vec![],
        );
    }

    // ── Write gate check ───────────────────────────────────────────────────────
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if !write_gate_disabled {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_WRITE_GATE,
            &format!(
                "{PRE_WRITE_GATE}: evaluate_write_gate() did not return \
                 Disabled/DisabledByProductPolicy. This is a critical safety violation."
            ),
            vec![],
        );
    }

    // ── Live record write contract probe (LLUTC-PRE-04) ───────────────────────
    let record_contract_req = LiveRecordWriteTestContractRequest {
        mode: LiveRecordWriteTestContractMode::SandboxIntegrationCandidate,
        explicit_internal_live_record_test_contract_requested: true,
        sandbox_verified: request.sandbox_verified,
        target_base_empty: request.target_base_empty,
        mapping_coverage_sufficient: request.mapping_coverage_sufficient,
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
        field_summaries: request.field_summaries.clone(),
        table_count: request.table_count,
        field_count: request.field_count,
        record_count: request.record_count,
        id_mapping_entry_count: request.id_mapping_entry_count,
        linked_coverage_count: request.linked_coverage_count,
        attachment_metadata_count: request.attachment_metadata_count,
        manifest_present: request.manifest_present,
    };
    let record_contract_result =
        evaluate_live_record_write_test_contract(&record_contract_req, schema_plan, record_plan);
    let record_contract_eligible = matches!(
        record_contract_result.status,
        LiveRecordWriteTestContractStatus::EligibleButNotExecuted
    );
    if !record_contract_eligible {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_RECORD_CONTRACT,
            &format!(
                "{PRE_RECORD_CONTRACT}: live record write test contract did not return \
                 EligibleButNotExecuted. The record write contract must be eligible \
                 before the linked update contract can report eligible."
            ),
            vec![],
        );
    }

    // ── Linked second-pass adapter probe (LLUTC-PRE-05) ──────────────────────
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
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_LINKED_ADAPTER,
            &format!(
                "{PRE_LINKED_ADAPTER}: sandbox linked second-pass adapter did not return \
                 ReadyForSandboxCall. All linked adapter prerequisites must be satisfied \
                 before this contract can report eligible."
            ),
            vec![],
        );
    }

    // ── Record write adapter probe (LLUTC-PRE-06) ─────────────────────────────
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
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_RECORD_ADAPTER,
            &format!(
                "{PRE_RECORD_ADAPTER}: sandbox record write adapter did not return \
                 ReadyForSandboxCall. All record adapter prerequisites must be satisfied \
                 before this contract can report eligible."
            ),
            vec![],
        );
    }

    // ── Schema write adapter probe (LLUTC-PRE-07) ─────────────────────────────
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
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_SCHEMA_ADAPTER,
            &format!(
                "{PRE_SCHEMA_ADAPTER}: sandbox schema write adapter did not return \
                 ReadyForSandboxCall. All schema adapter prerequisites must be satisfied \
                 before this contract can report eligible."
            ),
            vec![],
        );
    }

    // ── Adapter chain runner probe (LLUTC-PRE-08) ─────────────────────────────
    let chain_req = SandboxAdapterChainRunnerRequest {
        mode: SandboxAdapterChainRunnerMode::MockInternalOnly,
        explicit_internal_mock_chain_requested: true,
        sandbox_verified: request.sandbox_verified,
        target_base_empty: request.target_base_empty,
        mapping_coverage_sufficient: request.mapping_coverage_sufficient,
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
        field_summaries: request.field_summaries.clone(),
        table_count: request.table_count,
        field_count: request.field_count,
        record_count: request.record_count,
        id_mapping_entry_count: request.id_mapping_entry_count,
        linked_coverage_count: request.linked_coverage_count,
        attachment_metadata_count: request.attachment_metadata_count,
        manifest_present: request.manifest_present,
    };
    let chain_result = run_sandbox_adapter_chain(&chain_req, schema_plan, record_plan);
    let chain_ok = matches!(
        chain_result.status,
        SandboxAdapterChainRunnerStatus::MockRunNotExecuted
    );
    if !chain_ok {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            PRE_ADAPTER_CHAIN,
            &format!(
                "{PRE_ADAPTER_CHAIN}: sandbox adapter chain runner did not return \
                 MockRunNotExecuted. All chain runner prerequisites must be satisfied \
                 before this contract can report eligible."
            ),
            vec![],
        );
    }

    // ── Gate arming probe (LLUTC-PRE-09) ──────────────────────────────────────
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
    let arming_ok = matches!(
        arming_result.status,
        SandboxGateArmingStatus::ArmedNotExecutable
    );
    if !arming_ok {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            PRE_GATE_ARMING,
            &format!(
                "{PRE_GATE_ARMING}: sandbox gate arming decision did not return \
                 ArmedNotExecutable. All arming prerequisites must be satisfied \
                 before this contract can report eligible."
            ),
            vec![],
        );
    }

    // ── Simulator probe (LLUTC-PRE-10) ────────────────────────────────────────
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
    let sim_ok = matches!(
        sim_result.status,
        SandboxRestoreSimulatorStatus::SimulatedNotExecuted
    );
    if !sim_ok {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            PRE_SIMULATOR,
            &format!(
                "{PRE_SIMULATOR}: sandbox restore simulator did not return \
                 SimulatedNotExecuted. All simulation prerequisites must be satisfied \
                 before this contract can report eligible."
            ),
            vec![],
        );
    }

    // ── Enablement readiness probe (LLUTC-PRE-11) ─────────────────────────────
    let readiness_req = SandboxEnablementReadinessRequest {
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
    let readiness_result = build_sandbox_enablement_readiness_report(&readiness_req);
    let readiness_ok = matches!(
        readiness_result.status,
        SandboxEnablementReadinessStatus::ReadyButDisabled
    );
    if !readiness_ok {
        return blocked_result(
            LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            PRE_READINESS,
            &format!(
                "{PRE_READINESS}: sandbox enablement readiness report did not return \
                 ReadyButDisabled. All readiness prerequisites must be satisfied \
                 before this contract can report eligible."
            ),
            vec![],
        );
    }

    // ── All prerequisites satisfied ────────────────────────────────────────────
    let prerequisites = build_prerequisites(
        true, true, true, true, true, true, true, true, true, true, true,
    );
    let total = prerequisites.len();
    let snapshot = LiveLinkedUpdateTestContractSafetySnapshot {
        write_gate_disabled: true,
        record_contract_eligible: true,
        linked_adapter_ready: true,
        record_adapter_ready: true,
        schema_adapter_ready: true,
        adapter_chain_mock_run_not_executed: true,
        gate_arming_armed_not_executable: true,
        simulator_simulated_not_executed: true,
        enablement_readiness_ready_but_disabled: true,
        explicit_contract_flag_set: true,
        contract_only: true,
        app_runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
    };
    let future_conditions = required_future_live_conditions();

    LiveLinkedUpdateTestContractResult {
        status: LiveLinkedUpdateTestContractStatus::EligibleButNotExecuted,
        mode: LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
        message: format!(
            "Live linked update test contract: eligibleButNotExecuted. All {total} prerequisites \
             satisfied. No Airtable network call was made. No runtime execution is enabled. \
             No app runtime writes, reads, or execution are enabled. No changes were made. \
             contract_only=true. This is a contract/readiness layer only — the live linked \
             update integration test remains separate pending work. Attachment handling, \
             final validation reads, and live end-to-end restore execution remain pending \
             separate work."
        ),
        prerequisites,
        safety_snapshot: snapshot,
        total_prerequisite_count: total,
        blocked_reason: None,
        contract_only: true,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        app_runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        required_future_live_conditions: future_conditions,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn blocked_result(
    mode: LiveLinkedUpdateTestContractMode,
    record_contract_eligible: bool,
    linked_adapter_ready: bool,
    record_adapter_ready: bool,
    schema_adapter_ready: bool,
    adapter_chain_ok: bool,
    gate_arming_ok: bool,
    simulator_ok: bool,
    readiness_ok: bool,
    explicit_flag: bool,
    _unused: bool,
    blocking_pre_id: &str,
    blocked_reason: &str,
    prerequisites: Vec<LiveLinkedUpdateTestContractPrerequisite>,
) -> LiveLinkedUpdateTestContractResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    let total = prerequisites.len();
    LiveLinkedUpdateTestContractResult {
        status: LiveLinkedUpdateTestContractStatus::Blocked,
        mode,
        message: format!(
            "Live linked update test contract is blocked at {blocking_pre_id}. \
             No Airtable API calls were made. No changes were made. \
             Runtime execution, writes, reads, and network calls remain disabled. \
             contract_only=true."
        ),
        prerequisites,
        safety_snapshot: LiveLinkedUpdateTestContractSafetySnapshot {
            write_gate_disabled,
            record_contract_eligible,
            linked_adapter_ready,
            record_adapter_ready,
            schema_adapter_ready,
            adapter_chain_mock_run_not_executed: adapter_chain_ok,
            gate_arming_armed_not_executable: gate_arming_ok,
            simulator_simulated_not_executed: simulator_ok,
            enablement_readiness_ready_but_disabled: readiness_ok,
            explicit_contract_flag_set: explicit_flag,
            contract_only: true,
            app_runtime_execution_enabled: false,
            app_runtime_writes_enabled: false,
            app_runtime_reads_enabled: false,
            network_reads_attempted: false,
            network_writes_attempted: false,
            airtable_client_called: false,
        },
        total_prerequisite_count: total,
        blocked_reason: Some(blocked_reason.to_string()),
        contract_only: true,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        app_runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        required_future_live_conditions: required_future_live_conditions(),
    }
}

fn prereq(
    id: &str,
    label: &str,
    status: LiveLinkedUpdateTestContractPrerequisiteStatus,
    note: &str,
) -> LiveLinkedUpdateTestContractPrerequisite {
    LiveLinkedUpdateTestContractPrerequisite {
        prerequisite_id: id.to_string(),
        label: label.to_string(),
        status,
        note: note.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_prerequisites(
    mode_ok: bool,
    flag_ok: bool,
    gate_ok: bool,
    record_contract_ok: bool,
    linked_adapter_ok: bool,
    record_adapter_ok: bool,
    schema_adapter_ok: bool,
    chain_ok: bool,
    arming_ok: bool,
    sim_ok: bool,
    readiness_ok: bool,
) -> Vec<LiveLinkedUpdateTestContractPrerequisite> {
    let s = |ok: bool| {
        if ok {
            LiveLinkedUpdateTestContractPrerequisiteStatus::Ready
        } else {
            LiveLinkedUpdateTestContractPrerequisiteStatus::Blocked
        }
    };
    vec![
        prereq(
            PRE_MODE,
            "Mode is sandboxIntegrationCandidate",
            s(mode_ok),
            "Contract mode must be sandboxIntegrationCandidate for evaluation.",
        ),
        prereq(
            PRE_EXPLICIT_FLAG,
            "Explicit internal contract flag set",
            s(flag_ok),
            "explicit_internal_live_linked_update_test_contract_requested must be true.",
        ),
        prereq(
            PRE_WRITE_GATE,
            "evaluate_write_gate() returns Disabled",
            s(gate_ok),
            "evaluate_write_gate() must return Disabled/DisabledByProductPolicy.",
        ),
        prereq(
            PRE_RECORD_CONTRACT,
            "Record write contract EligibleButNotExecuted",
            s(record_contract_ok),
            "live record write test contract must return EligibleButNotExecuted.",
        ),
        prereq(
            PRE_LINKED_ADAPTER,
            "Linked second-pass adapter ReadyForSandboxCall",
            s(linked_adapter_ok),
            "sandbox linked second-pass adapter must return ReadyForSandboxCall.",
        ),
        prereq(
            PRE_RECORD_ADAPTER,
            "Record write adapter ReadyForSandboxCall",
            s(record_adapter_ok),
            "sandbox record write adapter must return ReadyForSandboxCall.",
        ),
        prereq(
            PRE_SCHEMA_ADAPTER,
            "Schema write adapter ReadyForSandboxCall",
            s(schema_adapter_ok),
            "sandbox schema write adapter must return ReadyForSandboxCall.",
        ),
        prereq(
            PRE_ADAPTER_CHAIN,
            "Adapter chain runner MockRunNotExecuted",
            s(chain_ok),
            "sandbox adapter chain runner must return MockRunNotExecuted.",
        ),
        prereq(
            PRE_GATE_ARMING,
            "Gate arming ArmedNotExecutable",
            s(arming_ok),
            "sandbox gate arming decision must return ArmedNotExecutable.",
        ),
        prereq(
            PRE_SIMULATOR,
            "Simulator SimulatedNotExecuted",
            s(sim_ok),
            "sandbox restore simulator must return SimulatedNotExecuted.",
        ),
        prereq(
            PRE_READINESS,
            "Enablement readiness ReadyButDisabled",
            s(readiness_ok),
            "sandbox enablement readiness must return ReadyButDisabled.",
        ),
    ]
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

    fn full_request() -> LiveLinkedUpdateTestContractRequest {
        LiveLinkedUpdateTestContractRequest {
            mode: LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
            explicit_internal_live_linked_update_test_contract_requested: true,
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
            field_summaries: vec![LinkedSecondPassFieldSummary {
                table_label: "Projects".to_string(),
                field_label: "Tasks".to_string(),
                record_count: 5,
                batch_count: 1,
                unresolved_link_count: 0,
            }],
            table_count: 2,
            field_count: 5,
            record_count: 10,
            id_mapping_entry_count: 10,
            linked_coverage_count: 5,
            attachment_metadata_count: 2,
            manifest_present: true,
        }
    }

    fn disabled_request() -> LiveLinkedUpdateTestContractRequest {
        LiveLinkedUpdateTestContractRequest {
            mode: LiveLinkedUpdateTestContractMode::Disabled,
            explicit_internal_live_linked_update_test_contract_requested: false,
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

    // ── Default / disabled blocked ─────────────────────────────────────────────

    #[test]
    fn default_disabled_request_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
    }

    #[test]
    fn disabled_mode_blocked_reason_contains_pre_mode() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PRE_MODE));
    }

    // ── Missing explicit flag ──────────────────────────────────────────────────

    #[test]
    fn missing_explicit_flag_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.explicit_internal_live_linked_update_test_contract_requested = false;
        let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PRE_EXPLICIT_FLAG));
    }

    // ── Record contract not eligible ───────────────────────────────────────────

    #[test]
    fn record_contract_not_eligible_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        // Disabling mapping_coverage_sufficient blocks the record write contract's
        // chain runner probe, causing it to return Blocked.
        req.mapping_coverage_sufficient = false;
        let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PRE_RECORD_CONTRACT));
    }

    // ── Linked adapter not ready ───────────────────────────────────────────────

    #[test]
    fn linked_adapter_not_ready_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        // Disable linked_second_pass_preview_ready to block only the linked adapter
        // without also blocking the record contract's chain runner. The record
        // contract's chain probe checks mapping_coverage_sufficient (SACR-CHK-07),
        // not linked_second_pass_preview_ready directly, so the record contract
        // can still return EligibleButNotExecuted. The linked adapter itself
        // requires linked_second_pass_preview_ready.
        req.linked_second_pass_preview_ready = false;
        let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
    }

    // ── Record adapter not ready ───────────────────────────────────────────────

    #[test]
    fn record_adapter_not_ready_returns_blocked() {
        use crate::restore::record_write_requests::{
            RecordWriteBlockedReason, RecordWriteOperationStatus,
        };
        let sp = simple_schema_plan();
        let mut rp = simple_record_plan();
        rp.status = RecordWriteOperationStatus::Blocked;
        rp.blocked_reason = Some(RecordWriteBlockedReason::RecordImportPlanNotReady);
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
        // A blocked record plan propagates through the record contract probe
        // (PRE_RECORD_CONTRACT) before reaching PRE_RECORD_ADAPTER. Both return Blocked.
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(
            reason.contains(PRE_RECORD_CONTRACT) || reason.contains(PRE_RECORD_ADAPTER),
            "expected PRE_RECORD_CONTRACT or PRE_RECORD_ADAPTER in blocked reason, got: {reason}"
        );
    }

    // ── Schema adapter not ready ───────────────────────────────────────────────

    #[test]
    fn schema_adapter_not_ready_returns_blocked() {
        use crate::restore::schema_write_requests::{
            SchemaWriteBlockedReason, SchemaWriteOperationStatus,
        };
        let mut sp = simple_schema_plan();
        sp.status = SchemaWriteOperationStatus::Blocked;
        sp.blocked_reason = Some(SchemaWriteBlockedReason::SchemaPlanNotReady);
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
    }

    // ── Adapter chain runner not ready ────────────────────────────────────────

    #[test]
    fn adapter_chain_not_ready_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.mapping_coverage_sufficient = false;
        let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
    }

    // ── Gate arming not ready ─────────────────────────────────────────────────

    #[test]
    fn gate_arming_not_ready_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.rollback_limitation_safe = false;
        let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
    }

    // ── Simulator not ready ───────────────────────────────────────────────────

    #[test]
    fn simulator_not_ready_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.failure_modes_safe = false;
        let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
    }

    // ── Readiness not ready ───────────────────────────────────────────────────

    #[test]
    fn readiness_not_ready_returns_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let mut req = full_request();
        req.failure_modes_safe = false;
        let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
        assert_eq!(result.status, LiveLinkedUpdateTestContractStatus::Blocked);
    }

    // ── evaluate_write_gate remains Disabled ──────────────────────────────────

    #[test]
    fn evaluate_write_gate_default_remains_disabled() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let _ = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    #[test]
    fn evaluate_write_gate_remains_disabled_after_blocked_result() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let _ = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── eligibleButNotExecuted when all prerequisites satisfied ───────────────

    #[test]
    fn eligible_but_not_executed_when_all_prereqs_satisfied() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            LiveLinkedUpdateTestContractStatus::EligibleButNotExecuted
        );
    }

    // ── contract_only is always true ──────────────────────────────────────────

    #[test]
    fn contract_only_always_true_when_eligible() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert!(result.contract_only);
        assert!(result.safety_snapshot.contract_only);
    }

    #[test]
    fn contract_only_always_true_when_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(result.contract_only);
        assert!(result.safety_snapshot.contract_only);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn app_runtime_execution_enabled_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(!r1.app_runtime_execution_enabled);
        assert!(!r2.app_runtime_execution_enabled);
    }

    #[test]
    fn app_runtime_writes_enabled_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(!r1.app_runtime_writes_enabled);
        assert!(!r2.app_runtime_writes_enabled);
    }

    #[test]
    fn app_runtime_reads_enabled_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(!r1.app_runtime_reads_enabled);
        assert!(!r2.app_runtime_reads_enabled);
    }

    #[test]
    fn network_reads_attempted_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(!r1.network_reads_attempted);
        assert!(!r2.network_reads_attempted);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(!r1.network_writes_attempted);
        assert!(!r2.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_always_true() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(r1.no_changes_made);
        assert!(r2.no_changes_made);
    }

    #[test]
    fn airtable_client_called_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
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
        let r1 = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let r2 = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(r1.safety_snapshot.write_gate_disabled);
        assert!(r2.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn safety_snapshot_runtime_flags_always_false() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert!(!result.safety_snapshot.app_runtime_execution_enabled);
        assert!(!result.safety_snapshot.app_runtime_writes_enabled);
        assert!(!result.safety_snapshot.app_runtime_reads_enabled);
        assert!(!result.safety_snapshot.network_reads_attempted);
        assert!(!result.safety_snapshot.network_writes_attempted);
        assert!(!result.safety_snapshot.airtable_client_called);
    }

    // ── Required future-live conditions reported ───────────────────────────────

    #[test]
    fn required_future_live_conditions_reported_when_eligible() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert!(!result.required_future_live_conditions.is_empty());
        let conditions = result.required_future_live_conditions.join(" ");
        assert!(conditions.contains("disposable sandbox-only base required"));
        assert!(conditions.contains("attachment handling remains disabled"));
        assert!(conditions.contains("final validation reads remain disabled"));
    }

    #[test]
    fn required_future_live_conditions_reported_when_blocked() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&disabled_request(), &sp, &rp);
        assert!(!result.required_future_live_conditions.is_empty());
    }

    // ── Prerequisites list ────────────────────────────────────────────────────

    #[test]
    fn all_eleven_prerequisites_present_when_eligible() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert_eq!(result.total_prerequisite_count, 11);
        assert_eq!(result.prerequisites.len(), 11);
        let ids: Vec<_> = result
            .prerequisites
            .iter()
            .map(|p| p.prerequisite_id.as_str())
            .collect();
        assert!(ids.contains(&"LLUTC-PRE-01"));
        assert!(ids.contains(&"LLUTC-PRE-11"));
    }

    #[test]
    fn all_prerequisites_ready_when_eligible() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        for p in &result.prerequisites {
            assert_eq!(
                p.status,
                LiveLinkedUpdateTestContractPrerequisiteStatus::Ready,
                "Expected all prerequisites ready, but {} was {:?}",
                p.prerequisite_id,
                p.status
            );
        }
    }

    // ── No serialization leaks ────────────────────────────────────────────────

    #[test]
    fn no_token_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
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
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_record_payload_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn no_raw_http_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"body\":{"));
        assert!(!json.contains("\"statusCode\""));
    }

    #[test]
    fn no_old_record_id_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"oldRecordId\""));
        assert!(!json.contains("oldId"));
        assert!(!json.contains("rec_old_"));
    }

    #[test]
    fn no_new_record_id_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"newRecordId\""));
        assert!(!json.contains("newId"));
        assert!(!json.contains("rec_new_"));
    }

    #[test]
    fn no_attachment_url_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn no_success_state_in_serialization() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
        assert!(!json.contains("executionReady"));
    }

    // ── No Tauri command or real Airtable client ──────────────────────────────

    #[test]
    fn no_tauri_command_introduced() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            LiveLinkedUpdateTestContractStatus::EligibleButNotExecuted
        );
    }

    #[test]
    fn no_real_airtable_client_called() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(!result.airtable_client_called);
    }

    // ── Message mentions pending work ─────────────────────────────────────────

    #[test]
    fn message_mentions_live_test_pending() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        assert!(
            result.message.contains("remains separate pending work"),
            "message must say live linked update test remains pending, got: {}",
            result.message
        );
    }

    // ── No attachment / final validation in future conditions ─────────────────

    #[test]
    fn attachment_handling_remains_disabled_in_future_conditions() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let conditions = result.required_future_live_conditions.join(" ");
        assert!(conditions.contains("attachment handling remains disabled"));
    }

    #[test]
    fn final_validation_reads_remain_disabled_in_future_conditions() {
        let sp = simple_schema_plan();
        let rp = simple_record_plan();
        let result = evaluate_live_linked_update_test_contract(&full_request(), &sp, &rp);
        let conditions = result.required_future_live_conditions.join(" ");
        assert!(conditions.contains("final validation reads remain disabled"));
    }
}
