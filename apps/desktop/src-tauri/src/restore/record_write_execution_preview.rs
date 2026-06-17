use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the record write execution preview.
///
/// Safety invariants:
/// - `DryRunReady` does NOT enable live record writes.
/// - `writes_enabled` is always `false`.
/// - No Airtable API calls are made by this module.
/// - No token, full path, record payload, raw HTTP, or attachment URL appears
///   in any result field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteExecutionPreviewStatus {
    /// All safety prerequisites are present; a dry-run batch preview is available.
    /// Live record writes remain disabled and no restore execution is started.
    DryRunReady,
    /// At least one required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Status of a single preview batch.
///
/// Note: `succeeded` / `completed` / `executed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteExecutionPreviewBatchStatus {
    /// The batch is planned and would be executed if writes were enabled.
    Pending,
    /// The batch is blocked by a safety prerequisite.
    Blocked,
    /// The batch is skipped because a prerequisite failed.
    Skipped,
}

/// Execution mode for the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteExecutionPreviewMode {
    /// Dry-run only — no live execution path is reachable.
    DryRunOnly,
    /// Live record writes are blocked by product policy.
    LiveBlocked,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single ordered batch in the record write execution preview.
///
/// Safety properties:
/// - No raw record field values.
/// - No raw request/response body.
/// - No token.
/// - No absolute path.
/// - No attachment URL.
/// - Only safe counts and labels are included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteExecutionPreviewBatch {
    pub batch_index: usize,
    pub batch_id: String,
    /// Safe label describing the table. Not a live Airtable ID.
    pub table_label: String,
    /// Class of operation (e.g. "first-pass-create", "second-pass-linked-update").
    pub operation_class: String,
    pub status: RecordWriteExecutionPreviewBatchStatus,
    /// Number of records in this batch. Always <= batch_size.
    pub record_count: usize,
    /// Estimated number of API requests this batch would require.
    pub estimated_request_count: usize,
    pub note: String,
}

/// A point-in-time safety snapshot for the record write preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteSafetySnapshot {
    pub write_gate_disabled: bool,
    pub schema_preview_ready: bool,
    pub sandbox_flag_present: bool,
    pub target_empty_verified: bool,
    pub record_import_plan_ready: bool,
    pub record_write_request_plan_ready: bool,
    pub batch_size_safe: bool,
    pub rate_limit_backoff_safe: bool,
    pub checkpoint_durability_safe: bool,
    pub sensitive_data_safe: bool,
    pub attachment_phase_disabled: bool,
    pub final_validation_enforcement_present: bool,
    pub live_write_readiness_satisfied: bool,
}

/// Request for the record write execution preview.
///
/// No token field — token is not required or accepted.
/// No full path field — filename label only.
/// No raw record payloads — only counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteExecutionPreviewRequest {
    /// Filename label for the backup package. No directory component.
    pub package_filename: Option<String>,
    /// Whether the schema write execution preview returned DryRunReady.
    pub schema_preview_ready: Option<bool>,
    /// Whether the sandbox environment check has passed.
    pub sandbox_flag_present: Option<bool>,
    /// Whether the target empty verification has passed.
    pub target_empty_verified: Option<bool>,
    /// Whether the record import plan is ready (not blocked).
    pub record_import_plan_ready: Option<bool>,
    /// Whether the record write request plan is ready (not blocked).
    pub record_write_request_plan_ready: Option<bool>,
    /// Number of tables in the import plan. Used to build ordered batches.
    pub table_count: Option<usize>,
    /// Total first-pass (create) batches across all tables.
    pub total_first_pass_batches: Option<usize>,
    /// Total second-pass (linked update) batches across all tables.
    pub total_second_pass_batches: Option<usize>,
    /// Total record count across all tables.
    pub total_record_count: Option<usize>,
    /// Batch size used when splitting records into batches.
    /// Must be <= 10. Defaults to 10 if not provided.
    pub batch_size: Option<usize>,
    /// Whether the rate-limit/backoff policy result is safe.
    pub rate_limit_backoff_safe: Option<bool>,
    /// Whether the checkpoint durability policy result is safe.
    pub checkpoint_durability_safe: Option<bool>,
    /// Whether the sensitive data safety policy result is safe.
    pub sensitive_data_safe: Option<bool>,
    /// Whether the attachment phase disabled policy result shows phase is disabled.
    pub attachment_phase_disabled: Option<bool>,
    /// Whether the final validation enforcement policy result is present.
    pub final_validation_enforcement_present: Option<bool>,
    /// Whether the live-write readiness result is ready or warning (advisory only).
    pub live_write_readiness_satisfied: Option<bool>,
}

