use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Execution status for the final validation reader foundation.
///
/// Safety invariants:
/// - `DryRunOnly` does NOT enable live validation reads.
/// - `NotExecuted` is the expected state when the validation read gate is disabled.
/// - `Blocked` indicates a safety prerequisite is missing.
/// - No status named `succeeded`, `complete`, `done`, or `passed` exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationReaderStatus {
    /// All prerequisites satisfied but the validation read gate is disabled.
    /// This is the current expected state — no reads occur.
    NotExecuted,
    /// Plan built; reads would be sandbox-only.
    /// Gate must be explicitly enabled before this transitions to execution.
    DryRunOnly,
    /// A required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Execution mode for the final validation reader.
///
/// Safety invariants:
/// - `Disabled` is the only reachable mode in the current implementation.
/// - `SandboxOnly` is defined for future use but is unreachable while the
///   validation read gate is disabled.
/// - No `Production` mode exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationReaderMode {
    /// Validation read gate is disabled — no reads are possible. Default state.
    Disabled,
    /// Sandbox-only mode — reads are restricted to verified sandbox targets.
    /// Unreachable in the current implementation.
    SandboxOnly,
}

/// Status of a single validation check descriptor in the reader's internal plan.
///
/// Note: `succeeded` / `completed` / `passed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinalValidationReaderCheckStatus {
    /// The check would execute if the gate were enabled. Not executed.
    Pending,
    /// The check is blocked by a safety prerequisite.
    Blocked,
    /// The check is skipped (e.g. no manifest present).
    Skipped,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single typed validation read descriptor in the internal plan.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No old or new Airtable record IDs.
/// - No raw record field values.
/// - No raw HTTP body.
/// - No attachment URL.
/// - `status` is never `succeeded` or `passed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationReaderCheck {
    /// Stable check identifier (mirrors preview: e.g. `FVRD-CHK-SCHEMA`).
    pub check_id: String,
    /// Human-readable label.
    pub label: String,
    /// Safe expected count (no raw IDs).
    pub expected_count: usize,
    pub status: FinalValidationReaderCheckStatus,
    pub note: String,
}

/// Point-in-time safety snapshot for the final validation reader foundation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationReaderSafetySnapshot {
    /// Validation read gate — always `true` (always disabled) in the current build.
    pub read_gate_disabled: bool,
    /// Write gate — always `true` (always disabled) in the current build.
    pub write_gate_disabled: bool,
    /// Whether the mode is sandbox-only (always `false` in the current build).
    pub sandbox_mode_active: bool,
    /// Whether the explicit internal final validation read flag was set.
    pub explicit_internal_read_requested: bool,
    /// Whether sandbox verification passed.
    pub sandbox_verified: bool,
    /// Whether the schema write executor foundation completed safely.
    pub schema_executor_safe: bool,
    /// Whether the record write executor foundation completed safely.
    pub record_executor_safe: bool,
    /// Whether the linked second-pass executor foundation completed safely.
    pub linked_executor_safe: bool,
    /// Whether the final validation execution preview returned DryRunReady.
    pub final_validation_preview_ready: bool,
    /// Whether the final validation enforcement policy is safe.
    pub final_validation_enforcement_safe: bool,
    /// Whether sensitive data safety policy is satisfied.
    pub sensitive_data_safe: bool,
    /// Whether attachment phase disabled policy is safe.
    pub attachment_phase_disabled_safe: bool,
}

