use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the sandbox-only gate contract evaluation.
///
/// Safety invariants:
/// - `EligibleButNotArmed` does NOT enable restore writes or reads.
/// - `EligibleButNotArmed` does NOT arm the gate or start any execution.
/// - `Disabled` is the default and only operationally reachable status in the
///   current build while `evaluate_write_gate()` returns `Disabled`.
/// - No `Armed`, `Enabled`, `Succeeded`, `Complete`, or `Done` status exists.
/// - `writes_enabled` is always `false` regardless of status.
/// - `reads_enabled` is always `false` regardless of status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxGateContractStatus {
    /// Default state. The write gate is disabled by product policy.
    /// No prerequisites are evaluated. No arming is possible.
    Disabled,
    /// A required safety prerequisite is missing or unsafe.
    /// The gate is not eligible to be armed even in future work.
    Blocked,
    /// All known prerequisites are reported as satisfied, but the gate is NOT
    /// armed and NOT enabled. This is a forward-looking diagnostic status only —
    /// it does not enable execution of any kind.
    EligibleButNotArmed,
}

/// Mode for the sandbox gate contract evaluation.
///
/// Safety invariants:
/// - `Disabled` is the only operationally reachable mode in the current build.
/// - `SandboxOnlyCandidate` is defined for future diagnostic use. Even when
///   selected, it does NOT arm the gate or enable execution.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxGateContractMode {
    /// Write gate is disabled — no arming or execution is possible. Default state.
    Disabled,
    /// Diagnostic mode for evaluating whether prerequisites are satisfied.
    /// Does NOT arm the gate or enable execution.
    SandboxOnlyCandidate,
}

/// Status of a single prerequisite in the gate contract evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxGateContractPrerequisiteStatus {
    /// The prerequisite is declared and meets the safety threshold.
    Safe,
    /// The prerequisite is declared but has a non-critical concern.
    Warning,
    /// The prerequisite is declared but has a critical safety violation.
    Blocked,
    /// The prerequisite has not been declared or evaluated.
    Missing,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single evaluated prerequisite in the gate contract.
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
pub struct SandboxGateContractPrerequisite {
    /// Stable prerequisite identifier (e.g. `SGC-PRE-01`).
    pub prereq_id: String,
    /// Human-readable label.
    pub label: String,
    pub status: SandboxGateContractPrerequisiteStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the gate contract evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxGateContractSafetySnapshot {
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether sandbox verification is declared and safe.
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
    /// Whether the restore orchestrator foundation is present and default-blocked.
    pub restore_orchestrator_present: bool,
    /// Whether schema executor foundation is present and default-blocked.
    pub schema_executor_present: bool,
    /// Whether record executor foundation is present and default-blocked.
    pub record_executor_present: bool,
    /// Whether linked second-pass executor foundation is present and default-blocked.
    pub linked_executor_present: bool,
    /// Whether final validation reader foundation is present and default-blocked.
    pub final_validation_reader_present: bool,
}

/// Request to the sandbox gate contract evaluator.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// All prerequisite booleans are caller-declared. The evaluator checks them in
/// order and reports the first missing or unsafe prerequisite.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxGateContractRequest {
    /// Must be `sandboxOnlyCandidate` for full prerequisite evaluation.
    /// `disabled` (the default) returns `Disabled` immediately with no
    /// prerequisite evaluation.
    pub mode: SandboxGateContractMode,
    /// Whether sandbox verification is declared and safe.
    pub sandbox_verification_safe: bool,
    /// Whether target empty verification is declared and safe.
    pub target_empty_safe: bool,
    /// Whether the explicit confirmation gate has been declared.
    pub confirmation_gate_declared: bool,
    /// Whether the destructive operation policy is safe.
    pub destructive_operation_policy_safe: bool,
    /// Whether the attachment phase disabled policy is safe.
    pub attachment_phase_disabled_safe: bool,
    /// Whether live write readiness is ready or warning-safe.
    pub live_write_readiness_safe: bool,
    /// Whether the restore orchestrator foundation is present and default-blocked.
    pub restore_orchestrator_present: bool,
    /// Whether the schema executor foundation is present and default-blocked.
    pub schema_executor_present: bool,
    /// Whether the record executor foundation is present and default-blocked.
    pub record_executor_present: bool,
    /// Whether the linked second-pass executor foundation is present and default-blocked.
    pub linked_executor_present: bool,
    /// Whether the final validation reader foundation is present and default-blocked.
    pub final_validation_reader_present: bool,
}

