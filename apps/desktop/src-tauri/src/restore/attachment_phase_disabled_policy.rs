use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

/// The overall verdict for an attachment phase disabled policy check.
///
/// Safety invariants:
/// - `Compliant` does NOT enable restore writes.
/// - `writesEnabled` is always false regardless of status.
/// - No attachment binary download, upload, fetch, or transfer is ever performed.
/// - Attachment handling is metadata-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentPhaseDisabledPolicyStatus {
    /// All attachment operations are metadata-only; no binary transfer planned.
    Compliant,
    /// Metadata verification skipped but documented as metadata-only unavailable.
    Warning,
    /// A binary download, upload, fetch, transfer, field mutation, or URL exposure
    /// is planned or allowed, or attachment phase is required for restore completion.
    Blocked,
}

/// The result of a single attachment phase disabled policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentPhaseDisabledCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// The class of attachment operation being declared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentPhaseOperation {
    /// Read attachment metadata (filename, MIME type, size). Permitted.
    MetadataInspect,
    /// Verify attachment metadata is well-formed. Permitted.
    MetadataVerify,
    /// Download attachment binary from a URL. Blocked.
    BinaryDownload,
    /// Upload attachment binary to Airtable. Blocked.
    BinaryUpload,
    /// Fetch an attachment URL (even read-only). Blocked.
    UrlFetch,
    /// Read an attachment binary from a local file. Blocked.
    FileRead,
    /// Write an attachment binary to a local file. Blocked.
    FileWrite,
    /// Transfer attachment bytes between any two endpoints. Blocked.
    RawAttachmentTransfer,
    /// Mutate an attachment field value in Airtable. Blocked.
    AttachmentFieldMutation,
    /// Expose an attachment URL in any result, log, or diagnostic. Blocked.
    AttachmentUrlExposure,
}

impl AttachmentPhaseOperation {
    /// Returns `true` if this operation is unconditionally blocked.
    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            AttachmentPhaseOperation::BinaryDownload
                | AttachmentPhaseOperation::BinaryUpload
                | AttachmentPhaseOperation::UrlFetch
                | AttachmentPhaseOperation::FileRead
                | AttachmentPhaseOperation::FileWrite
                | AttachmentPhaseOperation::RawAttachmentTransfer
                | AttachmentPhaseOperation::AttachmentFieldMutation
                | AttachmentPhaseOperation::AttachmentUrlExposure
        )
    }

    /// Returns `true` if this operation is permitted.
    pub fn is_permitted(&self) -> bool {
        matches!(
            self,
            AttachmentPhaseOperation::MetadataInspect | AttachmentPhaseOperation::MetadataVerify
        )
    }
}

// ── Request / result types ────────────────────────────────────────────────────

/// A single declared attachment operation to be policy-checked.
///
/// Safety: no token field, no path field, no attachment URL field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPhasePlan {
    /// The operation being declared.
    pub operation: AttachmentPhaseOperation,
    /// Whether this operation is planned (intended to execute).
    pub planned: bool,
    /// Whether this operation is required for restore completion.
    pub required_for_completion: bool,
    /// Optional justification for why this operation is declared.
    pub justification: Option<String>,
}

/// Metadata-only attachment handling plan.
///
/// Safety: no token field, no path field, no attachment URL field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMetadataOnlyPlan {
    /// Metadata inspection is enabled (reading field names, MIME types, sizes).
    pub metadata_inspection_enabled: bool,
    /// Metadata verification is enabled (checking metadata completeness).
    pub metadata_verification_enabled: bool,
    /// If metadata verification is not enabled, reason it is not required.
    pub metadata_verification_skip_reason: Option<String>,
    /// Explicit declaration that attachment binaries are not handled.
    pub binary_handling_disabled: bool,
    /// Explicit declaration that attachment URLs are not exposed.
    pub url_exposure_disabled: bool,
    /// Explicit declaration that attachment field mutation is not performed.
    pub field_mutation_disabled: bool,
    /// Explicit declaration that attachment phase cannot be required for completion.
    pub phase_required_for_completion_disabled: bool,
    /// Final validation treats attachments as metadata-only.
    pub final_validation_treats_as_metadata_only: bool,
}

