use crate::airtable::rate_limit::{DEFAULT_MAX_RETRIES, DEFAULT_PER_BASE_RPS};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum safe requests per second per base (Airtable limit is 5).
pub const SAFE_MAX_RPS: u32 = DEFAULT_PER_BASE_RPS;

/// Maximum safe batch size for record create/update operations.
pub const SAFE_MAX_BATCH_SIZE: u32 = 10;

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RateLimitBackoffPolicyStatus {
    /// All required rate-limit, backoff, and checkpoint fields are within safe bounds.
    Compliant,
    /// One or more fields are incomplete or indicate a partial/unknown state,
    /// but no unsafe threshold is exceeded.
    Warning,
    /// A hard safety threshold is exceeded or a required field is missing.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RateLimitBackoffCheckStatus {
    Passed,
    Warning,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RateLimitBackoffFailureReason {
    /// Declared RPS exceeds the Airtable-safe threshold.
    ExceedsMaxRequestsPerSecond,
    /// Declared batch size exceeds the safe maximum.
    ExceedsMaxBatchSize,
    /// No 429 handling strategy declared.
    Missing429Handling,
    /// No retry limit declared (unbounded retries are unsafe).
    UnboundedRetries,
    /// No backoff strategy declared.
    MissingBackoffStrategy,
    /// No stop condition after repeated rate-limit failures declared.
    MissingStopCondition,
    /// Write gate is unexpectedly enabled.
    WriteGateEnabled,
}

// ── Plan struct ───────────────────────────────────────────────────────────────

/// Declared throttling and backoff plan for a future restore write operation.
///
/// All fields are numeric counts or boolean flags — no token, no path, no
/// record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitBackoffPlan {
    /// Declared maximum requests per second per base (must be ≤ SAFE_MAX_RPS).
    pub max_requests_per_second: u32,
    /// Declared batch size for record create/update operations (must be ≤ 10).
    pub batch_size: u32,
    /// Whether a 429 response handling strategy is declared.
    pub handles_429: bool,
    /// Maximum number of retries before giving up (0 = no retries; None = not declared).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// Whether an exponential or fixed backoff delay between retries is declared.
    pub has_backoff_strategy: bool,
    /// Whether a stop condition is declared after repeated rate-limit failures.
    pub has_stop_condition: bool,
    /// Checkpoint/resume compatibility: "full", "partial", "none", or None (unknown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_compatibility: Option<String>,
}

// ── Request / Result structs ──────────────────────────────────────────────────