/// Result of the record write execution preview.
///
/// Safety invariants (enforced):
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No full filesystem path field.
/// - No raw record payload or field values.
/// - No raw HTTP request or response body.
/// - No attachment URL.
/// - Status is never `succeeded` or any completion state.
/// - `DryRunReady` does NOT enable live record writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteExecutionPreviewResult {
    pub status: RecordWriteExecutionPreviewStatus,
    pub mode: RecordWriteExecutionPreviewMode,
    pub message: String,
    pub batches: Vec<RecordWriteExecutionPreviewBatch>,
    pub safety_snapshot: RecordWriteSafetySnapshot,
    pub total_batch_count: usize,
    pub first_pass_batch_count: usize,
    pub second_pass_batch_count: usize,
    pub total_record_count: usize,
    pub batch_size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always false — live record writes are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const PREREQ_WRITE_GATE: &str = "RWEP-PRE-01";
const PREREQ_SCHEMA_PREVIEW: &str = "RWEP-PRE-02";
const PREREQ_SANDBOX: &str = "RWEP-PRE-03";
const PREREQ_TARGET_EMPTY: &str = "RWEP-PRE-04";
const PREREQ_RECORD_IMPORT_PLAN: &str = "RWEP-PRE-05";
const PREREQ_RECORD_WRITE_PLAN: &str = "RWEP-PRE-06";
const PREREQ_BATCH_SIZE: &str = "RWEP-PRE-07";
const PREREQ_RATE_LIMIT: &str = "RWEP-PRE-08";
const PREREQ_CHECKPOINT: &str = "RWEP-PRE-09";
const PREREQ_SENSITIVE_DATA: &str = "RWEP-PRE-10";
const PREREQ_ATTACHMENT_PHASE: &str = "RWEP-PRE-11";
const PREREQ_FINAL_VALIDATION: &str = "RWEP-PRE-12";
const PREREQ_LWR: &str = "RWEP-PRE-13";

