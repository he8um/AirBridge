use serde::{Deserialize, Serialize};

use crate::restore::linked_second_pass_execution_preview::{
    LinkedSecondPassExecutionPreviewStatus, LinkedSecondPassFieldSummary,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Execution status for the linked second-pass executor foundation.
///
/// Safety invariants:
/// - `DryRunOnly` does NOT enable live linked record updates.
/// - `NotExecuted` is the expected state when the write gate is disabled.
/// - `Blocked` indicates a safety prerequisite is missing.
/// - No status named `succeeded`, `complete`, or `done` exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedSecondPassExecutorStatus {
    /// All prerequisites satisfied but the write gate is disabled.
    /// This is the current expected state — no execution occurs.
    NotExecuted,
    /// Dry-run plan built; execution would be sandbox-only.
    /// Write gate must be explicitly enabled before this transitions to execution.
    DryRunOnly,
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Execution mode for the linked second-pass executor.
///
/// Safety invariants:
/// - `Disabled` is the only reachable mode in the current implementation.
/// - `SandboxOnly` is defined for future use but is unreachable while
///   `evaluate_write_gate()` returns `Disabled`.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedSecondPassExecutorMode {
    /// Write gate is disabled — no execution is possible. Default state.
    Disabled,
    /// Sandbox-only mode — execution is restricted to verified sandbox targets.
    /// Unreachable in the current implementation.
    SandboxOnly,
}

/// Status of a single batch in the executor's internal plan.
///
/// Note: `succeeded` / `completed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedSecondPassExecutorBatchStatus {
    /// The batch would be executed if the gate were enabled. Not executed.
    Pending,
    /// The batch is blocked by a safety prerequisite.
    Blocked,
    /// The batch is skipped (e.g. no records for this field pair).
    Skipped,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single ordered batch in the executor's internal plan.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No old or new Airtable record IDs.
/// - No raw record field values.
/// - No raw HTTP body.
/// - No attachment URL.
/// - `status` is never `succeeded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassExecutorBatch {
    pub batch_index: usize,
    pub batch_id: String,
    /// Safe table label — not a live Airtable ID.
    pub table_label: String,
    /// Safe field label — not a live Airtable field ID.
    pub field_label: String,
    /// Number of records in this batch. Always <= batch_size.
    pub update_count: usize,
    /// Safe count of mapping entries required (no raw IDs).
    pub mapping_coverage_count: usize,
    pub status: LinkedSecondPassExecutorBatchStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the linked second-pass executor foundation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassExecutorSafetySnapshot {
    /// Write gate result — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the mode is sandbox-only (always `false` in the current build).
    pub sandbox_mode_active: bool,
    /// Whether the explicit internal linked second-pass flag was set.
    pub explicit_internal_write_requested: bool,
    /// Whether sandbox verification passed.
    pub sandbox_verified: bool,
    /// Whether target empty verification passed.
    pub target_empty_verified: bool,
    /// Whether the record write executor foundation completed safely.
    pub record_executor_safe: bool,
    /// Whether the linked second-pass preview returned DryRunReady.
    pub linked_second_pass_preview_ready: bool,
    /// Whether the mapping/checkpoint preview returned DryRunReady.
    pub mapping_checkpoint_preview_ready: bool,
    /// Whether sensitive data safety policy is satisfied.
    pub sensitive_data_safe: bool,
    /// Whether live-write readiness is ready or warning-safe.
    pub live_write_readiness_satisfied: bool,
}