/// Request to the final validation reader foundation.
///
/// Safety invariants:
/// - No token field.
/// - No output path field.
/// - No record payload field.
/// - No old or new record IDs.
/// - No attachment URL.
/// - No raw HTTP body.
///
/// `explicit_internal_final_validation_read_requested` must be `true` for the reader
/// to proceed past the gate check. It is an internal-only guard — there is no
/// UI control that sets it, and the validation read gate must also allow reads
/// (which it currently never does).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationReaderRequest {
    /// Must be `sandboxOnly` for reads to be considered.
    /// `disabled` (the default) always results in `Blocked`.
    pub mode: FinalValidationReaderMode,
    /// Internal-only flag. Must be explicitly `true` to proceed past the gate.
    /// No UI control sets this; it is an internal safety guard.
    pub explicit_internal_final_validation_read_requested: bool,
    /// Whether the sandbox environment check has passed.
    pub sandbox_verified: bool,
    /// Whether the schema write executor foundation completed safely.
    pub schema_executor_safe: bool,
    /// Whether the record write executor foundation completed safely.
    pub record_executor_safe: bool,
    /// Whether the linked second-pass executor foundation completed safely.
    pub linked_executor_safe: bool,
    /// Whether the final validation execution preview returned DryRunReady.
    pub final_validation_preview_ready: bool,
    /// Whether the final validation enforcement policy is safe.
    pub final_validation_enforcement_safe: bool,
    /// Whether the sensitive data safety policy is satisfied.
    pub sensitive_data_safe: bool,
    /// Whether the attachment phase disabled policy is safe.
    pub attachment_phase_disabled_safe: bool,
    /// Safe count of tables to be validated (no raw IDs).
    pub table_count: usize,
    /// Safe count of fields to be validated (no raw IDs).
    pub field_count: usize,
    /// Safe count of records to be counted (no raw IDs).
    pub record_count: usize,
    /// Safe count of ID mapping entries (no raw IDs).
    pub id_mapping_entry_count: usize,
    /// Safe count of linked field coverage entries.
    pub linked_coverage_count: usize,
    /// Safe count of attachment metadata entries.
    pub attachment_metadata_count: usize,
    /// Whether a package manifest is present.
    pub manifest_present: bool,
}

/// Result of the final validation reader foundation.
///
/// Safety invariants (enforced):
/// - `reads_enabled` is always `false`.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `succeeded`, `complete`, `done`, or `passed`.
/// - `NotExecuted` / `DryRunOnly` do NOT enable live reads or writes.
/// - No Airtable client is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalValidationReaderResult {
    pub status: FinalValidationReaderStatus,
    pub mode: FinalValidationReaderMode,
    pub message: String,
    pub checks: Vec<FinalValidationReaderCheck>,
    pub safety_snapshot: FinalValidationReaderSafetySnapshot,
    pub total_check_count: usize,
    pub pending_check_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `true` — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always `false` — no network read operations were attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — live validation reads are not enabled.
    pub reads_enabled: bool,
    /// Always `false` — live writes are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs ──────────────────────────────────────────────────────────

const FVRD_PREREQ_READ_GATE: &str = "FVRD-PRE-01";
const FVRD_PREREQ_MODE: &str = "FVRD-PRE-02";
const FVRD_PREREQ_EXPLICIT_FLAG: &str = "FVRD-PRE-03";
const FVRD_PREREQ_SANDBOX: &str = "FVRD-PRE-04";
const FVRD_PREREQ_SCHEMA_EXECUTOR: &str = "FVRD-PRE-05";
const FVRD_PREREQ_RECORD_EXECUTOR: &str = "FVRD-PRE-06";
const FVRD_PREREQ_LINKED_EXECUTOR: &str = "FVRD-PRE-07";
const FVRD_PREREQ_FINAL_VALIDATION_PREVIEW: &str = "FVRD-PRE-08";
const FVRD_PREREQ_ENFORCEMENT: &str = "FVRD-PRE-09";
const FVRD_PREREQ_SENSITIVE_DATA: &str = "FVRD-PRE-10";
const FVRD_PREREQ_ATTACHMENT_PHASE: &str = "FVRD-PRE-11";

// ── Check IDs ─────────────────────────────────────────────────────────────────

