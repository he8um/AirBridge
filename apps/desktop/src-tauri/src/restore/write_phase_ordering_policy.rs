use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WritePhaseOrderingPolicyStatus {
    /// All phases are declared in canonical order with no unsafe transitions.
    Compliant,
    /// A non-critical ordering concern exists but no hard safety threshold is
    /// violated.
    Warning,
    /// A required phase is missing, mis-ordered, or an unsafe transition exists.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WritePhaseOrderingCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// The canonical restore write phases, in execution order.
///
/// Serializes to camelCase strings:
/// preflight, schemaCreate, schemaVerify, recordCreate, recordVerify,
/// linkedRecordUpdate, linkedRecordVerify, attachmentMetadataVerify,
/// finalValidation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreWritePhaseKind {
    Preflight,
    SchemaCreate,
    SchemaVerify,
    RecordCreate,
    RecordVerify,
    LinkedRecordUpdate,
    LinkedRecordVerify,
    AttachmentMetadataVerify,
    FinalValidation,
}

/// The status of a single declared phase in the phase plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreWritePhaseStatus {
    NotStarted,
    Planned,
    Ready,
    Blocked,
    Completed,
    Skipped,
}

/// One declared phase entry in the write phase ordering request.
///
/// No token, no path, no record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclaredWritePhase {
    pub kind: RestoreWritePhaseKind,
    pub status: RestoreWritePhaseStatus,
    /// Optional human-readable reason, e.g. "metadata-only: attachment files
    /// not downloaded". No path, no token, no record IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
}

/// Input to the write phase ordering policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WritePhaseOrderingPolicyRequest {
    /// The ordered list of write phases as declared by the restore plan.
    /// Absence of this field causes immediate Blocked result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phases: Option<Vec<DeclaredWritePhase>>,
    /// Safe display label for the restore target (base name only, no path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WritePhaseOrderingCheck {
    pub check_id: String,
    pub label: String,
    pub status: WritePhaseOrderingCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Read-only per-phase summary, safe for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WritePhaseOrderingSummaryEntry {
    pub kind: RestoreWritePhaseKind,
    pub status: RestoreWritePhaseStatus,
    pub canonical_position: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
}

/// Result from `verify_write_phase_ordering_policy`.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WritePhaseOrderingPolicyResult {
    pub status: WritePhaseOrderingPolicyStatus,
    pub checks: Vec<WritePhaseOrderingCheck>,
    pub message: String,
    /// Safe, read-only summary of the declared phase sequence (no sensitive values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_summary: Option<Vec<WritePhaseOrderingSummaryEntry>>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Canonical ordering ────────────────────────────────────────────────────────

/// The single authoritative sequence of restore write phases.
///
/// Any declared phase list must respect this order.
const CANONICAL_ORDER: &[RestoreWritePhaseKind] = &[
    RestoreWritePhaseKind::Preflight,
    RestoreWritePhaseKind::SchemaCreate,
    RestoreWritePhaseKind::SchemaVerify,
    RestoreWritePhaseKind::RecordCreate,
    RestoreWritePhaseKind::RecordVerify,
    RestoreWritePhaseKind::LinkedRecordUpdate,
    RestoreWritePhaseKind::LinkedRecordVerify,
    RestoreWritePhaseKind::AttachmentMetadataVerify,
    RestoreWritePhaseKind::FinalValidation,
];

fn canonical_position(kind: &RestoreWritePhaseKind) -> Option<usize> {
    CANONICAL_ORDER.iter().position(|k| k == kind)
}

fn phase_label(kind: &RestoreWritePhaseKind) -> &'static str {
    match kind {
        RestoreWritePhaseKind::Preflight => "preflight",
        RestoreWritePhaseKind::SchemaCreate => "schema_create",
        RestoreWritePhaseKind::SchemaVerify => "schema_verify",
        RestoreWritePhaseKind::RecordCreate => "record_create",
        RestoreWritePhaseKind::RecordVerify => "record_verify",
        RestoreWritePhaseKind::LinkedRecordUpdate => "linked_record_update",
        RestoreWritePhaseKind::LinkedRecordVerify => "linked_record_verify",
        RestoreWritePhaseKind::AttachmentMetadataVerify => "attachment_metadata_verify",
        RestoreWritePhaseKind::FinalValidation => "final_validation",
    }
}