/// Request for Gate 17 attachment phase disabled policy verification.
///
/// Safety: no token field, no path field, no attachment URL field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPhaseDisabledPolicyRequest {
    /// The metadata-only plan declaring attachment handling intent.
    pub plan: Option<AttachmentMetadataOnlyPlan>,
    /// Additional operation-level declarations (optional).
    pub declared_operations: Option<Vec<AttachmentPhasePlan>>,
    /// Human-readable description of the restore target (optional, for messages).
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPhaseDisabledCheck {
    pub check_id: String,
    pub label: String,
    pub status: AttachmentPhaseDisabledCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Summary of the attachment phase disabled policy evaluation.
///
/// Safety: no token field, no path field, no attachment URL field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPhaseDisabledSummary {
    pub metadata_inspection_enabled: bool,
    pub metadata_verification_enabled: bool,
    pub binary_handling_disabled: bool,
    pub url_exposure_disabled: bool,
    pub field_mutation_disabled: bool,
    pub phase_required_for_completion_disabled: bool,
    pub final_validation_treats_as_metadata_only: bool,
    pub blocked_operations_declared: usize,
}

/// Result from `verify_attachment_phase_disabled_policy`.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No full attachment URL field.
/// - No record payload field.
/// - No raw HTTP request/response field.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - `Compliant` does NOT enable restore writes.
/// - `Compliant` does NOT introduce a restore success state.
/// - No attachment binary download, upload, fetch, or transfer is ever performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPhaseDisabledPolicyResult {
    pub status: AttachmentPhaseDisabledPolicyStatus,
    pub checks: Vec<AttachmentPhaseDisabledCheck>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_summary: Option<AttachmentPhaseDisabledSummary>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn passed(check_id: &str, label: &str, message: &str) -> AttachmentPhaseDisabledCheck {
    AttachmentPhaseDisabledCheck {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: AttachmentPhaseDisabledCheckStatus::Passed,
        message: message.to_string(),
        remediation: None,
    }
}

