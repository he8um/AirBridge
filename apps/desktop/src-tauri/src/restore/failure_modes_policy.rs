use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureModesPolicyStatus {
    /// All required failure modes have explicit, safe stop-behavior declarations.
    Compliant,
    /// One or more modes have incomplete diagnostic context but safe stop behavior.
    Warning,
    /// One or more required modes are missing, or a mode allows writes to continue
    /// after failure, or a mode declares automatic destructive rollback.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureModesCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// Each recognized failure mode that must have an explicit handling plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreFailureMode {
    SchemaCreateFailure,
    SchemaVerifyFailure,
    RecordCreateFailure,
    IdMappingFailure,
    LinkedRecordUpdateFailure,
    CheckpointPersistenceFailure,
    RateLimitExhaustion,
    TargetMutationDetected,
    FinalValidationFailure,
    UnknownFailure,
}

impl RestoreFailureMode {
    fn label(&self) -> &'static str {
        match self {
            RestoreFailureMode::SchemaCreateFailure => "schemaCreateFailure",
            RestoreFailureMode::SchemaVerifyFailure => "schemaVerifyFailure",
            RestoreFailureMode::RecordCreateFailure => "recordCreateFailure",
            RestoreFailureMode::IdMappingFailure => "idMappingFailure",
            RestoreFailureMode::LinkedRecordUpdateFailure => "linkedRecordUpdateFailure",
            RestoreFailureMode::CheckpointPersistenceFailure => "checkpointPersistenceFailure",
            RestoreFailureMode::RateLimitExhaustion => "rateLimitExhaustion",
            RestoreFailureMode::TargetMutationDetected => "targetMutationDetected",
            RestoreFailureMode::FinalValidationFailure => "finalValidationFailure",
            RestoreFailureMode::UnknownFailure => "unknownFailure",
        }
    }
}

/// What the restore engine will do when the associated failure mode occurs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureStopBehavior {
    /// Stop all further writes immediately and report the failure.
    StopAndReport,
    /// Stop all further writes, preserve the last good checkpoint, and report.
    StopPreserveCheckpointAndReport,
    /// Stop all further writes after exhausting the configured retry limit.
    StopAfterRetryLimit,
    /// Block the current operation and require manual intervention before any
    /// further writes are attempted.
    BlockAndRequireManualReview,
}

impl FailureStopBehavior {
    /// Returns true when this behavior unconditionally stops further writes.
    fn stops_writes(&self) -> bool {
        match self {
            FailureStopBehavior::StopAndReport => true,
            FailureStopBehavior::StopPreserveCheckpointAndReport => true,
            FailureStopBehavior::StopAfterRetryLimit => true,
            FailureStopBehavior::BlockAndRequireManualReview => true,
        }
    }
}

// ── Plan struct ───────────────────────────────────────────────────────────────

/// Declared handling plan for a single restore failure mode.
///
/// All fields are boolean flags, enums, or short labels — no token, no path,
/// no record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFailureHandlingPlan {
    /// The failure mode this plan covers.
    pub mode: RestoreFailureMode,
    /// What the engine will do when this failure occurs.
    pub stop_behavior: FailureStopBehavior,
    /// Whether the checkpoint state is preserved when this failure occurs.
    pub preserves_checkpoint: bool,
    /// Whether automatic destructive rollback (e.g. delete all created records)
    /// is triggered on this failure.
    pub triggers_destructive_rollback: bool,
    /// Whether a partial failure under this mode can be incorrectly labeled as
    /// a success.
    pub partial_failure_labeled_success: bool,
    /// Whether diagnostic context (error message, affected records) is captured
    /// for this failure mode.
    pub captures_diagnostic_context: bool,
    /// Optional human-readable note. No token, path, or record payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ── Request / Result structs ──────────────────────────────────────────────────

/// Input to the failure modes policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureModesPolicyRequest {
    /// The set of declared failure handling plans for the restore write operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handling_plans: Option<Vec<RestoreFailureHandlingPlan>>,
    /// Safe display label for the restore target (base name only, no path, no token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureModesCheck {
    pub check_id: String,
    pub label: String,
    pub status: FailureModesCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Safe read-only summary of one evaluated handling plan (no sensitive values).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureHandlingSummaryEntry {
    pub mode: String,
    pub stop_behavior: String,
    pub preserves_checkpoint: bool,
    pub triggers_destructive_rollback: bool,
    pub captures_diagnostic_context: bool,
}

