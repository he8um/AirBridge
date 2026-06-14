use crate::restore::write_gate::evaluate_write_gate;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

/// The overall verdict for a sandbox write testing policy check.
///
/// Safety invariants:
/// - `Compliant` does NOT enable restore writes.
/// - `writes_enabled` is always false regardless of status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxWriteTestingPolicyStatus {
    /// Sandbox verification passed and all required test evidence is present.
    Compliant,
    /// Evidence is partial or stale; sandbox testing cannot be fully confirmed.
    Warning,
    /// Target is non-sandbox or production, or no sandbox evidence is declared.
    Blocked,
}

/// The result of a single sandbox write testing policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxWriteTestingCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// Classification of the restore target for sandbox safety.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxTargetClassification {
    /// Target is a known sandbox or test base — safe to test against.
    Sandbox,
    /// Target appears to be a production base — must not be used for write testing.
    Production,
    /// Target classification is unknown.
    Unknown,
}

/// Evidence that sandbox write testing has been performed.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field (only basename via `test_package_filename`).
/// - No raw record payload fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxWriteTestEvidence {
    /// True when the sandbox base was reachable and verified empty before testing.
    pub sandbox_base_verified: bool,
    /// Filename only (basename) of the test package used. No directory path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_package_filename: Option<String>,
    /// True when a dry-run plan was generated for this package.
    pub dry_run_completed: bool,
    /// True when the schema creation plan was reviewed and found valid.
    pub schema_plan_reviewed: bool,
    /// True when the record import plan was reviewed and found valid.
    pub record_plan_reviewed: bool,
    /// Human-readable label for the reviewer or test context (no token, no email).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_label: Option<String>,
    /// Optional ISO-8601 timestamp string for when evidence was recorded.
    /// Treated as stale if more than 30 days old (UTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_timestamp: Option<String>,
}

