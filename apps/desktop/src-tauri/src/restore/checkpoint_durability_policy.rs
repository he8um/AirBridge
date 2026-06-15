use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CheckpointDurabilityPolicyStatus {
    /// All required checkpoint and durability fields are declared and safe.
    Compliant,
    /// One or more fields indicate an incomplete or degraded durability state,
    /// but no hard safety threshold is violated.
    Warning,
    /// A required checkpoint field is missing or an unsafe condition exists.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CheckpointDurabilityCheckStatus {
    Passed,
    Warning,
    Failed,
}

// ── Plan struct ───────────────────────────────────────────────────────────────

/// Declared checkpoint and durability plan for a future restore write operation.
///
/// All fields are boolean flags or string labels — no token, no path, no
/// record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointDurabilityPlan {
    /// Whether a checkpoint is written after each individual table completes.
    pub checkpoint_after_each_table: bool,
    /// Whether a checkpoint is written after each record batch completes.
    pub checkpoint_after_each_batch: bool,
    /// Whether a phase marker is recorded for each of: schema, record_create,
    /// linked_update, final_validation.
    pub has_phase_markers: bool,
    /// Whether an old-to-new record ID mapping checkpoint is planned before
    /// linked record updates begin.
    pub has_id_mapping_checkpoint: bool,
    /// Whether the operation has a resume-safe stop condition that allows
    /// restart from the last checkpoint without duplicating work.
    pub has_resume_safe_stop_condition: bool,
    /// Whether linked record updates are declared in this plan.
    pub has_linked_updates: bool,
    /// Durability backend: "disk", "memory", "remote", or None (unknown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub durability_backend: Option<String>,
}

// ── Request / Result structs ──────────────────────────────────────────────────

/// Input to the checkpoint durability policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointDurabilityPolicyRequest {
    /// The declared checkpoint and durability plan for the restore write operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<CheckpointDurabilityPlan>,
    /// Safe display label for the restore target (base name only, no path, no token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointDurabilityCheck {
    pub check_id: String,
    pub label: String,
    pub status: CheckpointDurabilityCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Read-only summary of the evaluated plan fields, safe for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointDurabilityPlanSummary {
    pub checkpoint_after_each_table: bool,
    pub checkpoint_after_each_batch: bool,
    pub has_phase_markers: bool,
    pub has_id_mapping_checkpoint: bool,
    pub has_resume_safe_stop_condition: bool,
    pub has_linked_updates: bool,
    pub durability_backend: Option<String>,
}

