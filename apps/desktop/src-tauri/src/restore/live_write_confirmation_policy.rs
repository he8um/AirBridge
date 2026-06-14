use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

/// The overall verdict for the live-write-specific confirmation policy check.
///
/// Safety invariants:
/// - `Confirmed` does NOT enable restore writes.
/// - `writes_enabled` is always false regardless of status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveWriteConfirmationPolicyStatus {
    /// Confirmation text matched and all prior gate prerequisites are satisfied.
    Confirmed,
    /// One or more prior gates are in warning state; confirmation text still
    /// matched but operator should review warnings before proceeding.
    Warning,
    /// A hard prerequisite (prior gate blocked, target unsafe) prevents
    /// confirmation from being meaningful.
    Blocked,
    /// Confirmation text did not match the required phrase.
    Rejected,
}

/// The result of a single policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveWriteConfirmationCheckStatus {
    Passed,
    Warning,
    Failed,
}

// ── Data structs ──────────────────────────────────────────────────────────────

/// Prior-gate status summary passed in from the frontend.
///
/// All fields are safe strings — no token, no path, no record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorGateStatuses {
    /// Gate 1 sandbox verification status: "verified", "warning", or "blocked".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_verification_status: Option<String>,
    /// Gate 4 destructive operation policy status: "compliant", "warning", or "blocked".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_operation_policy_status: Option<String>,
    /// Gate 5 attachment upload policy status: "compliant", "warning", or "blocked".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_upload_policy_status: Option<String>,
    /// Gate 6 schema record order policy status: "compliant", "warning", or "blocked".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_record_order_policy_status: Option<String>,
    /// Gate 7 sandbox write testing policy status: "compliant", "warning", or "blocked".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_write_testing_policy_status: Option<String>,
}

/// Input to the live-write confirmation policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteConfirmationPolicyRequest {
    /// The text the user typed in the confirmation input.
    pub entered_text: String,
    /// Safe display label for the restore target (base name, no path, no token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
    /// Prior gate statuses used to check prerequisites.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_gate_statuses: Option<PriorGateStatuses>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteConfirmationCheck {
    pub check_id: String,
    pub label: String,
    pub status: LiveWriteConfirmationCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Result from `verify_live_write_confirmation_policy`.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - `Confirmed` does NOT enable restore writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWriteConfirmationPolicyResult {
    pub status: LiveWriteConfirmationPolicyStatus,
    pub checks: Vec<LiveWriteConfirmationCheck>,
    /// The exact phrase the user must type to confirm.
    pub required_text: String,
    pub message: String,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Required-text builder ─────────────────────────────────────────────────────

/// Builds the exact confirmation phrase the user must type for the live-write gate.
///
/// Format: `"LIVE RESTORE <TARGET> — WRITES REMAIN DISABLED"`
///
/// - Uses `target_label` if provided, sanitised.
/// - Falls back to `"TARGET"` if no label.
/// - Never includes a filesystem path, token, or sensitive value.
pub fn build_live_write_confirmation_text(target_label: Option<&str>) -> String {
    let safe_target = target_label
        .map(sanitize_label)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "TARGET".to_string());
    format!("LIVE RESTORE {} — WRITES REMAIN DISABLED", safe_target)
}

/// Strips unsafe characters and truncates to 64 chars.
fn sanitize_label(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == ' ')
        .collect();
    cleaned
        .chars()
        .take(64)
        .collect::<String>()
        .trim()
        .to_string()
}

// ── Helper: check a prior gate status string ──────────────────────────────────

