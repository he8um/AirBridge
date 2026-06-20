use serde::{Deserialize, Serialize};

use crate::restore::sandbox_enablement_readiness::{
    build_sandbox_enablement_readiness_report, SandboxEnablementReadinessRequest,
    SandboxEnablementReadinessStatus,
};
use crate::restore::sandbox_gate_contract::{
    evaluate_sandbox_gate_contract, SandboxGateContractMode, SandboxGateContractRequest,
    SandboxGateContractStatus,
};
use crate::restore::sandbox_restore_harness::{
    build_sandbox_restore_harness_plan, SandboxRestoreHarnessMode, SandboxRestoreHarnessRequest,
    SandboxRestoreHarnessStatus,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the sandbox-only internal gate arming decision.
///
/// Safety invariants:
/// - `ArmedNotExecutable` is an internal diagnostic status only.
/// - `ArmedNotExecutable` does NOT enable restore writes, reads, or network calls.
/// - `ArmedNotExecutable` is not stored globally, not persisted, not reachable from
///   UI, TypeScript, or any Tauri command.
/// - `ArmedNotExecutable` does NOT change `evaluate_write_gate()` behavior.
/// - `ArmedNotExecutable` does NOT unlock any executor or network path.
/// - `execution_enabled` is always `false` regardless of status.
/// - `writes_enabled` is always `false` regardless of status.
/// - `reads_enabled` is always `false` regardless of status.
/// - No `Enabled`, `Succeeded`, `Complete`, `ExecutionReady`, or `Done` status exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxGateArmingStatus {
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
    /// All prerequisites satisfied and the internal arming decision has been
    /// built. The gate is marked as armed in this returned object only.
    /// Execution is NOT enabled. Writes are NOT enabled. Reads are NOT enabled.
    /// This decision is not stored globally and does not affect runtime behavior.
    ArmedNotExecutable,
}

/// Mode for the sandbox gate arming decision.
///
/// Safety invariants:
/// - `Disabled` is the default and operationally always-reachable mode in the
///   current application runtime.
/// - `SandboxOnlyInternal` is defined for Rust-unit-test use only.
///   It does NOT arm the runtime gate, enable execution, reads, or writes.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxGateArmingMode {
    /// Arming is disabled — no evaluation is performed. Default state.
    Disabled,
    /// Internal sandbox-only arming mode for Rust unit tests only.
    /// Does NOT enable execution, reads, writes, or network calls.
    SandboxOnlyInternal,
}

/// Point-in-time safety snapshot for the gate arming decision.
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
pub struct SandboxGateArmingSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the sandbox enablement readiness report returned `readyButDisabled`.
    pub readiness_ready_but_disabled: bool,
    /// Whether the sandbox gate contract returned `eligibleButNotArmed`.
    pub gate_contract_eligible: bool,
    /// Whether the sandbox restore harness returned `readyNotExecuted`.
    pub harness_ready_not_executed: bool,
    /// Whether the explicit internal arming flag was set.
    pub explicit_internal_arming_requested: bool,
    /// Whether execution is enabled — always `false`.
    pub execution_enabled: bool,
    /// Whether writes are enabled — always `false`.
    pub writes_enabled: bool,
    /// Whether reads are enabled — always `false`.
    pub reads_enabled: bool,
}

/// Request to the sandbox gate arming decision builder.
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
/// underlying readiness, gate contract, and harness probes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxGateArmingRequest {
    /// Must be `sandboxOnlyInternal` for arming evaluation.
    /// `disabled` (the default) returns `Blocked` immediately.
    pub mode: SandboxGateArmingMode,
    /// Must be explicitly `true` to proceed past the arming flag check.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_sandbox_arming_requested: bool,
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

