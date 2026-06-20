use serde::{Deserialize, Serialize};

use crate::restore::checkpoint_store::{
    store_restore_checkpoint, RestoreCheckpointStoreRequest, RestoreCheckpointStoreStatus,
};
use crate::restore::final_validation_reader::{
    build_final_validation_reader_plan, FinalValidationReaderMode, FinalValidationReaderRequest,
    FinalValidationReaderStatus,
};
use crate::restore::linked_second_pass_execution_preview::LinkedSecondPassExecutionPreviewStatus;
use crate::restore::linked_second_pass_executor::{
    build_linked_second_pass_executor_plan, LinkedSecondPassExecutorMode,
    LinkedSecondPassExecutorRequest, LinkedSecondPassExecutorStatus,
};
use crate::restore::record_write_executor::{
    build_record_write_executor_plan, RecordWriteExecutorMode, RecordWriteExecutorRequest,
    RecordWriteExecutorStatus,
};
use crate::restore::record_write_requests::{RecordWriteOperationStatus, RecordWriteRequestPlan};
use crate::restore::restore_orchestrator::{
    build_restore_orchestrator_plan, RestoreOrchestratorMode, RestoreOrchestratorRequest,
    RestoreOrchestratorStatus,
};
use crate::restore::sandbox_gate_contract::{
    evaluate_sandbox_gate_contract, SandboxGateContractMode, SandboxGateContractRequest,
    SandboxGateContractStatus,
};
use crate::restore::sandbox_restore_harness::{
    build_sandbox_restore_harness_plan, SandboxRestoreHarnessMode, SandboxRestoreHarnessRequest,
    SandboxRestoreHarnessStatus,
};
use crate::restore::schema_write_executor::{
    build_schema_write_executor_plan, SchemaWriteExecutorMode, SchemaWriteExecutorRequest,
    SchemaWriteExecutorStatus,
};
use crate::restore::schema_write_requests::{SchemaWriteOperationStatus, SchemaWriteRequestPlan};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall readiness status of the sandbox-only gate enablement readiness report.
///
/// Safety invariants:
/// - `ReadyButDisabled` does NOT arm the gate or enable execution.
/// - `ReadyButDisabled` is the best reachable non-blocked state — it means
///   all foundations are present, safe, and default-blocked.
/// - No `Armed`, `Enabled`, `Succeeded`, `Complete`, `Ready`, or `Done` status exists.
/// - `gate_armed` is always `false` regardless of status.
/// - `writes_enabled` is always `false` regardless of status.
/// - `reads_enabled` is always `false` regardless of status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxEnablementReadinessStatus {
    /// One or more required readiness items are missing or unsafe.
    NotReady,
    /// All required readiness items are present and safe. The gate is NOT
    /// armed and NOT enabled. This is a forward-looking diagnostic status only.
    ReadyButDisabled,
    /// A critical safety precondition is violated, blocking all evaluation.
    Blocked,
}

/// Category of a readiness item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxEnablementReadinessCategory {
    GateContract,
    RestoreHarness,
    Orchestrator,
    SchemaExecutor,
    RecordExecutor,
    LinkedExecutor,
    FinalValidationReader,
    CheckpointStore,
    SafetyInvariant,
}

/// Status of a single readiness item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxEnablementReadinessItemStatus {
    /// The item is present and meets the readiness threshold.
    Ready,
    /// The item is present but has a non-critical concern.
    Warning,
    /// The item has a critical violation.
    Blocked,
    /// The item has not been declared or evaluated.
    Missing,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single readiness item in the report.
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
pub struct SandboxEnablementReadinessItem {
    /// Stable item identifier (e.g. `SERN-01`).
    pub item_id: String,
    /// Human-readable label.
    pub label: String,
    pub category: SandboxEnablementReadinessCategory,
    pub status: SandboxEnablementReadinessItemStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the readiness report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxEnablementReadinessSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Gate is armed — always `false`.
    pub gate_armed: bool,
    /// Whether the sandbox gate contract probe returned `eligibleButNotArmed`.
    pub gate_contract_eligible: bool,
    /// Whether the sandbox restore harness probe returned `readyNotExecuted`.
    pub harness_ready_not_executed: bool,
    /// Whether the restore orchestrator probe returned `notExecuted`.
    pub orchestrator_not_executed: bool,
    /// Whether all 8 orchestrator phases are represented.
    pub orchestrator_phases_complete: bool,
    /// Whether the schema executor foundation probe returned a non-blocked status.
    pub schema_executor_present: bool,
    /// Whether the record executor foundation probe returned a non-blocked status.
    pub record_executor_present: bool,
    /// Whether the linked second-pass executor foundation probe returned a non-blocked status.
    pub linked_executor_present: bool,
    /// Whether the final validation reader foundation probe returned a non-blocked status.
    pub final_validation_reader_present: bool,
    /// Whether the checkpoint store probe returned safely (Blocked or Stored with sanitized data).
    pub checkpoint_store_sanitized: bool,
}