/// Result of the sandbox gate contract evaluation.
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
/// - Status is never `armed`, `enabled`, `succeeded`, `complete`, or `done`.
/// - `EligibleButNotArmed` does NOT arm the gate or start any execution.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxGateContractResult {
    pub status: SandboxGateContractStatus,
    pub mode: SandboxGateContractMode,
    pub message: String,
    pub prerequisites: Vec<SandboxGateContractPrerequisite>,
    pub safety_snapshot: SandboxGateContractSafetySnapshot,
    pub total_prereq_count: usize,
    pub safe_prereq_count: usize,
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

const SGC_PREREQ_SANDBOX: &str = "SGC-PRE-01";
const SGC_PREREQ_TARGET_EMPTY: &str = "SGC-PRE-02";
const SGC_PREREQ_CONFIRMATION: &str = "SGC-PRE-03";
const SGC_PREREQ_DESTRUCTIVE_POLICY: &str = "SGC-PRE-04";
const SGC_PREREQ_ATTACHMENT_PHASE: &str = "SGC-PRE-05";
const SGC_PREREQ_LIVE_READINESS: &str = "SGC-PRE-06";
const SGC_PREREQ_ORCHESTRATOR: &str = "SGC-PRE-07";
const SGC_PREREQ_SCHEMA_EXECUTOR: &str = "SGC-PRE-08";
const SGC_PREREQ_RECORD_EXECUTOR: &str = "SGC-PRE-09";
const SGC_PREREQ_LINKED_EXECUTOR: &str = "SGC-PRE-10";
const SGC_PREREQ_FINAL_VALIDATION: &str = "SGC-PRE-11";
const SGC_PREREQ_WRITE_GATE_DEFAULT: &str = "SGC-PRE-12";

// ── Core function ─────────────────────────────────────────────────────────────

