use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationEnforcementPolicyStatus {
    /// All required validation states are explicitly passed or explicitly
    /// non-required with a safe documented reason. No result can be treated
    /// as complete without final validation having passed.
    Compliant,
    /// Some validation is metadata-only or non-required with a weak reason,
    /// but no hard safety threshold is violated.
    Warning,
    /// A required validation state is missing, failed, partial, or skipped
    /// unsafely. No result can be labeled complete or successful.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationEnforcementCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// The state of a specific validation step in the restore pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidationCompletionState {
    /// Validation ran and passed all assertions.
    Passed,
    /// Validation ran but found a discrepancy or assertion failure.
    Failed,
    /// Validation was not run and is not required for this operation type.
    /// Must include a non-empty documented reason.
    NotRequired,
    /// Validation was explicitly skipped. Only allowed with a safe documented
    /// reason; otherwise triggers Blocked.
    Skipped,
    /// Validation is running or has only partially completed.
    Partial,
    /// Validation state has not been declared.
    NotDeclared,
}

impl ValidationCompletionState {
    #[cfg(test)]
    fn is_safe_for_completion(&self) -> bool {
        matches!(self, ValidationCompletionState::Passed)
    }

    fn is_blocking(&self) -> bool {
        matches!(
            self,
            ValidationCompletionState::Failed
                | ValidationCompletionState::Partial
                | ValidationCompletionState::NotDeclared
        )
    }
}

/// A guard that prevents any result from being labeled complete or successful
/// unless final validation has explicitly passed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCompletionGuard {
    /// Explicitly declares that no result status may be set to any
    /// complete/success equivalent without final validation passing.
    pub blocks_completion_without_final_validation: bool,
    /// Explicitly declares that no partial validation state may be treated
    /// as a completion state.
    pub blocks_partial_validation_as_completion: bool,
    /// Explicitly declares that a failed validation unconditionally blocks
    /// the completion result.
    pub failed_validation_blocks_completion: bool,
    /// Optional human-readable note. No token, path, or record payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Enforcement plan declaring the state of each required validation step
/// and the completion guard for the restore write pipeline.
///
/// All fields are enums, booleans, or short strings — no token, no path,
/// no record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationEnforcementPlan {
    /// State of schema (tables/fields) validation.
    pub schema_validation_state: ValidationCompletionState,
    /// Non-required reason for schema validation (required when state is NotRequired).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_validation_non_required_reason: Option<String>,
    /// State of record count validation against the manifest.
    pub record_count_validation_state: ValidationCompletionState,
    /// Non-required reason for record count validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count_non_required_reason: Option<String>,
    /// State of old-to-new ID mapping validation (prerequisite for linked validation).
    pub id_mapping_validation_state: ValidationCompletionState,
    /// State of linked record second-pass validation.
    pub linked_record_validation_state: ValidationCompletionState,
    /// State of attachment metadata validation.
    pub attachment_metadata_validation_state: ValidationCompletionState,
    /// Whether attachment validation is limited to metadata only (no file download).
    /// True triggers a Warning check — not a blocking failure.
    pub attachment_validation_metadata_only: bool,
    /// Non-required reason for attachment validation (required when NotRequired).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_non_required_reason: Option<String>,
    /// State of manifest/checksum validation. Required when a package manifest exists.
    pub manifest_checksum_validation_state: ValidationCompletionState,
    /// Whether a package manifest is present (determines if manifest validation is required).
    pub package_manifest_present: bool,
    /// Non-required reason for manifest validation (required when NotRequired + manifest present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_non_required_reason: Option<String>,
    /// Completion guard that prevents any result from being labeled complete
    /// without final validation having passed.
    pub completion_guard: Option<RestoreCompletionGuard>,
}

// ── Request / Result structs ──────────────────────────────────────────────────

/// Input to the final validation enforcement policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationEnforcementPolicyRequest {
    /// Declared enforcement plan for the restore write operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<FinalValidationEnforcementPlan>,
    /// Safe display label for the restore target (base name only, no path, no token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationEnforcementCheck {
    pub check_id: String,
    pub label: String,
    pub status: FinalValidationEnforcementCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Safe read-only summary of the evaluated plan, suitable for display.
/// Contains no sensitive values — no token, path, or record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationEnforcementSummary {
    pub schema_validation_state: String,
    pub record_count_validation_state: String,
    pub id_mapping_validation_state: String,
    pub linked_record_validation_state: String,
    pub attachment_metadata_validation_state: String,
    pub attachment_validation_metadata_only: bool,
    pub manifest_checksum_validation_state: String,
    pub package_manifest_present: bool,
    pub completion_guard_declared: bool,
    pub blocks_completion_without_final_validation: bool,
    pub failed_validation_blocks_completion: bool,
}