/// Input to the rate-limit and backoff policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitBackoffPolicyRequest {
    /// The declared throttling and backoff plan for the restore write operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<RateLimitBackoffPlan>,
    /// Safe display label for the restore target (base name only, no path, no token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitBackoffCheck {
    pub check_id: String,
    pub label: String,
    pub status: RateLimitBackoffCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Result from `verify_rate_limit_backoff_policy`.
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
pub struct RateLimitBackoffPolicyResult {
    pub status: RateLimitBackoffPolicyStatus,
    pub checks: Vec<RateLimitBackoffCheck>,
    pub message: String,
    /// Safe, human-readable summary of the evaluated plan (no sensitive values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<RateLimitBackoffPlanSummary>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

/// Read-only summary of the evaluated plan fields, safe for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitBackoffPlanSummary {
    pub max_requests_per_second: u32,
    pub batch_size: u32,
    pub handles_429: bool,
    pub max_retries: Option<u32>,
    pub has_backoff_strategy: bool,
    pub has_stop_condition: bool,
    pub checkpoint_compatibility: Option<String>,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the rate-limit and backoff policy for a planned restore write operation.
///
/// Check IDs:
/// - RLB-01: Write gate is disabled.
/// - RLB-02: Plan is declared.
/// - RLB-03: Requests per second is within the safe threshold (≤ 5).
/// - RLB-04: Batch size is within the safe limit (≤ 10).
/// - RLB-05: 429 handling is declared.
/// - RLB-06: Retry limit is declared and bounded.
/// - RLB-07: Backoff strategy is declared.
/// - RLB-08: Stop condition is declared.
/// - RLB-09: Checkpoint/resume compatibility is declared.
/// - RLB-10: Writes remain disabled.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
pub fn verify_rate_limit_backoff_policy(
    request: &RateLimitBackoffPolicyRequest,
) -> RateLimitBackoffPolicyResult {
    let mut checks: Vec<RateLimitBackoffCheck> = Vec::new();

    // RLB-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: RateLimitBackoffCheckStatus::Passed,
            message: "Restore write gate is disabled. No live writes are possible.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: RateLimitBackoffCheckStatus::Failed,
            message: "Restore write gate is unexpectedly enabled.".to_string(),
            remediation: Some(
                "The write gate must remain disabled until all safety gates are satisfied."
                    .to_string(),
            ),
        });
    }

    // RLB-02: Plan declared
    let plan = match &request.plan {
        Some(p) => p,
        None => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-02".to_string(),
                label: "plan-declared".to_string(),
                status: RateLimitBackoffCheckStatus::Failed,
                message: "No rate-limit/backoff plan declared. A plan is required before any live write path is considered.".to_string(),
                remediation: Some("Declare a RateLimitBackoffPlan with all required fields.".to_string()),
            });
            // Cannot evaluate remaining checks without a plan.
            return build_result(
                checks,
                None,
                RateLimitBackoffPolicyStatus::Blocked,
                request.target_label.as_deref(),
            );
        }
    };

    checks.push(RateLimitBackoffCheck {
        check_id: "RLB-02".to_string(),
        label: "plan-declared".to_string(),
        status: RateLimitBackoffCheckStatus::Passed,
        message: "Rate-limit/backoff plan is declared.".to_string(),
        remediation: None,
    });

    // RLB-03: Requests per second within safe threshold
    if plan.max_requests_per_second > SAFE_MAX_RPS {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-03".to_string(),
            label: "requests-per-second-safe".to_string(),
            status: RateLimitBackoffCheckStatus::Failed,
            message: format!(
                "Declared max requests per second ({}) exceeds the safe threshold ({}).",
                plan.max_requests_per_second, SAFE_MAX_RPS
            ),
            remediation: Some(format!(
                "Reduce max_requests_per_second to {} or below.",
                SAFE_MAX_RPS
            )),
        });
    } else {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-03".to_string(),
            label: "requests-per-second-safe".to_string(),
            status: RateLimitBackoffCheckStatus::Passed,
            message: format!(
                "Declared max requests per second ({}) is within the safe threshold ({}).",
                plan.max_requests_per_second, SAFE_MAX_RPS
            ),
            remediation: None,
        });
    }

    // RLB-04: Batch size within safe limit
    if plan.batch_size > SAFE_MAX_BATCH_SIZE {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-04".to_string(),
            label: "batch-size-safe".to_string(),
            status: RateLimitBackoffCheckStatus::Failed,
            message: format!(
                "Declared batch size ({}) exceeds the safe maximum ({}).",
                plan.batch_size, SAFE_MAX_BATCH_SIZE
            ),
            remediation: Some(format!(
                "Reduce batch_size to {} or below.",
                SAFE_MAX_BATCH_SIZE
            )),
        });
    } else {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-04".to_string(),
            label: "batch-size-safe".to_string(),
            status: RateLimitBackoffCheckStatus::Passed,
            message: format!(
                "Declared batch size ({}) is within the safe maximum ({}).",
                plan.batch_size, SAFE_MAX_BATCH_SIZE
            ),
            remediation: None,
        });
    }

    // RLB-05: 429 handling declared
    if !plan.handles_429 {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-05".to_string(),
            label: "handles-429".to_string(),
            status: RateLimitBackoffCheckStatus::Failed,
            message: "No 429 (rate-limit) response handling strategy is declared.".to_string(),
            remediation: Some(
                "Declare a 429 handling strategy (backoff and retry) before any live write."
                    .to_string(),
            ),
        });
    } else {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-05".to_string(),
            label: "handles-429".to_string(),
            status: RateLimitBackoffCheckStatus::Passed,
            message: "A 429 response handling strategy is declared.".to_string(),
            remediation: None,
        });
    }

    // RLB-06: Retry limit declared and bounded
    match plan.max_retries {
        None => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-06".to_string(),
                label: "retry-limit-bounded".to_string(),
                status: RateLimitBackoffCheckStatus::Failed,
                message: "No maximum retry limit declared. Unbounded retries are unsafe."
                    .to_string(),
                remediation: Some(format!(
                    "Declare a bounded max_retries value (recommended: {}).",
                    DEFAULT_MAX_RETRIES
                )),
            });
        }
        Some(0) => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-06".to_string(),
                label: "retry-limit-bounded".to_string(),
                status: RateLimitBackoffCheckStatus::Passed,
                message: "Maximum retry limit is declared (0 — no retries).".to_string(),
                remediation: None,
            });
        }
        Some(n) => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-06".to_string(),
                label: "retry-limit-bounded".to_string(),
                status: RateLimitBackoffCheckStatus::Passed,
                message: format!("Maximum retry limit is declared ({}).", n),
                remediation: None,
            });
        }
    }

    // RLB-07: Backoff strategy declared
    if !plan.has_backoff_strategy {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-07".to_string(),
            label: "backoff-strategy-declared".to_string(),
            status: RateLimitBackoffCheckStatus::Failed,
            message: "No backoff delay strategy declared between retries.".to_string(),
            remediation: Some(
                "Declare an exponential or fixed backoff strategy before any live write."
                    .to_string(),
            ),
        });
    } else {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-07".to_string(),
            label: "backoff-strategy-declared".to_string(),
            status: RateLimitBackoffCheckStatus::Passed,
            message: "A backoff strategy between retries is declared.".to_string(),
            remediation: None,
        });
    }

    // RLB-08: Stop condition declared
    if !plan.has_stop_condition {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-08".to_string(),
            label: "stop-condition-declared".to_string(),
            status: RateLimitBackoffCheckStatus::Failed,
            message: "No stop condition declared after repeated rate-limit failures.".to_string(),
            remediation: Some(
                "Declare a stop condition that aborts the operation after too many 429 failures."
                    .to_string(),
            ),
        });
    } else {
        checks.push(RateLimitBackoffCheck {
            check_id: "RLB-08".to_string(),
            label: "stop-condition-declared".to_string(),
            status: RateLimitBackoffCheckStatus::Passed,
            message: "A stop condition for repeated rate-limit failures is declared.".to_string(),
            remediation: None,
        });
    }

    // RLB-09: Checkpoint/resume compatibility
    match plan.checkpoint_compatibility.as_deref() {
        Some("full") => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-09".to_string(),
                label: "checkpoint-compatibility".to_string(),
                status: RateLimitBackoffCheckStatus::Passed,
                message: "Full checkpoint/resume compatibility is declared.".to_string(),
                remediation: None,
            });
        }
        Some("partial") => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-09".to_string(),
                label: "checkpoint-compatibility".to_string(),
                status: RateLimitBackoffCheckStatus::Warning,
                message: "Checkpoint/resume compatibility is partial. Full compatibility is recommended for long operations.".to_string(),
                remediation: Some(
                    "Upgrade to full checkpoint/resume compatibility before enabling live writes."
                        .to_string(),
                ),
            });
        }
        Some("none") => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-09".to_string(),
                label: "checkpoint-compatibility".to_string(),
                status: RateLimitBackoffCheckStatus::Warning,
                message: "No checkpoint/resume compatibility declared. Long operations cannot be safely resumed after interruption.".to_string(),
                remediation: Some(
                    "Implement checkpoint/resume support before enabling live writes for long operations."
                        .to_string(),
                ),
            });
        }
        _ => {
            checks.push(RateLimitBackoffCheck {
                check_id: "RLB-09".to_string(),
                label: "checkpoint-compatibility".to_string(),
                status: RateLimitBackoffCheckStatus::Warning,
                message: "Checkpoint/resume compatibility is not declared or unknown.".to_string(),
                remediation: Some(
                    "Declare checkpoint_compatibility as \"full\", \"partial\", or \"none\"."
                        .to_string(),
                ),
            });
        }
    }

    // RLB-10: Writes remain disabled
    checks.push(RateLimitBackoffCheck {
        check_id: "RLB-10".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: RateLimitBackoffCheckStatus::Passed,
        message: "Restore write execution is not enabled. Verifying this policy does not start any write operation.".to_string(),
        remediation: None,
    });

    let plan_summary = Some(RateLimitBackoffPlanSummary {
        max_requests_per_second: plan.max_requests_per_second,
        batch_size: plan.batch_size,
        handles_429: plan.handles_429,
        max_retries: plan.max_retries,
        has_backoff_strategy: plan.has_backoff_strategy,
        has_stop_condition: plan.has_stop_condition,
        checkpoint_compatibility: plan.checkpoint_compatibility.clone(),
    });

    let has_blocked = checks
        .iter()
        .any(|c| c.status == RateLimitBackoffCheckStatus::Failed);
    let has_warning = checks
        .iter()
        .any(|c| c.status == RateLimitBackoffCheckStatus::Warning);

    let status = if has_blocked {
        RateLimitBackoffPolicyStatus::Blocked
    } else if has_warning {
        RateLimitBackoffPolicyStatus::Warning
    } else {
        RateLimitBackoffPolicyStatus::Compliant
    };

    build_result(
        checks,
        plan_summary,
        status,
        request.target_label.as_deref(),
    )
}