/// Result from `verify_checkpoint_durability_policy`.
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
pub struct CheckpointDurabilityPolicyResult {
    pub status: CheckpointDurabilityPolicyStatus,
    pub checks: Vec<CheckpointDurabilityCheck>,
    pub message: String,
    /// Safe, human-readable summary of the evaluated plan (no sensitive values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<CheckpointDurabilityPlanSummary>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the checkpoint durability policy for a planned restore write operation.
///
/// Check IDs:
/// - CDP-01: Write gate is disabled.
/// - CDP-02: Checkpoint plan is declared.
/// - CDP-03: Checkpoint after each table is declared.
/// - CDP-04: Checkpoint after each record batch is declared.
/// - CDP-05: Phase markers are declared for all required phases.
/// - CDP-06: Old-to-new ID mapping checkpoint is declared before linked updates.
/// - CDP-07: Resume-safe stop condition is declared.
/// - CDP-08: Durability backend is not memory-only.
/// - CDP-09: Writes remain disabled.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
pub fn verify_checkpoint_durability_policy(
    request: &CheckpointDurabilityPolicyRequest,
) -> CheckpointDurabilityPolicyResult {
    let mut checks: Vec<CheckpointDurabilityCheck> = Vec::new();

    // CDP-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: CheckpointDurabilityCheckStatus::Passed,
            message: "Restore write gate is disabled. No live writes are possible.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: CheckpointDurabilityCheckStatus::Failed,
            message: "Restore write gate is unexpectedly enabled.".to_string(),
            remediation: Some(
                "The write gate must remain disabled until all safety gates are satisfied."
                    .to_string(),
            ),
        });
    }

    // CDP-02: Plan declared
    let plan = match &request.plan {
        Some(p) => p,
        None => {
            checks.push(CheckpointDurabilityCheck {
                check_id: "CDP-02".to_string(),
                label: "plan-declared".to_string(),
                status: CheckpointDurabilityCheckStatus::Failed,
                message: "No checkpoint durability plan declared. A plan is required before any \
                           live write path is considered."
                    .to_string(),
                remediation: Some(
                    "Declare a CheckpointDurabilityPlan with all required fields.".to_string(),
                ),
            });
            // Cannot evaluate remaining checks without a plan.
            return build_result(
                checks,
                None,
                CheckpointDurabilityPolicyStatus::Blocked,
                request.target_label.as_deref(),
            );
        }
    };

    checks.push(CheckpointDurabilityCheck {
        check_id: "CDP-02".to_string(),
        label: "plan-declared".to_string(),
        status: CheckpointDurabilityCheckStatus::Passed,
        message: "Checkpoint durability plan is declared.".to_string(),
        remediation: None,
    });

    // CDP-03: Checkpoint after each table
    if !plan.checkpoint_after_each_table {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-03".to_string(),
            label: "checkpoint-after-each-table".to_string(),
            status: CheckpointDurabilityCheckStatus::Failed,
            message: "No checkpoint after each table is declared. A durable checkpoint must be \
                       written after each table completes to allow safe resumption."
                .to_string(),
            remediation: Some(
                "Declare checkpoint_after_each_table: true in the durability plan.".to_string(),
            ),
        });
    } else {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-03".to_string(),
            label: "checkpoint-after-each-table".to_string(),
            status: CheckpointDurabilityCheckStatus::Passed,
            message: "Checkpoint after each table is declared.".to_string(),
            remediation: None,
        });
    }

    // CDP-04: Checkpoint after each batch
    if !plan.checkpoint_after_each_batch {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-04".to_string(),
            label: "checkpoint-after-each-batch".to_string(),
            status: CheckpointDurabilityCheckStatus::Failed,
            message: "No checkpoint after each record batch is declared. A durable checkpoint \
                       must be written after each batch to prevent duplicate writes on resume."
                .to_string(),
            remediation: Some(
                "Declare checkpoint_after_each_batch: true in the durability plan.".to_string(),
            ),
        });
    } else {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-04".to_string(),
            label: "checkpoint-after-each-batch".to_string(),
            status: CheckpointDurabilityCheckStatus::Passed,
            message: "Checkpoint after each record batch is declared.".to_string(),
            remediation: None,
        });
    }

    // CDP-05: Phase markers for all required phases
    if !plan.has_phase_markers {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-05".to_string(),
            label: "phase-markers-declared".to_string(),
            status: CheckpointDurabilityCheckStatus::Failed,
            message: "Phase markers are not declared. Phase markers for schema, record_create, \
                       linked_update, and final_validation are required to track restore progress."
                .to_string(),
            remediation: Some(
                "Declare phase markers for all four required phases: schema, record_create, \
                 linked_update, final_validation."
                    .to_string(),
            ),
        });
    } else {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-05".to_string(),
            label: "phase-markers-declared".to_string(),
            status: CheckpointDurabilityCheckStatus::Passed,
            message: "Phase markers for all required phases are declared.".to_string(),
            remediation: None,
        });
    }

    // CDP-06: ID mapping checkpoint before linked updates
    if plan.has_linked_updates && !plan.has_id_mapping_checkpoint {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-06".to_string(),
            label: "id-mapping-checkpoint".to_string(),
            status: CheckpointDurabilityCheckStatus::Failed,
            message: "Linked record updates are declared but no old-to-new ID mapping checkpoint \
                       is planned. The mapping must be persisted before any linked update begins."
                .to_string(),
            remediation: Some(
                "Declare has_id_mapping_checkpoint: true to persist the record ID mapping before \
                 linked updates begin."
                    .to_string(),
            ),
        });
    } else if !plan.has_linked_updates && !plan.has_id_mapping_checkpoint {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-06".to_string(),
            label: "id-mapping-checkpoint".to_string(),
            status: CheckpointDurabilityCheckStatus::Passed,
            message: "No linked record updates declared — ID mapping checkpoint not required."
                .to_string(),
            remediation: None,
        });
    } else {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-06".to_string(),
            label: "id-mapping-checkpoint".to_string(),
            status: CheckpointDurabilityCheckStatus::Passed,
            message: "Old-to-new record ID mapping checkpoint is declared before linked updates."
                .to_string(),
            remediation: None,
        });
    }

    // CDP-07: Resume-safe stop condition
    if !plan.has_resume_safe_stop_condition {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-07".to_string(),
            label: "resume-safe-stop-condition".to_string(),
            status: CheckpointDurabilityCheckStatus::Failed,
            message: "No resume-safe stop condition is declared. The restore operation must be \
                       able to restart from the last checkpoint without duplicating writes."
                .to_string(),
            remediation: Some(
                "Declare a resume-safe stop condition that allows resumption from any checkpoint."
                    .to_string(),
            ),
        });
    } else {
        checks.push(CheckpointDurabilityCheck {
            check_id: "CDP-07".to_string(),
            label: "resume-safe-stop-condition".to_string(),
            status: CheckpointDurabilityCheckStatus::Passed,
            message: "A resume-safe stop condition is declared.".to_string(),
            remediation: None,
        });
    }

    // CDP-08: Durability backend not memory-only
    match plan.durability_backend.as_deref() {
        Some("memory") => {
            checks.push(CheckpointDurabilityCheck {
                check_id: "CDP-08".to_string(),
                label: "durability-backend".to_string(),
                status: CheckpointDurabilityCheckStatus::Warning,
                message: "Durability backend is memory-only. Checkpoints will be lost if the \
                           process exits — long operations cannot be safely resumed after a crash."
                    .to_string(),
                remediation: Some(
                    "Use a disk or remote durability backend for production restore operations."
                        .to_string(),
                ),
            });
        }
        Some("disk") | Some("remote") => {
            checks.push(CheckpointDurabilityCheck {
                check_id: "CDP-08".to_string(),
                label: "durability-backend".to_string(),
                status: CheckpointDurabilityCheckStatus::Passed,
                message: format!(
                    "Durability backend is '{}' — checkpoints survive process restart.",
                    plan.durability_backend.as_deref().unwrap_or("unknown")
                ),
                remediation: None,
            });
        }
        _ => {
            checks.push(CheckpointDurabilityCheck {
                check_id: "CDP-08".to_string(),
                label: "durability-backend".to_string(),
                status: CheckpointDurabilityCheckStatus::Warning,
                message: "Durability backend is not declared or unknown. Checkpoint durability \
                           cannot be confirmed."
                    .to_string(),
                remediation: Some(
                    "Declare durability_backend as \"disk\", \"remote\", or \"memory\"."
                        .to_string(),
                ),
            });
        }
    }

    // CDP-09: Writes remain disabled
    checks.push(CheckpointDurabilityCheck {
        check_id: "CDP-09".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: CheckpointDurabilityCheckStatus::Passed,
        message: "Restore write execution is not enabled. Verifying this policy does not start \
                   any write operation."
            .to_string(),
        remediation: None,
    });

    let plan_summary = Some(CheckpointDurabilityPlanSummary {
        checkpoint_after_each_table: plan.checkpoint_after_each_table,
        checkpoint_after_each_batch: plan.checkpoint_after_each_batch,
        has_phase_markers: plan.has_phase_markers,
        has_id_mapping_checkpoint: plan.has_id_mapping_checkpoint,
        has_resume_safe_stop_condition: plan.has_resume_safe_stop_condition,
        has_linked_updates: plan.has_linked_updates,
        durability_backend: plan.durability_backend.clone(),
    });

    let has_blocked = checks
        .iter()
        .any(|c| c.status == CheckpointDurabilityCheckStatus::Failed);
    let has_warning = checks
        .iter()
        .any(|c| c.status == CheckpointDurabilityCheckStatus::Warning);

    let status = if has_blocked {
        CheckpointDurabilityPolicyStatus::Blocked
    } else if has_warning {
        CheckpointDurabilityPolicyStatus::Warning
    } else {
        CheckpointDurabilityPolicyStatus::Compliant
    };

    build_result(
        checks,
        plan_summary,
        status,
        request.target_label.as_deref(),
    )
}

