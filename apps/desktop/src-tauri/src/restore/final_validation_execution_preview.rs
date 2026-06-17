use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the final validation execution preview.
///
/// Safety invariants:
/// - `DryRunReady` does NOT enable live final validation execution.
/// - `writes_enabled` is always `false`.
/// - No Airtable API calls are made by this module.
/// - No token, full path, record payload, raw HTTP, old/new record IDs,
///   or attachment URL appears in any result field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationExecutionPreviewStatus {
    /// All safety prerequisites are present; a dry-run final validation preview
    /// is available. Live validation execution remains disabled and no restore
    /// execution is started.
    DryRunReady,
    /// At least one required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Status of a single validation check in the preview.
///
/// Note: `succeeded` / `completed` / `executed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationExecutionPreviewCheckStatus {
    /// The check is planned and would execute if validation were enabled.
    Pending,
    /// The check is blocked by a safety prerequisite.
    Blocked,
    /// The check is skipped because a prerequisite failed.
    Skipped,
}

/// Execution mode for the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationExecutionPreviewMode {
    /// Dry-run only — no live execution path is reachable.
    DryRunOnly,
    /// Live final validation execution is blocked by product policy.
    LiveBlocked,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single ordered validation check in the final validation execution preview.
///
/// Safety properties:
/// - No old or new record IDs.
/// - No raw record field values.
/// - No raw request/response body.
/// - No token.
/// - No absolute path.
/// - No attachment URL.
/// - Only safe counts, labels, and check identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationExecutionPreviewCheck {
    /// Stable check identifier (e.g. `FVEP-CHK-SCHEMA`).
    pub check_id: String,
    /// Human-readable label for display.
    pub label: String,
    pub status: FinalValidationExecutionPreviewCheckStatus,
    /// Safe count — the number of items this check would inspect.
    pub expected_count: usize,
    pub note: String,
}

/// Point-in-time safety snapshot for the final validation execution preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationExecutionSafetySnapshot {
    pub write_gate_disabled: bool,
    pub schema_write_preview_ready: bool,
    pub record_write_preview_ready: bool,
    pub mapping_checkpoint_preview_ready: bool,
    pub linked_second_pass_preview_ready: bool,
    pub final_validation_policy_safe: bool,
    pub final_validation_enforcement_policy_safe: bool,
    pub sensitive_data_safe: bool,
    pub attachment_phase_disabled_safe: bool,
    pub live_write_readiness_satisfied: bool,
}

/// Safe summary of the final validation execution preview.
///
/// No sensitive values — no token, path, or record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationExecutionPreviewSummary {
    /// Total number of checks that would run.
    pub total_check_count: usize,
    /// Number of checks in `pending` state.
    pub pending_check_count: usize,
    /// Number of checks in `blocked` or `skipped` state.
    pub non_pending_check_count: usize,
    /// Safe count of tables to be validated.
    pub table_count: usize,
    /// Safe count of fields to be validated.
    pub field_count: usize,
    /// Safe count of records to be counted.
    pub record_count: usize,
    /// Safe count of ID mapping entries to be validated.
    pub id_mapping_entry_count: usize,
    /// Safe count of linked field coverage entries to be validated.
    pub linked_coverage_count: usize,
    /// Safe count of attachment metadata entries to be validated.
    pub attachment_metadata_count: usize,
    /// Whether a package manifest is present.
    pub manifest_present: bool,
    pub note: String,
}

