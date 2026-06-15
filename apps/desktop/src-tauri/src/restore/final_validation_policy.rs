use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationPolicyStatus {
    /// All required final validation checks are declared and safe.
    Compliant,
    /// One or more validation steps are incomplete or degraded, but no hard
    /// safety threshold is violated.
    Warning,
    /// A required final validation step is missing or an unsafe condition
    /// exists. The restore cannot proceed.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationCheckStatus {
    Passed,
    Warning,
    Failed,
}

// ── Plan struct ───────────────────────────────────────────────────────────────

/// Declared final validation plan for a future restore write operation.
///
/// All fields are boolean flags — no token, no path, no record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationPlan {
    /// Whether the plan includes schema count validation (tables, fields).
    pub has_schema_count_validation: bool,
    /// Whether the plan includes per-table and per-field presence checks.
    pub has_table_field_validation: bool,
    /// Whether the plan includes record count validation against the manifest.
    pub has_record_count_validation: bool,
    /// Whether the plan includes old-to-new ID mapping validation before linked
    /// records are considered valid.
    pub has_id_mapping_validation: bool,
    /// Whether the plan includes linked record second-pass validation.
    pub has_linked_record_validation: bool,
    /// Whether the plan includes attachment metadata validation.
    pub has_attachment_metadata_validation: bool,
    /// Whether attachment validation is limited to metadata only (no file download).
    pub attachment_validation_metadata_only: bool,
    /// Whether the plan includes final package checksum / manifest reference
    /// validation when a manifest is available.
    pub has_manifest_checksum_validation: bool,
    /// Whether the plan explicitly blocks the restore result from carrying
    /// a success status without all validation checks passing.
    pub blocks_success_without_validation: bool,
}

// ── Request / Result structs ──────────────────────────────────────────────────

/// Input to the final validation policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationPolicyRequest {
    /// The declared final validation plan for the restore write operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<FinalValidationPlan>,
    /// Safe display label for the restore target (base name only, no path, no token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationCheck {
    pub check_id: String,
    pub label: String,
    pub status: FinalValidationCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Read-only summary of the evaluated plan fields, safe for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationPlanSummary {
    pub has_schema_count_validation: bool,
    pub has_table_field_validation: bool,
    pub has_record_count_validation: bool,
    pub has_id_mapping_validation: bool,
    pub has_linked_record_validation: bool,
    pub has_attachment_metadata_validation: bool,
    pub attachment_validation_metadata_only: bool,
    pub has_manifest_checksum_validation: bool,
    pub blocks_success_without_validation: bool,
}

/// Result from `verify_final_validation_policy`.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - `Compliant` does NOT enable restore writes.
/// - `Compliant` does NOT introduce a restore success state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationPolicyResult {
    pub status: FinalValidationPolicyStatus,
    pub checks: Vec<FinalValidationCheck>,
    pub message: String,
    /// Safe, human-readable summary of the evaluated plan (no sensitive values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<FinalValidationPlanSummary>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the final validation policy for a planned restore write operation.
