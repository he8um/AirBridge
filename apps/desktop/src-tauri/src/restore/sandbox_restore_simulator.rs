use serde::{Deserialize, Serialize};

use crate::restore::restore_orchestrator::{
    build_restore_orchestrator_plan, RestoreOrchestratorMode, RestoreOrchestratorRequest,
    RestoreOrchestratorStatus,
};
use crate::restore::sandbox_gate_arming::{
    build_sandbox_gate_arming_decision, SandboxGateArmingMode, SandboxGateArmingRequest,
    SandboxGateArmingStatus,
};
use crate::restore::sandbox_restore_harness::{
    build_sandbox_restore_harness_plan, SandboxRestoreHarnessMode, SandboxRestoreHarnessRequest,
    SandboxRestoreHarnessStatus,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the sandbox restore simulator run.
///
/// Safety invariants:
/// - `SimulatedNotExecuted` is an internal in-memory status only.
/// - `SimulatedNotExecuted` does NOT enable restore writes, reads, or network calls.
/// - `SimulatedNotExecuted` is not stored globally, not persisted, not reachable from
///   UI, TypeScript, or any Tauri command.
/// - `SimulatedNotExecuted` does NOT call the real Airtable client.
/// - `SimulatedNotExecuted` does NOT write checkpoint files to disk.
/// - `SimulatedNotExecuted` does NOT change `evaluate_write_gate()` behavior.
/// - `execution_enabled` is always `false` regardless of status.
/// - `writes_enabled` is always `false` regardless of status.
/// - `reads_enabled` is always `false` regardless of status.
/// - `gate_armed` (runtime/global) is always `false`.
/// - No `Succeeded`, `Complete`, `ExecutionReady`, `Enabled`, or `Done` status exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxRestoreSimulatorStatus {
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
    /// All prerequisites satisfied and all 8 phases were simulated in memory.
    /// No Airtable calls were made. No execution occurred. No files were written.
    /// This is an in-memory simulation result only.
    SimulatedNotExecuted,
}

/// Mode for the sandbox restore simulator.
///
/// Safety invariants:
/// - `Disabled` is the default and operationally always-reachable mode.
/// - `SandboxOnlyInternalSimulation` is for Rust unit tests only.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxRestoreSimulatorMode {
    /// Simulator disabled — no evaluation is performed. Default state.
    Disabled,
    /// Internal simulation mode for Rust unit tests only.
    /// Does NOT call Airtable, enable execution, reads, writes, or network calls.
    SandboxOnlyInternalSimulation,
}

/// Status of a single simulated phase.
///
/// Note: `succeeded` / `completed` / `done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxRestoreSimulatorPhaseStatus {
    /// The phase is blocked by a safety prerequisite failure.
    Blocked,
    /// The phase was traversed in-memory only. No Airtable call was made.
    /// No execution occurred. No file was written.
    Simulated,
    /// The phase is a checkpoint boundary — described but not written to disk.
    Skipped,
    /// The simulator is in disabled mode. Phase plan was built but not traversed.
    NotExecuted,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single simulated phase in the restore sequence.
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
pub struct SandboxRestoreSimulatorPhase {
    /// Stable phase identifier (e.g. `SRS-PH-01`).
    pub phase_id: String,
    /// Human-readable label.
    pub label: String,
    pub status: SandboxRestoreSimulatorPhaseStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the simulator run.
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
pub struct SandboxRestoreSimulatorSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Runtime/global gate armed state — always `false`.
    pub gate_armed: bool,
    /// Whether the ephemeral arming decision returned `armedNotExecutable`.
    /// Describes the in-memory arming decision object only — does NOT
    /// reflect a global armed state.
    pub ephemeral_armed_decision_seen: bool,
    /// Whether the sandbox restore harness returned `readyNotExecuted`.
    pub harness_ready_not_executed: bool,
    /// Whether the restore orchestrator returned `notExecuted`.
    pub orchestrator_not_executed: bool,
    /// Whether all 8 simulator phases are represented.
    pub all_phases_represented: bool,
    /// Whether checkpoint boundary phases are represented (as simulated descriptors,
    /// not written to disk).
    pub checkpoint_phases_represented: bool,
    /// Whether any Airtable client call was made — always `false`.
    pub airtable_client_called: bool,
    /// Whether any checkpoint file was written to disk — always `false`.
    pub checkpoint_file_written: bool,
    /// Whether execution is enabled — always `false`.
    pub execution_enabled: bool,
    /// Whether writes are enabled — always `false`.
    pub writes_enabled: bool,
    /// Whether reads are enabled — always `false`.
    pub reads_enabled: bool,
}