/// Returns true if a phase's status means it is active (ready or completed).
fn is_active(status: &RestoreWritePhaseStatus) -> bool {
    matches!(
        status,
        RestoreWritePhaseStatus::Ready | RestoreWritePhaseStatus::Completed
    )
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies the write phase ordering policy for a planned restore write operation.
///
/// Check IDs:
/// - WPO-01: Write gate is disabled.
/// - WPO-02: Phase list is declared.
/// - WPO-03: All declared phases are in the canonical sequence.
/// - WPO-04: No phase appears before its prerequisite.
/// - WPO-05: record_create is not active before schema_verify is completed.
/// - WPO-06: linked_record_update is not active before record_verify is completed.
/// - WPO-07: final_validation is not active before linked_record_verify is completed.
/// - WPO-08: No attachment upload or attachment binary phase declared.
/// - WPO-09: attachment_metadata_verify skip is accepted with metadata-only reason (warning).
/// - WPO-10: Writes remain disabled.
///
/// No Airtable API calls are made.
/// No token is required.
/// No files are written.
/// No record payload is accepted or returned.
/// `writes_enabled` is always `false`.
/// `no_changes_made` is always `true`.
/// `network_writes_attempted` is always `false`.
pub fn verify_write_phase_ordering_policy(
    request: &WritePhaseOrderingPolicyRequest,
) -> WritePhaseOrderingPolicyResult {
    let mut checks: Vec<WritePhaseOrderingCheck> = Vec::new();

    // WPO-01: Write gate disabled
    let gate = evaluate_write_gate();
    if gate.status == RestoreWriteEngineStatus::Disabled {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: WritePhaseOrderingCheckStatus::Passed,
            message: "Restore write gate is disabled. No live writes are possible.".to_string(),
            remediation: None,
        });
    } else {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-01".to_string(),
            label: "write-gate-disabled".to_string(),
            status: WritePhaseOrderingCheckStatus::Failed,
            message: "Restore write gate is unexpectedly enabled.".to_string(),
            remediation: Some(
                "The write gate must remain disabled until all safety gates are satisfied."
                    .to_string(),
            ),
        });
    }

    // WPO-02: Phase list declared
    let phases = match &request.phases {
        Some(p) => p,
        None => {
            checks.push(WritePhaseOrderingCheck {
                check_id: "WPO-02".to_string(),
                label: "phase-list-declared".to_string(),
                status: WritePhaseOrderingCheckStatus::Failed,
                message: "No write phase list declared. A phase list is required before any \
                           live write path is considered."
                    .to_string(),
                remediation: Some(
                    "Declare a list of WritePhaseOrderingPolicyRequest phases in canonical order."
                        .to_string(),
                ),
            });
            return build_result(
                checks,
                None,
                WritePhaseOrderingPolicyStatus::Blocked,
                request.target_label.as_deref(),
            );
        }
    };

    checks.push(WritePhaseOrderingCheck {
        check_id: "WPO-02".to_string(),
        label: "phase-list-declared".to_string(),
        status: WritePhaseOrderingCheckStatus::Passed,
        message: format!("Write phase list is declared with {} phases.", phases.len()),
        remediation: None,
    });

    // WPO-03: All phases are in canonical sequence
    let mut last_canonical_pos: Option<usize> = None;
    let mut ordering_violation: Option<String> = None;
    for phase in phases.iter() {
        match canonical_position(&phase.kind) {
            None => {
                // Unknown phase kind — handled below in WPO-09/warning
            }
            Some(pos) => {
                if let Some(prev) = last_canonical_pos {
                    if pos < prev {
                        ordering_violation = Some(format!(
                            "Phase '{}' (canonical position {}) appears after '{}' (canonical \
                             position {}), which violates the required order.",
                            phase_label(&phase.kind),
                            pos + 1,
                            phase_label(&CANONICAL_ORDER[prev]),
                            prev + 1
                        ));
                        break;
                    }
                }
                last_canonical_pos = Some(pos);
            }
        }
    }

    if let Some(msg) = ordering_violation {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-03".to_string(),
            label: "canonical-order".to_string(),
            status: WritePhaseOrderingCheckStatus::Failed,
            message: msg,
            remediation: Some(
                "Reorder the phase list to match the canonical sequence: preflight → \
                 schema_create → schema_verify → record_create → record_verify → \
                 linked_record_update → linked_record_verify → attachment_metadata_verify → \
                 final_validation."
                    .to_string(),
            ),
        });
    } else {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-03".to_string(),
            label: "canonical-order".to_string(),
            status: WritePhaseOrderingCheckStatus::Passed,
            message: "Declared phases respect the canonical ordering.".to_string(),
            remediation: None,
        });
    }

    // Helper: find phase status by kind
    let find_status = |kind: &RestoreWritePhaseKind| -> Option<&RestoreWritePhaseStatus> {
        phases.iter().find(|p| &p.kind == kind).map(|p| &p.status)
    };

    // WPO-04: No phase appears before its prerequisite
    // We check that for each pair (A must precede B), if B is active then A must
    // be at least planned (not absent).
    let prerequisite_pairs: &[(RestoreWritePhaseKind, RestoreWritePhaseKind, &str)] = &[
        (
            RestoreWritePhaseKind::SchemaCreate,
            RestoreWritePhaseKind::SchemaVerify,
            "schema_verify requires schema_create",
        ),
        (
            RestoreWritePhaseKind::SchemaVerify,
            RestoreWritePhaseKind::RecordCreate,
            "record_create requires schema_verify",
        ),
        (
            RestoreWritePhaseKind::RecordCreate,
            RestoreWritePhaseKind::RecordVerify,
            "record_verify requires record_create",
        ),
        (
            RestoreWritePhaseKind::RecordVerify,
            RestoreWritePhaseKind::LinkedRecordUpdate,
            "linked_record_update requires record_verify",
        ),
        (
            RestoreWritePhaseKind::LinkedRecordUpdate,
            RestoreWritePhaseKind::LinkedRecordVerify,
            "linked_record_verify requires linked_record_update",
        ),
        (
            RestoreWritePhaseKind::LinkedRecordVerify,
            RestoreWritePhaseKind::FinalValidation,
            "final_validation requires linked_record_verify",
        ),
    ];

    let mut prereq_violation: Option<String> = None;
    for (prereq, dependent, description) in prerequisite_pairs {
        let dependent_status = find_status(dependent);
        let prereq_status = find_status(prereq);

        if let Some(dep_st) = dependent_status {
            if is_active(dep_st) && prereq_status.is_none() {
                prereq_violation = Some(format!(
                    "Phase '{}' is active but its prerequisite '{}' is not declared. {}.",
                    phase_label(dependent),
                    phase_label(prereq),
                    description
                ));
                break;
            }
        }
    }

    if let Some(msg) = prereq_violation {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-04".to_string(),
            label: "prerequisite-phases-present".to_string(),
            status: WritePhaseOrderingCheckStatus::Failed,
            message: msg,
            remediation: Some(
                "Ensure all prerequisite phases are declared before their dependent phases."
                    .to_string(),
            ),
        });
    } else {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-04".to_string(),
            label: "prerequisite-phases-present".to_string(),
            status: WritePhaseOrderingCheckStatus::Passed,
            message: "All active phases have their prerequisites declared.".to_string(),
            remediation: None,
        });
    }

    // WPO-05: record_create not active before schema_verify is completed
    let record_create_active = find_status(&RestoreWritePhaseKind::RecordCreate)
        .map(is_active)
        .unwrap_or(false);
    let schema_verify_completed = find_status(&RestoreWritePhaseKind::SchemaVerify)
        .map(|s| s == &RestoreWritePhaseStatus::Completed)
        .unwrap_or(false);

    if record_create_active && !schema_verify_completed {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-05".to_string(),
            label: "record-create-after-schema-verify".to_string(),
            status: WritePhaseOrderingCheckStatus::Failed,
            message: "record_create is active but schema_verify is not completed. Records \
                       cannot be created before the schema is verified."
                .to_string(),
            remediation: Some(
                "Ensure schema_verify reaches Completed status before record_create is set to \
                 Ready or Completed."
                    .to_string(),
            ),
        });
    } else {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-05".to_string(),
            label: "record-create-after-schema-verify".to_string(),
            status: WritePhaseOrderingCheckStatus::Passed,
            message: "record_create ordering relative to schema_verify is safe.".to_string(),
            remediation: None,
        });
    }

    // WPO-06: linked_record_update not active before record_verify is completed
    let linked_update_active = find_status(&RestoreWritePhaseKind::LinkedRecordUpdate)
        .map(is_active)
        .unwrap_or(false);
    let record_verify_completed = find_status(&RestoreWritePhaseKind::RecordVerify)
        .map(|s| s == &RestoreWritePhaseStatus::Completed)
        .unwrap_or(false);

    if linked_update_active && !record_verify_completed {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-06".to_string(),
            label: "linked-update-after-record-verify".to_string(),
            status: WritePhaseOrderingCheckStatus::Failed,
            message: "linked_record_update is active but record_verify is not completed. \
                       Linked record updates cannot begin before first-pass records are verified."
                .to_string(),
            remediation: Some(
                "Ensure record_verify reaches Completed status before linked_record_update is \
                 set to Ready or Completed."
                    .to_string(),
            ),
        });
    } else {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-06".to_string(),
            label: "linked-update-after-record-verify".to_string(),
            status: WritePhaseOrderingCheckStatus::Passed,
            message: "linked_record_update ordering relative to record_verify is safe.".to_string(),
            remediation: None,
        });
    }

    // WPO-07: final_validation not active before linked_record_verify is completed
    let final_validation_active = find_status(&RestoreWritePhaseKind::FinalValidation)
        .map(is_active)
        .unwrap_or(false);
    let linked_verify_completed = find_status(&RestoreWritePhaseKind::LinkedRecordVerify)
        .map(|s| s == &RestoreWritePhaseStatus::Completed)
        .unwrap_or(false);

    if final_validation_active && !linked_verify_completed {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-07".to_string(),
            label: "final-validation-after-linked-verify".to_string(),
            status: WritePhaseOrderingCheckStatus::Failed,
            message: "final_validation is active but linked_record_verify is not completed. \
                       Final validation cannot run before all linked record updates are verified."
                .to_string(),
            remediation: Some(
                "Ensure linked_record_verify reaches Completed status before final_validation \
                 is set to Ready or Completed."
                    .to_string(),
            ),
        });
    } else {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-07".to_string(),
            label: "final-validation-after-linked-verify".to_string(),
            status: WritePhaseOrderingCheckStatus::Passed,
            message: "final_validation ordering relative to linked_record_verify is safe."
                .to_string(),
            remediation: None,
        });
    }

    // WPO-08: No attachment upload or binary handling phase.
    //
    // For non-metadata phases: any skip_reason containing "upload", "binary",
    // or "download" is blocked — those words have no safe interpretation on a
    // phase that should never touch attachment files.
    //
    // For AttachmentMetadataVerify specifically: safe descriptive language such
    // as "files not downloaded" or "metadata-only" is permitted in a skip_reason
    // because that phase is defined to be metadata-only.  We block only on
    // language that explicitly demands binary handling:
    //   - "upload required" / "binary upload"
    //   - "download required" / "binary download"
    //   - "file transfer" / "attachment body"
    let unsafe_metadata_phase_language = |r: &str| -> bool {
        let lower = r.to_lowercase();
        lower.contains("upload required")
            || lower.contains("binary upload")
            || lower.contains("download required")
            || lower.contains("binary download")
            || lower.contains("file transfer")
            || lower.contains("attachment body")
    };
    let unsafe_general_language = |r: &str| -> bool {
        let lower = r.to_lowercase();
        lower.contains("upload") || lower.contains("binary") || lower.contains("download")
    };
    let has_attachment_upload_language = phases.iter().any(|p| {
        p.skip_reason
            .as_deref()
            .map(|r| {
                if p.kind == RestoreWritePhaseKind::AttachmentMetadataVerify {
                    unsafe_metadata_phase_language(r)
                } else {
                    unsafe_general_language(r)
                }
            })
            .unwrap_or(false)
    });

    if has_attachment_upload_language {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-08".to_string(),
            label: "no-attachment-upload-phase".to_string(),
            status: WritePhaseOrderingCheckStatus::Failed,
            message: "A phase skip reason contains attachment upload, binary, or download \
                       language. Attachment file upload and binary handling are not permitted in \
                       the restore write pipeline."
                .to_string(),
            remediation: Some(
                "Remove any attachment upload or binary download phase. The restore pipeline \
                 supports only attachment metadata validation (no file download)."
                    .to_string(),
            ),
        });
    } else {
        checks.push(WritePhaseOrderingCheck {
            check_id: "WPO-08".to_string(),
            label: "no-attachment-upload-phase".to_string(),
            status: WritePhaseOrderingCheckStatus::Passed,
            message: "No attachment upload or binary handling phase is declared.".to_string(),
            remediation: None,
        });
    }

    // WPO-09: attachment_metadata_verify skipped with metadata-only reason → warning
    let attachment_phase = phases
        .iter()
        .find(|p| p.kind == RestoreWritePhaseKind::AttachmentMetadataVerify);

    match attachment_phase {
        Some(p) if p.status == RestoreWritePhaseStatus::Skipped => {
            let has_metadata_reason = p
                .skip_reason
                .as_deref()
                .map(|r| r.to_lowercase().contains("metadata"))
                .unwrap_or(false);

            if has_metadata_reason {
                checks.push(WritePhaseOrderingCheck {
                    check_id: "WPO-09".to_string(),
                    label: "attachment-metadata-verify-scope".to_string(),
                    status: WritePhaseOrderingCheckStatus::Warning,
                    message: "attachment_metadata_verify is skipped with a metadata-only reason. \
                               Attachment file content integrity cannot be confirmed. Manual \
                               re-attachment is required after restore."
                        .to_string(),
                    remediation: Some(
                        "Note that attachment file content is not validated. Plan for manual \
                         re-attachment of all attachment fields after restore."
                            .to_string(),
                    ),
                });
            } else {
                checks.push(WritePhaseOrderingCheck {
                    check_id: "WPO-09".to_string(),
                    label: "attachment-metadata-verify-scope".to_string(),
                    status: WritePhaseOrderingCheckStatus::Warning,
                    message: "attachment_metadata_verify is skipped without a metadata-only \
                               explanation. Provide a skip reason to document why attachment \
                               verification was omitted."
                        .to_string(),
                    remediation: Some(
                        "Add a skip_reason to the attachment_metadata_verify phase explaining \
                         why it was skipped."
                            .to_string(),
                    ),
                });
            }
        }
        _ => {
            checks.push(WritePhaseOrderingCheck {
                check_id: "WPO-09".to_string(),
                label: "attachment-metadata-verify-scope".to_string(),
                status: WritePhaseOrderingCheckStatus::Passed,
                message: "attachment_metadata_verify is declared and not skipped.".to_string(),
                remediation: None,
            });
        }
    }

    // WPO-10: Writes remain disabled
    checks.push(WritePhaseOrderingCheck {
        check_id: "WPO-10".to_string(),
        label: "writes-remain-disabled".to_string(),
        status: WritePhaseOrderingCheckStatus::Passed,
        message: "Restore write gate remains disabled. Write phase ordering policy compliance \
                   does not enable writes."
            .to_string(),
        remediation: None,
    });

    // Aggregate status
    let has_failed = checks
        .iter()
        .any(|c| c.status == WritePhaseOrderingCheckStatus::Failed);
    let has_warning = checks
        .iter()
        .any(|c| c.status == WritePhaseOrderingCheckStatus::Warning);

    let aggregate_status = if has_failed {
        WritePhaseOrderingPolicyStatus::Blocked
    } else if has_warning {
        WritePhaseOrderingPolicyStatus::Warning
    } else {
        WritePhaseOrderingPolicyStatus::Compliant
    };

    build_result(
        checks,
        Some(phases),
        aggregate_status,
        request.target_label.as_deref(),
    )
}

