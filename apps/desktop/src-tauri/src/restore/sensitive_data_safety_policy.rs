use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SensitiveDataSafetyPolicyStatus {
    /// All exposure surfaces are covered by named redaction rules, and no
    /// forbidden sensitive pattern can be exposed through any surface.
    Compliant,
    /// Some surface has only generic redaction without a named rule, but no
    /// hard safety threshold is violated.
    Warning,
    /// A forbidden sensitive pattern (token, full path, record payload, raw
    /// HTTP, attachment URL) is reachable through at least one surface.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SensitiveDataSafetyCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// A surface through which sensitive data could be exposed to the UI,
/// logs, diagnostics, or serialized results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SensitiveDataExposureSurface {
    /// Tauri command return value / serialized result.
    CommandResult,
    /// UI panel display text.
    UiPanel,
    /// Diagnostic message string.
    DiagnosticMessage,
    /// Checkpoint persistence summary.
    CheckpointSummary,
    /// Validation result summary.
    ValidationSummary,
    /// Failure/error summary.
    FailureSummary,
    /// Structured or unstructured log message.
    LogMessage,
    /// Error message returned to the caller.
    ErrorMessage,
    /// Reference to a backup package (file or path).
    PackageReference,
    /// Reference to a restored record.
    RecordReference,
}

/// A class of sensitive data pattern that must not be exposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SensitiveDataPatternClass {
    /// Airtable personal-access-token (`pat_...`).
    AirtableToken,
    /// Generic API key material.
    ApiKey,
    /// Bearer token or Authorization header value.
    BearerToken,
    /// Full local filesystem path (`/Users/...`, `C:\...`).
    FullLocalPath,
    /// Path to the backup `.airbridge` package file.
    PackagePath,
    /// Serialized record payload (field names + values).
    RecordPayload,
    /// Individual field value payload.
    FieldPayload,
    /// Attachment CDN or signed URL.
    AttachmentUrl,
    /// Raw HTTP response body.
    RawHttpResponse,
    /// Raw HTTP request body.
    RawRequestBody,
}

/// Declares that a specific exposure surface is covered for a specific
/// sensitive pattern class, and names the redaction rule being applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataRedactionPlan {
    /// The surface being protected.
    pub surface: SensitiveDataExposureSurface,
    /// The sensitive pattern this rule protects against.
    pub pattern_class: SensitiveDataPatternClass,
    /// Named redaction strategy (e.g. "strip-token-field",
    /// "filename-only", "summary-message"). Must not be empty.
    pub redaction_rule: String,
    /// Whether the redaction coverage has been confirmed by a test.
    pub confirmed_by_test: bool,
}