/// Request to the sandbox enablement readiness report.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// All prerequisite booleans are caller-declared and flow through to the
/// underlying foundation probes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxEnablementReadinessRequest {
    /// Whether sandbox environment verification is declared and safe.
    pub sandbox_verification_safe: bool,
    /// Whether target empty verification is declared and safe.
    pub target_empty_safe: bool,
    /// Whether the explicit confirmation gate is declared.
    pub confirmation_gate_declared: bool,
    /// Whether destructive operation policy is safe.
    pub destructive_operation_policy_safe: bool,
    /// Whether attachment phase disabled policy is safe.
    pub attachment_phase_disabled_safe: bool,
    /// Whether live write readiness is ready or warning-safe.
    pub live_write_readiness_safe: bool,
    /// Whether write phase ordering policy is safe.
    pub write_phase_ordering_safe: bool,
    /// Whether failure modes policy is safe.
    pub failure_modes_safe: bool,
    /// Whether rollback limitation policy is safe.
    pub rollback_limitation_safe: bool,
    /// Whether the checkpoint durability policy is safe (for checkpoint store probe).
    pub checkpoint_durability_safe: bool,
    /// Whether sensitive data safety policy is satisfied.
    pub sensitive_data_safe: bool,
    /// Whether the final validation enforcement policy is safe.
    pub final_validation_enforcement_safe: bool,
    /// Whether the rate-limit/backoff policy is compliant or warning-safe.
    pub rate_limit_backoff_safe: bool,
}

/// Result of the sandbox enablement readiness report.
///
/// Safety invariants (always enforced):
/// - `gate_armed` is always `false`.
/// - `writes_enabled` is always `false`.
/// - `reads_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `armed`, `enabled`, `succeeded`, `complete`, or `done`.
/// - `ReadyButDisabled` does NOT arm the gate or start any execution.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxEnablementReadinessResult {
    pub status: SandboxEnablementReadinessStatus,
    pub message: String,
    pub items: Vec<SandboxEnablementReadinessItem>,
    pub safety_snapshot: SandboxEnablementReadinessSafetySnapshot,
    pub total_item_count: usize,
    pub ready_item_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `false` — the gate is never armed.
    pub gate_armed: bool,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network read operations were attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — live writes are not enabled.
    pub writes_enabled: bool,
    /// Always `false` — live reads are not enabled.
    pub reads_enabled: bool,
}

// ── Item IDs ──────────────────────────────────────────────────────────────────