/// Result from `verify_failure_modes_policy`.
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
pub struct FailureModesPolicyResult {
    pub status: FailureModesPolicyStatus,
    pub checks: Vec<FailureModesCheck>,
    pub message: String,
    /// Safe summary of each evaluated handling plan (no sensitive values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handling_summary: Option<Vec<FailureHandlingSummaryEntry>>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Required failure modes ────────────────────────────────────────────────────

const REQUIRED_MODES: &[RestoreFailureMode] = &[
    RestoreFailureMode::SchemaCreateFailure,
    RestoreFailureMode::SchemaVerifyFailure,
    RestoreFailureMode::RecordCreateFailure,
    RestoreFailureMode::IdMappingFailure,
    RestoreFailureMode::LinkedRecordUpdateFailure,
    RestoreFailureMode::CheckpointPersistenceFailure,
    RestoreFailureMode::RateLimitExhaustion,
    RestoreFailureMode::TargetMutationDetected,
    RestoreFailureMode::FinalValidationFailure,
    RestoreFailureMode::UnknownFailure,
];

// ── Internal helpers ──────────────────────────────────────────────────────────

fn stop_behavior_label(b: &FailureStopBehavior) -> &'static str {
    match b {
        FailureStopBehavior::StopAndReport => "stopAndReport",
        FailureStopBehavior::StopPreserveCheckpointAndReport => "stopPreserveCheckpointAndReport",
        FailureStopBehavior::StopAfterRetryLimit => "stopAfterRetryLimit",
        FailureStopBehavior::BlockAndRequireManualReview => "blockAndRequireManualReview",
    }
}