/// Request to the sandbox restore simulator.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// All prerequisite booleans are caller-declared and are forwarded to the
/// underlying arming, harness, and orchestrator probes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRestoreSimulatorRequest {
    /// Must be `sandboxOnlyInternalSimulation` for simulation.
    /// `disabled` returns `Blocked` immediately.
    pub mode: SandboxRestoreSimulatorMode,
    /// Must be explicitly `true` to proceed.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_simulation_requested: bool,
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
    /// Whether the checkpoint durability policy is safe.
    pub checkpoint_durability_safe: bool,
    /// Whether sensitive data safety policy is satisfied.
    pub sensitive_data_safe: bool,
    /// Whether the final validation enforcement policy is safe.
    pub final_validation_enforcement_safe: bool,
    /// Whether the rate-limit/backoff policy is compliant or warning-safe.
    pub rate_limit_backoff_safe: bool,
}

/// Result of the sandbox restore simulator run.
///
/// Safety invariants (always enforced):
/// - `execution_enabled` is always `false`.
/// - `writes_enabled` is always `false`.
/// - `reads_enabled` is always `false`.
/// - `gate_armed` (runtime/global) is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `airtable_client_called` is always `false`.
/// - `checkpoint_file_written` is always `false`.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `succeeded`, `complete`, `executionReady`, `enabled`, or `done`.
/// - No Airtable client is called.
/// - The result is not persisted globally.
/// - The result is not reachable from UI, TypeScript, or any Tauri command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRestoreSimulatorResult {
    pub status: SandboxRestoreSimulatorStatus,
    pub mode: SandboxRestoreSimulatorMode,
    pub message: String,
    pub phases: Vec<SandboxRestoreSimulatorPhase>,
    pub safety_snapshot: SandboxRestoreSimulatorSafetySnapshot,
    pub total_phase_count: usize,
    pub simulated_phase_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Runtime/global gate armed state — always `false`.
    /// The ephemeral arming decision object (internal) may have `gate_armed: true`,
    /// but that is not reflected here — it is not persisted globally.
    pub gate_armed: bool,
    /// Whether the ephemeral arming decision seen during this run was `armedNotExecutable`.
    /// Describes the in-memory decision only — does NOT indicate a globally armed state.
    pub ephemeral_armed_decision_seen: bool,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network read operations were attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — no Airtable client was called.
    pub airtable_client_called: bool,
    /// Always `false` — no checkpoint file was written to disk.
    pub checkpoint_file_written: bool,
    /// Always `false` — execution is not enabled.
    pub execution_enabled: bool,
    /// Always `false` — live writes are not enabled.
    pub writes_enabled: bool,
    /// Always `false` — live reads are not enabled.
    pub reads_enabled: bool,
}

// ── Phase IDs ─────────────────────────────────────────────────────────────────

const SRS_PH_01: &str = "SRS-PH-01";
const SRS_PH_02: &str = "SRS-PH-02";
const SRS_PH_03: &str = "SRS-PH-03";
const SRS_PH_04: &str = "SRS-PH-04";
const SRS_PH_05: &str = "SRS-PH-05";
const SRS_PH_06: &str = "SRS-PH-06";
const SRS_PH_07: &str = "SRS-PH-07";
const SRS_PH_08: &str = "SRS-PH-08";

// ── Core function ─────────────────────────────────────────────────────────────