/// Evaluates the sandbox-only gate contract prerequisites.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never arms the gate or enables execution.
/// - Never creates, updates, or deletes any record, table, or field.
/// - Always enforces the write gate (currently always disabled).
/// - Always returns `writes_enabled: false`, `reads_enabled: false`,
///   `no_changes_made: true`, `network_reads_attempted: false`,
///   `network_writes_attempted: false`.
/// - Returns `Disabled` when mode is `Disabled`.
/// - Returns `Blocked` when any prerequisite is missing or unsafe.
/// - Returns `EligibleButNotArmed` when all prerequisites are satisfied —
///   this does NOT arm the gate or enable execution of any kind.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn evaluate_sandbox_gate_contract(
    request: &SandboxGateContractRequest,
) -> SandboxGateContractResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let safety_snapshot = SandboxGateContractSafetySnapshot {
        write_gate_disabled,
        sandbox_verification_safe: request.sandbox_verification_safe,
        target_empty_safe: request.target_empty_safe,
        confirmation_gate_declared: request.confirmation_gate_declared,
        destructive_operation_policy_safe: request.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
        live_write_readiness_safe: request.live_write_readiness_safe,
        restore_orchestrator_present: request.restore_orchestrator_present,
        schema_executor_present: request.schema_executor_present,
        record_executor_present: request.record_executor_present,
        linked_executor_present: request.linked_executor_present,
        final_validation_reader_present: request.final_validation_reader_present,
    };

    // Mode gate: disabled mode returns immediately without prerequisite evaluation.
    if matches!(request.mode, SandboxGateContractMode::Disabled) {
        return SandboxGateContractResult {
            status: SandboxGateContractStatus::Disabled,
            mode: SandboxGateContractMode::Disabled,
            message: "Sandbox gate contract is in disabled mode. \
                      No prerequisite evaluation is performed. \
                      No writes, reads, or network calls are attempted."
                .to_string(),
            prerequisites: vec![],
            safety_snapshot,
            total_prereq_count: 0,
            safe_prereq_count: 0,
            blocked_reason: None,
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // Evaluate prerequisites in order.
    let prereqs = build_prerequisites(request, write_gate_disabled);
    let total = prereqs.len();
    let safe_count = prereqs
        .iter()
        .filter(|p| {
            matches!(
                p.status,
                SandboxGateContractPrerequisiteStatus::Safe
                    | SandboxGateContractPrerequisiteStatus::Warning
            )
        })
        .count();

    let blocked_prereq = prereqs.iter().find(|p| {
        matches!(
            p.status,
            SandboxGateContractPrerequisiteStatus::Blocked
                | SandboxGateContractPrerequisiteStatus::Missing
        )
    });

    if let Some(prereq) = blocked_prereq {
        let reason = format!("{}: {}", prereq.prereq_id, prereq.note);
        return SandboxGateContractResult {
            status: SandboxGateContractStatus::Blocked,
            mode: SandboxGateContractMode::SandboxOnlyCandidate,
            message: format!(
                "Sandbox gate contract is blocked. {reason} \
                 No writes, reads, or network calls are attempted. \
                 The gate cannot be armed."
            ),
            prerequisites: prereqs,
            safety_snapshot,
            total_prereq_count: total,
            safe_prereq_count: safe_count,
            blocked_reason: Some(reason),
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            writes_enabled: false,
            reads_enabled: false,
        };
    }

    // All prerequisites satisfied — eligible but NOT armed.
    SandboxGateContractResult {
        status: SandboxGateContractStatus::EligibleButNotArmed,
        mode: SandboxGateContractMode::SandboxOnlyCandidate,
        message: format!(
            "All {total} sandbox gate contract prerequisites are satisfied. \
             The gate is eligible for future arming but is NOT armed and NOT enabled. \
             No writes, reads, or network calls are attempted. \
             Arming the gate requires a separate explicit future action."
        ),
        prerequisites: prereqs,
        safety_snapshot,
        total_prereq_count: total,
        safe_prereq_count: safe_count,
        blocked_reason: None,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        writes_enabled: false,
        reads_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_prerequisites(
    request: &SandboxGateContractRequest,
    write_gate_disabled: bool,
) -> Vec<SandboxGateContractPrerequisite> {
    vec![
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_SANDBOX.to_string(),
            label: "Sandbox verification declared and safe".to_string(),
            status: if request.sandbox_verification_safe {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.sandbox_verification_safe {
                "Sandbox environment verification is declared and safe.".to_string()
            } else {
                "Sandbox verification has not been declared or is not safe. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_TARGET_EMPTY.to_string(),
            label: "Target empty verification declared and safe".to_string(),
            status: if request.target_empty_safe {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.target_empty_safe {
                "Target empty verification is declared and safe.".to_string()
            } else {
                "Target empty verification has not been declared or is not safe. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_CONFIRMATION.to_string(),
            label: "Explicit confirmation gate declared".to_string(),
            status: if request.confirmation_gate_declared {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.confirmation_gate_declared {
                "Explicit user confirmation gate is declared.".to_string()
            } else {
                "Explicit confirmation gate has not been declared. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_DESTRUCTIVE_POLICY.to_string(),
            label: "Destructive operation policy safe".to_string(),
            status: if request.destructive_operation_policy_safe {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Blocked
            },
            note: if request.destructive_operation_policy_safe {
                "Destructive operation policy is safe — no delete/overwrite operations planned."
                    .to_string()
            } else {
                "Destructive operation policy has a critical violation. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_ATTACHMENT_PHASE.to_string(),
            label: "Attachment phase disabled policy safe".to_string(),
            status: if request.attachment_phase_disabled_safe {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Blocked
            },
            note: if request.attachment_phase_disabled_safe {
                "Attachment phase disabled policy is safe — \
                 binary attachment operations remain disabled."
                    .to_string()
            } else {
                "Attachment phase disabled policy has a critical violation. \
                 Binary attachment operations must remain disabled. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_LIVE_READINESS.to_string(),
            label: "Live write readiness ready or warning-safe".to_string(),
            status: if request.live_write_readiness_safe {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.live_write_readiness_safe {
                "Live write readiness policy is ready or warning-safe.".to_string()
            } else {
                "Live write readiness has not been declared or is blocked. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_ORCHESTRATOR.to_string(),
            label: "Restore orchestrator foundation present and default-blocked".to_string(),
            status: if request.restore_orchestrator_present {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.restore_orchestrator_present {
                "Restore orchestrator foundation is present and default-blocked. \
                 Gate enablement would compose all executor foundations in sequence."
                    .to_string()
            } else {
                "Restore orchestrator foundation has not been declared. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_SCHEMA_EXECUTOR.to_string(),
            label: "Schema executor foundation present and default-blocked".to_string(),
            status: if request.schema_executor_present {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.schema_executor_present {
                "Schema write executor foundation is present and default-blocked.".to_string()
            } else {
                "Schema write executor foundation has not been declared. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_RECORD_EXECUTOR.to_string(),
            label: "Record executor foundation present and default-blocked".to_string(),
            status: if request.record_executor_present {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.record_executor_present {
                "Record write executor foundation is present and default-blocked.".to_string()
            } else {
                "Record write executor foundation has not been declared. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_LINKED_EXECUTOR.to_string(),
            label: "Linked second-pass executor foundation present and default-blocked".to_string(),
            status: if request.linked_executor_present {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.linked_executor_present {
                "Linked second-pass executor foundation is present and default-blocked.".to_string()
            } else {
                "Linked second-pass executor foundation has not been declared. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_FINAL_VALIDATION.to_string(),
            label: "Final validation reader foundation present and default-blocked".to_string(),
            status: if request.final_validation_reader_present {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Missing
            },
            note: if request.final_validation_reader_present {
                "Final validation reader foundation is present and default-blocked.".to_string()
            } else {
                "Final validation reader foundation has not been declared. \
                 Gate cannot be armed."
                    .to_string()
            },
        },
        SandboxGateContractPrerequisite {
            prereq_id: SGC_PREREQ_WRITE_GATE_DEFAULT.to_string(),
            label: "Write gate default remains Disabled/DisabledByProductPolicy".to_string(),
            status: if write_gate_disabled {
                SandboxGateContractPrerequisiteStatus::Safe
            } else {
                SandboxGateContractPrerequisiteStatus::Blocked
            },
            note: if write_gate_disabled {
                "evaluate_write_gate() returns Disabled/DisabledByProductPolicy. \
                 This is the required default state."
                    .to_string()
            } else {
                "evaluate_write_gate() does NOT return Disabled. \
                 The write gate default has been unexpectedly changed. \
                 Gate cannot be armed until this is resolved."
                    .to_string()
            },
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn disabled_request() -> SandboxGateContractRequest {
        SandboxGateContractRequest {
            mode: SandboxGateContractMode::Disabled,
            sandbox_verification_safe: false,
            target_empty_safe: false,
            confirmation_gate_declared: false,
            destructive_operation_policy_safe: false,
            attachment_phase_disabled_safe: false,
            live_write_readiness_safe: false,
            restore_orchestrator_present: false,
            schema_executor_present: false,
            record_executor_present: false,
            linked_executor_present: false,
            final_validation_reader_present: false,
        }
    }

    fn all_prereqs_request() -> SandboxGateContractRequest {
        SandboxGateContractRequest {
            mode: SandboxGateContractMode::SandboxOnlyCandidate,
            sandbox_verification_safe: true,
            target_empty_safe: true,
            confirmation_gate_declared: true,
            destructive_operation_policy_safe: true,
            attachment_phase_disabled_safe: true,
            live_write_readiness_safe: true,
            restore_orchestrator_present: true,
            schema_executor_present: true,
            record_executor_present: true,
            linked_executor_present: true,
            final_validation_reader_present: true,
        }
    }

    // ── Default disabled mode ─────────────────────────────────────────────────

    #[test]
    fn gate_contract_default_disabled_mode_returns_disabled() {
        let result = evaluate_sandbox_gate_contract(&disabled_request());
        assert_eq!(result.status, SandboxGateContractStatus::Disabled);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.prerequisites.is_empty());
    }

    #[test]
    fn gate_contract_write_gate_remains_disabled_by_default() {
        let result = evaluate_sandbox_gate_contract(&disabled_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn gate_contract_disabled_has_no_blocked_reason() {
        let result = evaluate_sandbox_gate_contract(&disabled_request());
        assert!(result.blocked_reason.is_none());
    }

    // ── Eligible but not armed ────────────────────────────────────────────────

    #[test]
    fn gate_contract_all_prereqs_returns_eligible_not_armed() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        assert_eq!(
            result.status,
            SandboxGateContractStatus::EligibleButNotArmed
        );
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn gate_contract_eligible_not_armed_is_not_enabled() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        // EligibleButNotArmed is categorically not "enabled" — no writes/reads possible.
        assert_ne!(result.status, SandboxGateContractStatus::Disabled);
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn gate_contract_eligible_not_armed_message_says_not_armed() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
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
    fn gate_contract_write_gate_still_disabled_when_eligible() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.writes_enabled);
    }

    // ── Blocked when prerequisites missing ───────────────────────────────────

    #[test]
    fn gate_contract_blocked_when_sandbox_not_safe() {
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-01"));
    }

    #[test]
    fn gate_contract_blocked_when_target_empty_not_safe() {
        let mut req = all_prereqs_request();
        req.target_empty_safe = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-02"));
    }

    #[test]
    fn gate_contract_blocked_when_confirmation_not_declared() {
        let mut req = all_prereqs_request();
        req.confirmation_gate_declared = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-03"));
    }

    #[test]
    fn gate_contract_blocked_when_destructive_policy_unsafe() {
        let mut req = all_prereqs_request();
        req.destructive_operation_policy_safe = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-04"));
    }

    #[test]
    fn gate_contract_blocked_when_attachment_phase_unsafe() {
        let mut req = all_prereqs_request();
        req.attachment_phase_disabled_safe = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-05"));
    }

    #[test]
    fn gate_contract_blocked_when_live_readiness_missing() {
        let mut req = all_prereqs_request();
        req.live_write_readiness_safe = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-06"));
    }

    #[test]
    fn gate_contract_blocked_when_orchestrator_not_present() {
        let mut req = all_prereqs_request();
        req.restore_orchestrator_present = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-07"));
    }

    #[test]
    fn gate_contract_blocked_when_schema_executor_not_present() {
        let mut req = all_prereqs_request();
        req.schema_executor_present = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-08"));
    }

    #[test]
    fn gate_contract_blocked_when_record_executor_not_present() {
        let mut req = all_prereqs_request();
        req.record_executor_present = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-09"));
    }

    #[test]
    fn gate_contract_blocked_when_linked_executor_not_present() {
        let mut req = all_prereqs_request();
        req.linked_executor_present = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-10"));
    }

    #[test]
    fn gate_contract_blocked_when_final_validation_reader_not_present() {
        let mut req = all_prereqs_request();
        req.final_validation_reader_present = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert_eq!(result.status, SandboxGateContractStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("SGC-PRE-11"));
    }

    // ── No production mode ────────────────────────────────────────────────────

    #[test]
    fn gate_contract_no_production_mode_exists() {
        let disabled = SandboxGateContractMode::Disabled;
        let candidate = SandboxGateContractMode::SandboxOnlyCandidate;
        assert_ne!(disabled, candidate);
        let json = serde_json::to_string(&disabled).expect("serialize");
        assert!(!json.contains("production"));
        let json = serde_json::to_string(&candidate).expect("serialize");
        assert!(!json.contains("production"));
    }

    // ── Prerequisite structure ────────────────────────────────────────────────

    #[test]
    fn gate_contract_prereq_count_is_twelve_when_eligible() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        assert_eq!(result.total_prereq_count, 12);
        assert_eq!(result.safe_prereq_count, 12);
        assert_eq!(result.prerequisites.len(), 12);
    }

    #[test]
    fn gate_contract_prereq_ordering_deterministic() {
        let r1 = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let r2 = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let ids1: Vec<_> = r1.prerequisites.iter().map(|p| &p.prereq_id).collect();
        let ids2: Vec<_> = r2.prerequisites.iter().map(|p| &p.prereq_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn gate_contract_prereq_ids_use_sgc_prefix() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        for prereq in &result.prerequisites {
            assert!(
                prereq.prereq_id.starts_with("SGC-PRE-"),
                "prereq_id must start with SGC-PRE-, got: {}",
                prereq.prereq_id
            );
        }
    }

    #[test]
    fn gate_contract_first_prereq_is_sandbox() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        assert_eq!(result.prerequisites[0].prereq_id, "SGC-PRE-01");
    }

    #[test]
    fn gate_contract_last_prereq_is_write_gate_default() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let last = result
            .prerequisites
            .last()
            .expect("prerequisites not empty");
        assert_eq!(last.prereq_id, "SGC-PRE-12");
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn gate_contract_no_success_state_introduced() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        assert!(!result.writes_enabled);
        assert!(!result.reads_enabled);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("\"armed\""));
        assert!(!json.contains("\"enabled\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn gate_contract_no_token_in_result() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn gate_contract_no_absolute_path_in_result() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn gate_contract_no_record_payload_in_result() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn gate_contract_no_attachment_url_in_result() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn gate_contract_no_old_or_new_record_id_in_result() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn gate_contract_no_airtable_client_called() {
        // evaluate_sandbox_gate_contract accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        assert_eq!(
            result.status,
            SandboxGateContractStatus::EligibleButNotArmed
        );
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn gate_contract_no_network_in_blocked_state() {
        let mut req = all_prereqs_request();
        req.sandbox_verification_safe = false;
        let result = evaluate_sandbox_gate_contract(&req);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn gate_contract_no_network_in_disabled_state() {
        let result = evaluate_sandbox_gate_contract(&disabled_request());
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn gate_contract_total_and_safe_counts_consistent() {
        let result = evaluate_sandbox_gate_contract(&all_prereqs_request());
        assert_eq!(result.total_prereq_count, result.prerequisites.len());
        let actual_safe = result
            .prerequisites
            .iter()
            .filter(|p| {
                matches!(
                    p.status,
                    SandboxGateContractPrerequisiteStatus::Safe
                        | SandboxGateContractPrerequisiteStatus::Warning
                )
            })
            .count();
        assert_eq!(result.safe_prereq_count, actual_safe);
    }
}