/// Request to the linked second-pass executor foundation.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_linked_second_pass_requested` must be `true` for the executor
/// to proceed past the gate check. It is an internal-only guard — there is no
/// UI control that sets it, and the write gate must also allow linked record updates
/// (which it currently never does).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassExecutorRequest {
    /// Must be `sandboxOnly` for execution to be considered.
    /// `disabled` (the default) always results in `Blocked`.
    pub mode: LinkedSecondPassExecutorMode,
    /// Internal-only flag. Must be explicitly `true` to proceed past the gate.
    /// No UI control sets this; it is an internal safety guard.
    pub explicit_internal_linked_second_pass_requested: bool,
    /// Whether the sandbox environment check has passed.
    pub sandbox_verified: bool,
    /// Whether the target empty verification has passed.
    pub target_empty_verified: bool,
    /// Whether the record write executor foundation completed safely.
    pub record_executor_safe: bool,
    /// Whether the linked second-pass execution preview returned DryRunReady.
    pub linked_second_pass_preview_ready: bool,
    /// Whether the linked second-pass preview is confirmed dryRunReady (not just warning).
    pub linked_second_pass_preview_status: LinkedSecondPassExecutionPreviewStatus,
    /// Whether the mapping/checkpoint execution preview returned DryRunReady.
    pub mapping_checkpoint_preview_ready: bool,
    /// Whether sensitive data safety policy is satisfied.
    pub sensitive_data_safe: bool,
    /// Whether live-write readiness is ready or warning-safe.
    pub live_write_readiness_satisfied: bool,
    /// Batch size. Must be <= 10.
    pub batch_size: usize,
    /// Per-field summaries used to build the internal batch plan.
    /// No raw record IDs — only safe counts and labels.
    pub field_summaries: Vec<LinkedSecondPassFieldSummary>,
}

/// Result of the linked second-pass executor foundation.
///
/// Safety invariants (enforced):
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `succeeded`, `complete`, or `done`.
/// - `NotExecuted` / `DryRunOnly` do NOT enable live linked record updates.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassExecutorResult {
    pub status: LinkedSecondPassExecutorStatus,
    pub mode: LinkedSecondPassExecutorMode,
    pub message: String,
    pub batches: Vec<LinkedSecondPassExecutorBatch>,
    pub safety_snapshot: LinkedSecondPassExecutorSafetySnapshot,
    pub total_batch_count: usize,
    pub first_field_batch_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — live linked record updates are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const LSEX_PREREQ_WRITE_GATE: &str = "LSEX-PRE-01";
const LSEX_PREREQ_MODE: &str = "LSEX-PRE-02";
const LSEX_PREREQ_EXPLICIT_FLAG: &str = "LSEX-PRE-03";
const LSEX_PREREQ_SANDBOX: &str = "LSEX-PRE-04";
const LSEX_PREREQ_TARGET_EMPTY: &str = "LSEX-PRE-05";
const LSEX_PREREQ_RECORD_EXECUTOR: &str = "LSEX-PRE-06";
const LSEX_PREREQ_LINKED_PREVIEW: &str = "LSEX-PRE-07";
const LSEX_PREREQ_MAPPING_CHECKPOINT: &str = "LSEX-PRE-08";
const LSEX_PREREQ_SENSITIVE_DATA: &str = "LSEX-PRE-09";
const LSEX_PREREQ_LWR: &str = "LSEX-PRE-10";