/// Request for the final validation execution preview.
///
/// No token field — token is not required or accepted.
/// No full path field — filename label only.
/// No raw record payloads — only counts and flags.
/// No old or new record IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationExecutionPreviewRequest {
    /// Filename label for the backup package. No directory component.
    pub package_filename: Option<String>,
    /// Whether the schema write execution preview returned DryRunReady.
    pub schema_write_preview_ready: Option<bool>,
    /// Whether the record write execution preview returned DryRunReady.
    pub record_write_preview_ready: Option<bool>,
    /// Whether the mapping/checkpoint execution preview returned DryRunReady.
    pub mapping_checkpoint_preview_ready: Option<bool>,
    /// Whether the linked second-pass execution preview returned DryRunReady.
    pub linked_second_pass_preview_ready: Option<bool>,
    /// Whether the final validation policy result is safe (compliant or warning).
    pub final_validation_policy_safe: Option<bool>,
    /// Whether the final validation enforcement policy result is safe (compliant or warning).
    pub final_validation_enforcement_policy_safe: Option<bool>,
    /// Whether the sensitive data safety policy result is safe.
    pub sensitive_data_safe: Option<bool>,
    /// Whether the attachment phase disabled policy result is safe.
    pub attachment_phase_disabled_safe: Option<bool>,
    /// Whether the live-write readiness result is ready or warning (advisory only).
    pub live_write_readiness_satisfied: Option<bool>,
    /// Safe count of tables to be validated. No raw table IDs.
    pub table_count: Option<usize>,
    /// Safe count of fields to be validated. No raw field IDs.
    pub field_count: Option<usize>,
    /// Safe count of records to be counted. No raw record IDs.
    pub record_count: Option<usize>,
    /// Safe count of ID mapping entries. No raw record IDs.
    pub id_mapping_entry_count: Option<usize>,
    /// Safe count of linked field coverage entries. No raw IDs.
    pub linked_coverage_count: Option<usize>,
    /// Safe count of attachment metadata entries.
    pub attachment_metadata_count: Option<usize>,
    /// Whether a package manifest is present.
    pub manifest_present: Option<bool>,
}

/// Result of the final validation execution preview.
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
/// - `DryRunReady` does NOT enable live final validation execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationExecutionPreviewResult {
    pub status: FinalValidationExecutionPreviewStatus,
    pub mode: FinalValidationExecutionPreviewMode,
    pub message: String,
    pub checks: Vec<FinalValidationExecutionPreviewCheck>,
    pub summary: FinalValidationExecutionPreviewSummary,
    pub safety_snapshot: FinalValidationExecutionSafetySnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always false — live final validation execution is not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const PREREQ_WRITE_GATE: &str = "FVEP-PRE-01";
const PREREQ_SCHEMA_WRITE_PREVIEW: &str = "FVEP-PRE-02";
const PREREQ_RECORD_WRITE_PREVIEW: &str = "FVEP-PRE-03";
const PREREQ_MAPPING_CHECKPOINT_PREVIEW: &str = "FVEP-PRE-04";
const PREREQ_LINKED_SECOND_PASS_PREVIEW: &str = "FVEP-PRE-05";
const PREREQ_FINAL_VALIDATION_POLICY: &str = "FVEP-PRE-06";
const PREREQ_FINAL_VALIDATION_ENFORCEMENT_POLICY: &str = "FVEP-PRE-07";
const PREREQ_SENSITIVE_DATA: &str = "FVEP-PRE-08";
const PREREQ_ATTACHMENT_PHASE_DISABLED: &str = "FVEP-PRE-09";
const PREREQ_LWR: &str = "FVEP-PRE-10";

// ── Check IDs ─────────────────────────────────────────────────────────────────