fn build_result(
    checks: Vec<CheckpointDurabilityCheck>,
    plan_summary: Option<CheckpointDurabilityPlanSummary>,
    status: CheckpointDurabilityPolicyStatus,
    target_label: Option<&str>,
) -> CheckpointDurabilityPolicyResult {
    let target_name = target_label
        .filter(|s| !s.is_empty())
        .unwrap_or("the restore target");

    let message = match &status {
        CheckpointDurabilityPolicyStatus::Compliant => format!(
            "Checkpoint durability policy for {} is compliant. All required checkpoint fields are \
             declared. Restore writes remain disabled.",
            target_name
        ),
        CheckpointDurabilityPolicyStatus::Warning => format!(
            "Checkpoint durability policy for {} has warnings. No required field is missing, but \
             some durability conditions are incomplete. Restore writes remain disabled.",
            target_name
        ),
        CheckpointDurabilityPolicyStatus::Blocked => format!(
            "Checkpoint durability policy for {} is blocked. One or more required checkpoint \
             fields are missing. Restore writes remain disabled.",
            target_name
        ),
    };

    CheckpointDurabilityPolicyResult {
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

    fn safe_plan() -> CheckpointDurabilityPlan {
        CheckpointDurabilityPlan {
            checkpoint_after_each_table: true,
            checkpoint_after_each_batch: true,
            has_phase_markers: true,
            has_id_mapping_checkpoint: true,
            has_resume_safe_stop_condition: true,
            has_linked_updates: true,
            durability_backend: Some("disk".to_string()),
        }
    }

    fn request_with_plan(plan: CheckpointDurabilityPlan) -> CheckpointDurabilityPolicyRequest {
        CheckpointDurabilityPolicyRequest {
            plan: Some(plan),
            target_label: Some("My Base".to_string()),
        }
    }

    // ── Status outcomes ───────────────────────────────────────────────────────

    #[test]
    fn complete_plan_returns_compliant() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Compliant);
    }

    #[test]
    fn no_plan_returns_blocked() {
        let request = CheckpointDurabilityPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_checkpoint_durability_policy(&request);
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Blocked);
    }

    #[test]
    fn missing_table_checkpoint_returns_blocked() {
        let mut plan = safe_plan();
        plan.checkpoint_after_each_table = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Blocked);
    }

    #[test]
    fn missing_batch_checkpoint_returns_blocked() {
        let mut plan = safe_plan();
        plan.checkpoint_after_each_batch = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Blocked);
    }

    #[test]
    fn missing_phase_markers_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_phase_markers = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Blocked);
    }

    #[test]
    fn linked_updates_without_id_mapping_checkpoint_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_linked_updates = true;
        plan.has_id_mapping_checkpoint = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Blocked);
    }

    #[test]
    fn no_linked_updates_no_id_mapping_checkpoint_returns_compliant() {
        let mut plan = safe_plan();
        plan.has_linked_updates = false;
        plan.has_id_mapping_checkpoint = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Compliant);
    }

    #[test]
    fn missing_resume_safe_stop_condition_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_resume_safe_stop_condition = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Blocked);
    }

    #[test]
    fn memory_only_backend_returns_warning() {
        let mut plan = safe_plan();
        plan.durability_backend = Some("memory".to_string());
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Warning);
    }

    #[test]
    fn unknown_backend_returns_warning() {
        let mut plan = safe_plan();
        plan.durability_backend = None;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Warning);
    }

    #[test]
    fn remote_backend_returns_compliant() {
        let mut plan = safe_plan();
        plan.durability_backend = Some("remote".to_string());
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Compliant);
    }

    // ── Check IDs ─────────────────────────────────────────────────────────────

    #[test]
    fn nine_checks_present_when_plan_declared() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.checks.len(), 9);
    }

    #[test]
    fn two_checks_when_no_plan() {
        let request = CheckpointDurabilityPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_checkpoint_durability_policy(&request);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn check_ids_cdp_01_through_09() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        for i in 1..=9 {
            let expected = format!("CDP-{:02}", i);
            assert!(
                ids.contains(&expected.as_str()),
                "missing check {}",
                expected
            );
        }
    }

    #[test]
    fn cdp_01_always_passes() {
        let request = CheckpointDurabilityPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_checkpoint_durability_policy(&request);
        let cdp01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CDP-01")
            .unwrap();
        assert_eq!(cdp01.status, CheckpointDurabilityCheckStatus::Passed);
    }

    #[test]
    fn cdp_09_always_passes() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        let cdp09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CDP-09")
            .unwrap();
        assert_eq!(cdp09.status, CheckpointDurabilityCheckStatus::Passed);
    }

    #[test]
    fn cdp_03_fails_when_no_table_checkpoint() {
        let mut plan = safe_plan();
        plan.checkpoint_after_each_table = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        let cdp03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CDP-03")
            .unwrap();
        assert_eq!(cdp03.status, CheckpointDurabilityCheckStatus::Failed);
    }

    #[test]
    fn cdp_04_fails_when_no_batch_checkpoint() {
        let mut plan = safe_plan();
        plan.checkpoint_after_each_batch = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        let cdp04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CDP-04")
            .unwrap();
        assert_eq!(cdp04.status, CheckpointDurabilityCheckStatus::Failed);
    }

    #[test]
    fn cdp_06_fails_when_linked_updates_without_id_mapping() {
        let mut plan = safe_plan();
        plan.has_linked_updates = true;
        plan.has_id_mapping_checkpoint = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        let cdp06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CDP-06")
            .unwrap();
        assert_eq!(cdp06.status, CheckpointDurabilityCheckStatus::Failed);
    }

    #[test]
    fn cdp_08_warns_on_memory_backend() {
        let mut plan = safe_plan();
        plan.durability_backend = Some("memory".to_string());
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        let cdp08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "CDP-08")
            .unwrap();
        assert_eq!(cdp08.status, CheckpointDurabilityCheckStatus::Warning);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true_compliant() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert!(result.no_changes_made);
    }

    #[test]
    fn no_changes_made_always_true_blocked() {
        let request = CheckpointDurabilityPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_checkpoint_durability_policy(&request);
        assert!(result.no_changes_made);
    }

    #[test]
    fn writes_enabled_always_false_compliant() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert!(!result.writes_enabled);
    }

    #[test]
    fn writes_enabled_always_false_blocked() {
        let mut plan = safe_plan();
        plan.checkpoint_after_each_batch = false;
        let result = verify_checkpoint_durability_policy(&request_with_plan(plan));
        assert!(!result.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn compliant_does_not_enable_writes() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Compliant);
        assert!(!result.writes_enabled);
    }

    // ── Serialization safety ──────────────────────────────────────────────────

    #[test]
    fn serialization_has_no_token() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
    }

    #[test]
    fn serialization_has_no_full_path() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn serialization_has_no_record_payload() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("\"fields\""));
        assert!(!json.contains("\"recordId\""));
    }

    #[test]
    fn message_does_not_contain_token() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.contains("token"));
        assert!(!result.message.contains("pat_"));
    }

    #[test]
    fn message_says_writes_remain_disabled_when_compliant() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, CheckpointDurabilityPolicyStatus::Compliant);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn plan_summary_present_when_plan_declared() {
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert!(result.plan_summary.is_some());
        let summary = result.plan_summary.unwrap();
        assert!(summary.checkpoint_after_each_table);
        assert!(summary.checkpoint_after_each_batch);
        assert!(summary.has_phase_markers);
        assert_eq!(summary.durability_backend, Some("disk".to_string()));
    }

    #[test]
    fn plan_summary_absent_when_no_plan() {
        let request = CheckpointDurabilityPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_checkpoint_durability_policy(&request);
        assert!(result.plan_summary.is_none());
    }

    #[test]
    fn no_write_calls_made_during_verification() {
        let gate = evaluate_write_gate();
        assert_eq!(gate.status, RestoreWriteEngineStatus::Disabled);
        let result = verify_checkpoint_durability_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
        assert!(!result.writes_enabled);
    }
}