///
/// Check IDs:
/// - FVP-01: Write gate is disabled.
/// - FVP-02: Final validation plan is declared.
/// - FVP-03: Schema count validation is declared.
/// - FVP-04: Table/field validation is declared.
/// - FVP-05: Record count validation is declared.
/// - FVP-06: Old-to-new ID mapping validation is declared.
/// - FVP-07: Linked record second-pass validation is declared.
/// - FVP-08: Attachment metadata validation is declared.
/// - FVP-09: Attachment validation is not file-download based (metadata-only warning).
/// - FVP-10: Manifest checksum/reference validation is declared.
/// - FVP-11: Success status blocked without validation passing.
/// - FVP-12: Writes remain disabled.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
/// `Compliant` does not introduce a restore success state.
pub fn verify_final_validation_policy(
    request: &FinalValidationPolicyRequest,
) -> FinalValidationPolicyResult {
    let mut checks: Vec<FinalValidationCheck> = Vec::new();

    // FVP-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(FinalValidationCheck {
            check_id: "FVP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Restore write gate is disabled. No live writes are possible.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "Restore write gate is unexpectedly enabled.".to_string(),
            remediation: Some(
                "The write gate must remain disabled until all safety gates are satisfied."
                    .to_string(),
            ),
        });
    }

    // FVP-02: Plan declared
    let plan = match &request.plan {
        Some(p) => p,
        None => {
            checks.push(FinalValidationCheck {
                check_id: "FVP-02".to_string(),
                label: "plan-declared".to_string(),
                status: FinalValidationCheckStatus::Failed,
                message: "No final validation plan declared. A plan is required before any live \
                           write path is considered."
                    .to_string(),
                remediation: Some(
                    "Declare a FinalValidationPlan with all required validation steps.".to_string(),
                ),
            });
            // Cannot evaluate remaining checks without a plan.
            return build_result(
                checks,
                None,
                FinalValidationPolicyStatus::Blocked,
                request.target_label.as_deref(),
            );
        }
    };

    checks.push(FinalValidationCheck {
        check_id: "FVP-02".to_string(),
        label: "plan-declared".to_string(),
        status: FinalValidationCheckStatus::Passed,
        message: "Final validation plan is declared.".to_string(),
        remediation: None,
    });

    // FVP-03: Schema count validation
    if !plan.has_schema_count_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-03".to_string(),
            label: "schema-count-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "Schema count validation is not declared. Table and field counts must be \
                       verified against the backup manifest after restore."
                .to_string(),
            remediation: Some(
                "Declare has_schema_count_validation: true in the final validation plan."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-03".to_string(),
            label: "schema-count-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Schema count validation is declared.".to_string(),
            remediation: None,
        });
    }

    // FVP-04: Table/field validation
    if !plan.has_table_field_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-04".to_string(),
            label: "table-field-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "Table and field presence validation is not declared. Each table and field \
                       from the manifest must be confirmed present in the restored base."
                .to_string(),
            remediation: Some(
                "Declare has_table_field_validation: true in the final validation plan."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-04".to_string(),
            label: "table-field-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Table and field presence validation is declared.".to_string(),
            remediation: None,
        });
    }

    // FVP-05: Record count validation
    if !plan.has_record_count_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-05".to_string(),
            label: "record-count-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "Record count validation is not declared. Restored record counts must be \
                       verified against the backup manifest before restore is considered complete."
                .to_string(),
            remediation: Some(
                "Declare has_record_count_validation: true in the final validation plan."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-05".to_string(),
            label: "record-count-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Record count validation is declared.".to_string(),
            remediation: None,
        });
    }

    // FVP-06: ID mapping validation
    if !plan.has_id_mapping_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-06".to_string(),
            label: "id-mapping-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message:
                "Old-to-new record ID mapping validation is not declared. ID mapping \
                       completeness must be verified before linked records can be considered valid."
                    .to_string(),
            remediation: Some(
                "Declare has_id_mapping_validation: true in the final validation plan.".to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-06".to_string(),
            label: "id-mapping-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Old-to-new ID mapping validation is declared.".to_string(),
            remediation: None,
        });
    }

    // FVP-07: Linked record second-pass validation
    if !plan.has_linked_record_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-07".to_string(),
            label: "linked-record-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "Linked record second-pass validation is not declared. Linked field \
                       references must be verified after the second-pass write phase completes."
                .to_string(),
            remediation: Some(
                "Declare has_linked_record_validation: true in the final validation plan."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-07".to_string(),
            label: "linked-record-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Linked record second-pass validation is declared.".to_string(),
            remediation: None,
        });
    }

    // FVP-08: Attachment metadata validation
    if !plan.has_attachment_metadata_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-08".to_string(),
            label: "attachment-metadata-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "Attachment metadata validation is not declared. Attachment field metadata \
                       must be validated against the backup manifest."
                .to_string(),
            remediation: Some(
                "Declare has_attachment_metadata_validation: true in the final validation plan."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-08".to_string(),
            label: "attachment-metadata-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Attachment metadata validation is declared.".to_string(),
            remediation: None,
        });
    }

    // FVP-09: Attachment validation metadata-only warning
    if plan.has_attachment_metadata_validation && plan.attachment_validation_metadata_only {
        checks.push(FinalValidationCheck {
            check_id: "FVP-09".to_string(),
            label: "attachment-validation-scope".to_string(),
            status: FinalValidationCheckStatus::Warning,
            message: "Attachment validation is metadata-only. Attachment file content integrity \
                       cannot be confirmed without a file download. Manual re-attachment is \
                       required after restore."
                .to_string(),
            remediation: Some(
                "Note that attachment file content is not validated. Plan for manual \
                 re-attachment of all attachment fields after restore."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-09".to_string(),
            label: "attachment-validation-scope".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Attachment validation scope is acceptable.".to_string(),
            remediation: None,
        });
    }

    // FVP-10: Manifest checksum/reference validation
    if !plan.has_manifest_checksum_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-10".to_string(),
            label: "manifest-checksum-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "Manifest checksum and reference validation is not declared. The final \
                       package checksum must be verified against the backup manifest when \
                       available."
                .to_string(),
            remediation: Some(
                "Declare has_manifest_checksum_validation: true in the final validation plan."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-10".to_string(),
            label: "manifest-checksum-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Manifest checksum and reference validation is declared.".to_string(),
            remediation: None,
        });
    }

    // FVP-11: Success status blocked without validation
    if !plan.blocks_success_without_validation {
        checks.push(FinalValidationCheck {
            check_id: "FVP-11".to_string(),
            label: "success-blocked-without-validation".to_string(),
            status: FinalValidationCheckStatus::Failed,
            message: "The restore result is not declared to block success status without \
                       validation passing. A restore must not be marked succeeded unless all \
                       final validation checks pass."
                .to_string(),
            remediation: Some(
                "Declare blocks_success_without_validation: true in the final validation plan."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FinalValidationCheck {
            check_id: "FVP-11".to_string(),
            label: "success-blocked-without-validation".to_string(),
            status: FinalValidationCheckStatus::Passed,
            message: "Restore success is blocked until all validation checks pass.".to_string(),
            remediation: None,
        });
    }

    // FVP-12: Writes remain disabled
    checks.push(FinalValidationCheck {
        check_id: "FVP-12".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: FinalValidationCheckStatus::Passed,
        message: "Restore write gate remains disabled. Final validation policy compliance does \
                   not enable writes."
            .to_string(),
        remediation: None,
    });

    // Aggregate status
    let has_failed = checks
        .iter()
        .any(|c| c.status == FinalValidationCheckStatus::Failed);
    let has_warning = checks
        .iter()
        .any(|c| c.status == FinalValidationCheckStatus::Warning);

    let aggregate_status = if has_failed {
        FinalValidationPolicyStatus::Blocked
    } else if has_warning {
        FinalValidationPolicyStatus::Warning
    } else {
        FinalValidationPolicyStatus::Compliant
    };

    build_result(
        checks,
        Some(plan),
        aggregate_status,
        request.target_label.as_deref(),
    )
}

// ── Builder ───────────────────────────────────────────────────────────────────

fn build_result(
    checks: Vec<FinalValidationCheck>,
    plan: Option<&FinalValidationPlan>,
    status: FinalValidationPolicyStatus,
    target_label: Option<&str>,
) -> FinalValidationPolicyResult {
    let label_suffix = target_label
        .map(|l| format!(" for target \"{l}\""))
        .unwrap_or_default();

    let message = match status {
        FinalValidationPolicyStatus::Compliant => {
            format!(
                "Final validation plan is complete and within safe bounds{label_suffix}. \
                 Restore writes remain disabled — compliance does not enable writes or \
                 introduce a restore success state."
            )
        }
        FinalValidationPolicyStatus::Warning => {
            format!(
                "Final validation plan has warnings{label_suffix}. Review incomplete validation \
                 steps before proceeding. Restore writes remain disabled."
            )
        }
        FinalValidationPolicyStatus::Blocked => {
            format!(
                "Final validation plan is blocked{label_suffix}. Resolve all missing validation \
                 steps before any live write is considered. Restore writes remain disabled."
            )
        }
    };

    let plan_summary = plan.map(|p| FinalValidationPlanSummary {
        has_schema_count_validation: p.has_schema_count_validation,
        has_table_field_validation: p.has_table_field_validation,
        has_record_count_validation: p.has_record_count_validation,
        has_id_mapping_validation: p.has_id_mapping_validation,
        has_linked_record_validation: p.has_linked_record_validation,
        has_attachment_metadata_validation: p.has_attachment_metadata_validation,
        attachment_validation_metadata_only: p.attachment_validation_metadata_only,
        has_manifest_checksum_validation: p.has_manifest_checksum_validation,
        blocks_success_without_validation: p.blocks_success_without_validation,
    });

    FinalValidationPolicyResult {
        status,
        checks,
        message,
        plan_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_plan() -> FinalValidationPlan {
        FinalValidationPlan {
            has_schema_count_validation: true,
            has_table_field_validation: true,
            has_record_count_validation: true,
            has_id_mapping_validation: true,
            has_linked_record_validation: true,
            has_attachment_metadata_validation: true,
            attachment_validation_metadata_only: false,
            has_manifest_checksum_validation: true,
            blocks_success_without_validation: true,
        }
    }

    fn request_with_plan(plan: FinalValidationPlan) -> FinalValidationPolicyRequest {
        FinalValidationPolicyRequest {
            plan: Some(plan),
            target_label: None,
        }
    }

    fn request_no_plan() -> FinalValidationPolicyRequest {
        FinalValidationPolicyRequest {
            plan: None,
            target_label: None,
        }
    }

    // ── Status ────────────────────────────────────────────────────────────────

    #[test]
    fn complete_plan_returns_compliant() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, FinalValidationPolicyStatus::Compliant);
    }

    #[test]
    fn no_plan_returns_blocked() {
        let result = verify_final_validation_policy(&request_no_plan());
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn missing_schema_count_validation_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_schema_count_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn missing_table_field_validation_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_table_field_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn missing_record_count_validation_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_record_count_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn missing_id_mapping_validation_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_id_mapping_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn missing_linked_record_validation_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_linked_record_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn missing_attachment_metadata_validation_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_attachment_metadata_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn missing_manifest_checksum_validation_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_manifest_checksum_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn blocks_success_false_returns_blocked() {
        let mut plan = safe_plan();
        plan.blocks_success_without_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Blocked);
    }

    #[test]
    fn metadata_only_attachment_returns_warning() {
        let mut plan = safe_plan();
        plan.attachment_validation_metadata_only = true;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        assert_eq!(result.status, FinalValidationPolicyStatus::Warning);
    }

    // ── Check count ───────────────────────────────────────────────────────────

    #[test]
    fn complete_plan_produces_12_checks() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.checks.len(), 12);
    }

    #[test]
    fn no_plan_produces_2_checks() {
        let result = verify_final_validation_policy(&request_no_plan());
        assert_eq!(result.checks.len(), 2);
    }

    // ── Check IDs ─────────────────────────────────────────────────────────────

    #[test]
    fn check_ids_are_fvp_01_through_fvp_12() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "FVP-01", "FVP-02", "FVP-03", "FVP-04", "FVP-05", "FVP-06", "FVP-07", "FVP-08",
                "FVP-09", "FVP-10", "FVP-11", "FVP-12"
            ]
        );
    }

    #[test]
    fn no_plan_check_ids_are_fvp_01_and_fvp_02() {
        let result = verify_final_validation_policy(&request_no_plan());
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert_eq!(ids, vec!["FVP-01", "FVP-02"]);
    }

    // ── FVP-01 and FVP-12 always pass ─────────────────────────────────────────

    #[test]
    fn fvp_01_always_passes_with_plan() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        let fvp01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-01")
            .unwrap();
        assert_eq!(fvp01.status, FinalValidationCheckStatus::Passed);
    }

    #[test]
    fn fvp_01_always_passes_without_plan() {
        let result = verify_final_validation_policy(&request_no_plan());
        let fvp01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-01")
            .unwrap();
        assert_eq!(fvp01.status, FinalValidationCheckStatus::Passed);
    }

    #[test]
    fn fvp_12_always_passes() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        let fvp12 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-12")
            .unwrap();
        assert_eq!(fvp12.status, FinalValidationCheckStatus::Passed);
    }

    // ── Individual check statuses ─────────────────────────────────────────────

    #[test]
    fn fvp_03_fails_when_schema_count_missing() {
        let mut plan = safe_plan();
        plan.has_schema_count_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        let fvp03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-03")
            .unwrap();
        assert_eq!(fvp03.status, FinalValidationCheckStatus::Failed);
    }

    #[test]
    fn fvp_05_fails_when_record_count_missing() {
        let mut plan = safe_plan();
        plan.has_record_count_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        let fvp05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-05")
            .unwrap();
        assert_eq!(fvp05.status, FinalValidationCheckStatus::Failed);
    }

    #[test]
    fn fvp_06_fails_when_id_mapping_validation_missing() {
        let mut plan = safe_plan();
        plan.has_id_mapping_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        let fvp06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-06")
            .unwrap();
        assert_eq!(fvp06.status, FinalValidationCheckStatus::Failed);
    }

    #[test]
    fn fvp_09_warns_when_attachment_metadata_only() {
        let mut plan = safe_plan();
        plan.attachment_validation_metadata_only = true;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        let fvp09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-09")
            .unwrap();
        assert_eq!(fvp09.status, FinalValidationCheckStatus::Warning);
    }

    #[test]
    fn fvp_11_fails_when_blocks_success_false() {
        let mut plan = safe_plan();
        plan.blocks_success_without_validation = false;
        let result = verify_final_validation_policy(&request_with_plan(plan));
        let fvp11 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVP-11")
            .unwrap();
        assert_eq!(fvp11.status, FinalValidationCheckStatus::Failed);
    }

    // ── Plan summary ──────────────────────────────────────────────────────────

    #[test]
    fn plan_summary_present_when_plan_provided() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert!(result.plan_summary.is_some());
    }

    #[test]
    fn plan_summary_absent_when_no_plan() {
        let result = verify_final_validation_policy(&request_no_plan());
        assert!(result.plan_summary.is_none());
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true_compliant() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert!(result.no_changes_made);
    }

    #[test]
    fn no_changes_made_always_true_blocked() {
        let result = verify_final_validation_policy(&request_no_plan());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_compliant() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn network_writes_attempted_always_false_blocked() {
        let result = verify_final_validation_policy(&request_no_plan());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false_compliant() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert!(!result.writes_enabled);
    }

    #[test]
    fn writes_enabled_always_false_blocked() {
        let result = verify_final_validation_policy(&request_no_plan());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn compliant_does_not_enable_writes() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, FinalValidationPolicyStatus::Compliant);
        assert!(!result.writes_enabled);
    }

    // ── Serialization safety ──────────────────────────────────────────────────

    #[test]
    fn result_does_not_serialize_token() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn result_does_not_serialize_path() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\\\"));
    }

    #[test]
    fn result_does_not_serialize_record_payload() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("\"fields\""));
        assert!(!json.contains("\"recordId\""));
    }

    // ── Message safety ────────────────────────────────────────────────────────

    #[test]
    fn message_says_writes_remain_disabled_when_compliant() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, FinalValidationPolicyStatus::Compliant);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn message_does_not_contain_succeeded_when_compliant() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.to_lowercase().contains("succeeded"));
    }

    #[test]
    fn message_does_not_contain_token() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.contains("pat_"));
        assert!(!result.message.contains("apiKey"));
    }

    #[test]
    fn message_does_not_contain_absolute_path() {
        let result = verify_final_validation_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.contains("/Users/"));
        assert!(!result.message.contains("/home/"));
    }
}