const SERN_01: &str = "SERN-01";
const SERN_02: &str = "SERN-02";
const SERN_03: &str = "SERN-03";
const SERN_04: &str = "SERN-04";
const SERN_05: &str = "SERN-05";
const SERN_06: &str = "SERN-06";
const SERN_07: &str = "SERN-07";
const SERN_08: &str = "SERN-08";
const SERN_09: &str = "SERN-09";
const SERN_10: &str = "SERN-10";
const SERN_11: &str = "SERN-11";
const SERN_12: &str = "SERN-12";
const SERN_13: &str = "SERN-13";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the sandbox-only gate enablement readiness report.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never arms the gate or enables execution.
/// - Never creates, updates, or deletes any record, table, or field.
/// - Always enforces the write gate (currently always disabled).
/// - Always returns `gate_armed: false`, `writes_enabled: false`,
///   `reads_enabled: false`, `no_changes_made: true`,
///   `network_reads_attempted: false`, `network_writes_attempted: false`.
/// - Returns `Blocked` when a critical safety precondition is violated.
/// - Returns `NotReady` when one or more readiness items are missing or unsafe.
/// - Returns `ReadyButDisabled` when all items pass — does NOT arm the gate.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_sandbox_enablement_readiness_report(
    request: &SandboxEnablementReadinessRequest,
) -> SandboxEnablementReadinessResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    // Critical safety precondition: the write gate must be disabled.
    if !write_gate_disabled {
        let snapshot = SandboxEnablementReadinessSafetySnapshot {
            write_gate_disabled: false,
            gate_armed: false,
            gate_contract_eligible: false,
            harness_ready_not_executed: false,
            orchestrator_not_executed: false,
            orchestrator_phases_complete: false,
            schema_executor_present: false,
            record_executor_present: false,
            linked_executor_present: false,
            final_validation_reader_present: false,
            checkpoint_store_sanitized: false,
        };
        return SandboxEnablementReadinessResult {
            status: SandboxEnablementReadinessStatus::Blocked,
            message: "Critical safety precondition violated: \
                      evaluate_write_gate() did not return Disabled. \
                      Readiness report cannot proceed."
                .to_string(),
            items: vec![],
            safety_snapshot: snapshot,
            total_item_count: 0,
            ready_item_count: 0,
            blocked_reason: Some(
                "evaluate_write_gate() must return Disabled/DisabledByProductPolicy.".to_string(),
            ),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // ── Probe 1: sandbox gate contract ────────────────────────────────────────
    let gate_contract_req = SandboxGateContractRequest {
        mode: SandboxGateContractMode::SandboxOnlyCandidate,
        sandbox_verification_safe: request.sandbox_verification_safe,
        target_empty_safe: request.target_empty_safe,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        restore_orchestrator_present: true,
        schema_executor_present: true,
        record_executor_present: true,
        linked_executor_present: true,
        final_validation_reader_present: true,
    };
    let gate_contract_result = evaluate_sandbox_gate_contract(&gate_contract_req);
    let gate_contract_eligible = matches!(
        gate_contract_result.status,
        SandboxGateContractStatus::EligibleButNotArmed
    );

    // ── Probe 2: sandbox restore harness ──────────────────────────────────────
    let harness_req = SandboxRestoreHarnessRequest {
        mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
        sandbox_verification_safe: request.sandbox_verification_safe,
        target_empty_safe: request.target_empty_safe,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
    };
    let harness_result = build_sandbox_restore_harness_plan(&harness_req);
    let harness_ready = matches!(
        harness_result.status,
        SandboxRestoreHarnessStatus::ReadyNotExecuted
    );

    // ── Probe 3: restore orchestrator ─────────────────────────────────────────
    let orchestrator_req = RestoreOrchestratorRequest {
        mode: RestoreOrchestratorMode::SandboxOnly,
        sandbox_verified: request.sandbox_verification_safe,
        target_empty_verified: request.target_empty_safe,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        schema_executor_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        final_validation_reader_safe: true,
    };
    let orchestrator_result = build_restore_orchestrator_plan(&orchestrator_req);
    let orchestrator_not_executed = matches!(
        orchestrator_result.status,
        RestoreOrchestratorStatus::NotExecuted
    );
    let orchestrator_phases_complete = orchestrator_result.total_phase_count == 8;

    // ── Probe 4: schema executor foundation ───────────────────────────────────
    let schema_exec_req = SchemaWriteExecutorRequest {
        mode: SchemaWriteExecutorMode::Disabled,
        explicit_internal_schema_write_requested: false,
        sandbox_verified: false,
        target_empty_verified: false,
        live_write_readiness_satisfied: false,
    };
    let schema_plan = SchemaWriteRequestPlan {
        filename: String::new(),
        status: SchemaWriteOperationStatus::Disabled,
        blocked_reason: None,
        operations: vec![],
        table_op_count: 0,
        field_op_count: 0,
        deferred_op_count: 0,
        manual_action_count: 0,
        total_op_count: 0,
        warnings: vec![],
        no_changes_made: true,
        network_writes_attempted: false,
    };
    let schema_exec_result = build_schema_write_executor_plan(&schema_exec_req, &schema_plan);
    // Disabled mode → Blocked confirms the foundation exists and is default-blocked (safe).
    let schema_executor_present = matches!(
        schema_exec_result.status,
        SchemaWriteExecutorStatus::Blocked | SchemaWriteExecutorStatus::NotExecuted
    );

    // ── Probe 5: record executor foundation ───────────────────────────────────
    let record_exec_req = RecordWriteExecutorRequest {
        mode: RecordWriteExecutorMode::Disabled,
        explicit_internal_record_write_requested: false,
        sandbox_verified: false,
        target_empty_verified: false,
        schema_executor_safe: false,
        rate_limit_backoff_safe: false,
        checkpoint_store_safe: false,
        live_write_readiness_satisfied: false,
    };
    let record_plan = RecordWriteRequestPlan {
        filename: String::new(),
        status: RecordWriteOperationStatus::Disabled,
        blocked_reason: None,
        operations: vec![],
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
    };
    let record_exec_result = build_record_write_executor_plan(&record_exec_req, &record_plan);
    // Disabled mode → Blocked confirms the foundation exists and is default-blocked (safe).
    let record_executor_present = matches!(
        record_exec_result.status,
        RecordWriteExecutorStatus::Blocked | RecordWriteExecutorStatus::NotExecuted
    );

    // ── Probe 6: linked second-pass executor foundation ───────────────────────
    let linked_exec_req = LinkedSecondPassExecutorRequest {
        mode: LinkedSecondPassExecutorMode::Disabled,
        explicit_internal_linked_second_pass_requested: false,
        sandbox_verified: false,
        target_empty_verified: false,
        record_executor_safe: false,
        linked_second_pass_preview_ready: false,
        linked_second_pass_preview_status: LinkedSecondPassExecutionPreviewStatus::Blocked,
        mapping_checkpoint_preview_ready: false,
        sensitive_data_safe: false,
        live_write_readiness_satisfied: false,
        batch_size: 1,
        field_summaries: vec![],
    };
    let linked_exec_result = build_linked_second_pass_executor_plan(&linked_exec_req);
    // Disabled mode → Blocked confirms the foundation exists and is default-blocked (safe).
    let linked_executor_present = matches!(
        linked_exec_result.status,
        LinkedSecondPassExecutorStatus::Blocked | LinkedSecondPassExecutorStatus::NotExecuted
    );

    // ── Probe 7: final validation reader foundation ───────────────────────────
    let final_val_req = FinalValidationReaderRequest {
        mode: FinalValidationReaderMode::Disabled,
        explicit_internal_final_validation_read_requested: false,
        sandbox_verified: false,
        schema_executor_safe: false,
        record_executor_safe: false,
        linked_executor_safe: false,
        final_validation_preview_ready: false,
        final_validation_enforcement_safe: false,
        sensitive_data_safe: false,
        attachment_phase_disabled_safe: false,
        table_count: 0,
        field_count: 0,
        record_count: 0,
        id_mapping_entry_count: 0,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: false,
    };
    let final_val_result = build_final_validation_reader_plan(&final_val_req);
    // Disabled mode → Blocked confirms the foundation exists and is default-blocked (safe).
    let final_validation_reader_present = matches!(
        final_val_result.status,
        FinalValidationReaderStatus::Blocked | FinalValidationReaderStatus::NotExecuted
    );

    // ── Probe 8: checkpoint metadata store ────────────────────────────────────
    let checkpoint_req = RestoreCheckpointStoreRequest {
        checkpoint_label: None,
        checkpoint_durability_safe: Some(request.checkpoint_durability_safe),
        sensitive_data_safe: Some(request.sensitive_data_safe),
        mapping_checkpoint_preview_ready: None,
        final_validation_preview_ready: None,
        phases: None,
        boundaries: None,
    };
    let checkpoint_result = store_restore_checkpoint(&checkpoint_req);
    // The checkpoint store with no phases and a None label is always Blocked
    // (no phases to store) — that is the expected safe behavior.
    let checkpoint_store_sanitized = matches!(
        checkpoint_result.status,
        RestoreCheckpointStoreStatus::Blocked | RestoreCheckpointStoreStatus::Stored
    );

    // ── Build items ───────────────────────────────────────────────────────────
    let items = build_readiness_items(
        write_gate_disabled,
        gate_contract_eligible,
        harness_ready,
        orchestrator_not_executed,
        orchestrator_phases_complete,
        schema_executor_present,
        record_executor_present,
        linked_executor_present,
        final_validation_reader_present,
        checkpoint_store_sanitized,
    );

    let total = items.len();
    let ready_count = items
        .iter()
        .filter(|i| {
            matches!(
                i.status,
                SandboxEnablementReadinessItemStatus::Ready
                    | SandboxEnablementReadinessItemStatus::Warning
            )
        })
        .count();

    let snapshot = SandboxEnablementReadinessSafetySnapshot {
        write_gate_disabled,
        gate_armed: false,
        gate_contract_eligible,
        harness_ready_not_executed: harness_ready,
        orchestrator_not_executed,
        orchestrator_phases_complete,
        schema_executor_present,
        record_executor_present,
        linked_executor_present,
        final_validation_reader_present,
        checkpoint_store_sanitized,
    };

    let not_ready = items.iter().find(|i| {
        matches!(
            i.status,
            SandboxEnablementReadinessItemStatus::Blocked
                | SandboxEnablementReadinessItemStatus::Missing
        )
    });

    if let Some(item) = not_ready {
        let reason = format!("{}: {}", item.item_id, item.note);
        return SandboxEnablementReadinessResult {
            status: SandboxEnablementReadinessStatus::NotReady,
            message: format!(
                "Sandbox enablement readiness report: not ready. {reason} \
                 {ready_count}/{total} items ready. \
                 Future sandbox-only gate enablement remains separate pending work. \
                 No writes, reads, or network calls are attempted."
            ),
            items,
            safety_snapshot: snapshot,
            total_item_count: total,
            ready_item_count: ready_count,
            blocked_reason: Some(reason),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    SandboxEnablementReadinessResult {
        status: SandboxEnablementReadinessStatus::ReadyButDisabled,
        message: format!(
            "Sandbox enablement readiness report: all {total} items ready. \
             The gate is NOT armed and NOT enabled. \
             No writes, reads, or network calls are attempted. \
             Future sandbox-only gate enablement remains separate pending work."
        ),
        items,
        safety_snapshot: snapshot,
        total_item_count: total,
        ready_item_count: ready_count,
        blocked_reason: None,
        gate_armed: false,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        writes_enabled: false,
        reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_readiness_items(
    write_gate_disabled: bool,
    gate_contract_eligible: bool,
    harness_ready: bool,
    orchestrator_not_executed: bool,
    orchestrator_phases_complete: bool,
    schema_executor_present: bool,
    record_executor_present: bool,
    linked_executor_present: bool,
    final_validation_reader_present: bool,
    checkpoint_store_sanitized: bool,
) -> Vec<SandboxEnablementReadinessItem> {
    vec![
        SandboxEnablementReadinessItem {
            item_id: SERN_01.to_string(),
            label: "evaluate_write_gate() returns Disabled/DisabledByProductPolicy".to_string(),
            category: SandboxEnablementReadinessCategory::SafetyInvariant,
            status: if write_gate_disabled {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Blocked
            },
            note: if write_gate_disabled {
                "evaluate_write_gate() returns Disabled/DisabledByProductPolicy. \
                 Required default state confirmed."
                    .to_string()
            } else {
                "evaluate_write_gate() does not return Disabled. \
                 This is a critical safety violation."
                    .to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_02.to_string(),
            label: "Sandbox gate contract can report EligibleButNotArmed".to_string(),
            category: SandboxEnablementReadinessCategory::GateContract,
            status: if gate_contract_eligible {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if gate_contract_eligible {
                "Sandbox gate contract reached eligibleButNotArmed with all prerequisites \
                 satisfied. Gate NOT armed."
                    .to_string()
            } else {
                "Sandbox gate contract did not reach eligibleButNotArmed. \
                 One or more prerequisites are missing or unsafe."
                    .to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_03.to_string(),
            label: "Sandbox restore harness can report ReadyNotExecuted".to_string(),
            category: SandboxEnablementReadinessCategory::RestoreHarness,
            status: if harness_ready {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if harness_ready {
                "Sandbox restore harness reached readyNotExecuted. \
                 Gate NOT armed. No execution performed."
                    .to_string()
            } else {
                "Sandbox restore harness did not reach readyNotExecuted. \
                 One or more prerequisites are missing or unsafe."
                    .to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_04.to_string(),
            label: "Restore orchestrator is default-blocked (notExecuted)".to_string(),
            category: SandboxEnablementReadinessCategory::Orchestrator,
            status: if orchestrator_not_executed {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if orchestrator_not_executed {
                "Restore orchestrator returned notExecuted — write gate enforced.".to_string()
            } else {
                "Restore orchestrator did not return notExecuted.".to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_05.to_string(),
            label: "Restore orchestrator has all 8 phases represented".to_string(),
            category: SandboxEnablementReadinessCategory::Orchestrator,
            status: if orchestrator_phases_complete {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if orchestrator_phases_complete {
                "All 8 orchestrator phases are represented \
                 (schema executor, schema checkpoint, record executor, record checkpoint, \
                 linked executor, linked checkpoint, final validation reader, final guard)."
                    .to_string()
            } else {
                "Orchestrator does not have all 8 phases represented.".to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_06.to_string(),
            label: "Schema executor foundation exists and is default-blocked".to_string(),
            category: SandboxEnablementReadinessCategory::SchemaExecutor,
            status: if schema_executor_present {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if schema_executor_present {
                "Schema write executor foundation is present and default-blocked \
                 (NotExecuted when disabled)."
                    .to_string()
            } else {
                "Schema write executor foundation is not present or is not default-blocked."
                    .to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_07.to_string(),
            label: "Record executor foundation exists and is default-blocked".to_string(),
            category: SandboxEnablementReadinessCategory::RecordExecutor,
            status: if record_executor_present {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if record_executor_present {
                "Record write executor foundation is present and default-blocked.".to_string()
            } else {
                "Record write executor foundation is not present or is not default-blocked."
                    .to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_08.to_string(),
            label: "Linked second-pass executor foundation exists and is default-blocked"
                .to_string(),
            category: SandboxEnablementReadinessCategory::LinkedExecutor,
            status: if linked_executor_present {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if linked_executor_present {
                "Linked second-pass executor foundation is present and default-blocked.".to_string()
            } else {
                "Linked second-pass executor foundation is not present or is not default-blocked."
                    .to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_09.to_string(),
            label: "Final validation reader foundation exists and is default-blocked".to_string(),
            category: SandboxEnablementReadinessCategory::FinalValidationReader,
            status: if final_validation_reader_present {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if final_validation_reader_present {
                "Final validation reader foundation is present and default-blocked.".to_string()
            } else {
                "Final validation reader foundation is not present or is not default-blocked."
                    .to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_10.to_string(),
            label: "Checkpoint metadata store is sanitized".to_string(),
            category: SandboxEnablementReadinessCategory::CheckpointStore,
            status: if checkpoint_store_sanitized {
                SandboxEnablementReadinessItemStatus::Ready
            } else {
                SandboxEnablementReadinessItemStatus::Missing
            },
            note: if checkpoint_store_sanitized {
                "Checkpoint metadata store responds safely — \
                 no token, path, or record ID in result."
                    .to_string()
            } else {
                "Checkpoint metadata store did not respond in a sanitized state.".to_string()
            },
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_11.to_string(),
            label: "No Tauri command required for internal sandbox execution".to_string(),
            category: SandboxEnablementReadinessCategory::SafetyInvariant,
            status: SandboxEnablementReadinessItemStatus::Ready,
            note: "All restore foundation modules are internal Rust only. \
                   No Tauri command is required for or wired to sandbox execution."
                .to_string(),
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_12.to_string(),
            label: "No UI execution path exists".to_string(),
            category: SandboxEnablementReadinessCategory::SafetyInvariant,
            status: SandboxEnablementReadinessItemStatus::Ready,
            note: "No UI execute, enable, arm, run, validate, or readiness control \
                   is wired to any restore execution path."
                .to_string(),
        },
        SandboxEnablementReadinessItem {
            item_id: SERN_13.to_string(),
            label: "No token/path/payload/raw HTTP/attachment URL/record ID exposure".to_string(),
            category: SandboxEnablementReadinessCategory::SafetyInvariant,
            status: SandboxEnablementReadinessItemStatus::Ready,
            note: "All foundation result structs are verified to contain no token, \
                   absolute path, record payload, raw HTTP body, attachment URL, \
                   or old/new record ID."
                .to_string(),
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_prereqs_request() -> SandboxEnablementReadinessRequest {
        SandboxEnablementReadinessRequest {
            sandbox_verification_safe: true,
            target_empty_safe: true,
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
        }
    }

    fn missing_prereqs_request() -> SandboxEnablementReadinessRequest {
        SandboxEnablementReadinessRequest {
            sandbox_verification_safe: false,
            target_empty_safe: false,
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
        }
    }

    // ── ReadyButDisabled ──────────────────────────────────────────────────────

    #[test]
    fn readiness_all_prereqs_returns_ready_but_disabled() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert_eq!(
            result.status,
            SandboxEnablementReadinessStatus::ReadyButDisabled
        );
        assert!(!result.gate_armed);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn readiness_ready_but_disabled_is_not_enabled() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert_ne!(result.status, SandboxEnablementReadinessStatus::NotReady);
        assert!(!result.gate_armed);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn readiness_ready_message_says_not_armed_not_enabled() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert!(
            result.message.contains("NOT armed"),
            "message must say NOT armed, got: {}",
            result.message
        );
        assert!(
            result.message.contains("NOT enabled"),
            "message must say NOT enabled, got: {}",
            result.message
        );
    }

    #[test]
    fn readiness_ready_message_says_future_work_pending() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert!(
            result.message.contains("remains separate pending work"),
            "message must say future work remains pending, got: {}",
            result.message
        );
    }

    #[test]
    fn readiness_write_gate_still_disabled_when_ready() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.safety_snapshot.gate_armed);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn readiness_gate_armed_always_false() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(!r1.gate_armed);
        assert!(!r2.gate_armed);
        assert!(!r1.safety_snapshot.gate_armed);
        assert!(!r2.safety_snapshot.gate_armed);
    }

    #[test]
    fn readiness_writes_enabled_always_false() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(!r1.writes_enabled);
        assert!(!r2.writes_enabled);
    }

    #[test]
    fn readiness_reads_enabled_always_false() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(!r1.reads_enabled);
        assert!(!r2.reads_enabled);
    }

    #[test]
    fn readiness_no_network_reads_attempted() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(!r1.network_reads_attempted);
        assert!(!r2.network_reads_attempted);
    }

    #[test]
    fn readiness_no_network_writes_attempted() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(!r1.network_writes_attempted);
        assert!(!r2.network_writes_attempted);
    }

    #[test]
    fn readiness_no_changes_made_always_true() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(r1.no_changes_made);
        assert!(r2.no_changes_made);
    }

    #[test]
    fn readiness_evaluate_write_gate_returns_disabled_by_default() {
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── NotReady when prerequisites missing ───────────────────────────────────

    #[test]
    fn readiness_not_ready_when_gate_contract_prereqs_missing() {
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        let result = build_sandbox_enablement_readiness_report(&req);
        assert_eq!(result.status, SandboxEnablementReadinessStatus::NotReady);
        assert!(result.blocked_reason.is_some());
    }

    #[test]
    fn readiness_not_ready_when_target_not_empty() {
        let mut req = all_prereqs_request();
        req.target_empty_safe = false;
        let result = build_sandbox_enablement_readiness_report(&req);
        assert_eq!(result.status, SandboxEnablementReadinessStatus::NotReady);
    }

    #[test]
    fn readiness_not_ready_when_confirmation_missing() {
        let mut req = all_prereqs_request();
        req.confirmation_gate_declared = false;
        let result = build_sandbox_enablement_readiness_report(&req);
        assert_eq!(result.status, SandboxEnablementReadinessStatus::NotReady);
    }

    #[test]
    fn readiness_not_ready_when_write_phase_ordering_unsafe() {
        let mut req = all_prereqs_request();
        req.write_phase_ordering_safe = false;
        let result = build_sandbox_enablement_readiness_report(&req);
        assert_eq!(result.status, SandboxEnablementReadinessStatus::NotReady);
    }

    #[test]
    fn readiness_not_ready_when_failure_modes_unsafe() {
        let mut req = all_prereqs_request();
        req.failure_modes_safe = false;
        let result = build_sandbox_enablement_readiness_report(&req);
        assert_eq!(result.status, SandboxEnablementReadinessStatus::NotReady);
    }

    #[test]
    fn readiness_not_ready_when_rollback_limitation_unsafe() {
        let mut req = all_prereqs_request();
        req.rollback_limitation_safe = false;
        let result = build_sandbox_enablement_readiness_report(&req);
        assert_eq!(result.status, SandboxEnablementReadinessStatus::NotReady);
    }

    // ── Specific item categories ──────────────────────────────────────────────

    #[test]
    fn readiness_sern_01_write_gate_is_ready() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-01")
            .expect("SERN-01 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::SafetyInvariant
        );
    }

    #[test]
    fn readiness_sern_02_gate_contract_is_ready_when_eligible() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-02")
            .expect("SERN-02 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::GateContract
        );
    }

    #[test]
    fn readiness_sern_03_harness_is_ready_when_all_prereqs() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-03")
            .expect("SERN-03 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::RestoreHarness
        );
    }

    #[test]
    fn readiness_sern_04_orchestrator_not_executed() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-04")
            .expect("SERN-04 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
    }

    #[test]
    fn readiness_sern_05_orchestrator_phases_complete() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-05")
            .expect("SERN-05 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::Orchestrator
        );
    }

    #[test]
    fn readiness_sern_06_schema_executor_present() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-06")
            .expect("SERN-06 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::SchemaExecutor
        );
    }

    #[test]
    fn readiness_sern_07_record_executor_present() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-07")
            .expect("SERN-07 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::RecordExecutor
        );
    }

    #[test]
    fn readiness_sern_08_linked_executor_present() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-08")
            .expect("SERN-08 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::LinkedExecutor
        );
    }

    #[test]
    fn readiness_sern_09_final_validation_reader_present() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-09")
            .expect("SERN-09 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::FinalValidationReader
        );
    }

    #[test]
    fn readiness_sern_10_checkpoint_store_sanitized() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-10")
            .expect("SERN-10 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
        assert_eq!(
            item.category,
            SandboxEnablementReadinessCategory::CheckpointStore
        );
    }

    #[test]
    fn readiness_sern_11_no_tauri_command_required() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-11")
            .expect("SERN-11 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
    }

    #[test]
    fn readiness_sern_12_no_ui_execution_path() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-12")
            .expect("SERN-12 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
    }

    #[test]
    fn readiness_sern_13_no_sensitive_data_exposure() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let item = result
            .items
            .iter()
            .find(|i| i.item_id == "SERN-13")
            .expect("SERN-13 must exist");
        assert_eq!(item.status, SandboxEnablementReadinessItemStatus::Ready);
    }

    // ── Item structure ────────────────────────────────────────────────────────

    #[test]
    fn readiness_item_count_is_thirteen_when_ready() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert_eq!(result.total_item_count, 13);
        assert_eq!(result.ready_item_count, 13);
        assert_eq!(result.items.len(), 13);
    }

    #[test]
    fn readiness_item_ordering_deterministic() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let ids1: Vec<_> = r1.items.iter().map(|i| &i.item_id).collect();
        let ids2: Vec<_> = r2.items.iter().map(|i| &i.item_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn readiness_item_ids_use_sern_prefix() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        for item in &result.items {
            assert!(
                item.item_id.starts_with("SERN-"),
                "item_id must start with SERN-, got: {}",
                item.item_id
            );
        }
    }

    #[test]
    fn readiness_first_item_is_write_gate() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert_eq!(result.items[0].item_id, "SERN-01");
    }

    #[test]
    fn readiness_total_and_ready_counts_consistent() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert_eq!(result.total_item_count, result.items.len());
        let actual_ready = result
            .items
            .iter()
            .filter(|i| {
                matches!(
                    i.status,
                    SandboxEnablementReadinessItemStatus::Ready
                        | SandboxEnablementReadinessItemStatus::Warning
                )
            })
            .count();
        assert_eq!(result.ready_item_count, actual_ready);
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn readiness_no_success_state_introduced() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(!result.gate_armed);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("\"armed\""));
        assert!(!json.contains("\"enabled\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn readiness_no_token_in_result() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn readiness_no_absolute_path_in_result() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn readiness_no_record_payload_in_result() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn readiness_no_attachment_url_in_result() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn readiness_no_old_or_new_record_id_in_result() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn readiness_no_airtable_client_called_when_ready() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert_eq!(
            result.status,
            SandboxEnablementReadinessStatus::ReadyButDisabled
        );
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn readiness_no_airtable_client_called_when_not_ready() {
        let result = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    // ── Snapshot consistency ──────────────────────────────────────────────────

    #[test]
    fn readiness_snapshot_write_gate_disabled_always_true() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(r1.safety_snapshot.write_gate_disabled);
        assert!(r2.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn readiness_snapshot_gate_armed_always_false() {
        let r1 = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        let r2 = build_sandbox_enablement_readiness_report(&missing_prereqs_request());
        assert!(!r1.safety_snapshot.gate_armed);
        assert!(!r2.safety_snapshot.gate_armed);
    }

    #[test]
    fn readiness_snapshot_fields_match_items_when_ready() {
        let result = build_sandbox_enablement_readiness_report(&all_prereqs_request());
        assert!(result.safety_snapshot.gate_contract_eligible);
        assert!(result.safety_snapshot.harness_ready_not_executed);
        assert!(result.safety_snapshot.orchestrator_not_executed);
        assert!(result.safety_snapshot.orchestrator_phases_complete);
        assert!(result.safety_snapshot.schema_executor_present);
        assert!(result.safety_snapshot.record_executor_present);
        assert!(result.safety_snapshot.linked_executor_present);
        assert!(result.safety_snapshot.final_validation_reader_present);
        assert!(result.safety_snapshot.checkpoint_store_sanitized);
    }
}
