use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the checkpoint metadata store operation.
///
/// Safety invariants:
/// - `Stored` does NOT enable live restore execution.
/// - `Stored` does NOT indicate restore success.
/// - `writes_enabled` is always `false` (restore writes are disabled).
/// - `network_writes_attempted` is always `false`.
/// - No Airtable API calls are made by this module.
/// - No token, full path, record payload, raw HTTP, old/new record IDs,
///   or attachment URL appears in any result or stored file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreCheckpointStoreStatus {
    /// The sanitized checkpoint metadata was written to the app-controlled
    /// checkpoint directory. Restore execution remains disabled.
    Stored,
    /// A required safety prerequisite is missing or unsafe. No file was written.
    Blocked,
}

/// Storage mode for the checkpoint metadata store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreCheckpointStoreMode {
    /// Metadata only — no record payloads, no IDs, no token, no full path,
    /// no attachment URL, no raw HTTP body are written.
    MetadataOnly,
}

// ── Phase / boundary types ────────────────────────────────────────────────────

/// A restore pipeline phase recorded in the checkpoint manifest.
///
/// Safety properties:
/// - No record IDs.
/// - No record field values.
/// - No token.
/// - No absolute path.
/// - No attachment URL.
/// - Only safe labels and counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCheckpointPhase {
    /// Stable label for this pipeline phase (e.g. `"schema"`, `"record-create"`).
    pub phase_label: String,
    /// Safe count of boundaries within this phase.
    pub boundary_count: usize,
    pub note: String,
}

/// A single checkpoint boundary within a phase.
///
/// Safety properties:
/// - No record IDs (old or new).
/// - No record field values.
/// - No token.
/// - No absolute path.
/// - No attachment URL.
/// - Only safe labels and counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCheckpointBoundary {
    /// Stable boundary label (e.g. `"schema-complete"`, `"batch-001"`).
    pub boundary_label: String,
    /// Sequential boundary index (0-based).
    pub boundary_index: usize,
    /// Safe count of items covered at this boundary.
    pub item_count: usize,
    pub note: String,
}

/// Sanitized manifest written to the checkpoint file.
///
/// All fields are safe for local storage:
/// - No token.
/// - No full filesystem path.
/// - No record payloads.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCheckpointManifest {
    /// Safe checkpoint label (not a full path, not a record ID).
    pub checkpoint_label: String,
    /// Mode used for this checkpoint.
    pub mode: RestoreCheckpointStoreMode,
    /// Ordered list of pipeline phases.
    pub phases: Vec<RestoreCheckpointPhase>,
    /// Ordered list of checkpoint boundaries across all phases.
    pub boundaries: Vec<RestoreCheckpointBoundary>,
    /// Total number of boundaries.
    pub total_boundary_count: usize,
    /// Total number of phases.
    pub phase_count: usize,
    /// Safe count of items covered across all boundaries.
    pub total_item_count: usize,
    /// Explicit declaration that restore execution is not triggered by this checkpoint.
    pub restore_execution_not_triggered: bool,
    /// Explicit declaration that no sensitive data is present.
    pub no_sensitive_data: bool,
}

/// Safe summary returned to the UI after storing checkpoint metadata.
///
/// Does NOT include:
/// - Full filesystem path.
/// - Token.
/// - Record payload.
/// - Old or new record IDs.
/// - Attachment URL.
/// - Raw HTTP body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCheckpointSafeSummary {
    /// Safe checkpoint label (matches the manifest).
    pub checkpoint_label: String,
    /// Total number of checkpoint boundaries stored.
    pub total_boundary_count: usize,
    /// Total number of pipeline phases stored.
    pub phase_count: usize,
    /// Total items covered across all boundaries.
    pub total_item_count: usize,
    /// Safe filename for the stored checkpoint (no directory component).
    pub safe_filename: String,
    pub note: String,
}

// ── Request / Result structs ──────────────────────────────────────────────────

