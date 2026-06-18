use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the internal restore orchestration plan.
///
/// Safety invariants:
/// - `NotExecuted` is the only non-blocked reachable status while the write
///   gate is disabled (which it always is in the current build).
/// - No `Succeeded`, `Complete`, or `Done` status exists.
/// - `NotExecuted` does NOT enable any restore writes, reads, or network calls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreOrchestratorStatus {
    /// All prerequisites satisfied but the write gate is disabled.
    /// This is the current expected state — no execution occurs.
    NotExecuted,
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Execution mode for the restore orchestrator.
///
/// Safety invariants:
/// - `Disabled` is the only reachable mode in the current build.
/// - `SandboxOnly` is defined for future use but is unreachable while
///   `evaluate_write_gate()` returns `Disabled`.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreOrchestratorMode {
    /// Write gate is disabled — no execution is possible. Default state.
    Disabled,
    /// Sandbox-only mode — execution would be restricted to verified targets.
    /// Unreachable in the current implementation.
    SandboxOnly,
}

/// Status of a single ordered phase in the orchestration plan.
///
/// Note: `succeeded` / `completed` / `done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreOrchestratorPhaseStatus {
    /// The phase would execute if the gate were enabled. Not executed.
    Pending,
    /// The phase is blocked by a safety prerequisite.
    Blocked,
    /// The phase is a checkpoint boundary — it would persist metadata if enabled.
    Skipped,
    /// The gate is disabled; the phase plan was built but not executed.
    NotExecuted,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single ordered phase in the orchestration plan.
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
pub struct RestoreOrchestratorPhase {
    /// Stable identifier for this phase (e.g. `ORCH-PH-01`).
    pub phase_id: String,
    /// Human-readable label.
    pub label: String,
    pub status: RestoreOrchestratorPhaseStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the orchestration plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOrchestratorSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the mode is sandbox-only (always `false` in the current build).
    pub sandbox_mode_active: bool,
    /// Whether sandbox verification passed.
    pub sandbox_verified: bool,
    /// Whether target empty verification passed.
    pub target_empty_verified: bool,
    /// Whether write phase ordering policy is safe.
    pub write_phase_ordering_safe: bool,
    /// Whether failure modes policy is safe.
    pub failure_modes_safe: bool,
    /// Whether rollback limitation policy is safe.
    pub rollback_limitation_safe: bool,
    /// Whether live write readiness is ready or warning-safe.
    pub live_write_readiness_safe: bool,
    /// Whether the schema write executor foundation completed safely.
    pub schema_executor_safe: bool,
    /// Whether the record write executor foundation completed safely.
    pub record_executor_safe: bool,
    /// Whether the linked second-pass executor foundation completed safely.
    pub linked_executor_safe: bool,
    /// Whether the final validation reader foundation completed safely.
    pub final_validation_reader_safe: bool,
}

/// Request to the restore orchestrator foundation.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// All prerequisite booleans are caller-declared; the orchestrator verifies
/// them in order and blocks on the first unsatisfied prerequisite.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOrchestratorRequest {
    /// Must be `sandboxOnly` for execution to be considered.
    /// `disabled` (the default) always results in `Blocked`.
    pub mode: RestoreOrchestratorMode,
    /// Whether sandbox environment check has passed.
    pub sandbox_verified: bool,
    /// Whether target empty verification has passed.
    pub target_empty_verified: bool,
    /// Whether write phase ordering policy is safe.
    pub write_phase_ordering_safe: bool,
    /// Whether failure modes policy is safe.
    pub failure_modes_safe: bool,
    /// Whether rollback limitation policy is safe.
    pub rollback_limitation_safe: bool,
    /// Whether live write readiness is ready or warning-safe.
    pub live_write_readiness_safe: bool,
    /// Whether schema write executor foundation completed safely.
    pub schema_executor_safe: bool,
    /// Whether record write executor foundation completed safely.
    pub record_executor_safe: bool,
    /// Whether linked second-pass executor foundation completed safely.
    pub linked_executor_safe: bool,
    /// Whether final validation reader foundation completed safely.
    pub final_validation_reader_safe: bool,
}