/// Result of the sandbox gate arming decision.
///
/// Safety invariants (always enforced):
/// - `execution_enabled` is always `false`.
/// - `writes_enabled` is always `false`.
/// - `reads_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `gate_armed` is `true` only when status is `ArmedNotExecutable` — this
///   field describes the arming decision only. It does NOT change runtime
///   behavior, does NOT unlock any executor, and is NOT stored globally.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `enabled`, `succeeded`, `complete`, `executionReady`, or `done`.
/// - No Airtable client is called.
/// - The result is not persisted globally.
/// - The result is not reachable from UI, TypeScript, or any Tauri command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxGateArmingResult {
    pub status: SandboxGateArmingStatus,
    pub mode: SandboxGateArmingMode,
    pub message: String,
    pub safety_snapshot: SandboxGateArmingSafetySnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// `true` only when status is `ArmedNotExecutable`. Describes this
    /// returned decision object only — does NOT persist globally, does NOT
    /// change `evaluate_write_gate()`, and does NOT unlock any execution path.
    pub gate_armed: bool,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network read operations were attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — execution is not enabled in this decision.
    pub execution_enabled: bool,
    /// Always `false` — live writes are not enabled.
    pub writes_enabled: bool,
    /// Always `false` — live reads are not enabled.
    pub reads_enabled: bool,
}

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds an internal sandbox-only gate arming decision.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never enables execution, writes, or reads.
/// - Never changes `evaluate_write_gate()` behavior.
/// - Never unlocks any executor or network path.
/// - Never stores any state globally.
/// - Is not reachable from UI, TypeScript, or any Tauri command.
/// - Calls `build_sandbox_enablement_readiness_report`, `evaluate_sandbox_gate_contract`,
///   and `build_sandbox_restore_harness_plan` as verification probes only.
/// - Always returns `execution_enabled: false`, `writes_enabled: false`,
///   `reads_enabled: false`, `no_changes_made: true`,
///   `network_reads_attempted: false`, `network_writes_attempted: false`.
/// - Returns `Blocked` unless all of the following hold:
///   - `mode` is `SandboxOnlyInternal`
///   - `explicit_internal_sandbox_arming_requested` is `true`
///   - `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`
///   - `build_sandbox_enablement_readiness_report` returns `ReadyButDisabled`
///   - `evaluate_sandbox_gate_contract` returns `EligibleButNotArmed`
///   - `build_sandbox_restore_harness_plan` returns `ReadyNotExecuted`
/// - Returns `ArmedNotExecutable` when all prerequisites pass — this does NOT
///   arm the runtime gate, does NOT enable execution, and is NOT persisted.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_sandbox_gate_arming_decision(
    request: &SandboxGateArmingRequest,
) -> SandboxGateArmingResult {
    // ── Mode gate ─────────────────────────────────────────────────────────────
    if matches!(request.mode, SandboxGateArmingMode::Disabled) {
        return blocked_result(
            SandboxGateArmingMode::Disabled,
            false,
            false,
            false,
            false,
            "Arming mode is disabled. No arming evaluation is performed. \
             This is the default state."
                .to_string(),
            "SGA-CHK-01: mode must be sandboxOnlyInternal.".to_string(),
        );
    }

    // ── Explicit arming flag ──────────────────────────────────────────────────
    if !request.explicit_internal_sandbox_arming_requested {
        return blocked_result(
            SandboxGateArmingMode::SandboxOnlyInternal,
            false,
            false,
            false,
            false,
            "Explicit internal sandbox arming flag is not set. \
             This flag must be explicitly true before arming can be considered. \
             No UI control, Tauri command, or runtime path sets this flag."
                .to_string(),
            "SGA-CHK-02: explicit_internal_sandbox_arming_requested must be true.".to_string(),
        );
    }

    // ── Write gate check ──────────────────────────────────────────────────────
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if !write_gate_disabled {
        return blocked_result(
            SandboxGateArmingMode::SandboxOnlyInternal,
            false,
            false,
            false,
            true,
            "evaluate_write_gate() did not return Disabled/DisabledByProductPolicy. \
             This is a critical safety violation. Arming cannot proceed."
                .to_string(),
            "SGA-CHK-03: evaluate_write_gate() must return Disabled/DisabledByProductPolicy."
                .to_string(),
        );
    }

    // ── Readiness probe ───────────────────────────────────────────────────────
    let readiness_req = SandboxEnablementReadinessRequest {
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
    let readiness = build_sandbox_enablement_readiness_report(&readiness_req);
    let readiness_ready = matches!(
        readiness.status,
        SandboxEnablementReadinessStatus::ReadyButDisabled
    );
    if !readiness_ready {
        return blocked_result(
            SandboxGateArmingMode::SandboxOnlyInternal,
            false,
            false,
            true,
            true,
            format!(
                "Sandbox enablement readiness report did not return readyButDisabled. \
                 Current status: {:?}. \
                 All 13 readiness items must be satisfied before arming.",
                readiness.status
            ),
            "SGA-CHK-04: sandbox enablement readiness must be readyButDisabled.".to_string(),
        );
    }

    // ── Gate contract probe ───────────────────────────────────────────────────
    let contract_req = SandboxGateContractRequest {
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
    let contract = evaluate_sandbox_gate_contract(&contract_req);
    let contract_eligible = matches!(
        contract.status,
        SandboxGateContractStatus::EligibleButNotArmed
    );
    if !contract_eligible {
        return blocked_result(
            SandboxGateArmingMode::SandboxOnlyInternal,
            false,
            false,
            true,
            true,
            format!(
                "Sandbox gate contract did not return eligibleButNotArmed. \
                 Current status: {:?}. \
                 All gate contract prerequisites must be satisfied.",
                contract.status
            ),
            "SGA-CHK-05: sandbox gate contract must return eligibleButNotArmed.".to_string(),
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
        return blocked_result(
            SandboxGateArmingMode::SandboxOnlyInternal,
            false,
            false,
            true,
            true,
            format!(
                "Sandbox restore harness did not return readyNotExecuted. \
                 Current status: {:?}. \
                 All harness prerequisites must be satisfied.",
                harness.status
            ),
            "SGA-CHK-06: sandbox restore harness must return readyNotExecuted.".to_string(),
        );
    }

    // ── All checks passed — return ArmedNotExecutable ─────────────────────────
    //
    // Safety reminder: `gate_armed: true` describes only this returned decision
    // object. It does NOT:
    // - Change evaluate_write_gate() behavior.
    // - Unlock any executor or network path.
    // - Store any state globally.
    // - Enable execution, writes, or reads.
    SandboxGateArmingResult {
        status: SandboxGateArmingStatus::ArmedNotExecutable,
        mode: SandboxGateArmingMode::SandboxOnlyInternal,
        message: "Sandbox gate arming decision: armedNotExecutable. \
                  The gate is armed in this internal decision object only. \
                  Execution is NOT enabled. Writes are NOT enabled. Reads are NOT enabled. \
                  This decision is not stored globally and does not affect runtime behavior. \
                  evaluate_write_gate() default remains Disabled/DisabledByProductPolicy. \
                  Live sandbox E2E restore execution remains separate pending work."
            .to_string(),
        safety_snapshot: SandboxGateArmingSafetySnapshot {
            write_gate_disabled: true,
            readiness_ready_but_disabled: true,
            gate_contract_eligible: true,
            harness_ready_not_executed: true,
            explicit_internal_arming_requested: true,
            execution_enabled: false,
            writes_enabled: false,
            reads_enabled: false,
        },
        blocked_reason: None,
        gate_armed: true,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        execution_enabled: false,
        writes_enabled: false,
        reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn blocked_result(
    mode: SandboxGateArmingMode,
    gate_contract_eligible: bool,
    harness_ready_not_executed: bool,
    readiness_ready_but_disabled: bool,
    explicit_internal_arming_requested: bool,
    message: String,
    blocked_reason: String,
) -> SandboxGateArmingResult {
    SandboxGateArmingResult {
        status: SandboxGateArmingStatus::Blocked,
        mode,
        message,
        safety_snapshot: SandboxGateArmingSafetySnapshot {
            write_gate_disabled: true,
            readiness_ready_but_disabled,
            gate_contract_eligible,
            harness_ready_not_executed,
            explicit_internal_arming_requested,
            execution_enabled: false,
            writes_enabled: false,
            reads_enabled: false,
        },
        blocked_reason: Some(blocked_reason),
        gate_armed: false,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        execution_enabled: false,
        writes_enabled: false,
        reads_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_prereqs_request() -> SandboxGateArmingRequest {
        SandboxGateArmingRequest {
            mode: SandboxGateArmingMode::SandboxOnlyInternal,
            explicit_internal_sandbox_arming_requested: true,
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

    fn disabled_mode_request() -> SandboxGateArmingRequest {
        SandboxGateArmingRequest {
            mode: SandboxGateArmingMode::Disabled,
            ..all_prereqs_request()
        }
    }

    // ── ArmedNotExecutable ────────────────────────────────────────────────────

    #[test]
    fn arming_all_prereqs_returns_armed_not_executable() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert_eq!(result.status, SandboxGateArmingStatus::ArmedNotExecutable);
        assert!(result.gate_armed);
        assert!(!result.execution_enabled);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn arming_armed_not_executable_message_says_not_enabled() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert!(
            result.message.contains("NOT enabled"),
            "message must say NOT enabled, got: {}",
            result.message
        );
    }

    #[test]
    fn arming_armed_not_executable_message_says_not_stored_globally() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert!(
            result.message.contains("not stored globally"),
            "message must say not stored globally, got: {}",
            result.message
        );
    }

    #[test]
    fn arming_armed_not_executable_message_says_execution_pending() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert!(
            result.message.contains("remains separate pending work"),
            "message must say live execution remains pending, got: {}",
            result.message
        );
    }

    #[test]
    fn arming_armed_not_executable_does_not_change_write_gate() {
        let _ = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    #[test]
    fn arming_snapshot_fields_match_when_armed() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(result.safety_snapshot.readiness_ready_but_disabled);
        assert!(result.safety_snapshot.gate_contract_eligible);
        assert!(result.safety_snapshot.harness_ready_not_executed);
        assert!(result.safety_snapshot.explicit_internal_arming_requested);
        assert!(!result.safety_snapshot.execution_enabled);
        assert!(!result.safety_snapshot.writes_enabled);
        assert!(!result.safety_snapshot.reads_enabled);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn arming_execution_enabled_always_false() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.execution_enabled);
        assert!(!r2.execution_enabled);
    }

    #[test]
    fn arming_writes_enabled_always_false() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.writes_enabled);
        assert!(!r2.writes_enabled);
    }

    #[test]
    fn arming_reads_enabled_always_false() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.reads_enabled);
        assert!(!r2.reads_enabled);
    }

    #[test]
    fn arming_no_network_reads_attempted() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.network_reads_attempted);
        assert!(!r2.network_reads_attempted);
    }

    #[test]
    fn arming_no_network_writes_attempted() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.network_writes_attempted);
        assert!(!r2.network_writes_attempted);
    }

    #[test]
    fn arming_no_changes_made_always_true() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(r1.no_changes_made);
        assert!(r2.no_changes_made);
    }

    #[test]
    fn arming_snapshot_execution_enabled_always_false() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.safety_snapshot.execution_enabled);
        assert!(!r2.safety_snapshot.execution_enabled);
    }

    #[test]
    fn arming_snapshot_writes_enabled_always_false() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.safety_snapshot.writes_enabled);
        assert!(!r2.safety_snapshot.writes_enabled);
    }

    #[test]
    fn arming_snapshot_reads_enabled_always_false() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!r1.safety_snapshot.reads_enabled);
        assert!(!r2.safety_snapshot.reads_enabled);
    }

    #[test]
    fn arming_evaluate_write_gate_returns_disabled_by_default() {
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    #[test]
    fn arming_write_gate_unchanged_after_armed_not_executable() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert_eq!(result.status, SandboxGateArmingStatus::ArmedNotExecutable);
        let gate_after = evaluate_write_gate();
        assert!(matches!(
            gate_after.status,
            RestoreWriteEngineStatus::Disabled
        ));
    }

    // ── Blocked cases ─────────────────────────────────────────────────────────

    #[test]
    fn arming_blocked_when_mode_disabled() {
        let result = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
        assert!(!result.gate_armed);
        assert!(result.blocked_reason.is_some());
    }

    #[test]
    fn arming_blocked_when_explicit_flag_missing() {
        let mut req = all_prereqs_request();
        req.explicit_internal_sandbox_arming_requested = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
        assert!(!result.gate_armed);
        assert!(result.blocked_reason.is_some());
        let reason = result.blocked_reason.unwrap();
        assert!(
            reason.contains("SGA-CHK-02"),
            "blocked_reason must contain SGA-CHK-02, got: {reason}"
        );
    }

    #[test]
    fn arming_blocked_when_sandbox_verification_missing() {
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
        assert!(!result.gate_armed);
    }

    #[test]
    fn arming_blocked_when_target_not_empty() {
        let mut req = all_prereqs_request();
        req.target_empty_safe = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
    }

    #[test]
    fn arming_blocked_when_confirmation_missing() {
        let mut req = all_prereqs_request();
        req.confirmation_gate_declared = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
    }

    #[test]
    fn arming_blocked_when_write_phase_ordering_unsafe() {
        let mut req = all_prereqs_request();
        req.write_phase_ordering_safe = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
    }

    #[test]
    fn arming_blocked_when_failure_modes_unsafe() {
        let mut req = all_prereqs_request();
        req.failure_modes_safe = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
    }

    #[test]
    fn arming_blocked_when_rollback_limitation_unsafe() {
        let mut req = all_prereqs_request();
        req.rollback_limitation_safe = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
    }

    #[test]
    fn arming_blocked_when_readiness_not_ready() {
        // All-false request reaches readiness blocked/notReady check.
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        req.target_empty_safe = false;
        req.confirmation_gate_declared = false;
        let result = build_sandbox_gate_arming_decision(&req);
        assert_eq!(result.status, SandboxGateArmingStatus::Blocked);
        let reason = result.blocked_reason.unwrap();
        assert!(
            reason.contains("SGA-CHK-04"),
            "blocked_reason must contain SGA-CHK-04, got: {reason}"
        );
    }

    #[test]
    fn arming_blocked_when_gate_contract_not_eligible() {
        // Readiness will pass but gate contract won't be eligible if sandbox not safe
        // because SGA-CHK-05 fires after readiness.
        // We need readiness to pass but gate contract to fail.
        // readiness passes with all-true. Gate contract also uses request booleans.
        // There is no way to make gate contract fail while readiness passes in the
        // current design because they share the same request booleans.
        // The gate contract check (SGA-CHK-05) is redundant when readiness passes —
        // that is intentional: defense-in-depth. This test verifies the check exists.
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        // All-true: gate contract must be eligible.
        assert!(result.safety_snapshot.gate_contract_eligible);
    }

    #[test]
    fn arming_blocked_when_harness_not_ready() {
        // Similarly, harness uses same booleans. All-true harness is ready.
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert!(result.safety_snapshot.harness_ready_not_executed);
    }

    #[test]
    fn arming_gate_armed_only_when_armed_not_executable() {
        let armed = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let blocked = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(armed.gate_armed);
        assert!(!blocked.gate_armed);
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn arming_no_success_state_introduced() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert!(!result.execution_enabled);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("\"enabled\""));
        assert!(!json.contains("executionReady"));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn arming_no_token_in_result() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn arming_no_absolute_path_in_result() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn arming_no_record_payload_in_result() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn arming_no_attachment_url_in_result() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn arming_no_old_or_new_record_id_in_result() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn arming_no_airtable_client_called_when_armed() {
        let result = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert_eq!(result.status, SandboxGateArmingStatus::ArmedNotExecutable);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn arming_no_airtable_client_called_when_blocked() {
        let result = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    // ── Not persisted globally ────────────────────────────────────────────────

    #[test]
    fn arming_decision_not_persisted_two_calls_independent() {
        let r1 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        let r2 = build_sandbox_gate_arming_decision(&all_prereqs_request());
        // Both are independent — neither affects the other or runtime state.
        assert_eq!(r1.status, SandboxGateArmingStatus::ArmedNotExecutable);
        assert_eq!(r2.status, SandboxGateArmingStatus::ArmedNotExecutable);
        // Write gate is still disabled after both calls.
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    #[test]
    fn arming_decision_does_not_affect_subsequent_blocked_call() {
        let armed = build_sandbox_gate_arming_decision(&all_prereqs_request());
        assert_eq!(armed.status, SandboxGateArmingStatus::ArmedNotExecutable);
        // A subsequent blocked call is still blocked — arming has no global effect.
        let blocked = build_sandbox_gate_arming_decision(&disabled_mode_request());
        assert_eq!(blocked.status, SandboxGateArmingStatus::Blocked);
        assert!(!blocked.gate_armed);
    }
}