/// Result from `verify_final_validation_enforcement_policy`.
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
/// - No result may be labeled complete/successful before final validation passes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationEnforcementPolicyResult {
    pub status: FinalValidationEnforcementPolicyStatus,
    pub checks: Vec<FinalValidationEnforcementCheck>,
    pub message: String,
    /// Safe summary of the evaluated plan (no sensitive values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforcement_summary: Option<FinalValidationEnforcementSummary>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn state_label(state: &ValidationCompletionState) -> &'static str {
    match state {
        ValidationCompletionState::Passed => "passed",
        ValidationCompletionState::Failed => "failed",
        ValidationCompletionState::NotRequired => "notRequired",
        ValidationCompletionState::Skipped => "skipped",
        ValidationCompletionState::Partial => "partial",
        ValidationCompletionState::NotDeclared => "notDeclared",
    }
}

fn build_result(
    checks: Vec<FinalValidationEnforcementCheck>,
    enforcement_summary: Option<FinalValidationEnforcementSummary>,
    status: FinalValidationEnforcementPolicyStatus,
    target_label: Option<&str>,
) -> FinalValidationEnforcementPolicyResult {
    let target_note = target_label
        .map(|l| format!(" for '{l}'"))
        .unwrap_or_default();
    let message = match status {
        FinalValidationEnforcementPolicyStatus::Compliant => format!(
            "Final validation enforcement policy is compliant{target_note}. All required \
             validation states are explicitly passed or non-required with safe reasons. \
             The completion guard prevents any result from being labeled complete or \
             successful without final validation passing. Restore writes remain disabled — \
             compliance does not start any write operation and does not introduce a restore \
             success state."
        ),
        FinalValidationEnforcementPolicyStatus::Warning => format!(
            "Final validation enforcement policy has warnings{target_note}. Validation \
             enforcement is in place but some validation is metadata-only or marked \
             non-required with a limited reason. Restore writes remain disabled."
        ),
        FinalValidationEnforcementPolicyStatus::Blocked => format!(
            "Final validation enforcement policy is blocked{target_note}. One or more \
             required validation states are missing, failed, partial, or unsafely skipped. \
             No result may be labeled complete or successful. Resolve all violations before \
             any live write is considered. Restore writes remain disabled."
        ),
    };
    FinalValidationEnforcementPolicyResult {
        status,
        checks,
        message,
        enforcement_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the final validation enforcement policy for a planned restore write operation.
///
/// Check IDs:
/// - FVE-01: Write gate is disabled.
/// - FVE-02: Final validation enforcement plan is declared.
/// - FVE-03: Completion guard is declared.
/// - FVE-04: Schema validation must pass (or be NotRequired with reason) before completion.
/// - FVE-05: Record count validation must pass (or be NotRequired with reason) before completion.
/// - FVE-06: ID mapping validation must pass before linked validation completion.
/// - FVE-07: Linked record validation must pass (or be NotRequired with reason) before completion.
/// - FVE-08: Attachment metadata validation state must be explicit.
/// - FVE-09: Manifest/checksum validation state must be explicit if package manifest present.
/// - FVE-10: Partial validation state cannot be treated as completion.
/// - FVE-11: Failed validation blocks completion.
/// - FVE-12: Skipped validation blocks completion unless explicitly non-required with reason.
/// - FVE-13: No restore success state introduced.
/// - FVE-14: No token/path/payload exposure (safety invariant — always passes).
/// - FVE-15: Writes remain disabled even when compliant (safety invariant — always passes).
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
/// `Compliant` does not introduce a restore success state.
pub fn verify_final_validation_enforcement_policy(
    request: &FinalValidationEnforcementPolicyRequest,
) -> FinalValidationEnforcementPolicyResult {
    let mut checks: Vec<FinalValidationEnforcementCheck> = Vec::new();

    // FVE-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: FinalValidationEnforcementCheckStatus::Passed,
            message: "Write gate is disabled. No restore writes are attempted.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: FinalValidationEnforcementCheckStatus::Failed,
            message: "Write gate is not disabled. Final validation enforcement policy cannot be \
                      evaluated while write gate is active."
                .to_string(),
            remediation: Some(
                "Ensure evaluate_write_gate() returns Disabled before running policy checks."
                    .to_string(),
            ),
        });
        return build_result(
            checks,
            None,
            FinalValidationEnforcementPolicyStatus::Blocked,
            None,
        );
    }

    // FVE-02: Plan declared — short-circuit if absent
    let plan = match &request.plan {
        Some(p) => p,
        None => {
            checks.push(FinalValidationEnforcementCheck {
                check_id: "FVE-02".to_string(),
                label: "plan-declared".to_string(),
                status: FinalValidationEnforcementCheckStatus::Failed,
                message: "No final validation enforcement plan declared. A plan declaring all \
                          required validation states and a completion guard is required before \
                          any live write path is considered."
                    .to_string(),
                remediation: Some(
                    "Declare a FinalValidationEnforcementPlan with all validation states and a \
                     RestoreCompletionGuard."
                        .to_string(),
                ),
            });
            return build_result(
                checks,
                None,
                FinalValidationEnforcementPolicyStatus::Blocked,
                request.target_label.as_deref(),
            );
        }
    };

    checks.push(FinalValidationEnforcementCheck {
        check_id: "FVE-02".to_string(),
        label: "plan-declared".to_string(),
        status: FinalValidationEnforcementCheckStatus::Passed,
        message: "Final validation enforcement plan is declared.".to_string(),
        remediation: None,
    });

    let mut blocked = false;
    let mut has_warning = false;

    // FVE-03: Completion guard declared
    let guard = match &plan.completion_guard {
        Some(g) => g,
        None => {
            checks.push(FinalValidationEnforcementCheck {
                check_id: "FVE-03".to_string(),
                label: "completion-guard-declared".to_string(),
                status: FinalValidationEnforcementCheckStatus::Failed,
                message: "No completion guard declared. A RestoreCompletionGuard is required to \
                          prevent any result from being labeled complete or successful without \
                          final validation passing."
                    .to_string(),
                remediation: Some(
                    "Declare a RestoreCompletionGuard with blocksCompletionWithoutFinalValidation, \
                     blocksPartialValidationAsCompletion, and failedValidationBlocksCompletion all \
                     set to true."
                        .to_string(),
                ),
            });
            blocked = true;
            // Continue evaluating remaining checks even without guard
            &RestoreCompletionGuard {
                blocks_completion_without_final_validation: false,
                blocks_partial_validation_as_completion: false,
                failed_validation_blocks_completion: false,
                note: None,
            }
        }
    };

    if plan.completion_guard.is_some() {
        // Validate the guard fields
        let guard_ok = guard.blocks_completion_without_final_validation
            && guard.blocks_partial_validation_as_completion
            && guard.failed_validation_blocks_completion;
        if guard_ok {
            checks.push(FinalValidationEnforcementCheck {
                check_id: "FVE-03".to_string(),
                label: "completion-guard-declared".to_string(),
                status: FinalValidationEnforcementCheckStatus::Passed,
                message: "Completion guard is declared with all three blocking conditions set."
                    .to_string(),
                remediation: None,
            });
        } else {
            checks.push(FinalValidationEnforcementCheck {
                check_id: "FVE-03".to_string(),
                label: "completion-guard-declared".to_string(),
                status: FinalValidationEnforcementCheckStatus::Failed,
                message: "Completion guard is declared but one or more blocking conditions are \
                          not set. All three conditions must be true: \
                          blocksCompletionWithoutFinalValidation, \
                          blocksPartialValidationAsCompletion, failedValidationBlocksCompletion."
                    .to_string(),
                remediation: Some(
                    "Set all three guard fields to true in the RestoreCompletionGuard.".to_string(),
                ),
            });
            blocked = true;
        }
    }

    // Helper: check a required validation state
    let check_validation_state = |check_id: &str,
                                  label: &str,
                                  state: &ValidationCompletionState,
                                  non_required_reason: Option<&str>,
                                  field_name: &str|
     -> (FinalValidationEnforcementCheck, bool, bool) {
        let is_blocked;
        let is_warning;
        let (status, message, remediation) = match state {
            ValidationCompletionState::Passed => {
                is_blocked = false;
                is_warning = false;
                (
                    FinalValidationEnforcementCheckStatus::Passed,
                    format!("{field_name} validation has passed."),
                    None,
                )
            }
            ValidationCompletionState::Failed => {
                is_blocked = true;
                is_warning = false;
                (
                    FinalValidationEnforcementCheckStatus::Failed,
                    format!(
                        "{field_name} validation has failed. A failed validation \
                             unconditionally blocks completion."
                    ),
                    Some(format!(
                        "Resolve the {field_name} validation failure before any result can \
                             be labeled complete."
                    )),
                )
            }
            ValidationCompletionState::NotRequired => {
                let reason = non_required_reason.unwrap_or("").trim();
                if reason.is_empty() {
                    is_blocked = true;
                    is_warning = false;
                    (
                        FinalValidationEnforcementCheckStatus::Failed,
                        format!(
                            "{field_name} validation is marked NotRequired but no reason is \
                                 documented. A non-empty reason is required."
                        ),
                        Some(format!(
                            "Provide a non-empty reason in the corresponding non_required_reason \
                                 field for {field_name} validation."
                        )),
                    )
                } else {
                    is_blocked = false;
                    is_warning = true;
                    (
                        FinalValidationEnforcementCheckStatus::Warning,
                        format!(
                            "{field_name} validation is marked NotRequired with reason: \
                                 \"{reason}\"."
                        ),
                        None,
                    )
                }
            }
            ValidationCompletionState::Skipped => {
                is_blocked = true;
                is_warning = false;
                (
                    FinalValidationEnforcementCheckStatus::Failed,
                    format!(
                        "{field_name} validation is marked Skipped. Skipped validation \
                             blocks completion unless explicitly marked NotRequired with a \
                             documented reason."
                    ),
                    Some(format!(
                        "Change {field_name} validation state to Passed, or use NotRequired \
                             with a documented reason if this validation does not apply."
                    )),
                )
            }
            ValidationCompletionState::Partial => {
                is_blocked = true;
                is_warning = false;
                (
                    FinalValidationEnforcementCheckStatus::Failed,
                    format!(
                        "{field_name} validation is only partially complete. Partial \
                             validation cannot be treated as a completion state."
                    ),
                    Some(format!(
                        "Ensure {field_name} validation runs to completion before the result \
                             is evaluated."
                    )),
                )
            }
            ValidationCompletionState::NotDeclared => {
                is_blocked = true;
                is_warning = false;
                (
                    FinalValidationEnforcementCheckStatus::Failed,
                    format!(
                        "{field_name} validation state has not been declared. An explicit \
                             state is required."
                    ),
                    Some(format!(
                        "Declare the {field_name} validation state in the enforcement plan."
                    )),
                )
            }
        };
        (
            FinalValidationEnforcementCheck {
                check_id: check_id.to_string(),
                label: label.to_string(),
                status,
                message,
                remediation,
            },
            is_blocked,
            is_warning,
        )
    };

    // FVE-04: Schema validation
    let (check, b, w) = check_validation_state(
        "FVE-04",
        "schema-validation",
        &plan.schema_validation_state,
        plan.schema_validation_non_required_reason.as_deref(),
        "Schema",
    );
    checks.push(check);
    blocked |= b;
    has_warning |= w;

    // FVE-05: Record count validation
    let (check, b, w) = check_validation_state(
        "FVE-05",
        "record-count-validation",
        &plan.record_count_validation_state,
        plan.record_count_non_required_reason.as_deref(),
        "Record count",
    );
    checks.push(check);
    blocked |= b;
    has_warning |= w;

    // FVE-06: ID mapping validation — prerequisite for linked validation
    // ID mapping is special: if linked record validation is Passed but ID mapping
    // is not Passed, that is a blocking inconsistency.
    let id_map_passed = plan.id_mapping_validation_state == ValidationCompletionState::Passed;
    let linked_needs_id_map = !matches!(
        plan.linked_record_validation_state,
        ValidationCompletionState::NotRequired | ValidationCompletionState::NotDeclared
    );

    if plan.id_mapping_validation_state.is_blocking() && linked_needs_id_map {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-06".to_string(),
            label: "id-mapping-before-linked-validation".to_string(),
            status: FinalValidationEnforcementCheckStatus::Failed,
            message: "ID mapping validation has not passed but linked record validation requires \
                      it. ID mapping must pass before linked record validation can complete."
                .to_string(),
            remediation: Some(
                "Ensure id_mapping_validation_state is Passed before linked record validation \
                 is evaluated."
                    .to_string(),
            ),
        });
        blocked = true;
    } else if !id_map_passed
        && plan.id_mapping_validation_state == ValidationCompletionState::NotRequired
    {
        let reason = plan
            .schema_validation_non_required_reason
            .as_deref()
            .unwrap_or("")
            .trim();
        if reason.is_empty() {
            checks.push(FinalValidationEnforcementCheck {
                check_id: "FVE-06".to_string(),
                label: "id-mapping-before-linked-validation".to_string(),
                status: FinalValidationEnforcementCheckStatus::Warning,
                message: "ID mapping validation is marked NotRequired. Linked record validation \
                          may be incomplete without ID mapping."
                    .to_string(),
                remediation: None,
            });
            has_warning = true;
        } else {
            checks.push(FinalValidationEnforcementCheck {
                check_id: "FVE-06".to_string(),
                label: "id-mapping-before-linked-validation".to_string(),
                status: FinalValidationEnforcementCheckStatus::Passed,
                message: "ID mapping validation is NotRequired (linked validation does not apply)."
                    .to_string(),
                remediation: None,
            });
        }
    } else {
        let (check, b, w) = check_validation_state(
            "FVE-06",
            "id-mapping-before-linked-validation",
            &plan.id_mapping_validation_state,
            None,
            "ID mapping",
        );
        checks.push(check);
        blocked |= b;
        has_warning |= w;
    }

    // FVE-07: Linked record validation
    let (check, b, w) = check_validation_state(
        "FVE-07",
        "linked-record-validation",
        &plan.linked_record_validation_state,
        None,
        "Linked record",
    );
    checks.push(check);
    blocked |= b;
    has_warning |= w;

    // FVE-08: Attachment metadata validation state must be explicit
    if plan.attachment_metadata_validation_state == ValidationCompletionState::NotDeclared {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-08".to_string(),
            label: "attachment-validation-explicit".to_string(),
            status: FinalValidationEnforcementCheckStatus::Failed,
            message: "Attachment metadata validation state has not been declared. An explicit \
                      state (Passed, NotRequired, or metadata-only) is required."
                .to_string(),
            remediation: Some(
                "Declare the attachment_metadata_validation_state in the enforcement plan."
                    .to_string(),
            ),
        });
        blocked = true;
    } else if plan.attachment_validation_metadata_only {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-08".to_string(),
            label: "attachment-validation-explicit".to_string(),
            status: FinalValidationEnforcementCheckStatus::Warning,
            message: "Attachment validation is limited to metadata only — file content is not \
                      verified. This is acceptable for this version but should be documented."
                .to_string(),
            remediation: Some(
                "Consider adding full attachment validation in a future gate when attachment \
                 download is supported."
                    .to_string(),
            ),
        });
        has_warning = true;
    } else {
        let (check, b, w) = check_validation_state(
            "FVE-08",
            "attachment-validation-explicit",
            &plan.attachment_metadata_validation_state,
            plan.attachment_non_required_reason.as_deref(),
            "Attachment metadata",
        );
        checks.push(check);
        blocked |= b;
        has_warning |= w;
    }

    // FVE-09: Manifest/checksum validation must be explicit if manifest present
    if plan.package_manifest_present {
        if plan.manifest_checksum_validation_state == ValidationCompletionState::NotDeclared {
            checks.push(FinalValidationEnforcementCheck {
                check_id: "FVE-09".to_string(),
                label: "manifest-checksum-validation".to_string(),
                status: FinalValidationEnforcementCheckStatus::Failed,
                message: "A package manifest is present but manifest/checksum validation state \
                          has not been declared. An explicit state is required when a manifest \
                          exists."
                    .to_string(),
                remediation: Some(
                    "Declare manifest_checksum_validation_state in the enforcement plan."
                        .to_string(),
                ),
            });
            blocked = true;
        } else {
            let (check, b, w) = check_validation_state(
                "FVE-09",
                "manifest-checksum-validation",
                &plan.manifest_checksum_validation_state,
                plan.manifest_non_required_reason.as_deref(),
                "Manifest/checksum",
            );
            checks.push(check);
            blocked |= b;
            has_warning |= w;
        }
    } else {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-09".to_string(),
            label: "manifest-checksum-validation".to_string(),
            status: FinalValidationEnforcementCheckStatus::Passed,
            message: "No package manifest present — manifest/checksum validation is not \
                      required."
                .to_string(),
            remediation: None,
        });
    }

    // FVE-10: Partial validation cannot be completion
    let any_partial = [
        &plan.schema_validation_state,
        &plan.record_count_validation_state,
        &plan.id_mapping_validation_state,
        &plan.linked_record_validation_state,
        &plan.attachment_metadata_validation_state,
        &plan.manifest_checksum_validation_state,
    ]
    .iter()
    .any(|s| **s == ValidationCompletionState::Partial);

    if any_partial {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-10".to_string(),
            label: "no-partial-as-completion".to_string(),
            status: FinalValidationEnforcementCheckStatus::Failed,
            message: "One or more validation states are Partial. Partial validation cannot be \
                      treated as a completion state."
                .to_string(),
            remediation: Some(
                "Ensure all validation runs to completion before evaluating the result."
                    .to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-10".to_string(),
            label: "no-partial-as-completion".to_string(),
            status: FinalValidationEnforcementCheckStatus::Passed,
            message: "No validation state is Partial.".to_string(),
            remediation: None,
        });
    }

    // FVE-11: Failed validation blocks completion
    let any_failed = [
        &plan.schema_validation_state,
        &plan.record_count_validation_state,
        &plan.id_mapping_validation_state,
        &plan.linked_record_validation_state,
        &plan.attachment_metadata_validation_state,
        &plan.manifest_checksum_validation_state,
    ]
    .iter()
    .any(|s| **s == ValidationCompletionState::Failed);

    if any_failed {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-11".to_string(),
            label: "failed-validation-blocks-completion".to_string(),
            status: FinalValidationEnforcementCheckStatus::Failed,
            message: "One or more validation states are Failed. Failed validation \
                      unconditionally blocks completion."
                .to_string(),
            remediation: Some(
                "Resolve all validation failures before any result can be labeled complete."
                    .to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-11".to_string(),
            label: "failed-validation-blocks-completion".to_string(),
            status: FinalValidationEnforcementCheckStatus::Passed,
            message: "No validation state is Failed.".to_string(),
            remediation: None,
        });
    }

    // FVE-12: Skipped validation blocks completion unless NotRequired with reason
    let any_unsafely_skipped = [
        &plan.schema_validation_state,
        &plan.record_count_validation_state,
        &plan.id_mapping_validation_state,
        &plan.linked_record_validation_state,
        &plan.attachment_metadata_validation_state,
        &plan.manifest_checksum_validation_state,
    ]
    .iter()
    .any(|s| **s == ValidationCompletionState::Skipped);

    if any_unsafely_skipped {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-12".to_string(),
            label: "no-unsafe-skip".to_string(),
            status: FinalValidationEnforcementCheckStatus::Failed,
            message: "One or more validation states are Skipped. Skipped validation blocks \
                      completion. Use NotRequired with a documented reason instead of Skipped."
                .to_string(),
            remediation: Some(
                "Replace Skipped states with NotRequired (and provide a reason) or run the \
                 validation to completion."
                    .to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(FinalValidationEnforcementCheck {
            check_id: "FVE-12".to_string(),
            label: "no-unsafe-skip".to_string(),
            status: FinalValidationEnforcementCheckStatus::Passed,
            message: "No validation state is unsafely Skipped.".to_string(),
            remediation: None,
        });
    }

    // FVE-13: No restore success state introduced (safety invariant — always passes)
    checks.push(FinalValidationEnforcementCheck {
        check_id: "FVE-13".to_string(),
        label: "no-restore-success-state".to_string(),
        status: FinalValidationEnforcementCheckStatus::Passed,
        message: "No restore success state is introduced. Compliant does not label any result \
                  as a restore completion or success."
            .to_string(),
        remediation: None,
    });

    // FVE-14: No token/path/payload exposure (safety invariant — always passes)
    checks.push(FinalValidationEnforcementCheck {
        check_id: "FVE-14".to_string(),
        label: "no-token-path-payload".to_string(),
        status: FinalValidationEnforcementCheckStatus::Passed,
        message: "No token, filesystem path, or record payload is present in any result field."
            .to_string(),
        remediation: None,
    });

    // FVE-15: Writes remain disabled (safety invariant — always passes)
    checks.push(FinalValidationEnforcementCheck {
        check_id: "FVE-15".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: FinalValidationEnforcementCheckStatus::Passed,
        message: "Restore writes remain disabled. Policy compliance does not enable write \
                  execution."
            .to_string(),
        remediation: None,
    });

    // Build enforcement summary (safe — no sensitive values)
    let enforcement_summary = FinalValidationEnforcementSummary {
        schema_validation_state: state_label(&plan.schema_validation_state).to_string(),
        record_count_validation_state: state_label(&plan.record_count_validation_state).to_string(),
        id_mapping_validation_state: state_label(&plan.id_mapping_validation_state).to_string(),
        linked_record_validation_state: state_label(&plan.linked_record_validation_state)
            .to_string(),
        attachment_metadata_validation_state: state_label(
            &plan.attachment_metadata_validation_state,
        )
        .to_string(),
        attachment_validation_metadata_only: plan.attachment_validation_metadata_only,
        manifest_checksum_validation_state: state_label(&plan.manifest_checksum_validation_state)
            .to_string(),
        package_manifest_present: plan.package_manifest_present,
        completion_guard_declared: plan.completion_guard.is_some(),
        blocks_completion_without_final_validation: plan
            .completion_guard
            .as_ref()
            .map(|g| g.blocks_completion_without_final_validation)
            .unwrap_or(false),
        failed_validation_blocks_completion: plan
            .completion_guard
            .as_ref()
            .map(|g| g.failed_validation_blocks_completion)
            .unwrap_or(false),
    };

    let status = if blocked {
        FinalValidationEnforcementPolicyStatus::Blocked
    } else if has_warning {
        FinalValidationEnforcementPolicyStatus::Warning
    } else {
        FinalValidationEnforcementPolicyStatus::Compliant
    };

    build_result(
        checks,
        Some(enforcement_summary),
        status,
        request.target_label.as_deref(),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_guard() -> RestoreCompletionGuard {
        RestoreCompletionGuard {
            blocks_completion_without_final_validation: true,
            blocks_partial_validation_as_completion: true,
            failed_validation_blocks_completion: true,
            note: None,
        }
    }

    fn safe_plan() -> FinalValidationEnforcementPlan {
        FinalValidationEnforcementPlan {
            schema_validation_state: ValidationCompletionState::Passed,
            schema_validation_non_required_reason: None,
            record_count_validation_state: ValidationCompletionState::Passed,
            record_count_non_required_reason: None,
            id_mapping_validation_state: ValidationCompletionState::Passed,
            linked_record_validation_state: ValidationCompletionState::Passed,
            attachment_metadata_validation_state: ValidationCompletionState::Passed,
            attachment_validation_metadata_only: false,
            attachment_non_required_reason: None,
            manifest_checksum_validation_state: ValidationCompletionState::Passed,
            package_manifest_present: true,
            manifest_non_required_reason: None,
            completion_guard: Some(safe_guard()),
        }
    }

    fn request_with_plan(
        plan: FinalValidationEnforcementPlan,
    ) -> FinalValidationEnforcementPolicyRequest {
        FinalValidationEnforcementPolicyRequest {
            plan: Some(plan),
            target_label: None,
        }
    }

    fn request_no_plan() -> FinalValidationEnforcementPolicyRequest {
        FinalValidationEnforcementPolicyRequest {
            plan: None,
            target_label: None,
        }
    }

    #[test]
    fn complete_safe_plan_is_compliant() {
        let result = verify_final_validation_enforcement_policy(&request_with_plan(safe_plan()));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Compliant
        );
    }

    #[test]
    fn missing_plan_is_blocked() {
        let result = verify_final_validation_enforcement_policy(&request_no_plan());
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        assert_eq!(result.checks.len(), 2);
        let fve02 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-02")
            .unwrap();
        assert_eq!(fve02.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn missing_completion_guard_is_blocked() {
        let mut plan = safe_plan();
        plan.completion_guard = None;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-03")
            .unwrap();
        assert_eq!(fve03.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn incomplete_guard_is_blocked() {
        let mut plan = safe_plan();
        plan.completion_guard = Some(RestoreCompletionGuard {
            blocks_completion_without_final_validation: true,
            blocks_partial_validation_as_completion: false,
            failed_validation_blocks_completion: true,
            note: None,
        });
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-03")
            .unwrap();
        assert_eq!(fve03.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn failed_schema_validation_is_blocked() {
        let mut plan = safe_plan();
        plan.schema_validation_state = ValidationCompletionState::Failed;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-04")
            .unwrap();
        assert_eq!(fve04.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn failed_record_count_validation_is_blocked() {
        let mut plan = safe_plan();
        plan.record_count_validation_state = ValidationCompletionState::Failed;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-05")
            .unwrap();
        assert_eq!(fve05.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn id_mapping_not_passed_with_linked_validation_needed_is_blocked() {
        let mut plan = safe_plan();
        plan.id_mapping_validation_state = ValidationCompletionState::Failed;
        plan.linked_record_validation_state = ValidationCompletionState::Passed;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-06")
            .unwrap();
        assert_eq!(fve06.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn failed_linked_validation_is_blocked() {
        let mut plan = safe_plan();
        plan.linked_record_validation_state = ValidationCompletionState::Failed;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-07")
            .unwrap();
        assert_eq!(fve07.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn attachment_not_declared_is_blocked() {
        let mut plan = safe_plan();
        plan.attachment_metadata_validation_state = ValidationCompletionState::NotDeclared;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-08")
            .unwrap();
        assert_eq!(fve08.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn attachment_metadata_only_is_warning() {
        let mut plan = safe_plan();
        plan.attachment_validation_metadata_only = true;
        plan.attachment_metadata_validation_state = ValidationCompletionState::Passed;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Warning
        );
        let fve08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-08")
            .unwrap();
        assert_eq!(fve08.status, FinalValidationEnforcementCheckStatus::Warning);
    }

    #[test]
    fn manifest_validation_required_when_manifest_present_and_not_declared() {
        let mut plan = safe_plan();
        plan.package_manifest_present = true;
        plan.manifest_checksum_validation_state = ValidationCompletionState::NotDeclared;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-09")
            .unwrap();
        assert_eq!(fve09.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn no_manifest_skips_manifest_check() {
        let mut plan = safe_plan();
        plan.package_manifest_present = false;
        plan.manifest_checksum_validation_state = ValidationCompletionState::NotDeclared;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Compliant
        );
        let fve09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-09")
            .unwrap();
        assert_eq!(fve09.status, FinalValidationEnforcementCheckStatus::Passed);
    }

    #[test]
    fn partial_validation_is_blocked() {
        let mut plan = safe_plan();
        plan.record_count_validation_state = ValidationCompletionState::Partial;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve10 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-10")
            .unwrap();
        assert_eq!(fve10.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn skipped_validation_is_blocked() {
        let mut plan = safe_plan();
        plan.schema_validation_state = ValidationCompletionState::Skipped;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
        let fve12 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-12")
            .unwrap();
        assert_eq!(fve12.status, FinalValidationEnforcementCheckStatus::Failed);
    }

    #[test]
    fn not_required_with_reason_is_warning() {
        let mut plan = safe_plan();
        plan.schema_validation_state = ValidationCompletionState::NotRequired;
        plan.schema_validation_non_required_reason =
            Some("Schema-only restore, no record data".to_string());
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Warning
        );
        let fve04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-04")
            .unwrap();
        assert_eq!(fve04.status, FinalValidationEnforcementCheckStatus::Warning);
    }

    #[test]
    fn not_required_without_reason_is_blocked() {
        let mut plan = safe_plan();
        plan.schema_validation_state = ValidationCompletionState::NotRequired;
        plan.schema_validation_non_required_reason = None;
        let result = verify_final_validation_enforcement_policy(&request_with_plan(plan));
        assert_eq!(
            result.status,
            FinalValidationEnforcementPolicyStatus::Blocked
        );
    }

    #[test]
    fn complete_plan_has_fifteen_checks() {
        let result = verify_final_validation_enforcement_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.checks.len(), 15);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let results = vec![
            verify_final_validation_enforcement_policy(&request_no_plan()),
            verify_final_validation_enforcement_policy(&request_with_plan(safe_plan())),
            verify_final_validation_enforcement_policy(&request_with_plan({
                let mut p = safe_plan();
                p.schema_validation_state = ValidationCompletionState::Failed;
                p
            })),
        ];
        for r in results {
            assert!(r.no_changes_made);
        }
    }

    #[test]
    fn writes_enabled_is_always_false() {
        let results = vec![
            verify_final_validation_enforcement_policy(&request_no_plan()),
            verify_final_validation_enforcement_policy(&request_with_plan(safe_plan())),
        ];
        for r in results {
            assert!(!r.writes_enabled);
        }
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let result = verify_final_validation_enforcement_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn fve13_fve14_fve15_always_pass() {
        let result = verify_final_validation_enforcement_policy(&request_with_plan(safe_plan()));
        let fve13 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-13")
            .unwrap();
        let fve14 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-14")
            .unwrap();
        let fve15 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVE-15")
            .unwrap();
        assert_eq!(fve13.status, FinalValidationEnforcementCheckStatus::Passed);
        assert_eq!(fve14.status, FinalValidationEnforcementCheckStatus::Passed);
        assert_eq!(fve15.status, FinalValidationEnforcementCheckStatus::Passed);
    }

    #[test]
    fn enforcement_summary_present_for_complete_plan() {
        let result = verify_final_validation_enforcement_policy(&request_with_plan(safe_plan()));
        assert!(result.enforcement_summary.is_some());
        let summary = result.enforcement_summary.unwrap();
        assert_eq!(summary.schema_validation_state, "passed");
        assert!(summary.completion_guard_declared);
        assert!(summary.blocks_completion_without_final_validation);
        assert!(summary.failed_validation_blocks_completion);
    }

    #[test]
    fn enforcement_summary_absent_when_no_plan() {
        let result = verify_final_validation_enforcement_policy(&request_no_plan());
        assert!(result.enforcement_summary.is_none());
    }

    #[test]
    fn no_token_or_path_in_serialized_result() {
        let result = verify_final_validation_enforcement_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("apiKey"));
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("\"token\":"));
    }

    #[test]
    fn no_success_state_in_result_message() {
        let result = verify_final_validation_enforcement_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.to_lowercase().contains("succeeded"));
        assert!(!result.message.to_lowercase().contains("restore complete"));
    }

    #[test]
    fn target_label_appears_in_compliant_message() {
        let request = FinalValidationEnforcementPolicyRequest {
            plan: Some(safe_plan()),
            target_label: Some("test-base".to_string()),
        };
        let result = verify_final_validation_enforcement_policy(&request);
        assert!(result.message.contains("test-base"));
    }
}