/// Runs the sandbox restore simulator.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never enables execution, writes, or reads.
/// - Never changes `evaluate_write_gate()` behavior.
/// - Never writes checkpoint files to disk.
/// - Never stores any state globally.
/// - Is not reachable from UI, TypeScript, or any Tauri command.
/// - Simulates the 8-phase restore sequence entirely in memory:
///   1. Schema write executor phase (SRS-PH-01)
///   2. Schema checkpoint boundary (SRS-PH-02)
///   3. Record write executor phase (SRS-PH-03)
///   4. Record checkpoint boundary (SRS-PH-04)
///   5. Linked second-pass executor phase (SRS-PH-05)
///   6. Linked checkpoint boundary (SRS-PH-06)
///   7. Final validation reader phase (SRS-PH-07)
///   8. Final guard phase (SRS-PH-08)
/// - Always returns `execution_enabled: false`, `writes_enabled: false`,
///   `reads_enabled: false`, `gate_armed: false` (runtime/global),
///   `no_changes_made: true`, `network_reads_attempted: false`,
///   `network_writes_attempted: false`, `airtable_client_called: false`,
///   `checkpoint_file_written: false`.
/// - Returns `Blocked` unless all of the following hold:
///   - `mode` is `SandboxOnlyInternalSimulation`
///   - `explicit_internal_simulation_requested` is `true`
///   - `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`
///   - sandbox gate arming decision returns `ArmedNotExecutable`
///   - sandbox restore harness returns `ReadyNotExecuted`
///   - restore orchestrator returns `NotExecuted`
/// - Returns `SimulatedNotExecuted` when all prerequisites pass — this does NOT
///   enable execution, arm the gate globally, or call any real client.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn run_sandbox_restore_simulator(
    request: &SandboxRestoreSimulatorRequest,
) -> SandboxRestoreSimulatorResult {
    // ── Mode gate ─────────────────────────────────────────────────────────────
    if matches!(request.mode, SandboxRestoreSimulatorMode::Disabled) {
        return sim_blocked(
            SandboxRestoreSimulatorMode::Disabled,
            false,
            false,
            false,
            "Simulator mode is disabled. No simulation is performed. \
             This is the default state."
                .to_string(),
            "SRS-CHK-01: mode must be sandboxOnlyInternalSimulation.".to_string(),
        );
    }

    // ── Explicit simulation flag ──────────────────────────────────────────────
    if !request.explicit_internal_simulation_requested {
        return sim_blocked(
            SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
            false,
            false,
            false,
            "Explicit internal simulation flag is not set. \
             This flag must be explicitly true before simulation can proceed. \
             No UI control, Tauri command, or runtime path sets this flag."
                .to_string(),
            "SRS-CHK-02: explicit_internal_simulation_requested must be true.".to_string(),
        );
    }

    // ── Write gate check ──────────────────────────────────────────────────────
    let gate = evaluate_write_gate();
    if !matches!(gate.status, RestoreWriteEngineStatus::Disabled) {
        return sim_blocked(
            SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
            false,
            false,
            false,
            "evaluate_write_gate() did not return Disabled/DisabledByProductPolicy. \
             This is a critical safety violation. Simulation cannot proceed."
                .to_string(),
            "SRS-CHK-03: evaluate_write_gate() must return Disabled/DisabledByProductPolicy."
                .to_string(),
        );
    }

    // ── Arming probe ──────────────────────────────────────────────────────────
    let arming_req = SandboxGateArmingRequest {
        mode: SandboxGateArmingMode::SandboxOnlyInternal,
        explicit_internal_sandbox_arming_requested: true,
        sandbox_verification_safe: request.sandbox_verification_safe,
        target_empty_safe: request.target_empty_safe,
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
    let arming = build_sandbox_gate_arming_decision(&arming_req);
    let ephemeral_armed = matches!(arming.status, SandboxGateArmingStatus::ArmedNotExecutable);
    if !ephemeral_armed {
        return sim_blocked(
            SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
            false,
            false,
            false,
            format!(
                "Sandbox gate arming decision did not return armedNotExecutable. \
                 Current status: {:?}. \
                 All arming prerequisites must be satisfied before simulation.",
                arming.status
            ),
            "SRS-CHK-04: sandbox gate arming must return armedNotExecutable.".to_string(),
        );
    }

    // ── Harness probe ─────────────────────────────────────────────────────────
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
    let harness = build_sandbox_restore_harness_plan(&harness_req);
    let harness_ready = matches!(
        harness.status,
        SandboxRestoreHarnessStatus::ReadyNotExecuted
    );
    if !harness_ready {
        return sim_blocked(
            SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
            true,
            false,
            false,
            format!(
                "Sandbox restore harness did not return readyNotExecuted. \
                 Current status: {:?}.",
                harness.status
            ),
            "SRS-CHK-05: sandbox restore harness must return readyNotExecuted.".to_string(),
        );
    }

    // ── Orchestrator probe ────────────────────────────────────────────────────
    let orch_req = RestoreOrchestratorRequest {
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
    let orch = build_restore_orchestrator_plan(&orch_req);
    let orch_not_executed = matches!(orch.status, RestoreOrchestratorStatus::NotExecuted);
    if !orch_not_executed {
        return sim_blocked(
            SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
            true,
            true,
            false,
            format!(
                "Restore orchestrator did not return notExecuted. \
                 Current status: {:?}.",
                orch.status
            ),
            "SRS-CHK-06: restore orchestrator must return notExecuted.".to_string(),
        );
    }

    // ── Build in-memory phase simulation ─────────────────────────────────────
    let phases = build_simulation_phases();
    let total = phases.len();
    let simulated_count = phases
        .iter()
        .filter(|p| {
            matches!(
                p.status,
                SandboxRestoreSimulatorPhaseStatus::Simulated
                    | SandboxRestoreSimulatorPhaseStatus::Skipped
            )
        })
        .count();

    SandboxRestoreSimulatorResult {
        status: SandboxRestoreSimulatorStatus::SimulatedNotExecuted,
        mode: SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
        message: "Sandbox restore simulator: simulatedNotExecuted. \
                  All 8 phases were traversed in memory only. \
                  No Airtable calls were made. No execution occurred. No files were written. \
                  The runtime gate is NOT armed. Execution is NOT enabled. \
                  Writes are NOT enabled. Reads are NOT enabled. \
                  This result is not stored globally and does not affect runtime behavior. \
                  evaluate_write_gate() default remains Disabled/DisabledByProductPolicy. \
                  Live sandbox E2E restore execution remains separate pending work."
            .to_string(),
        phases,
        safety_snapshot: SandboxRestoreSimulatorSafetySnapshot {
            write_gate_disabled: true,
            gate_armed: false,
            ephemeral_armed_decision_seen: true,
            harness_ready_not_executed: true,
            orchestrator_not_executed: true,
            all_phases_represented: total == 8,
            checkpoint_phases_represented: true,
            airtable_client_called: false,
            checkpoint_file_written: false,
            execution_enabled: false,
            writes_enabled: false,
            reads_enabled: false,
        },
        total_phase_count: total,
        simulated_phase_count: simulated_count,
        blocked_reason: None,
        gate_armed: false,
        ephemeral_armed_decision_seen: true,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        checkpoint_file_written: false,
        execution_enabled: false,
        writes_enabled: false,
        reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_simulation_phases() -> Vec<SandboxRestoreSimulatorPhase> {
    vec![
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_01.to_string(),
            label: "Schema write executor".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Simulated,
            note: "Schema write executor traversed in memory. \
                   No Airtable schema API call was made. \
                   No table or field was created."
                .to_string(),
        },
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_02.to_string(),
            label: "Schema checkpoint boundary".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Skipped,
            note: "Schema checkpoint boundary described. \
                   No checkpoint file was written to disk."
                .to_string(),
        },
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_03.to_string(),
            label: "Record write executor".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Simulated,
            note: "Record write executor traversed in memory. \
                   No Airtable record API call was made. \
                   No record was created."
                .to_string(),
        },
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_04.to_string(),
            label: "Record checkpoint boundary".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Skipped,
            note: "Record checkpoint boundary described. \
                   No checkpoint file was written to disk."
                .to_string(),
        },
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_05.to_string(),
            label: "Linked second-pass executor".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Simulated,
            note: "Linked second-pass executor traversed in memory. \
                   No Airtable linked record API call was made. \
                   No record was updated."
                .to_string(),
        },
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_06.to_string(),
            label: "Linked checkpoint boundary".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Skipped,
            note: "Linked checkpoint boundary described. \
                   No checkpoint file was written to disk."
                .to_string(),
        },
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_07.to_string(),
            label: "Final validation reader".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Simulated,
            note: "Final validation reader traversed in memory. \
                   No Airtable read API call was made. \
                   No record count or schema was verified against a live base."
                .to_string(),
        },
        SandboxRestoreSimulatorPhase {
            phase_id: SRS_PH_08.to_string(),
            label: "Final guard".to_string(),
            status: SandboxRestoreSimulatorPhaseStatus::Simulated,
            note: "Final guard traversed in memory. \
                   Gate remains disabled. Execution is NOT enabled."
                .to_string(),
        },
    ]
}