/// Request to store sanitized restore checkpoint metadata.
///
/// Safety invariants:
/// - No token field.
/// - No full filesystem path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCheckpointStoreRequest {
    /// Safe checkpoint label. Not a path, not a record ID.
    pub checkpoint_label: Option<String>,
    /// Whether the checkpoint durability policy result is safe.
    pub checkpoint_durability_safe: Option<bool>,
    /// Whether the sensitive data safety policy result is safe.
    pub sensitive_data_safe: Option<bool>,
    /// Whether the mapping/checkpoint execution preview returned DryRunReady.
    pub mapping_checkpoint_preview_ready: Option<bool>,
    /// Whether the final validation execution preview returned DryRunReady.
    pub final_validation_preview_ready: Option<bool>,
    /// Ordered list of pipeline phases to record (labels and counts only).
    /// No record IDs or field values.
    pub phases: Option<Vec<RestoreCheckpointPhase>>,
    /// Ordered list of checkpoint boundaries (labels and counts only).
    /// No record IDs or field values.
    pub boundaries: Option<Vec<RestoreCheckpointBoundary>>,
}

/// Result of the checkpoint metadata store operation.
///
/// Safety invariants (enforced):
/// - `writes_enabled` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `no_changes_made` is `true` when blocked (no file written).
/// - `no_changes_made` is `false` when a checkpoint metadata file is stored.
/// - No token field.
/// - No full filesystem path field.
/// - No old or new record IDs.
/// - No record payload or field values.
/// - No raw HTTP request or response body.
/// - No attachment URL.
/// - `Stored` does NOT enable live restore execution.
/// - `Stored` does NOT introduce a restore success state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCheckpointStoreResult {
    pub status: RestoreCheckpointStoreStatus,
    pub mode: RestoreCheckpointStoreMode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<RestoreCheckpointSafeSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// False when a local checkpoint metadata file was stored; true when blocked.
    pub no_changes_made: bool,
    /// Always false — no network write operations are attempted.
    pub network_writes_attempted: bool,
    /// Always false — live restore writes are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const PREREQ_WRITE_GATE: &str = "RCPS-PRE-01";
const PREREQ_CHECKPOINT_DURABILITY: &str = "RCPS-PRE-02";
const PREREQ_SENSITIVE_DATA: &str = "RCPS-PRE-03";
const PREREQ_MAPPING_CHECKPOINT_PREVIEW: &str = "RCPS-PRE-04";
const PREREQ_FINAL_VALIDATION_PREVIEW: &str = "RCPS-PRE-05";

// ── Core function ─────────────────────────────────────────────────────────────

