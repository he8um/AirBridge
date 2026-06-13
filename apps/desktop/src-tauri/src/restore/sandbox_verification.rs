use serde::{Deserialize, Serialize};

use crate::restore::plan::RestoreTargetMode;
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ---------------------------------------------------------------------------
// Status enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxVerificationStatus {
    Verified,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxVerificationCheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxVerificationFailureReason {
    TargetModeNotAllowed,
    TargetNotEmpty,
    MissingTargetIdentifier,
    MissingTargetName,
    /// Indicates the write gate is correctly disabled — this is the expected/good state.
    WriteGateDisabled,
    WriteGateUnexpectedState,
    DestructiveOperationRequested,
    AttachmentUploadRequested,
    TokenReturnForbidden,
    FullPathReturnForbidden,
    LiveMetadataCheckUnavailable,
    InvalidRequest,
    UnsupportedTarget,
}

// ---------------------------------------------------------------------------
// Data structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxVerificationTarget {
    pub target_mode: RestoreTargetMode,
    pub target_base_id: Option<String>,
    pub target_base_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxVerificationSafetySummary {
    /// Always false — writes are never enabled.
    pub writes_enabled: bool,
    /// Always false — no network write calls are made.
    pub network_writes_attempted: bool,
    /// Always true — no changes are made.
    pub no_changes_made: bool,
    pub write_gate_status: String,
    /// Always false — live metadata check is not implemented.
    pub live_metadata_check_performed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxVerificationCheck {
    pub check_id: String,
    pub label: String,
    pub status: SandboxVerificationCheckStatus,
    pub message: String,
    pub remediation: Option<String>,
}

/// Request payload for sandbox verification.
///
/// Safety invariants enforced by type:
/// - No token field.
/// - No full path field (source_package_filename must be filename-only; CHK-07 enforces this at runtime).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxVerificationRequest {
    pub target_mode: RestoreTargetMode,
    pub target_base_id: Option<String>,
    pub target_base_name: Option<String>,
    pub target_table_count: Option<u32>,
    pub target_record_count: Option<u32>,
    pub expects_empty_target: bool,
    pub allow_attachment_upload: bool,
    pub allow_destructive_operations: bool,
    pub source_package_filename: Option<String>,
    pub schema_plan_status: Option<String>,
    pub record_import_plan_status: Option<String>,
}

/// Result of sandbox environment verification.
///
/// Safety invariants:
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxVerificationResult {
    pub status: SandboxVerificationStatus,
    pub checks: Vec<SandboxVerificationCheck>,
    pub safety_summary: SandboxVerificationSafetySummary,
    pub message: String,
    /// Always true.
    pub no_changes_made: bool,
    /// Always false.
    pub network_writes_attempted: bool,
    /// Always false.
    pub writes_enabled: bool,
}

// ---------------------------------------------------------------------------
// Verification logic
// ---------------------------------------------------------------------------

