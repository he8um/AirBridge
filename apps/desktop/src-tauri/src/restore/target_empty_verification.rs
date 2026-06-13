use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ---------------------------------------------------------------------------
// Status enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetEmptyVerificationStatus {
    Verified,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetEmptyVerificationCheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetEmptyFailureReason {
    UnsafeTargetMode,
    TargetHasTables,
    TargetHasRecords,
    CountsUnknown,
    WriteGateUnexpectedState,
}

// ---------------------------------------------------------------------------
// Data structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetEmptyVerificationCheck {
    pub check_id: String,
    pub label: String,
    pub status: TargetEmptyVerificationCheckStatus,
    pub message: String,
    pub remediation: Option<String>,
}

/// Request payload for target empty verification.
///
/// Safety invariants enforced by type:
/// - No token field.
/// - No full path field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetEmptyVerificationRequest {
    /// The intended target mode: "newBase" or "emptyExistingBase".
    pub target_mode: String,
    /// Known table count in the target base, if available.
    pub target_table_count: Option<u32>,
    /// Known record count across all tables in the target base, if available.
    pub target_record_count: Option<u32>,
    /// Optional display name of the target base (for the message only).
    pub target_display_name: Option<String>,
    /// Whether a live metadata check was performed to obtain the counts.
    pub live_check_performed: bool,
}

/// Result of target empty verification.
///
/// Safety invariants:
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetEmptyVerificationResult {
    pub status: TargetEmptyVerificationStatus,
    pub checks: Vec<TargetEmptyVerificationCheck>,
    pub message: String,
    /// Always true — no writes performed.
    pub no_changes_made: bool,
    /// Always false — no network write calls.
    pub network_writes_attempted: bool,
    /// Always false — write engine is disabled.
    pub writes_enabled: bool,
}

// ---------------------------------------------------------------------------
// Verification logic
// ---------------------------------------------------------------------------

