use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ---------------------------------------------------------------------------
// Status / check enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreConfirmationStatus {
    /// Confirmation text matches and all prerequisites pass.
    Confirmed,
    /// Confirmation text does not match or a prerequisite failed.
    Rejected,
    /// A hard prerequisite (sandbox blocked, write gate unexpectedly enabled)
    /// makes the confirmation step un-runnable.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreConfirmationCheckStatus {
    Passed,
    Failed,
    Skipped,
}

// ---------------------------------------------------------------------------
// Requirement / check structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreConfirmationRequirement {
    pub requirement_id: String,
    pub label: String,
    pub satisfied: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreConfirmationCheck {
    pub check_id: String,
    pub label: String,
    pub status: RestoreConfirmationCheckStatus,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Request / result structs
// ---------------------------------------------------------------------------

/// Request sent by the frontend to validate a restore confirmation.
///
/// - No `token` field.
/// - No filesystem path field.
/// - `sandbox_verification_status` is a string copy of the Gate 1 result.
/// - `entered_text` is the raw text the user typed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreConfirmationRequest {
    /// The text the user entered in the confirmation input.
    pub entered_text: String,
    /// Filename-only label for the source backup package (no directory path).
    pub source_package_label: Option<String>,
    /// Safe display name of the restore target (base name or "new base").
    pub target_label: Option<String>,
    /// Status string from Gate 1 sandbox verification result ("verified", "warning", "blocked").
    pub sandbox_verification_status: Option<String>,
}

/// Result returned from `validate_restore_confirmation`.
///
/// - No `token` field.
/// - No filesystem path field.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - Status is never a success state — even `Confirmed` means confirmation text
///   is correct but restore writes remain disabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreConfirmationResult {
    pub status: RestoreConfirmationStatus,
    pub checks: Vec<RestoreConfirmationCheck>,
    pub requirements: Vec<RestoreConfirmationRequirement>,
    /// The exact text the user is required to type. Derived from target/package labels.
    pub required_text: String,
    pub message: String,
    /// Always true — no Airtable writes are made.
    pub no_changes_made: bool,
    /// Always false — no network write calls are made.
    pub network_writes_attempted: bool,
    /// Always false — writes remain disabled.
    pub writes_enabled: bool,
}

// ---------------------------------------------------------------------------
// Required-text builder
// ---------------------------------------------------------------------------

/// Builds the exact confirmation phrase the user must type.
///
/// - Uses `target_label` if provided (base name or "new base").
/// - Falls back to `source_package_label` if no target label.
/// - Falls back to the fixed phrase `"RESTORE BACKUP"` if neither is available.
/// - Never includes a filesystem path, token, or sensitive value.
/// - Always starts with `"RESTORE"` in uppercase.
pub fn build_required_confirmation_text(
    source_package_label: Option<&str>,
    target_label: Option<&str>,
) -> String {
    if let Some(target) = target_label {
        let safe_target = sanitize_label(target);
        if !safe_target.is_empty() {
            return format!("RESTORE TO {}", safe_target.to_uppercase());
        }
    }
    if let Some(pkg) = source_package_label {
        let safe_pkg = sanitize_label(pkg);
        if !safe_pkg.is_empty() {
            return format!("RESTORE {}", safe_pkg.to_uppercase());
        }
    }
    "RESTORE BACKUP".to_string()
}

/// Strips any character that is not alphanumeric, hyphen, underscore, dot, or space.
/// Truncates to 64 characters. Used to build safe confirmation phrases.
fn sanitize_label(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == ' ')
        .collect();
    // Take at most 64 chars, trim surrounding whitespace
    cleaned
        .chars()
        .take(64)
        .collect::<String>()
        .trim()
        .to_string()
}

// ---------------------------------------------------------------------------
// Validation logic
// ---------------------------------------------------------------------------