pub fn verify_sandbox_environment(
    request: &SandboxVerificationRequest,
) -> SandboxVerificationResult {
    let mut checks: Vec<SandboxVerificationCheck> = Vec::new();

    // ------------------------------------------------------------------
    // CHK-01: target-mode
    // ------------------------------------------------------------------
    {
        let (status, message, remediation) = match &request.target_mode {
            RestoreTargetMode::NewBase => (
                SandboxVerificationCheckStatus::Passed,
                "Target mode is safe for restore.".to_string(),
                None,
            ),
            RestoreTargetMode::EmptyExistingBase => {
                // Special case: EmptyExistingBase with no identifier → Warning
                if request.target_base_id.is_none() && request.target_base_name.is_none() {
                    (
                        SandboxVerificationCheckStatus::Warning,
                        "EmptyExistingBase target is missing an identifier. Provide target_base_id or target_base_name.".to_string(),
                        Some("Provide either target_base_id or target_base_name to identify the target base.".to_string()),
                    )
                } else {
                    (
                        SandboxVerificationCheckStatus::Passed,
                        "Target mode is safe for restore.".to_string(),
                        None,
                    )
                }
            }
        };

        checks.push(SandboxVerificationCheck {
            check_id: "CHK-01".to_string(),
            label: "target-mode".to_string(),
            status,
            message,
            remediation,
        });
    }

    // ------------------------------------------------------------------
    // CHK-02: empty-target-expectation
    // ------------------------------------------------------------------
    {
        if request.expects_empty_target {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-02".to_string(),
                label: "empty-target-expectation".to_string(),
                status: SandboxVerificationCheckStatus::Passed,
                message: "Target is expected to be empty before restore.".to_string(),
                remediation: None,
            });
        } else {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-02".to_string(),
                label: "empty-target-expectation".to_string(),
                status: SandboxVerificationCheckStatus::Failed,
                message: "Target is not expected to be empty. Restoring into a non-empty target is not safe.".to_string(),
                remediation: Some("Set expects_empty_target to true, or choose an empty target base.".to_string()),
            });
        }
    }

    // ------------------------------------------------------------------
    // CHK-03: write-gate
    // ------------------------------------------------------------------
    {
        let gate = evaluate_write_gate();
        if gate.status == RestoreWriteEngineStatus::Disabled {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-03".to_string(),
                label: "write-gate".to_string(),
                status: SandboxVerificationCheckStatus::Passed,
                message: "Write gate is disabled — no writes can be executed.".to_string(),
                remediation: None,
            });
        } else {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-03".to_string(),
                label: "write-gate".to_string(),
                status: SandboxVerificationCheckStatus::Failed,
                message: format!(
                    "Write gate returned an unexpected state. Expected Disabled, got a different status. Gate message: {}",
                    gate.message
                ),
                remediation: Some("The write gate must return Disabled for sandbox verification to pass.".to_string()),
            });
        }
    }

    // ------------------------------------------------------------------
    // CHK-04: destructive-operations
    // ------------------------------------------------------------------
    {
        if !request.allow_destructive_operations {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-04".to_string(),
                label: "destructive-operations".to_string(),
                status: SandboxVerificationCheckStatus::Passed,
                message: "Destructive operations are not requested.".to_string(),
                remediation: None,
            });
        } else {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-04".to_string(),
                label: "destructive-operations".to_string(),
                status: SandboxVerificationCheckStatus::Failed,
                message: "Destructive operations have been requested. This is not permitted in sandbox mode.".to_string(),
                remediation: Some("Set allow_destructive_operations to false.".to_string()),
            });
        }
    }

    // ------------------------------------------------------------------
    // CHK-05: attachment-upload (Warning, not Failed)
    // ------------------------------------------------------------------
    {
        if !request.allow_attachment_upload {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-05".to_string(),
                label: "attachment-upload".to_string(),
                status: SandboxVerificationCheckStatus::Passed,
                message: "Attachment upload is not requested.".to_string(),
                remediation: None,
            });
        } else {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-05".to_string(),
                label: "attachment-upload".to_string(),
                status: SandboxVerificationCheckStatus::Warning,
                message: "Attachment upload has been requested. Attachment handling is not supported in this version.".to_string(),
                remediation: Some("Set allow_attachment_upload to false to suppress this warning.".to_string()),
            });
        }
    }

    // ------------------------------------------------------------------
    // CHK-06: plan-status
    // ------------------------------------------------------------------
    {
        let schema_blocked = request
            .schema_plan_status
            .as_deref()
            .map(|s| s == "blocked")
            .unwrap_or(false);
        let record_blocked = request
            .record_import_plan_status
            .as_deref()
            .map(|s| s == "blocked")
            .unwrap_or(false);

        if schema_blocked || record_blocked {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-06".to_string(),
                label: "plan-status".to_string(),
                status: SandboxVerificationCheckStatus::Failed,
                message: "A plan dependency is blocked.".to_string(),
                remediation: Some(
                    "Resolve the blocked plan before proceeding with restore.".to_string(),
                ),
            });
        } else if request.schema_plan_status.is_none()
            && request.record_import_plan_status.is_none()
        {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-06".to_string(),
                label: "plan-status".to_string(),
                status: SandboxVerificationCheckStatus::Warning,
                message: "Plan statuses not provided — verification is incomplete.".to_string(),
                remediation: Some("Provide schema_plan_status and record_import_plan_status for full verification.".to_string()),
            });
        } else {
            checks.push(SandboxVerificationCheck {
                check_id: "CHK-06".to_string(),
                label: "plan-status".to_string(),
                status: SandboxVerificationCheckStatus::Passed,
                message: "Plan statuses are present and not blocked.".to_string(),
                remediation: None,
            });
        }
    }

    // ------------------------------------------------------------------
    // CHK-07: filename-safety
    // ------------------------------------------------------------------
    {
        match &request.source_package_filename {
            None => {
                checks.push(SandboxVerificationCheck {
                    check_id: "CHK-07".to_string(),
                    label: "filename-safety".to_string(),
                    status: SandboxVerificationCheckStatus::Skipped,
                    message: "No source package filename provided.".to_string(),
                    remediation: None,
                });
            }
            Some(filename) => {
                if filename.contains('/') || filename.contains('\\') {
                    checks.push(SandboxVerificationCheck {
                        check_id: "CHK-07".to_string(),
                        label: "filename-safety".to_string(),
                        status: SandboxVerificationCheckStatus::Failed,
                        message: "Source package filename must not be a full path.".to_string(),
                        remediation: Some("Provide only the filename (e.g., backup.zip), not an absolute or relative path.".to_string()),
                    });
                } else {
                    checks.push(SandboxVerificationCheck {
                        check_id: "CHK-07".to_string(),
                        label: "filename-safety".to_string(),
                        status: SandboxVerificationCheckStatus::Passed,
                        message: "Source package filename is safe.".to_string(),
                        remediation: None,
                    });
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // CHK-08: token-safety (structural guarantee — always Passed)
    // ------------------------------------------------------------------
    {
        checks.push(SandboxVerificationCheck {
            check_id: "CHK-08".to_string(),
            label: "token-safety".to_string(),
            status: SandboxVerificationCheckStatus::Passed,
            message: "No token is accepted or returned by sandbox verification.".to_string(),
            remediation: None,
        });
    }

    // ------------------------------------------------------------------
    // CHK-09: network-safety (always Passed)
    // ------------------------------------------------------------------
    {
        checks.push(SandboxVerificationCheck {
            check_id: "CHK-09".to_string(),
            label: "network-safety".to_string(),
            status: SandboxVerificationCheckStatus::Passed,
            message: "No Airtable writes are attempted. No network write calls are made."
                .to_string(),
            remediation: None,
        });
    }

    // ------------------------------------------------------------------
    // CHK-10: live-metadata-check (always Skipped)
    // ------------------------------------------------------------------
    {
        checks.push(SandboxVerificationCheck {
            check_id: "CHK-10".to_string(),
            label: "live-metadata-check".to_string(),
            status: SandboxVerificationCheckStatus::Skipped,
            message: "Live metadata check is not implemented in this version. Full verification requires a future implementation.".to_string(),
            remediation: Some("A future version will support live metadata verification against the Airtable API.".to_string()),
        });
    }

    // ------------------------------------------------------------------
    // Derive overall status
    // ------------------------------------------------------------------
    let overall_status = derive_overall_status(&checks);

    let message = match &overall_status {
        SandboxVerificationStatus::Verified => {
            "Sandbox verification passed. No changes have been made and no writes are enabled.".to_string()
        }
        SandboxVerificationStatus::Warning => {
            "Sandbox verification completed with warnings. Review the checks before proceeding.".to_string()
        }
        SandboxVerificationStatus::Blocked => {
            "Sandbox verification is blocked. One or more checks failed. No restore operation can proceed.".to_string()
        }
    };

    let safety_summary = SandboxVerificationSafetySummary {
        writes_enabled: false,
        network_writes_attempted: false,
        no_changes_made: true,
        write_gate_status: "disabled".to_string(),
        live_metadata_check_performed: false,
    };

    SandboxVerificationResult {
        status: overall_status,
        checks,
        safety_summary,
        message,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

fn derive_overall_status(checks: &[SandboxVerificationCheck]) -> SandboxVerificationStatus {
    let has_failed = checks
        .iter()
        .any(|c| c.status == SandboxVerificationCheckStatus::Failed);
    if has_failed {
        return SandboxVerificationStatus::Blocked;
    }

    let has_warning_or_skipped = checks.iter().any(|c| {
        c.status == SandboxVerificationCheckStatus::Warning
            || c.status == SandboxVerificationCheckStatus::Skipped
    });
    if has_warning_or_skipped {
        return SandboxVerificationStatus::Warning;
    }

    SandboxVerificationStatus::Verified
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_request() -> SandboxVerificationRequest {
        SandboxVerificationRequest {
            target_mode: RestoreTargetMode::NewBase,
            target_base_id: None,
            target_base_name: Some("My Restored Base".to_string()),
            target_table_count: Some(3),
            target_record_count: Some(100),
            expects_empty_target: true,
            allow_attachment_upload: false,
            allow_destructive_operations: false,
            source_package_filename: Some("backup.zip".to_string()),
            schema_plan_status: Some("ready".to_string()),
            record_import_plan_status: Some("ready".to_string()),
        }
    }

    #[test]
    fn new_base_target_returns_verified_or_warning() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        // With a safe request the result must not be Blocked.
        assert_ne!(result.status, SandboxVerificationStatus::Blocked);
        let chk01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-01")
            .unwrap();
        assert_eq!(chk01.status, SandboxVerificationCheckStatus::Passed);
    }

    #[test]
    fn empty_existing_base_target_returns_verified_or_warning() {
        let mut req = safe_request();
        req.target_mode = RestoreTargetMode::EmptyExistingBase;
        req.target_base_id = Some("appXYZ".to_string());
        let result = verify_sandbox_environment(&req);
        assert_ne!(result.status, SandboxVerificationStatus::Blocked);
        let chk01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-01")
            .unwrap();
        assert_eq!(chk01.status, SandboxVerificationCheckStatus::Passed);
    }

    #[test]
    fn both_target_modes_are_passed_on_chk01() {
        // RestoreTargetMode only has NewBase and EmptyExistingBase — both are allowed.
        for mode in [
            RestoreTargetMode::NewBase,
            RestoreTargetMode::EmptyExistingBase,
        ] {
            let mut req = safe_request();
            req.target_mode = mode.clone();
            if mode == RestoreTargetMode::EmptyExistingBase {
                req.target_base_id = Some("appABC".to_string());
            }
            let result = verify_sandbox_environment(&req);
            let chk01 = result
                .checks
                .iter()
                .find(|c| c.check_id == "CHK-01")
                .unwrap();
            assert_eq!(
                chk01.status,
                SandboxVerificationCheckStatus::Passed,
                "CHK-01 should pass for {:?}",
                mode
            );
        }
    }

    #[test]
    fn non_empty_target_returns_blocked() {
        let mut req = safe_request();
        req.expects_empty_target = false;
        let result = verify_sandbox_environment(&req);
        assert_eq!(result.status, SandboxVerificationStatus::Blocked);
        let chk02 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-02")
            .unwrap();
        assert_eq!(chk02.status, SandboxVerificationCheckStatus::Failed);
    }

    #[test]
    fn destructive_operations_returns_blocked() {
        let mut req = safe_request();
        req.allow_destructive_operations = true;
        let result = verify_sandbox_environment(&req);
        assert_eq!(result.status, SandboxVerificationStatus::Blocked);
        let chk04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-04")
            .unwrap();
        assert_eq!(chk04.status, SandboxVerificationCheckStatus::Failed);
    }

    #[test]
    fn attachment_upload_returns_warning_not_blocked() {
        let mut req = safe_request();
        req.allow_attachment_upload = true;
        let result = verify_sandbox_environment(&req);
        // Must be Warning, not Blocked
        assert_eq!(result.status, SandboxVerificationStatus::Warning);
        let chk05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-05")
            .unwrap();
        assert_eq!(chk05.status, SandboxVerificationCheckStatus::Warning);
    }

    #[test]
    fn write_gate_check_is_included_and_passed() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        let chk03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-03")
            .unwrap();
        assert_eq!(chk03.status, SandboxVerificationCheckStatus::Passed);
        assert!(chk03.message.contains("disabled") || chk03.message.contains("Write gate"));
    }

    #[test]
    fn gate_cannot_return_succeeded() {
        // Structural test: enumerate all allowed gate statuses — Succeeded must not exist.
        // RestoreWriteEngineStatus variants: Disabled, Blocked, NotStarted
        // None of them are "Succeeded", which confirms no succeeded path exists.
        let allowed_statuses = [
            RestoreWriteEngineStatus::Disabled,
            RestoreWriteEngineStatus::Blocked,
            RestoreWriteEngineStatus::NotStarted,
        ];
        for status in &allowed_statuses {
            // None of the variant names should be "Succeeded"
            let debug_str = format!("{:?}", status);
            assert_ne!(debug_str, "Succeeded");
        }

        // The actual gate must always return Disabled.
        let gate = evaluate_write_gate();
        assert_eq!(gate.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn missing_target_name_for_empty_existing_base_returns_warning() {
        let mut req = safe_request();
        req.target_mode = RestoreTargetMode::EmptyExistingBase;
        req.target_base_id = None;
        req.target_base_name = None;
        let result = verify_sandbox_environment(&req);
        let chk01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-01")
            .unwrap();
        assert_eq!(chk01.status, SandboxVerificationCheckStatus::Warning);
        assert!(chk01.message.contains("missing an identifier"));
        // Overall should be Warning, not Blocked (CHK-01 is Warning not Failed)
        assert_ne!(result.status, SandboxVerificationStatus::Blocked);
        // But it cannot be Verified either, since there's at least one Warning/Skipped
        assert_eq!(result.status, SandboxVerificationStatus::Warning);
    }

    #[test]
    fn full_path_filename_returns_blocked() {
        let mut req = safe_request();
        req.source_package_filename = Some("/Users/someone/backups/backup.zip".to_string());
        let result = verify_sandbox_environment(&req);
        assert_eq!(result.status, SandboxVerificationStatus::Blocked);
        let chk07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-07")
            .unwrap();
        assert_eq!(chk07.status, SandboxVerificationCheckStatus::Failed);
    }

    #[test]
    fn filename_only_is_accepted() {
        let mut req = safe_request();
        req.source_package_filename = Some("my_backup.zip".to_string());
        let result = verify_sandbox_environment(&req);
        let chk07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-07")
            .unwrap();
        assert_eq!(chk07.status, SandboxVerificationCheckStatus::Passed);
    }

    #[test]
    fn windows_path_filename_returns_blocked() {
        let mut req = safe_request();
        req.source_package_filename = Some("C:\\Users\\someone\\backup.zip".to_string());
        let result = verify_sandbox_environment(&req);
        let chk07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-07")
            .unwrap();
        assert_eq!(chk07.status, SandboxVerificationCheckStatus::Failed);
    }

    #[test]
    fn schema_plan_blocked_returns_blocked() {
        let mut req = safe_request();
        req.schema_plan_status = Some("blocked".to_string());
        let result = verify_sandbox_environment(&req);
        assert_eq!(result.status, SandboxVerificationStatus::Blocked);
        let chk06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-06")
            .unwrap();
        assert_eq!(chk06.status, SandboxVerificationCheckStatus::Failed);
    }

    #[test]
    fn record_import_plan_blocked_returns_blocked() {
        let mut req = safe_request();
        req.record_import_plan_status = Some("blocked".to_string());
        let result = verify_sandbox_environment(&req);
        assert_eq!(result.status, SandboxVerificationStatus::Blocked);
        let chk06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-06")
            .unwrap();
        assert_eq!(chk06.status, SandboxVerificationCheckStatus::Failed);
    }

    #[test]
    fn result_serialization_has_no_token() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        let serialized = serde_json::to_string(&result).unwrap();
        // Must not contain actual credential patterns — bearer tokens or API key values.
        assert!(
            !serialized.contains("pat_"),
            "must not contain Airtable personal access token prefix"
        );
        assert!(
            !serialized.contains("apiKey"),
            "must not contain apiKey field"
        );
        assert!(
            !serialized.contains("api_key"),
            "must not contain api_key field"
        );
        // The struct definition must not have a token field at all.
        // We verify this by ensuring no JSON key named "token" appears.
        // (The word "token" may legitimately appear in label/message text — that is fine.)
        let json_val: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        fn has_key_named_token(v: &serde_json::Value) -> bool {
            match v {
                serde_json::Value::Object(map) => {
                    map.contains_key("token") || map.values().any(has_key_named_token)
                }
                serde_json::Value::Array(arr) => arr.iter().any(has_key_named_token),
                _ => false,
            }
        }
        assert!(
            !has_key_named_token(&json_val),
            "serialized result must not contain a JSON key named 'token'"
        );
    }

    #[test]
    fn result_serialization_has_no_full_path() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(!serialized.contains("/Users/"));
        assert!(!serialized.contains("/tmp/"));
        assert!(!serialized.contains("C:\\"));
    }

    #[test]
    fn no_changes_made_always_true() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        assert!(result.no_changes_made);
        assert!(result.safety_summary.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        assert!(!result.network_writes_attempted);
        assert!(!result.safety_summary.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        assert!(!result.writes_enabled);
        assert!(!result.safety_summary.writes_enabled);
    }

    #[test]
    fn live_metadata_check_is_skipped() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        let chk10 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-10")
            .unwrap();
        assert_eq!(chk10.status, SandboxVerificationCheckStatus::Skipped);
        assert!(!result.safety_summary.live_metadata_check_performed);
    }

    #[test]
    fn safety_summary_write_gate_status_is_disabled() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        assert_eq!(result.safety_summary.write_gate_status, "disabled");
    }

    #[test]
    fn chk08_token_safety_always_passed() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        let chk08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-08")
            .unwrap();
        assert_eq!(chk08.status, SandboxVerificationCheckStatus::Passed);
    }

    #[test]
    fn chk09_network_safety_always_passed() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        let chk09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-09")
            .unwrap();
        assert_eq!(chk09.status, SandboxVerificationCheckStatus::Passed);
    }

    #[test]
    fn all_ten_checks_are_present() {
        let req = safe_request();
        let result = verify_sandbox_environment(&req);
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        for i in 1..=10 {
            let expected = format!("CHK-{:02}", i);
            assert!(
                ids.contains(&expected.as_str()),
                "Missing check {}",
                expected
            );
        }
    }

    #[test]
    fn blocked_result_has_correct_message() {
        let mut req = safe_request();
        req.expects_empty_target = false;
        let result = verify_sandbox_environment(&req);
        assert_eq!(result.status, SandboxVerificationStatus::Blocked);
        assert!(result.message.contains("blocked") || result.message.contains("Blocked"));
    }

    #[test]
    fn missing_filename_skips_chk07() {
        let mut req = safe_request();
        req.source_package_filename = None;
        let result = verify_sandbox_environment(&req);
        let chk07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CHK-07")
            .unwrap();
        assert_eq!(chk07.status, SandboxVerificationCheckStatus::Skipped);
    }
}
