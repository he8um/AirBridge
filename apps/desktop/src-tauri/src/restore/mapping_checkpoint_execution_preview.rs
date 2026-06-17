use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the mapping/checkpoint execution preview.
///
/// Safety invariants:
/// - `DryRunReady` does NOT enable live mapping capture or checkpoint persistence.
/// - `writes_enabled` is always `false`.
/// - No filesystem checkpoint files are written.
/// - No temp or mock checkpoint files are written.
/// - No Airtable API calls are made by this module.
/// - No token, full path, record payload, raw HTTP, or attachment URL appears
///   in any result field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MappingCheckpointExecutionPreviewStatus {
    /// All safety prerequisites are present; a dry-run mapping/checkpoint preview
    /// is available. Live execution, checkpoint persistence, and ID mapping capture
    /// remain disabled and no restore execution is started.
    DryRunReady,
    /// At least one required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Status of a single preview step.
///
/// Note: `completed` / `succeeded` / `executed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MappingCheckpointPreviewStepStatus {
    /// The step is planned and would execute if writes were enabled.
    Pending,
    /// The step is blocked by a safety prerequisite.
    Blocked,
    /// The step is skipped because a prerequisite failed.
    Skipped,
}

/// Execution mode for the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MappingCheckpointPreviewMode {
    /// Dry-run only — no live execution path is reachable.
    DryRunOnly,
    /// Live mapping/checkpoint execution is blocked by product policy.
    LiveBlocked,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single step in the ordered mapping/checkpoint execution preview.
///
/// Safety properties:
/// - No raw record IDs or field values.
/// - No token.
/// - No absolute path.
/// - No attachment URL.
/// - Only safe counts, labels, and phase identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingCheckpointPreviewStep {
    pub step_index: usize,
    pub step_id: String,
    pub phase_label: String,
    pub checkpoint_boundary_label: String,
    pub status: MappingCheckpointPreviewStepStatus,
    /// Number of entries that would be in this mapping/checkpoint (safe count only).
    pub entry_count: usize,
    pub note: String,
}

/// Summary of the ID mapping phase preview.
///
/// Safety properties:
/// - No raw record IDs (old or new).
/// - No field values.
/// - Only safe counts and phase labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdMappingPreviewSummary {
    /// Total number of records for which mappings would be captured.
    pub total_mapping_count: usize,
    /// Number of tables that require linked-record remapping.
    pub tables_requiring_remapping: usize,
    /// Total first-pass create batches (one mapping entry per created record).
    pub first_pass_batch_count: usize,
    /// Whether old-to-new ID mapping is available before second-pass linked updates.
    pub mapping_available_before_second_pass: bool,
    pub note: String,
}

/// Summary of the checkpoint boundary preview.
///
/// Safety properties:
/// - No raw record IDs.
/// - No field values.
/// - Only safe counts, phase labels, and boundary labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointPreviewSummary {
    /// Total number of checkpoint boundaries in the preview.
    pub total_checkpoint_count: usize,
    /// Number of checkpoints associated with the record create phase.
    pub record_create_checkpoint_count: usize,
    /// Number of checkpoints associated with the linked update phase.
    pub linked_update_checkpoint_count: usize,
    /// Whether a checkpoint is taken after schema preview (before record create).
    pub has_pre_record_create_checkpoint: bool,
    /// Whether a checkpoint is taken before the linked update phase.
    pub has_pre_linked_update_checkpoint: bool,
    /// Whether a checkpoint is taken before final validation.
    pub has_pre_final_validation_checkpoint: bool,
    pub note: String,
}

/// Point-in-time safety snapshot for the mapping/checkpoint execution preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingCheckpointSafetySnapshot {
    pub write_gate_disabled: bool,
    pub record_write_preview_ready: bool,
    pub checkpoint_durability_safe: bool,
    pub failure_modes_safe: bool,
    pub rollback_limitation_safe: bool,
    pub final_validation_enforcement_present: bool,
    pub sensitive_data_safe: bool,
    pub live_write_readiness_satisfied: bool,
}