fn sim_blocked(
    mode: SandboxRestoreSimulatorMode,
    ephemeral_armed_decision_seen: bool,
    harness_ready_not_executed: bool,
    orchestrator_not_executed: bool,
    message: String,
    blocked_reason: String,
) -> SandboxRestoreSimulatorResult {
    SandboxRestoreSimulatorResult {
        status: SandboxRestoreSimulatorStatus::Blocked,
        mode,
        message,
        phases: vec![],
        safety_snapshot: SandboxRestoreSimulatorSafetySnapshot {
            write_gate_disabled: true,
            gate_armed: false,
            ephemeral_armed_decision_seen,
            harness_ready_not_executed,
            orchestrator_not_executed,
            all_phases_represented: false,
            checkpoint_phases_represented: false,
            airtable_client_called: false,
            checkpoint_file_written: false,
            execution_enabled: false,
            writes_enabled: false,
            reads_enabled: false,
        },
        total_phase_count: 0,
        simulated_phase_count: 0,
        blocked_reason: Some(blocked_reason),
        gate_armed: false,
        ephemeral_armed_decision_seen,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        checkpoint_file_written: false,
        execution_enabled: false,
        writes_enabled: false,
        reads_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_prereqs_request() -> SandboxRestoreSimulatorRequest {
        SandboxRestoreSimulatorRequest {
            mode: SandboxRestoreSimulatorMode::SandboxOnlyInternalSimulation,
            explicit_internal_simulation_requested: true,
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

    fn disabled_request() -> SandboxRestoreSimulatorRequest {
        SandboxRestoreSimulatorRequest {
            mode: SandboxRestoreSimulatorMode::Disabled,
            ..all_prereqs_request()
        }
    }

    // ── SimulatedNotExecuted ──────────────────────────────────────────────────

    #[test]
    fn sim_all_prereqs_returns_simulated_not_executed() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert_eq!(
            result.status,
            SandboxRestoreSimulatorStatus::SimulatedNotExecuted
        );
        assert!(!result.execution_enabled);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(!result.gate_armed);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(!result.airtable_client_called);
        assert!(!result.checkpoint_file_written);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn sim_message_says_not_armed_not_enabled() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
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
    fn sim_message_says_no_airtable_calls() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert!(
            result.message.contains("No Airtable calls were made"),
            "message must say no Airtable calls, got: {}",
            result.message
        );
    }

    #[test]
    fn sim_message_says_execution_pending() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert!(
            result.message.contains("remains separate pending work"),
            "message must say live execution remains pending, got: {}",
            result.message
        );
    }

    #[test]
    fn sim_write_gate_unchanged_after_simulation() {
        let _ = run_sandbox_restore_simulator(&all_prereqs_request());
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── Phase sequence ────────────────────────────────────────────────────────

    #[test]
    fn sim_all_8_phases_represented() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert_eq!(result.total_phase_count, 8);
        assert_eq!(result.phases.len(), 8);
    }

    #[test]
    fn sim_phase_ordering_deterministic() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&all_prereqs_request());
        let ids1: Vec<_> = r1.phases.iter().map(|p| &p.phase_id).collect();
        let ids2: Vec<_> = r2.phases.iter().map(|p| &p.phase_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn sim_first_phase_is_schema_executor() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert_eq!(result.phases[0].phase_id, "SRS-PH-01");
    }

    #[test]
    fn sim_last_phase_is_final_guard() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert_eq!(result.phases[7].phase_id, "SRS-PH-08");
    }

    #[test]
    fn sim_phase_ids_use_srs_ph_prefix() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        for phase in &result.phases {
            assert!(
                phase.phase_id.starts_with("SRS-PH-"),
                "phase_id must start with SRS-PH-, got: {}",
                phase.phase_id
            );
        }
    }

    #[test]
    fn sim_checkpoint_phases_represented_and_skipped() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        let checkpoint_phases: Vec<_> = result
            .phases
            .iter()
            .filter(|p| matches!(p.phase_id.as_str(), "SRS-PH-02" | "SRS-PH-04" | "SRS-PH-06"))
            .collect();
        assert_eq!(checkpoint_phases.len(), 3);
        for cp in checkpoint_phases {
            assert_eq!(
                cp.status,
                SandboxRestoreSimulatorPhaseStatus::Skipped,
                "checkpoint phase {} must be Skipped, got {:?}",
                cp.phase_id,
                cp.status
            );
        }
    }

    #[test]
    fn sim_checkpoint_file_written_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.checkpoint_file_written);
        assert!(!r2.checkpoint_file_written);
        assert!(!r1.safety_snapshot.checkpoint_file_written);
        assert!(!r2.safety_snapshot.checkpoint_file_written);
    }

    #[test]
    fn sim_snapshot_all_phases_represented() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert!(result.safety_snapshot.all_phases_represented);
        assert!(result.safety_snapshot.checkpoint_phases_represented);
    }

    #[test]
    fn sim_no_phase_has_succeeded_status() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        for phase in &result.phases {
            assert!(!matches!(
                phase.status,
                SandboxRestoreSimulatorPhaseStatus::NotExecuted
                    | SandboxRestoreSimulatorPhaseStatus::Blocked
            ));
        }
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn sim_execution_enabled_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.execution_enabled);
        assert!(!r2.execution_enabled);
    }

    #[test]
    fn sim_writes_enabled_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.writes_enabled);
        assert!(!r2.writes_enabled);
    }

    #[test]
    fn sim_reads_enabled_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.reads_enabled);
        assert!(!r2.reads_enabled);
    }

    #[test]
    fn sim_gate_armed_runtime_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.gate_armed);
        assert!(!r2.gate_armed);
        assert!(!r1.safety_snapshot.gate_armed);
        assert!(!r2.safety_snapshot.gate_armed);
    }

    #[test]
    fn sim_no_network_reads_attempted() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.network_reads_attempted);
        assert!(!r2.network_reads_attempted);
    }

    #[test]
    fn sim_no_network_writes_attempted() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.network_writes_attempted);
        assert!(!r2.network_writes_attempted);
    }

    #[test]
    fn sim_no_changes_made_always_true() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(r1.no_changes_made);
        assert!(r2.no_changes_made);
    }

    #[test]
    fn sim_airtable_client_called_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.airtable_client_called);
        assert!(!r2.airtable_client_called);
        assert!(!r1.safety_snapshot.airtable_client_called);
        assert!(!r2.safety_snapshot.airtable_client_called);
    }

    #[test]
    fn sim_evaluate_write_gate_returns_disabled_by_default() {
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    #[test]
    fn sim_snapshot_write_gate_disabled_always_true() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(r1.safety_snapshot.write_gate_disabled);
        assert!(r2.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn sim_snapshot_gate_armed_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.safety_snapshot.gate_armed);
        assert!(!r2.safety_snapshot.gate_armed);
    }

    #[test]
    fn sim_snapshot_execution_enabled_always_false() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&disabled_request());
        assert!(!r1.safety_snapshot.execution_enabled);
        assert!(!r2.safety_snapshot.execution_enabled);
    }

    // ── Ephemeral arming decision ─────────────────────────────────────────────

    #[test]
    fn sim_ephemeral_armed_decision_seen_when_all_prereqs() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert!(result.ephemeral_armed_decision_seen);
        assert!(result.safety_snapshot.ephemeral_armed_decision_seen);
    }

    #[test]
    fn sim_ephemeral_decision_not_persisted_globally() {
        let _ = run_sandbox_restore_simulator(&all_prereqs_request());
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
        // Subsequent blocked call is still blocked.
        let blocked = run_sandbox_restore_simulator(&disabled_request());
        assert_eq!(blocked.status, SandboxRestoreSimulatorStatus::Blocked);
        assert!(!blocked.gate_armed);
    }

    #[test]
    fn sim_two_independent_calls_produce_independent_results() {
        let r1 = run_sandbox_restore_simulator(&all_prereqs_request());
        let r2 = run_sandbox_restore_simulator(&all_prereqs_request());
        assert_eq!(
            r1.status,
            SandboxRestoreSimulatorStatus::SimulatedNotExecuted
        );
        assert_eq!(
            r2.status,
            SandboxRestoreSimulatorStatus::SimulatedNotExecuted
        );
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── Blocked cases ─────────────────────────────────────────────────────────

    #[test]
    fn sim_blocked_when_mode_disabled() {
        let result = run_sandbox_restore_simulator(&disabled_request());
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
        assert!(!result.gate_armed);
        assert!(result.blocked_reason.is_some());
        let reason = result.blocked_reason.unwrap();
        assert!(
            reason.contains("SRS-CHK-01"),
            "blocked_reason must contain SRS-CHK-01, got: {reason}"
        );
    }

    #[test]
    fn sim_blocked_when_explicit_flag_missing() {
        let mut req = all_prereqs_request();
        req.explicit_internal_simulation_requested = false;
        let result = run_sandbox_restore_simulator(&req);
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
        assert!(!result.gate_armed);
        let reason = result.blocked_reason.unwrap();
        assert!(
            reason.contains("SRS-CHK-02"),
            "blocked_reason must contain SRS-CHK-02, got: {reason}"
        );
    }

    #[test]
    fn sim_blocked_when_arming_decision_blocked() {
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        let result = run_sandbox_restore_simulator(&req);
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
        let reason = result.blocked_reason.unwrap();
        assert!(
            reason.contains("SRS-CHK-04"),
            "blocked_reason must contain SRS-CHK-04, got: {reason}"
        );
    }

    #[test]
    fn sim_blocked_when_target_not_empty() {
        let mut req = all_prereqs_request();
        req.target_empty_safe = false;
        let result = run_sandbox_restore_simulator(&req);
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
    }

    #[test]
    fn sim_blocked_when_confirmation_missing() {
        let mut req = all_prereqs_request();
        req.confirmation_gate_declared = false;
        let result = run_sandbox_restore_simulator(&req);
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
    }

    #[test]
    fn sim_blocked_when_write_phase_ordering_unsafe() {
        let mut req = all_prereqs_request();
        req.write_phase_ordering_safe = false;
        let result = run_sandbox_restore_simulator(&req);
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
    }

    #[test]
    fn sim_blocked_when_failure_modes_unsafe() {
        let mut req = all_prereqs_request();
        req.failure_modes_safe = false;
        let result = run_sandbox_restore_simulator(&req);
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
    }

    #[test]
    fn sim_blocked_when_rollback_limitation_unsafe() {
        let mut req = all_prereqs_request();
        req.rollback_limitation_safe = false;
        let result = run_sandbox_restore_simulator(&req);
        assert_eq!(result.status, SandboxRestoreSimulatorStatus::Blocked);
    }

    #[test]
    fn sim_phases_empty_when_blocked() {
        let result = run_sandbox_restore_simulator(&disabled_request());
        assert_eq!(result.total_phase_count, 0);
        assert_eq!(result.phases.len(), 0);
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn sim_no_success_state_introduced() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert!(!result.execution_enabled);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(!result.gate_armed);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("\"enabled\""));
        assert!(!json.contains("executionReady"));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn sim_no_token_in_result() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn sim_no_absolute_path_in_result() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn sim_no_record_payload_in_result() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn sim_no_attachment_url_in_result() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn sim_no_old_or_new_record_id_in_result() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── Phase count / count consistency ──────────────────────────────────────

    #[test]
    fn sim_total_and_simulated_counts_consistent() {
        let result = run_sandbox_restore_simulator(&all_prereqs_request());
        assert_eq!(result.total_phase_count, result.phases.len());
        let actual_sim = result
            .phases
            .iter()
            .filter(|p| {
                matches!(
                    p.status,
                    SandboxRestoreSimulatorPhaseStatus::Simulated
                        | SandboxRestoreSimulatorPhaseStatus::Skipped
                )
            })
            .count();
        assert_eq!(result.simulated_phase_count, actual_sim);
    }
}