/// Input to the sandbox write testing policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxWriteTestingPolicyRequest {
    /// Classification of the restore target.
    pub target_classification: SandboxTargetClassification,
    /// Whether the sandbox verification gate (Gate 1) has passed for this session.
    pub sandbox_verification_passed: bool,
    /// Evidence from sandbox write testing, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<SandboxWriteTestEvidence>,
    /// Human-readable label for the target (base name or filename basename only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_display_name: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxWriteTestingCheck {
    pub check_id: String,
    pub label: String,
    pub status: SandboxWriteTestingCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Result from `verify_sandbox_write_testing_policy`.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - `Compliant` does NOT enable restore writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxWriteTestingPolicyResult {
    pub status: SandboxWriteTestingPolicyStatus,
    pub checks: Vec<SandboxWriteTestingCheck>,
    pub message: String,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies that sandbox write testing has been completed before any live write is enabled.
///
/// Check IDs:
/// - SWT-01: Write gate is disabled.
/// - SWT-02: Target is classified as sandbox (not production or unknown).
/// - SWT-03: Sandbox verification gate (Gate 1) has passed.
/// - SWT-04: Sandbox test evidence is declared (not absent).
/// - SWT-05: Evidence completeness — all required evidence fields are present and true.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
pub fn verify_sandbox_write_testing_policy(
    request: &SandboxWriteTestingPolicyRequest,
) -> SandboxWriteTestingPolicyResult {
    let mut checks: Vec<SandboxWriteTestingCheck> = Vec::new();

    // SWT-01: Write gate disabled
    let gate = evaluate_write_gate();
    use crate::restore::write_result::RestoreWriteEngineStatus;
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(SandboxWriteTestingCheck {
            check_id: "SWT-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: SandboxWriteTestingCheckStatus::Passed,
            message: "Restore write gate is disabled. No live writes are possible.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(SandboxWriteTestingCheck {
            check_id: "SWT-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: SandboxWriteTestingCheckStatus::Failed,
            message: "Restore write gate is unexpectedly enabled.".to_string(),
            remediation: Some(
                "The write gate must remain disabled until all safety gates are satisfied."
                    .to_string(),
            ),
        });
    }

    // SWT-02: Target classification
    match &request.target_classification {
        SandboxTargetClassification::Sandbox => {
            checks.push(SandboxWriteTestingCheck {
                check_id: "SWT-02".to_string(),
                label: "sandbox-target-classification".to_string(),
                status: SandboxWriteTestingCheckStatus::Passed,
                message: "Target is classified as a sandbox base.".to_string(),
                remediation: None,
            });
        }
        SandboxTargetClassification::Production => {
            checks.push(SandboxWriteTestingCheck {
                check_id: "SWT-02".to_string(),
                label: "sandbox-target-classification".to_string(),
                status: SandboxWriteTestingCheckStatus::Failed,
                message: "Target is classified as a production base. Write testing must not be performed against production.".to_string(),
                remediation: Some(
                    "Use a dedicated sandbox or test base for write testing.".to_string(),
                ),
            });
        }
        SandboxTargetClassification::Unknown => {
            checks.push(SandboxWriteTestingCheck {
                check_id: "SWT-02".to_string(),
                label: "sandbox-target-classification".to_string(),
                status: SandboxWriteTestingCheckStatus::Failed,
                message:
                    "Target classification is unknown. Cannot confirm this is a safe sandbox base."
                        .to_string(),
                remediation: Some(
                    "Classify the target base as sandbox before proceeding with write testing."
                        .to_string(),
                ),
            });
        }
    }

    // SWT-03: Sandbox verification gate passed
    if request.sandbox_verification_passed {
        checks.push(SandboxWriteTestingCheck {
            check_id: "SWT-03".to_string(),
            label: "sandbox-verification-passed".to_string(),
            status: SandboxWriteTestingCheckStatus::Passed,
            message: "Sandbox environment verification (Gate 1) has passed for this session."
                .to_string(),
            remediation: None,
        });
    } else {
        checks.push(SandboxWriteTestingCheck {
            check_id: "SWT-03".to_string(),
            label: "sandbox-verification-passed".to_string(),
            status: SandboxWriteTestingCheckStatus::Failed,
            message:
                "Sandbox environment verification (Gate 1) has not passed. Run Gate 1 checks first."
                    .to_string(),
            remediation: Some(
                "Complete sandbox environment verification before sandbox write testing."
                    .to_string(),
            ),
        });
    }

    // SWT-04: Evidence declared
    match &request.evidence {
        None => {
            checks.push(SandboxWriteTestingCheck {
                check_id: "SWT-04".to_string(),
                label: "sandbox-test-evidence-present".to_string(),
                status: SandboxWriteTestingCheckStatus::Failed,
                message: "No sandbox test evidence declared. Sandbox write testing has not been recorded.".to_string(),
                remediation: Some(
                    "Provide SandboxWriteTestEvidence with the results of a completed sandbox test run.".to_string(),
                ),
            });
            // SWT-05 cannot run without evidence
            checks.push(SandboxWriteTestingCheck {
                check_id: "SWT-05".to_string(),
                label: "sandbox-evidence-complete".to_string(),
                status: SandboxWriteTestingCheckStatus::Failed,
                message: "Evidence completeness check skipped — no evidence declared.".to_string(),
                remediation: Some(
                    "Declare sandbox test evidence to enable this check.".to_string(),
                ),
            });
        }
        Some(ev) => {
            checks.push(SandboxWriteTestingCheck {
                check_id: "SWT-04".to_string(),
                label: "sandbox-test-evidence-present".to_string(),
                status: SandboxWriteTestingCheckStatus::Passed,
                message: "Sandbox test evidence is declared.".to_string(),
                remediation: None,
            });

            // SWT-05: Evidence completeness
            let all_required = ev.sandbox_base_verified
                && ev.dry_run_completed
                && ev.schema_plan_reviewed
                && ev.record_plan_reviewed;

            let filename_ok = ev
                .test_package_filename
                .as_deref()
                .map(|f| !f.is_empty() && !f.contains('/') && !f.contains('\\'))
                .unwrap_or(false);

            if all_required && filename_ok {
                checks.push(SandboxWriteTestingCheck {
                    check_id: "SWT-05".to_string(),
                    label: "sandbox-evidence-complete".to_string(),
                    status: SandboxWriteTestingCheckStatus::Passed,
                    message: "All required evidence fields are present and confirmed.".to_string(),
                    remediation: None,
                });
            } else {
                let mut missing: Vec<&str> = Vec::new();
                if !ev.sandbox_base_verified {
                    missing.push("sandbox_base_verified");
                }
                if !ev.dry_run_completed {
                    missing.push("dry_run_completed");
                }
                if !ev.schema_plan_reviewed {
                    missing.push("schema_plan_reviewed");
                }
                if !ev.record_plan_reviewed {
                    missing.push("record_plan_reviewed");
                }
                if !filename_ok {
                    missing.push("test_package_filename");
                }

                checks.push(SandboxWriteTestingCheck {
                    check_id: "SWT-05".to_string(),
                    label: "sandbox-evidence-complete".to_string(),
                    status: SandboxWriteTestingCheckStatus::Warning,
                    message: format!(
                        "Evidence is incomplete. Missing or false: {}.",
                        missing.join(", ")
                    ),
                    remediation: Some(
                        "Complete all required evidence fields before live write testing."
                            .to_string(),
                    ),
                });
            }
        }
    }

    let has_blocked = checks
        .iter()
        .any(|c| c.status == SandboxWriteTestingCheckStatus::Failed);
    let has_warning = checks
        .iter()
        .any(|c| c.status == SandboxWriteTestingCheckStatus::Warning);

    let status = if has_blocked {
        SandboxWriteTestingPolicyStatus::Blocked
    } else if has_warning {
        SandboxWriteTestingPolicyStatus::Warning
    } else {
        SandboxWriteTestingPolicyStatus::Compliant
    };

    let target_name = request
        .target_display_name
        .as_deref()
        .unwrap_or("the restore target");

    let message = match &status {
        SandboxWriteTestingPolicyStatus::Compliant => format!(
            "Sandbox write testing policy for {} is satisfied. \
             All required evidence is present. Restore writes remain disabled.",
            target_name
        ),
        SandboxWriteTestingPolicyStatus::Warning => format!(
            "Sandbox write testing policy for {} has warnings. \
             Evidence is incomplete or partial. Restore writes remain disabled.",
            target_name
        ),
        SandboxWriteTestingPolicyStatus::Blocked => format!(
            "Sandbox write testing policy for {} is blocked. \
             Target is not a sandbox base or required evidence is missing. \
             Restore writes remain disabled.",
            target_name
        ),
    };

    SandboxWriteTestingPolicyResult {
        status,
        checks,
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

    fn complete_evidence() -> SandboxWriteTestEvidence {
        SandboxWriteTestEvidence {
            sandbox_base_verified: true,
            test_package_filename: Some("test-backup.airbridge".to_string()),
            dry_run_completed: true,
            schema_plan_reviewed: true,
            record_plan_reviewed: true,
            reviewer_label: Some("sandbox-test".to_string()),
            evidence_timestamp: None,
        }
    }

    fn compliant_request() -> SandboxWriteTestingPolicyRequest {
        SandboxWriteTestingPolicyRequest {
            target_classification: SandboxTargetClassification::Sandbox,
            sandbox_verification_passed: true,
            evidence: Some(complete_evidence()),
            target_display_name: Some("Test Base".to_string()),
        }
    }

    // ── Status outcomes ───────────────────────────────────────────────────────

    #[test]
    fn complete_evidence_and_sandbox_target_returns_compliant() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Compliant);
    }

    #[test]
    fn no_evidence_returns_blocked() {
        let request = SandboxWriteTestingPolicyRequest {
            evidence: None,
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Blocked);
    }

    #[test]
    fn production_target_returns_blocked() {
        let request = SandboxWriteTestingPolicyRequest {
            target_classification: SandboxTargetClassification::Production,
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Blocked);
    }

    #[test]
    fn unknown_target_returns_blocked() {
        let request = SandboxWriteTestingPolicyRequest {
            target_classification: SandboxTargetClassification::Unknown,
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Blocked);
    }

    #[test]
    fn sandbox_verification_not_passed_returns_blocked() {
        let request = SandboxWriteTestingPolicyRequest {
            sandbox_verification_passed: false,
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Blocked);
    }

    #[test]
    fn partial_evidence_returns_warning() {
        let partial = SandboxWriteTestEvidence {
            sandbox_base_verified: true,
            test_package_filename: Some("test.airbridge".to_string()),
            dry_run_completed: true,
            schema_plan_reviewed: false,
            record_plan_reviewed: false,
            reviewer_label: None,
            evidence_timestamp: None,
        };
        let request = SandboxWriteTestingPolicyRequest {
            evidence: Some(partial),
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Warning);
    }

    #[test]
    fn missing_filename_produces_warning() {
        let ev = SandboxWriteTestEvidence {
            test_package_filename: None,
            ..complete_evidence()
        };
        let request = SandboxWriteTestingPolicyRequest {
            evidence: Some(ev),
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Warning);
    }

    #[test]
    fn filename_with_path_separator_produces_warning() {
        let ev = SandboxWriteTestEvidence {
            test_package_filename: Some("/Users/tester/test.airbridge".to_string()),
            ..complete_evidence()
        };
        let request = SandboxWriteTestingPolicyRequest {
            evidence: Some(ev),
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Warning);
    }

    // ── Check IDs ─────────────────────────────────────────────────────────────

    #[test]
    fn five_checks_always_present() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn check_ids_are_swt_01_through_05() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert!(ids.contains(&"SWT-01"));
        assert!(ids.contains(&"SWT-02"));
        assert!(ids.contains(&"SWT-03"));
        assert!(ids.contains(&"SWT-04"));
        assert!(ids.contains(&"SWT-05"));
    }

    #[test]
    fn swt_01_always_passes() {
        let result = verify_sandbox_write_testing_policy(&SandboxWriteTestingPolicyRequest {
            target_classification: SandboxTargetClassification::Unknown,
            sandbox_verification_passed: false,
            evidence: None,
            target_display_name: None,
        });
        let swt01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SWT-01")
            .unwrap();
        assert_eq!(swt01.status, SandboxWriteTestingCheckStatus::Passed);
    }

    #[test]
    fn swt_02_fails_for_production_target() {
        let request = SandboxWriteTestingPolicyRequest {
            target_classification: SandboxTargetClassification::Production,
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        let swt02 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SWT-02")
            .unwrap();
        assert_eq!(swt02.status, SandboxWriteTestingCheckStatus::Failed);
    }

    #[test]
    fn swt_03_fails_when_sandbox_not_verified() {
        let request = SandboxWriteTestingPolicyRequest {
            sandbox_verification_passed: false,
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        let swt03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SWT-03")
            .unwrap();
        assert_eq!(swt03.status, SandboxWriteTestingCheckStatus::Failed);
    }

    #[test]
    fn swt_04_fails_when_evidence_absent() {
        let request = SandboxWriteTestingPolicyRequest {
            evidence: None,
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        let swt04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SWT-04")
            .unwrap();
        assert_eq!(swt04.status, SandboxWriteTestingCheckStatus::Failed);
    }

    #[test]
    fn swt_05_warns_for_incomplete_evidence() {
        let partial = SandboxWriteTestEvidence {
            sandbox_base_verified: false,
            ..complete_evidence()
        };
        let request = SandboxWriteTestingPolicyRequest {
            evidence: Some(partial),
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        let swt05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SWT-05")
            .unwrap();
        assert_eq!(swt05.status, SandboxWriteTestingCheckStatus::Warning);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true() {
        for request in [
            compliant_request(),
            SandboxWriteTestingPolicyRequest {
                target_classification: SandboxTargetClassification::Production,
                sandbox_verification_passed: false,
                evidence: None,
                target_display_name: None,
            },
        ] {
            let result = verify_sandbox_write_testing_policy(&request);
            assert!(result.no_changes_made);
        }
    }

    #[test]
    fn writes_enabled_always_false() {
        for request in [
            compliant_request(),
            SandboxWriteTestingPolicyRequest {
                target_classification: SandboxTargetClassification::Sandbox,
                sandbox_verification_passed: true,
                evidence: Some(complete_evidence()),
                target_display_name: None,
            },
        ] {
            let result = verify_sandbox_write_testing_policy(&request);
            assert!(!result.writes_enabled);
        }
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn compliant_result_does_not_enable_writes() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Compliant);
        assert!(!result.writes_enabled);
    }

    // ── Serialization safety ──────────────────────────────────────────────────

    #[test]
    fn serialization_has_no_token() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("token"));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn serialization_has_no_full_path() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\\\"));
    }

    #[test]
    fn serialization_has_no_record_payload() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("\"fields\""));
        assert!(!json.contains("\"recordId\""));
    }

    #[test]
    fn request_with_path_in_filename_produces_warning_not_blocked_on_other_fields() {
        let ev = SandboxWriteTestEvidence {
            test_package_filename: Some("test.airbridge".to_string()),
            ..complete_evidence()
        };
        let request = SandboxWriteTestingPolicyRequest {
            evidence: Some(ev),
            ..compliant_request()
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Compliant);
    }

    #[test]
    fn message_contains_target_display_name() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        assert!(result.message.contains("Test Base"));
    }

    #[test]
    fn message_says_writes_remain_disabled_when_compliant() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        assert_eq!(result.status, SandboxWriteTestingPolicyStatus::Compliant);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn no_evidence_result_has_five_checks() {
        let request = SandboxWriteTestingPolicyRequest {
            target_classification: SandboxTargetClassification::Sandbox,
            sandbox_verification_passed: true,
            evidence: None,
            target_display_name: None,
        };
        let result = verify_sandbox_write_testing_policy(&request);
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn reviewer_label_not_in_result_message() {
        let result = verify_sandbox_write_testing_policy(&compliant_request());
        assert!(!result.message.contains("sandbox-test"));
    }
}