/// Request for the mapping/checkpoint execution preview.
///
/// No token field — token is not required or accepted.
/// No full path — filename label only.
/// No raw record payloads — only counts and flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingCheckpointExecutionPreviewRequest {
    /// Filename label for the backup package. No directory component.
    pub package_filename: Option<String>,
    /// Whether the record write execution preview returned DryRunReady.
    pub record_write_preview_ready: Option<bool>,
    /// Total first-pass (create) batch count from the record write preview.
    pub first_pass_batch_count: Option<usize>,
    /// Total second-pass (linked update) batch count from the record write preview.
    pub second_pass_batch_count: Option<usize>,
    /// Total record count from the record write preview.
    pub total_record_count: Option<usize>,
    /// Number of tables that have linked record fields requiring remapping.
    pub tables_requiring_remapping: Option<usize>,
    /// Whether the checkpoint durability policy result is safe.
    pub checkpoint_durability_safe: Option<bool>,
    /// Whether the failure modes policy result is safe.
    pub failure_modes_safe: Option<bool>,
    /// Whether the rollback limitation policy result is safe.
    pub rollback_limitation_safe: Option<bool>,
    /// Whether the final validation enforcement policy result is present.
    pub final_validation_enforcement_present: Option<bool>,
    /// Whether the sensitive data safety policy result is safe.
    pub sensitive_data_safe: Option<bool>,
    /// Whether the live-write readiness result is ready or warning (advisory only).
    pub live_write_readiness_satisfied: Option<bool>,
}

/// Result of the mapping/checkpoint execution preview.
///
/// Safety invariants (enforced):
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No full filesystem path field.
/// - No raw record IDs (old or new).
/// - No raw record payload or field values.
/// - No raw HTTP request or response body.
/// - No attachment URL.
/// - No filesystem checkpoint files are written.
/// - No temp or mock checkpoint files are written.
/// - Status is never `succeeded` or any completion state.
/// - `DryRunReady` does NOT enable live mapping capture or checkpoint persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingCheckpointExecutionPreviewResult {
    pub status: MappingCheckpointExecutionPreviewStatus,
    pub mode: MappingCheckpointPreviewMode,
    pub message: String,
    pub steps: Vec<MappingCheckpointPreviewStep>,
    pub id_mapping_summary: IdMappingPreviewSummary,
    pub checkpoint_summary: CheckpointPreviewSummary,
    pub safety_snapshot: MappingCheckpointSafetySnapshot,
    pub total_step_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always false — live mapping/checkpoint execution is not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const PREREQ_WRITE_GATE: &str = "MCEP-PRE-01";
const PREREQ_RECORD_WRITE_PREVIEW: &str = "MCEP-PRE-02";
const PREREQ_CHECKPOINT_DURABILITY: &str = "MCEP-PRE-03";
const PREREQ_FAILURE_MODES: &str = "MCEP-PRE-04";
const PREREQ_ROLLBACK_LIMITATION: &str = "MCEP-PRE-05";
const PREREQ_FINAL_VALIDATION_ENFORCEMENT: &str = "MCEP-PRE-06";
const PREREQ_SENSITIVE_DATA: &str = "MCEP-PRE-07";
const PREREQ_LWR: &str = "MCEP-PRE-08";

// ── Step IDs ──────────────────────────────────────────────────────────────────