/// Declares the complete sensitive-data safety posture for a restore
/// write pipeline before any live write is enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataSafetyPlan {
    /// All surface-level redaction declarations for this pipeline.
    pub redaction_coverage: Vec<SensitiveDataRedactionPlan>,
    /// Declares that no token field exists in any command result or result
    /// type returned to the frontend.
    pub no_token_in_results: bool,
    /// Declares that no full filesystem path appears in any result field.
    pub no_full_path_in_results: bool,
    /// Declares that package references use filename-only labels.
    pub package_references_filename_only: bool,
    /// Declares that record payload data does not appear in any result field.
    pub no_record_payload_in_results: bool,
    /// Declares that attachment URLs are not returned to the frontend.
    pub no_attachment_url_in_results: bool,
    /// Declares that raw HTTP request/response bodies are not logged or
    /// returned to the frontend.
    pub no_raw_http_in_results: bool,
    /// Declares that error and diagnostic messages use safe, structured
    /// summaries with no sensitive data embedded.
    pub error_messages_use_safe_summaries: bool,
    /// Declares that checkpoint, validation, and failure summaries contain
    /// only metadata (counts, IDs, statuses) — no field values or payloads.
    pub summaries_are_payload_free: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataSafetyPolicyRequest {
    pub plan: Option<SensitiveDataSafetyPlan>,
    /// Optional human-readable label for the restore target. No token,
    /// path, or record payload.
    pub target_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataSafetyCheck {
    pub check_id: String,
    pub label: String,
    pub status: SensitiveDataSafetyCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Safe summary emitted in the result when a plan is present.
/// Contains only metadata — no tokens, paths, or payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataSafetySummary {
    pub total_redaction_rules: usize,
    pub surfaces_covered: usize,
    pub all_rules_named: bool,
    pub no_token_in_results: bool,
    pub no_full_path_in_results: bool,
    pub package_references_filename_only: bool,
    pub no_record_payload_in_results: bool,
    pub no_attachment_url_in_results: bool,
    pub no_raw_http_in_results: bool,
    pub error_messages_use_safe_summaries: bool,
    pub summaries_are_payload_free: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataSafetyPolicyResult {
    pub status: SensitiveDataSafetyPolicyStatus,
    pub checks: Vec<SensitiveDataSafetyCheck>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_summary: Option<SensitiveDataSafetySummary>,
    /// Always true — this policy check performs no writes of any kind.
    pub no_changes_made: bool,
    /// Always false — no network writes are attempted.
    pub network_writes_attempted: bool,
    /// Always false — policy compliance does not enable restore writes.
    pub writes_enabled: bool,
}

// ── All required exposure surfaces ──────────────────────────────────────────

const ALL_SURFACES: &[SensitiveDataExposureSurface] = &[
    SensitiveDataExposureSurface::CommandResult,
    SensitiveDataExposureSurface::UiPanel,
    SensitiveDataExposureSurface::DiagnosticMessage,
    SensitiveDataExposureSurface::CheckpointSummary,
    SensitiveDataExposureSurface::ValidationSummary,
    SensitiveDataExposureSurface::FailureSummary,
    SensitiveDataExposureSurface::LogMessage,
    SensitiveDataExposureSurface::ErrorMessage,
    SensitiveDataExposureSurface::PackageReference,
    SensitiveDataExposureSurface::RecordReference,
];

// ── Helper ───────────────────────────────────────────────────────────────────

fn passed(check_id: &str, label: &str, message: &str) -> SensitiveDataSafetyCheck {
    SensitiveDataSafetyCheck {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: SensitiveDataSafetyCheckStatus::Passed,
        message: message.to_string(),
        remediation: None,
    }
}

fn warning(
    check_id: &str,
    label: &str,
    message: &str,
    remediation: &str,
) -> SensitiveDataSafetyCheck {
    SensitiveDataSafetyCheck {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: SensitiveDataSafetyCheckStatus::Warning,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}

fn failed(
    check_id: &str,
    label: &str,
    message: &str,
    remediation: &str,
) -> SensitiveDataSafetyCheck {
    SensitiveDataSafetyCheck {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: SensitiveDataSafetyCheckStatus::Failed,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}

fn build_result(
    status: SensitiveDataSafetyPolicyStatus,
    checks: Vec<SensitiveDataSafetyCheck>,
    message: String,
    safety_summary: Option<SensitiveDataSafetySummary>,
) -> SensitiveDataSafetyPolicyResult {
    SensitiveDataSafetyPolicyResult {
        status,
        checks,
        message,
        safety_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Core policy function ─────────────────────────────────────────────────────

pub fn verify_sensitive_data_safety_policy(
    request: &SensitiveDataSafetyPolicyRequest,
) -> SensitiveDataSafetyPolicyResult {
    let mut checks = Vec::new();

    // SDS-01: Write gate disabled (always passes)
    let gate = evaluate_write_gate();
    let gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    if gate_disabled {
        checks.push(passed(
            "SDS-01",
            "write-gate-disabled",
            "Write gate is disabled. No restore writes are attempted.",
        ));
    } else {
        checks.push(failed(
            "SDS-01",
            "write-gate-disabled",
            "Write gate is unexpectedly enabled. Sensitive data safety policy must not be \
             evaluated while writes are enabled.",
            "Disable the write gate before evaluating this policy.",
        ));
        return build_result(
            SensitiveDataSafetyPolicyStatus::Blocked,
            checks,
            "Sensitive data safety policy is blocked. Write gate is unexpectedly enabled. \
             Sensitive data exposure cannot be assessed while live writes are active. \
             Restore writes remain disabled."
                .to_string(),
            None,
        );
    }

    // SDS-02: Plan declared — short-circuit if absent
    let plan = match &request.plan {
        Some(p) => p,
        None => {
            checks.push(failed(
                "SDS-02",
                "plan-declared",
                "No sensitive data safety plan declared. A plan declaring redaction coverage \
                 for all exposure surfaces and sensitive pattern classes is required before \
                 any live write path is considered.",
                "Declare a SensitiveDataSafetyPlan with redaction_coverage entries for all \
                 required exposure surfaces, and set all boolean safety declarations.",
            ));
            return build_result(
                SensitiveDataSafetyPolicyStatus::Blocked,
                checks,
                "Sensitive data safety policy is blocked. No plan was declared. Tokens, full \
                 paths, record payloads, and raw HTTP data must not be exposed through any \
                 restore write result, diagnostic, or log surface. Restore writes remain disabled."
                    .to_string(),
                None,
            );
        }
    };

    checks.push(passed(
        "SDS-02",
        "plan-declared",
        "Sensitive data safety plan is declared.",
    ));

    let mut blocked = false;
    let mut has_warning = false;

    // SDS-03: Every exposure surface has redaction coverage
    let covered_surfaces: std::collections::HashSet<String> = plan
        .redaction_coverage
        .iter()
        .map(|r| format!("{:?}", r.surface))
        .collect();

    let missing_surfaces: Vec<&SensitiveDataExposureSurface> = ALL_SURFACES
        .iter()
        .filter(|s| !covered_surfaces.contains(&format!("{:?}", s)))
        .collect();

    if missing_surfaces.is_empty() {
        checks.push(passed(
            "SDS-03",
            "all-surfaces-covered",
            "All required exposure surfaces have at least one redaction coverage declaration.",
        ));
    } else {
        checks.push(failed(
            "SDS-03",
            "all-surfaces-covered",
            &format!(
                "{} exposure surface(s) have no redaction coverage declared: {}.",
                missing_surfaces.len(),
                missing_surfaces
                    .iter()
                    .map(|s| format!("{:?}", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "Add SensitiveDataRedactionPlan entries for every exposure surface listed above.",
        ));
        blocked = true;
    }

    // SDS-04: Token/API-key patterns blocked
    if !plan.no_token_in_results {
        checks.push(failed(
            "SDS-04",
            "token-api-key-blocked",
            "Plan does not declare that tokens and API keys are absent from all result fields. \
             Airtable tokens (pat_...), API keys, and bearer tokens must never appear in any \
             command result, UI panel, log, or diagnostic.",
            "Set no_token_in_results: true and confirm by serialization test.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-04",
            "token-api-key-blocked",
            "Plan declares that no token or API key appears in any result field.",
        ));
    }

    // SDS-05: Full local path patterns blocked
    if !plan.no_full_path_in_results {
        checks.push(failed(
            "SDS-05",
            "full-local-path-blocked",
            "Plan does not declare that full local filesystem paths are absent from all result \
             fields. Paths such as /Users/..., /home/..., or C:\\... must never appear in any \
             command result, UI panel, log, or diagnostic.",
            "Set no_full_path_in_results: true and confirm by serialization test.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-05",
            "full-local-path-blocked",
            "Plan declares that no full local filesystem path appears in any result field.",
        ));
    }

    // SDS-06: Package path exposure blocked; filename-only safe label allowed
    if !plan.package_references_filename_only {
        checks.push(failed(
            "SDS-06",
            "package-path-filename-only",
            "Plan does not declare that package references use filename-only labels. The full \
             path to the .airbridge package file must never appear in any result, panel, or log. \
             Only the filename (basename) is safe to display.",
            "Set package_references_filename_only: true and verify by path extraction test.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-06",
            "package-path-filename-only",
            "Plan declares that package references use filename-only safe labels. Full package \
             paths are not exposed.",
        ));
    }

    // SDS-07: Record/field payload exposure blocked
    if !plan.no_record_payload_in_results {
        checks.push(failed(
            "SDS-07",
            "record-field-payload-blocked",
            "Plan does not declare that record and field payloads are absent from all results. \
             Serialized record data (field names + values), field-level payloads, and raw record \
             bodies must never appear in any command result or UI panel.",
            "Set no_record_payload_in_results: true and confirm no record payload field exists \
             in any result type.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-07",
            "record-field-payload-blocked",
            "Plan declares that no record or field payload appears in any result field.",
        ));
    }

    // SDS-08: Attachment URL/raw URL exposure blocked
    if !plan.no_attachment_url_in_results {
        checks.push(failed(
            "SDS-08",
            "attachment-url-blocked",
            "Plan does not declare that attachment URLs are blocked from results. CDN URLs and \
             signed attachment URLs must never appear in any command result, UI panel, or log.",
            "Set no_attachment_url_in_results: true and verify no attachment URL field exists \
             in any result type.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-08",
            "attachment-url-blocked",
            "Plan declares that no attachment URL appears in any result field.",
        ));
    }

    // SDS-09: Raw HTTP request/response exposure blocked
    if !plan.no_raw_http_in_results {
        checks.push(failed(
            "SDS-09",
            "raw-http-blocked",
            "Plan does not declare that raw HTTP request and response bodies are blocked. \
             Raw HTTP bodies may contain tokens, record payloads, or attachment URLs and must \
             never be returned to the frontend or written to logs.",
            "Set no_raw_http_in_results: true.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-09",
            "raw-http-blocked",
            "Plan declares that raw HTTP request and response bodies are not exposed.",
        ));
    }

    // SDS-10: Error/diagnostic messages use safe summaries only
    if !plan.error_messages_use_safe_summaries {
        checks.push(failed(
            "SDS-10",
            "error-messages-safe-summaries",
            "Plan does not declare that error and diagnostic messages use safe, structured \
             summaries. Error messages must not embed token values, full paths, record payloads, \
             or raw HTTP bodies.",
            "Set error_messages_use_safe_summaries: true. Use structured error codes and \
             human-readable summaries with no raw sensitive data.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-10",
            "error-messages-safe-summaries",
            "Plan declares that error and diagnostic messages use safe summaries with no \
             sensitive data embedded.",
        ));
    }

    // SDS-11: Checkpoint/validation/failure summaries are payload-free
    if !plan.summaries_are_payload_free {
        checks.push(failed(
            "SDS-11",
            "summaries-payload-free",
            "Plan does not declare that checkpoint, validation, and failure summaries are \
             payload-free. These summaries must contain only metadata (counts, IDs, statuses) \
             — no field values, record payloads, or attachment URLs.",
            "Set summaries_are_payload_free: true.",
        ));
        blocked = true;
    } else {
        checks.push(passed(
            "SDS-11",
            "summaries-payload-free",
            "Plan declares that checkpoint, validation, and failure summaries contain only \
             metadata — no field values or payloads.",
        ));
    }

    // SDS-12: Redaction rules are named (warning if any rule has empty name)
    let unnamed_rules: Vec<usize> = plan
        .redaction_coverage
        .iter()
        .enumerate()
        .filter(|(_, r)| r.redaction_rule.trim().is_empty())
        .map(|(i, _)| i)
        .collect();

    if !unnamed_rules.is_empty() {
        checks.push(warning(
            "SDS-12",
            "redaction-rules-named",
            &format!(
                "{} redaction rule(s) have an empty redaction_rule name. Named rules improve \
                 auditability but do not block policy compliance.",
                unnamed_rules.len()
            ),
            "Provide a descriptive redaction_rule name for each coverage entry.",
        ));
        has_warning = true;
    } else {
        checks.push(passed(
            "SDS-12",
            "redaction-rules-named",
            "All redaction rules have non-empty names.",
        ));
    }

    // SDS-13: No success/completion state introduced (always passes)
    checks.push(passed(
        "SDS-13",
        "no-success-state-introduced",
        "No success or completion state is introduced by this policy check. Restore writes \
         remain disabled.",
    ));

    // SDS-14: No token/path/payload fields in serialized result (always passes)
    checks.push(passed(
        "SDS-14",
        "no-token-path-payload-in-result",
        "No token, filesystem path, or record payload field exists in this result type. \
         Verified by policy design and serialization test.",
    ));

    // SDS-15: Writes remain disabled even when compliant (always passes)
    checks.push(passed(
        "SDS-15",
        "writes-remain-disabled",
        "Restore writes remain disabled. Policy compliance does not enable write execution.",
    ));

    // Build summary
    let all_named = plan
        .redaction_coverage
        .iter()
        .all(|r| !r.redaction_rule.trim().is_empty());

    let surfaces_covered = covered_surfaces.len();

    let safety_summary = Some(SensitiveDataSafetySummary {
        total_redaction_rules: plan.redaction_coverage.len(),
        surfaces_covered,
        all_rules_named: all_named,
        no_token_in_results: plan.no_token_in_results,
        no_full_path_in_results: plan.no_full_path_in_results,
        package_references_filename_only: plan.package_references_filename_only,
        no_record_payload_in_results: plan.no_record_payload_in_results,
        no_attachment_url_in_results: plan.no_attachment_url_in_results,
        no_raw_http_in_results: plan.no_raw_http_in_results,
        error_messages_use_safe_summaries: plan.error_messages_use_safe_summaries,
        summaries_are_payload_free: plan.summaries_are_payload_free,
    });

    let status = if blocked {
        SensitiveDataSafetyPolicyStatus::Blocked
    } else if has_warning {
        SensitiveDataSafetyPolicyStatus::Warning
    } else {
        SensitiveDataSafetyPolicyStatus::Compliant
    };

    let label = request
        .target_label
        .as_deref()
        .map(|l| format!(" for '{l}'"))
        .unwrap_or_default();

    let message = match status {
        SensitiveDataSafetyPolicyStatus::Compliant => format!(
            "Sensitive data safety policy is compliant{label}. All exposure surfaces are covered \
             by named redaction rules. Tokens, full paths, package paths, record payloads, \
             attachment URLs, and raw HTTP data are blocked from all result fields, UI panels, \
             diagnostics, and logs. Restore writes remain disabled."
        ),
        SensitiveDataSafetyPolicyStatus::Warning => format!(
            "Sensitive data safety policy has warnings{label}. All hard safety thresholds are \
             satisfied, but one or more redaction rules lack a descriptive name. Restore writes \
             remain disabled."
        ),
        SensitiveDataSafetyPolicyStatus::Blocked => format!(
            "Sensitive data safety policy is blocked{label}. One or more sensitive data patterns \
             — tokens, full paths, record payloads, attachment URLs, or raw HTTP data — can be \
             exposed through a restore write result, diagnostic, or log surface. Resolve all \
             violations before any live write is considered. Restore writes remain disabled."
        ),
    };

    build_result(status, checks, message, safety_summary)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_surfaces_coverage() -> Vec<SensitiveDataRedactionPlan> {
        ALL_SURFACES
            .iter()
            .map(|surface| SensitiveDataRedactionPlan {
                surface: surface.clone(),
                pattern_class: SensitiveDataPatternClass::AirtableToken,
                redaction_rule: "strip-token-field".to_string(),
                confirmed_by_test: true,
            })
            .collect()
    }

    fn safe_plan() -> SensitiveDataSafetyPlan {
        SensitiveDataSafetyPlan {
            redaction_coverage: all_surfaces_coverage(),
            no_token_in_results: true,
            no_full_path_in_results: true,
            package_references_filename_only: true,
            no_record_payload_in_results: true,
            no_attachment_url_in_results: true,
            no_raw_http_in_results: true,
            error_messages_use_safe_summaries: true,
            summaries_are_payload_free: true,
        }
    }

    fn request_with_plan(plan: SensitiveDataSafetyPlan) -> SensitiveDataSafetyPolicyRequest {
        SensitiveDataSafetyPolicyRequest {
            plan: Some(plan),
            target_label: None,
        }
    }

    fn request_no_plan() -> SensitiveDataSafetyPolicyRequest {
        SensitiveDataSafetyPolicyRequest {
            plan: None,
            target_label: None,
        }
    }

    #[test]
    fn complete_safe_plan_is_compliant() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Compliant);
    }

    #[test]
    fn missing_plan_is_blocked() {
        let result = verify_sensitive_data_safety_policy(&request_no_plan());
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn missing_plan_short_circuits_after_sds02() {
        let result = verify_sensitive_data_safety_policy(&request_no_plan());
        assert_eq!(result.checks[0].check_id, "SDS-01");
        assert_eq!(result.checks[1].check_id, "SDS-02");
        assert_eq!(
            result.checks[1].status,
            SensitiveDataSafetyCheckStatus::Failed
        );
    }

    #[test]
    fn missing_surface_coverage_is_blocked() {
        let mut plan = safe_plan();
        // Remove CommandResult coverage
        plan.redaction_coverage
            .retain(|r| r.surface != SensitiveDataExposureSurface::CommandResult);
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-03")
            .unwrap();
        assert_eq!(sds03.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn token_not_blocked_in_results_is_blocked() {
        let mut plan = safe_plan();
        plan.no_token_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-04")
            .unwrap();
        assert_eq!(sds04.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn api_key_coverage_missing_is_blocked_via_sds04() {
        let mut plan = safe_plan();
        plan.no_token_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
    }

    #[test]
    fn bearer_token_no_token_flag_false_is_blocked() {
        let mut plan = safe_plan();
        plan.no_token_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        let sds04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-04")
            .unwrap();
        assert!(sds04.remediation.is_some());
    }

    #[test]
    fn full_local_path_not_blocked_is_blocked() {
        let mut plan = safe_plan();
        plan.no_full_path_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-05")
            .unwrap();
        assert_eq!(sds05.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn package_path_not_filename_only_is_blocked() {
        let mut plan = safe_plan();
        plan.package_references_filename_only = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-06")
            .unwrap();
        assert_eq!(sds06.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn filename_only_package_label_is_allowed() {
        let mut plan = safe_plan();
        plan.package_references_filename_only = true;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        let sds06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-06")
            .unwrap();
        assert_eq!(sds06.status, SensitiveDataSafetyCheckStatus::Passed);
    }

    #[test]
    fn record_payload_not_blocked_is_blocked() {
        let mut plan = safe_plan();
        plan.no_record_payload_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-07")
            .unwrap();
        assert_eq!(sds07.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn field_payload_no_record_payload_flag_false_is_blocked() {
        let mut plan = safe_plan();
        plan.no_record_payload_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
    }

    #[test]
    fn attachment_url_not_blocked_is_blocked() {
        let mut plan = safe_plan();
        plan.no_attachment_url_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-08")
            .unwrap();
        assert_eq!(sds08.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn raw_http_not_blocked_is_blocked() {
        let mut plan = safe_plan();
        plan.no_raw_http_in_results = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-09")
            .unwrap();
        assert_eq!(sds09.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn error_messages_not_safe_is_blocked() {
        let mut plan = safe_plan();
        plan.error_messages_use_safe_summaries = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds10 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-10")
            .unwrap();
        assert_eq!(sds10.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn summaries_not_payload_free_is_blocked() {
        let mut plan = safe_plan();
        plan.summaries_are_payload_free = false;
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Blocked);
        let sds11 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-11")
            .unwrap();
        assert_eq!(sds11.status, SensitiveDataSafetyCheckStatus::Failed);
    }

    #[test]
    fn unnamed_redaction_rule_is_warning() {
        let mut plan = safe_plan();
        // Set first rule to empty name
        if let Some(first) = plan.redaction_coverage.first_mut() {
            first.redaction_rule = String::new();
        }
        let result = verify_sensitive_data_safety_policy(&request_with_plan(plan));
        assert_eq!(result.status, SensitiveDataSafetyPolicyStatus::Warning);
        let sds12 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-12")
            .unwrap();
        assert_eq!(sds12.status, SensitiveDataSafetyCheckStatus::Warning);
    }

    #[test]
    fn no_success_state_introduced() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        let sds13 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-13")
            .unwrap();
        assert_eq!(sds13.status, SensitiveDataSafetyCheckStatus::Passed);
    }

    #[test]
    fn complete_plan_has_fifteen_checks() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.checks.len(), 15);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let result_compliant = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        assert!(result_compliant.no_changes_made);
        let result_blocked = verify_sensitive_data_safety_policy(&request_no_plan());
        assert!(result_blocked.no_changes_made);
    }

    #[test]
    fn writes_enabled_is_always_false() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        assert!(!result.writes_enabled);
        let result_blocked = verify_sensitive_data_safety_policy(&request_no_plan());
        assert!(!result_blocked.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
        let result_blocked = verify_sensitive_data_safety_policy(&request_no_plan());
        assert!(!result_blocked.network_writes_attempted);
    }

    #[test]
    fn sds13_sds14_sds15_always_pass() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        let sds13 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-13")
            .unwrap();
        let sds14 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-14")
            .unwrap();
        let sds15 = result
            .checks
            .iter()
            .find(|c| c.check_id == "SDS-15")
            .unwrap();
        assert_eq!(sds13.status, SensitiveDataSafetyCheckStatus::Passed);
        assert_eq!(sds14.status, SensitiveDataSafetyCheckStatus::Passed);
        assert_eq!(sds15.status, SensitiveDataSafetyCheckStatus::Passed);
    }

    #[test]
    fn safety_summary_present_for_complete_plan() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        assert!(result.safety_summary.is_some());
        let summary = result.safety_summary.unwrap();
        assert!(summary.no_token_in_results);
        assert!(summary.no_full_path_in_results);
        assert!(summary.package_references_filename_only);
        assert!(summary.no_record_payload_in_results);
        assert!(summary.all_rules_named);
    }

    #[test]
    fn safety_summary_absent_when_no_plan() {
        let result = verify_sensitive_data_safety_policy(&request_no_plan());
        assert!(result.safety_summary.is_none());
    }

    #[test]
    fn no_token_or_path_in_serialized_result() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("apiKey"));
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("\"token\":"));
    }

    #[test]
    fn no_success_state_in_result_message() {
        let result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.to_lowercase().contains("succeeded"));
        assert!(!result.message.to_lowercase().contains("restore complete"));
    }

    #[test]
    fn target_label_appears_in_compliant_message() {
        let request = SensitiveDataSafetyPolicyRequest {
            plan: Some(safe_plan()),
            target_label: Some("test-base".to_string()),
        };
        let result = verify_sensitive_data_safety_policy(&request);
        assert!(result.message.contains("test-base"));
    }

    #[test]
    fn no_write_calls_made() {
        use crate::restore::write_gate::evaluate_write_gate;
        use crate::restore::write_result::RestoreWriteEngineStatus;
        let gate_before = evaluate_write_gate();
        let _result = verify_sensitive_data_safety_policy(&request_with_plan(safe_plan()));
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
}
