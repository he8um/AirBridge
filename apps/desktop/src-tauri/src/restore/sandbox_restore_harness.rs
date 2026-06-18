use serde::{Deserialize, Serialize};

use crate::restore::restore_orchestrator::{
    build_restore_orchestrator_plan, RestoreOrchestratorMode, RestoreOrchestratorRequest,
    RestoreOrchestratorResult, RestoreOrchestratorStatus,
};
use crate::restore::sandbox_gate_contract::{
    evaluate_sandbox_gate_contract, SandboxGateContractMode, SandboxGateContractRequest,
    SandboxGateContractResult, SandboxGateContractStatus,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the sandbox restore harness evaluation.
///
/// Safety invariants:
/// - `ReadyNotExecuted` does NOT enable restore writes, reads, or network calls.
/// - `ReadyNotExecuted` does NOT arm the gate.
/// - `NotExecuted` is the default state when the harness is in `disabled` mode.
/// - No `Armed`, `Enabled`, `Succeeded`, `Complete`, or `Done` status exists.
/// - `gate_armed` is always `false` regardless of status.
/// - `writes_enabled` is always `false` regardless of status.
/// - `reads_enabled` is always `false` regardless of status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxRestoreHarnessStatus {
    /// A required safety prerequisite is missing or unsafe.
    /// The harness cannot proceed toward any execution.
    Blocked,
    /// All prerequisites satisfied, gate contract is `eligibleButNotArmed`,
    /// orchestrator is `notExecuted`. The harness plan is built but no
    /// execution occurred and none is possible. This is the best reachable
    /// non-blocked state — it does not arm the gate or start any execution.
    ReadyNotExecuted,
    /// The harness is in `disabled` mode. No prerequisite evaluation was
    /// performed. Default state.
    NotExecuted,
}

/// Mode for the sandbox restore harness.
///
/// Safety invariants:
/// - `Disabled` is the only operationally reachable mode in the current build.
/// - `SandboxOnlyDryHarness` is defined for future diagnostic use only.
///   Even when selected, it does NOT arm the gate or enable execution.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxRestoreHarnessMode {
    /// Harness is disabled — no evaluation is performed. Default state.
    Disabled,
    /// Dry harness evaluation mode for future sandbox E2E testing preparation.
    /// Does NOT arm the gate or enable execution.
    SandboxOnlyDryHarness,
}

/// Status of a single phase in the harness plan.
///
/// Note: `succeeded` / `completed` / `done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxRestoreHarnessPhaseStatus {
    /// The phase is blocked by a safety prerequisite failure.
    Blocked,
    /// The phase would execute if the gate were armed and enabled.
    /// Not executed in the current build.
    Pending,
    /// The phase is a checkpoint or boundary that is skipped at this harness level.
    Skipped,
    /// The gate is disabled; the phase plan is built but not executed.
    NotExecuted,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single ordered phase in the harness plan.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No old or new Airtable record IDs.
/// - No raw record field values.
/// - No raw HTTP body.
/// - No attachment URL.
/// - `status` is never `succeeded`, `complete`, or `done`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRestoreHarnessPhase {
    /// Stable identifier for this harness phase (e.g. `SRH-PH-01`).
    pub phase_id: String,
    /// Human-readable label.
    pub label: String,
    pub status: SandboxRestoreHarnessPhaseStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the harness evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRestoreHarnessSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Gate is armed — always `false` in the current build.
    pub gate_armed: bool,
    /// Whether the sandbox gate contract was evaluated (not in disabled mode).
    pub gate_contract_evaluated: bool,
    /// Whether the gate contract returned `eligibleButNotArmed`.
    pub gate_contract_eligible: bool,
    /// Whether the restore orchestrator returned `notExecuted`.
    pub orchestrator_not_executed: bool,
    /// Whether the schema executor phase is represented in the orchestrator plan.
    pub schema_phase_represented: bool,
    /// Whether the record executor phase is represented in the orchestrator plan.
    pub record_phase_represented: bool,
    /// Whether the linked second-pass executor phase is represented.
    pub linked_phase_represented: bool,
    /// Whether the final validation reader phase is represented.
    pub final_validation_phase_represented: bool,
    /// Whether at least one checkpoint boundary phase is represented.
    pub checkpoint_phase_represented: bool,
}