fn warning(
    check_id: &str,
    label: &str,
    message: &str,
    remediation: &str,
) -> AttachmentPhaseDisabledCheck {
    AttachmentPhaseDisabledCheck {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: AttachmentPhaseDisabledCheckStatus::Warning,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}

fn failed(
    check_id: &str,
    label: &str,
    message: &str,
    remediation: &str,
) -> AttachmentPhaseDisabledCheck {
    AttachmentPhaseDisabledCheck {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: AttachmentPhaseDisabledCheckStatus::Failed,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}

fn build_result(
    status: AttachmentPhaseDisabledPolicyStatus,
    checks: Vec<AttachmentPhaseDisabledCheck>,
    message: String,
    phase_summary: Option<AttachmentPhaseDisabledSummary>,
) -> AttachmentPhaseDisabledPolicyResult {
    AttachmentPhaseDisabledPolicyResult {
        status,
        checks,
        message,
        phase_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies that no attachment binary phase is planned or permitted.
///
/// Check IDs:
/// - APD-01: Write gate is disabled (always passes; short-circuits to Blocked if gate enabled).
/// - APD-02: Attachment phase plan declared (short-circuits to Blocked if None).
/// - APD-03: Metadata inspection is explicitly allowed.
/// - APD-04: Metadata verification is explicitly allowed or skipped with reason.
/// - APD-05: Binary download is explicitly disabled.
/// - APD-06: Binary upload is explicitly disabled.
/// - APD-07: URL fetch is explicitly disabled.
/// - APD-08: File read/write is explicitly disabled (covered by binary_handling_disabled).
/// - APD-09: Raw attachment transfer is explicitly disabled.
/// - APD-10: Attachment field mutation is explicitly disabled.
/// - APD-11: Attachment URL exposure is explicitly disabled.
/// - APD-12: Attachment phase cannot be required for restore completion.
/// - APD-13: Final validation treats attachments as metadata-only.
/// - APD-14: No success state introduced (safety invariant — always passes).
/// - APD-15: No writes attempted (safety invariant — always passes).
/// - APD-16: Writes remain disabled even when compliant (safety invariant — always passes).
///
/// Additionally, any declared operations that are blocked cause immediate Blocked.
///
/// No Airtable API calls are made.
/// No token is accepted or returned.
/// No filesystem path is accepted or returned.
/// No attachment URL is accepted or returned.
pub fn verify_attachment_phase_disabled_policy(
    request: &AttachmentPhaseDisabledPolicyRequest,
) -> AttachmentPhaseDisabledPolicyResult {
    let mut checks: Vec<AttachmentPhaseDisabledCheck> = Vec::new();

    // APD-01: Write gate disabled
    let gate = evaluate_write_gate();
    let gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if !gate_disabled {
        checks.push(failed(
            "APD-01",
            "write-gate-disabled",
            "Write gate is unexpectedly enabled. Attachment phase disabled policy cannot run while writes are enabled.",
            "Ensure evaluate_write_gate() returns Disabled before running attachment phase policy.",
        ));
        return build_result(
            AttachmentPhaseDisabledPolicyStatus::Blocked,
            checks,
            "Attachment phase disabled policy is blocked. Write gate is unexpectedly enabled. No attachment operation may proceed.".to_string(),
            None,
        );
    }
    checks.push(passed(
        "APD-01",
        "write-gate-disabled",
        "Write gate is disabled. No restore writes are attempted.",
    ));

    // APD-02: Plan declared — short-circuit if absent
    let plan = match &request.plan {
        Some(p) => p,
        None => {
            checks.push(failed(
                "APD-02",
                "attachment-phase-plan-declared",
                "No attachment phase plan declared. A plan explicitly disabling all binary attachment operations is required before any live restore write can proceed.",
                "Declare an AttachmentMetadataOnlyPlan with all binary operations disabled and metadata-only flags set.",
            ));
            return build_result(
                AttachmentPhaseDisabledPolicyStatus::Blocked,
                checks,
                "Attachment phase disabled policy is blocked. No plan was declared. Binary attachment download, upload, fetch, and transfer are not permitted. Restore writes remain disabled.".to_string(),
                None,
            );
        }
    };
    checks.push(passed(
        "APD-02",
        "attachment-phase-plan-declared",
        "Attachment phase plan is declared.",
    ));

    let mut blocked = false;
    let mut has_warning = false;

    // APD-03: Metadata inspection allowed
    if !plan.metadata_inspection_enabled {
        checks.push(failed(
            "APD-03",
            "metadata-inspection-allowed",
            "metadataInspectionEnabled is not set. Metadata inspection (reading field names, MIME types, sizes) must be explicitly enabled to confirm it is the only attachment operation.",
            "Set metadataInspectionEnabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-03",
            "metadata-inspection-allowed",
            "Metadata inspection is enabled. Attachment field names, MIME types, and sizes may be read.",
        ));
    }

    // APD-04: Metadata verification allowed (Warning if skipped with reason)
    if plan.metadata_verification_enabled {
        checks.push(passed(
            "APD-04",
            "metadata-verification-allowed",
            "Metadata verification is enabled. Attachment metadata completeness is checked.",
        ));
    } else if plan.metadata_verification_skip_reason.is_some() {
        checks.push(warning(
            "APD-04",
            "metadata-verification-allowed",
            "Metadata verification is not enabled. A skip reason is provided, but metadata verification is preferred.",
            "Enable metadataVerificationEnabled or confirm that the skip reason is valid for this restore target.",
        ));
        has_warning = true;
    } else {
        checks.push(failed(
            "APD-04",
            "metadata-verification-allowed",
            "Metadata verification is not enabled and no skip reason is provided. Either enable metadata verification or provide a metadataVerificationSkipReason.",
            "Set metadataVerificationEnabled to true, or provide a metadataVerificationSkipReason explaining why it is not required.",
        ));
        blocked = true;
    }

    // APD-05: Binary download blocked
    if !plan.binary_handling_disabled {
        checks.push(failed(
            "APD-05",
            "binary-download-blocked",
            "binaryHandlingDisabled is not set. Binary attachment download must be explicitly disabled.",
            "Set binaryHandlingDisabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-05",
            "binary-download-blocked",
            "Binary attachment download is explicitly disabled.",
        ));
    }

    // APD-06: Binary upload blocked
    if !plan.binary_handling_disabled {
        checks.push(failed(
            "APD-06",
            "binary-upload-blocked",
            "binaryHandlingDisabled is not set. Binary attachment upload must be explicitly disabled.",
            "Set binaryHandlingDisabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-06",
            "binary-upload-blocked",
            "Binary attachment upload is explicitly disabled.",
        ));
    }

    // APD-07: URL fetch blocked
    if !plan.url_exposure_disabled {
        checks.push(failed(
            "APD-07",
            "url-fetch-blocked",
            "urlExposureDisabled is not set. Attachment URL fetching must be explicitly disabled.",
            "Set urlExposureDisabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-07",
            "url-fetch-blocked",
            "Attachment URL fetching is explicitly disabled.",
        ));
    }

    // APD-08: File read/write blocked for attachment binaries
    if !plan.binary_handling_disabled {
        checks.push(failed(
            "APD-08",
            "file-read-write-blocked",
            "binaryHandlingDisabled is not set. File read and write of attachment binaries must be explicitly disabled.",
            "Set binaryHandlingDisabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-08",
            "file-read-write-blocked",
            "File read and write of attachment binaries is explicitly disabled.",
        ));
    }

    // APD-09: Raw attachment transfer blocked
    if !plan.binary_handling_disabled {
        checks.push(failed(
            "APD-09",
            "raw-attachment-transfer-blocked",
            "binaryHandlingDisabled is not set. Raw attachment transfer must be explicitly disabled.",
            "Set binaryHandlingDisabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-09",
            "raw-attachment-transfer-blocked",
            "Raw attachment transfer is explicitly disabled.",
        ));
    }

    // APD-10: Attachment field mutation blocked
    if !plan.field_mutation_disabled {
        checks.push(failed(
            "APD-10",
            "attachment-field-mutation-blocked",
            "fieldMutationDisabled is not set. Attachment field mutation in Airtable must be explicitly disabled.",
            "Set fieldMutationDisabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-10",
            "attachment-field-mutation-blocked",
            "Attachment field mutation is explicitly disabled.",
        ));
    }

    // APD-11: Attachment URL exposure blocked
    if !plan.url_exposure_disabled {
        checks.push(failed(
            "APD-11",
            "attachment-url-exposure-blocked",
            "urlExposureDisabled is not set. Attachment URL exposure in results, logs, or diagnostics must be explicitly disabled.",
            "Set urlExposureDisabled to true in the attachment phase plan.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-11",
            "attachment-url-exposure-blocked",
            "Attachment URL exposure is explicitly disabled.",
        ));
    }

    // APD-12: Attachment phase cannot be required for restore completion
    if !plan.phase_required_for_completion_disabled {
        checks.push(failed(
            "APD-12",
            "phase-not-required-for-completion",
            "phaseRequiredForCompletionDisabled is not set. Restore completion must not require attachment phase execution. Binary attachment restore is out of scope.",
            "Set phaseRequiredForCompletionDisabled to true. Binary attachment restore is out of scope; restore completion must not depend on it.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-12",
            "phase-not-required-for-completion",
            "Attachment phase is not required for restore completion. Binary attachment restore is out of scope.",
        ));
    }

    // APD-13: Final validation treats attachments as metadata-only
    if !plan.final_validation_treats_as_metadata_only {
        checks.push(failed(
            "APD-13",
            "final-validation-metadata-only",
            "finalValidationTreatsAsMetadataOnly is not set. Final validation must treat attachment fields as metadata-only.",
            "Set finalValidationTreatsAsMetadataOnly to true. Final validation must not require binary attachment content to be present or validated.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "APD-13",
            "final-validation-metadata-only",
            "Final validation treats attachment fields as metadata-only.",
        ));
    }

    // Check declared operations for any blocked ones
    if let Some(ops) = &request.declared_operations {
        for op in ops {
            if op.operation.is_blocked() && op.planned {
                let op_name = format!("{:?}", op.operation);
                checks.push(failed(
                    "APD-05",
                    "declared-operation-blocked",
                    &format!(
                        "Declared operation {:?} is planned but is blocked. Binary attachment operations are not permitted.",
                        op.operation
                    ),
                    &format!(
                        "Remove {} from declared operations or set planned to false.",
                        op_name
                    ),
                ));
                blocked = true;
            }
            if op.operation.is_blocked() && op.required_for_completion {
                let op_name = format!("{:?}", op.operation);
                checks.push(failed(
                    "APD-12",
                    "declared-operation-required-blocked",
                    &format!(
                        "Declared operation {:?} is marked as required for completion but is blocked. Blocked operations cannot be required for restore completion.",
                        op.operation
                    ),
                    &format!(
                        "Set requiredForCompletion to false for {} or remove this operation.",
                        op_name
                    ),
                ));
                blocked = true;
            }
        }
    }

    // APD-14: No success state introduced (always passes)
    checks.push(passed(
        "APD-14",
        "no-success-state",
        "No restore success or completion state is introduced by this policy check.",
    ));

    // APD-15: No writes attempted (always passes)
    checks.push(passed(
        "APD-15",
        "no-writes-attempted",
        "No write operations are attempted. No Airtable API calls are made. No attachment binary is downloaded, uploaded, fetched, or transferred.",
    ));

    // APD-16: Writes remain disabled even when compliant (always passes)
    checks.push(passed(
        "APD-16",
        "writes-remain-disabled",
        "Restore writes remain disabled. Policy compliance does not enable write execution. Binary attachment restore is out of scope.",
    ));

    let blocked_operations_count = request
        .declared_operations
        .as_ref()
        .map(|ops| ops.iter().filter(|o| o.operation.is_blocked()).count())
        .unwrap_or(0);

    let phase_summary = Some(AttachmentPhaseDisabledSummary {
        metadata_inspection_enabled: plan.metadata_inspection_enabled,
        metadata_verification_enabled: plan.metadata_verification_enabled,
        binary_handling_disabled: plan.binary_handling_disabled,
        url_exposure_disabled: plan.url_exposure_disabled,
        field_mutation_disabled: plan.field_mutation_disabled,
        phase_required_for_completion_disabled: plan.phase_required_for_completion_disabled,
        final_validation_treats_as_metadata_only: plan.final_validation_treats_as_metadata_only,
        blocked_operations_declared: blocked_operations_count,
    });

    let status = if blocked {
        AttachmentPhaseDisabledPolicyStatus::Blocked
    } else if has_warning {
        AttachmentPhaseDisabledPolicyStatus::Warning
    } else {
        AttachmentPhaseDisabledPolicyStatus::Compliant
    };

    let label = request
        .target_label
        .as_ref()
        .map(|l| format!(" for '{l}'"))
        .unwrap_or_default();
    let message = match status {
        AttachmentPhaseDisabledPolicyStatus::Compliant => format!(
            "Attachment phase disabled policy is compliant{label}. All attachment operations are metadata-only. Binary attachment download, upload, fetch, transfer, field mutation, and URL exposure are explicitly disabled. Binary attachment restore is out of scope. Restore writes remain disabled."
        ),
        AttachmentPhaseDisabledPolicyStatus::Warning => format!(
            "Attachment phase disabled policy has warnings{label}. Core binary attachment safeguards are in place, but metadata verification is not enabled. Confirm that the skip reason is valid. Restore writes remain disabled."
        ),
        AttachmentPhaseDisabledPolicyStatus::Blocked => format!(
            "Attachment phase disabled policy is blocked{label}. Binary attachment download, upload, fetch, transfer, field mutation, or URL exposure is not explicitly disabled, or the attachment phase is required for completion. Binary attachment restore is out of scope. Restore writes remain disabled."
        ),
    };

    build_result(status, checks, message, phase_summary)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_plan() -> AttachmentMetadataOnlyPlan {
        AttachmentMetadataOnlyPlan {
            metadata_inspection_enabled: true,
            metadata_verification_enabled: true,
            metadata_verification_skip_reason: None,
            binary_handling_disabled: true,
            url_exposure_disabled: true,
            field_mutation_disabled: true,
            phase_required_for_completion_disabled: true,
            final_validation_treats_as_metadata_only: true,
        }
    }

    fn safe_request() -> AttachmentPhaseDisabledPolicyRequest {
        AttachmentPhaseDisabledPolicyRequest {
            plan: Some(safe_plan()),
            declared_operations: None,
            target_label: None,
        }
    }

    #[test]
    fn complete_metadata_only_plan_is_compliant() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert_eq!(
            result.status,
            AttachmentPhaseDisabledPolicyStatus::Compliant
        );
        assert!(result.no_changes_made);
        assert!(!result.writes_enabled);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn missing_plan_is_blocked() {
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: None,
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
    }

    #[test]
    fn missing_plan_short_circuits_after_apd02() {
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: None,
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.checks.len(), 2);
        assert_eq!(result.checks[0].check_id, "APD-01");
        assert_eq!(result.checks[1].check_id, "APD-02");
    }

    #[test]
    fn binary_download_blocked_when_binary_handling_not_disabled() {
        let mut plan = safe_plan();
        plan.binary_handling_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-05")
            .unwrap();
        assert_eq!(apd05.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn binary_upload_blocked_when_binary_handling_not_disabled() {
        let mut plan = safe_plan();
        plan.binary_handling_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-06")
            .unwrap();
        assert_eq!(apd06.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn url_fetch_blocked_when_url_exposure_not_disabled() {
        let mut plan = safe_plan();
        plan.url_exposure_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-07")
            .unwrap();
        assert_eq!(apd07.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn file_read_write_blocked_when_binary_handling_not_disabled() {
        let mut plan = safe_plan();
        plan.binary_handling_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-08")
            .unwrap();
        assert_eq!(apd08.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn raw_attachment_transfer_blocked_when_binary_handling_not_disabled() {
        let mut plan = safe_plan();
        plan.binary_handling_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-09")
            .unwrap();
        assert_eq!(apd09.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn attachment_field_mutation_blocked_when_not_disabled() {
        let mut plan = safe_plan();
        plan.field_mutation_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd10 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-10")
            .unwrap();
        assert_eq!(apd10.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn attachment_url_exposure_blocked_when_not_disabled() {
        let mut plan = safe_plan();
        plan.url_exposure_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd11 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-11")
            .unwrap();
        assert_eq!(apd11.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn attachment_phase_required_for_completion_is_blocked() {
        let mut plan = safe_plan();
        plan.phase_required_for_completion_disabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd12 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-12")
            .unwrap();
        assert_eq!(apd12.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn skipped_metadata_verification_with_reason_is_warning() {
        let mut plan = safe_plan();
        plan.metadata_verification_enabled = false;
        plan.metadata_verification_skip_reason =
            Some("No attachment fields in this backup".to_string());
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Warning);
        let apd04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-04")
            .unwrap();
        assert_eq!(apd04.status, AttachmentPhaseDisabledCheckStatus::Warning);
    }

    #[test]
    fn skipped_metadata_verification_without_reason_is_blocked() {
        let mut plan = safe_plan();
        plan.metadata_verification_enabled = false;
        plan.metadata_verification_skip_reason = None;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
    }

    #[test]
    fn final_validation_metadata_only_required() {
        let mut plan = safe_plan();
        plan.final_validation_treats_as_metadata_only = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd13 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-13")
            .unwrap();
        assert_eq!(apd13.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn declared_binary_download_operation_blocks() {
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(safe_plan()),
            declared_operations: Some(vec![AttachmentPhasePlan {
                operation: AttachmentPhaseOperation::BinaryDownload,
                planned: true,
                required_for_completion: false,
                justification: None,
            }]),
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
    }

    #[test]
    fn declared_binary_upload_operation_blocks() {
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(safe_plan()),
            declared_operations: Some(vec![AttachmentPhasePlan {
                operation: AttachmentPhaseOperation::BinaryUpload,
                planned: true,
                required_for_completion: false,
                justification: None,
            }]),
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
    }

    #[test]
    fn declared_url_fetch_operation_blocks() {
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(safe_plan()),
            declared_operations: Some(vec![AttachmentPhasePlan {
                operation: AttachmentPhaseOperation::UrlFetch,
                planned: true,
                required_for_completion: false,
                justification: None,
            }]),
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
    }

    #[test]
    fn declared_raw_transfer_operation_blocks() {
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(safe_plan()),
            declared_operations: Some(vec![AttachmentPhasePlan {
                operation: AttachmentPhaseOperation::RawAttachmentTransfer,
                planned: true,
                required_for_completion: false,
                justification: None,
            }]),
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
    }

    #[test]
    fn declared_field_mutation_required_for_completion_blocks() {
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(safe_plan()),
            declared_operations: Some(vec![AttachmentPhasePlan {
                operation: AttachmentPhaseOperation::AttachmentFieldMutation,
                planned: false,
                required_for_completion: true,
                justification: None,
            }]),
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
    }

    #[test]
    fn no_success_state_introduced() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        let apd14 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-14")
            .unwrap();
        assert_eq!(apd14.status, AttachmentPhaseDisabledCheckStatus::Passed);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.to_lowercase().contains("restore complete"));
        assert!(!json.to_lowercase().contains("succeeded"));
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert!(result.no_changes_made);

        let blocked_req = AttachmentPhaseDisabledPolicyRequest {
            plan: None,
            declared_operations: None,
            target_label: None,
        };
        let blocked = verify_attachment_phase_disabled_policy(&blocked_req);
        assert!(blocked.no_changes_made);
    }

    #[test]
    fn writes_enabled_is_always_false() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert!(!result.writes_enabled);

        let blocked_req = AttachmentPhaseDisabledPolicyRequest {
            plan: None,
            declared_operations: None,
            target_label: None,
        };
        let blocked = verify_attachment_phase_disabled_policy(&blocked_req);
        assert!(!blocked.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert!(!result.network_writes_attempted);

        let blocked_req = AttachmentPhaseDisabledPolicyRequest {
            plan: None,
            declared_operations: None,
            target_label: None,
        };
        let blocked = verify_attachment_phase_disabled_policy(&blocked_req);
        assert!(!blocked.network_writes_attempted);
    }

    #[test]
    fn compliant_has_sixteen_checks() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert_eq!(result.checks.len(), 16);
    }

    #[test]
    fn no_token_or_path_in_serialized_result() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("pat_"), "token prefix must not appear");
        assert!(!json.contains("/Users/"), "absolute path must not appear");
        assert!(!json.contains("/home/"), "home path must not appear");
        assert!(!json.contains("http"), "no URLs in result");
    }

    #[test]
    fn no_write_calls_made() {
        use crate::restore::write_gate::evaluate_write_gate;
        let gate_before = evaluate_write_gate();
        let _result = verify_attachment_phase_disabled_policy(&safe_request());
        let gate_after = evaluate_write_gate();
        assert!(matches!(
            gate_before.status,
            RestoreWriteEngineStatus::Disabled
        ));
        assert!(matches!(
            gate_after.status,
            RestoreWriteEngineStatus::Disabled
        ));
    }

    #[test]
    fn phase_summary_present_for_compliant() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert!(result.phase_summary.is_some());
        let summary = result.phase_summary.unwrap();
        assert!(summary.metadata_inspection_enabled);
        assert!(summary.metadata_verification_enabled);
        assert!(summary.binary_handling_disabled);
        assert!(summary.url_exposure_disabled);
        assert!(summary.field_mutation_disabled);
        assert!(summary.phase_required_for_completion_disabled);
        assert!(summary.final_validation_treats_as_metadata_only);
    }

    #[test]
    fn metadata_inspection_not_set_is_blocked() {
        let mut plan = safe_plan();
        plan.metadata_inspection_enabled = false;
        let req = AttachmentPhaseDisabledPolicyRequest {
            plan: Some(plan),
            declared_operations: None,
            target_label: None,
        };
        let result = verify_attachment_phase_disabled_policy(&req);
        assert_eq!(result.status, AttachmentPhaseDisabledPolicyStatus::Blocked);
        let apd03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "APD-03")
            .unwrap();
        assert_eq!(apd03.status, AttachmentPhaseDisabledCheckStatus::Failed);
    }

    #[test]
    fn compliant_message_contains_writes_disabled() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert!(result
            .message
            .to_lowercase()
            .contains("writes remain disabled"));
    }

    #[test]
    fn compliant_message_contains_out_of_scope() {
        let result = verify_attachment_phase_disabled_policy(&safe_request());
        assert!(result.message.to_lowercase().contains("out of scope"));
    }
}