const CHK_SCHEMA: &str = "FVRD-CHK-SCHEMA";
const CHK_FIELDS: &str = "FVRD-CHK-FIELDS";
const CHK_RECORDS: &str = "FVRD-CHK-RECORDS";
const CHK_MAPPING: &str = "FVRD-CHK-MAPPING";
const CHK_LINKED: &str = "FVRD-CHK-LINKED";
const CHK_ATTACH: &str = "FVRD-CHK-ATTACH";
const CHK_MANIFEST: &str = "FVRD-CHK-MANIFEST";
const CHK_GUARD: &str = "FVRD-CHK-GUARD";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds the final validation reader foundation plan.
///
/// This function:
/// - Never calls the Airtable API (reads or writes).
/// - Never creates, updates, or deletes any record, table, or field.
/// - Always enforces the validation read gate (currently always disabled).
/// - Always returns `reads_enabled: false`, `writes_enabled: false`,
///   `no_changes_made: true`, `network_reads_attempted: false`,
///   `network_writes_attempted: false`.
/// - Returns `Blocked` when any prerequisite is missing.
/// - Returns `NotExecuted` when all prerequisites are met but the gate is disabled.
/// - Returns `DryRunOnly` only when all prerequisites pass AND the read gate
///   is explicitly enabled — currently unreachable.
/// - Does not return any token, full path, record payload, raw HTTP body,
///   old/new record IDs, or attachment URL.
pub fn build_final_validation_reader_plan(
    request: &FinalValidationReaderRequest,
) -> FinalValidationReaderResult {
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);
    // The validation read gate reuses the write gate in this implementation:
    // reads cannot proceed while writes are disabled.
    let read_gate_disabled = write_gate_disabled;

    let safety_snapshot = FinalValidationReaderSafetySnapshot {
        read_gate_disabled,
        write_gate_disabled,
        sandbox_mode_active: matches!(request.mode, FinalValidationReaderMode::SandboxOnly),
        explicit_internal_read_requested: request.explicit_internal_final_validation_read_requested,
        sandbox_verified: request.sandbox_verified,
        schema_executor_safe: request.schema_executor_safe,
        record_executor_safe: request.record_executor_safe,
        linked_executor_safe: request.linked_executor_safe,
        final_validation_preview_ready: request.final_validation_preview_ready,
        final_validation_enforcement_safe: request.final_validation_enforcement_safe,
        sensitive_data_safe: request.sensitive_data_safe,
        attachment_phase_disabled_safe: request.attachment_phase_disabled_safe,
    };

    // Check prerequisites in order; first failure blocks.
    let blocked_reason: Option<String> = if !read_gate_disabled {
        // Defense-in-depth: unreachable given write gate always returns Disabled.
        Some(format!(
            "{FVRD_PREREQ_READ_GATE}: Validation read gate is not disabled. \
             Final validation reader must not proceed while the read gate could be enabled."
        ))
    } else if !matches!(request.mode, FinalValidationReaderMode::SandboxOnly) {
        Some(format!(
            "{FVRD_PREREQ_MODE}: Reader mode must be sandboxOnly. \
             Mode 'disabled' does not permit reads. \
             No validation reads will be attempted."
        ))
    } else if !request.explicit_internal_final_validation_read_requested {
        Some(format!(
            "{FVRD_PREREQ_EXPLICIT_FLAG}: Explicit internal final validation read flag is not set. \
             The internal flag must be explicitly true before reads are considered. \
             No UI control sets this flag."
        ))
    } else if !request.sandbox_verified {
        Some(format!(
            "{FVRD_PREREQ_SANDBOX}: Sandbox environment verification has not passed. \
             A verified sandbox target is required before validation reads are considered."
        ))
    } else if !request.schema_executor_safe {
        Some(format!(
            "{FVRD_PREREQ_SCHEMA_EXECUTOR}: Schema write executor foundation has not completed \
             safely. Schema writes must be safe or notExecuted before final validation reads."
        ))
    } else if !request.record_executor_safe {
        Some(format!(
            "{FVRD_PREREQ_RECORD_EXECUTOR}: Record write executor foundation has not completed \
             safely. Record writes must be safe or notExecuted before final validation reads."
        ))
    } else if !request.linked_executor_safe {
        Some(format!(
            "{FVRD_PREREQ_LINKED_EXECUTOR}: Linked second-pass executor foundation has not \
             completed safely. Linked updates must be safe or notExecuted before \
             final validation reads."
        ))
    } else if !request.final_validation_preview_ready {
        Some(format!(
            "{FVRD_PREREQ_FINAL_VALIDATION_PREVIEW}: Final validation execution preview has not \
             returned DryRunReady. Complete the preview before requesting validation reads."
        ))
    } else if !request.final_validation_enforcement_safe {
        Some(format!(
            "{FVRD_PREREQ_ENFORCEMENT}: Final validation enforcement policy is not safe. \
             All three completion guards must be declared before reads are considered."
        ))
    } else if !request.sensitive_data_safe {
        Some(format!(
            "{FVRD_PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied. \
             All exposure surfaces must have redaction coverage before validation reads."
        ))
    } else if !request.attachment_phase_disabled_safe {
        Some(format!(
            "{FVRD_PREREQ_ATTACHMENT_PHASE}: Attachment phase disabled policy is not safe. \
             Attachment operations must be metadata-only before validation reads."
        ))
    } else {
        None
    };

    if let Some(ref reason) = blocked_reason {
        let blocked_checks = build_blocked_checks();
        let total = blocked_checks.len();
        return FinalValidationReaderResult {
            status: FinalValidationReaderStatus::Blocked,
            mode: FinalValidationReaderMode::Disabled,
            message: format!(
                "Final validation reader is blocked. {reason} \
                 No validation reads will be attempted."
            ),
            checks: blocked_checks,
            safety_snapshot,
            total_check_count: total,
            pending_check_count: 0,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_reads_attempted: false,
            network_writes_attempted: false,
            reads_enabled: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the internal validation read plan.
    let checks = build_reader_checks(request);
    let total = checks.len();
    let pending = checks
        .iter()
        .filter(|c| c.status == FinalValidationReaderCheckStatus::Pending)
        .count();

    // Read gate is disabled — result is NotExecuted (not DryRunOnly).
    FinalValidationReaderResult {
        status: FinalValidationReaderStatus::NotExecuted,
        mode: FinalValidationReaderMode::Disabled,
        message: format!(
            "Final validation reader plan built ({total} check(s), {pending} pending). \
             Validation read gate is disabled — no Airtable reads are attempted. \
             No old or new record IDs are present. \
             No Airtable changes made."
        ),
        checks,
        safety_snapshot,
        total_check_count: total,
        pending_check_count: pending,
        blocked_reason: None,
        no_changes_made: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        reads_enabled: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_blocked_checks() -> Vec<FinalValidationReaderCheck> {
    vec![FinalValidationReaderCheck {
        check_id: "FVRD-CHK-BLOCKED".to_string(),
        label: "Blocked".to_string(),
        expected_count: 0,
        status: FinalValidationReaderCheckStatus::Blocked,
        note: "Safety prerequisites not satisfied. No validation reads can be planned.".to_string(),
    }]
}

fn build_reader_checks(request: &FinalValidationReaderRequest) -> Vec<FinalValidationReaderCheck> {
    vec![
        FinalValidationReaderCheck {
            check_id: CHK_SCHEMA.to_string(),
            label: "Schema/table count read".to_string(),
            expected_count: request.table_count,
            status: FinalValidationReaderCheckStatus::Pending,
            note: format!(
                "Would read table list from Airtable and compare against {} expected table(s). \
                 Read gate disabled — no network call made.",
                request.table_count
            ),
        },
        FinalValidationReaderCheck {
            check_id: CHK_FIELDS.to_string(),
            label: "Field count read".to_string(),
            expected_count: request.field_count,
            status: FinalValidationReaderCheckStatus::Pending,
            note: format!(
                "Would read field definitions from Airtable and compare against {} expected \
                 field(s). Read gate disabled — no network call made.",
                request.field_count
            ),
        },
        FinalValidationReaderCheck {
            check_id: CHK_RECORDS.to_string(),
            label: "Record count read".to_string(),
            expected_count: request.record_count,
            status: FinalValidationReaderCheckStatus::Pending,
            note: format!(
                "Would read record count from Airtable and compare against {} expected record(s). \
                 No raw record IDs returned. Read gate disabled — no network call made.",
                request.record_count
            ),
        },
        FinalValidationReaderCheck {
            check_id: CHK_MAPPING.to_string(),
            label: "ID mapping coverage read".to_string(),
            expected_count: request.id_mapping_entry_count,
            status: FinalValidationReaderCheckStatus::Pending,
            note: format!(
                "Would verify ID mapping coverage for {} entry/entries. \
                 No raw record IDs returned. Read gate disabled — no network call made.",
                request.id_mapping_entry_count
            ),
        },
        FinalValidationReaderCheck {
            check_id: CHK_LINKED.to_string(),
            label: "Linked field coverage read".to_string(),
            expected_count: request.linked_coverage_count,
            status: FinalValidationReaderCheckStatus::Pending,
            note: format!(
                "Would verify linked field coverage for {} entry/entries. \
                 No raw record IDs returned. Read gate disabled — no network call made.",
                request.linked_coverage_count
            ),
        },
        FinalValidationReaderCheck {
            check_id: CHK_ATTACH.to_string(),
            label: "Attachment metadata-only read".to_string(),
            expected_count: request.attachment_metadata_count,
            status: FinalValidationReaderCheckStatus::Pending,
            note: format!(
                "Would read attachment metadata (filename, MIME type, size) for {} \
                 entry/entries. Metadata inspection only — no binary retrieval, \
                 no attachment URL returned. \
                 Read gate disabled — no network call made.",
                request.attachment_metadata_count
            ),
        },
        FinalValidationReaderCheck {
            check_id: CHK_MANIFEST.to_string(),
            label: "Manifest/checksum reference comparison".to_string(),
            expected_count: if request.manifest_present { 1 } else { 0 },
            status: if request.manifest_present {
                FinalValidationReaderCheckStatus::Pending
            } else {
                FinalValidationReaderCheckStatus::Skipped
            },
            note: if request.manifest_present {
                "Would compare package manifest checksums against restored base state. \
                 Read gate disabled — no network call made."
                    .to_string()
            } else {
                "No manifest present — manifest/checksum comparison skipped.".to_string()
            },
        },
        FinalValidationReaderCheck {
            check_id: CHK_GUARD.to_string(),
            label: "Final completion guard".to_string(),
            expected_count: 0,
            status: FinalValidationReaderCheckStatus::Pending,
            note: "Completion guard: no result can carry a success status without all \
                   prior checks passing. Read gate disabled — guard is a descriptor only."
                .to_string(),
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn all_prereqs_request() -> FinalValidationReaderRequest {
        FinalValidationReaderRequest {
            mode: FinalValidationReaderMode::SandboxOnly,
            explicit_internal_final_validation_read_requested: true,
            sandbox_verified: true,
            schema_executor_safe: true,
            record_executor_safe: true,
            linked_executor_safe: true,
            final_validation_preview_ready: true,
            final_validation_enforcement_safe: true,
            sensitive_data_safe: true,
            attachment_phase_disabled_safe: true,
            table_count: 3,
            field_count: 12,
            record_count: 50,
            id_mapping_entry_count: 50,
            linked_coverage_count: 15,
            attachment_metadata_count: 4,
            manifest_present: true,
        }
    }

    // ── Blocked when prerequisites missing ────────────────────────────────────

    #[test]
    fn foundation_blocked_when_mode_disabled() {
        let mut req = all_prereqs_request();
        req.mode = FinalValidationReaderMode::Disabled;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-02"));
        assert!(!result.reads_enabled);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_blocked_when_explicit_flag_not_set() {
        let mut req = all_prereqs_request();
        req.explicit_internal_final_validation_read_requested = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-03"));
    }

    #[test]
    fn foundation_blocked_when_sandbox_not_verified() {
        let mut req = all_prereqs_request();
        req.sandbox_verified = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-04"));
    }

    #[test]
    fn foundation_blocked_when_schema_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.schema_executor_safe = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-05"));
    }

    #[test]
    fn foundation_blocked_when_record_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.record_executor_safe = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-06"));
    }

    #[test]
    fn foundation_blocked_when_linked_executor_not_safe() {
        let mut req = all_prereqs_request();
        req.linked_executor_safe = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-07"));
    }

    #[test]
    fn foundation_blocked_when_final_validation_preview_not_ready() {
        let mut req = all_prereqs_request();
        req.final_validation_preview_ready = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-08"));
    }

    #[test]
    fn foundation_blocked_when_enforcement_not_safe() {
        let mut req = all_prereqs_request();
        req.final_validation_enforcement_safe = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-09"));
    }

    #[test]
    fn foundation_blocked_when_sensitive_data_not_safe() {
        let mut req = all_prereqs_request();
        req.sensitive_data_safe = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-10"));
    }

    #[test]
    fn foundation_blocked_when_attachment_phase_not_safe() {
        let mut req = all_prereqs_request();
        req.attachment_phase_disabled_safe = false;
        let result = build_final_validation_reader_plan(&req);
        assert_eq!(result.status, FinalValidationReaderStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("FVRD-PRE-11"));
    }

    // ── NotExecuted when all prerequisites met ────────────────────────────────

    #[test]
    fn foundation_not_executed_when_all_prereqs_met() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert_eq!(result.status, FinalValidationReaderStatus::NotExecuted);
        assert!(!result.reads_enabled);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn foundation_read_gate_still_disabled_after_plan() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert!(result.safety_snapshot.read_gate_disabled);
        assert!(result.safety_snapshot.write_gate_disabled);
        assert!(!result.reads_enabled);
        assert!(!result.writes_enabled);
    }

    #[test]
    fn foundation_safety_snapshot_gates_always_disabled() {
        let mut req = all_prereqs_request();
        req.mode = FinalValidationReaderMode::Disabled;
        let result = build_final_validation_reader_plan(&req);
        assert!(result.safety_snapshot.read_gate_disabled);
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── No production mode ────────────────────────────────────────────────────

    #[test]
    fn foundation_no_production_mode_exists() {
        let disabled = FinalValidationReaderMode::Disabled;
        let sandbox = FinalValidationReaderMode::SandboxOnly;
        assert_ne!(disabled, sandbox);
        let json = serde_json::to_string(&disabled).expect("serialize");
        assert!(!json.contains("production"));
        let json = serde_json::to_string(&sandbox).expect("serialize");
        assert!(!json.contains("production"));
    }

    // ── Check ordering and content ────────────────────────────────────────────

    #[test]
    fn foundation_checks_built_in_not_executed_result() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert_eq!(result.status, FinalValidationReaderStatus::NotExecuted);
        assert!(!result.checks.is_empty());
        assert!(result.total_check_count > 0);
        assert!(result.pending_check_count > 0);
    }

    #[test]
    fn foundation_check_ordering_is_deterministic() {
        let r1 = build_final_validation_reader_plan(&all_prereqs_request());
        let r2 = build_final_validation_reader_plan(&all_prereqs_request());
        let ids1: Vec<_> = r1.checks.iter().map(|c| &c.check_id).collect();
        let ids2: Vec<_> = r2.checks.iter().map(|c| &c.check_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn foundation_check_ids_use_stable_prefixes() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        for check in &result.checks {
            assert!(
                check.check_id.starts_with("FVRD-CHK-"),
                "check_id must start with FVRD-CHK-, got: {}",
                check.check_id
            );
        }
    }

    #[test]
    fn foundation_schema_check_comes_first() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert_eq!(result.checks[0].check_id, "FVRD-CHK-SCHEMA");
    }

    #[test]
    fn foundation_guard_check_comes_last() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let last = result.checks.last().expect("checks not empty");
        assert_eq!(last.check_id, "FVRD-CHK-GUARD");
    }

    #[test]
    fn foundation_manifest_check_skipped_when_not_present() {
        let mut req = all_prereqs_request();
        req.manifest_present = false;
        let result = build_final_validation_reader_plan(&req);
        let manifest_check = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVRD-CHK-MANIFEST")
            .expect("manifest check must exist");
        assert_eq!(
            manifest_check.status,
            FinalValidationReaderCheckStatus::Skipped
        );
    }

    #[test]
    fn foundation_manifest_check_pending_when_present() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let manifest_check = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVRD-CHK-MANIFEST")
            .expect("manifest check must exist");
        assert_eq!(
            manifest_check.status,
            FinalValidationReaderCheckStatus::Pending
        );
    }

    #[test]
    fn foundation_attach_check_metadata_only() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let attach = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVRD-CHK-ATTACH")
            .expect("attach check must exist");
        assert!(
            attach.note.to_lowercase().contains("metadata"),
            "attach check note must state metadata-only"
        );
        assert!(
            !attach.note.contains("download"),
            "attach check must not mention download"
        );
    }

    // ── No success state ──────────────────────────────────────────────────────

    #[test]
    fn foundation_no_success_state_introduced() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert!(!result.reads_enabled);
        assert!(!result.writes_enabled);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("\"passed\""));
        assert!(!json.contains("restoreComplete"));
        assert!(!json.contains("restoreSuccess"));
    }

    // ── Safety serialization ──────────────────────────────────────────────────

    #[test]
    fn foundation_no_token_in_result() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn foundation_no_absolute_path_in_result() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn foundation_no_record_payload_in_result() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":[{"));
    }

    #[test]
    fn foundation_no_attachment_url_in_result() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn foundation_no_old_or_new_record_id_in_result() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("oldRecordId"));
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_old_"));
        assert!(!json.contains("rec_new_"));
    }

    // ── No Airtable client called ─────────────────────────────────────────────

    #[test]
    fn foundation_no_airtable_client_called() {
        // build_final_validation_reader_plan accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert_eq!(result.status, FinalValidationReaderStatus::NotExecuted);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn foundation_no_network_in_blocked_state() {
        let mut req = all_prereqs_request();
        req.mode = FinalValidationReaderMode::Disabled;
        let result = build_final_validation_reader_plan(&req);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(result.no_changes_made);
    }

    #[test]
    fn foundation_sandboxonly_still_not_executed_while_gate_disabled() {
        // SandboxOnly + all prereqs → NotExecuted (not DryRunOnly), because gate is disabled.
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert_ne!(result.status, FinalValidationReaderStatus::DryRunOnly);
        assert_eq!(result.status, FinalValidationReaderStatus::NotExecuted);
    }

    #[test]
    fn foundation_total_and_pending_count_consistent() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        assert_eq!(result.total_check_count, result.checks.len());
        let actual_pending = result
            .checks
            .iter()
            .filter(|c| c.status == FinalValidationReaderCheckStatus::Pending)
            .count();
        assert_eq!(result.pending_check_count, actual_pending);
    }

    #[test]
    fn foundation_expected_count_reflects_request() {
        let result = build_final_validation_reader_plan(&all_prereqs_request());
        let schema_check = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVRD-CHK-SCHEMA")
            .expect("schema check");
        assert_eq!(schema_check.expected_count, 3); // table_count = 3
        let record_check = result
            .checks
            .iter()
            .find(|c| c.check_id == "FVRD-CHK-RECORDS")
            .expect("record check");
        assert_eq!(record_check.expected_count, 50); // record_count = 50
    }
}