/// Request to the sandbox restore harness.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// All prerequisite booleans are caller-declared. They reflect whether the
/// corresponding module is present and in a safe/notExecuted state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRestoreHarnessRequest {
    /// Must be `sandboxOnlyDryHarness` for full harness evaluation.
    /// `disabled` (the default) returns `NotExecuted` immediately.
    pub mode: SandboxRestoreHarnessMode,
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
}

/// Result of the sandbox restore harness evaluation.
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
/// - `ReadyNotExecuted` does NOT arm the gate or start any execution.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRestoreHarnessResult {
    pub status: SandboxRestoreHarnessStatus,
    pub mode: SandboxRestoreHarnessMode,
    pub message: String,
    pub phases: Vec<SandboxRestoreHarnessPhase>,
    pub safety_snapshot: SandboxRestoreHarnessSafetySnapshot,
    pub total_phase_count: usize,
    pub pending_phase_count: usize,
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

// ── Phase IDs ─────────────────────────────────────────────────────────────────

const SRH_PHASE_GATE_CONTRACT: &str = "SRH-PH-01";
const SRH_PHASE_ORCHESTRATOR: &str = "SRH-PH-02";
const SRH_PHASE_SCHEMA_EXECUTOR: &str = "SRH-PH-03";
const SRH_PHASE_RECORD_EXECUTOR: &str = "SRH-PH-04";
const SRH_PHASE_LINKED_EXECUTOR: &str = "SRH-PH-05";
const SRH_PHASE_FINAL_VALIDATION: &str = "SRH-PH-06";
const SRH_PHASE_CHECKPOINT_BOUNDARIES: &str = "SRH-PH-07";
const SRH_PHASE_FINAL_GUARD: &str = "SRH-PH-08";

// ── Known orchestrator phase IDs ──────────────────────────────────────────────