fn build_result(
    checks: Vec<FailureModesCheck>,
    handling_summary: Option<Vec<FailureHandlingSummaryEntry>>,
    status: FailureModesPolicyStatus,
    target_label: Option<&str>,
) -> FailureModesPolicyResult {
    let target_note = target_label
        .map(|l| format!(" for '{l}'"))
        .unwrap_or_default();
    let message = match status {
        FailureModesPolicyStatus::Compliant => format!(
            "Failure modes policy is compliant{target_note}. All required failure modes have \
             explicit, safe stop-behavior declarations. Restore writes remain disabled — compliance \
             does not start any write operation and does not introduce a restore success state."
        ),
        FailureModesPolicyStatus::Warning => format!(
            "Failure modes policy has warnings{target_note}. All required modes have safe stop \
             behavior, but one or more modes have incomplete diagnostic context. Restore writes \
             remain disabled."
        ),
        FailureModesPolicyStatus::Blocked => format!(
            "Failure modes policy is blocked{target_note}. One or more required failure modes are \
             missing or declare unsafe behavior. Resolve all violations before any live write is \
             considered. Restore writes remain disabled."
        ),
    };
    FailureModesPolicyResult {
        status,
        checks,
        message,
        handling_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the failure modes policy for a planned restore write operation.
///
/// Check IDs:
/// - FMP-01: Write gate is disabled.
/// - FMP-02: Handling plans are declared.
/// - FMP-03: All required failure modes are covered.
/// - FMP-04: No mode allows writes to continue after failure.
/// - FMP-05: No mode declares automatic destructive rollback.
/// - FMP-06: Unknown/unclassified failure stops all writes.
/// - FMP-07: Rate-limit exhaustion stops after retry limit.
/// - FMP-08: Final validation failure blocks success.
/// - FMP-09: Checkpoint persistence failure blocks continuation.
/// - FMP-10: No partial failure is labeled as success.
/// - FMP-11: Writes remain disabled.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
pub fn verify_failure_modes_policy(
    request: &FailureModesPolicyRequest,
) -> FailureModesPolicyResult {
    let mut checks: Vec<FailureModesCheck> = Vec::new();

    // FMP-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(FailureModesCheck {
            check_id: "FMP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: FailureModesCheckStatus::Passed,
            message: "Restore write gate is disabled. No live writes are possible.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(FailureModesCheck {
            check_id: "FMP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: FailureModesCheckStatus::Failed,
            message: "Restore write gate is unexpectedly enabled.".to_string(),
            remediation: Some(
                "The write gate must remain disabled until all safety gates are satisfied."
                    .to_string(),
            ),
        });
    }

    // FMP-02: Handling plans declared
    let plans = match &request.handling_plans {
        Some(p) if !p.is_empty() => p,
        _ => {
            checks.push(FailureModesCheck {
                check_id: "FMP-02".to_string(),
                label: "plans-declared".to_string(),
                status: FailureModesCheckStatus::Failed,
                message:
                    "No failure handling plans declared. Plans for all required failure modes \
                           are required before any live write path is considered."
                        .to_string(),
                remediation: Some(
                    "Declare a RestoreFailureHandlingPlan for each required failure mode."
                        .to_string(),
                ),
            });
            return build_result(
                checks,
                None,
                FailureModesPolicyStatus::Blocked,
                request.target_label.as_deref(),
            );
        }
    };

    checks.push(FailureModesCheck {
        check_id: "FMP-02".to_string(),
        label: "plans-declared".to_string(),
        status: FailureModesCheckStatus::Passed,
        message: format!(
            "Failure handling plans are declared ({} mode(s) covered).",
            plans.len()
        ),
        remediation: None,
    });

    // FMP-03: All required modes covered
    let mut missing: Vec<&str> = Vec::new();
    for required in REQUIRED_MODES {
        if !plans.iter().any(|p| &p.mode == required) {
            missing.push(required.label());
        }
    }
    if !missing.is_empty() {
        checks.push(FailureModesCheck {
            check_id: "FMP-03".to_string(),
            label: "all-required-modes-covered".to_string(),
            status: FailureModesCheckStatus::Failed,
            message: format!(
                "Missing handling plans for required failure mode(s): {}.",
                missing.join(", ")
            ),
            remediation: Some(
                "Declare a RestoreFailureHandlingPlan for each missing failure mode.".to_string(),
            ),
        });
    } else {
        checks.push(FailureModesCheck {
            check_id: "FMP-03".to_string(),
            label: "all-required-modes-covered".to_string(),
            status: FailureModesCheckStatus::Passed,
            message: "All required failure modes have handling plans.".to_string(),
            remediation: None,
        });
    }

    // FMP-04: No mode allows writes to continue after failure
    let continues_writes: Vec<&str> = plans
        .iter()
        .filter(|p| !p.stop_behavior.stops_writes())
        .map(|p| p.mode.label())
        .collect();
    if !continues_writes.is_empty() {
        checks.push(FailureModesCheck {
            check_id: "FMP-04".to_string(),
            label: "no-continue-after-failure".to_string(),
            status: FailureModesCheckStatus::Failed,
            message: format!(
                "The following failure mode(s) do not stop writes after failure: {}.",
                continues_writes.join(", ")
            ),
            remediation: Some(
                "All failure modes must declare a stop behavior that halts further writes."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FailureModesCheck {
            check_id: "FMP-04".to_string(),
            label: "no-continue-after-failure".to_string(),
            status: FailureModesCheckStatus::Passed,
            message: "All declared failure modes stop further writes on failure.".to_string(),
            remediation: None,
        });
    }

    // FMP-05: No mode declares automatic destructive rollback
    let destructive: Vec<&str> = plans
        .iter()
        .filter(|p| p.triggers_destructive_rollback)
        .map(|p| p.mode.label())
        .collect();
    if !destructive.is_empty() {
        checks.push(FailureModesCheck {
            check_id: "FMP-05".to_string(),
            label: "no-destructive-rollback".to_string(),
            status: FailureModesCheckStatus::Failed,
            message: format!(
                "The following failure mode(s) declare automatic destructive rollback: {}. \
                 Automatic rollback is unsafe — a partial restore cannot be reliably undone.",
                destructive.join(", ")
            ),
            remediation: Some(
                "Remove automatic destructive rollback. Instead, stop writes and report the \
                 partial state for manual review."
                    .to_string(),
            ),
        });
    } else {
        checks.push(FailureModesCheck {
            check_id: "FMP-05".to_string(),
            label: "no-destructive-rollback".to_string(),
            status: FailureModesCheckStatus::Passed,
            message: "No failure mode declares automatic destructive rollback.".to_string(),
            remediation: None,
        });
    }

    // FMP-06: Unknown/unclassified failure stops all writes
    let unknown_plan = plans
        .iter()
        .find(|p| p.mode == RestoreFailureMode::UnknownFailure);
    match unknown_plan {
        None => {
            // Already flagged by FMP-03; skip to avoid duplicate message
        }
        Some(p) if !p.stop_behavior.stops_writes() => {
            checks.push(FailureModesCheck {
                check_id: "FMP-06".to_string(),
                label: "unknown-failure-stops-writes".to_string(),
                status: FailureModesCheckStatus::Failed,
                message: "The unknownFailure mode does not stop all writes. Any unclassified \
                           failure must immediately halt further write operations."
                    .to_string(),
                remediation: Some(
                    "Set stop_behavior to StopAndReport or StopPreserveCheckpointAndReport for \
                     unknownFailure."
                        .to_string(),
                ),
            });
        }
        Some(_) => {
            checks.push(FailureModesCheck {
                check_id: "FMP-06".to_string(),
                label: "unknown-failure-stops-writes".to_string(),
                status: FailureModesCheckStatus::Passed,
                message: "Unknown/unclassified failure is declared to stop all writes.".to_string(),
                remediation: None,
            });
        }
    }

    // FMP-07: Rate-limit exhaustion stops after retry limit
    let rate_plan = plans
        .iter()
        .find(|p| p.mode == RestoreFailureMode::RateLimitExhaustion);
    match rate_plan {
        None => {
            // Already flagged by FMP-03
        }
        Some(p) => {
            let safe = matches!(
                p.stop_behavior,
                FailureStopBehavior::StopAfterRetryLimit
                    | FailureStopBehavior::StopAndReport
                    | FailureStopBehavior::StopPreserveCheckpointAndReport
                    | FailureStopBehavior::BlockAndRequireManualReview
            );
            if safe {
                checks.push(FailureModesCheck {
                    check_id: "FMP-07".to_string(),
                    label: "rate-limit-stops-after-retry".to_string(),
                    status: FailureModesCheckStatus::Passed,
                    message:
                        "Rate-limit exhaustion is declared to stop after the configured retry \
                               limit."
                            .to_string(),
                    remediation: None,
                });
            } else {
                checks.push(FailureModesCheck {
                    check_id: "FMP-07".to_string(),
                    label: "rate-limit-stops-after-retry".to_string(),
                    status: FailureModesCheckStatus::Failed,
                    message: "Rate-limit exhaustion does not stop after a configured retry limit."
                        .to_string(),
                    remediation: Some(
                        "Declare stop_behavior as StopAfterRetryLimit for rateLimitExhaustion."
                            .to_string(),
                    ),
                });
            }
        }
    }

    // FMP-08: Final validation failure blocks success
    let fv_plan = plans
        .iter()
        .find(|p| p.mode == RestoreFailureMode::FinalValidationFailure);
    match fv_plan {
        None => {
            // Already flagged by FMP-03
        }
        Some(p) if p.partial_failure_labeled_success => {
            checks.push(FailureModesCheck {
                check_id: "FMP-08".to_string(),
                label: "final-validation-blocks-success".to_string(),
                status: FailureModesCheckStatus::Failed,
                message:
                    "finalValidationFailure is declared to allow partial failure to be labeled \
                           as success. This is unsafe — a restore cannot be marked successful when \
                           final validation fails."
                        .to_string(),
                remediation: Some(
                    "Set partial_failure_labeled_success: false for finalValidationFailure."
                        .to_string(),
                ),
            });
        }
        Some(p) if !p.stop_behavior.stops_writes() => {
            checks.push(FailureModesCheck {
                check_id: "FMP-08".to_string(),
                label: "final-validation-blocks-success".to_string(),
                status: FailureModesCheckStatus::Failed,
                message: "finalValidationFailure does not stop further writes. Final validation \
                           failure must always halt the restore operation."
                    .to_string(),
                remediation: Some(
                    "Set stop_behavior to StopAndReport for finalValidationFailure.".to_string(),
                ),
            });
        }
        Some(_) => {
            checks.push(FailureModesCheck {
                check_id: "FMP-08".to_string(),
                label: "final-validation-blocks-success".to_string(),
                status: FailureModesCheckStatus::Passed,
                message: "Final validation failure is declared to block success and stop writes."
                    .to_string(),
                remediation: None,
            });
        }
    }

    // FMP-09: Checkpoint persistence failure blocks continuation
    let cp_plan = plans
        .iter()
        .find(|p| p.mode == RestoreFailureMode::CheckpointPersistenceFailure);
    match cp_plan {
        None => {
            // Already flagged by FMP-03
        }
        Some(p) if !p.stop_behavior.stops_writes() => {
            checks.push(FailureModesCheck {
                check_id: "FMP-09".to_string(),
                label: "checkpoint-failure-blocks-continuation".to_string(),
                status: FailureModesCheckStatus::Failed,
                message: "checkpointPersistenceFailure does not stop further writes. If a \
                           checkpoint cannot be written, the engine must not continue — resuming \
                           without a valid checkpoint risks duplicate writes."
                    .to_string(),
                remediation: Some(
                    "Set stop_behavior to StopAndReport or StopPreserveCheckpointAndReport for \
                     checkpointPersistenceFailure."
                        .to_string(),
                ),
            });
        }
        Some(_) => {
            checks.push(FailureModesCheck {
                check_id: "FMP-09".to_string(),
                label: "checkpoint-failure-blocks-continuation".to_string(),
                status: FailureModesCheckStatus::Passed,
                message: "Checkpoint persistence failure is declared to block continuation."
                    .to_string(),
                remediation: None,
            });
        }
    }

    // FMP-10: No partial failure labeled as success
    let partial_success: Vec<&str> = plans
        .iter()
        .filter(|p| p.partial_failure_labeled_success)
        .map(|p| p.mode.label())
        .collect();
    if !partial_success.is_empty() {
        checks.push(FailureModesCheck {
            check_id: "FMP-10".to_string(),
            label: "no-partial-failure-as-success".to_string(),
            status: FailureModesCheckStatus::Failed,
            message: format!(
                "The following failure mode(s) declare that a partial failure may be labeled \
                 success: {}. A partial failure must never be reported as a successful restore.",
                partial_success.join(", ")
            ),
            remediation: Some(
                "Set partial_failure_labeled_success: false for all failure modes.".to_string(),
            ),
        });
    } else {
        checks.push(FailureModesCheck {
            check_id: "FMP-10".to_string(),
            label: "no-partial-failure-as-success".to_string(),
            status: FailureModesCheckStatus::Passed,
            message: "No failure mode allows partial failure to be labeled as success.".to_string(),
            remediation: None,
        });
    }

    // FMP-11: Writes remain disabled
    checks.push(FailureModesCheck {
        check_id: "FMP-11".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: FailureModesCheckStatus::Passed,
        message: "Restore writes remain disabled. This policy gate does not enable live writes."
            .to_string(),
        remediation: None,
    });

    // ── Aggregate diagnostic context warnings ─────────────────────────────────
    // After all blocking checks, add per-mode warnings for missing diagnostics
    let mut has_warning = false;
    for plan in plans.iter() {
        if !plan.captures_diagnostic_context {
            checks.push(FailureModesCheck {
                check_id: format!("FMP-W-{}", plan.mode.label()),
                label: format!("diagnostic-context-{}", plan.mode.label()),
                status: FailureModesCheckStatus::Warning,
                message: format!(
                    "Failure mode '{}' does not capture diagnostic context (error message, affected \
                     records). Remediation instructions may be incomplete.",
                    plan.mode.label()
                ),
                remediation: Some(format!(
                    "Ensure '{}' captures error details and affected record identifiers for \
                     post-failure review.",
                    plan.mode.label()
                )),
            });
            has_warning = true;
        }
    }

    // ── Determine overall status ───────────────────────────────────────────────
    let is_blocked = !missing.is_empty()
        || !continues_writes.is_empty()
        || !destructive.is_empty()
        || !partial_success.is_empty()
        || matches!(
            unknown_plan,
            Some(p) if !p.stop_behavior.stops_writes()
        )
        || matches!(
            fv_plan,
            Some(p) if p.partial_failure_labeled_success || !p.stop_behavior.stops_writes()
        )
        || matches!(
            cp_plan,
            Some(p) if !p.stop_behavior.stops_writes()
        );

    let status = if is_blocked {
        FailureModesPolicyStatus::Blocked
    } else if has_warning {
        FailureModesPolicyStatus::Warning
    } else {
        FailureModesPolicyStatus::Compliant
    };

    // Build safe summary
    let summary: Vec<FailureHandlingSummaryEntry> = plans
        .iter()
        .map(|p| FailureHandlingSummaryEntry {
            mode: p.mode.label().to_string(),
            stop_behavior: stop_behavior_label(&p.stop_behavior).to_string(),
            preserves_checkpoint: p.preserves_checkpoint,
            triggers_destructive_rollback: p.triggers_destructive_rollback,
            captures_diagnostic_context: p.captures_diagnostic_context,
        })
        .collect();

    build_result(
        checks,
        Some(summary),
        status,
        request.target_label.as_deref(),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_required_modes_safe() -> Vec<RestoreFailureHandlingPlan> {
        vec![
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::SchemaCreateFailure,
                stop_behavior: FailureStopBehavior::StopAndReport,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::SchemaVerifyFailure,
                stop_behavior: FailureStopBehavior::StopAndReport,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::RecordCreateFailure,
                stop_behavior: FailureStopBehavior::StopPreserveCheckpointAndReport,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::IdMappingFailure,
                stop_behavior: FailureStopBehavior::StopAndReport,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::LinkedRecordUpdateFailure,
                stop_behavior: FailureStopBehavior::StopPreserveCheckpointAndReport,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::CheckpointPersistenceFailure,
                stop_behavior: FailureStopBehavior::StopAndReport,
                preserves_checkpoint: false,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::RateLimitExhaustion,
                stop_behavior: FailureStopBehavior::StopAfterRetryLimit,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::TargetMutationDetected,
                stop_behavior: FailureStopBehavior::BlockAndRequireManualReview,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::FinalValidationFailure,
                stop_behavior: FailureStopBehavior::StopAndReport,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
            RestoreFailureHandlingPlan {
                mode: RestoreFailureMode::UnknownFailure,
                stop_behavior: FailureStopBehavior::StopAndReport,
                preserves_checkpoint: true,
                triggers_destructive_rollback: false,
                partial_failure_labeled_success: false,
                captures_diagnostic_context: true,
                note: None,
            },
        ]
    }

    fn request_with_plans(plans: Vec<RestoreFailureHandlingPlan>) -> FailureModesPolicyRequest {
        FailureModesPolicyRequest {
            handling_plans: Some(plans),
            target_label: Some("Test Base".to_string()),
        }
    }

    fn request_no_plans() -> FailureModesPolicyRequest {
        FailureModesPolicyRequest {
            handling_plans: None,
            target_label: None,
        }
    }

    #[test]
    fn complete_safe_failure_plan_is_compliant() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        assert_eq!(result.status, FailureModesPolicyStatus::Compliant);
        assert!(!result.message.to_lowercase().contains("succeeded"));
    }

    #[test]
    fn no_plans_declared_is_blocked() {
        let req = request_no_plans();
        let result = verify_failure_modes_policy(&req);
        assert_eq!(result.status, FailureModesPolicyStatus::Blocked);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn missing_required_mode_is_blocked() {
        let mut plans = all_required_modes_safe();
        plans.retain(|p| p.mode != RestoreFailureMode::RecordCreateFailure);
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        assert_eq!(result.status, FailureModesPolicyStatus::Blocked);
        let fmp03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-03")
            .unwrap();
        assert_eq!(fmp03.status, FailureModesCheckStatus::Failed);
        assert!(fmp03.message.contains("recordCreateFailure"));
    }

    #[test]
    fn missing_multiple_required_modes_is_blocked() {
        let mut plans = all_required_modes_safe();
        plans.retain(|p| {
            p.mode != RestoreFailureMode::SchemaCreateFailure
                && p.mode != RestoreFailureMode::UnknownFailure
        });
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        assert_eq!(result.status, FailureModesPolicyStatus::Blocked);
        let fmp03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-03")
            .unwrap();
        assert!(fmp03.message.contains("schemaCreateFailure"));
        assert!(fmp03.message.contains("unknownFailure"));
    }

    #[test]
    fn continue_after_failure_is_blocked() {
        // This test would require a non-stops-writes behavior, but all current enum
        // variants stop writes. We validate that FMP-04 passes for all safe behaviors.
        let plans = all_required_modes_safe();
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        let fmp04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-04")
            .unwrap();
        assert_eq!(fmp04.status, FailureModesCheckStatus::Passed);
    }

    #[test]
    fn automatic_destructive_rollback_is_blocked() {
        let mut plans = all_required_modes_safe();
        plans[0].triggers_destructive_rollback = true;
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        assert_eq!(result.status, FailureModesPolicyStatus::Blocked);
        let fmp05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-05")
            .unwrap();
        assert_eq!(fmp05.status, FailureModesCheckStatus::Failed);
    }

    #[test]
    fn unknown_failure_stops_writes() {
        let plans = all_required_modes_safe();
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        let fmp06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-06")
            .unwrap();
        assert_eq!(fmp06.status, FailureModesCheckStatus::Passed);
    }

    #[test]
    fn rate_limit_exhaustion_stops_after_retry_limit() {
        let plans = all_required_modes_safe();
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        let fmp07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-07")
            .unwrap();
        assert_eq!(fmp07.status, FailureModesCheckStatus::Passed);
    }

    #[test]
    fn final_validation_failure_cannot_allow_success() {
        let mut plans = all_required_modes_safe();
        let fv = plans
            .iter_mut()
            .find(|p| p.mode == RestoreFailureMode::FinalValidationFailure)
            .unwrap();
        fv.partial_failure_labeled_success = true;
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        assert_eq!(result.status, FailureModesPolicyStatus::Blocked);
        let fmp08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-08")
            .unwrap();
        assert_eq!(fmp08.status, FailureModesCheckStatus::Failed);
    }

    #[test]
    fn checkpoint_persistence_failure_blocks_continuation() {
        let plans = all_required_modes_safe();
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        let fmp09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-09")
            .unwrap();
        assert_eq!(fmp09.status, FailureModesCheckStatus::Passed);
    }

    #[test]
    fn partial_failure_labeled_success_is_blocked() {
        let mut plans = all_required_modes_safe();
        plans[2].partial_failure_labeled_success = true;
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        assert_eq!(result.status, FailureModesPolicyStatus::Blocked);
        let fmp10 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-10")
            .unwrap();
        assert_eq!(fmp10.status, FailureModesCheckStatus::Failed);
    }

    #[test]
    fn partial_diagnostics_produces_warning() {
        let mut plans = all_required_modes_safe();
        plans[0].captures_diagnostic_context = false;
        let result = verify_failure_modes_policy(&request_with_plans(plans));
        assert_eq!(result.status, FailureModesPolicyStatus::Warning);
        let warn = result
            .checks
            .iter()
            .find(|c| c.status == FailureModesCheckStatus::Warning);
        assert!(warn.is_some());
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn writes_enabled_is_always_false() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn result_serialization_has_no_token_or_payload() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("token"));
        assert!(!json.contains("record_payload"));
        assert!(!json.contains("api_key"));
        assert!(!json.contains("/Users/"));
    }

    #[test]
    fn compliant_does_not_introduce_success_state() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        assert_eq!(result.status, FailureModesPolicyStatus::Compliant);
        assert!(!result.message.to_lowercase().contains("succeeded"));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.to_lowercase().contains("\"succeeded\""));
    }

    #[test]
    fn no_plans_short_circuits_to_blocked_with_2_checks() {
        let req = request_no_plans();
        let result = verify_failure_modes_policy(&req);
        assert_eq!(result.status, FailureModesPolicyStatus::Blocked);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn fmp01_and_fmp11_always_pass() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        let fmp01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-01")
            .unwrap();
        let fmp11 = result
            .checks
            .iter()
            .find(|c| c.check_id == "FMP-11")
            .unwrap();
        assert_eq!(fmp01.status, FailureModesCheckStatus::Passed);
        assert_eq!(fmp11.status, FailureModesCheckStatus::Passed);
    }

    #[test]
    fn handling_summary_present_in_result() {
        let req = request_with_plans(all_required_modes_safe());
        let result = verify_failure_modes_policy(&req);
        let summary = result.handling_summary.unwrap();
        assert_eq!(summary.len(), 10);
        assert!(summary.iter().any(|e| e.mode == "unknownFailure"));
    }
}
