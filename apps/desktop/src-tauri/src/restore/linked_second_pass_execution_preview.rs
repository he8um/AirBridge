use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the linked second-pass execution preview.
///
/// Safety invariants:
/// - `DryRunReady` does NOT enable live linked record updates.
/// - `writes_enabled` is always `false`.
/// - No Airtable API calls are made by this module.
/// - No token, full path, record payload, raw HTTP, old/new record IDs,
///   or attachment URL appears in any result field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedSecondPassExecutionPreviewStatus {
    /// All safety prerequisites are present; a dry-run second-pass preview is
    /// available. Live linked record updates remain disabled and no restore
    /// execution is started.
    DryRunReady,
    /// At least one required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Status of a single preview batch.
///
/// Note: `succeeded` / `completed` / `executed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedSecondPassPreviewBatchStatus {
    /// The batch is planned and would execute if writes were enabled.
    Pending,
    /// The batch is blocked by a safety prerequisite.
    Blocked,
    /// The batch is skipped because a prerequisite failed.
    Skipped,
}

/// Execution mode for the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedSecondPassPreviewMode {
    /// Dry-run only — no live execution path is reachable.
    DryRunOnly,
    /// Live linked record updates are blocked by product policy.
    LiveBlocked,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single ordered batch in the linked second-pass execution preview.
///
/// Safety properties:
/// - No old or new record IDs.
/// - No raw record field values.
/// - No raw request/response body.
/// - No token.
/// - No absolute path.
/// - No attachment URL.
/// - Only safe counts, labels, and phase identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassPreviewBatch {
    pub batch_index: usize,
    pub batch_id: String,
    /// Safe label describing the table. Not a live Airtable ID.
    pub table_label: String,
    /// Safe label describing the linked field. Not a live Airtable field ID.
    pub field_label: String,
    pub status: LinkedSecondPassPreviewBatchStatus,
    /// Number of records in this batch. Always <= batch_size.
    pub update_count: usize,
    /// Number of mapping entries required for this batch (safe count only).
    pub mapping_coverage_count: usize,
    /// Number of links that could not be resolved — IDs unavailable until execution.
    pub unresolved_link_count: usize,
    pub note: String,
}

/// Summary of the ID mapping coverage for the second pass.
///
/// Safety properties:
/// - No raw record IDs (old or new).
/// - No field values.
/// - Only safe counts and labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassMappingSummary {
    /// Total records that require linked-field updates in the second pass.
    pub total_update_count: usize,
    /// Total tables that have linked fields requiring a second pass.
    pub tables_with_linked_fields: usize,
    /// Total distinct linked fields across all tables.
    pub total_linked_fields: usize,
    /// Total second-pass batches.
    pub total_batch_count: usize,
    /// Whether the full ID mapping is available before the second pass begins.
    pub mapping_complete_before_second_pass: bool,
    /// Total unresolved-link count (links whose target IDs are unknown until execution).
    pub unresolved_link_count: usize,
    pub note: String,
}

/// Summary of linked field coverage in the second-pass preview.
///
/// Safety properties:
/// - No raw field IDs.
/// - No record payloads.
/// - Only safe counts and labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassFieldSummary {
    /// Safe label of the table that owns this linked field.
    pub table_label: String,
    /// Safe label of the linked field.
    pub field_label: String,
    /// Number of records in this table that require second-pass updates.
    pub record_count: usize,
    /// Number of batches for this table/field pair.
    pub batch_count: usize,
    /// Number of unresolved links in this field's second pass.
    pub unresolved_link_count: usize,
}

/// Point-in-time safety snapshot for the linked second-pass preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassSafetySnapshot {
    pub write_gate_disabled: bool,
    pub record_write_preview_ready: bool,
    pub mapping_checkpoint_preview_ready: bool,
    pub write_phase_ordering_safe: bool,
    pub checkpoint_durability_safe: bool,
    pub sensitive_data_safe: bool,
    pub final_validation_enforcement_present: bool,
    pub live_write_readiness_satisfied: bool,
}