const LSEX_MAX_BATCH_SIZE: usize = 10;

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the linked second-pass executor foundation plan.
///
/// This function:
/// - Never calls the Airtable API.
/// - Never creates, updates, or deletes any record.
/// - Always enforces the write gate (`evaluate_write_gate()` always returns Disabled).
/// - Always returns `writes_enabled: false`, `no_changes_made: true`,
///   `network_writes_attempted: false`.
/// - Returns `Blocked` when any prerequisite is missing.
/// - Returns `NotExecuted` when all prerequisites are met but the gate is Disabled.
/// - Returns `DryRunOnly` only when all safety prerequisites pass AND the write gate
///   permits linked record updates. Since `evaluate_write_gate()` currently always
///   returns `Disabled`, `DryRunOnly` is currently unreachable.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_linked_second_pass_executor_plan(
    request: &LinkedSecondPassExecutorRequest,
) -> LinkedSecondPassExecutorResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let safety_snapshot = LinkedSecondPassExecutorSafetySnapshot {
        write_gate_disabled,
        sandbox_mode_active: matches!(request.mode, LinkedSecondPassExecutorMode::SandboxOnly),
        explicit_internal_write_requested: request.explicit_internal_linked_second_pass_requested,
        sandbox_verified: request.sandbox_verified,
        target_empty_verified: request.target_empty_verified,
        record_executor_safe: request.record_executor_safe,
        linked_second_pass_preview_ready: request.linked_second_pass_preview_ready,
        mapping_checkpoint_preview_ready: request.mapping_checkpoint_preview_ready,
        sensitive_data_safe: request.sensitive_data_safe,
        live_write_readiness_satisfied: request.live_write_readiness_satisfied,
    };

    // Check prerequisites in order; first failure blocks.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        // Defense-in-depth: unreachable given evaluate_write_gate() always returns Disabled.
        Some(format!(
            "{LSEX_PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Linked second-pass executor must not proceed while write gate could be enabled."
        ))
    } else if !matches!(request.mode, LinkedSecondPassExecutorMode::SandboxOnly) {
        Some(format!(
            "{LSEX_PREREQ_MODE}: Executor mode must be sandboxOnly. \
             Mode 'disabled' does not permit execution. \
             No linked record updates will be attempted."
        ))
    } else if !request.explicit_internal_linked_second_pass_requested {
        Some(format!(
            "{LSEX_PREREQ_EXPLICIT_FLAG}: Explicit internal linked second-pass flag is not set. \
             The internal flag must be explicitly true before execution is considered. \
             No UI control sets this flag."
        ))
    } else if !request.sandbox_verified {
        Some(format!(
            "{LSEX_PREREQ_SANDBOX}: Sandbox environment verification has not passed. \
             A verified sandbox target is required before linked record updates are considered."
        ))
    } else if !request.target_empty_verified {
        Some(format!(
            "{LSEX_PREREQ_TARGET_EMPTY}: Target empty verification has not passed. \
             The target base must be confirmed empty before linked record updates are considered."
        ))
    } else if !request.record_executor_safe {
        Some(format!(
            "{LSEX_PREREQ_RECORD_EXECUTOR}: Record write executor foundation has not completed \
             safely. First-pass record creates must be safe or notExecuted before the \
             linked second pass can proceed."
        ))
    } else if !request.linked_second_pass_preview_ready
        || request.linked_second_pass_preview_status
            == LinkedSecondPassExecutionPreviewStatus::Blocked
    {
        Some(format!(
            "{LSEX_PREREQ_LINKED_PREVIEW}: Linked second-pass execution preview has not returned \
             DryRunReady. Complete the linked second-pass preview before requesting execution."
        ))
    } else if !request.mapping_checkpoint_preview_ready {
        Some(format!(
            "{LSEX_PREREQ_MAPPING_CHECKPOINT}: Mapping/checkpoint execution preview has not \
             returned DryRunReady. Complete the mapping/checkpoint preview before the \
             linked second pass can proceed."
        ))
    } else if !request.sensitive_data_safe {
        Some(format!(
            "{LSEX_PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied. \
             All exposure surfaces must have redaction coverage before linked record updates."
        ))
    } else if !request.live_write_readiness_satisfied {
        Some(format!(
            "{LSEX_PREREQ_LWR}: Live-write readiness policy is not satisfied. \
             All upstream safety gates must be ready or warning-safe."
        ))
    } else {
        // Validate batch size
        if request.batch_size > LSEX_MAX_BATCH_SIZE || request.batch_size == 0 {
            Some(format!(
                "LSEX-BATCH-SIZE: Batch size {} is outside the safe range [1, {LSEX_MAX_BATCH_SIZE}]. \
                 Reduce batch size before proceeding.",
                request.batch_size
            ))
        } else {
            // Validate mapping coverage: each field summary with record_count > 0 must have
            // at least the preview's mapping_coverage_count available (safe count check only).
            let over_batch: Vec<&LinkedSecondPassFieldSummary> = request
                .field_summaries
                .iter()
                .filter(|f| f.record_count > 0)
                .filter(|f| {
                    // Each field batch must not exceed max batch size
                    let batches_needed = f.record_count.div_ceil(request.batch_size);
                    batches_needed > 0 && f.record_count > LSEX_MAX_BATCH_SIZE * batches_needed
                })
                .collect();
            if !over_batch.is_empty() {
                Some(format!(
                    "LSEX-MAPPING: One or more field summaries have record counts that exceed \
                     the safe batch ceiling. Count: {}.",
                    over_batch.len()
                ))
            } else {
                None
            }
        }
    };

    if let Some(ref reason) = blocked_reason {
        return LinkedSecondPassExecutorResult {
            status: LinkedSecondPassExecutorStatus::Blocked,
            mode: LinkedSecondPassExecutorMode::Disabled,
            message: format!(
                "Linked second-pass executor is blocked. {reason} \
                 No linked record updates will be attempted."
            ),
            batches: vec![blocked_executor_batch()],
            safety_snapshot,
            total_batch_count: 0,
            first_field_batch_count: 0,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the internal batch plan.
    let batches = build_executor_batches(&request.field_summaries, request.batch_size);
    let total = batches.len();
    // Count batches for first field only (for reporting granularity)
    let first_field_label = request.field_summaries.first().map(|f| &f.field_label);
    let first_field_batch_count = first_field_label
        .map(|lbl| batches.iter().filter(|b| &b.field_label == lbl).count())
        .unwrap_or(0);

    // Write gate is disabled — result is NotExecuted (not DryRunOnly).
    LinkedSecondPassExecutorResult {
        status: LinkedSecondPassExecutorStatus::NotExecuted,
        mode: LinkedSecondPassExecutorMode::Disabled,
        message: format!(
            "Linked second-pass executor plan built ({total} total batch(es)). \
             Write gate is disabled — no linked record updates are attempted. \
             No old or new record IDs are present. \
             No Airtable changes made."
        ),
        batches,
        safety_snapshot,
        total_batch_count: total,
        first_field_batch_count,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn blocked_executor_batch() -> LinkedSecondPassExecutorBatch {
    LinkedSecondPassExecutorBatch {
        batch_index: 0,
        batch_id: "LSEX-BATCH-BLOCKED".to_string(),
        table_label: "—".to_string(),
        field_label: "—".to_string(),
        update_count: 0,
        mapping_coverage_count: 0,
        status: LinkedSecondPassExecutorBatchStatus::Blocked,
        note: "Safety prerequisites not satisfied. No linked second-pass batches can be planned."
            .to_string(),
    }
}

fn build_executor_batches(
    field_summaries: &[LinkedSecondPassFieldSummary],
    batch_size: usize,
) -> Vec<LinkedSecondPassExecutorBatch> {
    let mut batches = Vec::new();
    let mut idx = 0usize;
    let effective_batch_size = batch_size.max(1);

    for field in field_summaries {
        if field.record_count == 0 {
            continue;
        }
        let n_batches = field.record_count.div_ceil(effective_batch_size);
        for b in 0..n_batches {
            let offset = b * effective_batch_size;
            let count = (field.record_count - offset).min(effective_batch_size);
            let safe_field_slug = field
                .field_label
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .take(16)
                .collect::<String>();
            batches.push(LinkedSecondPassExecutorBatch {
                batch_index: idx,
                batch_id: format!("LSEX-B{idx:03}-{safe_field_slug}"),
                table_label: field.table_label.clone(),
                field_label: field.field_label.clone(),
                update_count: count,
                mapping_coverage_count: count,
                status: LinkedSecondPassExecutorBatchStatus::Pending,
                note: format!(
                    "Would apply remapped IDs for '{}' in '{}' — batch {} of {}. \
                     {} record(s). Write gate disabled — no network call made. \
                     No old or new record IDs present.",
                    field.field_label,
                    field.table_label,
                    b + 1,
                    n_batches,
                    count
                ),
            });
            idx += 1;
        }
    }

    if batches.is_empty() {
        batches.push(LinkedSecondPassExecutorBatch {
            batch_index: 0,
            batch_id: "LSEX-BATCH-EMPTY".to_string(),
            table_label: "—".to_string(),
            field_label: "—".to_string(),
            update_count: 0,
            mapping_coverage_count: 0,
            status: LinkedSecondPassExecutorBatchStatus::Skipped,
            note: "No linked field summaries with record_count > 0.".to_string(),
        });
    }

    batches
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn field_summary(table: &str, field: &str, count: usize) -> LinkedSecondPassFieldSummary {
        LinkedSecondPassFieldSummary {
            table_label: table.to_string(),
            field_label: field.to_string(),
            record_count: count,
            batch_count: count.div_ceil(LSEX_MAX_BATCH_SIZE),
            unresolved_link_count: 0,
        }
    }

    fn all_prereqs_request() -> LinkedSecondPassExecutorRequest {
        LinkedSecondPassExecutorRequest {
            mode: LinkedSecondPassExecutorMode::SandboxOnly,
            explicit_internal_linked_second_pass_requested: true,
            sandbox_verified: true,
            target_empty_verified: true,
            record_executor_safe: true,
            linked_second_pass_preview_ready: true,
            linked_second_pass_preview_status: LinkedSecondPassExecutionPreviewStatus::DryRunReady,
            mapping_checkpoint_preview_ready: true,
            sensitive_data_safe: true,
            live_write_readiness_satisfied: true,
            batch_size: 10,
            field_summaries: vec![
                field_summary("Projects", "Tasks", 20),
                field_summary("Tasks", "Owner", 5),
            ],
        }
    }

    // ── Blocked when prerequisites missing ────────────────────────────────────

    #[test]
    fn foundation_blocked_when_mode_disabled() {
        let mut req = all_prereqs_request();
        req.mode = LinkedSecondPassExecutorMode::Disabled;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-02"));
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_blocked_when_explicit_flag_not_set() {
        let mut req = all_prereqs_request();
        req.explicit_internal_linked_second_pass_requested = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-03"));
    }

    #[test]
    fn foundation_blocked_when_sandbox_not_verified() {
        let mut req = all_prereqs_request();
        req.sandbox_verified = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-04"));
    }

    #[test]
    fn foundation_blocked_when_target_not_empty() {
        let mut req = all_prereqs_request();
        req.target_empty_verified = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-05"));
    }

    #[test]
    fn foundation_blocked_when_record_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.record_executor_safe = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-06"));
    }

    #[test]
    fn foundation_blocked_when_linked_preview_not_ready() {
        let mut req = all_prereqs_request();
        req.linked_second_pass_preview_ready = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-07"));
    }

    #[test]
    fn foundation_blocked_when_linked_preview_status_blocked() {
        let mut req = all_prereqs_request();
        req.linked_second_pass_preview_status = LinkedSecondPassExecutionPreviewStatus::Blocked;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-07"));
    }

    #[test]
    fn foundation_blocked_when_mapping_checkpoint_not_ready() {
        let mut req = all_prereqs_request();
        req.mapping_checkpoint_preview_ready = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-08"));
    }

    #[test]
    fn foundation_blocked_when_sensitive_data_not_safe() {
        let mut req = all_prereqs_request();
        req.sensitive_data_safe = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-09"));
    }

    #[test]
    fn foundation_blocked_when_live_write_readiness_not_satisfied() {
        let mut req = all_prereqs_request();
        req.live_write_readiness_satisfied = false;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-PRE-10"));
    }

    #[test]
    fn foundation_blocked_when_batch_size_exceeds_max() {
        let mut req = all_prereqs_request();
        req.batch_size = 11;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEX-BATCH-SIZE"));
    }

    #[test]
    fn foundation_blocked_when_batch_size_is_zero() {
        let mut req = all_prereqs_request();
        req.batch_size = 0;
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::Blocked);
    }

    // ── NotExecuted when all prerequisites met ────────────────────────────────

    #[test]
    fn foundation_not_executed_when_all_prereqs_met() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::NotExecuted);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn foundation_write_gate_still_disabled_after_plan() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn foundation_safety_snapshot_write_gate_disabled_always_true() {
        let mut req = all_prereqs_request();
        req.mode = LinkedSecondPassExecutorMode::Disabled;
        let result = build_linked_second_pass_executor_plan(&req);
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── No production mode ────────────────────────────────────────────────────

    #[test]
    fn foundation_no_production_mode_exists() {
        let disabled = LinkedSecondPassExecutorMode::Disabled;
        let sandbox = LinkedSecondPassExecutorMode::SandboxOnly;
        assert_ne!(disabled, sandbox);
        let json = serde_json::to_string(&disabled).expect("serialize");
        assert!(!json.contains("production"));
        let json = serde_json::to_string(&sandbox).expect("serialize");
        assert!(!json.contains("production"));
    }

    // ── Batch ordering and content ────────────────────────────────────────────

    #[test]
    fn foundation_batches_built_in_not_executed_result() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::NotExecuted);
        assert!(!result.batches.is_empty());
        assert!(result.total_batch_count > 0);
    }

    #[test]
    fn foundation_batch_indices_are_sequential() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        for (i, batch) in result.batches.iter().enumerate() {
            assert_eq!(batch.batch_index, i, "batch_index must be sequential");
        }
    }

    #[test]
    fn foundation_batch_ordering_is_deterministic() {
        let r1 = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let r2 = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let ids1: Vec<_> = r1.batches.iter().map(|b| &b.batch_id).collect();
        let ids2: Vec<_> = r2.batches.iter().map(|b| &b.batch_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn foundation_batch_size_never_exceeds_max() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        for batch in &result.batches {
            assert!(
                batch.update_count <= LSEX_MAX_BATCH_SIZE || batch.update_count == 0,
                "batch {} update_count {} exceeds max {}",
                batch.batch_id,
                batch.update_count,
                LSEX_MAX_BATCH_SIZE
            );
        }
    }

    #[test]
    fn foundation_field_order_preserved() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        // Projects/Tasks (20 records, 2 batches) must come before Tasks/Owner (5 records, 1 batch)
        let projects_last = result
            .batches
            .iter()
            .filter(|b| b.field_label == "Tasks")
            .map(|b| b.batch_index)
            .max();
        let owner_first = result
            .batches
            .iter()
            .find(|b| b.field_label == "Owner")
            .map(|b| b.batch_index);
        if let (Some(last_p), Some(first_o)) = (projects_last, owner_first) {
            assert!(
                last_p < first_o,
                "field ordering must be preserved from field_summaries"
            );
        }
    }

    #[test]
    fn foundation_unresolved_optional_links_warning_safe() {
        // Unresolved links in preview are warning-safe — they don't block the executor
        // as long as the preview returned DryRunReady.
        let mut req = all_prereqs_request();
        req.field_summaries = vec![LinkedSecondPassFieldSummary {
            table_label: "Projects".to_string(),
            field_label: "Tasks".to_string(),
            record_count: 10,
            batch_count: 1,
            unresolved_link_count: 3,
        }];
        let result = build_linked_second_pass_executor_plan(&req);
        // Should be NotExecuted — unresolved links don't block when preview is DryRunReady
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::NotExecuted);
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn foundation_no_token_in_result() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn foundation_no_absolute_path_in_result() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn foundation_no_record_payload_in_result() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn foundation_no_succeeded_in_serialization() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn foundation_no_attachment_url_in_result() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn foundation_no_old_record_id_in_result() {
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn foundation_no_airtable_client_called() {
        // build_linked_second_pass_executor_plan accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::NotExecuted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_no_network_writes_in_blocked_state() {
        let mut req = all_prereqs_request();
        req.mode = LinkedSecondPassExecutorMode::Disabled;
        let result = build_linked_second_pass_executor_plan(&req);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn foundation_sandboxonly_still_blocked_while_write_gate_disabled() {
        // SandboxOnly mode alone is insufficient — write gate is always Disabled.
        // The result must be NotExecuted (all prereqs met) not DryRunOnly.
        let result = build_linked_second_pass_executor_plan(&all_prereqs_request());
        assert_ne!(result.status, LinkedSecondPassExecutorStatus::DryRunOnly);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::NotExecuted);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn foundation_empty_field_summaries_produces_skipped_batch() {
        let mut req = all_prereqs_request();
        req.field_summaries = vec![];
        let result = build_linked_second_pass_executor_plan(&req);
        assert_eq!(result.status, LinkedSecondPassExecutorStatus::NotExecuted);
        assert_eq!(result.batches.len(), 1);
        assert_eq!(
            result.batches[0].status,
            LinkedSecondPassExecutorBatchStatus::Skipped
        );
    }
}