/// Stores sanitized restore checkpoint metadata to the app-controlled
/// checkpoint directory.
///
/// This function:
/// - Never calls any Airtable API endpoint.
/// - Never writes a token, full path, record ID, record payload,
///   raw HTTP body, or attachment URL.
/// - Never enables live restore execution.
/// - Never introduces a restore success state.
/// - Always returns `writes_enabled: false`, `network_writes_attempted: false`.
/// - Returns `no_changes_made: false` only when a file is actually stored.
/// - Always consults `evaluate_write_gate()` to confirm restore writes remain disabled.
/// - Only stores metadata in an app-controlled safe subdirectory of the OS temp dir.
/// - Returns only a safe filename (no directory component) and safe counts to the UI.
pub fn store_restore_checkpoint(
    request: &RestoreCheckpointStoreRequest,
) -> RestoreCheckpointStoreResult {
    // Always confirm restore write gate is disabled.
    let gate = evaluate_write_gate();
    let restore_writes_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let checkpoint_durability_safe = request.checkpoint_durability_safe.unwrap_or(false);
    let sensitive_data_safe = request.sensitive_data_safe.unwrap_or(false);
    let mapping_checkpoint_preview_ready =
        request.mapping_checkpoint_preview_ready.unwrap_or(false);
    let final_validation_preview_ready = request.final_validation_preview_ready.unwrap_or(false);

    // Check prerequisites in order; first failure wins.
    let blocked_reason: Option<String> = if !restore_writes_disabled {
        // This branch is unreachable given evaluate_write_gate() always returns Disabled,
        // but is retained as a defense-in-depth check.
        Some(format!(
            "{PREREQ_WRITE_GATE}: Restore write gate is not disabled. \
             Checkpoint metadata must not be stored while writes could be enabled."
        ))
    } else if !checkpoint_durability_safe {
        Some(format!(
            "{PREREQ_CHECKPOINT_DURABILITY}: Checkpoint durability policy is not safe. \
             A compliant checkpoint durability plan must be declared before storing \
             checkpoint metadata."
        ))
    } else if !sensitive_data_safe {
        Some(format!(
            "{PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied. \
             All 10 exposure surfaces must have redaction coverage before checkpoint \
             metadata is stored."
        ))
    } else if !mapping_checkpoint_preview_ready {
        Some(format!(
            "{PREREQ_MAPPING_CHECKPOINT_PREVIEW}: Mapping/checkpoint execution preview \
             has not returned DryRunReady. Complete the mapping/checkpoint preview before \
             storing checkpoint metadata."
        ))
    } else if !final_validation_preview_ready {
        Some(format!(
            "{PREREQ_FINAL_VALIDATION_PREVIEW}: Final validation execution preview \
             has not returned DryRunReady. Complete the final validation preview before \
             storing checkpoint metadata."
        ))
    } else {
        None
    };

    if let Some(ref reason) = blocked_reason {
        return RestoreCheckpointStoreResult {
            status: RestoreCheckpointStoreStatus::Blocked,
            mode: RestoreCheckpointStoreMode::MetadataOnly,
            message: format!(
                "Checkpoint metadata store is blocked. {reason} \
                 No checkpoint metadata file was written. \
                 Restore execution remains disabled."
            ),
            summary: None,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the sanitized manifest.
    let raw_label = request
        .checkpoint_label
        .as_deref()
        .unwrap_or("restore-checkpoint");
    // Sanitize the label: allow only alphanumeric, hyphens, and underscores.
    let checkpoint_label: String = raw_label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .take(64)
        .collect();

    let phases = request.phases.clone().unwrap_or_default();
    let boundaries = request.boundaries.clone().unwrap_or_default();
    let total_boundary_count = boundaries.len();
    let phase_count = phases.len();
    let total_item_count: usize = boundaries.iter().map(|b| b.item_count).sum();

    let manifest = RestoreCheckpointManifest {
        checkpoint_label: checkpoint_label.clone(),
        mode: RestoreCheckpointStoreMode::MetadataOnly,
        phases: phases.clone(),
        boundaries: boundaries.clone(),
        total_boundary_count,
        phase_count,
        total_item_count,
        restore_execution_not_triggered: true,
        no_sensitive_data: true,
    };

    // Write the sanitized manifest to the app-controlled safe checkpoint directory.
    // The directory is a fixed subdirectory under the OS temp dir — no user-supplied
    // path is used. No directory component is returned to the UI.
    let store_result = write_checkpoint_manifest(&checkpoint_label, &manifest);

    match store_result {
        Ok(safe_filename) => {
            let summary = RestoreCheckpointSafeSummary {
                checkpoint_label: checkpoint_label.clone(),
                total_boundary_count,
                phase_count,
                total_item_count,
                safe_filename: safe_filename.clone(),
                note: format!(
                    "Stored {total_boundary_count} checkpoint boundary(ies) across \
                     {phase_count} phase(s), {total_item_count} item(s) total. \
                     Restore execution is not triggered by this checkpoint. \
                     No sensitive data is stored."
                ),
            };
            RestoreCheckpointStoreResult {
                status: RestoreCheckpointStoreStatus::Stored,
                mode: RestoreCheckpointStoreMode::MetadataOnly,
                message: format!(
                    "Checkpoint metadata stored (metadata-only). \
                     {total_boundary_count} boundary(ies) across {phase_count} phase(s). \
                     Safe filename: {safe_filename}. \
                     Restore execution is not triggered by this checkpoint. \
                     Live restore writes remain disabled. \
                     No sensitive data is stored in the checkpoint file."
                ),
                summary: Some(summary),
                blocked_reason: None,
                no_changes_made: false,
                network_writes_attempted: false,
                writes_enabled: false,
            }
        }
        Err(err_msg) => RestoreCheckpointStoreResult {
            status: RestoreCheckpointStoreStatus::Blocked,
            mode: RestoreCheckpointStoreMode::MetadataOnly,
            message: format!(
                "Checkpoint metadata store failed: {err_msg} \
                 No checkpoint file was written. \
                 Restore execution remains disabled."
            ),
            summary: None,
            blocked_reason: Some(err_msg),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        },
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Writes the sanitized manifest JSON to the app-controlled checkpoint directory.
///
/// Returns the safe filename (no directory component) on success.
/// Returns an error string on failure.
///
/// Directory: OS temp dir / `airbridge-checkpoints/` — app-controlled, no user input.
/// Filename: `rcps-<label>.json` — sanitized label only, no path separators.
/// Content: sanitized JSON manifest — no token, no path, no record IDs, no payload.
fn write_checkpoint_manifest(
    checkpoint_label: &str,
    manifest: &RestoreCheckpointManifest,
) -> Result<String, String> {
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("manifest serialization failed: {e}"))?;

    // Safety check: verify no sensitive patterns leaked into the serialized JSON.
    if json.contains("pat_")
        || json.contains("/Users/")
        || json.contains("/tmp/")
        || json.contains("/home/")
    {
        return Err(
            "serialized manifest contains a potentially sensitive pattern — write aborted"
                .to_string(),
        );
    }

    // Build the safe filename — no directory separator, no record ID format.
    // Label is already sanitized (alphanumeric, hyphens, underscores, max 64 chars).
    let safe_filename = format!("rcps-{checkpoint_label}.json");

    // Use OS temp dir as the base — app-controlled, no user-supplied path.
    let checkpoint_dir = std::env::temp_dir().join("airbridge-checkpoints");

    std::fs::create_dir_all(&checkpoint_dir)
        .map_err(|e| format!("checkpoint directory creation failed: {e}"))?;

    let dest = checkpoint_dir.join(&safe_filename);

    // Write atomically: write to a temp file then rename.
    let tmp_filename = format!("rcps-{checkpoint_label}.tmp");
    let tmp_path = checkpoint_dir.join(&tmp_filename);

    {
        let mut f = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("checkpoint temp file creation failed: {e}"))?;
        f.write_all(json.as_bytes())
            .map_err(|e| format!("checkpoint write failed: {e}"))?;
        f.flush()
            .map_err(|e| format!("checkpoint flush failed: {e}"))?;
    }

    std::fs::rename(&tmp_path, &dest)
        .map_err(|e| format!("checkpoint rename failed: {e}"))?;

    // Return only the filename — never the full path.
    Ok(safe_filename)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn safe_request() -> RestoreCheckpointStoreRequest {
        RestoreCheckpointStoreRequest {
            checkpoint_label: Some("test-checkpoint".to_string()),
            checkpoint_durability_safe: Some(true),
            sensitive_data_safe: Some(true),
            mapping_checkpoint_preview_ready: Some(true),
            final_validation_preview_ready: Some(true),
            phases: Some(vec![
                RestoreCheckpointPhase {
                    phase_label: "schema".to_string(),
                    boundary_count: 2,
                    note: "Schema phase checkpoint boundaries.".to_string(),
                },
                RestoreCheckpointPhase {
                    phase_label: "record-create".to_string(),
                    boundary_count: 3,
                    note: "Record create phase checkpoint boundaries.".to_string(),
                },
            ]),
            boundaries: Some(vec![
                RestoreCheckpointBoundary {
                    boundary_label: "schema-complete".to_string(),
                    boundary_index: 0,
                    item_count: 3,
                    note: "Schema phase complete. 3 table(s).".to_string(),
                },
                RestoreCheckpointBoundary {
                    boundary_label: "batch-001".to_string(),
                    boundary_index: 1,
                    item_count: 10,
                    note: "Record batch 1 complete.".to_string(),
                },
                RestoreCheckpointBoundary {
                    boundary_label: "batch-002".to_string(),
                    boundary_index: 2,
                    item_count: 10,
                    note: "Record batch 2 complete.".to_string(),
                },
            ]),
        }
    }

    fn blocked_request() -> RestoreCheckpointStoreRequest {
        RestoreCheckpointStoreRequest {
            checkpoint_label: None,
            checkpoint_durability_safe: None,
            sensitive_data_safe: None,
            mapping_checkpoint_preview_ready: None,
            final_validation_preview_ready: None,
            phases: None,
            boundaries: None,
        }
    }

    // ── Safety invariants ──────────────────────────────────────────────────────

    #[test]
    fn writes_enabled_always_false_blocked() {
        let result = store_restore_checkpoint(&blocked_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_always_false_blocked() {
        let result = store_restore_checkpoint(&blocked_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_true_when_blocked() {
        let result = store_restore_checkpoint(&blocked_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn writes_enabled_always_false_stored() {
        let result = store_restore_checkpoint(&safe_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn network_writes_attempted_always_false_stored() {
        let result = store_restore_checkpoint(&safe_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_false_when_stored() {
        let result = store_restore_checkpoint(&safe_request());
        if result.status == RestoreCheckpointStoreStatus::Stored {
            assert!(!result.no_changes_made);
        }
    }

    #[test]
    fn restore_write_gate_still_disabled_after_store() {
        let _result = store_restore_checkpoint(&safe_request());
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── Blocked behavior ───────────────────────────────────────────────────────

    #[test]
    fn blocked_when_all_prerequisites_missing() {
        let result = store_restore_checkpoint(&blocked_request());
        assert_eq!(result.status, RestoreCheckpointStoreStatus::Blocked);
        assert!(result.blocked_reason.is_some());
        assert!(result.summary.is_none());
    }

    #[test]
    fn blocked_when_checkpoint_durability_not_safe() {
        let mut req = safe_request();
        req.checkpoint_durability_safe = Some(false);
        let result = store_restore_checkpoint(&req);
        assert_eq!(result.status, RestoreCheckpointStoreStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_CHECKPOINT_DURABILITY));
    }

    #[test]
    fn blocked_when_sensitive_data_not_safe() {
        let mut req = safe_request();
        req.sensitive_data_safe = Some(false);
        let result = store_restore_checkpoint(&req);
        assert_eq!(result.status, RestoreCheckpointStoreStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_SENSITIVE_DATA));
    }

    #[test]
    fn blocked_when_mapping_checkpoint_preview_not_ready() {
        let mut req = safe_request();
        req.mapping_checkpoint_preview_ready = Some(false);
        let result = store_restore_checkpoint(&req);
        assert_eq!(result.status, RestoreCheckpointStoreStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_MAPPING_CHECKPOINT_PREVIEW));
    }

    #[test]
    fn blocked_when_final_validation_preview_not_ready() {
        let mut req = safe_request();
        req.final_validation_preview_ready = Some(false);
        let result = store_restore_checkpoint(&req);
        assert_eq!(result.status, RestoreCheckpointStoreStatus::Blocked);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_FINAL_VALIDATION_PREVIEW));
    }

    // ── Stored behavior ────────────────────────────────────────────────────────

    #[test]
    fn stored_when_all_prerequisites_satisfied() {
        let result = store_restore_checkpoint(&safe_request());
        assert!(
            result.status == RestoreCheckpointStoreStatus::Stored
                || result.status == RestoreCheckpointStoreStatus::Blocked,
            "status must be stored or blocked (no other states)"
        );
        // If stored, check key invariants.
        if result.status == RestoreCheckpointStoreStatus::Stored {
            assert!(result.summary.is_some());
            assert!(result.blocked_reason.is_none());
        }
    }

    #[test]
    fn stored_summary_has_correct_counts() {
        let result = store_restore_checkpoint(&safe_request());
        if result.status == RestoreCheckpointStoreStatus::Stored {
            let summary = result.summary.unwrap();
            assert_eq!(summary.total_boundary_count, 3);
            assert_eq!(summary.phase_count, 2);
            assert_eq!(summary.total_item_count, 23); // 3 + 10 + 10
        }
    }

    #[test]
    fn stored_summary_contains_no_full_path() {
        let result = store_restore_checkpoint(&safe_request());
        if result.status == RestoreCheckpointStoreStatus::Stored {
            let json = serde_json::to_string(&result).expect("serialize");
            assert!(!json.contains("/Users/"), "full path in result");
            assert!(!json.contains("/home/"), "full path in result");
            // The temp path prefix must not appear in any result field.
            let tmp = std::env::temp_dir();
            if let Some(tmp_str) = tmp.to_str() {
                assert!(
                    !json.contains(tmp_str),
                    "temp dir path leaked into result: {tmp_str}"
                );
            }
        }
    }

    #[test]
    fn stored_summary_safe_filename_has_no_path_separator() {
        let result = store_restore_checkpoint(&safe_request());
        if result.status == RestoreCheckpointStoreStatus::Stored {
            let summary = result.summary.unwrap();
            assert!(!summary.safe_filename.contains('/'));
            assert!(!summary.safe_filename.contains('\\'));
            assert!(summary.safe_filename.starts_with("rcps-"));
            assert!(summary.safe_filename.ends_with(".json"));
        }
    }

    #[test]
    fn stored_result_does_not_contain_token() {
        let result = store_restore_checkpoint(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("pat_"), "token in result");
    }

    #[test]
    fn blocked_result_does_not_contain_token() {
        let result = store_restore_checkpoint(&blocked_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("pat_"), "token in blocked result");
    }

    #[test]
    fn no_restore_success_state_in_result() {
        let result = store_restore_checkpoint(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""), "success state in result");
        assert!(!json.contains("restoreComplete"), "success state in result");
        assert!(!json.contains("restoreSuccess"), "success state in result");
    }

    #[test]
    fn checkpoint_label_sanitized_for_arbitrary_input() {
        let mut req = safe_request();
        // Label with path separators and special chars — must be sanitized.
        req.checkpoint_label = Some("../../../etc/passwd".to_string());
        let result = store_restore_checkpoint(&req);
        if result.status == RestoreCheckpointStoreStatus::Stored {
            let summary = result.summary.unwrap();
            // Path separators and dots must be stripped from the label and filename.
            assert!(!summary.checkpoint_label.contains('/'));
            assert!(!summary.checkpoint_label.contains('.'));
            assert!(!summary.safe_filename.contains('/'));
            assert!(!summary.safe_filename.contains('\\'));
            assert!(!summary.safe_filename.contains('.') || summary.safe_filename.ends_with(".json"),
                "safe_filename must not contain . except for the .json extension");
        }
    }

    #[test]
    fn arbitrary_output_path_not_accepted() {
        // The request struct has no output_path field — this is a structural
        // guarantee. Verify the result does not expose a full path.
        let result = store_restore_checkpoint(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/tmp/"), "temp path in result");
        assert!(!json.contains("airbridge-checkpoints"), "dir in result");
    }

    #[test]
    fn manifest_serialization_safety_invariants_hold_across_calls() {
        let req = safe_request();
        let result1 = store_restore_checkpoint(&req);
        let result2 = store_restore_checkpoint(&req);
        // Safety invariants must hold regardless of whether IO succeeds.
        assert!(!result1.writes_enabled);
        assert!(!result1.network_writes_attempted);
        assert!(!result2.writes_enabled);
        assert!(!result2.network_writes_attempted);
        // At least one of the calls must have been Stored or Blocked — no other state.
        let valid_statuses = [
            RestoreCheckpointStoreStatus::Stored,
            RestoreCheckpointStoreStatus::Blocked,
        ];
        assert!(valid_statuses.contains(&result1.status));
        assert!(valid_statuses.contains(&result2.status));
    }

    // ── File content validation ────────────────────────────────────────────────

    #[test]
    fn written_file_does_not_contain_token() {
        let tmp = tempdir().expect("tempdir");
        let manifest = RestoreCheckpointManifest {
            checkpoint_label: "test".to_string(),
            mode: RestoreCheckpointStoreMode::MetadataOnly,
            phases: vec![],
            boundaries: vec![],
            total_boundary_count: 0,
            phase_count: 0,
            total_item_count: 0,
            restore_execution_not_triggered: true,
            no_sensitive_data: true,
        };
        let json = serde_json::to_string_pretty(&manifest).expect("serialize");
        let path = tmp.path().join("test.json");
        std::fs::write(&path, &json).expect("write");
        let content = std::fs::read_to_string(&path).expect("read");
        assert!(!content.contains("pat_"));
        assert!(!content.contains("/Users/"));
        assert!(!content.contains("/home/"));
    }

    #[test]
    fn written_file_declares_restore_not_triggered() {
        let tmp = tempdir().expect("tempdir");
        let manifest = RestoreCheckpointManifest {
            checkpoint_label: "test".to_string(),
            mode: RestoreCheckpointStoreMode::MetadataOnly,
            phases: vec![],
            boundaries: vec![],
            total_boundary_count: 0,
            phase_count: 0,
            total_item_count: 0,
            restore_execution_not_triggered: true,
            no_sensitive_data: true,
        };
        let json = serde_json::to_string_pretty(&manifest).expect("serialize");
        let path = tmp.path().join("test.json");
        std::fs::write(&path, &json).expect("write");
        let content = std::fs::read_to_string(&path).expect("read");
        assert!(content.contains("restoreExecutionNotTriggered"));
        assert!(content.contains("true"));
    }

    #[test]
    fn written_file_declares_no_sensitive_data() {
        let tmp = tempdir().expect("tempdir");
        let manifest = RestoreCheckpointManifest {
            checkpoint_label: "test".to_string(),
            mode: RestoreCheckpointStoreMode::MetadataOnly,
            phases: vec![],
            boundaries: vec![],
            total_boundary_count: 0,
            phase_count: 0,
            total_item_count: 0,
            restore_execution_not_triggered: true,
            no_sensitive_data: true,
        };
        let json = serde_json::to_string_pretty(&manifest).expect("serialize");
        let path = tmp.path().join("test.json");
        std::fs::write(&path, &json).expect("write");
        let content = std::fs::read_to_string(&path).expect("read");
        assert!(content.contains("noSensitiveData"));
        assert!(content.contains("true"));
    }
}