/// Request for the linked second-pass execution preview.
///
/// No token field — token is not required or accepted.
/// No full path field — filename label only.
/// No raw record payloads — only counts and flags.
/// No old or new record IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassExecutionPreviewRequest {
    /// Filename label for the backup package. No directory component.
    pub package_filename: Option<String>,
    /// Whether the record write execution preview returned DryRunReady.
    pub record_write_preview_ready: Option<bool>,
    /// Whether the mapping/checkpoint execution preview returned DryRunReady.
    pub mapping_checkpoint_preview_ready: Option<bool>,
    /// Total second-pass (linked update) batch count.
    pub second_pass_batch_count: Option<usize>,
    /// Total records requiring linked-field updates.
    pub total_update_count: Option<usize>,
    /// Number of tables that have linked fields.
    pub tables_with_linked_fields: Option<usize>,
    /// Total distinct linked fields across all tables.
    pub total_linked_fields: Option<usize>,
    /// Batch size used when splitting records into batches. Must be <= 10.
    pub batch_size: Option<usize>,
    /// Per-field summaries (table_label, field_label, record_count, batch_count,
    /// unresolved_link_count). No raw IDs.
    pub field_summaries: Option<Vec<LinkedSecondPassFieldSummary>>,
    /// Whether the write phase ordering policy result is safe.
    pub write_phase_ordering_safe: Option<bool>,
    /// Whether the checkpoint durability policy result is safe.
    pub checkpoint_durability_safe: Option<bool>,
    /// Whether the sensitive data safety policy result is safe.
    pub sensitive_data_safe: Option<bool>,
    /// Whether the final validation enforcement policy result is present.
    pub final_validation_enforcement_present: Option<bool>,
    /// Whether the live-write readiness result is ready or warning (advisory only).
    pub live_write_readiness_satisfied: Option<bool>,
}

/// Result of the linked second-pass execution preview.
///
/// Safety invariants (enforced):
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No full filesystem path field.
/// - No old or new record IDs.
/// - No raw record payload or field values.
/// - No raw HTTP request or response body.
/// - No attachment URL.
/// - Status is never `succeeded` or any completion state.
/// - `DryRunReady` does NOT enable live linked record updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedSecondPassExecutionPreviewResult {
    pub status: LinkedSecondPassExecutionPreviewStatus,
    pub mode: LinkedSecondPassPreviewMode,
    pub message: String,
    pub batches: Vec<LinkedSecondPassPreviewBatch>,
    pub mapping_summary: LinkedSecondPassMappingSummary,
    pub field_summaries: Vec<LinkedSecondPassFieldSummary>,
    pub safety_snapshot: LinkedSecondPassSafetySnapshot,
    pub total_batch_count: usize,
    pub batch_size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always false — live linked record updates are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const PREREQ_WRITE_GATE: &str = "LSEP-PRE-01";
const PREREQ_RECORD_WRITE_PREVIEW: &str = "LSEP-PRE-02";
const PREREQ_MAPPING_CHECKPOINT_PREVIEW: &str = "LSEP-PRE-03";
const PREREQ_WRITE_PHASE_ORDERING: &str = "LSEP-PRE-04";
const PREREQ_CHECKPOINT_DURABILITY: &str = "LSEP-PRE-05";
const PREREQ_SENSITIVE_DATA: &str = "LSEP-PRE-06";
const PREREQ_FINAL_VALIDATION: &str = "LSEP-PRE-07";
const PREREQ_LWR: &str = "LSEP-PRE-08";

