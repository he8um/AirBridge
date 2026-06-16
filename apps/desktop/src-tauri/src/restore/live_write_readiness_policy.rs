use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

/// The overall verdict for the live-write readiness aggregate policy.
///
/// Safety invariants:
/// - `Ready` does NOT enable restore writes.
/// - `writesEnabled` is always `false` regardless of status.
/// - No Airtable API calls are made.
/// - No token, full path, record payload, attachment URL, or raw HTTP data
///   appears in any result field.
/// - This result is advisory only. It does not start any write execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveWriteReadinessPolicyStatus {
    /// All required safety gates are declared and none are failed.
    /// Writes remain disabled; this is advisory only.
    Ready,
    /// At least one gate has a warning, but no gate is failed or missing.
    /// Writes remain disabled; this is advisory only.
    Warning,
    /// At least one required gate is missing, failed, not evaluated,
    /// or a hard safety invariant is violated.
    Blocked,
}

/// The result of a single readiness check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveWriteReadinessCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// The status of an individual upstream gate in the aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveWriteReadinessGateStatus {
    Passed,
    Warning,
    Failed,
    NotEvaluated,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// Declares the known status of one upstream safety gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteReadinessGate {
    /// Machine-readable gate identifier (e.g. `"sandboxEnvironment"`).
    pub gate_id: String,
    /// Human-readable label (e.g. `"Sandbox environment"`).
    pub label: String,
    /// The reported status of this gate.
    pub status: LiveWriteReadinessGateStatus,
    /// Optional note from the caller about this gate's current state.
    pub note: Option<String>,
}

/// Summary of gate counts included in the result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteReadinessSummary {
    pub total_gates: usize,
    pub passed_gates: usize,
    pub warning_gates: usize,
    pub failed_gates: usize,
    pub not_evaluated_gates: usize,
    pub missing_required_gates: usize,
    pub all_required_gates_declared: bool,
    pub live_execution_available: bool,
}

/// One individual check run as part of the readiness policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteReadinessCheck {
    pub check_id: String,
    pub label: String,
    pub status: LiveWriteReadinessCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Request for the live-write readiness aggregate policy.
///
/// Safety invariants:
/// - No `token` field.
/// - No `base_id` or `workspace_id` field that could trigger a network call.
/// - No attachment URL field.
/// - No record payload field.
/// - No full filesystem path field.
/// - No raw HTTP field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteReadinessPolicyRequest {
    /// The known statuses of each upstream gate.
    pub gates: Option<Vec<LiveWriteReadinessGate>>,
    /// Set to `true` to indicate that live write execution is currently
    /// available in the build (always causes `Blocked`).
    pub live_execution_available: Option<bool>,
    /// Optional label for the target base (display only; never a real ID).
    pub target_label: Option<String>,
}

/// Result of the live-write readiness aggregate policy.
///
/// Safety invariants:
/// - No `token` field.
/// - No filesystem path field.
/// - No attachment URL field.
/// - No record payload field.
/// - No raw HTTP data field.
/// - `writesEnabled` is always `false`.
/// - `noChangesMade` is always `true`.
/// - `networkWritesAttempted` is always `false`.
/// - `Ready` does NOT enable restore writes.
/// - `Ready` does NOT introduce a restore success state.
/// - This result is advisory only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteReadinessPolicyResult {
    pub status: LiveWriteReadinessPolicyStatus,
    pub checks: Vec<LiveWriteReadinessCheck>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_summary: Option<LiveWriteReadinessSummary>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Required gate IDs ─────────────────────────────────────────────────────────

const REQUIRED_GATE_IDS: &[&str] = &[
    "sandboxEnvironment",
    "restoreConfirmation",
    "targetEmpty",
    "destructiveOperationPolicy",
    "attachmentUploadPolicy",
    "schemaRecordOrder",
    "sandboxWriteTesting",
    "liveWriteConfirmation",
    "rateLimitBackoff",
    "checkpointDurability",
    "finalValidationPlan",
    "writePhaseOrdering",
    "failureModes",
    "rollbackLimitation",
    "finalValidationEnforcement",
    "sensitiveDataSafety",
    "attachmentPhaseDisabled",
];