const ORCH_PH_SCHEMA_EXECUTOR: &str = "ORCH-PH-01";
const ORCH_PH_SCHEMA_CHECKPOINT: &str = "ORCH-PH-02";
const ORCH_PH_RECORD_EXECUTOR: &str = "ORCH-PH-03";
const ORCH_PH_RECORD_CHECKPOINT: &str = "ORCH-PH-04";
const ORCH_PH_LINKED_EXECUTOR: &str = "ORCH-PH-05";
const ORCH_PH_LINKED_CHECKPOINT: &str = "ORCH-PH-06";
const ORCH_PH_FINAL_VALIDATION: &str = "ORCH-PH-07";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the sandbox restore harness plan.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never arms the gate or enables execution.
/// - Never creates, updates, or deletes any record, table, or field.
/// - Always enforces the write gate (currently always disabled).
/// - Always returns `gate_armed: false`, `writes_enabled: false`,
///   `reads_enabled: false`, `no_changes_made: true`,
///   `network_reads_attempted: false`, `network_writes_attempted: false`.
/// - Returns `NotExecuted` when mode is `disabled`.
/// - Returns `Blocked` when any required prerequisite is missing or unsafe.
/// - Returns `ReadyNotExecuted` when all prerequisites are satisfied, the gate
///   contract is `eligibleButNotArmed`, and the orchestrator is `notExecuted`.
///   This does NOT arm the gate or enable execution of any kind.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_sandbox_restore_harness_plan(
    request: &SandboxRestoreHarnessRequest,
) -> SandboxRestoreHarnessResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let base_snapshot = SandboxRestoreHarnessSafetySnapshot {
        write_gate_disabled,
        gate_armed: false,
        gate_contract_evaluated: false,
        gate_contract_eligible: false,
        orchestrator_not_executed: false,
        schema_phase_represented: false,
        record_phase_represented: false,
        linked_phase_represented: false,
        final_validation_phase_represented: false,
        checkpoint_phase_represented: false,
    };

    // Mode gate: disabled mode returns immediately without evaluation.
    if matches!(request.mode, SandboxRestoreHarnessMode::Disabled) {
        let phases = vec![SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_GATE_CONTRACT.to_string(),
            label: "Sandbox gate contract evaluation".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Skipped,
            note: "Harness is in disabled mode — no evaluation performed.".to_string(),
        }];
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::NotExecuted,
            mode: SandboxRestoreHarnessMode::Disabled,
            message: "Sandbox restore harness is in disabled mode. \
                      No evaluation is performed. \
                      No writes, reads, or network calls are attempted."
                .to_string(),
            phases,
            safety_snapshot: base_snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: None,
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // Evaluate gate contract.
    let gate_contract_request = build_gate_contract_request(request);
    let gate_contract_result = evaluate_sandbox_gate_contract(&gate_contract_request);
    let gate_contract_eligible = matches!(
        gate_contract_result.status,
        SandboxGateContractStatus::EligibleButNotArmed
    );

    // Gate contract must be eligible (or at minimum not disabled) to proceed.
    if !gate_contract_eligible {
        let reason = gate_contract_result
            .blocked_reason
            .clone()
            .unwrap_or_else(|| "Gate contract not eligible.".to_string());
        let snapshot = SandboxRestoreHarnessSafetySnapshot {
            write_gate_disabled,
            gate_armed: false,
            gate_contract_evaluated: true,
            gate_contract_eligible: false,
            orchestrator_not_executed: false,
            schema_phase_represented: false,
            record_phase_represented: false,
            linked_phase_represented: false,
            final_validation_phase_represented: false,
            checkpoint_phase_represented: false,
        };
        let phases = build_gate_contract_blocked_phases(&gate_contract_result);
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::Blocked,
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            message: format!(
                "Sandbox restore harness is blocked. \
                 Gate contract did not reach eligibleButNotArmed. {reason} \
                 No writes, reads, or network calls are attempted."
            ),
            phases,
            safety_snapshot: snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some(format!("Gate contract: {reason}")),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // Evaluate restore orchestrator.
    let orchestrator_request = build_orchestrator_request(request);
    let orchestrator_result = build_restore_orchestrator_plan(&orchestrator_request);
    let orchestrator_not_executed = matches!(
        orchestrator_result.status,
        RestoreOrchestratorStatus::NotExecuted
    );

    // Orchestrator must be notExecuted (write gate enforces this).
    if !orchestrator_not_executed {
        let reason = orchestrator_result
            .blocked_reason
            .clone()
            .unwrap_or_else(|| "Orchestrator prerequisite not satisfied.".to_string());
        let snapshot = SandboxRestoreHarnessSafetySnapshot {
            write_gate_disabled,
            gate_armed: false,
            gate_contract_evaluated: true,
            gate_contract_eligible: true,
            orchestrator_not_executed: false,
            schema_phase_represented: false,
            record_phase_represented: false,
            linked_phase_represented: false,
            final_validation_phase_represented: false,
            checkpoint_phase_represented: false,
        };
        let phases = build_orchestrator_blocked_phases(&orchestrator_result);
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::Blocked,
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            message: format!(
                "Sandbox restore harness is blocked. \
                 Restore orchestrator did not reach notExecuted. {reason} \
                 No writes, reads, or network calls are attempted."
            ),
            phases,
            safety_snapshot: snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some(format!("Orchestrator: {reason}")),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // Verify required phase representation in the orchestrator plan.
    let phase_ids: Vec<&str> = orchestrator_result
        .phases
        .iter()
        .map(|p| p.phase_id.as_str())
        .collect();
    let schema_represented = phase_ids.contains(&ORCH_PH_SCHEMA_EXECUTOR);
    let record_represented = phase_ids.contains(&ORCH_PH_RECORD_EXECUTOR);
    let linked_represented = phase_ids.contains(&ORCH_PH_LINKED_EXECUTOR);
    let final_val_represented = phase_ids.contains(&ORCH_PH_FINAL_VALIDATION);
    let checkpoint_represented = phase_ids.contains(&ORCH_PH_SCHEMA_CHECKPOINT)
        || phase_ids.contains(&ORCH_PH_RECORD_CHECKPOINT)
        || phase_ids.contains(&ORCH_PH_LINKED_CHECKPOINT);

    let snapshot = SandboxRestoreHarnessSafetySnapshot {
        write_gate_disabled,
        gate_armed: false,
        gate_contract_evaluated: true,
        gate_contract_eligible: true,
        orchestrator_not_executed: true,
        schema_phase_represented: schema_represented,
        record_phase_represented: record_represented,
        linked_phase_represented: linked_represented,
        final_validation_phase_represented: final_val_represented,
        checkpoint_phase_represented: checkpoint_represented,
    };

    if !schema_represented {
        let phases = vec![make_blocked_phase(
            SRH_PHASE_SCHEMA_EXECUTOR,
            "Schema executor phase verification",
            "Schema executor phase (ORCH-PH-01) is not represented in the orchestrator plan.",
        )];
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::Blocked,
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            message: "Sandbox restore harness is blocked. \
                      Schema executor phase is not represented in the orchestrator plan. \
                      No writes, reads, or network calls are attempted."
                .to_string(),
            phases,
            safety_snapshot: snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some("SRH-PH-03: Schema executor phase not represented.".to_string()),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    if !record_represented {
        let phases = vec![make_blocked_phase(
            SRH_PHASE_RECORD_EXECUTOR,
            "Record executor phase verification",
            "Record executor phase (ORCH-PH-03) is not represented in the orchestrator plan.",
        )];
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::Blocked,
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            message: "Sandbox restore harness is blocked. \
                      Record executor phase is not represented in the orchestrator plan. \
                      No writes, reads, or network calls are attempted."
                .to_string(),
            phases,
            safety_snapshot: snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some("SRH-PH-04: Record executor phase not represented.".to_string()),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    if !linked_represented {
        let phases = vec![make_blocked_phase(
            SRH_PHASE_LINKED_EXECUTOR,
            "Linked second-pass executor phase verification",
            "Linked second-pass executor phase (ORCH-PH-05) is not represented in the orchestrator plan.",
        )];
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::Blocked,
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            message: "Sandbox restore harness is blocked. \
                      Linked second-pass executor phase is not represented in the orchestrator plan. \
                      No writes, reads, or network calls are attempted."
                .to_string(),
            phases,
            safety_snapshot: snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some(
                "SRH-PH-05: Linked second-pass executor phase not represented.".to_string(),
            ),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    if !final_val_represented {
        let phases = vec![make_blocked_phase(
            SRH_PHASE_FINAL_VALIDATION,
            "Final validation reader phase verification",
            "Final validation reader phase (ORCH-PH-07) is not represented in the orchestrator plan.",
        )];
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::Blocked,
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            message: "Sandbox restore harness is blocked. \
                      Final validation reader phase is not represented in the orchestrator plan. \
                      No writes, reads, or network calls are attempted."
                .to_string(),
            phases,
            safety_snapshot: snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some(
                "SRH-PH-06: Final validation reader phase not represented.".to_string(),
            ),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    if !checkpoint_represented {
        let phases = vec![make_blocked_phase(
            SRH_PHASE_CHECKPOINT_BOUNDARIES,
            "Checkpoint boundary phase verification",
            "No checkpoint boundary phase (ORCH-PH-02/04/06) is represented in the orchestrator plan.",
        )];
        let total = phases.len();
        return SandboxRestoreHarnessResult {
            status: SandboxRestoreHarnessStatus::Blocked,
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            message: "Sandbox restore harness is blocked. \
                      No checkpoint boundary phase is represented in the orchestrator plan. \
                      No writes, reads, or network calls are attempted."
                .to_string(),
            phases,
            safety_snapshot: snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some(
                "SRH-PH-07: No checkpoint boundary phase represented.".to_string(),
            ),
            gate_armed: false,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // All prerequisites satisfied — build the full harness plan.
    let phases = build_harness_phases();
    let total = phases.len();
    let pending_count = phases
        .iter()
        .filter(|p| matches!(p.status, SandboxRestoreHarnessPhaseStatus::Pending))
        .count();

    SandboxRestoreHarnessResult {
        status: SandboxRestoreHarnessStatus::ReadyNotExecuted,
        mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
        message: format!(
            "Sandbox restore harness plan built with {total} phases. \
             Gate contract is eligibleButNotArmed. Orchestrator is notExecuted. \
             All executor and checkpoint phases are represented. \
             The gate is NOT armed and NOT enabled. \
             No writes, reads, or network calls are attempted. \
             Live sandbox E2E restore execution remains pending."
        ),
        phases,
        safety_snapshot: snapshot,
        total_phase_count: total,
        pending_phase_count: pending_count,
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

fn build_gate_contract_request(
    request: &SandboxRestoreHarnessRequest,
) -> SandboxGateContractRequest {
    SandboxGateContractRequest {
        mode: SandboxGateContractMode::SandboxOnlyCandidate,
        sandbox_verification_safe: request.sandbox_verification_safe,
        target_empty_safe: request.target_empty_safe,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        // The harness itself represents the orchestrator — always declared.
        restore_orchestrator_present: true,
        schema_executor_present: true,
        record_executor_present: true,
        linked_executor_present: true,
        final_validation_reader_present: true,
    }
}

fn build_orchestrator_request(
    request: &SandboxRestoreHarnessRequest,
) -> RestoreOrchestratorRequest {
    RestoreOrchestratorRequest {
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
    }
}

fn make_blocked_phase(phase_id: &str, label: &str, note: &str) -> SandboxRestoreHarnessPhase {
    SandboxRestoreHarnessPhase {
        phase_id: phase_id.to_string(),
        label: label.to_string(),
        status: SandboxRestoreHarnessPhaseStatus::Blocked,
        note: note.to_string(),
    }
}

fn build_gate_contract_blocked_phases(
    gate_result: &SandboxGateContractResult,
) -> Vec<SandboxRestoreHarnessPhase> {
    let note = gate_result
        .blocked_reason
        .clone()
        .unwrap_or_else(|| "Gate contract not eligible.".to_string());
    vec![SandboxRestoreHarnessPhase {
        phase_id: SRH_PHASE_GATE_CONTRACT.to_string(),
        label: "Sandbox gate contract evaluation".to_string(),
        status: SandboxRestoreHarnessPhaseStatus::Blocked,
        note,
    }]
}

fn build_orchestrator_blocked_phases(
    orch_result: &RestoreOrchestratorResult,
) -> Vec<SandboxRestoreHarnessPhase> {
    let note = orch_result
        .blocked_reason
        .clone()
        .unwrap_or_else(|| "Orchestrator prerequisite not satisfied.".to_string());
    vec![
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_GATE_CONTRACT.to_string(),
            label: "Sandbox gate contract evaluation".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::NotExecuted,
            note: "Gate contract reached eligibleButNotArmed.".to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_ORCHESTRATOR.to_string(),
            label: "Restore orchestrator evaluation".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Blocked,
            note,
        },
    ]
}

fn build_harness_phases() -> Vec<SandboxRestoreHarnessPhase> {
    vec![
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_GATE_CONTRACT.to_string(),
            label: "Sandbox gate contract evaluation".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::NotExecuted,
            note: "Gate contract reached eligibleButNotArmed — gate NOT armed.".to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_ORCHESTRATOR.to_string(),
            label: "Restore orchestrator evaluation".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::NotExecuted,
            note: "Orchestrator reached notExecuted — write gate enforced.".to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_SCHEMA_EXECUTOR.to_string(),
            label: "Schema executor phase (ORCH-PH-01) represented".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Pending,
            note: "Schema executor foundation is represented in the orchestrator plan. \
                   Not executed — gate not armed."
                .to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_RECORD_EXECUTOR.to_string(),
            label: "Record executor phase (ORCH-PH-03) represented".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Pending,
            note: "Record executor foundation is represented in the orchestrator plan. \
                   Not executed — gate not armed."
                .to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_LINKED_EXECUTOR.to_string(),
            label: "Linked second-pass executor phase (ORCH-PH-05) represented".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Pending,
            note: "Linked second-pass executor foundation is represented. \
                   Not executed — gate not armed."
                .to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_FINAL_VALIDATION.to_string(),
            label: "Final validation reader phase (ORCH-PH-07) represented".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Pending,
            note: "Final validation reader foundation is represented in the orchestrator plan. \
                   Not executed — gate not armed."
                .to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_CHECKPOINT_BOUNDARIES.to_string(),
            label: "Checkpoint boundary phases represented".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Skipped,
            note: "Checkpoint boundary phases (ORCH-PH-02/04/06) are represented. \
                   No checkpoint files are written — gate not armed."
                .to_string(),
        },
        SandboxRestoreHarnessPhase {
            phase_id: SRH_PHASE_FINAL_GUARD.to_string(),
            label: "Final harness guard".to_string(),
            status: SandboxRestoreHarnessPhaseStatus::Pending,
            note: "Final harness guard passed — harness plan is complete. \
                   Gate NOT armed. Execution NOT enabled. \
                   Live sandbox E2E restore execution remains pending."
                .to_string(),
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn disabled_request() -> SandboxRestoreHarnessRequest {
        SandboxRestoreHarnessRequest {
            mode: SandboxRestoreHarnessMode::Disabled,
            sandbox_verification_safe: false,
            target_empty_safe: false,
            confirmation_gate_declared: false,
            destructive_operation_policy_safe: false,
            attachment_phase_disabled_safe: false,
            live_write_readiness_safe: false,
            write_phase_ordering_safe: false,
            failure_modes_safe: false,
            rollback_limitation_safe: false,
        }
    }

    fn all_prereqs_request() -> SandboxRestoreHarnessRequest {
        SandboxRestoreHarnessRequest {
            mode: SandboxRestoreHarnessMode::SandboxOnlyDryHarness,
            sandbox_verification_safe: true,
            target_empty_safe: true,
            confirmation_gate_declared: true,
            destructive_operation_policy_safe: true,
            attachment_phase_disabled_safe: true,
            live_write_readiness_safe: true,
            write_phase_ordering_safe: true,
            failure_modes_safe: true,
            rollback_limitation_safe: true,
        }
    }

    // ── Default disabled mode ─────────────────────────────────────────────────

    #[test]
    fn harness_default_disabled_mode_returns_not_executed() {
        let result = build_sandbox_restore_harness_plan(&disabled_request());
        assert_eq!(result.status, SandboxRestoreHarnessStatus::NotExecuted);
        assert!(!result.gate_armed);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn harness_disabled_write_gate_remains_disabled() {
        let result = build_sandbox_restore_harness_plan(&disabled_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.safety_snapshot.gate_armed);
    }

    #[test]
    fn harness_disabled_has_no_blocked_reason() {
        let result = build_sandbox_restore_harness_plan(&disabled_request());
        assert!(result.blocked_reason.is_none());
    }

    // ── ReadyNotExecuted when all prereqs satisfied ───────────────────────────

    #[test]
    fn harness_all_prereqs_returns_ready_not_executed() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert_eq!(result.status, SandboxRestoreHarnessStatus::ReadyNotExecuted);
        assert!(!result.gate_armed);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn harness_ready_not_executed_is_not_enabled() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(!result.gate_armed);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn harness_ready_not_executed_message_says_not_armed() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
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
    fn harness_ready_not_executed_message_says_pending() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(
            result.message.contains("remains pending"),
            "message must say execution remains pending, got: {}",
            result.message
        );
    }

    #[test]
    fn harness_write_gate_still_disabled_when_ready() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.safety_snapshot.gate_armed);
    }

    #[test]
    fn harness_gate_contract_eligible_when_ready() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(result.safety_snapshot.gate_contract_evaluated);
        assert!(result.safety_snapshot.gate_contract_eligible);
    }

    #[test]
    fn harness_orchestrator_not_executed_when_ready() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(result.safety_snapshot.orchestrator_not_executed);
    }

    #[test]
    fn harness_all_phase_representations_true_when_ready() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(result.safety_snapshot.schema_phase_represented);
        assert!(result.safety_snapshot.record_phase_represented);
        assert!(result.safety_snapshot.linked_phase_represented);
        assert!(result.safety_snapshot.final_validation_phase_represented);
        assert!(result.safety_snapshot.checkpoint_phase_represented);
    }

    // ── Blocked when gate contract not eligible ───────────────────────────────

    #[test]
    fn harness_blocked_when_sandbox_not_safe() {
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
        assert!(!result.gate_armed);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn harness_blocked_when_target_not_empty() {
        let mut req = all_prereqs_request();
        req.target_empty_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
    }

    #[test]
    fn harness_blocked_when_confirmation_not_declared() {
        let mut req = all_prereqs_request();
        req.confirmation_gate_declared = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
    }

    #[test]
    fn harness_blocked_when_destructive_policy_unsafe() {
        let mut req = all_prereqs_request();
        req.destructive_operation_policy_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
    }

    #[test]
    fn harness_blocked_when_attachment_phase_unsafe() {
        let mut req = all_prereqs_request();
        req.attachment_phase_disabled_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
    }

    // ── Blocked when orchestrator prerequisites missing ────────────────────────

    #[test]
    fn harness_blocked_when_write_phase_ordering_unsafe() {
        let mut req = all_prereqs_request();
        req.write_phase_ordering_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("Orchestrator"));
    }

    #[test]
    fn harness_blocked_when_failure_modes_unsafe() {
        let mut req = all_prereqs_request();
        req.failure_modes_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
    }

    #[test]
    fn harness_blocked_when_rollback_limitation_unsafe() {
        let mut req = all_prereqs_request();
        req.rollback_limitation_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert_eq!(result.status, SandboxRestoreHarnessStatus::Blocked);
    }

    // ── Write gate invariants ─────────────────────────────────────────────────

    #[test]
    fn harness_evaluate_write_gate_returns_disabled_by_default() {
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    #[test]
    fn harness_gate_armed_is_always_false() {
        let result_disabled = build_sandbox_restore_harness_plan(&disabled_request());
        let result_ready = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(!result_disabled.gate_armed);
        assert!(!result_ready.gate_armed);
        assert!(!result_disabled.safety_snapshot.gate_armed);
        assert!(!result_ready.safety_snapshot.gate_armed);
    }

    #[test]
    fn harness_writes_enabled_always_false() {
        let result_disabled = build_sandbox_restore_harness_plan(&disabled_request());
        let result_ready = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(!result_disabled.writes_enabled);
        assert!(!result_ready.writes_enabled);
    }

    #[test]
    fn harness_reads_enabled_always_false() {
        let result_disabled = build_sandbox_restore_harness_plan(&disabled_request());
        let result_ready = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(!result_disabled.reads_enabled);
        assert!(!result_ready.reads_enabled);
    }

    #[test]
    fn harness_no_network_reads_attempted() {
        let result_disabled = build_sandbox_restore_harness_plan(&disabled_request());
        let result_ready = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(!result_disabled.network_reads_attempted);
        assert!(!result_ready.network_reads_attempted);
    }

    #[test]
    fn harness_no_network_writes_attempted() {
        let result_disabled = build_sandbox_restore_harness_plan(&disabled_request());
        let result_ready = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(!result_disabled.network_writes_attempted);
        assert!(!result_ready.network_writes_attempted);
    }

    #[test]
    fn harness_no_changes_made_always_true() {
        let result_disabled = build_sandbox_restore_harness_plan(&disabled_request());
        let result_ready = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert!(result_disabled.no_changes_made);
        assert!(result_ready.no_changes_made);
    }

    // ── Phase structure ───────────────────────────────────────────────────────

    #[test]
    fn harness_phase_count_is_eight_when_ready() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert_eq!(result.total_phase_count, 8);
        assert_eq!(result.phases.len(), 8);
    }

    #[test]
    fn harness_phase_ordering_deterministic() {
        let r1 = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let r2 = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let ids1: Vec<_> = r1.phases.iter().map(|p| &p.phase_id).collect();
        let ids2: Vec<_> = r2.phases.iter().map(|p| &p.phase_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn harness_first_phase_is_gate_contract() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert_eq!(result.phases[0].phase_id, SRH_PHASE_GATE_CONTRACT);
    }

    #[test]
    fn harness_last_phase_is_final_guard() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let last = result.phases.last().expect("phases not empty");
        assert_eq!(last.phase_id, SRH_PHASE_FINAL_GUARD);
    }

    #[test]
    fn harness_phase_ids_use_srh_prefix() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        for phase in &result.phases {
            assert!(
                phase.phase_id.starts_with("SRH-PH-"),
                "phase_id must start with SRH-PH-, got: {}",
                phase.phase_id
            );
        }
    }

    #[test]
    fn harness_pending_count_consistent() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let actual_pending = result
            .phases
            .iter()
            .filter(|p| matches!(p.status, SandboxRestoreHarnessPhaseStatus::Pending))
            .count();
        assert_eq!(result.pending_phase_count, actual_pending);
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn harness_no_success_state_introduced() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
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

    // ── No production mode ────────────────────────────────────────────────────

    #[test]
    fn harness_no_production_mode_exists() {
        let disabled = SandboxRestoreHarnessMode::Disabled;
        let dry = SandboxRestoreHarnessMode::SandboxOnlyDryHarness;
        assert_ne!(disabled, dry);
        let json_disabled = serde_json::to_string(&disabled).expect("serialize");
        assert!(!json_disabled.contains("production"));
        let json_dry = serde_json::to_string(&dry).expect("serialize");
        assert!(!json_dry.contains("production"));
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn harness_no_token_in_result() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn harness_no_absolute_path_in_result() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn harness_no_record_payload_in_result() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn harness_no_attachment_url_in_result() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn harness_no_old_or_new_record_id_in_result() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn harness_no_airtable_client_called_when_ready() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert_eq!(result.status, SandboxRestoreHarnessStatus::ReadyNotExecuted);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn harness_no_airtable_client_called_when_blocked() {
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        let result = build_sandbox_restore_harness_plan(&req);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn harness_no_airtable_client_called_when_disabled() {
        let result = build_sandbox_restore_harness_plan(&disabled_request());
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn harness_total_and_pending_counts_consistent() {
        let result = build_sandbox_restore_harness_plan(&all_prereqs_request());
        assert_eq!(result.total_phase_count, result.phases.len());
        let actual_pending = result
            .phases
            .iter()
            .filter(|p| matches!(p.status, SandboxRestoreHarnessPhaseStatus::Pending))
            .count();
        assert_eq!(result.pending_phase_count, actual_pending);
    }
}
