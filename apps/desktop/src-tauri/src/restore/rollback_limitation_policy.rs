use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RollbackLimitationPolicyStatus {
    /// Rollback limitation plan is declared and all automatic destructive
    /// rollback/cleanup paths are disabled.
    Compliant,
    /// Plan is declared with safe rollback behavior but recovery guidance or
    /// user-facing notice is incomplete.
    Warning,
    /// Automatic destructive rollback or cleanup is allowed, or the plan is
    /// missing required declarations.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RollbackLimitationCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// Whether automatic rollback is declared for partial restore failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RollbackBehavior {
    /// No automatic rollback — all created objects remain until manual cleanup.
    NoAutomaticRollback,
    /// Automatic destructive rollback declared (deletes all created objects). Blocked.
    AutomaticDestructiveRollback,
    /// Automatic delete cleanup of created records. Blocked.
    AutomaticDeleteCleanup,
    /// Automatic update/revert cleanup of linked fields. Blocked.
    AutomaticUpdateRevertCleanup,
}

impl RollbackBehavior {
    #[cfg(test)]
    fn is_destructive(&self) -> bool {
        match self {
            RollbackBehavior::NoAutomaticRollback => false,
            RollbackBehavior::AutomaticDestructiveRollback => true,
            RollbackBehavior::AutomaticDeleteCleanup => true,
            RollbackBehavior::AutomaticUpdateRevertCleanup => true,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            RollbackBehavior::NoAutomaticRollback => "noAutomaticRollback",
            RollbackBehavior::AutomaticDestructiveRollback => "automaticDestructiveRollback",
            RollbackBehavior::AutomaticDeleteCleanup => "automaticDeleteCleanup",
            RollbackBehavior::AutomaticUpdateRevertCleanup => "automaticUpdateRevertCleanup",
        }
    }
}

/// How users are guided to recover from a partial restore failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecoveryGuidance {
    /// Resume from checkpoint: the restore engine can resume from the last
    /// durable checkpoint, avoiding re-creating already-created objects.
    CheckpointBasedResume,
    /// Manual cleanup: the user must manually inspect and delete partially
    /// created objects in the target base before retrying.
    ManualCleanupRequired,
    /// No guidance declared (warning — user is not informed of options).
    NoneDeClared,
}

impl RecoveryGuidance {
    fn is_declared(&self) -> bool {
        !matches!(self, RecoveryGuidance::NoneDeClared)
    }

    fn includes_checkpoint(&self) -> bool {
        matches!(self, RecoveryGuidance::CheckpointBasedResume)
    }
}

// ── Plan struct ───────────────────────────────────────────────────────────────

/// Rollback limitation declaration for the restore write pipeline.
///
/// All fields are boolean flags, enums, or short labels — no token, no path,
/// no record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackLimitationPlan {
    /// What happens when a partial restore failure occurs.
    pub rollback_behavior: RollbackBehavior,
    /// Whether the plan explicitly declares that partial restore is NOT success.
    pub partial_restore_is_not_success: bool,
    /// How users are guided to recover after a partial failure.
    pub recovery_guidance: RecoveryGuidance,
    /// Whether the UI shows a visible notice that automatic rollback is not available.
    pub user_visible_limitation_notice: bool,
    /// Whether limitation notice includes specific details (tables/records at risk,
    /// manual steps). False triggers Warning.
    pub notice_includes_limitation_details: bool,
    /// Whether manual cleanup requires a separate, explicit future user action
    /// (not triggered automatically by the restore engine).
    pub manual_cleanup_requires_separate_action: bool,
    /// Optional human-readable note. No token, path, or record payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ── Request / Result structs ──────────────────────────────────────────────────

/// Input to the rollback limitation policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackLimitationPolicyRequest {
    /// Declared rollback limitation plan for the restore write operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<RollbackLimitationPlan>,
    /// Safe display label for the restore target (base name only, no path, no token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackLimitationCheck {
    pub check_id: String,
    pub label: String,
    pub status: RollbackLimitationCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Safe read-only summary of the evaluated rollback limitation plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackLimitationSummary {
    pub rollback_behavior: String,
    pub partial_restore_is_not_success: bool,
    pub recovery_guidance_declared: bool,
    pub includes_checkpoint_guidance: bool,
    pub user_visible_notice: bool,
    pub manual_cleanup_requires_separate_action: bool,
}