// ── Builder ───────────────────────────────────────────────────────────────────

fn build_result(
    checks: Vec<WritePhaseOrderingCheck>,
    phases: Option<&Vec<DeclaredWritePhase>>,
    status: WritePhaseOrderingPolicyStatus,
    target_label: Option<&str>,
) -> WritePhaseOrderingPolicyResult {
    let label_suffix = target_label
        .map(|l| format!(" for target \"{l}\""))
        .unwrap_or_default();

    let message = match status {
        WritePhaseOrderingPolicyStatus::Compliant => {
            format!(
                "Write phase ordering is compliant{label_suffix}. All phases are declared in \
                 canonical order with no unsafe transitions. Restore writes remain disabled — \
                 compliance does not enable writes or introduce a restore success state."
            )
        }
        WritePhaseOrderingPolicyStatus::Warning => {
            format!(
                "Write phase ordering has warnings{label_suffix}. Review skipped or incomplete \
                 phases before proceeding. Restore writes remain disabled."
            )
        }
        WritePhaseOrderingPolicyStatus::Blocked => {
            format!(
                "Write phase ordering is blocked{label_suffix}. Resolve all phase ordering \
                 violations before any live write is considered. Restore writes remain disabled."
            )
        }
    };

    let phase_summary = phases.map(|ps| {
        ps.iter()
            .map(|p| WritePhaseOrderingSummaryEntry {
                kind: p.kind.clone(),
                status: p.status.clone(),
                canonical_position: canonical_position(&p.kind)
                    .map(|pos| (pos + 1) as u8)
                    .unwrap_or(0),
                skip_reason: p.skip_reason.clone(),
            })
            .collect()
    });

    WritePhaseOrderingPolicyResult {
        status,
        checks,
        message,
        phase_summary,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_phases() -> Vec<DeclaredWritePhase> {
        vec![
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::Preflight,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::SchemaCreate,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::SchemaVerify,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::RecordCreate,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::RecordVerify,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::LinkedRecordUpdate,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::LinkedRecordVerify,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::AttachmentMetadataVerify,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::FinalValidation,
                status: RestoreWritePhaseStatus::Planned,
                skip_reason: None,
            },
        ]
    }

    fn request_with_phases(phases: Vec<DeclaredWritePhase>) -> WritePhaseOrderingPolicyRequest {
        WritePhaseOrderingPolicyRequest {
            phases: Some(phases),
            target_label: None,
        }
    }

    fn request_no_phases() -> WritePhaseOrderingPolicyRequest {
        WritePhaseOrderingPolicyRequest {
            phases: None,
            target_label: None,
        }
    }

    // ── Status ────────────────────────────────────────────────────────────────

    #[test]
    fn canonical_order_compliant() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Compliant);
    }

    #[test]
    fn no_phases_blocked() {
        let result = verify_write_phase_ordering_policy(&request_no_phases());
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
    }

    #[test]
    fn record_create_before_schema_verify_blocked() {
        // record_create Ready, schema_verify only Planned (not Completed)
        let mut phases = canonical_phases();
        // Set schema_verify to Planned
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::SchemaVerify {
                p.status = RestoreWritePhaseStatus::Planned;
            }
            // record_create remains Completed — triggers WPO-05
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
        let wpo05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-05")
            .unwrap();
        assert_eq!(wpo05.status, WritePhaseOrderingCheckStatus::Failed);
    }

    #[test]
    fn linked_record_update_before_record_verify_blocked() {
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::RecordVerify {
                p.status = RestoreWritePhaseStatus::Planned;
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
        let wpo06 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-06")
            .unwrap();
        assert_eq!(wpo06.status, WritePhaseOrderingCheckStatus::Failed);
    }

    #[test]
    fn final_validation_before_linked_verify_blocked() {
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::LinkedRecordVerify {
                p.status = RestoreWritePhaseStatus::Planned;
            }
            if p.kind == RestoreWritePhaseKind::FinalValidation {
                p.status = RestoreWritePhaseStatus::Ready;
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
        let wpo07 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-07")
            .unwrap();
        assert_eq!(wpo07.status, WritePhaseOrderingCheckStatus::Failed);
    }

    #[test]
    fn attachment_upload_language_in_non_metadata_phase_blocked() {
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::RecordCreate {
                p.skip_reason = Some("attachment binary upload required".to_string());
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
        let wpo08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-08")
            .unwrap();
        assert_eq!(wpo08.status, WritePhaseOrderingCheckStatus::Failed);
    }

    #[test]
    fn attachment_binary_download_required_in_metadata_phase_blocked() {
        // "binary download required" inside AttachmentMetadataVerify must be blocked.
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::AttachmentMetadataVerify {
                p.skip_reason = Some("attachment binary download required".to_string());
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
        let wpo08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-08")
            .unwrap();
        assert_eq!(wpo08.status, WritePhaseOrderingCheckStatus::Failed);
    }

    #[test]
    fn metadata_only_files_not_downloaded_in_metadata_phase_is_not_blocked() {
        // "metadata-only: files not downloaded" is safe descriptive language, not a demand
        // for binary handling. WPO-08 must pass; WPO-09 produces Warning.
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::AttachmentMetadataVerify {
                p.status = RestoreWritePhaseStatus::Skipped;
                p.skip_reason = Some("metadata-only: files not downloaded".to_string());
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        // WPO-08 must pass
        let wpo08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-08")
            .unwrap();
        assert_eq!(wpo08.status, WritePhaseOrderingCheckStatus::Passed);
        // Overall must be Warning (from WPO-09), not Blocked
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Warning);
    }

    #[test]
    fn upload_required_in_metadata_phase_blocked() {
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::AttachmentMetadataVerify {
                p.skip_reason = Some("upload required for this phase".to_string());
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
        let wpo08 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-08")
            .unwrap();
        assert_eq!(wpo08.status, WritePhaseOrderingCheckStatus::Failed);
    }

    #[test]
    fn metadata_only_attachment_verify_skipped_warning() {
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::AttachmentMetadataVerify {
                p.status = RestoreWritePhaseStatus::Skipped;
                p.skip_reason = Some("metadata-only: attachment files not downloaded".to_string());
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Warning);
        let wpo09 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-09")
            .unwrap();
        assert_eq!(wpo09.status, WritePhaseOrderingCheckStatus::Warning);
    }

    #[test]
    fn skipped_attachment_verify_without_metadata_reason_warning() {
        let mut phases = canonical_phases();
        for p in phases.iter_mut() {
            if p.kind == RestoreWritePhaseKind::AttachmentMetadataVerify {
                p.status = RestoreWritePhaseStatus::Skipped;
                p.skip_reason = None;
            }
        }
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Warning);
    }

    #[test]
    fn out_of_order_phases_blocked() {
        // Place linked_record_update before record_create
        let phases = vec![
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::Preflight,
                status: RestoreWritePhaseStatus::Completed,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::LinkedRecordUpdate,
                status: RestoreWritePhaseStatus::Planned,
                skip_reason: None,
            },
            DeclaredWritePhase {
                kind: RestoreWritePhaseKind::RecordCreate,
                status: RestoreWritePhaseStatus::Planned,
                skip_reason: None,
            },
        ];
        let result = verify_write_phase_ordering_policy(&request_with_phases(phases));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Blocked);
        let wpo03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-03")
            .unwrap();
        assert_eq!(wpo03.status, WritePhaseOrderingCheckStatus::Failed);
    }

    // ── Check count ───────────────────────────────────────────────────────────

    #[test]
    fn canonical_phases_produce_10_checks() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert_eq!(result.checks.len(), 10);
    }

    #[test]
    fn no_phases_produce_2_checks() {
        let result = verify_write_phase_ordering_policy(&request_no_phases());
        assert_eq!(result.checks.len(), 2);
    }

    // ── Check IDs ─────────────────────────────────────────────────────────────

    #[test]
    fn check_ids_are_wpo_01_through_wpo_10() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "WPO-01", "WPO-02", "WPO-03", "WPO-04", "WPO-05", "WPO-06", "WPO-07", "WPO-08",
                "WPO-09", "WPO-10"
            ]
        );
    }

    #[test]
    fn no_phases_check_ids_are_wpo_01_and_wpo_02() {
        let result = verify_write_phase_ordering_policy(&request_no_phases());
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert_eq!(ids, vec!["WPO-01", "WPO-02"]);
    }

    // ── WPO-01 and WPO-10 always pass ─────────────────────────────────────────

    #[test]
    fn wpo_01_always_passes_with_phases() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        let wpo01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-01")
            .unwrap();
        assert_eq!(wpo01.status, WritePhaseOrderingCheckStatus::Passed);
    }

    #[test]
    fn wpo_01_always_passes_without_phases() {
        let result = verify_write_phase_ordering_policy(&request_no_phases());
        let wpo01 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-01")
            .unwrap();
        assert_eq!(wpo01.status, WritePhaseOrderingCheckStatus::Passed);
    }

    #[test]
    fn wpo_10_always_passes() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        let wpo10 = result
            .checks
            .iter()
            .find(|c| c.check_id == "WPO-10")
            .unwrap();
        assert_eq!(wpo10.status, WritePhaseOrderingCheckStatus::Passed);
    }

    // ── Phase summary ─────────────────────────────────────────────────────────

    #[test]
    fn phase_summary_present_when_phases_provided() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert!(result.phase_summary.is_some());
        assert_eq!(result.phase_summary.unwrap().len(), 9);
    }

    #[test]
    fn phase_summary_absent_when_no_phases() {
        let result = verify_write_phase_ordering_policy(&request_no_phases());
        assert!(result.phase_summary.is_none());
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true_compliant() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert!(result.no_changes_made);
    }

    #[test]
    fn no_changes_made_always_true_blocked() {
        let result = verify_write_phase_ordering_policy(&request_no_phases());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false_compliant() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert!(!result.writes_enabled);
    }

    #[test]
    fn writes_enabled_always_false_blocked() {
        let result = verify_write_phase_ordering_policy(&request_no_phases());
        assert!(!result.writes_enabled);
    }

    // ── Serialization safety ──────────────────────────────────────────────────

    #[test]
    fn result_does_not_serialize_token() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("pat_"));
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn result_does_not_serialize_path() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\\\"));
    }

    #[test]
    fn result_does_not_serialize_record_payload() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("\"fields\""));
        assert!(!json.contains("\"recordId\""));
    }

    // ── Message safety ────────────────────────────────────────────────────────

    #[test]
    fn message_says_writes_remain_disabled_when_compliant() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert_eq!(result.status, WritePhaseOrderingPolicyStatus::Compliant);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn message_does_not_contain_succeeded_when_compliant() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert!(!result.message.to_lowercase().contains("succeeded"));
    }

    #[test]
    fn message_does_not_contain_token() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert!(!result.message.contains("pat_"));
        assert!(!result.message.contains("apiKey"));
    }

    #[test]
    fn message_does_not_contain_absolute_path() {
        let result = verify_write_phase_ordering_policy(&request_with_phases(canonical_phases()));
        assert!(!result.message.contains("/Users/"));
        assert!(!result.message.contains("/home/"));
    }
}