const CHK_SCHEMA: &str = "FVEP-CHK-SCHEMA";
const CHK_FIELDS: &str = "FVEP-CHK-FIELDS";
const CHK_RECORDS: &str = "FVEP-CHK-RECORDS";
const CHK_MAPPING: &str = "FVEP-CHK-MAPPING";
const CHK_LINKED: &str = "FVEP-CHK-LINKED";
const CHK_ATTACH: &str = "FVEP-CHK-ATTACH";
const CHK_MANIFEST: &str = "FVEP-CHK-MANIFEST";
const CHK_GUARD: &str = "FVEP-CHK-GUARD";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds a final validation execution preview from a request.
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
/// - A `DryRunReady` result does NOT enable live final validation execution.
pub fn preview_final_validation_execution(
    request: &FinalValidationExecutionPreviewRequest,
) -> FinalValidationExecutionPreviewResult {
    // Always consult the write gate first.
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let schema_write_preview_ready = request.schema_write_preview_ready.unwrap_or(false);
    let record_write_preview_ready = request.record_write_preview_ready.unwrap_or(false);
    let mapping_checkpoint_preview_ready =
        request.mapping_checkpoint_preview_ready.unwrap_or(false);
    let linked_second_pass_preview_ready =
        request.linked_second_pass_preview_ready.unwrap_or(false);
    let final_validation_policy_safe = request.final_validation_policy_safe.unwrap_or(false);
    let final_validation_enforcement_policy_safe = request
        .final_validation_enforcement_policy_safe
        .unwrap_or(false);
    let sensitive_data_safe = request.sensitive_data_safe.unwrap_or(false);
    let attachment_phase_disabled_safe = request.attachment_phase_disabled_safe.unwrap_or(false);
    let live_write_readiness_satisfied = request.live_write_readiness_satisfied.unwrap_or(false);

    let snapshot = FinalValidationExecutionSafetySnapshot {
        write_gate_disabled,
        schema_write_preview_ready,
        record_write_preview_ready,
        mapping_checkpoint_preview_ready,
        linked_second_pass_preview_ready,
        final_validation_policy_safe,
        final_validation_enforcement_policy_safe,
        sensitive_data_safe,
        attachment_phase_disabled_safe,
        live_write_readiness_satisfied,
    };

    // Check prerequisites in order; first failure wins.
    let blocked_reason: Option<String> = if !write_gate_disabled {
        Some(format!(
            "{PREREQ_WRITE_GATE}: Write gate is not disabled. \
             Live final validation execution must not be attempted."
        ))
    } else if !schema_write_preview_ready {
        Some(format!(
            "{PREREQ_SCHEMA_WRITE_PREVIEW}: Schema write execution preview has not returned \
             DryRunReady. Complete the schema write preview before requesting a \
             final validation preview."
        ))
    } else if !record_write_preview_ready {
        Some(format!(
            "{PREREQ_RECORD_WRITE_PREVIEW}: Record write execution preview has not returned \
             DryRunReady. Complete the record write preview before requesting a \
             final validation preview."
        ))
    } else if !mapping_checkpoint_preview_ready {
        Some(format!(
            "{PREREQ_MAPPING_CHECKPOINT_PREVIEW}: Mapping/checkpoint execution preview has not \
             returned DryRunReady. Complete the mapping/checkpoint preview before requesting a \
             final validation preview."
        ))
    } else if !linked_second_pass_preview_ready {
        Some(format!(
            "{PREREQ_LINKED_SECOND_PASS_PREVIEW}: Linked second-pass execution preview has not \
             returned DryRunReady. Complete the linked second-pass preview before requesting a \
             final validation preview."
        ))
    } else if !final_validation_policy_safe {
        Some(format!(
            "{PREREQ_FINAL_VALIDATION_POLICY}: Final validation policy is not safe. \
             All required validation steps must be declared."
        ))
    } else if !final_validation_enforcement_policy_safe {
        Some(format!(
            "{PREREQ_FINAL_VALIDATION_ENFORCEMENT_POLICY}: Final validation enforcement policy \
             is not safe. All completion guards must be declared and compliant."
        ))
    } else if !sensitive_data_safe {
        Some(format!(
            "{PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied. \
             All 10 exposure surfaces must have redaction coverage."
        ))
    } else if !attachment_phase_disabled_safe {
        Some(format!(
            "{PREREQ_ATTACHMENT_PHASE_DISABLED}: Attachment phase disabled policy is not safe. \
             Attachment operations must be confirmed disabled."
        ))
    } else if !live_write_readiness_satisfied {
        Some(format!(
            "{PREREQ_LWR}: Live write readiness policy is not satisfied. \
             All required safety gates must be declared."
        ))
    } else {
        None
    };

    let table_count = request.table_count.unwrap_or(0);
    let field_count = request.field_count.unwrap_or(0);
    let record_count = request.record_count.unwrap_or(0);
    let id_mapping_entry_count = request.id_mapping_entry_count.unwrap_or(0);
    let linked_coverage_count = request.linked_coverage_count.unwrap_or(0);
    let attachment_metadata_count = request.attachment_metadata_count.unwrap_or(0);
    let manifest_present = request.manifest_present.unwrap_or(false);

    let empty_summary = FinalValidationExecutionPreviewSummary {
        total_check_count: 8,
        pending_check_count: 0,
        non_pending_check_count: 8,
        table_count: 0,
        field_count: 0,
        record_count: 0,
        id_mapping_entry_count: 0,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: false,
        note: "Final validation preview unavailable — prerequisites not satisfied.".to_string(),
    };

    if let Some(ref reason) = blocked_reason {
        let blocked_check = FinalValidationExecutionPreviewCheck {
            check_id: "FVEP-BLOCKED".to_string(),
            label: "prerequisites-blocked".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Blocked,
            expected_count: 0,
            note: "Safety prerequisites not satisfied. \
                   No final validation checks can be previewed."
                .to_string(),
        };
        return FinalValidationExecutionPreviewResult {
            status: FinalValidationExecutionPreviewStatus::Blocked,
            mode: FinalValidationExecutionPreviewMode::LiveBlocked,
            message: format!(
                "Final validation execution preview is blocked. {reason} \
                 Live final validation execution remains disabled."
            ),
            checks: vec![blocked_check],
            summary: empty_summary,
            safety_snapshot: snapshot,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the dry-run preview checks.
    let checks = build_preview_checks(
        table_count,
        field_count,
        record_count,
        id_mapping_entry_count,
        linked_coverage_count,
        attachment_metadata_count,
        manifest_present,
    );

    let pending_count = checks
        .iter()
        .filter(|c| c.status == FinalValidationExecutionPreviewCheckStatus::Pending)
        .count();
    let non_pending_count = checks.len() - pending_count;

    let summary = FinalValidationExecutionPreviewSummary {
        total_check_count: checks.len(),
        pending_check_count: pending_count,
        non_pending_check_count: non_pending_count,
        table_count,
        field_count,
        record_count,
        id_mapping_entry_count,
        linked_coverage_count,
        attachment_metadata_count,
        manifest_present,
        note: format!(
            "Would validate {table_count} table(s), {field_count} field(s), \
             {record_count} record count(s), {id_mapping_entry_count} ID mapping \
             entry(ies), {linked_coverage_count} linked field coverage entry(ies), \
             {attachment_metadata_count} attachment metadata entry(ies). \
             Manifest checksum validation {}. \
             No raw record IDs present in this preview.",
            if manifest_present {
                "is included"
            } else {
                "is not applicable (no manifest)"
            }
        ),
    };

    FinalValidationExecutionPreviewResult {
        status: FinalValidationExecutionPreviewStatus::DryRunReady,
        mode: FinalValidationExecutionPreviewMode::DryRunOnly,
        message: format!(
            "Final validation execution preview is ready (dry-run only). \
             {} check(s) planned: {table_count} table(s), {field_count} field(s), \
             {record_count} record count(s), {id_mapping_entry_count} mapping \
             entry(ies), {linked_coverage_count} linked field(s), \
             {attachment_metadata_count} attachment entry(ies). \
             Manifest checksum validation {}. \
             Live final validation execution remains disabled. \
             This preview does not start any restore execution. \
             No checkpoint files are written. \
             No record IDs are present in this preview.",
            checks.len(),
            if manifest_present {
                "included"
            } else {
                "not applicable (no manifest)"
            }
        ),
        checks,
        summary,
        safety_snapshot: snapshot,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_preview_checks(
    table_count: usize,
    field_count: usize,
    record_count: usize,
    id_mapping_entry_count: usize,
    linked_coverage_count: usize,
    attachment_metadata_count: usize,
    manifest_present: bool,
) -> Vec<FinalValidationExecutionPreviewCheck> {
    vec![
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_SCHEMA.to_string(),
            label: "schema-table-count".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Pending,
            expected_count: table_count,
            note: format!(
                "Would verify {table_count} table(s) are present in the restored base \
                 and match the backup manifest. No raw table IDs in this preview."
            ),
        },
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_FIELDS.to_string(),
            label: "field-count".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Pending,
            expected_count: field_count,
            note: format!(
                "Would verify {field_count} field(s) are present across all restored tables \
                 and match the backup manifest. No raw field IDs in this preview."
            ),
        },
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_RECORDS.to_string(),
            label: "record-count".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Pending,
            expected_count: record_count,
            note: format!(
                "Would verify {record_count} record(s) are present in the restored base \
                 and match the backup manifest. No raw record IDs in this preview."
            ),
        },
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_MAPPING.to_string(),
            label: "id-mapping-coverage".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Pending,
            expected_count: id_mapping_entry_count,
            note: format!(
                "Would verify {id_mapping_entry_count} old-to-new ID mapping entry(ies) \
                 are complete and cover all restored records. \
                 No raw record IDs in this preview."
            ),
        },
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_LINKED.to_string(),
            label: "linked-record-coverage".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Pending,
            expected_count: linked_coverage_count,
            note: format!(
                "Would verify {linked_coverage_count} linked field(s) have been updated \
                 in the second pass and all target record references are resolvable. \
                 No raw record IDs in this preview."
            ),
        },
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_ATTACH.to_string(),
            label: "attachment-metadata-only".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Pending,
            expected_count: attachment_metadata_count,
            note: format!(
                "Would verify {attachment_metadata_count} attachment metadata entry(ies) \
                 are present in the restored base (metadata only — no file download). \
                 No raw attachment URLs in this preview."
            ),
        },
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_MANIFEST.to_string(),
            label: "manifest-checksum-reference".to_string(),
            status: if manifest_present {
                FinalValidationExecutionPreviewCheckStatus::Pending
            } else {
                FinalValidationExecutionPreviewCheckStatus::Skipped
            },
            expected_count: if manifest_present { 1 } else { 0 },
            note: if manifest_present {
                "Would verify the restored base references match the package manifest \
                 checksum and all declared entries are accounted for. \
                 No raw paths or tokens in this preview."
                    .to_string()
            } else {
                "No package manifest present — manifest/checksum validation is not applicable \
                 for this restore package."
                    .to_string()
            },
        },
        FinalValidationExecutionPreviewCheck {
            check_id: CHK_GUARD.to_string(),
            label: "final-completion-guard".to_string(),
            status: FinalValidationExecutionPreviewCheckStatus::Pending,
            expected_count: 1,
            note: "Would verify the completion guard is active and no result status may be \
                   set to any complete/success equivalent without all prior validation checks \
                   having passed. Restore writes remain disabled."
                .to_string(),
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_request() -> FinalValidationExecutionPreviewRequest {
        FinalValidationExecutionPreviewRequest {
            package_filename: Some("test-backup.airbridge".to_string()),
            schema_write_preview_ready: Some(true),
            record_write_preview_ready: Some(true),
            mapping_checkpoint_preview_ready: Some(true),
            linked_second_pass_preview_ready: Some(true),
            final_validation_policy_safe: Some(true),
            final_validation_enforcement_policy_safe: Some(true),
            sensitive_data_safe: Some(true),
            attachment_phase_disabled_safe: Some(true),
            live_write_readiness_satisfied: Some(true),
            table_count: Some(3),
            field_count: Some(12),
            record_count: Some(150),
            id_mapping_entry_count: Some(150),
            linked_coverage_count: Some(4),
            attachment_metadata_count: Some(8),
            manifest_present: Some(true),
        }
    }

    fn blocked_request() -> FinalValidationExecutionPreviewRequest {
        FinalValidationExecutionPreviewRequest {
            package_filename: None,
            schema_write_preview_ready: None,
            record_write_preview_ready: None,
            mapping_checkpoint_preview_ready: None,
            linked_second_pass_preview_ready: None,
            final_validation_policy_safe: None,
            final_validation_enforcement_policy_safe: None,
            sensitive_data_safe: None,
            attachment_phase_disabled_safe: None,
            live_write_readiness_satisfied: None,
            table_count: None,
            field_count: None,
            record_count: None,
            id_mapping_entry_count: None,
            linked_coverage_count: None,
            attachment_metadata_count: None,
            manifest_present: None,
        }
    }

    // ── Safety invariants ──────────────────────────────────────────────────────

    #[test]
    fn writes_enabled_always_false_safe_request() {
        let result = preview_final_validation_execution(&safe_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_safe_request() {
        let result = preview_final_validation_execution(&safe_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_safe_request() {
        let result = preview_final_validation_execution(&safe_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false_blocked_request() {
        let result = preview_final_validation_execution(&blocked_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_blocked_request() {
        let result = preview_final_validation_execution(&blocked_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_blocked_request() {
        let result = preview_final_validation_execution(&blocked_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn safety_snapshot_write_gate_disabled_always_true() {
        let result = preview_final_validation_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    #[test]
    fn safety_snapshot_write_gate_disabled_always_true_blocked() {
        let result = preview_final_validation_execution(&blocked_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── Blocked behavior ───────────────────────────────────────────────────────

    #[test]
    fn blocked_when_all_prerequisites_missing() {
        let result = preview_final_validation_execution(&blocked_request());
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        assert_eq!(
            result.mode,
            FinalValidationExecutionPreviewMode::LiveBlocked
        );
        assert!(result.blocked_reason.is_some());
    }

    #[test]
    fn blocked_reason_contains_write_gate_prereq() {
        let mut req = blocked_request();
        // Simulate write gate check explicitly by ensuring it's the first blocker.
        // Since the module always reads from evaluate_write_gate(), which returns Disabled,
        // the first blocker will be the next unmet prerequisite.
        req.schema_write_preview_ready = None;
        let result = preview_final_validation_execution(&req);
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(
            reason.contains(PREREQ_SCHEMA_WRITE_PREVIEW),
            "expected FVEP-PRE-02 in blocked reason, got: {reason}"
        );
    }

    #[test]
    fn blocked_when_schema_write_preview_not_ready() {
        let mut req = safe_request();
        req.schema_write_preview_ready = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_SCHEMA_WRITE_PREVIEW));
    }

    #[test]
    fn blocked_when_record_write_preview_not_ready() {
        let mut req = safe_request();
        req.record_write_preview_ready = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_RECORD_WRITE_PREVIEW));
    }

    #[test]
    fn blocked_when_mapping_checkpoint_preview_not_ready() {
        let mut req = safe_request();
        req.mapping_checkpoint_preview_ready = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_MAPPING_CHECKPOINT_PREVIEW));
    }

    #[test]
    fn blocked_when_linked_second_pass_preview_not_ready() {
        let mut req = safe_request();
        req.linked_second_pass_preview_ready = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_LINKED_SECOND_PASS_PREVIEW));
    }

    #[test]
    fn blocked_when_final_validation_policy_not_safe() {
        let mut req = safe_request();
        req.final_validation_policy_safe = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_FINAL_VALIDATION_POLICY));
    }

    #[test]
    fn blocked_when_enforcement_policy_not_safe() {
        let mut req = safe_request();
        req.final_validation_enforcement_policy_safe = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_FINAL_VALIDATION_ENFORCEMENT_POLICY));
    }

    #[test]
    fn blocked_when_sensitive_data_not_safe() {
        let mut req = safe_request();
        req.sensitive_data_safe = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_SENSITIVE_DATA));
    }

    #[test]
    fn blocked_when_attachment_phase_disabled_not_safe() {
        let mut req = safe_request();
        req.attachment_phase_disabled_safe = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_ATTACHMENT_PHASE_DISABLED));
    }

    #[test]
    fn blocked_when_lwr_not_satisfied() {
        let mut req = safe_request();
        req.live_write_readiness_satisfied = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::Blocked
        );
        let reason = result.blocked_reason.unwrap_or_default();
        assert!(reason.contains(PREREQ_LWR));
    }

    // ── DryRunReady behavior ───────────────────────────────────────────────────

    #[test]
    fn dry_run_ready_when_all_prerequisites_satisfied() {
        let result = preview_final_validation_execution(&safe_request());
        assert_eq!(
            result.status,
            FinalValidationExecutionPreviewStatus::DryRunReady
        );
        assert_eq!(result.mode, FinalValidationExecutionPreviewMode::DryRunOnly);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn dry_run_ready_has_eight_checks() {
        let result = preview_final_validation_execution(&safe_request());
        assert_eq!(result.checks.len(), 8);
    }

    #[test]
    fn dry_run_ready_check_ids_in_order() {
        let result = preview_final_validation_execution(&safe_request());
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                CHK_SCHEMA,
                CHK_FIELDS,
                CHK_RECORDS,
                CHK_MAPPING,
                CHK_LINKED,
                CHK_ATTACH,
                CHK_MANIFEST,
                CHK_GUARD
            ]
        );
    }

    #[test]
    fn dry_run_ready_checks_are_pending_except_skipped_manifest() {
        let mut req = safe_request();
        req.manifest_present = Some(false);
        let result = preview_final_validation_execution(&req);
        for check in &result.checks {
            if check.check_id == CHK_MANIFEST {
                assert_eq!(
                    check.status,
                    FinalValidationExecutionPreviewCheckStatus::Skipped
                );
            } else {
                assert_eq!(
                    check.status,
                    FinalValidationExecutionPreviewCheckStatus::Pending
                );
            }
        }
    }

    #[test]
    fn all_checks_pending_when_manifest_present() {
        let result = preview_final_validation_execution(&safe_request());
        for check in &result.checks {
            assert_eq!(
                check.status,
                FinalValidationExecutionPreviewCheckStatus::Pending
            );
        }
    }

    #[test]
    fn summary_counts_reflect_request_values() {
        let result = preview_final_validation_execution(&safe_request());
        assert_eq!(result.summary.table_count, 3);
        assert_eq!(result.summary.field_count, 12);
        assert_eq!(result.summary.record_count, 150);
        assert_eq!(result.summary.id_mapping_entry_count, 150);
        assert_eq!(result.summary.linked_coverage_count, 4);
        assert_eq!(result.summary.attachment_metadata_count, 8);
        assert!(result.summary.manifest_present);
    }

    #[test]
    fn summary_pending_check_count_correct_with_manifest() {
        let result = preview_final_validation_execution(&safe_request());
        assert_eq!(result.summary.pending_check_count, 8);
        assert_eq!(result.summary.non_pending_check_count, 0);
    }

    #[test]
    fn summary_pending_check_count_correct_without_manifest() {
        let mut req = safe_request();
        req.manifest_present = Some(false);
        let result = preview_final_validation_execution(&req);
        assert_eq!(result.summary.pending_check_count, 7);
        assert_eq!(result.summary.non_pending_check_count, 1);
    }

    #[test]
    fn message_contains_dry_run_advisory() {
        let result = preview_final_validation_execution(&safe_request());
        assert!(result.message.contains("dry-run only"));
        assert!(result.message.contains("remains disabled"));
    }

    #[test]
    fn message_contains_no_checkpoint_advisory() {
        let result = preview_final_validation_execution(&safe_request());
        assert!(result.message.contains("No checkpoint files are written"));
    }

    // ── No sensitive data leaks ────────────────────────────────────────────────

    #[test]
    fn no_raw_record_ids_in_checks() {
        // Serialize only user-visible check data — not the safety snapshot whose
        // camelCase field names (e.g. "recordWritePreviewReady") share the "rec"
        // prefix used in Airtable record IDs.
        let result = preview_final_validation_execution(&safe_request());
        let json = serde_json::to_string(&result.checks).expect("serialize");
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
        assert!(!has_airtable_id("rec"), "raw record ID in checks");
        assert!(!has_airtable_id("fld"), "raw field ID in checks");
        assert!(!has_airtable_id("tbl"), "raw table ID in checks");
    }

    #[test]
    fn no_raw_record_ids_in_summary() {
        let result = preview_final_validation_execution(&safe_request());
        let json = serde_json::to_string(&result.summary).expect("serialize");
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
        assert!(!has_airtable_id("rec"), "raw record ID in summary");
        assert!(!has_airtable_id("fld"), "raw field ID in summary");
        assert!(!has_airtable_id("tbl"), "raw table ID in summary");
    }

    #[test]
    fn no_token_in_result() {
        let result = preview_final_validation_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("pat_"), "token prefix found in result");
    }

    #[test]
    fn no_absolute_path_in_result() {
        let result = preview_final_validation_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
        assert!(!json.contains("/home/"));
    }

    // ── Write gate passthrough ─────────────────────────────────────────────────

    #[test]
    fn write_gate_not_bypassed_by_safe_request() {
        let result = preview_final_validation_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn evaluate_write_gate_still_disabled_after_preview() {
        use crate::restore::write_gate::evaluate_write_gate;
        let _result = preview_final_validation_execution(&safe_request());
        let gate = evaluate_write_gate();
        assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
    }

    // ── Check content ──────────────────────────────────────────────────────────

    #[test]
    fn schema_check_expected_count_matches_request() {
        let result = preview_final_validation_execution(&safe_request());
        let schema_check = result
            .checks
            .iter()
            .find(|c| c.check_id == CHK_SCHEMA)
            .expect("schema check present");
        assert_eq!(schema_check.expected_count, 3);
    }

    #[test]
    fn field_check_expected_count_matches_request() {
        let result = preview_final_validation_execution(&safe_request());
        let field_check = result
            .checks
            .iter()
            .find(|c| c.check_id == CHK_FIELDS)
            .expect("field check present");
        assert_eq!(field_check.expected_count, 12);
    }

    #[test]
    fn record_check_expected_count_matches_request() {
        let result = preview_final_validation_execution(&safe_request());
        let record_check = result
            .checks
            .iter()
            .find(|c| c.check_id == CHK_RECORDS)
            .expect("record check present");
        assert_eq!(record_check.expected_count, 150);
    }

    #[test]
    fn mapping_check_expected_count_matches_request() {
        let result = preview_final_validation_execution(&safe_request());
        let mapping_check = result
            .checks
            .iter()
            .find(|c| c.check_id == CHK_MAPPING)
            .expect("mapping check present");
        assert_eq!(mapping_check.expected_count, 150);
    }

    #[test]
    fn manifest_check_skipped_when_no_manifest() {
        let mut req = safe_request();
        req.manifest_present = Some(false);
        let result = preview_final_validation_execution(&req);
        let manifest_check = result
            .checks
            .iter()
            .find(|c| c.check_id == CHK_MANIFEST)
            .expect("manifest check present");
        assert_eq!(
            manifest_check.status,
            FinalValidationExecutionPreviewCheckStatus::Skipped
        );
        assert_eq!(manifest_check.expected_count, 0);
    }

    #[test]
    fn manifest_check_pending_when_manifest_present() {
        let result = preview_final_validation_execution(&safe_request());
        let manifest_check = result
            .checks
            .iter()
            .find(|c| c.check_id == CHK_MANIFEST)
            .expect("manifest check present");
        assert_eq!(
            manifest_check.status,
            FinalValidationExecutionPreviewCheckStatus::Pending
        );
        assert_eq!(manifest_check.expected_count, 1);
    }

    #[test]
    fn guard_check_always_pending_when_ready() {
        let result = preview_final_validation_execution(&safe_request());
        let guard_check = result
            .checks
            .iter()
            .find(|c| c.check_id == CHK_GUARD)
            .expect("guard check present");
        assert_eq!(
            guard_check.status,
            FinalValidationExecutionPreviewCheckStatus::Pending
        );
        assert_eq!(guard_check.expected_count, 1);
    }

    #[test]
    fn checks_note_contains_no_raw_ids_advisory() {
        let result = preview_final_validation_execution(&safe_request());
        for check in &result.checks {
            if check.check_id != CHK_MANIFEST && check.check_id != CHK_GUARD {
                assert!(
                    check.note.contains("No raw") || check.note.contains("no raw"),
                    "check {} note missing no-raw advisory: {}",
                    check.check_id,
                    check.note
                );
            }
        }
    }
}