// ── Helper builders ───────────────────────────────────────────────────────────

fn passed(id: &str, label: &str, message: &str) -> LiveWriteReadinessCheck {
    LiveWriteReadinessCheck {
        check_id: id.to_string(),
        label: label.to_string(),
        status: LiveWriteReadinessCheckStatus::Passed,
        message: message.to_string(),
        remediation: None,
    }
}

fn warning_check(
    id: &str,
    label: &str,
    message: &str,
    remediation: &str,
) -> LiveWriteReadinessCheck {
    LiveWriteReadinessCheck {
        check_id: id.to_string(),
        label: label.to_string(),
        status: LiveWriteReadinessCheckStatus::Warning,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}

fn failed(id: &str, label: &str, message: &str, remediation: &str) -> LiveWriteReadinessCheck {
    LiveWriteReadinessCheck {
        check_id: id.to_string(),
        label: label.to_string(),
        status: LiveWriteReadinessCheckStatus::Failed,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}

fn build_result(
    status: LiveWriteReadinessPolicyStatus,
    checks: Vec<LiveWriteReadinessCheck>,
    message: String,
    gate_summary: Option<LiveWriteReadinessSummary>,
) -> LiveWriteReadinessPolicyResult {
    LiveWriteReadinessPolicyResult {
        status,
        checks,
        message,
        gate_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Core policy function ──────────────────────────────────────────────────────

/// Evaluates the live-write readiness aggregate policy.
///
/// This function runs 10 checks (LWR-01 through LWR-10). It aggregates
/// the results of all upstream safety gates and produces an advisory verdict.
///
/// Safety guarantees:
/// - No Airtable API calls are made.
/// - No token is required or accepted.
/// - No writes are performed.
/// - No network calls are made.
/// - No full filesystem path appears in any result field.
/// - No record payload appears in any result field.
/// - No attachment URL appears in any result field.
/// - No raw HTTP data appears in any result field.
/// - `Ready` does NOT enable restore writes.
/// - `Ready` does NOT introduce a restore success state.
/// - `writesEnabled` is always `false`.
/// - This result is advisory only.
pub fn verify_live_write_readiness_policy(
    request: &LiveWriteReadinessPolicyRequest,
) -> LiveWriteReadinessPolicyResult {
    let mut checks = Vec::new();

    // LWR-01: Write gate disabled (always passes — gate always returns Disabled)
    let gate = evaluate_write_gate();
    let gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if gate_disabled {
        checks.push(passed(
            "LWR-01",
            "write-gate-disabled",
            "Write gate is disabled. No restore writes are attempted by this policy check.",
        ));
    } else {
        checks.push(failed(
            "LWR-01",
            "write-gate-disabled",
            "Write gate is unexpectedly enabled. The live-write readiness policy must not be \
             evaluated while writes are enabled.",
            "Disable the write gate before evaluating the readiness policy.",
        ));
        return build_result(
            LiveWriteReadinessPolicyStatus::Blocked,
            checks,
            "Live-write readiness policy is blocked. Write gate is unexpectedly enabled. \
             Restore writes remain disabled."
                .to_string(),
            None,
        );
    }

    // LWR-02: All required gates declared
    let gates = match &request.gates {
        Some(g) if !g.is_empty() => g,
        _ => {
            checks.push(failed(
                "LWR-02",
                "all-required-gates-declared",
                "No gate statuses were provided. All 17 required safety gates must be declared \
                 before readiness can be assessed.",
                "Provide a gates array containing the status of every required safety gate.",
            ));
            return build_result(
                LiveWriteReadinessPolicyStatus::Blocked,
                checks,
                "Live-write readiness policy is blocked. No gates were declared. All required \
                 safety gates must be evaluated before any live write path is considered. \
                 Restore writes remain disabled."
                    .to_string(),
                None,
            );
        }
    };

    let declared_ids: std::collections::HashSet<&str> =
        gates.iter().map(|g| g.gate_id.as_str()).collect();

    let missing: Vec<&str> = REQUIRED_GATE_IDS
        .iter()
        .filter(|id| !declared_ids.contains(*id))
        .copied()
        .collect();

    if missing.is_empty() {
        checks.push(passed(
            "LWR-02",
            "all-required-gates-declared",
            "All 17 required safety gates are declared.",
        ));
    } else {
        checks.push(failed(
            "LWR-02",
            "all-required-gates-declared",
            &format!(
                "{} required gate(s) are not declared: {}.",
                missing.len(),
                missing.join(", ")
            ),
            "Declare the missing gates with an appropriate status before assessing readiness.",
        ));
        let summary = build_summary(
            gates,
            &missing,
            request.live_execution_available.unwrap_or(false),
        );
        return build_result(
            LiveWriteReadinessPolicyStatus::Blocked,
            checks,
            format!(
                "Live-write readiness policy is blocked. {} required gate(s) are missing. \
                 Restore writes remain disabled.",
                missing.len()
            ),
            Some(summary),
        );
    }

    let mut blocked = false;
    let mut has_warning = false;

    // LWR-03: No failed required gate
    let failed_gates: Vec<&LiveWriteReadinessGate> = gates
        .iter()
        .filter(|g| {
            REQUIRED_GATE_IDS.contains(&g.gate_id.as_str())
                && matches!(g.status, LiveWriteReadinessGateStatus::Failed)
        })
        .collect();

    if failed_gates.is_empty() {
        checks.push(passed(
            "LWR-03",
            "no-failed-required-gate",
            "No required gate has a failed status.",
        ));
    } else {
        let names: Vec<&str> = failed_gates.iter().map(|g| g.gate_id.as_str()).collect();
        checks.push(failed(
            "LWR-03",
            "no-failed-required-gate",
            &format!(
                "{} required gate(s) have a failed status: {}.",
                names.len(),
                names.join(", ")
            ),
            "Resolve all failed gates before reassessing readiness.",
        ));
        blocked = true;
    }

    // LWR-04: Warnings summarized without enabling writes
    let warning_gates: Vec<&LiveWriteReadinessGate> = gates
        .iter()
        .filter(|g| {
            REQUIRED_GATE_IDS.contains(&g.gate_id.as_str())
                && matches!(g.status, LiveWriteReadinessGateStatus::Warning)
        })
        .collect();

    if warning_gates.is_empty() {
        checks.push(passed(
            "LWR-04",
            "warnings-summarized",
            "No required gate has a warning status. Writes remain disabled.",
        ));
    } else {
        let names: Vec<&str> = warning_gates.iter().map(|g| g.gate_id.as_str()).collect();
        checks.push(warning_check(
            "LWR-04",
            "warnings-summarized",
            &format!(
                "{} required gate(s) have a warning status: {}. \
                 Writes remain disabled. Warnings are advisory only.",
                names.len(),
                names.join(", ")
            ),
            "Review warning gates and resolve any issues before live write implementation.",
        ));
        has_warning = true;
    }

    // LWR-05: Live write execution remains unavailable
    let live_execution = request.live_execution_available.unwrap_or(false);
    if !live_execution {
        checks.push(passed(
            "LWR-05",
            "live-execution-unavailable",
            "Live write execution is not available. No restore execution path is enabled.",
        ));
    } else {
        checks.push(failed(
            "LWR-05",
            "live-execution-unavailable",
            "Live write execution is marked as available. This violates the write-disabled \
             product policy. No restore execution must be enabled at this stage.",
            "Disable live write execution before evaluating the readiness policy.",
        ));
        blocked = true;
    }

    // LWR-06: No restore success state introduced
    // This is an invariant check — the result itself never introduces a success state.
    // We verify that no gate carries a success-equivalent wording in its note.
    let success_gate = gates.iter().find(|g| {
        g.note.as_deref().map_or(false, |n| {
            let nl = n.to_lowercase();
            nl.contains("restore complete")
                || nl.contains("restore succeeded")
                || nl.contains("restore success")
                || nl.contains("writes enabled")
        })
    });
    if success_gate.is_none() {
        checks.push(passed(
            "LWR-06",
            "no-restore-success-state",
            "No gate note contains restore-success wording. Restore completion remains unavailable.",
        ));
    } else {
        checks.push(failed(
            "LWR-06",
            "no-restore-success-state",
            "At least one gate note contains restore-success equivalent wording. \
             Restore completion must remain unavailable.",
            "Remove any success-equivalent wording from gate notes.",
        ));
        blocked = true;
    }

    // LWR-07: No token/path/payload/URL/raw HTTP exposure
    // Invariant — the result itself never exposes these. We also check gate notes.
    let sensitive_gate = gates.iter().find(|g| {
        g.note.as_deref().map_or(false, |n| {
            let nl = n.to_lowercase();
            nl.contains("pat_")
                || nl.contains("apikey")
                || nl.contains("api_key")
                || nl.contains("bearer")
                || nl.contains("/users/")
                || nl.contains("/tmp/")
                || nl.contains("c:\\")
                || nl.contains("attachment_url")
                || nl.contains("attachmenturl")
        })
    });
    if sensitive_gate.is_none() {
        checks.push(passed(
            "LWR-07",
            "no-sensitive-exposure",
            "No sensitive data (token, path, payload, attachment URL, raw HTTP) is present \
             in any gate note.",
        ));
    } else {
        checks.push(failed(
            "LWR-07",
            "no-sensitive-exposure",
            "At least one gate note contains sensitive data material. \
             Tokens, paths, payloads, attachment URLs, and raw HTTP data must not appear.",
            "Remove all sensitive material from gate notes before evaluating readiness.",
        ));
        blocked = true;
    }

    // LWR-08: No dependency or external execution requirement
    // Not-evaluated gates in required positions block readiness.
    let not_evaluated: Vec<&LiveWriteReadinessGate> = gates
        .iter()
        .filter(|g| {
            REQUIRED_GATE_IDS.contains(&g.gate_id.as_str())
                && matches!(g.status, LiveWriteReadinessGateStatus::NotEvaluated)
        })
        .collect();

    if not_evaluated.is_empty() {
        checks.push(passed(
            "LWR-08",
            "no-unevaluated-required-gate",
            "All required gates have been evaluated. No external dependency is outstanding.",
        ));
    } else {
        let names: Vec<&str> = not_evaluated.iter().map(|g| g.gate_id.as_str()).collect();
        checks.push(failed(
            "LWR-08",
            "no-unevaluated-required-gate",
            &format!(
                "{} required gate(s) are not yet evaluated: {}.",
                names.len(),
                names.join(", ")
            ),
            "Evaluate all required gates before assessing overall readiness.",
        ));
        blocked = true;
    }

    // LWR-09: Future implementation is allowed only behind disabled gate
    // This is an invariant that the write gate always returns Disabled.
    // Re-checked here as a policy assertion.
    let gate2 = evaluate_write_gate();
    if matches!(gate2.status, RestoreWriteEngineStatus::Disabled) {
        checks.push(passed(
            "LWR-09",
            "future-implementation-behind-disabled-gate",
            "Any future live write implementation must remain behind the disabled write gate. \
             Write gate is currently disabled.",
        ));
    } else {
        checks.push(failed(
            "LWR-09",
            "future-implementation-behind-disabled-gate",
            "Write gate is not disabled. Future implementation must be behind the gate.",
            "Keep write gate disabled until all safety gates are satisfied and a full \
             live-write review is complete.",
        ));
        blocked = true;
    }

    // LWR-10: Readiness result is advisory only
    // This check always passes — the result carries writesEnabled=false unconditionally.
    checks.push(passed(
        "LWR-10",
        "readiness-result-advisory-only",
        "The readiness result is advisory only. A Ready status does not enable writes, \
         does not start any restore operation, and does not introduce a restore success state. \
         Restore writes remain disabled.",
    ));

    // Compute final status
    let summary = build_summary(gates, &[], live_execution);
    let status = if blocked {
        LiveWriteReadinessPolicyStatus::Blocked
    } else if has_warning {
        LiveWriteReadinessPolicyStatus::Warning
    } else {
        LiveWriteReadinessPolicyStatus::Ready
    };

    let message = match &status {
        LiveWriteReadinessPolicyStatus::Ready => {
            "Live-write readiness policy is satisfied. All 17 required safety gates are declared \
             and none are failed. This result is advisory only — restore writes remain disabled, \
             and a Ready status does not enable any restore execution."
                .to_string()
        }
        LiveWriteReadinessPolicyStatus::Warning => {
            "Live-write readiness policy has warnings. All required gates are declared and none \
             are failed, but at least one gate has a warning. This result is advisory only — \
             restore writes remain disabled."
                .to_string()
        }
        LiveWriteReadinessPolicyStatus::Blocked => {
            "Live-write readiness policy is blocked. One or more required safety gates are \
             missing, failed, not evaluated, or a hard safety invariant is violated. \
             Restore writes remain disabled."
                .to_string()
        }
    };

    build_result(status, checks, message, Some(summary))
}

fn build_summary(
    gates: &[LiveWriteReadinessGate],
    missing_required: &[&str],
    live_execution_available: bool,
) -> LiveWriteReadinessSummary {
    let required_declared: Vec<&LiveWriteReadinessGate> = gates
        .iter()
        .filter(|g| REQUIRED_GATE_IDS.contains(&g.gate_id.as_str()))
        .collect();

    let passed_count = required_declared
        .iter()
        .filter(|g| matches!(g.status, LiveWriteReadinessGateStatus::Passed))
        .count();
    let warning_count = required_declared
        .iter()
        .filter(|g| matches!(g.status, LiveWriteReadinessGateStatus::Warning))
        .count();
    let failed_count = required_declared
        .iter()
        .filter(|g| matches!(g.status, LiveWriteReadinessGateStatus::Failed))
        .count();
    let not_evaluated_count = required_declared
        .iter()
        .filter(|g| matches!(g.status, LiveWriteReadinessGateStatus::NotEvaluated))
        .count();

    LiveWriteReadinessSummary {
        total_gates: REQUIRED_GATE_IDS.len(),
        passed_gates: passed_count,
        warning_gates: warning_count,
        failed_gates: failed_count,
        not_evaluated_gates: not_evaluated_count,
        missing_required_gates: missing_required.len(),
        all_required_gates_declared: missing_required.is_empty(),
        live_execution_available,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_gates_passed() -> Vec<LiveWriteReadinessGate> {
        REQUIRED_GATE_IDS
            .iter()
            .map(|id| LiveWriteReadinessGate {
                gate_id: id.to_string(),
                label: id.to_string(),
                status: LiveWriteReadinessGateStatus::Passed,
                note: None,
            })
            .collect()
    }

    fn req_with_gates(gates: Vec<LiveWriteReadinessGate>) -> LiveWriteReadinessPolicyRequest {
        LiveWriteReadinessPolicyRequest {
            gates: Some(gates),
            live_execution_available: Some(false),
            target_label: None,
        }
    }

    fn gate(id: &str, status: LiveWriteReadinessGateStatus) -> LiveWriteReadinessGate {
        LiveWriteReadinessGate {
            gate_id: id.to_string(),
            label: id.to_string(),
            status,
            note: None,
        }
    }

    // ── Basic status ──────────────────────────────────────────────────────────

    #[test]
    fn all_gates_passed_returns_ready() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Ready);
    }

    #[test]
    fn one_warning_gate_returns_warning() {
        let mut gates = all_gates_passed();
        gates[0].status = LiveWriteReadinessGateStatus::Warning;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Warning);
    }

    #[test]
    fn one_failed_gate_returns_blocked() {
        let mut gates = all_gates_passed();
        gates[0].status = LiveWriteReadinessGateStatus::Failed;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn missing_required_gate_returns_blocked() {
        let mut gates = all_gates_passed();
        gates.retain(|g| g.gate_id != "sandboxEnvironment");
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn not_evaluated_gate_returns_blocked() {
        let mut gates = all_gates_passed();
        gates[0].status = LiveWriteReadinessGateStatus::NotEvaluated;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn no_gates_returns_blocked() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: None,
            live_execution_available: Some(false),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn empty_gates_list_returns_blocked() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: Some(vec![]),
            live_execution_available: Some(false),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn live_execution_available_returns_blocked() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: Some(all_gates_passed()),
            live_execution_available: Some(true),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn success_wording_in_gate_note_returns_blocked() {
        let mut gates = all_gates_passed();
        gates[0].note = Some("restore complete".to_string());
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn restore_succeeded_wording_in_gate_note_returns_blocked() {
        let mut gates = all_gates_passed();
        gates[1].note = Some("restore succeeded for this base".to_string());
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn writes_enabled_wording_in_gate_note_returns_blocked() {
        let mut gates = all_gates_passed();
        gates[2].note = Some("writes enabled now".to_string());
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn token_in_gate_note_returns_blocked() {
        let mut gates = all_gates_passed();
        gates[0].note = Some("pat_tokenvalue123".to_string());
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn full_path_in_gate_note_returns_blocked() {
        let mut gates = all_gates_passed();
        gates[0].note = Some("/Users/someone/backup.airbridge".to_string());
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn ready_result_writes_enabled_is_false() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Ready);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn blocked_result_writes_enabled_is_false() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: None,
            live_execution_available: Some(false),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn warning_result_writes_enabled_is_false() {
        let mut gates = all_gates_passed();
        gates[0].status = LiveWriteReadinessGateStatus::Warning;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Warning);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn blocked_no_changes_made_is_true() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: None,
            live_execution_available: Some(false),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn blocked_network_writes_attempted_is_false() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: None,
            live_execution_available: Some(false),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn ready_message_says_writes_remain_disabled() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert!(result
            .message
            .to_lowercase()
            .contains("writes remain disabled"));
    }

    #[test]
    fn ready_message_says_advisory_only() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert!(result.message.to_lowercase().contains("advisory only"));
    }

    #[test]
    fn blocked_message_says_writes_remain_disabled() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: None,
            live_execution_available: Some(false),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert!(result
            .message
            .to_lowercase()
            .contains("writes remain disabled"));
    }

    #[test]
    fn ready_message_does_not_say_restore_complete() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert!(!result.message.to_lowercase().contains("restore complete"));
        assert!(!result.message.to_lowercase().contains("succeeded"));
    }

    // ── Check counts ──────────────────────────────────────────────────────────

    #[test]
    fn no_gates_returns_two_checks() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: None,
            live_execution_available: Some(false),
            target_label: None,
        };
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn all_passed_returns_ten_checks() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.checks.len(), 10);
    }

    #[test]
    fn lwr_01_always_passes() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let check = result
            .checks
            .iter()
            .find(|c| c.check_id == "LWR-01")
            .unwrap();
        assert_eq!(check.status, LiveWriteReadinessCheckStatus::Passed);
    }

    #[test]
    fn lwr_10_always_passes() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let check = result
            .checks
            .iter()
            .find(|c| c.check_id == "LWR-10")
            .unwrap();
        assert_eq!(check.status, LiveWriteReadinessCheckStatus::Passed);
    }

    #[test]
    fn lwr_10_message_says_advisory_only() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let check = result
            .checks
            .iter()
            .find(|c| c.check_id == "LWR-10")
            .unwrap();
        assert!(check.message.to_lowercase().contains("advisory only"));
    }

    // ── Gate summary ──────────────────────────────────────────────────────────

    #[test]
    fn ready_result_has_gate_summary() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        assert!(result.gate_summary.is_some());
    }

    #[test]
    fn gate_summary_passed_count_correct() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let summary = result.gate_summary.unwrap();
        assert_eq!(summary.passed_gates, REQUIRED_GATE_IDS.len());
    }

    #[test]
    fn gate_summary_total_gates_is_seventeen() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let summary = result.gate_summary.unwrap();
        assert_eq!(summary.total_gates, 17);
    }

    #[test]
    fn gate_summary_all_required_gates_declared_true_when_all_present() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let summary = result.gate_summary.unwrap();
        assert!(summary.all_required_gates_declared);
    }

    #[test]
    fn gate_summary_live_execution_available_false() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let summary = result.gate_summary.unwrap();
        assert!(!summary.live_execution_available);
    }

    #[test]
    fn gate_summary_warning_count_correct() {
        let mut gates = all_gates_passed();
        gates[0].status = LiveWriteReadinessGateStatus::Warning;
        gates[1].status = LiveWriteReadinessGateStatus::Warning;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        let summary = result.gate_summary.unwrap();
        assert_eq!(summary.warning_gates, 2);
    }

    #[test]
    fn gate_summary_failed_count_correct() {
        let mut gates = all_gates_passed();
        gates[0].status = LiveWriteReadinessGateStatus::Failed;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        let summary = result.gate_summary.unwrap();
        assert_eq!(summary.failed_gates, 1);
    }

    // ── All required gate IDs are tracked ─────────────────────────────────────

    #[test]
    fn all_seventeen_required_gate_ids_present() {
        assert_eq!(REQUIRED_GATE_IDS.len(), 17);
        assert!(REQUIRED_GATE_IDS.contains(&"sandboxEnvironment"));
        assert!(REQUIRED_GATE_IDS.contains(&"attachmentPhaseDisabled"));
        assert!(REQUIRED_GATE_IDS.contains(&"sensitiveDataSafety"));
        assert!(REQUIRED_GATE_IDS.contains(&"finalValidationEnforcement"));
    }

    #[test]
    fn extra_non_required_gate_does_not_affect_status() {
        let mut gates = all_gates_passed();
        gates.push(LiveWriteReadinessGate {
            gate_id: "customExtraGate".to_string(),
            label: "Custom extra".to_string(),
            status: LiveWriteReadinessGateStatus::Failed,
            note: None,
        });
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        // Extra failed non-required gate does not block
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Ready);
    }

    #[test]
    fn missing_attachment_phase_disabled_gate_blocked() {
        let mut gates = all_gates_passed();
        gates.retain(|g| g.gate_id != "attachmentPhaseDisabled");
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    #[test]
    fn missing_sensitive_data_safety_gate_blocked() {
        let mut gates = all_gates_passed();
        gates.retain(|g| g.gate_id != "sensitiveDataSafety");
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
    }

    // ── Serialization safety ──────────────────────────────────────────────────

    #[test]
    fn result_serializes_without_token_field() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn result_serializes_without_path_field() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn result_serializes_without_attachment_url_field() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn status_serializes_as_camel_case() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"ready\""));
        assert!(!json.contains("\"Ready\""));
    }

    #[test]
    fn writes_enabled_field_is_false_in_serialized_output() {
        let req = req_with_gates(all_gates_passed());
        let result = verify_live_write_readiness_policy(&req);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"writesEnabled\":false"));
    }

    #[test]
    fn non_required_gate_with_gate_with_note_only_blocks_if_sensitive() {
        let mut gates = all_gates_passed();
        gates[0].note = Some("all checks passed normally".to_string());
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Ready);
    }

    #[test]
    fn target_label_does_not_affect_result() {
        let req = LiveWriteReadinessPolicyRequest {
            gates: Some(all_gates_passed()),
            live_execution_available: Some(false),
            target_label: Some("My Base".to_string()),
        };
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Ready);
    }

    #[test]
    fn multiple_failures_all_reported() {
        let mut gates = all_gates_passed();
        gates[0].status = LiveWriteReadinessGateStatus::Failed;
        gates[1].status = LiveWriteReadinessGateStatus::Failed;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
        let lwr03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "LWR-03")
            .unwrap();
        assert!(lwr03.message.contains("2 required gate(s)"));
    }

    #[test]
    fn single_warning_and_no_failure_returns_warning() {
        let mut gates = all_gates_passed();
        gates[5].status = LiveWriteReadinessGateStatus::Warning;
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Warning);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn gate_with_passed_status_returns_gate_id() {
        let gates = vec![gate(
            "sandboxEnvironment",
            LiveWriteReadinessGateStatus::Passed,
        )];
        // only one gate provided — missing remaining required gates causes blocked
        let req = req_with_gates(gates);
        let result = verify_live_write_readiness_policy(&req);
        assert_eq!(result.status, LiveWriteReadinessPolicyStatus::Blocked);
        let lwr02 = result
            .checks
            .iter()
            .find(|c| c.check_id == "LWR-02")
            .unwrap();
        assert_eq!(lwr02.status, LiveWriteReadinessCheckStatus::Failed);
    }
}