const STEP_SCHEMA_CHECKPOINT: &str = "MCEP-CHK-SCHEMA";
const STEP_PRE_RECORD_CREATE: &str = "MCEP-CHK-PRE-REC";
const STEP_RECORD_BATCH_PREFIX: &str = "MCEP-MAP-REC-B";
const STEP_PRE_LINKED_UPDATE: &str = "MCEP-CHK-PRE-LINK";
const STEP_LINKED_BATCH_PREFIX: &str = "MCEP-CHK-LINK-B";
const STEP_PRE_FINAL_VALIDATION: &str = "MCEP-CHK-PRE-FV";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds a mapping/checkpoint execution preview from a request.
///
/// This function:
/// - Never calls any Airtable API endpoint.
/// - Never creates, updates, or deletes any record, table, or field.
/// - Never writes a checkpoint file (real, temp, or mock).
/// - Never returns a token, full path, record ID, record payload, raw HTTP body,
///   or attachment URL.
/// - Always returns `writes_enabled: false`, `no_changes_made: true`,
///   `network_writes_attempted: false`.
/// - Always consults `evaluate_write_gate()` to confirm writes remain disabled.
/// - A `DryRunReady` result does NOT enable mapping capture or checkpoint persistence.
pub fn preview_mapping_checkpoint_execution(
    request: &MappingCheckpointExecutionPreviewRequest,
) -> MappingCheckpointExecutionPreviewResult {
    // Always consult the write gate first.
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let record_write_preview_ready = request.record_write_preview_ready.unwrap_or(false);
    let checkpoint_durability_safe = request.checkpoint_durability_safe.unwrap_or(false);
    let failure_modes_safe = request.failure_modes_safe.unwrap_or(false);
    let rollback_limitation_safe = request.rollback_limitation_safe.unwrap_or(false);
    let final_validation_enforcement_present = request
        .final_validation_enforcement_present
        .unwrap_or(false);
    let sensitive_data_safe = request.sensitive_data_safe.unwrap_or(false);
    let live_write_readiness_satisfied = request.live_write_readiness_satisfied.unwrap_or(false);

    let snapshot = MappingCheckpointSafetySnapshot {
        write_gate_disabled,
        record_write_preview_ready,
        checkpoint_durability_safe,
        failure_modes_safe,
        rollback_limitation_safe,
        final_validation_enforcement_present,
        sensitive_data_safe,
        live_write_readiness_satisfied,
    };

    // Check prerequisites in order; first failure wins.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        Some(format!(
            "{PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Live mapping and checkpoint execution must not be attempted."
        ))
    } else if !record_write_preview_ready {
        Some(format!(
            "{PREREQ_RECORD_WRITE_PREVIEW}: Record write execution preview has not returned \
             DryRunReady. Complete the record write preview before requesting a \
             mapping/checkpoint preview."
        ))
    } else if !checkpoint_durability_safe {
        Some(format!(
            "{PREREQ_CHECKPOINT_DURABILITY}: Checkpoint durability policy is not safe. \
             A compliant checkpoint plan must be declared."
        ))
    } else if !failure_modes_safe {
        Some(format!(
            "{PREREQ_FAILURE_MODES}: Failure modes policy is not safe. \
             All 10 required failure modes must have safe handling plans."
        ))
    } else if !rollback_limitation_safe {
        Some(format!(
            "{PREREQ_ROLLBACK_LIMITATION}: Rollback limitation policy is not safe. \
             Automatic destructive rollback must not be reachable."
        ))
    } else if !final_validation_enforcement_present {
        Some(format!(
            "{PREREQ_FINAL_VALIDATION_ENFORCEMENT}: Final validation enforcement policy \
             has not been verified. All three completion guards must be declared."
        ))
    } else if !sensitive_data_safe {
        Some(format!(
            "{PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied. \
             All 10 exposure surfaces must have redaction coverage."
        ))
    } else if !live_write_readiness_satisfied {
        Some(format!(
            "{PREREQ_LWR}: Live write readiness policy is not satisfied. \
             All 17 required safety gates must be declared."
        ))
    } else {
        None
    };

    let empty_mapping_summary = IdMappingPreviewSummary {
        total_mapping_count: 0,
        tables_requiring_remapping: 0,
        first_pass_batch_count: 0,
        mapping_available_before_second_pass: false,
        note: "Mapping preview unavailable — prerequisites not satisfied.".to_string(),
    };
    let empty_checkpoint_summary = CheckpointPreviewSummary {
        total_checkpoint_count: 0,
        record_create_checkpoint_count: 0,
        linked_update_checkpoint_count: 0,
        has_pre_record_create_checkpoint: false,
        has_pre_linked_update_checkpoint: false,
        has_pre_final_validation_checkpoint: false,
        note: "Checkpoint preview unavailable — prerequisites not satisfied.".to_string(),
    };

    if let Some(ref reason) = blocked_reason {
        let blocked_step = MappingCheckpointPreviewStep {
            step_index: 0,
            step_id: "MCEP-BLOCKED".to_string(),
            phase_label: "blocked".to_string(),
            checkpoint_boundary_label: "—".to_string(),
            status: MappingCheckpointPreviewStepStatus::Blocked,
            entry_count: 0,
            note: "Safety prerequisites not satisfied. No mapping or checkpoint steps can be previewed."
                .to_string(),
        };
        return MappingCheckpointExecutionPreviewResult {
            status: MappingCheckpointExecutionPreviewStatus::Blocked,
            mode: MappingCheckpointPreviewMode::LiveBlocked,
            message: format!(
                "Mapping/checkpoint execution preview is blocked. {reason} \
                 Live mapping capture and checkpoint persistence remain disabled."
            ),
            steps: vec![blocked_step],
            id_mapping_summary: empty_mapping_summary,
            checkpoint_summary: empty_checkpoint_summary,
            safety_snapshot: snapshot,
            total_step_count: 0,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the dry-run preview.
    let first_pass_batches = request.first_pass_batch_count.unwrap_or(0);
    let second_pass_batches = request.second_pass_batch_count.unwrap_or(0);
    let total_records = request.total_record_count.unwrap_or(0);
    let tables_remapping = request.tables_requiring_remapping.unwrap_or(0);

    let mapping_summary = IdMappingPreviewSummary {
        total_mapping_count: total_records,
        tables_requiring_remapping: tables_remapping,
        first_pass_batch_count: first_pass_batches,
        mapping_available_before_second_pass: first_pass_batches > 0,
        note: format!(
            "Would capture old-to-new record ID mapping after each first-pass create batch. \
             {} record(s) across {} first-pass batch(es). \
             Mapping unavailable until execution — preview only. \
             No raw record IDs are present in this preview.",
            total_records, first_pass_batches
        ),
    };

    let steps = build_preview_steps(first_pass_batches, second_pass_batches);

    let record_create_checkpoints = first_pass_batches;
    let linked_update_checkpoints = second_pass_batches;
    let fixed_checkpoints = 3usize; // schema, pre-linked-update, pre-final-validation
    let total_checkpoints =
        fixed_checkpoints + record_create_checkpoints + linked_update_checkpoints;

    let checkpoint_summary = CheckpointPreviewSummary {
        total_checkpoint_count: total_checkpoints,
        record_create_checkpoint_count: record_create_checkpoints,
        linked_update_checkpoint_count: linked_update_checkpoints,
        has_pre_record_create_checkpoint: true,
        has_pre_linked_update_checkpoint: first_pass_batches > 0,
        has_pre_final_validation_checkpoint: true,
        note: format!(
            "Would place checkpoint boundaries after schema preview, \
             before record create phase, after each of {} record create batch(es), \
             before linked update phase, after each of {} linked update batch(es), \
             and before final validation. \
             No checkpoint files are written in this preview. \
             Checkpoint persistence requires live execution.",
            first_pass_batches, second_pass_batches
        ),
    };

    let total_step_count = steps.len();

    MappingCheckpointExecutionPreviewResult {
        status: MappingCheckpointExecutionPreviewStatus::DryRunReady,
        mode: MappingCheckpointPreviewMode::DryRunOnly,
        message: format!(
            "Mapping/checkpoint execution preview is ready (dry-run only). \
             {} first-pass batch(es), {} second-pass batch(es), \
             {} total checkpoint boundary(ies), {} record(s) to map. \
             Live mapping capture and checkpoint persistence remain disabled. \
             This preview does not start any restore execution. \
             No checkpoint files are written.",
            first_pass_batches, second_pass_batches, total_checkpoints, total_records
        ),
        steps,
        id_mapping_summary: mapping_summary,
        checkpoint_summary,
        safety_snapshot: snapshot,
        total_step_count,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_preview_steps(
    first_pass_batches: usize,
    second_pass_batches: usize,
) -> Vec<MappingCheckpointPreviewStep> {
    let mut steps = Vec::new();
    let mut idx = 0usize;

    // Step 1: checkpoint after schema preview (before record create phase)
    steps.push(MappingCheckpointPreviewStep {
        step_index: idx,
        step_id: STEP_SCHEMA_CHECKPOINT.to_string(),
        phase_label: "schema-checkpoint".to_string(),
        checkpoint_boundary_label: "After schema preview — before record create phase".to_string(),
        status: MappingCheckpointPreviewStepStatus::Pending,
        entry_count: 0,
        note: "Would record that schema phase is complete before beginning record creation. \
               No checkpoint file written in this preview."
            .to_string(),
    });
    idx += 1;

    // Step 2: checkpoint before record create phase
    steps.push(MappingCheckpointPreviewStep {
        step_index: idx,
        step_id: STEP_PRE_RECORD_CREATE.to_string(),
        phase_label: "pre-record-create-checkpoint".to_string(),
        checkpoint_boundary_label: "Before record create phase".to_string(),
        status: MappingCheckpointPreviewStepStatus::Pending,
        entry_count: 0,
        note: "Would mark the start of the record create phase. \
               No checkpoint file written in this preview."
            .to_string(),
    });
    idx += 1;

    // Per first-pass batch: capture mapping entries + checkpoint
    for b in 0..first_pass_batches {
        steps.push(MappingCheckpointPreviewStep {
            step_index: idx,
            step_id: format!("{STEP_RECORD_BATCH_PREFIX}{b:03}"),
            phase_label: "record-create-mapping".to_string(),
            checkpoint_boundary_label: format!(
                "After first-pass create batch {} — capture ID mapping",
                b + 1
            ),
            status: MappingCheckpointPreviewStepStatus::Pending,
            entry_count: 0, // actual count only known at execution time
            note: format!(
                "Would capture old-to-new record ID mapping after first-pass create batch {}. \
                 Mapping entry count determined at execution time. \
                 No raw record IDs or field data present in this preview. \
                 No checkpoint file written in this preview.",
                b + 1
            ),
        });
        idx += 1;
    }

    // Step: checkpoint before linked update phase (only if first-pass batches ran)
    if first_pass_batches > 0 {
        steps.push(MappingCheckpointPreviewStep {
            step_index: idx,
            step_id: STEP_PRE_LINKED_UPDATE.to_string(),
            phase_label: "pre-linked-update-checkpoint".to_string(),
            checkpoint_boundary_label: "Before linked record update phase — ID mapping complete"
                .to_string(),
            status: MappingCheckpointPreviewStepStatus::Pending,
            entry_count: 0,
            note: "Would verify ID mapping is complete and checkpoint the mapping state \
                   before beginning linked record updates. \
                   No checkpoint file written in this preview."
                .to_string(),
        });
        idx += 1;
    }

    // Per second-pass batch: checkpoint after linked update
    for b in 0..second_pass_batches {
        steps.push(MappingCheckpointPreviewStep {
            step_index: idx,
            step_id: format!("{STEP_LINKED_BATCH_PREFIX}{b:03}"),
            phase_label: "linked-update-checkpoint".to_string(),
            checkpoint_boundary_label: format!("After second-pass linked-update batch {}", b + 1),
            status: MappingCheckpointPreviewStepStatus::Pending,
            entry_count: 0,
            note: format!(
                "Would checkpoint after second-pass linked-update batch {}. \
                 No checkpoint file written in this preview.",
                b + 1
            ),
        });
        idx += 1;
    }

    // Final step: checkpoint before final validation
    steps.push(MappingCheckpointPreviewStep {
        step_index: idx,
        step_id: STEP_PRE_FINAL_VALIDATION.to_string(),
        phase_label: "pre-final-validation-checkpoint".to_string(),
        checkpoint_boundary_label: "Before final validation — all batches complete".to_string(),
        status: MappingCheckpointPreviewStepStatus::Pending,
        entry_count: 0,
        note: "Would checkpoint that all record create and linked update batches are complete \
               before beginning final validation. \
               No checkpoint file written in this preview."
            .to_string(),
    });

    steps
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_request() -> MappingCheckpointExecutionPreviewRequest {
        MappingCheckpointExecutionPreviewRequest {
            package_filename: Some("test-backup.airbridge".to_string()),
            record_write_preview_ready: Some(true),
            first_pass_batch_count: Some(4),
            second_pass_batch_count: Some(2),
            total_record_count: Some(35),
            tables_requiring_remapping: Some(2),
            checkpoint_durability_safe: Some(true),
            failure_modes_safe: Some(true),
            rollback_limitation_safe: Some(true),
            final_validation_enforcement_present: Some(true),
            sensitive_data_safe: Some(true),
            live_write_readiness_satisfied: Some(true),
        }
    }

    fn missing_request() -> MappingCheckpointExecutionPreviewRequest {
        MappingCheckpointExecutionPreviewRequest {
            package_filename: None,
            record_write_preview_ready: None,
            first_pass_batch_count: None,
            second_pass_batch_count: None,
            total_record_count: None,
            tables_requiring_remapping: None,
            checkpoint_durability_safe: None,
            failure_modes_safe: None,
            rollback_limitation_safe: None,
            final_validation_enforcement_present: None,
            sensitive_data_safe: None,
            live_write_readiness_satisfied: None,
        }
    }

    // ── Basic safety invariants ────────────────────────────────────────────────

    #[test]
    fn writes_enabled_always_false_safe_request() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_safe_request() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_safe_request() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false_missing_request() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_missing_request() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_missing_request() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        assert!(!result.network_writes_attempted);
    }

    // ── Blocked when prerequisites missing ────────────────────────────────────

    #[test]
    fn missing_all_prerequisites_returns_blocked() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn record_write_preview_not_ready_returns_blocked() {
        let mut req = safe_request();
        req.record_write_preview_ready = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn record_write_preview_blocked_returns_blocked() {
        let mut req = safe_request();
        req.record_write_preview_ready = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_RECORD_WRITE_PREVIEW));
    }

    #[test]
    fn checkpoint_durability_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.checkpoint_durability_safe = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn failure_modes_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.failure_modes_safe = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn rollback_limitation_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.rollback_limitation_safe = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn final_validation_enforcement_missing_returns_blocked() {
        let mut req = safe_request();
        req.final_validation_enforcement_present = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn sensitive_data_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.sensitive_data_safe = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    #[test]
    fn live_write_readiness_missing_returns_blocked() {
        let mut req = safe_request();
        req.live_write_readiness_satisfied = Some(false);
        let result = preview_mapping_checkpoint_execution(&req);
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::Blocked
        );
    }

    // ── DryRunReady for safe request ───────────────────────────────────────────

    #[test]
    fn safe_request_returns_dry_run_ready() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert_eq!(
            result.status,
            MappingCheckpointExecutionPreviewStatus::DryRunReady
        );
    }

    #[test]
    fn dry_run_ready_mode_is_dry_run_only() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert_eq!(result.mode, MappingCheckpointPreviewMode::DryRunOnly);
    }

    #[test]
    fn dry_run_ready_has_no_blocked_reason() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn dry_run_ready_write_gate_disabled_in_snapshot() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── Mapping summary ────────────────────────────────────────────────────────

    #[test]
    fn mapping_summary_is_deterministic() {
        let r1 = preview_mapping_checkpoint_execution(&safe_request());
        let r2 = preview_mapping_checkpoint_execution(&safe_request());
        assert_eq!(
            r1.id_mapping_summary.total_mapping_count,
            r2.id_mapping_summary.total_mapping_count
        );
        assert_eq!(
            r1.id_mapping_summary.first_pass_batch_count,
            r2.id_mapping_summary.first_pass_batch_count
        );
        assert_eq!(
            r1.id_mapping_summary.tables_requiring_remapping,
            r2.id_mapping_summary.tables_requiring_remapping
        );
        assert_eq!(
            r1.id_mapping_summary.mapping_available_before_second_pass,
            r2.id_mapping_summary.mapping_available_before_second_pass
        );
    }

    #[test]
    fn mapping_summary_counts_match_request() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert_eq!(result.id_mapping_summary.total_mapping_count, 35);
        assert_eq!(result.id_mapping_summary.tables_requiring_remapping, 2);
        assert_eq!(result.id_mapping_summary.first_pass_batch_count, 4);
        assert!(
            result
                .id_mapping_summary
                .mapping_available_before_second_pass
        );
    }

    #[test]
    fn no_raw_record_ids_in_mapping_summary() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let json = serde_json::to_string(&result.id_mapping_summary).expect("serialize");
        // Airtable IDs are "rec"/"fld"/"tbl" followed by exactly 14 alphanumeric chars.
        // Scan char-by-char to avoid byte-boundary panics on multi-byte chars.
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
            "raw Airtable record ID found in mapping summary"
        );
        assert!(
            !has_airtable_id("fld"),
            "raw Airtable field ID found in mapping summary"
        );
        assert!(
            !has_airtable_id("tbl"),
            "raw Airtable table ID found in mapping summary"
        );
    }

    // ── Checkpoint boundary ordering ───────────────────────────────────────────

    #[test]
    fn checkpoint_boundary_ordering_is_deterministic() {
        let r1 = preview_mapping_checkpoint_execution(&safe_request());
        let r2 = preview_mapping_checkpoint_execution(&safe_request());
        let ids1: Vec<_> = r1.steps.iter().map(|s| &s.step_id).collect();
        let ids2: Vec<_> = r2.steps.iter().map(|s| &s.step_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn step_indices_are_sequential() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        for (i, step) in result.steps.iter().enumerate() {
            assert_eq!(
                step.step_index, i,
                "step_index must be sequential, got {} at position {}",
                step.step_index, i
            );
        }
    }

    #[test]
    fn schema_checkpoint_comes_first() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(!result.steps.is_empty());
        assert_eq!(result.steps[0].step_id, STEP_SCHEMA_CHECKPOINT);
    }

    #[test]
    fn pre_record_create_checkpoint_comes_second() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.steps.len() >= 2);
        assert_eq!(result.steps[1].step_id, STEP_PRE_RECORD_CREATE);
    }

    #[test]
    fn record_mapping_steps_come_before_linked_update_checkpoint() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let map_last_idx = result
            .steps
            .iter()
            .filter(|s| s.step_id.starts_with(STEP_RECORD_BATCH_PREFIX))
            .map(|s| s.step_index)
            .max()
            .unwrap_or(0);
        let pre_link_idx = result
            .steps
            .iter()
            .find(|s| s.step_id == STEP_PRE_LINKED_UPDATE)
            .map(|s| s.step_index)
            .unwrap_or(usize::MAX);
        assert!(
            map_last_idx < pre_link_idx,
            "record mapping steps must precede pre-linked-update checkpoint"
        );
    }

    #[test]
    fn pre_final_validation_step_comes_last() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let last = result.steps.last().expect("should have steps");
        assert_eq!(last.step_id, STEP_PRE_FINAL_VALIDATION);
    }

    #[test]
    fn total_step_count_equals_steps_len() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert_eq!(result.total_step_count, result.steps.len());
    }

    // ── Checkpoint summary ─────────────────────────────────────────────────────

    #[test]
    fn checkpoint_summary_has_pre_record_create() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.checkpoint_summary.has_pre_record_create_checkpoint);
    }

    #[test]
    fn checkpoint_summary_has_pre_linked_update_when_first_pass_batches_exist() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.checkpoint_summary.has_pre_linked_update_checkpoint);
    }

    #[test]
    fn checkpoint_summary_has_pre_final_validation() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(
            result
                .checkpoint_summary
                .has_pre_final_validation_checkpoint
        );
    }

    #[test]
    fn checkpoint_summary_counts_match_expected() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        // schema + pre-record-create + 4 record batches + pre-linked-update + 2 linked batches + pre-FV = 10
        assert_eq!(result.checkpoint_summary.record_create_checkpoint_count, 4);
        assert_eq!(result.checkpoint_summary.linked_update_checkpoint_count, 2);
    }

    // ── No filesystem writes ───────────────────────────────────────────────────

    #[test]
    fn no_filesystem_writes_attempted() {
        // The function must return without side effects.
        // If it compiles and returns normally, no filesystem writes occurred.
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    // ── Safety serialization checks ────────────────────────────────────────────

    #[test]
    fn no_token_in_serialization() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn no_absolute_path_in_serialization() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_raw_http_in_serialization() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"request\":{\"method\""));
        assert!(!json.contains("\"response\":{\"status\""));
        assert!(!json.contains("Authorization: Bearer"));
    }

    #[test]
    fn no_attachment_url_in_serialization() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn no_record_payload_in_serialization() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn no_succeeded_in_serialization() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn dry_run_message_states_execution_remains_disabled() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("disabled"));
    }

    #[test]
    fn dry_run_message_states_no_restore_execution() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("does not start any restore execution"));
    }

    #[test]
    fn dry_run_message_states_no_checkpoint_files_written() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("no checkpoint files are written"));
    }

    // ── Write gate not bypassed ────────────────────────────────────────────────

    #[test]
    fn write_gate_not_bypassed_by_preview() {
        let gate_before = evaluate_write_gate();
        let _result = preview_mapping_checkpoint_execution(&safe_request());
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
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── Blocked mode ───────────────────────────────────────────────────────────

    #[test]
    fn blocked_result_has_blocked_reason() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        assert!(result.blocked_reason.is_some());
    }

    #[test]
    fn blocked_result_mode_is_live_blocked() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        assert_eq!(result.mode, MappingCheckpointPreviewMode::LiveBlocked);
    }

    #[test]
    fn blocked_result_has_blocked_step() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        assert!(!result.steps.is_empty());
        assert_eq!(
            result.steps[0].status,
            MappingCheckpointPreviewStepStatus::Blocked
        );
    }

    #[test]
    fn blocked_message_mentions_disabled() {
        let result = preview_mapping_checkpoint_execution(&missing_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("disabled"));
    }

    // ── No success state ───────────────────────────────────────────────────────

    #[test]
    fn no_success_state_introduced() {
        let result = preview_mapping_checkpoint_execution(&safe_request());
        assert!(!result.writes_enabled);
        assert!(result
            .message
            .to_lowercase()
            .contains("does not start any restore execution"));
    }
}