const MAX_SAFE_BATCH_SIZE: usize = 10;

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds a linked second-pass execution preview from a request.
///
/// This function:
/// - Never calls any Airtable API endpoint.
/// - Never creates, updates, or deletes any record, table, or field.
/// - Never writes a checkpoint file (real, temp, or mock).
/// - Never returns a token, full path, old/new record ID, record payload,
///   raw HTTP body, or attachment URL.
/// - Always returns `writes_enabled: false`, `no_changes_made: true`,
///   `network_writes_attempted: false`.
/// - Always consults `evaluate_write_gate()` to confirm writes remain disabled.
/// - A `DryRunReady` result does NOT enable live linked record updates.
pub fn preview_linked_second_pass_execution(
    request: &LinkedSecondPassExecutionPreviewRequest,
) -> LinkedSecondPassExecutionPreviewResult {
    // Always consult the write gate first.
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let record_write_preview_ready = request.record_write_preview_ready.unwrap_or(false);
    let mapping_checkpoint_preview_ready =
        request.mapping_checkpoint_preview_ready.unwrap_or(false);
    let write_phase_ordering_safe = request.write_phase_ordering_safe.unwrap_or(false);
    let checkpoint_durability_safe = request.checkpoint_durability_safe.unwrap_or(false);
    let sensitive_data_safe = request.sensitive_data_safe.unwrap_or(false);
    let final_validation_enforcement_present = request
        .final_validation_enforcement_present
        .unwrap_or(false);
    let live_write_readiness_satisfied = request.live_write_readiness_satisfied.unwrap_or(false);

    let batch_size = request
        .batch_size
        .unwrap_or(MAX_SAFE_BATCH_SIZE)
        .min(MAX_SAFE_BATCH_SIZE);

    let snapshot = LinkedSecondPassSafetySnapshot {
        write_gate_disabled,
        record_write_preview_ready,
        mapping_checkpoint_preview_ready,
        write_phase_ordering_safe,
        checkpoint_durability_safe,
        sensitive_data_safe,
        final_validation_enforcement_present,
        live_write_readiness_satisfied,
    };

    // Validate batch size before other prerequisites so an oversized batch is
    // caught even when other prerequisites are missing.
    let batch_size_safe = request
        .batch_size
        .map(|s| s <= MAX_SAFE_BATCH_SIZE)
        .unwrap_or(true);

    // Check prerequisites in order; first failure wins.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        Some(format!(
            "{PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Live linked record updates must not be attempted."
        ))
    } else if !record_write_preview_ready {
        Some(format!(
            "{PREREQ_RECORD_WRITE_PREVIEW}: Record write execution preview has not returned \
             DryRunReady. Complete the record write preview before requesting a \
             linked second-pass preview."
        ))
    } else if !mapping_checkpoint_preview_ready {
        Some(format!(
            "{PREREQ_MAPPING_CHECKPOINT_PREVIEW}: Mapping/checkpoint execution preview has not \
             returned DryRunReady. Complete the mapping/checkpoint preview before requesting a \
             linked second-pass preview."
        ))
    } else if !write_phase_ordering_safe {
        Some(format!(
            "{PREREQ_WRITE_PHASE_ORDERING}: Write phase ordering policy is not safe. \
             The second pass must follow the first-pass record create phase."
        ))
    } else if !checkpoint_durability_safe {
        Some(format!(
            "{PREREQ_CHECKPOINT_DURABILITY}: Checkpoint durability policy is not safe. \
             A compliant checkpoint plan must be declared."
        ))
    } else if !sensitive_data_safe {
        Some(format!(
            "{PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied. \
             All 10 exposure surfaces must have redaction coverage."
        ))
    } else if !final_validation_enforcement_present {
        Some(format!(
            "{PREREQ_FINAL_VALIDATION}: Final validation enforcement policy \
             has not been verified. All three completion guards must be declared."
        ))
    } else if !live_write_readiness_satisfied {
        Some(format!(
            "{PREREQ_LWR}: Live write readiness policy is not satisfied. \
             All 17 required safety gates must be declared."
        ))
    } else if !batch_size_safe {
        Some(format!(
            "LSEP-BATCH: Batch size exceeds the maximum safe value of {MAX_SAFE_BATCH_SIZE}. \
             Reduce batch size to continue."
        ))
    } else {
        None
    };

    let empty_mapping_summary = LinkedSecondPassMappingSummary {
        total_update_count: 0,
        tables_with_linked_fields: 0,
        total_linked_fields: 0,
        total_batch_count: 0,
        mapping_complete_before_second_pass: false,
        unresolved_link_count: 0,
        note: "Linked second-pass preview unavailable — prerequisites not satisfied.".to_string(),
    };

    if let Some(ref reason) = blocked_reason {
        let blocked_batch = LinkedSecondPassPreviewBatch {
            batch_index: 0,
            batch_id: "LSEP-BLOCKED".to_string(),
            table_label: "blocked".to_string(),
            field_label: "blocked".to_string(),
            status: LinkedSecondPassPreviewBatchStatus::Blocked,
            update_count: 0,
            mapping_coverage_count: 0,
            unresolved_link_count: 0,
            note: "Safety prerequisites not satisfied. \
                   No linked second-pass batches can be previewed."
                .to_string(),
        };
        return LinkedSecondPassExecutionPreviewResult {
            status: LinkedSecondPassExecutionPreviewStatus::Blocked,
            mode: LinkedSecondPassPreviewMode::LiveBlocked,
            message: format!(
                "Linked second-pass execution preview is blocked. {reason} \
                 Live linked record updates remain disabled."
            ),
            batches: vec![blocked_batch],
            mapping_summary: empty_mapping_summary,
            field_summaries: vec![],
            safety_snapshot: snapshot,
            total_batch_count: 0,
            batch_size,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the dry-run preview.
    let second_pass_batch_count = request.second_pass_batch_count.unwrap_or(0);
    let total_update_count = request.total_update_count.unwrap_or(0);
    let tables_with_linked_fields = request.tables_with_linked_fields.unwrap_or(0);
    let total_linked_fields = request.total_linked_fields.unwrap_or(0);
    let field_summaries = request.field_summaries.clone().unwrap_or_default();

    let batches = build_preview_batches(&field_summaries, batch_size);
    let actual_batch_count = batches.len();

    let total_unresolved: usize = field_summaries
        .iter()
        .map(|f| f.unresolved_link_count)
        .sum();

    let mapping_summary = LinkedSecondPassMappingSummary {
        total_update_count,
        tables_with_linked_fields,
        total_linked_fields,
        total_batch_count: actual_batch_count,
        mapping_complete_before_second_pass: true,
        unresolved_link_count: total_unresolved,
        note: format!(
            "Would apply old-to-new ID mapping across {} linked field(s) in {} table(s). \
             {} record(s) require second-pass updates across {} batch(es). \
             {} unresolved link(s) would be skipped at execution time — \
             target IDs are unavailable until first-pass creation completes. \
             No raw record IDs present in this preview.",
            total_linked_fields,
            tables_with_linked_fields,
            total_update_count,
            actual_batch_count,
            total_unresolved
        ),
    };

    let unresolved_note = if total_unresolved > 0 {
        format!(
            " {total_unresolved} unresolved link(s) would be skipped — \
             target record IDs are unavailable until execution."
        )
    } else {
        " No unresolved links detected.".to_string()
    };

    LinkedSecondPassExecutionPreviewResult {
        status: LinkedSecondPassExecutionPreviewStatus::DryRunReady,
        mode: LinkedSecondPassPreviewMode::DryRunOnly,
        message: format!(
            "Linked second-pass execution preview is ready (dry-run only). \
             {} linked field(s) across {} table(s), {} batch(es), \
             {} record(s) to update.{unresolved_note} \
             Live linked record updates remain disabled. \
             This preview does not start any restore execution. \
             No checkpoint files are written. \
             No record IDs are present in this preview.",
            total_linked_fields, tables_with_linked_fields, actual_batch_count, total_update_count
        ),
        batches,
        mapping_summary,
        field_summaries,
        safety_snapshot: snapshot,
        total_batch_count: second_pass_batch_count,
        batch_size,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_preview_batches(
    field_summaries: &[LinkedSecondPassFieldSummary],
    batch_size: usize,
) -> Vec<LinkedSecondPassPreviewBatch> {
    let mut batches = Vec::new();
    let mut batch_index = 0usize;

    for field in field_summaries {
        if field.record_count == 0 {
            continue;
        }
        let effective_batch_size = if batch_size == 0 { 1 } else { batch_size };
        let n_batches = (field.record_count + effective_batch_size - 1) / effective_batch_size;
        for b in 0..n_batches {
            let offset = b * effective_batch_size;
            let count = (field.record_count - offset).min(effective_batch_size);
            let batch_id = format!(
                "LSEP-B{:03}-{}",
                batch_index,
                field
                    .field_label
                    .to_lowercase()
                    .replace(' ', "-")
                    .chars()
                    .take(20)
                    .collect::<String>()
            );
            batches.push(LinkedSecondPassPreviewBatch {
                batch_index,
                batch_id,
                table_label: field.table_label.clone(),
                field_label: field.field_label.clone(),
                status: LinkedSecondPassPreviewBatchStatus::Pending,
                update_count: count,
                mapping_coverage_count: count,
                unresolved_link_count: 0,
                note: format!(
                    "Would apply remapped IDs for '{}' in '{}' — batch {} of {}. \
                     {} record(s). No raw record IDs present in this preview.",
                    field.field_label,
                    field.table_label,
                    b + 1,
                    n_batches,
                    count
                ),
            });
            batch_index += 1;
        }
    }

    batches
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_request() -> LinkedSecondPassExecutionPreviewRequest {
        LinkedSecondPassExecutionPreviewRequest {
            package_filename: Some("test-backup.airbridge".to_string()),
            record_write_preview_ready: Some(true),
            mapping_checkpoint_preview_ready: Some(true),
            second_pass_batch_count: Some(2),
            total_update_count: Some(20),
            tables_with_linked_fields: Some(2),
            total_linked_fields: Some(3),
            batch_size: Some(10),
            field_summaries: Some(vec![
                LinkedSecondPassFieldSummary {
                    table_label: "Projects".to_string(),
                    field_label: "Tasks".to_string(),
                    record_count: 15,
                    batch_count: 2,
                    unresolved_link_count: 0,
                },
                LinkedSecondPassFieldSummary {
                    table_label: "Tasks".to_string(),
                    field_label: "Owner".to_string(),
                    record_count: 5,
                    batch_count: 1,
                    unresolved_link_count: 0,
                },
            ]),
            write_phase_ordering_safe: Some(true),
            checkpoint_durability_safe: Some(true),
            sensitive_data_safe: Some(true),
            final_validation_enforcement_present: Some(true),
            live_write_readiness_satisfied: Some(true),
        }
    }

    fn blocked_request() -> LinkedSecondPassExecutionPreviewRequest {
        LinkedSecondPassExecutionPreviewRequest {
            package_filename: None,
            record_write_preview_ready: None,
            mapping_checkpoint_preview_ready: None,
            second_pass_batch_count: None,
            total_update_count: None,
            tables_with_linked_fields: None,
            total_linked_fields: None,
            batch_size: None,
            field_summaries: None,
            write_phase_ordering_safe: None,
            checkpoint_durability_safe: None,
            sensitive_data_safe: None,
            final_validation_enforcement_present: None,
            live_write_readiness_satisfied: None,
        }
    }

    // ── Safety invariants ──────────────────────────────────────────────────────

    #[test]
    fn writes_enabled_always_false_safe_request() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_safe_request() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_safe_request() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false_blocked_request() {
        let result = preview_linked_second_pass_execution(&blocked_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_blocked_request() {
        let result = preview_linked_second_pass_execution(&blocked_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_blocked_request() {
        let result = preview_linked_second_pass_execution(&blocked_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn write_gate_disabled_always_reported() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn no_filesystem_writes_attempted() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(!result.network_writes_attempted);
        assert!(!result.writes_enabled);
    }

    // ── Blocked prerequisites ──────────────────────────────────────────────────

    #[test]
    fn missing_linked_update_plan_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            field_summaries: None,
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        // No field summaries → zero batches but still dryRunReady (safe default)
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::DryRunReady
        );
        assert!(result.batches.is_empty());
    }

    #[test]
    fn missing_record_write_preview_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            record_write_preview_ready: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains(PREREQ_RECORD_WRITE_PREVIEW));
    }

    #[test]
    fn missing_mapping_checkpoint_preview_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            mapping_checkpoint_preview_ready: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains(PREREQ_MAPPING_CHECKPOINT_PREVIEW));
    }

    #[test]
    fn record_write_preview_false_causes_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            record_write_preview_ready: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn mapping_checkpoint_preview_false_causes_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            mapping_checkpoint_preview_ready: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn write_phase_ordering_unsafe_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            write_phase_ordering_safe: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains(PREREQ_WRITE_PHASE_ORDERING));
    }

    #[test]
    fn checkpoint_durability_unsafe_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            checkpoint_durability_safe: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains(PREREQ_CHECKPOINT_DURABILITY));
    }

    #[test]
    fn sensitive_data_unsafe_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            sensitive_data_safe: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains(PREREQ_SENSITIVE_DATA));
    }

    #[test]
    fn final_validation_enforcement_missing_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            final_validation_enforcement_present: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains(PREREQ_FINAL_VALIDATION));
    }

    #[test]
    fn live_write_readiness_missing_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            live_write_readiness_satisfied: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains(PREREQ_LWR));
    }

    #[test]
    fn batch_size_exceeds_max_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            batch_size: Some(11),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("LSEP-BATCH"));
    }

    // ── DryRunReady behavior ───────────────────────────────────────────────────

    #[test]
    fn safe_plan_returns_dry_run_ready() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::DryRunReady
        );
        assert_eq!(result.mode, LinkedSecondPassPreviewMode::DryRunOnly);
    }

    #[test]
    fn dry_run_ready_has_no_blocked_reason() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn unresolved_links_produce_warning_not_blocked() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            field_summaries: Some(vec![LinkedSecondPassFieldSummary {
                table_label: "Projects".to_string(),
                field_label: "Tasks".to_string(),
                record_count: 10,
                batch_count: 1,
                unresolved_link_count: 3,
            }]),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::DryRunReady
        );
        assert!(result.mapping_summary.unresolved_link_count > 0);
        assert!(result.message.contains("unresolved"));
    }

    #[test]
    fn linked_field_summary_is_deterministic() {
        let result1 = preview_linked_second_pass_execution(&safe_request());
        let result2 = preview_linked_second_pass_execution(&safe_request());
        assert_eq!(result1.batches.len(), result2.batches.len());
        for (b1, b2) in result1.batches.iter().zip(result2.batches.iter()) {
            assert_eq!(b1.batch_id, b2.batch_id);
            assert_eq!(b1.update_count, b2.update_count);
        }
    }

    #[test]
    fn batch_ordering_is_deterministic() {
        let result = preview_linked_second_pass_execution(&safe_request());
        for (i, batch) in result.batches.iter().enumerate() {
            assert_eq!(batch.batch_index, i);
        }
    }

    #[test]
    fn batch_count_matches_field_summaries() {
        let result = preview_linked_second_pass_execution(&safe_request());
        // Projects/Tasks: 15 records / batch 10 = 2 batches
        // Tasks/Owner: 5 records / batch 10 = 1 batch
        assert_eq!(result.batches.len(), 3);
    }

    #[test]
    fn batch_update_count_never_exceeds_batch_size() {
        let result = preview_linked_second_pass_execution(&safe_request());
        for batch in &result.batches {
            assert!(batch.update_count <= MAX_SAFE_BATCH_SIZE);
        }
    }

    #[test]
    fn missing_mapping_coverage_blocked_when_required() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            mapping_checkpoint_preview_ready: Some(false),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::Blocked
        );
    }

    // ── No-leak assertions ─────────────────────────────────────────────────────

    #[test]
    fn no_token_in_serialized_result() {
        let result = preview_linked_second_pass_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("pat_"));
        assert!(!json.contains("Bearer "));
    }

    #[test]
    fn no_absolute_path_in_serialized_result() {
        let result = preview_linked_second_pass_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("C:\\\\"));
    }

    #[test]
    fn no_record_payload_in_serialized_result() {
        let result = preview_linked_second_pass_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn no_attachment_url_in_serialized_result() {
        let result = preview_linked_second_pass_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn no_success_state_in_result() {
        let result = preview_linked_second_pass_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_raw_record_ids_in_batches() {
        let result = preview_linked_second_pass_execution(&safe_request());
        let json = serde_json::to_string(&result.batches).expect("serialize");
        // Check that no Airtable-formatted IDs (rec/fld/tbl + 10+ alphanumeric) appear.
        let has_airtable_id = |prefix: &str| -> bool {
            let chars: Vec<char> = json.chars().collect();
            let plen = prefix.len();
            let prefix_chars: Vec<char> = prefix.chars().collect();
            for i in 0..chars.len().saturating_sub(plen + 9) {
                if chars[i..i + plen] == prefix_chars[..] {
                    let run = chars[i + plen..]
                        .iter()
                        .take(14)
                        .take_while(|c| c.is_ascii_alphanumeric())
                        .count();
                    if run >= 10 {
                        return true;
                    }
                }
            }
            false
        };
        assert!(
            !has_airtable_id("rec"),
            "raw Airtable record ID found in batches"
        );
        assert!(
            !has_airtable_id("fld"),
            "raw Airtable field ID found in batches"
        );
    }

    #[test]
    fn message_states_live_updates_disabled() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result.message.to_lowercase().contains("disabled"));
    }

    #[test]
    fn message_states_no_restore_execution_started() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result
            .message
            .to_lowercase()
            .contains("does not start any restore execution"));
    }

    #[test]
    fn message_states_no_checkpoint_files_written() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result
            .message
            .to_lowercase()
            .contains("no checkpoint files are written"));
    }

    #[test]
    fn message_states_no_record_ids() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result
            .message
            .to_lowercase()
            .contains("no record ids are present"));
    }

    #[test]
    fn no_success_state_introduced() {
        let result = preview_linked_second_pass_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("succeeded"));
        assert!(!json.contains("\"complete\""));
        assert!(!json.contains("\"executed\""));
    }

    // ── Mapping summary ────────────────────────────────────────────────────────

    #[test]
    fn mapping_summary_counts_match_request() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert_eq!(result.mapping_summary.total_update_count, 20);
        assert_eq!(result.mapping_summary.tables_with_linked_fields, 2);
        assert_eq!(result.mapping_summary.total_linked_fields, 3);
        assert!(result.mapping_summary.mapping_complete_before_second_pass);
    }

    #[test]
    fn mapping_summary_note_is_non_empty() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(!result.mapping_summary.note.is_empty());
    }

    // ── Safety snapshot ────────────────────────────────────────────────────────

    #[test]
    fn safety_snapshot_write_gate_disabled_always_true() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn safety_snapshot_reflects_request_values() {
        let result = preview_linked_second_pass_execution(&safe_request());
        assert!(result.safety_snapshot.record_write_preview_ready);
        assert!(result.safety_snapshot.mapping_checkpoint_preview_ready);
        assert!(result.safety_snapshot.write_phase_ordering_safe);
        assert!(result.safety_snapshot.checkpoint_durability_safe);
        assert!(result.safety_snapshot.sensitive_data_safe);
        assert!(result.safety_snapshot.final_validation_enforcement_present);
        assert!(result.safety_snapshot.live_write_readiness_satisfied);
    }

    // ── Empty field summaries ──────────────────────────────────────────────────

    #[test]
    fn empty_field_summaries_produces_zero_batches() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            field_summaries: Some(vec![]),
            total_update_count: Some(0),
            tables_with_linked_fields: Some(0),
            total_linked_fields: Some(0),
            second_pass_batch_count: Some(0),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert_eq!(
            result.status,
            LinkedSecondPassExecutionPreviewStatus::DryRunReady
        );
        assert!(result.batches.is_empty());
    }

    #[test]
    fn zero_record_count_field_skipped_in_batches() {
        let req = LinkedSecondPassExecutionPreviewRequest {
            field_summaries: Some(vec![LinkedSecondPassFieldSummary {
                table_label: "Projects".to_string(),
                field_label: "Tasks".to_string(),
                record_count: 0,
                batch_count: 0,
                unresolved_link_count: 0,
            }]),
            ..safe_request()
        };
        let result = preview_linked_second_pass_execution(&req);
        assert!(result.batches.is_empty());
    }
}