/// Result of the restore orchestrator foundation.
///
/// Safety invariants (enforced):
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
/// - Status is never `succeeded`, `complete`, or `done`.
/// - `NotExecuted` / `Blocked` do NOT enable live execution.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOrchestratorResult {
    pub status: RestoreOrchestratorStatus,
    pub mode: RestoreOrchestratorMode,
    pub message: String,
    pub phases: Vec<RestoreOrchestratorPhase>,
    pub safety_snapshot: RestoreOrchestratorSafetySnapshot,
    pub total_phase_count: usize,
    pub pending_phase_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
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

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const ORCH_PREREQ_WRITE_GATE: &str = "ORCH-PRE-01";
const ORCH_PREREQ_MODE: &str = "ORCH-PRE-02";
const ORCH_PREREQ_SANDBOX: &str = "ORCH-PRE-03";
const ORCH_PREREQ_TARGET_EMPTY: &str = "ORCH-PRE-04";
const ORCH_PREREQ_PHASE_ORDERING: &str = "ORCH-PRE-05";
const ORCH_PREREQ_FAILURE_MODES: &str = "ORCH-PRE-06";
const ORCH_PREREQ_ROLLBACK: &str = "ORCH-PRE-07";
const ORCH_PREREQ_LIVE_READINESS: &str = "ORCH-PRE-08";
const ORCH_PREREQ_SCHEMA_EXECUTOR: &str = "ORCH-PRE-09";
const ORCH_PREREQ_RECORD_EXECUTOR: &str = "ORCH-PRE-10";
const ORCH_PREREQ_LINKED_EXECUTOR: &str = "ORCH-PRE-11";
const ORCH_PREREQ_FINAL_VALIDATION: &str = "ORCH-PRE-12";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the internal restore orchestration plan.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never creates, updates, or deletes any record, table, or field.
/// - Always enforces the write gate (currently always disabled).
/// - Always returns `writes_enabled: false`, `reads_enabled: false`,
///   `no_changes_made: true`, `network_reads_attempted: false`,
///   `network_writes_attempted: false`.
/// - Returns `Blocked` when any prerequisite is missing.
/// - Returns `NotExecuted` when all prerequisites are met but the gate is
///   disabled — which it always is in the current build.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_restore_orchestrator_plan(
    request: &RestoreOrchestratorRequest,
) -> RestoreOrchestratorResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let safety_snapshot = RestoreOrchestratorSafetySnapshot {
        write_gate_disabled,
        sandbox_mode_active: matches!(request.mode, RestoreOrchestratorMode::SandboxOnly),
        sandbox_verified: request.sandbox_verified,
        target_empty_verified: request.target_empty_verified,
        write_phase_ordering_safe: request.write_phase_ordering_safe,
        failure_modes_safe: request.failure_modes_safe,
        rollback_limitation_safe: request.rollback_limitation_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        schema_executor_safe: request.schema_executor_safe,
        record_executor_safe: request.record_executor_safe,
        linked_executor_safe: request.linked_executor_safe,
        final_validation_reader_safe: request.final_validation_reader_safe,
    };

    // Check prerequisites in order; first failure blocks.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        // Defense-in-depth: unreachable given write gate always returns Disabled.
        Some(format!(
            "{ORCH_PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Restore orchestrator must not proceed while the write gate could be enabled."
        ))
    } else if !matches!(request.mode, RestoreOrchestratorMode::SandboxOnly) {
        Some(format!(
            "{ORCH_PREREQ_MODE}: Orchestrator mode must be sandboxOnly. \
             Mode 'disabled' does not permit any execution. \
             No writes or reads will be attempted."
        ))
    } else if !request.sandbox_verified {
        Some(format!(
            "{ORCH_PREREQ_SANDBOX}: Sandbox environment verification has not passed. \
             A verified sandbox target is required before orchestration is considered."
        ))
    } else if !request.target_empty_verified {
        Some(format!(
            "{ORCH_PREREQ_TARGET_EMPTY}: Target empty verification has not passed. \
             The target base must be verified empty before orchestration is considered."
        ))
    } else if !request.write_phase_ordering_safe {
        Some(format!(
            "{ORCH_PREREQ_PHASE_ORDERING}: Write phase ordering policy is not safe. \
             All phases must be declared in canonical order before orchestration."
        ))
    } else if !request.failure_modes_safe {
        Some(format!(
            "{ORCH_PREREQ_FAILURE_MODES}: Failure modes policy is not safe. \
             All 10 required failure modes must have stop-behavior declarations."
        ))
    } else if !request.rollback_limitation_safe {
        Some(format!(
            "{ORCH_PREREQ_ROLLBACK}: Rollback limitation policy is not safe. \
             Automatic destructive rollback must be disabled before orchestration."
        ))
    } else if !request.live_write_readiness_safe {
        Some(format!(
            "{ORCH_PREREQ_LIVE_READINESS}: Live write readiness is not ready or warning-safe. \
             All 17 required safety gates must be declared before orchestration."
        ))
    } else if !request.schema_executor_safe {
        Some(format!(
            "{ORCH_PREREQ_SCHEMA_EXECUTOR}: Schema write executor foundation has not completed \
             safely. Schema executor must be safe or notExecuted before orchestration."
        ))
    } else if !request.record_executor_safe {
        Some(format!(
            "{ORCH_PREREQ_RECORD_EXECUTOR}: Record write executor foundation has not completed \
             safely. Record executor must be safe or notExecuted before orchestration."
        ))
    } else if !request.linked_executor_safe {
        Some(format!(
            "{ORCH_PREREQ_LINKED_EXECUTOR}: Linked second-pass executor foundation has not \
             completed safely. Linked executor must be safe or notExecuted before orchestration."
        ))
    } else if !request.final_validation_reader_safe {
        Some(format!(
            "{ORCH_PREREQ_FINAL_VALIDATION}: Final validation reader foundation has not \
             completed safely. Reader must be safe or notExecuted before orchestration."
        ))
    } else {
        None
    };

    if let Some(ref reason) = blocked_reason {
        let blocked_phases = build_blocked_phases();
        let total = blocked_phases.len();
        return RestoreOrchestratorResult {
            status: RestoreOrchestratorStatus::Blocked,
            mode: RestoreOrchestratorMode::Disabled,
            message: format!(
                "Restore orchestrator is blocked. {reason} \
                 No writes, reads, or network calls will be attempted."
            ),
            phases: blocked_phases,
            safety_snapshot,
            total_phase_count: total,
            pending_phase_count: 0,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // All prerequisites satisfied — build the internal orchestration plan.
    let phases = build_orchestration_phases();
    let total = phases.len();
    let pending = phases
        .iter()
        .filter(|p| p.status == RestoreOrchestratorPhaseStatus::Pending)
        .count();

    // Write gate is disabled — result is NotExecuted.
    RestoreOrchestratorResult {
        status: RestoreOrchestratorStatus::NotExecuted,
        mode: RestoreOrchestratorMode::Disabled,
        message: format!(
            "Restore orchestration plan built ({total} phase(s), {pending} pending). \
             Write gate is disabled — no Airtable writes or reads are attempted. \
             No old or new record IDs are present. \
             No Airtable changes made."
        ),
        phases,
        safety_snapshot,
        total_phase_count: total,
        pending_phase_count: pending,
        blocked_reason: None,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        writes_enabled: false,
        reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_blocked_phases() -> Vec<RestoreOrchestratorPhase> {
    vec![RestoreOrchestratorPhase {
        phase_id: "ORCH-PH-BLOCKED".to_string(),
        label: "Blocked".to_string(),
        status: RestoreOrchestratorPhaseStatus::Blocked,
        note: "Safety prerequisites not satisfied. No orchestration plan can be built.".to_string(),
    }]
}

fn build_orchestration_phases() -> Vec<RestoreOrchestratorPhase> {
    vec![
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-01".to_string(),
            label: "Schema write executor".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Would invoke schema write executor foundation (tables → direct fields → \
                   deferred linked fields → manual actions). Write gate disabled — \
                   no network call made."
                .to_string(),
        },
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-02".to_string(),
            label: "Schema checkpoint boundary".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Would record schema phase checkpoint boundary. \
                   Metadata-only — no record IDs, no payloads. \
                   Write gate disabled — no file written."
                .to_string(),
        },
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-03".to_string(),
            label: "Record write executor".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Would invoke record write executor foundation (first-pass create batches). \
                   Write gate disabled — no network call made."
                .to_string(),
        },
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-04".to_string(),
            label: "Record checkpoint boundary".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Would record record-create phase checkpoint boundary. \
                   Metadata-only — no record IDs, no payloads. \
                   Write gate disabled — no file written."
                .to_string(),
        },
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-05".to_string(),
            label: "Linked second-pass executor".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Would invoke linked second-pass executor foundation \
                   (second-pass linked-update batches). \
                   Write gate disabled — no network call made."
                .to_string(),
        },
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-06".to_string(),
            label: "Linked phase checkpoint boundary".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Would record linked-update phase checkpoint boundary. \
                   Metadata-only — no record IDs, no payloads. \
                   Write gate disabled — no file written."
                .to_string(),
        },
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-07".to_string(),
            label: "Final validation reader".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Would invoke final validation reader foundation \
                   (schema/table count, field count, record count, \
                   ID mapping coverage, linked coverage, attachment metadata, \
                   manifest/checksum). \
                   Validation read gate disabled — no network call made."
                .to_string(),
        },
        RestoreOrchestratorPhase {
            phase_id: "ORCH-PH-08".to_string(),
            label: "Final guard".to_string(),
            status: RestoreOrchestratorPhaseStatus::Pending,
            note: "Completion guard: no orchestration result can carry a success status \
                   without all prior phases completing and final validation passing. \
                   Write gate disabled — guard is a descriptor only."
                .to_string(),
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_prereqs_request() -> RestoreOrchestratorRequest {
        RestoreOrchestratorRequest {
            mode: RestoreOrchestratorMode::SandboxOnly,
            sandbox_verified: true,
            target_empty_verified: true,
            write_phase_ordering_safe: true,
            failure_modes_safe: true,
            rollback_limitation_safe: true,
            live_write_readiness_safe: true,
            schema_executor_safe: true,
            record_executor_safe: true,
            linked_executor_safe: true,
            final_validation_reader_safe: true,
        }
    }

    // ── Blocked when prerequisites missing ────────────────────────────────────

    #[test]
    fn orchestrator_blocked_when_mode_disabled() {
        let mut req = all_prereqs_request();
        req.mode = RestoreOrchestratorMode::Disabled;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-02"));
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn orchestrator_blocked_when_sandbox_not_verified() {
        let mut req = all_prereqs_request();
        req.sandbox_verified = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-03"));
    }

    #[test]
    fn orchestrator_blocked_when_target_not_empty() {
        let mut req = all_prereqs_request();
        req.target_empty_verified = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-04"));
    }

    #[test]
    fn orchestrator_blocked_when_phase_ordering_unsafe() {
        let mut req = all_prereqs_request();
        req.write_phase_ordering_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-05"));
    }

    #[test]
    fn orchestrator_blocked_when_failure_modes_unsafe() {
        let mut req = all_prereqs_request();
        req.failure_modes_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-06"));
    }

    #[test]
    fn orchestrator_blocked_when_rollback_unsafe() {
        let mut req = all_prereqs_request();
        req.rollback_limitation_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-07"));
    }

    #[test]
    fn orchestrator_blocked_when_live_readiness_not_safe() {
        let mut req = all_prereqs_request();
        req.live_write_readiness_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-08"));
    }

    #[test]
    fn orchestrator_blocked_when_schema_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.schema_executor_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-09"));
    }

    #[test]
    fn orchestrator_blocked_when_record_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.record_executor_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-10"));
    }

    #[test]
    fn orchestrator_blocked_when_linked_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.linked_executor_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-11"));
    }

    #[test]
    fn orchestrator_blocked_when_final_validation_reader_not_safe() {
        let mut req = all_prereqs_request();
        req.final_validation_reader_safe = false;
        let result = build_restore_orchestrator_plan(&req);
        assert_eq!(result.status, RestoreOrchestratorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("ORCH-PRE-12"));
    }

    // ── NotExecuted when all prerequisites met ────────────────────────────────

    #[test]
    fn orchestrator_not_executed_when_all_prereqs_met() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.status, RestoreOrchestratorStatus::NotExecuted);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn orchestrator_write_gate_still_disabled_after_plan() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
    }

    #[test]
    fn orchestrator_safety_snapshot_gate_always_disabled() {
        let mut req = all_prereqs_request();
        req.mode = RestoreOrchestratorMode::Disabled;
        let result = build_restore_orchestrator_plan(&req);
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── No production mode ────────────────────────────────────────────────────

    #[test]
    fn orchestrator_no_production_mode_exists() {
        let disabled = RestoreOrchestratorMode::Disabled;
        let sandbox = RestoreOrchestratorMode::SandboxOnly;
        assert_ne!(disabled, sandbox);
        let json = serde_json::to_string(&disabled).expect("serialize");
        assert!(!json.contains("production"));
        let json = serde_json::to_string(&sandbox).expect("serialize");
        assert!(!json.contains("production"));
    }

    // ── Phase ordering and content ────────────────────────────────────────────

    #[test]
    fn orchestrator_phases_built_in_not_executed_result() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.status, RestoreOrchestratorStatus::NotExecuted);
        assert!(!result.phases.is_empty());
        assert_eq!(result.total_phase_count, 8);
        assert!(result.pending_phase_count > 0);
    }

    #[test]
    fn orchestrator_phase_ordering_is_deterministic() {
        let r1 = build_restore_orchestrator_plan(&all_prereqs_request());
        let r2 = build_restore_orchestrator_plan(&all_prereqs_request());
        let ids1: Vec<_> = r1.phases.iter().map(|p| &p.phase_id).collect();
        let ids2: Vec<_> = r2.phases.iter().map(|p| &p.phase_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn orchestrator_phase_ids_use_stable_prefixes() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        for phase in &result.phases {
            assert!(
                phase.phase_id.starts_with("ORCH-PH-"),
                "phase_id must start with ORCH-PH-, got: {}",
                phase.phase_id
            );
        }
    }

    #[test]
    fn orchestrator_schema_phase_comes_first() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.phases[0].phase_id, "ORCH-PH-01");
    }

    #[test]
    fn orchestrator_final_guard_comes_last() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        let last = result.phases.last().expect("phases not empty");
        assert_eq!(last.phase_id, "ORCH-PH-08");
    }

    #[test]
    fn orchestrator_schema_checkpoint_follows_schema_executor() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.phases[0].phase_id, "ORCH-PH-01"); // schema executor
        assert_eq!(result.phases[1].phase_id, "ORCH-PH-02"); // schema checkpoint
    }

    #[test]
    fn orchestrator_record_checkpoint_follows_record_executor() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.phases[2].phase_id, "ORCH-PH-03"); // record executor
        assert_eq!(result.phases[3].phase_id, "ORCH-PH-04"); // record checkpoint
    }

    #[test]
    fn orchestrator_linked_checkpoint_follows_linked_executor() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.phases[4].phase_id, "ORCH-PH-05"); // linked executor
        assert_eq!(result.phases[5].phase_id, "ORCH-PH-06"); // linked checkpoint
    }

    #[test]
    fn orchestrator_final_validation_reader_precedes_guard() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.phases[6].phase_id, "ORCH-PH-07"); // final validation reader
        assert_eq!(result.phases[7].phase_id, "ORCH-PH-08"); // final guard
    }

    #[test]
    fn orchestrator_total_and_pending_count_consistent() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.total_phase_count, result.phases.len());
        let actual_pending = result
            .phases
            .iter()
            .filter(|p| p.status == RestoreOrchestratorPhaseStatus::Pending)
            .count();
        assert_eq!(result.pending_phase_count, actual_pending);
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn orchestrator_no_success_state_introduced() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("\"completed\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn orchestrator_no_token_in_result() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn orchestrator_no_absolute_path_in_result() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn orchestrator_no_record_payload_in_result() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn orchestrator_no_attachment_url_in_result() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn orchestrator_no_old_or_new_record_id_in_result() {
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn orchestrator_no_airtable_client_called() {
        // build_restore_orchestrator_plan accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.status, RestoreOrchestratorStatus::NotExecuted);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn orchestrator_no_network_in_blocked_state() {
        let mut req = all_prereqs_request();
        req.mode = RestoreOrchestratorMode::Disabled;
        let result = build_restore_orchestrator_plan(&req);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn orchestrator_sandboxonly_still_not_executed_while_gate_disabled() {
        // SandboxOnly + all prereqs → NotExecuted (gate is disabled).
        let result = build_restore_orchestrator_plan(&all_prereqs_request());
        assert_eq!(result.status, RestoreOrchestratorStatus::NotExecuted);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
    }
}