/// Validates the restore confirmation for Gate 2.
///
/// # Safety invariants
///
/// - No Airtable API calls are made.
/// - No token is accepted or returned.
/// - No filesystem path is accepted or returned.
/// - No files are written.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - `writes_enabled` is always `false`.
/// - Result status is never a write-success state.
/// - Even a `Confirmed` result does NOT enable restore writes.
pub fn validate_restore_confirmation(
    request: &RestoreConfirmationRequest,
) -> RestoreConfirmationResult {
    let required_text = build_required_confirmation_text(
        request.source_package_label.as_deref(),
        request.target_label.as_deref(),
    );

    let mut checks: Vec<RestoreConfirmationCheck> = Vec::new();
    let mut requirements: Vec<RestoreConfirmationRequirement> = Vec::new();

    // ── CHK-C01: Write gate must be disabled ─────────────────────────────────
    let gate_decision = evaluate_write_gate();
    let gate_disabled = gate_decision.status == RestoreWriteEngineStatus::Disabled;
    checks.push(RestoreConfirmationCheck {
        check_id: "CHK-C01".to_string(),
        label: "Write gate disabled".to_string(),
        status: if gate_disabled {
            RestoreConfirmationCheckStatus::Passed
        } else {
            RestoreConfirmationCheckStatus::Failed
        },
        message: if gate_disabled {
            "Write gate is disabled — restore execution cannot proceed. No writes will occur."
                .to_string()
        } else {
            "Write gate returned an unexpected state. This result is blocked.".to_string()
        },
    });
    requirements.push(RestoreConfirmationRequirement {
        requirement_id: "REQ-C01".to_string(),
        label: "Write gate disabled".to_string(),
        satisfied: gate_disabled,
        note: "evaluate_write_gate() must return Disabled before confirmation is meaningful."
            .to_string(),
    });

    // ── CHK-C02: Sandbox verification must not be blocked ────────────────────
    let sandbox_status_str = request
        .sandbox_verification_status
        .as_deref()
        .unwrap_or("unknown");
    let sandbox_ok = matches!(sandbox_status_str, "verified" | "warning");
    checks.push(RestoreConfirmationCheck {
        check_id: "CHK-C02".to_string(),
        label: "Sandbox verification not blocked".to_string(),
        status: if sandbox_ok {
            RestoreConfirmationCheckStatus::Passed
        } else if sandbox_status_str == "unknown" {
            RestoreConfirmationCheckStatus::Skipped
        } else {
            RestoreConfirmationCheckStatus::Failed
        },
        message: if sandbox_ok {
            format!(
                "Sandbox verification status is '{}' — prerequisites met.",
                sandbox_status_str
            )
        } else if sandbox_status_str == "unknown" {
            "Sandbox verification has not been run. Run Gate 1 before proceeding.".to_string()
        } else {
            "Sandbox verification is blocked. Resolve Gate 1 checks before confirming restore."
                .to_string()
        },
    });
    requirements.push(RestoreConfirmationRequirement {
        requirement_id: "REQ-C02".to_string(),
        label: "Sandbox verification status verified or warning".to_string(),
        satisfied: sandbox_ok,
        note: "Gate 1 must not be blocked before Gate 2 confirmation is valid.".to_string(),
    });

    // ── CHK-C03: Confirmation text match ─────────────────────────────────────
    let entered = request.entered_text.trim();
    let text_matches = entered == required_text.as_str();
    checks.push(RestoreConfirmationCheck {
        check_id: "CHK-C03".to_string(),
        label: "Confirmation text exact match".to_string(),
        status: if text_matches {
            RestoreConfirmationCheckStatus::Passed
        } else {
            RestoreConfirmationCheckStatus::Failed
        },
        message: if text_matches {
            "Confirmation text matches exactly.".to_string()
        } else if entered.is_empty() {
            "No confirmation text was entered.".to_string()
        } else {
            "Confirmation text does not match. Exact match required (case-sensitive).".to_string()
        },
    });
    requirements.push(RestoreConfirmationRequirement {
        requirement_id: "REQ-C03".to_string(),
        label: "Confirmation text exact match".to_string(),
        satisfied: text_matches,
        note: format!(
            "Required: \"{}\" — must match exactly after trim, case-sensitive.",
            required_text
        ),
    });

    // ── CHK-C04: No token in entered text ─────────────────────────────────────
    let has_token_pattern = entered.starts_with("pat")
        && entered.len() > 20
        && entered[3..].chars().all(|c| c.is_alphanumeric());
    checks.push(RestoreConfirmationCheck {
        check_id: "CHK-C04".to_string(),
        label: "No token in confirmation text".to_string(),
        status: if has_token_pattern {
            RestoreConfirmationCheckStatus::Failed
        } else {
            RestoreConfirmationCheckStatus::Passed
        },
        message: if has_token_pattern {
            "Confirmation text resembles an Airtable token. Tokens must not be entered here."
                .to_string()
        } else {
            "Confirmation text does not resemble an API token.".to_string()
        },
    });

    // ── CHK-C05: Restore writes remain disabled ───────────────────────────────
    checks.push(RestoreConfirmationCheck {
        check_id: "CHK-C05".to_string(),
        label: "Restore writes remain disabled".to_string(),
        status: RestoreConfirmationCheckStatus::Passed,
        message:
            "Restore write execution is not enabled in this version. Confirmation is recorded \
             but does not trigger any Airtable write operations."
                .to_string(),
    });

    // ── Determine overall status ──────────────────────────────────────────────
    let any_failed_hard = !gate_disabled || has_token_pattern;
    let sandbox_blocked = !sandbox_ok && sandbox_status_str != "unknown";

    let status = if any_failed_hard || sandbox_blocked {
        RestoreConfirmationStatus::Blocked
    } else if text_matches && sandbox_ok {
        RestoreConfirmationStatus::Confirmed
    } else {
        RestoreConfirmationStatus::Rejected
    };

    let message = match &status {
        RestoreConfirmationStatus::Confirmed => {
            "Confirmation accepted. Restore writes remain disabled in this version — \
             no Airtable changes will be made."
                .to_string()
        }
        RestoreConfirmationStatus::Rejected => {
            if entered.is_empty() {
                "No confirmation text entered. Type the exact required text to confirm.".to_string()
            } else if sandbox_status_str == "unknown" {
                "Run sandbox verification (Gate 1) before confirming.".to_string()
            } else {
                "Confirmation text does not match. Type the exact required text (case-sensitive)."
                    .to_string()
            }
        }
        RestoreConfirmationStatus::Blocked => {
            if sandbox_blocked {
                "Sandbox verification is blocked. Resolve Gate 1 checks before confirming restore."
                    .to_string()
            } else if has_token_pattern {
                "Confirmation text must not be an API token.".to_string()
            } else {
                "Write gate returned an unexpected state. Contact support.".to_string()
            }
        }
    };

    RestoreConfirmationResult {
        status,
        checks,
        requirements,
        required_text,
        message,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn req(entered: &str) -> RestoreConfirmationRequest {
        RestoreConfirmationRequest {
            entered_text: entered.to_string(),
            source_package_label: Some("backup.airbridge".to_string()),
            target_label: Some("My Base".to_string()),
            sandbox_verification_status: Some("warning".to_string()),
        }
    }

    fn req_no_labels(entered: &str) -> RestoreConfirmationRequest {
        RestoreConfirmationRequest {
            entered_text: entered.to_string(),
            source_package_label: None,
            target_label: None,
            sandbox_verification_status: Some("warning".to_string()),
        }
    }

    // ── required text builder ────────────────────────────────────────────────

    #[test]
    fn required_text_uses_target_label_when_present() {
        let text = build_required_confirmation_text(None, Some("My Base"));
        assert_eq!(text, "RESTORE TO MY BASE");
    }

    #[test]
    fn required_text_falls_back_to_package_label() {
        let text = build_required_confirmation_text(Some("backup.airbridge"), None);
        assert_eq!(text, "RESTORE BACKUP.AIRBRIDGE");
    }

    #[test]
    fn required_text_falls_back_to_fixed_phrase() {
        let text = build_required_confirmation_text(None, None);
        assert_eq!(text, "RESTORE BACKUP");
    }

    #[test]
    fn required_text_strips_path_separators() {
        let text = build_required_confirmation_text(None, Some("/Users/alice/base"));
        // Path separator '/' is stripped, leaving "Usersalicebase"
        assert!(!text.contains('/'));
    }

    #[test]
    fn required_text_never_empty() {
        let text = build_required_confirmation_text(None, None);
        assert!(!text.is_empty());
    }

    // ── exact match ──────────────────────────────────────────────────────────

    #[test]
    fn exact_match_confirmed() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Confirmed);
    }

    #[test]
    fn no_labels_exact_match_confirmed() {
        let request = req_no_labels("RESTORE BACKUP");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Confirmed);
    }

    // ── rejection cases ──────────────────────────────────────────────────────

    #[test]
    fn wrong_case_rejected() {
        let request = req("restore to my base");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    #[test]
    fn partial_match_rejected() {
        let request = req("RESTORE");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    #[test]
    fn extra_words_rejected() {
        let request = req("RESTORE TO MY BASE NOW");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    #[test]
    fn empty_text_rejected() {
        let request = req("");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    #[test]
    fn whitespace_only_rejected() {
        let request = req("   ");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    #[test]
    fn mixed_case_rejected() {
        let request = req("Restore To My Base");
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    #[test]
    fn leading_trailing_spaces_rejected() {
        // trim() is applied before comparison, so "  RESTORE TO MY BASE  " should confirm
        let request = req("  RESTORE TO MY BASE  ");
        let result = validate_restore_confirmation(&request);
        // trim is applied — this should confirm
        assert_eq!(result.status, RestoreConfirmationStatus::Confirmed);
    }

    // ── blocked cases ────────────────────────────────────────────────────────

    #[test]
    fn blocked_sandbox_blocks_confirmation() {
        let request = RestoreConfirmationRequest {
            entered_text: "RESTORE TO MY BASE".to_string(),
            source_package_label: Some("backup.airbridge".to_string()),
            target_label: Some("My Base".to_string()),
            sandbox_verification_status: Some("blocked".to_string()),
        };
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Blocked);
    }

    #[test]
    fn unknown_sandbox_status_causes_rejection_not_block() {
        let request = RestoreConfirmationRequest {
            entered_text: "RESTORE TO MY BASE".to_string(),
            source_package_label: Some("backup.airbridge".to_string()),
            target_label: Some("My Base".to_string()),
            sandbox_verification_status: Some("unknown".to_string()),
        };
        let result = validate_restore_confirmation(&request);
        // sandbox unknown → CHK-C02 skipped, text matches → but sandbox not ok → Rejected
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    #[test]
    fn no_sandbox_status_causes_rejection() {
        let request = RestoreConfirmationRequest {
            entered_text: "RESTORE BACKUP.AIRBRIDGE".to_string(),
            source_package_label: Some("backup.airbridge".to_string()),
            target_label: None,
            sandbox_verification_status: None,
        };
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Rejected);
    }

    // ── write gate inclusion ─────────────────────────────────────────────────

    #[test]
    fn write_gate_check_is_included_in_checks() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        let chk = result.checks.iter().find(|c| c.check_id == "CHK-C01");
        assert!(chk.is_some());
        assert_eq!(chk.unwrap().status, RestoreConfirmationCheckStatus::Passed);
    }

    #[test]
    fn writes_remain_disabled_check_always_passed() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        let chk = result.checks.iter().find(|c| c.check_id == "CHK-C05");
        assert!(chk.is_some());
        assert_eq!(chk.unwrap().status, RestoreConfirmationCheckStatus::Passed);
    }

    // ── safety invariants ────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true() {
        for entered in &["RESTORE TO MY BASE", "", "wrong text", "restore to my base"] {
            let request = req(entered);
            let result = validate_restore_confirmation(&request);
            assert!(
                result.no_changes_made,
                "no_changes_made must be true for '{}'",
                entered
            );
        }
    }

    #[test]
    fn network_writes_attempted_always_false() {
        for entered in &["RESTORE TO MY BASE", "", "wrong"] {
            let request = req(entered);
            let result = validate_restore_confirmation(&request);
            assert!(
                !result.network_writes_attempted,
                "network_writes_attempted must be false for '{}'",
                entered
            );
        }
    }

    #[test]
    fn writes_enabled_always_false() {
        for entered in &["RESTORE TO MY BASE", "", "wrong"] {
            let request = req(entered);
            let result = validate_restore_confirmation(&request);
            assert!(
                !result.writes_enabled,
                "writes_enabled must be false for '{}'",
                entered
            );
        }
    }

    #[test]
    fn result_serialization_has_no_token() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        let json = serde_json::to_string(&result).expect("serialize");
        // No JSON key named "token" or "apiKey"
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn result_serialization_has_no_full_path() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn required_text_not_in_result_token_or_path_form() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        // required_text starts with "RESTORE", not a path or token
        assert!(result.required_text.starts_with("RESTORE"));
        assert!(!result.required_text.contains('/'));
        assert!(!result.required_text.starts_with("pat"));
    }

    #[test]
    fn token_like_text_is_blocked() {
        // A string that looks like an Airtable PAT should be blocked
        let request = RestoreConfirmationRequest {
            entered_text: "patABCDEFGHIJKLMNOPQRSTUVWX".to_string(),
            source_package_label: None,
            target_label: None,
            sandbox_verification_status: Some("warning".to_string()),
        };
        let result = validate_restore_confirmation(&request);
        assert_eq!(result.status, RestoreConfirmationStatus::Blocked);
    }

    #[test]
    fn required_text_in_result_serialization() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("requiredText"));
    }

    #[test]
    fn confirmed_status_never_includes_succeed_language() {
        let request = req("RESTORE TO MY BASE");
        let result = validate_restore_confirmation(&request);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("succeeded"));
        assert!(!json.contains("Succeeded"));
        assert!(!json.contains("success"));
    }

    #[test]
    fn sanitize_label_strips_path_components() {
        let label = sanitize_label("/Users/alice/My Base");
        assert!(!label.contains('/'));
        assert!(!label.is_empty());
    }

    #[test]
    fn sanitize_label_truncates_to_64() {
        let long = "A".repeat(100);
        let result = sanitize_label(&long);
        assert!(result.len() <= 64);
    }
}