const MAX_SAFE_BATCH_SIZE: usize = 10;

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds a record write execution preview from a request.
///
/// This function:
/// - Never calls any Airtable API endpoint.
/// - Never creates, updates, or deletes any record, table, or field.
/// - Never returns a token, full path, record payload, raw HTTP body, or
///   attachment URL.
/// - Always returns `writes_enabled: false`, `no_changes_made: true`,
///   `network_writes_attempted: false`.
/// - Always consults `evaluate_write_gate()` to confirm writes remain disabled.
/// - A `DryRunReady` result does NOT enable write execution.
pub fn preview_record_write_execution(
    request: &RecordWriteExecutionPreviewRequest,
) -> RecordWriteExecutionPreviewResult {
    // Always consult the write gate first.
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let schema_preview_ready = request.schema_preview_ready.unwrap_or(false);
    let sandbox_flag_present = request.sandbox_flag_present.unwrap_or(false);
    let target_empty_verified = request.target_empty_verified.unwrap_or(false);
    let record_import_plan_ready = request.record_import_plan_ready.unwrap_or(false);
    let record_write_request_plan_ready = request.record_write_request_plan_ready.unwrap_or(false);
    let batch_size = request.batch_size.unwrap_or(MAX_SAFE_BATCH_SIZE);
    let batch_size_safe = batch_size <= MAX_SAFE_BATCH_SIZE && batch_size > 0;
    let rate_limit_backoff_safe = request.rate_limit_backoff_safe.unwrap_or(false);
    let checkpoint_durability_safe = request.checkpoint_durability_safe.unwrap_or(false);
    let sensitive_data_safe = request.sensitive_data_safe.unwrap_or(false);
    let attachment_phase_disabled = request.attachment_phase_disabled.unwrap_or(false);
    let final_validation_enforcement_present = request
        .final_validation_enforcement_present
        .unwrap_or(false);
    let live_write_readiness_satisfied = request.live_write_readiness_satisfied.unwrap_or(false);

    let snapshot = RecordWriteSafetySnapshot {
        write_gate_disabled,
        schema_preview_ready,
        sandbox_flag_present,
        target_empty_verified,
        record_import_plan_ready,
        record_write_request_plan_ready,
        batch_size_safe,
        rate_limit_backoff_safe,
        checkpoint_durability_safe,
        sensitive_data_safe,
        attachment_phase_disabled,
        final_validation_enforcement_present,
        live_write_readiness_satisfied,
    };

    // Check prerequisites in order; first failure wins.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        Some(format!(
            "{PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Live record writes must not be attempted."
        ))
    } else if !schema_preview_ready {
        Some(format!(
            "{PREREQ_SCHEMA_PREVIEW}: Schema write execution preview has not returned \
             DryRunReady. Complete the schema preview before requesting a record preview."
        ))
    } else if !sandbox_flag_present {
        Some(format!(
            "{PREREQ_SANDBOX}: Sandbox environment check has not passed."
        ))
    } else if !target_empty_verified {
        Some(format!(
            "{PREREQ_TARGET_EMPTY}: Target empty verification has not passed."
        ))
    } else if !record_import_plan_ready {
        Some(format!(
            "{PREREQ_RECORD_IMPORT_PLAN}: Record import plan is not ready. \
             Complete the record import plan before requesting a preview."
        ))
    } else if !record_write_request_plan_ready {
        Some(format!(
            "{PREREQ_RECORD_WRITE_PLAN}: Record write request plan is not ready. \
             Complete the record write request plan before requesting a preview."
        ))
    } else if !batch_size_safe {
        Some(format!(
            "{PREREQ_BATCH_SIZE}: Batch size {batch_size} exceeds the safe maximum \
             of {MAX_SAFE_BATCH_SIZE}. Reduce batch size before requesting a preview."
        ))
    } else if !rate_limit_backoff_safe {
        Some(format!(
            "{PREREQ_RATE_LIMIT}: Rate-limit/backoff policy is not safe."
        ))
    } else if !checkpoint_durability_safe {
        Some(format!(
            "{PREREQ_CHECKPOINT}: Checkpoint durability policy is not safe."
        ))
    } else if !sensitive_data_safe {
        Some(format!(
            "{PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied."
        ))
    } else if !attachment_phase_disabled {
        Some(format!(
            "{PREREQ_ATTACHMENT_PHASE}: Attachment phase is not disabled or metadata-only."
        ))
    } else if !final_validation_enforcement_present {
        Some(format!(
            "{PREREQ_FINAL_VALIDATION}: Final validation enforcement policy \
             has not been verified."
        ))
    } else if !live_write_readiness_satisfied {
        Some(format!(
            "{PREREQ_LWR}: Live write readiness policy is not satisfied. \
             All 17 required safety gates must be declared."
        ))
    } else {
        None
    };

    if let Some(ref reason) = blocked_reason {
        return RecordWriteExecutionPreviewResult {
            status: RecordWriteExecutionPreviewStatus::Blocked,
            mode: RecordWriteExecutionPreviewMode::LiveBlocked,
            message: format!(
                "Record write execution preview is blocked. {reason} \
                 Live record writes remain disabled."
            ),
            batches: blocked_batches(),
            safety_snapshot: snapshot,
            total_batch_count: 0,
            first_pass_batch_count: 0,
            second_pass_batch_count: 0,
            total_record_count: 0,
            batch_size,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the dry-run batch preview.
    let table_count = request.table_count.unwrap_or(0);
    let total_first_pass = request.total_first_pass_batches.unwrap_or(0);
    let total_second_pass = request.total_second_pass_batches.unwrap_or(0);
    let total_records = request.total_record_count.unwrap_or(0);

    let batches = build_preview_batches(
        table_count,
        total_first_pass,
        total_second_pass,
        batch_size,
        total_records,
    );
    let total = batches.len();

    RecordWriteExecutionPreviewResult {
        status: RecordWriteExecutionPreviewStatus::DryRunReady,
        mode: RecordWriteExecutionPreviewMode::DryRunOnly,
        message: format!(
            "Record write execution preview is ready (dry-run only). \
             {} first-pass create batch(es), {} second-pass linked-update batch(es), \
             {} total record(s), batch size {}. \
             Live record writes remain disabled. \
             This preview does not start any restore execution.",
            total_first_pass, total_second_pass, total_records, batch_size
        ),
        batches,
        safety_snapshot: snapshot,
        total_batch_count: total,
        first_pass_batch_count: total_first_pass,
        second_pass_batch_count: total_second_pass,
        total_record_count: total_records,
        batch_size,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn blocked_batches() -> Vec<RecordWriteExecutionPreviewBatch> {
    vec![RecordWriteExecutionPreviewBatch {
        batch_index: 0,
        batch_id: "RWEP-BATCH-BLOCKED".to_string(),
        table_label: "—".to_string(),
        operation_class: "blocked".to_string(),
        status: RecordWriteExecutionPreviewBatchStatus::Blocked,
        record_count: 0,
        estimated_request_count: 0,
        note: "Safety prerequisites not satisfied. No batches can be previewed.".to_string(),
    }]
}

fn build_preview_batches(
    table_count: usize,
    total_first_pass: usize,
    total_second_pass: usize,
    batch_size: usize,
    total_records: usize,
) -> Vec<RecordWriteExecutionPreviewBatch> {
    let mut batches = Vec::new();
    let mut idx = 0usize;

    // Distribute first-pass create batches evenly across tables (or one block if no tables known)
    let tables = if table_count > 0 { table_count } else { 1 };

    // Phase 1 — first-pass create batches
    let fp_per_table = if tables > 0 {
        (total_first_pass + tables - 1) / tables
    } else {
        0
    };
    let records_per_table = if tables > 0 {
        (total_records + tables - 1) / tables
    } else {
        0
    };

    for t in 0..tables {
        let table_batches = if t < tables - 1 {
            fp_per_table
        } else {
            total_first_pass.saturating_sub(fp_per_table * (tables - 1))
        };
        for b in 0..table_batches {
            let rec_count = if b < table_batches - 1 {
                batch_size
            } else {
                records_per_table.saturating_sub(batch_size * b).max(1)
            };
            batches.push(RecordWriteExecutionPreviewBatch {
                batch_index: idx,
                batch_id: format!("RWEP-BATCH-FP-T{t:02}-B{b:02}"),
                table_label: format!("Table {} (first-pass)", t + 1),
                operation_class: "first-pass-create".to_string(),
                status: RecordWriteExecutionPreviewBatchStatus::Pending,
                record_count: rec_count.min(batch_size),
                estimated_request_count: 1,
                note: format!(
                    "Would call Airtable create-records endpoint for table {} \
                     batch {}. Disabled — no network call made.",
                    t + 1,
                    b + 1
                ),
            });
            idx += 1;
        }
    }

    // Phase 2 — second-pass linked update batches
    let sp_per_table = if tables > 0 {
        (total_second_pass + tables - 1) / tables
    } else {
        0
    };
    for t in 0..tables {
        let table_batches = if t < tables - 1 {
            sp_per_table
        } else {
            total_second_pass.saturating_sub(sp_per_table * (tables - 1))
        };
        for b in 0..table_batches {
            batches.push(RecordWriteExecutionPreviewBatch {
                batch_index: idx,
                batch_id: format!("RWEP-BATCH-SP-T{t:02}-B{b:02}"),
                table_label: format!("Table {} (second-pass)", t + 1),
                operation_class: "second-pass-linked-update".to_string(),
                status: RecordWriteExecutionPreviewBatchStatus::Pending,
                record_count: batch_size,
                estimated_request_count: 1,
                note: format!(
                    "Would call Airtable update-records endpoint for table {} \
                     linked field batch {}. ID mapping unavailable until execution. \
                     Disabled — no network call made.",
                    t + 1,
                    b + 1
                ),
            });
            idx += 1;
        }
    }

    // If no batches were generated (zero tables/counts), still show a placeholder
    if batches.is_empty() {
        batches.push(RecordWriteExecutionPreviewBatch {
            batch_index: 0,
            batch_id: "RWEP-BATCH-EMPTY".to_string(),
            table_label: "—".to_string(),
            operation_class: "no-operations".to_string(),
            status: RecordWriteExecutionPreviewBatchStatus::Skipped,
            record_count: 0,
            estimated_request_count: 0,
            note: "No records or tables declared. Nothing to preview.".to_string(),
        });
    }

    batches
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_request() -> RecordWriteExecutionPreviewRequest {
        RecordWriteExecutionPreviewRequest {
            package_filename: Some("test-backup.airbridge".to_string()),
            schema_preview_ready: Some(true),
            sandbox_flag_present: Some(true),
            target_empty_verified: Some(true),
            record_import_plan_ready: Some(true),
            record_write_request_plan_ready: Some(true),
            table_count: Some(2),
            total_first_pass_batches: Some(4),
            total_second_pass_batches: Some(2),
            total_record_count: Some(35),
            batch_size: Some(10),
            rate_limit_backoff_safe: Some(true),
            checkpoint_durability_safe: Some(true),
            sensitive_data_safe: Some(true),
            attachment_phase_disabled: Some(true),
            final_validation_enforcement_present: Some(true),
            live_write_readiness_satisfied: Some(true),
        }
    }

    fn missing_request() -> RecordWriteExecutionPreviewRequest {
        RecordWriteExecutionPreviewRequest {
            package_filename: None,
            schema_preview_ready: None,
            sandbox_flag_present: None,
            target_empty_verified: None,
            record_import_plan_ready: None,
            record_write_request_plan_ready: None,
            table_count: None,
            total_first_pass_batches: None,
            total_second_pass_batches: None,
            total_record_count: None,
            batch_size: None,
            rate_limit_backoff_safe: None,
            checkpoint_durability_safe: None,
            sensitive_data_safe: None,
            attachment_phase_disabled: None,
            final_validation_enforcement_present: None,
            live_write_readiness_satisfied: None,
        }
    }

    // ── Basic safety invariants ────────────────────────────────────────────────

    #[test]
    fn writes_enabled_always_false_safe_request() {
        let result = preview_record_write_execution(&safe_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_safe_request() {
        let result = preview_record_write_execution(&safe_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_safe_request() {
        let result = preview_record_write_execution(&safe_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false_missing_request() {
        let result = preview_record_write_execution(&missing_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_missing_request() {
        let result = preview_record_write_execution(&missing_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_missing_request() {
        let result = preview_record_write_execution(&missing_request());
        assert!(!result.network_writes_attempted);
    }

    // ── Blocked when prerequisites missing ────────────────────────────────────

    #[test]
    fn missing_all_prerequisites_returns_blocked() {
        let result = preview_record_write_execution(&missing_request());
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn missing_record_import_plan_returns_blocked() {
        let mut req = safe_request();
        req.record_import_plan_ready = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn missing_record_write_request_plan_returns_blocked() {
        let mut req = safe_request();
        req.record_write_request_plan_ready = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn schema_preview_missing_returns_blocked() {
        let mut req = safe_request();
        req.schema_preview_ready = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn sandbox_flag_missing_returns_blocked() {
        let mut req = safe_request();
        req.sandbox_flag_present = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn target_empty_missing_returns_blocked() {
        let mut req = safe_request();
        req.target_empty_verified = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn batch_size_too_large_returns_blocked() {
        let mut req = safe_request();
        req.batch_size = Some(11);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn batch_size_zero_returns_blocked() {
        let mut req = safe_request();
        req.batch_size = Some(0);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn rate_limit_backoff_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.rate_limit_backoff_safe = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn checkpoint_durability_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.checkpoint_durability_safe = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn sensitive_data_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.sensitive_data_safe = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn attachment_phase_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.attachment_phase_disabled = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn final_validation_enforcement_missing_returns_blocked() {
        let mut req = safe_request();
        req.final_validation_enforcement_present = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn live_write_readiness_missing_returns_blocked() {
        let mut req = safe_request();
        req.live_write_readiness_satisfied = Some(false);
        let result = preview_record_write_execution(&req);
        assert_eq!(result.status, RecordWriteExecutionPreviewStatus::Blocked);
    }

    // ── DryRunReady for safe request ───────────────────────────────────────────

    #[test]
    fn safe_request_returns_dry_run_ready() {
        let result = preview_record_write_execution(&safe_request());
        assert_eq!(
            result.status,
            RecordWriteExecutionPreviewStatus::DryRunReady
        );
    }

    #[test]
    fn dry_run_ready_mode_is_dry_run_only() {
        let result = preview_record_write_execution(&safe_request());
        assert_eq!(result.mode, RecordWriteExecutionPreviewMode::DryRunOnly);
    }

    #[test]
    fn dry_run_ready_has_no_blocked_reason() {
        let result = preview_record_write_execution(&safe_request());
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn dry_run_ready_write_gate_disabled_in_snapshot() {
        let result = preview_record_write_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn dry_run_ready_counts_match_request() {
        let result = preview_record_write_execution(&safe_request());
        assert_eq!(result.first_pass_batch_count, 4);
        assert_eq!(result.second_pass_batch_count, 2);
        assert_eq!(result.total_record_count, 35);
        assert_eq!(result.batch_size, 10);
    }

    // ── Batch ordering ─────────────────────────────────────────────────────────

    #[test]
    fn batch_ordering_is_deterministic() {
        let r1 = preview_record_write_execution(&safe_request());
        let r2 = preview_record_write_execution(&safe_request());
        let ids1: Vec<_> = r1.batches.iter().map(|b| &b.batch_id).collect();
        let ids2: Vec<_> = r2.batches.iter().map(|b| &b.batch_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn batch_indices_are_sequential() {
        let result = preview_record_write_execution(&safe_request());
        for (i, batch) in result.batches.iter().enumerate() {
            assert_eq!(batch.batch_index, i, "batch_index must be sequential");
        }
    }

    #[test]
    fn first_pass_batches_come_before_second_pass() {
        let result = preview_record_write_execution(&safe_request());
        let fp_last = result
            .batches
            .iter()
            .filter(|b| b.operation_class == "first-pass-create")
            .map(|b| b.batch_index)
            .max()
            .unwrap_or(0);
        let sp_first = result
            .batches
            .iter()
            .find(|b| b.operation_class == "second-pass-linked-update")
            .map(|b| b.batch_index)
            .unwrap_or(usize::MAX);
        assert!(
            fp_last < sp_first,
            "first-pass batches must precede second-pass batches"
        );
    }

    #[test]
    fn total_batch_count_equals_batches_len() {
        let result = preview_record_write_execution(&safe_request());
        assert_eq!(result.total_batch_count, result.batches.len());
    }

    #[test]
    fn batch_record_count_does_not_exceed_batch_size() {
        let result = preview_record_write_execution(&safe_request());
        for batch in &result.batches {
            assert!(
                batch.record_count <= result.batch_size || batch.record_count == 0,
                "batch {} record_count {} exceeds batch_size {}",
                batch.batch_id,
                batch.record_count,
                result.batch_size
            );
        }
    }

    // ── Safety serialization checks ────────────────────────────────────────────

    #[test]
    fn no_token_in_dry_run_ready_serialization() {
        let result = preview_record_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn no_absolute_path_in_serialization() {
        let result = preview_record_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_succeeded_in_serialization() {
        let result = preview_record_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn no_attachment_url_in_serialization() {
        let result = preview_record_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn no_record_payload_in_serialization() {
        let result = preview_record_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn dry_run_message_does_not_imply_restore_complete() {
        let result = preview_record_write_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(!lower.contains("restore complete"));
        assert!(!lower.contains("restore succeeded"));
    }

    #[test]
    fn dry_run_message_states_writes_remain_disabled() {
        let result = preview_record_write_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("disabled"));
    }

    #[test]
    fn dry_run_message_states_no_restore_execution() {
        let result = preview_record_write_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("does not start any restore execution"));
    }

    // ── Write gate not bypassed ────────────────────────────────────────────────

    #[test]
    fn write_gate_not_bypassed_by_preview() {
        let gate_before = evaluate_write_gate();
        let _result = preview_record_write_execution(&safe_request());
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
    fn write_gate_snapshot_always_disabled() {
        let result = preview_record_write_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── Blocked mode ───────────────────────────────────────────────────────────

    #[test]
    fn blocked_result_has_blocked_reason() {
        let result = preview_record_write_execution(&missing_request());
        assert!(result.blocked_reason.is_some());
    }

    #[test]
    fn blocked_result_mode_is_live_blocked() {
        let result = preview_record_write_execution(&missing_request());
        assert_eq!(result.mode, RecordWriteExecutionPreviewMode::LiveBlocked);
    }

    #[test]
    fn blocked_result_has_blocked_batch() {
        let result = preview_record_write_execution(&missing_request());
        assert!(!result.batches.is_empty());
        assert_eq!(
            result.batches[0].status,
            RecordWriteExecutionPreviewBatchStatus::Blocked
        );
    }

    #[test]
    fn blocked_message_mentions_disabled() {
        let result = preview_record_write_execution(&missing_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("disabled"));
    }

    // ── No success state ───────────────────────────────────────────────────────

    #[test]
    fn no_success_state_introduced() {
        let result = preview_record_write_execution(&safe_request());
        assert!(!result.writes_enabled);
        assert!(result
            .message
            .to_lowercase()
            .contains("does not start any restore execution"));
    }
}