/// Result from `verify_rollback_limitation_policy`.
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
/// - No automatic destructive rollback, delete, or update cleanup path exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackLimitationPolicyResult {
    pub status: RollbackLimitationPolicyStatus,
    pub checks: Vec<RollbackLimitationCheck>,
    pub message: String,
    /// Safe summary of the evaluated rollback limitation plan (no sensitive values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<RollbackLimitationSummary>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn build_result(
    checks: Vec<RollbackLimitationCheck>,
    plan_summary: Option<RollbackLimitationSummary>,
    status: RollbackLimitationPolicyStatus,
    target_label: Option<&str>,
) -> RollbackLimitationPolicyResult {
    let target_note = target_label
        .map(|l| format!(" for '{l}'"))
        .unwrap_or_default();
    let message = match status {
        RollbackLimitationPolicyStatus::Compliant => format!(
            "Rollback limitation policy is compliant{target_note}. Automatic destructive rollback \
             and cleanup are disabled. Partial restore is not success. Recovery guidance and \
             user-visible limitation notice are declared. Restore writes remain disabled — \
             compliance does not start any write operation and does not introduce a restore \
             success state."
        ),
        RollbackLimitationPolicyStatus::Warning => format!(
            "Rollback limitation policy has warnings{target_note}. Automatic rollback is safely \
             disabled, but recovery guidance or the user-visible notice is incomplete. Restore \
             writes remain disabled."
        ),
        RollbackLimitationPolicyStatus::Blocked => format!(
            "Rollback limitation policy is blocked{target_note}. Automatic destructive rollback \
             or cleanup is allowed, or required declarations are missing. Resolve all violations \
             before any live write is considered. Restore writes remain disabled."
        ),
    };
    RollbackLimitationPolicyResult {
        status,
        checks,
        message,
        plan_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the rollback limitation policy for a planned restore write operation.
///
/// Check IDs:
/// - RLP-01: Write gate is disabled.
/// - RLP-02: Rollback limitation plan is declared.
/// - RLP-03: Automatic destructive rollback is disabled.
/// - RLP-04: Automatic delete cleanup is disabled.
/// - RLP-05: Automatic update/revert cleanup is disabled.
/// - RLP-06: Partial restore is not labeled success.
/// - RLP-07: Checkpoint-based recovery guidance is declared.
/// - RLP-08: User-visible rollback limitation notice is declared.
/// - RLP-09: Manual cleanup requires a separate, explicit future action.
/// - RLP-10: No token, path, or payload exposure (safety invariant).
/// - RLP-11: No network writes attempted (safety invariant).
/// - RLP-12: Writes remain disabled even when compliant.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
pub fn verify_rollback_limitation_policy(
    request: &RollbackLimitationPolicyRequest,
) -> RollbackLimitationPolicyResult {
    let mut checks: Vec<RollbackLimitationCheck> = Vec::new();

    // RLP-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "Write gate is disabled. No restore writes are attempted.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: RollbackLimitationCheckStatus::Failed,
            message: "Write gate is not disabled. Rollback limitation policy cannot be evaluated \
                      while write gate is active."
                .to_string(),
            remediation: Some(
                "Ensure evaluate_write_gate() returns Disabled before running policy checks."
                    .to_string(),
            ),
        });
        return build_result(checks, None, RollbackLimitationPolicyStatus::Blocked, None);
    }

    // RLP-02: Plan declared — short-circuit if absent
    let plan = match &request.plan {
        Some(p) => p,
        None => {
            checks.push(RollbackLimitationCheck {
                check_id: "RLP-02".to_string(),
                label: "plan-declared".to_string(),
                status: RollbackLimitationCheckStatus::Failed,
                message: "No rollback limitation plan declared. A plan declaring rollback \
                          behavior, recovery guidance, and user-visible notice is required before \
                          any live write path is considered."
                    .to_string(),
                remediation: Some(
                    "Declare a RollbackLimitationPlan with rollbackBehavior, recoveryGuidance, \
                     and userVisibleLimitationNotice."
                        .to_string(),
                ),
            });
            return build_result(
                checks,
                None,
                RollbackLimitationPolicyStatus::Blocked,
                request.target_label.as_deref(),
            );
        }
    };

    checks.push(RollbackLimitationCheck {
        check_id: "RLP-02".to_string(),
        label: "plan-declared".to_string(),
        status: RollbackLimitationCheckStatus::Passed,
        message: "Rollback limitation plan is declared.".to_string(),
        remediation: None,
    });

    // Build plan summary (safe — no sensitive values)
    let plan_summary = RollbackLimitationSummary {
        rollback_behavior: plan.rollback_behavior.label().to_string(),
        partial_restore_is_not_success: plan.partial_restore_is_not_success,
        recovery_guidance_declared: plan.recovery_guidance.is_declared(),
        includes_checkpoint_guidance: plan.recovery_guidance.includes_checkpoint(),
        user_visible_notice: plan.user_visible_limitation_notice,
        manual_cleanup_requires_separate_action: plan.manual_cleanup_requires_separate_action,
    };

    let mut blocked = false;
    let mut has_warning = false;

    // RLP-03: No automatic destructive rollback
    if plan.rollback_behavior == RollbackBehavior::AutomaticDestructiveRollback {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-03".to_string(),
            label: "no-automatic-destructive-rollback".to_string(),
            status: RollbackLimitationCheckStatus::Failed,
            message: "Automatic destructive rollback is declared. The restore engine must not \
                      automatically delete all created objects on failure — this would cause \
                      data loss without user consent."
                .to_string(),
            remediation: Some(
                "Set rollbackBehavior to noAutomaticRollback. Destructive cleanup must require \
                 explicit separate user action."
                    .to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-03".to_string(),
            label: "no-automatic-destructive-rollback".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "Automatic destructive rollback is not declared.".to_string(),
            remediation: None,
        });
    }

    // RLP-04: No automatic delete cleanup
    if plan.rollback_behavior == RollbackBehavior::AutomaticDeleteCleanup {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-04".to_string(),
            label: "no-automatic-delete-cleanup".to_string(),
            status: RollbackLimitationCheckStatus::Failed,
            message: "Automatic delete cleanup is declared. The restore engine must not \
                      automatically delete partially created records on failure."
                .to_string(),
            remediation: Some(
                "Set rollbackBehavior to noAutomaticRollback. Delete cleanup must require \
                 explicit separate user action."
                    .to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-04".to_string(),
            label: "no-automatic-delete-cleanup".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "Automatic delete cleanup is not declared.".to_string(),
            remediation: None,
        });
    }

    // RLP-05: No automatic update/revert cleanup
    if plan.rollback_behavior == RollbackBehavior::AutomaticUpdateRevertCleanup {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-05".to_string(),
            label: "no-automatic-update-revert-cleanup".to_string(),
            status: RollbackLimitationCheckStatus::Failed,
            message: "Automatic update/revert cleanup is declared. The restore engine must not \
                      automatically revert or update partially linked records on failure."
                .to_string(),
            remediation: Some(
                "Set rollbackBehavior to noAutomaticRollback. Revert cleanup must require \
                 explicit separate user action."
                    .to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-05".to_string(),
            label: "no-automatic-update-revert-cleanup".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "Automatic update/revert cleanup is not declared.".to_string(),
            remediation: None,
        });
    }

    // RLP-06: Partial restore is not success
    if !plan.partial_restore_is_not_success {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-06".to_string(),
            label: "partial-restore-is-not-success".to_string(),
            status: RollbackLimitationCheckStatus::Failed,
            message:
                "Plan does not declare that partial restore is not success. A partial restore \
                      that fails mid-way must never be reported as a successful completion."
                    .to_string(),
            remediation: Some(
                "Set partialRestoreIsNotSuccess: true in the rollback limitation plan.".to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-06".to_string(),
            label: "partial-restore-is-not-success".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "Partial restore is explicitly declared as not a success state.".to_string(),
            remediation: None,
        });
    }

    // RLP-07: Checkpoint-based recovery guidance declared
    if !plan.recovery_guidance.is_declared() {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-07".to_string(),
            label: "checkpoint-recovery-guidance".to_string(),
            status: RollbackLimitationCheckStatus::Warning,
            message: "No recovery guidance is declared. Users must be informed of how to recover \
                      from a partial restore failure (checkpoint resume or manual cleanup)."
                .to_string(),
            remediation: Some(
                "Set recoveryGuidance to checkpointBasedResume or manualCleanupRequired."
                    .to_string(),
            ),
        });
        has_warning = true;
    } else if !plan.recovery_guidance.includes_checkpoint() {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-07".to_string(),
            label: "checkpoint-recovery-guidance".to_string(),
            status: RollbackLimitationCheckStatus::Warning,
            message: "Recovery guidance is declared but does not include checkpoint-based resume. \
                      Checkpoint-based recovery is preferred to avoid duplicating already-created \
                      objects."
                .to_string(),
            remediation: Some(
                "Consider adding checkpointBasedResume as the primary recovery guidance."
                    .to_string(),
            ),
        });
        has_warning = true;
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-07".to_string(),
            label: "checkpoint-recovery-guidance".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "Checkpoint-based recovery guidance is declared.".to_string(),
            remediation: None,
        });
    }

    // RLP-08: User-visible limitation notice declared
    if !plan.user_visible_limitation_notice {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-08".to_string(),
            label: "user-visible-limitation-notice".to_string(),
            status: RollbackLimitationCheckStatus::Warning,
            message: "No user-visible rollback limitation notice is declared. Users must be \
                      informed that automatic rollback is not available before any restore \
                      operation begins."
                .to_string(),
            remediation: Some(
                "Set userVisibleLimitationNotice: true and show the notice in the UI before \
                 restore execution."
                    .to_string(),
            ),
        });
        has_warning = true;
    } else if !plan.notice_includes_limitation_details {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-08".to_string(),
            label: "user-visible-limitation-notice".to_string(),
            status: RollbackLimitationCheckStatus::Warning,
            message: "User-visible notice is declared but does not include limitation details \
                      (affected tables/records, manual steps). Incomplete notice may leave users \
                      unable to understand the risk."
                .to_string(),
            remediation: Some(
                "Set noticeIncludesLimitationDetails: true and ensure the notice describes \
                 which objects may remain and what manual cleanup is required."
                    .to_string(),
            ),
        });
        has_warning = true;
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-08".to_string(),
            label: "user-visible-limitation-notice".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "User-visible rollback limitation notice with details is declared."
                .to_string(),
            remediation: None,
        });
    }

    // RLP-09: Manual cleanup requires separate explicit future action
    if !plan.manual_cleanup_requires_separate_action {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-09".to_string(),
            label: "manual-cleanup-separate-action".to_string(),
            status: RollbackLimitationCheckStatus::Failed,
            message: "Plan does not declare that manual cleanup requires a separate, explicit \
                      future user action. The restore engine must never trigger cleanup \
                      automatically — users must initiate it explicitly in a separate flow."
                .to_string(),
            remediation: Some(
                "Set manualCleanupRequiresSeparateAction: true. Cleanup tooling must be a \
                 separate future feature requiring explicit user initiation."
                    .to_string(),
            ),
        });
        blocked = true;
    } else {
        checks.push(RollbackLimitationCheck {
            check_id: "RLP-09".to_string(),
            label: "manual-cleanup-separate-action".to_string(),
            status: RollbackLimitationCheckStatus::Passed,
            message: "Manual cleanup is declared to require a separate, explicit future user \
                      action."
                .to_string(),
            remediation: None,
        });
    }

    // RLP-10: No token/path/payload exposure (safety invariant — always passes)
    checks.push(RollbackLimitationCheck {
        check_id: "RLP-10".to_string(),
        label: "no-token-path-payload".to_string(),
        status: RollbackLimitationCheckStatus::Passed,
        message: "No token, filesystem path, or record payload is present in any result field."
            .to_string(),
        remediation: None,
    });

    // RLP-11: No network writes attempted (safety invariant — always passes)
    checks.push(RollbackLimitationCheck {
        check_id: "RLP-11".to_string(),
        label: "no-network-writes".to_string(),
        status: RollbackLimitationCheckStatus::Passed,
        message: "No network writes have been attempted. networkWritesAttempted is always false."
            .to_string(),
        remediation: None,
    });

    // RLP-12: Writes remain disabled (safety invariant — always passes)
    checks.push(RollbackLimitationCheck {
        check_id: "RLP-12".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: RollbackLimitationCheckStatus::Passed,
        message: "Restore writes remain disabled. Policy compliance does not enable write \
                  execution."
            .to_string(),
        remediation: None,
    });

    let status = if blocked {
        RollbackLimitationPolicyStatus::Blocked
    } else if has_warning {
        RollbackLimitationPolicyStatus::Warning
    } else {
        RollbackLimitationPolicyStatus::Compliant
    };

    build_result(
        checks,
        Some(plan_summary),
        status,
        request.target_label.as_deref(),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_plan() -> RollbackLimitationPlan {
        RollbackLimitationPlan {
            rollback_behavior: RollbackBehavior::NoAutomaticRollback,
            partial_restore_is_not_success: true,
            recovery_guidance: RecoveryGuidance::CheckpointBasedResume,
            user_visible_limitation_notice: true,
            notice_includes_limitation_details: true,
            manual_cleanup_requires_separate_action: true,
            note: None,
        }
    }

    fn request_with_plan(plan: RollbackLimitationPlan) -> RollbackLimitationPolicyRequest {
        RollbackLimitationPolicyRequest {
            plan: Some(plan),
            target_label: None,
        }
    }

    fn request_no_plan() -> RollbackLimitationPolicyRequest {
        RollbackLimitationPolicyRequest {
            plan: None,
            target_label: None,
        }
    }

    #[test]
    fn complete_safe_plan_is_compliant() {
        let result = verify_rollback_limitation_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Compliant);
    }

    #[test]
    fn missing_plan_is_blocked() {
        let result = verify_rollback_limitation_policy(&request_no_plan());
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Blocked);
        assert_eq!(result.checks.len(), 2);
        assert_eq!(
            result.checks[1].status,
            RollbackLimitationCheckStatus::Failed
        );
    }

    #[test]
    fn automatic_destructive_rollback_is_blocked() {
        let mut plan = safe_plan();
        plan.rollback_behavior = RollbackBehavior::AutomaticDestructiveRollback;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Blocked);
        let rlp03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-03")
            .unwrap();
        assert_eq!(rlp03.status, RollbackLimitationCheckStatus::Failed);
    }

    #[test]
    fn automatic_delete_cleanup_is_blocked() {
        let mut plan = safe_plan();
        plan.rollback_behavior = RollbackBehavior::AutomaticDeleteCleanup;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Blocked);
        let rlp04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-04")
            .unwrap();
        assert_eq!(rlp04.status, RollbackLimitationCheckStatus::Failed);
    }

    #[test]
    fn automatic_update_revert_cleanup_is_blocked() {
        let mut plan = safe_plan();
        plan.rollback_behavior = RollbackBehavior::AutomaticUpdateRevertCleanup;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Blocked);
        let rlp05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-05")
            .unwrap();
        assert_eq!(rlp05.status, RollbackLimitationCheckStatus::Failed);
    }

    #[test]
    fn partial_restore_labeled_success_is_blocked() {
        let mut plan = safe_plan();
        plan.partial_restore_is_not_success = false;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Blocked);
        let rlp06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-06")
            .unwrap();
        assert_eq!(rlp06.status, RollbackLimitationCheckStatus::Failed);
    }

    #[test]
    fn missing_manual_cleanup_separate_action_is_blocked() {
        let mut plan = safe_plan();
        plan.manual_cleanup_requires_separate_action = false;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Blocked);
        let rlp09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-09")
            .unwrap();
        assert_eq!(rlp09.status, RollbackLimitationCheckStatus::Failed);
    }

    #[test]
    fn no_recovery_guidance_is_warning() {
        let mut plan = safe_plan();
        plan.recovery_guidance = RecoveryGuidance::NoneDeClared;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Warning);
        let rlp07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-07")
            .unwrap();
        assert_eq!(rlp07.status, RollbackLimitationCheckStatus::Warning);
    }

    #[test]
    fn manual_cleanup_only_recovery_guidance_is_warning() {
        let mut plan = safe_plan();
        plan.recovery_guidance = RecoveryGuidance::ManualCleanupRequired;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Warning);
        let rlp07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-07")
            .unwrap();
        assert_eq!(rlp07.status, RollbackLimitationCheckStatus::Warning);
    }

    #[test]
    fn missing_user_visible_notice_is_warning() {
        let mut plan = safe_plan();
        plan.user_visible_limitation_notice = false;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Warning);
        let rlp08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-08")
            .unwrap();
        assert_eq!(rlp08.status, RollbackLimitationCheckStatus::Warning);
    }

    #[test]
    fn notice_without_details_is_warning() {
        let mut plan = safe_plan();
        plan.notice_includes_limitation_details = false;
        let result = verify_rollback_limitation_policy(&request_with_plan(plan));
        assert_eq!(result.status, RollbackLimitationPolicyStatus::Warning);
        let rlp08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-08")
            .unwrap();
        assert_eq!(rlp08.status, RollbackLimitationCheckStatus::Warning);
    }

    #[test]
    fn complete_plan_has_twelve_checks() {
        let result = verify_rollback_limitation_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.checks.len(), 12);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let results = vec![
            verify_rollback_limitation_policy(&request_no_plan()),
            verify_rollback_limitation_policy(&request_with_plan(safe_plan())),
            verify_rollback_limitation_policy(&request_with_plan({
                let mut p = safe_plan();
                p.rollback_behavior = RollbackBehavior::AutomaticDestructiveRollback;
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
            verify_rollback_limitation_policy(&request_no_plan()),
            verify_rollback_limitation_policy(&request_with_plan(safe_plan())),
        ];
        for r in results {
            assert!(!r.writes_enabled);
        }
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let result = verify_rollback_limitation_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn rlp01_and_rlp12_always_pass() {
        let result = verify_rollback_limitation_policy(&request_with_plan(safe_plan()));
        let rlp01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-01")
            .unwrap();
        let rlp12 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLP-12")
            .unwrap();
        assert_eq!(rlp01.status, RollbackLimitationCheckStatus::Passed);
        assert_eq!(rlp12.status, RollbackLimitationCheckStatus::Passed);
    }

    #[test]
    fn plan_summary_present_for_complete_plan() {
        let result = verify_rollback_limitation_policy(&request_with_plan(safe_plan()));
        assert!(result.plan_summary.is_some());
        let summary = result.plan_summary.unwrap();
        assert_eq!(summary.rollback_behavior, "noAutomaticRollback");
        assert!(summary.partial_restore_is_not_success);
        assert!(summary.recovery_guidance_declared);
        assert!(summary.includes_checkpoint_guidance);
        assert!(summary.user_visible_notice);
        assert!(summary.manual_cleanup_requires_separate_action);
    }

    #[test]
    fn plan_summary_absent_when_no_plan() {
        let result = verify_rollback_limitation_policy(&request_no_plan());
        assert!(result.plan_summary.is_none());
    }

    #[test]
    fn no_token_or_path_in_serialized_result() {
        let result = verify_rollback_limitation_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        // No actual token value patterns
        assert!(!json.contains("pat_"));
        assert!(!json.contains("apiKey"));
        // No filesystem path
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        // No token field in JSON object (the word appears in check messages but not as a key-value pair with a sensitive value)
        assert!(!json.contains("\"token\":"));
    }

    #[test]
    fn no_success_state_in_result_message() {
        let result = verify_rollback_limitation_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.to_lowercase().contains("succeeded"));
        assert!(!result.message.to_lowercase().contains("restore complete"));
    }

    #[test]
    fn rollback_behavior_is_destructive_for_all_cleanup_variants() {
        assert!(!RollbackBehavior::NoAutomaticRollback.is_destructive());
        assert!(RollbackBehavior::AutomaticDestructiveRollback.is_destructive());
        assert!(RollbackBehavior::AutomaticDeleteCleanup.is_destructive());
        assert!(RollbackBehavior::AutomaticUpdateRevertCleanup.is_destructive());
    }

    #[test]
    fn target_label_appears_in_compliant_message() {
        let request = RollbackLimitationPolicyRequest {
            plan: Some(safe_plan()),
            target_label: Some("my-base".to_string()),
        };
        let result = verify_rollback_limitation_policy(&request);
        assert!(result.message.contains("my-base"));
    }
}