pub fn verify_target_empty(
    request: &TargetEmptyVerificationRequest,
) -> TargetEmptyVerificationResult {
    let mut checks: Vec<TargetEmptyVerificationCheck> = Vec::new();

    // ------------------------------------------------------------------
    // TEV-01: write-gate
    // ------------------------------------------------------------------
    {
        let gate = evaluate_write_gate();
        if gate.status == RestoreWriteEngineStatus::Disabled {
            checks.push(TargetEmptyVerificationCheck {
                check_id: "TEV-01".to_string(),
                label: "write-gate".to_string(),
                status: TargetEmptyVerificationCheckStatus::Passed,
                message: "Write gate is disabled — no writes can be executed.".to_string(),
                remediation: None,
            });
        } else {
            checks.push(TargetEmptyVerificationCheck {
                check_id: "TEV-01".to_string(),
                label: "write-gate".to_string(),
                status: TargetEmptyVerificationCheckStatus::Failed,
                message: format!(
                    "Write gate returned an unexpected state. Expected Disabled. Gate message: {}",
                    gate.message
                ),
                remediation: Some(
                    "The write gate must return Disabled for target verification to pass."
                        .to_string(),
                ),
            });
        }
    }

    // ------------------------------------------------------------------
    // TEV-02: target-mode
    // ------------------------------------------------------------------
    let mode_safe = {
        let mode = request.target_mode.as_str();
        if mode == "newBase" || mode == "emptyExistingBase" {
            checks.push(TargetEmptyVerificationCheck {
                check_id: "TEV-02".to_string(),
                label: "target-mode".to_string(),
                status: TargetEmptyVerificationCheckStatus::Passed,
                message: format!("Target mode '{}' is supported.", request.target_mode),
                remediation: None,
            });
            true
        } else {
            checks.push(TargetEmptyVerificationCheck {
                check_id: "TEV-02".to_string(),
                label: "target-mode".to_string(),
                status: TargetEmptyVerificationCheckStatus::Failed,
                message: format!(
                    "Target mode '{}' is not supported. Only 'newBase' and 'emptyExistingBase' are allowed.",
                    request.target_mode
                ),
                remediation: Some(
                    "Set target_mode to 'newBase' or 'emptyExistingBase'.".to_string(),
                ),
            });
            false
        }
    };

    // ------------------------------------------------------------------
    // TEV-03: table-count
    // ------------------------------------------------------------------
    let table_count_ok = {
        match request.target_table_count {
            None => {
                // newBase intent: no tables can exist yet — treat as verified.
                if request.target_mode == "newBase" {
                    checks.push(TargetEmptyVerificationCheck {
                        check_id: "TEV-03".to_string(),
                        label: "table-count".to_string(),
                        status: TargetEmptyVerificationCheckStatus::Passed,
                        message: "New base target — no existing tables expected.".to_string(),
                        remediation: None,
                    });
                    true
                } else {
                    checks.push(TargetEmptyVerificationCheck {
                        check_id: "TEV-03".to_string(),
                        label: "table-count".to_string(),
                        status: TargetEmptyVerificationCheckStatus::Warning,
                        message: "Table count is not known. Live metadata check was not performed."
                            .to_string(),
                        remediation: Some(
                            "Perform a live metadata check to verify the target base is empty before enabling writes.".to_string(),
                        ),
                    });
                    false
                }
            }
            Some(0) => {
                checks.push(TargetEmptyVerificationCheck {
                    check_id: "TEV-03".to_string(),
                    label: "table-count".to_string(),
                    status: TargetEmptyVerificationCheckStatus::Passed,
                    message: "Target base has zero tables — safe to restore.".to_string(),
                    remediation: None,
                });
                true
            }
            Some(n) => {
                checks.push(TargetEmptyVerificationCheck {
                    check_id: "TEV-03".to_string(),
                    label: "table-count".to_string(),
                    status: TargetEmptyVerificationCheckStatus::Failed,
                    message: format!(
                        "Target base has {} table(s). Restoring into a non-empty base is not safe.",
                        n
                    ),
                    remediation: Some(
                        "Choose an empty base or create a new base as the restore target."
                            .to_string(),
                    ),
                });
                false
            }
        }
    };

    // ------------------------------------------------------------------
    // TEV-04: record-count
    // ------------------------------------------------------------------
    let record_count_ok = {
        match request.target_record_count {
            None => {
                if request.target_mode == "newBase" {
                    checks.push(TargetEmptyVerificationCheck {
                        check_id: "TEV-04".to_string(),
                        label: "record-count".to_string(),
                        status: TargetEmptyVerificationCheckStatus::Passed,
                        message: "New base target — no existing records expected.".to_string(),
                        remediation: None,
                    });
                    true
                } else {
                    checks.push(TargetEmptyVerificationCheck {
                        check_id: "TEV-04".to_string(),
                        label: "record-count".to_string(),
                        status: TargetEmptyVerificationCheckStatus::Warning,
                        message:
                            "Record count is not known. Live metadata check was not performed."
                                .to_string(),
                        remediation: Some(
                            "Perform a live metadata check to verify the target base is empty before enabling writes.".to_string(),
                        ),
                    });
                    false
                }
            }
            Some(0) => {
                checks.push(TargetEmptyVerificationCheck {
                    check_id: "TEV-04".to_string(),
                    label: "record-count".to_string(),
                    status: TargetEmptyVerificationCheckStatus::Passed,
                    message: "Target base has zero records — safe to restore.".to_string(),
                    remediation: None,
                });
                true
            }
            Some(n) => {
                checks.push(TargetEmptyVerificationCheck {
                    check_id: "TEV-04".to_string(),
                    label: "record-count".to_string(),
                    status: TargetEmptyVerificationCheckStatus::Failed,
                    message: format!(
                        "Target base has {} record(s). Restoring into a non-empty base is not safe.",
                        n
                    ),
                    remediation: Some(
                        "Choose an empty base or delete all records before restoring.".to_string(),
                    ),
                });
                false
            }
        }
    };

    // ------------------------------------------------------------------
    // TEV-05: no-writes-enabled
    // ------------------------------------------------------------------
    {
        checks.push(TargetEmptyVerificationCheck {
            check_id: "TEV-05".to_string(),
            label: "no-writes-enabled".to_string(),
            status: TargetEmptyVerificationCheckStatus::Passed,
            message: "Restore writes are not enabled. This check always passes in this version."
                .to_string(),
            remediation: None,
        });
    }

    // ------------------------------------------------------------------
    // Determine overall status
    // ------------------------------------------------------------------
    let any_hard_fail = checks
        .iter()
        .any(|c| c.status == TargetEmptyVerificationCheckStatus::Failed);

    let any_warning_only = !any_hard_fail && (!table_count_ok || !record_count_ok);

    let status = if !mode_safe || any_hard_fail {
        TargetEmptyVerificationStatus::Blocked
    } else if any_warning_only {
        TargetEmptyVerificationStatus::Warning
    } else {
        TargetEmptyVerificationStatus::Verified
    };

    let target_name = request
        .target_display_name
        .as_deref()
        .unwrap_or("the target base");

    let message = match &status {
        TargetEmptyVerificationStatus::Verified => {
            if request.target_mode == "newBase" {
                "New base target — no existing data to conflict with. Restore is safe to proceed when writes are enabled.".to_string()
            } else {
                format!(
                    "{} is confirmed empty (0 tables, 0 records). Restore is safe to proceed when writes are enabled.",
                    target_name
                )
            }
        }
        TargetEmptyVerificationStatus::Warning => format!(
            "Target base emptiness could not be confirmed for {}. Live metadata check was not performed. Resolve this before enabling live writes.",
            target_name
        ),
        TargetEmptyVerificationStatus::Blocked => {
            if !mode_safe {
                format!(
                    "Target mode '{}' is not supported. Only 'newBase' and 'emptyExistingBase' are allowed.",
                    request.target_mode
                )
            } else {
                format!(
                    "{} is not empty. Restoring into a non-empty base is blocked to prevent data loss.",
                    target_name
                )
            }
        }
    };

    TargetEmptyVerificationResult {
        status,
        checks,
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

    fn new_base_request() -> TargetEmptyVerificationRequest {
        TargetEmptyVerificationRequest {
            target_mode: "newBase".to_string(),
            target_table_count: None,
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        }
    }

    fn empty_existing_request() -> TargetEmptyVerificationRequest {
        TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: Some(0),
            target_record_count: Some(0),
            target_display_name: Some("My Test Base".to_string()),
            live_check_performed: true,
        }
    }

    // ------------------------------------------------------------------
    // Status determination
    // ------------------------------------------------------------------

    #[test]
    fn new_base_intent_is_verified() {
        let result = verify_target_empty(&new_base_request());
        assert_eq!(result.status, TargetEmptyVerificationStatus::Verified);
    }

    #[test]
    fn empty_existing_base_zero_tables_zero_records_is_verified() {
        let result = verify_target_empty(&empty_existing_request());
        assert_eq!(result.status, TargetEmptyVerificationStatus::Verified);
    }

    #[test]
    fn table_count_greater_than_zero_is_blocked() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: Some(3),
            target_record_count: Some(0),
            target_display_name: None,
            live_check_performed: true,
        };
        let result = verify_target_empty(&req);
        assert_eq!(result.status, TargetEmptyVerificationStatus::Blocked);
    }

    #[test]
    fn record_count_greater_than_zero_is_blocked() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: Some(0),
            target_record_count: Some(15),
            target_display_name: None,
            live_check_performed: true,
        };
        let result = verify_target_empty(&req);
        assert_eq!(result.status, TargetEmptyVerificationStatus::Blocked);
    }

    #[test]
    fn unknown_counts_for_existing_base_is_warning() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: None,
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert_eq!(result.status, TargetEmptyVerificationStatus::Warning);
    }

    #[test]
    fn unknown_table_count_only_is_warning() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: None,
            target_record_count: Some(0),
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert_eq!(result.status, TargetEmptyVerificationStatus::Warning);
    }

    #[test]
    fn unknown_record_count_only_is_warning() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: Some(0),
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert_eq!(result.status, TargetEmptyVerificationStatus::Warning);
    }

    #[test]
    fn unsafe_target_mode_is_blocked() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "existingBase".to_string(),
            target_table_count: Some(0),
            target_record_count: Some(0),
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert_eq!(result.status, TargetEmptyVerificationStatus::Blocked);
    }

    #[test]
    fn empty_mode_string_is_blocked() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "".to_string(),
            target_table_count: None,
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert_eq!(result.status, TargetEmptyVerificationStatus::Blocked);
    }

    // ------------------------------------------------------------------
    // Check presence and count
    // ------------------------------------------------------------------

    #[test]
    fn result_has_five_checks() {
        let result = verify_target_empty(&new_base_request());
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn check_ids_are_correct() {
        let result = verify_target_empty(&new_base_request());
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert_eq!(ids, vec!["TEV-01", "TEV-02", "TEV-03", "TEV-04", "TEV-05"]);
    }

    #[test]
    fn tev_01_always_passes_write_gate_disabled() {
        let result = verify_target_empty(&new_base_request());
        let tev01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "TEV-01")
            .unwrap();
        assert_eq!(tev01.status, TargetEmptyVerificationCheckStatus::Passed);
    }

    #[test]
    fn tev_05_always_passes() {
        let result = verify_target_empty(&empty_existing_request());
        let tev05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "TEV-05")
            .unwrap();
        assert_eq!(tev05.status, TargetEmptyVerificationCheckStatus::Passed);
    }

    #[test]
    fn tev_03_warning_when_existing_base_count_unknown() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: None,
            target_record_count: Some(0),
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        let tev03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "TEV-03")
            .unwrap();
        assert_eq!(tev03.status, TargetEmptyVerificationCheckStatus::Warning);
    }

    #[test]
    fn tev_04_warning_when_existing_base_record_count_unknown() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: Some(0),
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        let tev04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "TEV-04")
            .unwrap();
        assert_eq!(tev04.status, TargetEmptyVerificationCheckStatus::Warning);
    }

    // ------------------------------------------------------------------
    // Safety invariants
    // ------------------------------------------------------------------

    #[test]
    fn no_changes_made_always_true_verified() {
        let result = verify_target_empty(&new_base_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn no_changes_made_always_true_warning() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: None,
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn no_changes_made_always_true_blocked() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: Some(5),
            target_record_count: Some(100),
            target_display_name: None,
            live_check_performed: true,
        };
        let result = verify_target_empty(&req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn writes_enabled_always_false() {
        let result = verify_target_empty(&new_base_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let result = verify_target_empty(&new_base_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_false_when_blocked() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: Some(10),
            target_record_count: Some(0),
            target_display_name: None,
            live_check_performed: true,
        };
        let result = verify_target_empty(&req);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn writes_enabled_false_when_warning() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "emptyExistingBase".to_string(),
            target_table_count: None,
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert!(!result.writes_enabled);
    }

    // ------------------------------------------------------------------
    // Serialization — no token or path leaks
    // ------------------------------------------------------------------

    #[test]
    fn result_serialization_has_no_token() {
        let result = verify_target_empty(&empty_existing_request());
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("token"));
        assert!(!json.contains("pat"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn result_serialization_has_no_full_path() {
        let result = verify_target_empty(&empty_existing_request());
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\\\"));
    }

    #[test]
    fn message_is_non_empty_for_all_statuses() {
        let cases = vec![
            new_base_request(),
            empty_existing_request(),
            TargetEmptyVerificationRequest {
                target_mode: "emptyExistingBase".to_string(),
                target_table_count: None,
                target_record_count: None,
                target_display_name: None,
                live_check_performed: false,
            },
            TargetEmptyVerificationRequest {
                target_mode: "emptyExistingBase".to_string(),
                target_table_count: Some(3),
                target_record_count: Some(50),
                target_display_name: None,
                live_check_performed: true,
            },
        ];
        for req in &cases {
            let result = verify_target_empty(req);
            assert!(!result.message.is_empty());
        }
    }

    #[test]
    fn message_does_not_contain_token_for_any_status() {
        let cases = vec![new_base_request(), empty_existing_request()];
        for req in &cases {
            let result = verify_target_empty(req);
            assert!(!result.message.contains("pat"));
            assert!(!result.message.contains("token"));
        }
    }

    #[test]
    fn display_name_appears_in_verified_message() {
        let result = verify_target_empty(&empty_existing_request());
        assert!(result.message.contains("My Test Base"));
    }

    #[test]
    fn blocked_message_names_unsupported_mode() {
        let req = TargetEmptyVerificationRequest {
            target_mode: "existingBase".to_string(),
            target_table_count: None,
            target_record_count: None,
            target_display_name: None,
            live_check_performed: false,
        };
        let result = verify_target_empty(&req);
        assert!(result.message.contains("existingBase"));
    }

    #[test]
    fn no_write_calls_are_made() {
        // Structural: verify_target_empty signature takes no token, no HTTP client.
        // Confirmed at compile time — this test documents the intent.
        let result = verify_target_empty(&new_base_request());
        assert!(!result.network_writes_attempted);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
    }
}