fn build_result(
    checks: Vec<RateLimitBackoffCheck>,
    plan_summary: Option<RateLimitBackoffPlanSummary>,
    status: RateLimitBackoffPolicyStatus,
    target_label: Option<&str>,
) -> RateLimitBackoffPolicyResult {
    let target_name = target_label
        .filter(|s| !s.is_empty())
        .unwrap_or("the restore target");

    let message = match &status {
        RateLimitBackoffPolicyStatus::Compliant => format!(
            "Rate-limit and backoff policy for {} is compliant. All required fields are within safe bounds. Restore writes remain disabled.",
            target_name
        ),
        RateLimitBackoffPolicyStatus::Warning => format!(
            "Rate-limit and backoff policy for {} has warnings. No unsafe threshold is exceeded, but some fields are incomplete or partial. Restore writes remain disabled.",
            target_name
        ),
        RateLimitBackoffPolicyStatus::Blocked => format!(
            "Rate-limit and backoff policy for {} is blocked. One or more required fields are missing or exceed safe thresholds. Restore writes remain disabled.",
            target_name
        ),
    };

    RateLimitBackoffPolicyResult {
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

    fn safe_plan() -> RateLimitBackoffPlan {
        RateLimitBackoffPlan {
            max_requests_per_second: 5,
            batch_size: 10,
            handles_429: true,
            max_retries: Some(3),
            has_backoff_strategy: true,
            has_stop_condition: true,
            checkpoint_compatibility: Some("full".to_string()),
        }
    }

    fn request_with_plan(plan: RateLimitBackoffPlan) -> RateLimitBackoffPolicyRequest {
        RateLimitBackoffPolicyRequest {
            plan: Some(plan),
            target_label: Some("My Base".to_string()),
        }
    }

    // ── Status outcomes ───────────────────────────────────────────────────────

    #[test]
    fn safe_plan_returns_compliant() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Compliant);
    }

    #[test]
    fn no_plan_returns_blocked() {
        let request = RateLimitBackoffPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_rate_limit_backoff_policy(&request);
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Blocked);
    }

    #[test]
    fn rps_exceeds_threshold_returns_blocked() {
        let mut plan = safe_plan();
        plan.max_requests_per_second = 6;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Blocked);
    }

    #[test]
    fn rps_at_threshold_returns_compliant() {
        let mut plan = safe_plan();
        plan.max_requests_per_second = 5;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Compliant);
    }

    #[test]
    fn batch_size_exceeds_10_returns_blocked() {
        let mut plan = safe_plan();
        plan.batch_size = 11;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Blocked);
    }

    #[test]
    fn batch_size_at_10_returns_compliant() {
        let mut plan = safe_plan();
        plan.batch_size = 10;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Compliant);
    }

    #[test]
    fn missing_429_handling_returns_blocked() {
        let mut plan = safe_plan();
        plan.handles_429 = false;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Blocked);
    }

    #[test]
    fn unbounded_retries_returns_blocked() {
        let mut plan = safe_plan();
        plan.max_retries = None;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Blocked);
    }

    #[test]
    fn zero_retries_returns_compliant() {
        let mut plan = safe_plan();
        plan.max_retries = Some(0);
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Compliant);
    }

    #[test]
    fn missing_backoff_strategy_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_backoff_strategy = false;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Blocked);
    }

    #[test]
    fn missing_stop_condition_returns_blocked() {
        let mut plan = safe_plan();
        plan.has_stop_condition = false;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Blocked);
    }

    #[test]
    fn partial_checkpoint_returns_warning() {
        let mut plan = safe_plan();
        plan.checkpoint_compatibility = Some("partial".to_string());
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Warning);
    }

    #[test]
    fn no_checkpoint_returns_warning() {
        let mut plan = safe_plan();
        plan.checkpoint_compatibility = Some("none".to_string());
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Warning);
    }

    #[test]
    fn unknown_checkpoint_returns_warning() {
        let mut plan = safe_plan();
        plan.checkpoint_compatibility = None;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Warning);
    }

    // ── Check IDs ─────────────────────────────────────────────────────────────

    #[test]
    fn ten_checks_present_when_plan_declared() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.checks.len(), 10);
    }

    #[test]
    fn two_checks_when_no_plan() {
        let request = RateLimitBackoffPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_rate_limit_backoff_policy(&request);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn check_ids_rlb_01_through_10() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        for i in 1..=10 {
            let expected = format!("RLB-{:02}", i);
            assert!(
                ids.contains(&expected.as_str()),
                "missing check {}",
                expected
            );
        }
    }

    #[test]
    fn rlb_01_always_passes() {
        let request = RateLimitBackoffPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_rate_limit_backoff_policy(&request);
        let rlb01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLB-01")
            .unwrap();
        assert_eq!(rlb01.status, RateLimitBackoffCheckStatus::Passed);
    }

    #[test]
    fn rlb_10_always_passes() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        let rlb10 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLB-10")
            .unwrap();
        assert_eq!(rlb10.status, RateLimitBackoffCheckStatus::Passed);
    }

    #[test]
    fn rlb_03_fails_when_rps_exceeds_threshold() {
        let mut plan = safe_plan();
        plan.max_requests_per_second = 100;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        let rlb03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLB-03")
            .unwrap();
        assert_eq!(rlb03.status, RateLimitBackoffCheckStatus::Failed);
    }

    #[test]
    fn rlb_04_fails_when_batch_exceeds_10() {
        let mut plan = safe_plan();
        plan.batch_size = 50;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        let rlb04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "RLB-04")
            .unwrap();
        assert_eq!(rlb04.status, RateLimitBackoffCheckStatus::Failed);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true_compliant() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert!(result.no_changes_made);
    }

    #[test]
    fn no_changes_made_always_true_blocked() {
        let request = RateLimitBackoffPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_rate_limit_backoff_policy(&request);
        assert!(result.no_changes_made);
    }

    #[test]
    fn writes_enabled_always_false_compliant() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert!(!result.writes_enabled);
    }

    #[test]
    fn writes_enabled_always_false_blocked() {
        let mut plan = safe_plan();
        plan.max_requests_per_second = 100;
        let result = verify_rate_limit_backoff_policy(&request_with_plan(plan));
        assert!(!result.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn compliant_does_not_enable_writes() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Compliant);
        assert!(!result.writes_enabled);
    }

    // ── Serialization safety ──────────────────────────────────────────────────

    #[test]
    fn serialization_has_no_token() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
    }

    #[test]
    fn serialization_has_no_full_path() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn serialization_has_no_record_payload() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("\"fields\""));
        assert!(!json.contains("\"recordId\""));
    }

    #[test]
    fn message_does_not_contain_token() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert!(!result.message.contains("token"));
        assert!(!result.message.contains("pat_"));
    }

    #[test]
    fn message_says_writes_remain_disabled_when_compliant() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert_eq!(result.status, RateLimitBackoffPolicyStatus::Compliant);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn plan_summary_present_when_plan_declared() {
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert!(result.plan_summary.is_some());
        let summary = result.plan_summary.unwrap();
        assert_eq!(summary.max_requests_per_second, 5);
        assert_eq!(summary.batch_size, 10);
        assert!(summary.handles_429);
        assert_eq!(summary.max_retries, Some(3));
    }

    #[test]
    fn plan_summary_absent_when_no_plan() {
        let request = RateLimitBackoffPolicyRequest {
            plan: None,
            target_label: None,
        };
        let result = verify_rate_limit_backoff_policy(&request);
        assert!(result.plan_summary.is_none());
    }

    #[test]
    fn no_write_calls_made_during_verification() {
        // Gate is always disabled — Airtable cannot be reached from this function.
        let gate = evaluate_write_gate();
        assert_eq!(gate.status, RestoreWriteEngineStatus::Disabled);
        let result = verify_rate_limit_backoff_policy(&request_with_plan(safe_plan()));
        assert!(!result.network_writes_attempted);
        assert!(!result.writes_enabled);
    }
}