/// Returns (is_blocked, has_warning) for a gate status string.
/// `None` / `"unknown"` / not-yet-run → not blocked, not warning.
/// `"blocked"` → blocked.
/// `"warning"` → warning.
/// `"compliant"` / `"verified"` → ok.
fn gate_status_flags(status: Option<&str>) -> (bool, bool) {
    match status {
        Some("blocked") => (true, false),
        Some("warning") => (false, true),
        _ => (false, false),
    }
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the live-write-specific user confirmation policy.
///
/// Check IDs:
/// - LWC-01: Write gate is disabled.
/// - LWC-02: Prior safety gates are not blocked.
/// - LWC-03: Sandbox write testing gate (Gate 7) is not blocked.
/// - LWC-04: Confirmation text matches the required phrase exactly.
/// - LWC-05: Confirmed status does not enable writes.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
pub fn verify_live_write_confirmation_policy(
    request: &LiveWriteConfirmationPolicyRequest,
) -> LiveWriteConfirmationPolicyResult {
    let required_text = build_live_write_confirmation_text(request.target_label.as_deref());
    let mut checks: Vec<LiveWriteConfirmationCheck> = Vec::new();

    // LWC-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: LiveWriteConfirmationCheckStatus::Passed,
            message: "Restore write gate is disabled. No live writes are possible.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: LiveWriteConfirmationCheckStatus::Failed,
            message: "Restore write gate is unexpectedly enabled.".to_string(),
            remediation: Some(
                "The write gate must remain disabled until all safety gates are satisfied."
                    .to_string(),
            ),
        });
    }

    // LWC-02: Prior safety gates (1–6) not blocked
    let gates = request.prior_gate_statuses.as_ref();
    let (sandbox_blocked, sandbox_warn) =
        gate_status_flags(gates.and_then(|g| g.sandbox_verification_status.as_deref()));
    let (dop_blocked, dop_warn) =
        gate_status_flags(gates.and_then(|g| g.destructive_operation_policy_status.as_deref()));
    let (aup_blocked, aup_warn) =
        gate_status_flags(gates.and_then(|g| g.attachment_upload_policy_status.as_deref()));
    let (sro_blocked, sro_warn) =
        gate_status_flags(gates.and_then(|g| g.schema_record_order_policy_status.as_deref()));

    let any_prior_blocked = sandbox_blocked || dop_blocked || aup_blocked || sro_blocked;
    let any_prior_warn = sandbox_warn || dop_warn || aup_warn || sro_warn;

    if any_prior_blocked {
        let mut blocked_names: Vec<&str> = Vec::new();
        if sandbox_blocked {
            blocked_names.push("sandbox-verification");
        }
        if dop_blocked {
            blocked_names.push("destructive-operation-policy");
        }
        if aup_blocked {
            blocked_names.push("attachment-upload-policy");
        }
        if sro_blocked {
            blocked_names.push("schema-record-order-policy");
        }
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-02".to_string(),
            label: "prior-gates-not-blocked".to_string(),
            status: LiveWriteConfirmationCheckStatus::Failed,
            message: format!(
                "Prior safety gate(s) are blocked: {}. Resolve before confirming.",
                blocked_names.join(", ")
            ),
            remediation: Some(
                "Resolve all blocked prior gates before attempting live-write confirmation."
                    .to_string(),
            ),
        });
    } else if any_prior_warn {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-02".to_string(),
            label: "prior-gates-not-blocked".to_string(),
            status: LiveWriteConfirmationCheckStatus::Warning,
            message: "Prior safety gates have warnings. Review before proceeding.".to_string(),
            remediation: Some(
                "Resolve prior-gate warnings where possible before confirming live writes."
                    .to_string(),
            ),
        });
    } else {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-02".to_string(),
            label: "prior-gates-not-blocked".to_string(),
            status: LiveWriteConfirmationCheckStatus::Passed,
            message: "Prior safety gates are not blocked.".to_string(),
            remediation: None,
        });
    }

    // LWC-03: Sandbox write testing gate (Gate 7) not blocked
    let (swt_blocked, swt_warn) =
        gate_status_flags(gates.and_then(|g| g.sandbox_write_testing_policy_status.as_deref()));

    if swt_blocked {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-03".to_string(),
            label: "sandbox-write-testing-not-blocked".to_string(),
            status: LiveWriteConfirmationCheckStatus::Failed,
            message: "Sandbox write testing policy (Gate 7) is blocked. Resolve before confirming."
                .to_string(),
            remediation: Some(
                "Complete sandbox write testing with required evidence before confirming live writes."
                    .to_string(),
            ),
        });
    } else if swt_warn {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-03".to_string(),
            label: "sandbox-write-testing-not-blocked".to_string(),
            status: LiveWriteConfirmationCheckStatus::Warning,
            message:
                "Sandbox write testing policy (Gate 7) has warnings. Review evidence completeness."
                    .to_string(),
            remediation: Some(
                "Complete all required sandbox test evidence fields before confirming live writes."
                    .to_string(),
            ),
        });
    } else {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-03".to_string(),
            label: "sandbox-write-testing-not-blocked".to_string(),
            status: LiveWriteConfirmationCheckStatus::Passed,
            message: "Sandbox write testing policy (Gate 7) is not blocked.".to_string(),
            remediation: None,
        });
    }

    // LWC-04: Confirmation text match (case-sensitive, trim outer whitespace only)
    let entered = request.entered_text.trim();
    let text_matches = entered == required_text.as_str();

    if text_matches {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-04".to_string(),
            label: "confirmation-text-match".to_string(),
            status: LiveWriteConfirmationCheckStatus::Passed,
            message: "Confirmation text matches the required phrase.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(LiveWriteConfirmationCheck {
            check_id: "LWC-04".to_string(),
            label: "confirmation-text-match".to_string(),
            status: LiveWriteConfirmationCheckStatus::Failed,
            message: "Confirmation text does not match the required phrase. Text must match exactly, case-sensitively.".to_string(),
            remediation: Some(format!("Type exactly: {}", required_text)),
        });
    }

    // LWC-05: Writes remain disabled
    checks.push(LiveWriteConfirmationCheck {
        check_id: "LWC-05".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: LiveWriteConfirmationCheckStatus::Passed,
        message: "Restore write execution is not enabled. Confirming this phrase does not start any write operation.".to_string(),
        remediation: None,
    });

    // ── Compute overall status ────────────────────────────────────────────────

    let has_blocked = checks
        .iter()
        .any(|c| c.status == LiveWriteConfirmationCheckStatus::Failed);
    let has_warning = checks
        .iter()
        .any(|c| c.status == LiveWriteConfirmationCheckStatus::Warning);

    // A blocked prerequisite overrides everything.
    let status = if has_blocked {
        // If text matched but gates are blocked → Blocked (not Rejected)
        if !text_matches {
            LiveWriteConfirmationPolicyStatus::Rejected
        } else {
            LiveWriteConfirmationPolicyStatus::Blocked
        }
    } else if !text_matches {
        LiveWriteConfirmationPolicyStatus::Rejected
    } else if has_warning {
        LiveWriteConfirmationPolicyStatus::Warning
    } else {
        LiveWriteConfirmationPolicyStatus::Confirmed
    };

    let target_name = request
        .target_label
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("the restore target");

    let message = match &status {
        LiveWriteConfirmationPolicyStatus::Confirmed => format!(
            "Live-write confirmation for {} accepted. \
             Required phrase matched. Restore writes remain disabled — \
             confirmation does not enable live writes.",
            target_name
        ),
        LiveWriteConfirmationPolicyStatus::Warning => format!(
            "Live-write confirmation for {} accepted with warnings. \
             Required phrase matched, but prior gates have unresolved warnings. \
             Restore writes remain disabled.",
            target_name
        ),
        LiveWriteConfirmationPolicyStatus::Blocked => format!(
            "Live-write confirmation for {} is blocked. \
             One or more prior safety gates are blocked. \
             Restore writes remain disabled.",
            target_name
        ),
        LiveWriteConfirmationPolicyStatus::Rejected => format!(
            "Live-write confirmation for {} rejected. \
             Confirmation text did not match the required phrase. \
             Restore writes remain disabled.",
            target_name
        ),
    };

    LiveWriteConfirmationPolicyResult {
        status,
        checks,
        required_text,
        message,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_gates_ok() -> PriorGateStatuses {
        PriorGateStatuses {
            sandbox_verification_status: Some("verified".to_string()),
            destructive_operation_policy_status: Some("compliant".to_string()),
            attachment_upload_policy_status: Some("compliant".to_string()),
            schema_record_order_policy_status: Some("compliant".to_string()),
            sandbox_write_testing_policy_status: Some("compliant".to_string()),
        }
    }

    fn required_text_for(label: &str) -> String {
        build_live_write_confirmation_text(Some(label))
    }

    fn confirmed_request(target: &str) -> LiveWriteConfirmationPolicyRequest {
        LiveWriteConfirmationPolicyRequest {
            entered_text: required_text_for(target),
            target_label: Some(target.to_string()),
            prior_gate_statuses: Some(all_gates_ok()),
        }
    }

    // ── Status outcomes ───────────────────────────────────────────────────────

    #[test]
    fn exact_match_all_gates_ok_returns_confirmed() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Confirmed);
    }

    #[test]
    fn wrong_text_returns_rejected() {
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: "wrong text".to_string(),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(all_gates_ok()),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Rejected);
    }

    #[test]
    fn wrong_case_returns_rejected() {
        let req_text = required_text_for("My Base");
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: req_text.to_lowercase(),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(all_gates_ok()),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Rejected);
    }

    #[test]
    fn partial_text_returns_rejected() {
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: "LIVE RESTORE".to_string(),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(all_gates_ok()),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Rejected);
    }

    #[test]
    fn extra_words_returns_rejected() {
        let req_text = required_text_for("My Base");
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: format!("{} EXTRA", req_text),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(all_gates_ok()),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Rejected);
    }

    #[test]
    fn blocked_prior_gate_returns_blocked() {
        let mut gates = all_gates_ok();
        gates.sandbox_verification_status = Some("blocked".to_string());
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: required_text_for("My Base"),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(gates),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Blocked);
    }

    #[test]
    fn blocked_dop_gate_returns_blocked() {
        let mut gates = all_gates_ok();
        gates.destructive_operation_policy_status = Some("blocked".to_string());
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: required_text_for("My Base"),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(gates),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Blocked);
    }

    #[test]
    fn blocked_sandbox_testing_gate_returns_blocked_even_with_correct_text() {
        let mut gates = all_gates_ok();
        gates.sandbox_write_testing_policy_status = Some("blocked".to_string());
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: required_text_for("My Base"),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(gates),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Blocked);
    }

    #[test]
    fn warning_prior_gate_with_correct_text_returns_warning() {
        let mut gates = all_gates_ok();
        gates.sandbox_verification_status = Some("warning".to_string());
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: required_text_for("My Base"),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(gates),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Warning);
    }

    #[test]
    fn no_prior_gate_statuses_with_correct_text_returns_confirmed() {
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: required_text_for("My Base"),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: None,
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Confirmed);
    }

    // ── Check IDs ─────────────────────────────────────────────────────────────

    #[test]
    fn five_checks_always_present() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn check_ids_are_lwc_01_through_05() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert!(ids.contains(&"LWC-01"));
        assert!(ids.contains(&"LWC-02"));
        assert!(ids.contains(&"LWC-03"));
        assert!(ids.contains(&"LWC-04"));
        assert!(ids.contains(&"LWC-05"));
    }

    #[test]
    fn lwc_01_always_passes() {
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: "wrong".to_string(),
            target_label: None,
            prior_gate_statuses: None,
        };
        let result = verify_live_write_confirmation_policy(&request);
        let lwc01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "LWC-01")
            .unwrap();
        assert_eq!(lwc01.status, LiveWriteConfirmationCheckStatus::Passed);
    }

    #[test]
    fn lwc_05_always_passes() {
        for req in [
            confirmed_request("My Base"),
            LiveWriteConfirmationPolicyRequest {
                entered_text: "wrong".to_string(),
                target_label: None,
                prior_gate_statuses: None,
            },
        ] {
            let result = verify_live_write_confirmation_policy(&req);
            let lwc05 = result
                .checks
                .iter()
                .find(|c| c.check_id == "LWC-05")
                .unwrap();
            assert_eq!(lwc05.status, LiveWriteConfirmationCheckStatus::Passed);
        }
    }

    // ── Required text ─────────────────────────────────────────────────────────

    #[test]
    fn required_text_contains_writes_remain_disabled() {
        let text = build_live_write_confirmation_text(Some("My Base"));
        assert!(text.contains("WRITES REMAIN DISABLED"));
    }

    #[test]
    fn required_text_contains_live_restore() {
        let text = build_live_write_confirmation_text(Some("My Base"));
        assert!(text.starts_with("LIVE RESTORE "));
    }

    #[test]
    fn required_text_includes_uppercased_target() {
        let text = build_live_write_confirmation_text(Some("My Base"));
        assert!(text.contains("MY BASE"));
    }

    #[test]
    fn required_text_fallback_when_no_target() {
        let text = build_live_write_confirmation_text(None);
        assert!(text.contains("TARGET"));
        assert!(text.contains("WRITES REMAIN DISABLED"));
    }

    #[test]
    fn sanitize_strips_path_separators() {
        let text = build_live_write_confirmation_text(Some("/Users/test/base"));
        assert!(!text.contains('/'));
        assert!(!text.contains("Users"));
    }

    #[test]
    fn result_contains_required_text_field() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        assert!(!result.required_text.is_empty());
        assert!(result.required_text.contains("WRITES REMAIN DISABLED"));
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true() {
        for req in [
            confirmed_request("My Base"),
            LiveWriteConfirmationPolicyRequest {
                entered_text: "wrong".to_string(),
                target_label: None,
                prior_gate_statuses: None,
            },
        ] {
            let result = verify_live_write_confirmation_policy(&req);
            assert!(result.no_changes_made);
        }
    }

    #[test]
    fn writes_enabled_always_false() {
        for req in [
            confirmed_request("My Base"),
            LiveWriteConfirmationPolicyRequest {
                entered_text: required_text_for("My Base"),
                target_label: Some("My Base".to_string()),
                prior_gate_statuses: Some(all_gates_ok()),
            },
        ] {
            let result = verify_live_write_confirmation_policy(&req);
            assert!(!result.writes_enabled);
        }
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn confirmed_result_does_not_enable_writes() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Confirmed);
        assert!(!result.writes_enabled);
    }

    // ── Serialization safety ──────────────────────────────────────────────────

    #[test]
    fn serialization_has_no_token() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
    }

    #[test]
    fn serialization_has_no_full_path() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn serialization_has_no_record_payload() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("\"fields\""));
        assert!(!json.contains("\"recordId\""));
    }

    #[test]
    fn message_does_not_contain_token() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        assert!(!result.message.contains("token"));
        assert!(!result.message.contains("pat_"));
    }

    #[test]
    fn message_says_writes_remain_disabled_when_confirmed() {
        let result = verify_live_write_confirmation_policy(&confirmed_request("My Base"));
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Confirmed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn blocked_prior_gate_with_wrong_text_returns_rejected_not_blocked() {
        let mut gates = all_gates_ok();
        gates.sandbox_verification_status = Some("blocked".to_string());
        let request = LiveWriteConfirmationPolicyRequest {
            entered_text: "wrong text".to_string(),
            target_label: Some("My Base".to_string()),
            prior_gate_statuses: Some(gates),
        };
        let result = verify_live_write_confirmation_policy(&request);
        assert_eq!(result.status, LiveWriteConfirmationPolicyStatus::Rejected);
    }
}
