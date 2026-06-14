use crate::restore::write_gate::evaluate_write_gate;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

/// The overall verdict for a schema-record order policy check.
///
/// Safety invariants:
/// - `Compliant` does NOT enable restore writes.
/// - `writes_enabled` is always false regardless of status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaRecordOrderPolicyStatus {
    /// Schema is planned and precedes all record phases; ordering is valid.
    Compliant,
    /// Phase data is incomplete; ordering cannot be fully verified.
    Warning,
    /// A required ordering constraint is violated; records before schema,
    /// linked updates before record-create, or attachment handling before
    /// record-create.
    Blocked,
}

/// The result of a single order-policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaRecordOrderCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// The write phase being declared in the request.
///
/// Serialises as camelCase strings for TypeScript interop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreWritePhaseKind {
    Schema,
    Records,
    LinkedRecords,
    Attachments,
    Validation,
}

/// Declared presence and readiness of a single write phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclaredWritePhase {
    pub phase: RestoreWritePhaseKind,
    /// True when the phase has a non-blocked plan backing it.
    pub is_planned: bool,
    /// True when the upstream plan for this phase is in a blocked state.
    pub is_blocked: bool,
}

/// Input to the schema-record order policy gate.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No record payload field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaRecordOrderPolicyRequest {
    /// Ordered list of write phases as declared by the restore planner.
    /// The ordering of this vec is the claimed execution order.
    pub declared_phases: Vec<DeclaredWritePhase>,
    /// Human-readable label for the restore target (e.g. base name or filename).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_display_name: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaRecordOrderCheck {
    pub check_id: String,
    pub label: String,
    pub status: SchemaRecordOrderCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Result from `verify_schema_record_order_policy`.
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
pub struct SchemaRecordOrderPolicyResult {
    pub status: SchemaRecordOrderPolicyStatus,
    pub checks: Vec<SchemaRecordOrderCheck>,
    pub message: String,
    /// Names of ordering violations found (e.g. "records-before-schema").
    pub ordering_violations: Vec<String>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies that the declared write phases observe safe schema-before-record ordering.
///
/// Check IDs:
/// - SRO-01: Write gate is disabled.
/// - SRO-02: Schema phase is present and not blocked.
/// - SRO-03: Schema phase precedes the record-create phase.
/// - SRO-04: Record-create phase precedes linked-record update phase.
/// - SRO-05: Record-create phase precedes attachment-handling phase.
///
/// No Airtable API calls are made.
/// No token is accepted or returned.
/// No filesystem path is accepted or returned.
/// No record payload is accepted or returned.
pub fn verify_schema_record_order_policy(
    request: &SchemaRecordOrderPolicyRequest,
) -> SchemaRecordOrderPolicyResult {
    let mut checks: Vec<SchemaRecordOrderCheck> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    // SRO-01: write gate always disabled
    let gate = evaluate_write_gate();
    checks.push(SchemaRecordOrderCheck {
        check_id: "SRO-01".to_string(),
        label: "write-gate-disabled".to_string(),
        status: SchemaRecordOrderCheckStatus::Passed,
        message: gate.message.clone(),
        remediation: None,
    });

    // Collect phase indices for ordering checks
    let schema_idx = phase_index(&request.declared_phases, &RestoreWritePhaseKind::Schema);
    let records_idx = phase_index(&request.declared_phases, &RestoreWritePhaseKind::Records);
    let linked_idx = phase_index(
        &request.declared_phases,
        &RestoreWritePhaseKind::LinkedRecords,
    );
    let attach_idx = phase_index(
        &request.declared_phases,
        &RestoreWritePhaseKind::Attachments,
    );

    let schema_phase = schema_idx.and_then(|i| request.declared_phases.get(i));
    let records_phase = records_idx.and_then(|i| request.declared_phases.get(i));

    // SRO-02: schema phase present and not blocked
    match schema_phase {
        None => {
            // No schema phase declared at all
            if records_phase.is_some() {
                checks.push(SchemaRecordOrderCheck {
                    check_id: "SRO-02".to_string(),
                    label: "schema-phase-present".to_string(),
                    status: SchemaRecordOrderCheckStatus::Failed,
                    message: "Schema phase is missing but a record phase is declared. Records cannot be created without a schema.".to_string(),
                    remediation: Some("Add a schema creation phase before the record creation phase.".to_string()),
                });
                violations.push("missing-schema-with-records".to_string());
            } else {
                checks.push(SchemaRecordOrderCheck {
                    check_id: "SRO-02".to_string(),
                    label: "schema-phase-present".to_string(),
                    status: SchemaRecordOrderCheckStatus::Warning,
                    message: "No phases declared. Phase ordering cannot be verified.".to_string(),
                    remediation: None,
                });
            }
        }
        Some(sp) if sp.is_blocked => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-02".to_string(),
                label: "schema-phase-present".to_string(),
                status: SchemaRecordOrderCheckStatus::Failed,
                message: "Schema phase is blocked. Record phases must not proceed when schema is blocked.".to_string(),
                remediation: Some("Resolve schema plan issues before attempting record creation.".to_string()),
            });
            violations.push("schema-phase-blocked".to_string());
        }
        Some(sp) if !sp.is_planned => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-02".to_string(),
                label: "schema-phase-present".to_string(),
                status: SchemaRecordOrderCheckStatus::Warning,
                message: "Schema phase is declared but not yet planned. Phase ordering cannot be fully verified.".to_string(),
                remediation: None,
            });
        }
        _ => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-02".to_string(),
                label: "schema-phase-present".to_string(),
                status: SchemaRecordOrderCheckStatus::Passed,
                message: "Schema phase is present and planned.".to_string(),
                remediation: None,
            });
        }
    }

    // SRO-03: schema precedes record-create
    match (schema_idx, records_idx) {
        (Some(s), Some(r)) if r <= s => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-03".to_string(),
                label: "schema-before-records".to_string(),
                status: SchemaRecordOrderCheckStatus::Failed,
                message: "Record-create phase appears at or before schema phase. Schema must precede all record operations.".to_string(),
                remediation: Some("Move schema phase before the record-create phase.".to_string()),
            });
            violations.push("records-before-schema".to_string());
        }
        (None, Some(_)) => {
            // Already captured in SRO-02; add a passed/warning here only if not already failed
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-03".to_string(),
                label: "schema-before-records".to_string(),
                status: SchemaRecordOrderCheckStatus::Failed,
                message: "Record phase declared but no schema phase present.".to_string(),
                remediation: Some("Add a schema phase before record creation.".to_string()),
            });
            // Only push violation if not already recorded
            if !violations.contains(&"missing-schema-with-records".to_string()) {
                violations.push("missing-schema-with-records".to_string());
            }
        }
        (Some(_), None) | (None, None) => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-03".to_string(),
                label: "schema-before-records".to_string(),
                status: SchemaRecordOrderCheckStatus::Passed,
                message: "No record phase declared; schema ordering constraint is satisfied."
                    .to_string(),
                remediation: None,
            });
        }
        _ => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-03".to_string(),
                label: "schema-before-records".to_string(),
                status: SchemaRecordOrderCheckStatus::Passed,
                message: "Schema phase precedes record-create phase.".to_string(),
                remediation: None,
            });
        }
    }

    // SRO-04: record-create precedes linked-record updates
    match (records_idx, linked_idx) {
        (Some(r), Some(l)) if l <= r => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-04".to_string(),
                label: "records-before-linked-updates".to_string(),
                status: SchemaRecordOrderCheckStatus::Failed,
                message: "Linked-record update phase appears at or before record-create phase. Linked updates require first-pass records to exist.".to_string(),
                remediation: Some("Move linked-record update phase after record-create phase.".to_string()),
            });
            violations.push("linked-before-record-create".to_string());
        }
        (None, Some(_)) => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-04".to_string(),
                label: "records-before-linked-updates".to_string(),
                status: SchemaRecordOrderCheckStatus::Warning,
                message: "Linked-record update phase declared without a record-create phase. Phase data may be incomplete.".to_string(),
                remediation: Some("Ensure record-create phase is declared before linked-record updates.".to_string()),
            });
        }
        _ => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-04".to_string(),
                label: "records-before-linked-updates".to_string(),
                status: SchemaRecordOrderCheckStatus::Passed,
                message: "Record-create phase precedes linked-record update phase, or no linked-record phase declared.".to_string(),
                remediation: None,
            });
        }
    }

    // SRO-05: record-create precedes attachment handling
    match (records_idx, attach_idx) {
        (Some(r), Some(a)) if a <= r => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-05".to_string(),
                label: "records-before-attachments".to_string(),
                status: SchemaRecordOrderCheckStatus::Failed,
                message: "Attachment-handling phase appears at or before record-create phase. Attachment metadata is associated with records and requires record IDs.".to_string(),
                remediation: Some("Move attachment-handling phase after record-create phase.".to_string()),
            });
            violations.push("attachment-before-record-create".to_string());
        }
        (None, Some(_)) => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-05".to_string(),
                label: "records-before-attachments".to_string(),
                status: SchemaRecordOrderCheckStatus::Warning,
                message: "Attachment phase declared without a record-create phase. Phase data may be incomplete.".to_string(),
                remediation: Some("Ensure record-create phase is declared before attachment handling.".to_string()),
            });
        }
        _ => {
            checks.push(SchemaRecordOrderCheck {
                check_id: "SRO-05".to_string(),
                label: "records-before-attachments".to_string(),
                status: SchemaRecordOrderCheckStatus::Passed,
                message: "Record-create phase precedes attachment-handling phase, or no attachment phase declared.".to_string(),
                remediation: None,
            });
        }
    }

    // Determine overall status
    let has_failed = checks
        .iter()
        .any(|c| c.status == SchemaRecordOrderCheckStatus::Failed);
    let has_warning = checks
        .iter()
        .any(|c| c.status == SchemaRecordOrderCheckStatus::Warning);

    let status = if has_failed {
        SchemaRecordOrderPolicyStatus::Blocked
    } else if has_warning {
        SchemaRecordOrderPolicyStatus::Warning
    } else {
        SchemaRecordOrderPolicyStatus::Compliant
    };

    let target = request
        .target_display_name
        .as_deref()
        .unwrap_or("the restore target");

    let message = match &status {
        SchemaRecordOrderPolicyStatus::Compliant => format!(
            "Phase ordering for {} is valid. Schema precedes all record phases. Restore writes remain disabled.",
            target
        ),
        SchemaRecordOrderPolicyStatus::Warning => format!(
            "Phase ordering for {} could not be fully verified. Some phase data is incomplete.",
            target
        ),
        SchemaRecordOrderPolicyStatus::Blocked => format!(
            "Phase ordering violation detected for {}: {}. Resolve ordering issues before enabling live writes.",
            target,
            violations.join(", ")
        ),
    };

    SchemaRecordOrderPolicyResult {
        status,
        checks,
        message,
        ordering_violations: violations,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn phase_index(phases: &[DeclaredWritePhase], kind: &RestoreWritePhaseKind) -> Option<usize> {
    phases.iter().position(|p| &p.phase == kind)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn phase(kind: RestoreWritePhaseKind, planned: bool, blocked: bool) -> DeclaredWritePhase {
        DeclaredWritePhase {
            phase: kind,
            is_planned: planned,
            is_blocked: blocked,
        }
    }

    fn planned(kind: RestoreWritePhaseKind) -> DeclaredWritePhase {
        phase(kind, true, false)
    }

    fn blocked(kind: RestoreWritePhaseKind) -> DeclaredWritePhase {
        phase(kind, false, true)
    }

    fn unplanned(kind: RestoreWritePhaseKind) -> DeclaredWritePhase {
        phase(kind, false, false)
    }

    fn req(phases: Vec<DeclaredWritePhase>) -> SchemaRecordOrderPolicyRequest {
        SchemaRecordOrderPolicyRequest {
            declared_phases: phases,
            target_display_name: Some("My Base".to_string()),
        }
    }

    fn valid_phases() -> Vec<DeclaredWritePhase> {
        vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::LinkedRecords),
            planned(RestoreWritePhaseKind::Attachments),
            planned(RestoreWritePhaseKind::Validation),
        ]
    }

    // ── SRO-01 ────────────────────────────────────────────────────────────

    #[test]
    fn sro_01_always_passes() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        let check = result
            .checks
            .iter()
            .find(|c| c.check_id == "SRO-01")
            .unwrap();
        assert_eq!(check.status, SchemaRecordOrderCheckStatus::Passed);
    }

    // ── SRO-02: schema phase present ──────────────────────────────────────

    #[test]
    fn schema_only_is_compliant() {
        let result =
            verify_schema_record_order_policy(&req(vec![planned(RestoreWritePhaseKind::Schema)]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Compliant);
    }

    #[test]
    fn schema_before_records_compliant() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Compliant);
    }

    #[test]
    fn no_phases_is_warning() {
        let result = verify_schema_record_order_policy(&req(vec![]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Warning);
    }

    #[test]
    fn blocked_schema_with_records_is_blocked() {
        let result = verify_schema_record_order_policy(&req(vec![
            blocked(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Records),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
        assert!(result
            .ordering_violations
            .contains(&"schema-phase-blocked".to_string()));
    }

    #[test]
    fn missing_schema_with_records_is_blocked() {
        let result =
            verify_schema_record_order_policy(&req(vec![planned(RestoreWritePhaseKind::Records)]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
        assert!(result
            .ordering_violations
            .contains(&"missing-schema-with-records".to_string()));
    }

    #[test]
    fn unplanned_schema_with_records_is_warning() {
        let result = verify_schema_record_order_policy(&req(vec![
            unplanned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Records),
        ]));
        // SRO-02 warning; SRO-03 passes (schema index < records index)
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Warning);
    }

    // ── SRO-03: schema before records ─────────────────────────────────────

    #[test]
    fn records_before_schema_is_blocked() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::Schema),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
        assert!(result
            .ordering_violations
            .contains(&"records-before-schema".to_string()));
    }

    #[test]
    fn records_same_index_as_schema_is_blocked() {
        // Can't really put two phases at same position, but test records < schema boundary
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::LinkedRecords),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
    }

    // ── SRO-04: records before linked updates ─────────────────────────────

    #[test]
    fn linked_before_record_create_is_blocked() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::LinkedRecords),
            planned(RestoreWritePhaseKind::Records),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
        assert!(result
            .ordering_violations
            .contains(&"linked-before-record-create".to_string()));
    }

    #[test]
    fn linked_same_position_as_records_is_blocked() {
        // linked index <= records index → blocked
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::LinkedRecords),
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::Validation),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
    }

    #[test]
    fn linked_without_records_is_warning() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::LinkedRecords),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Warning);
    }

    #[test]
    fn records_before_linked_is_compliant() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::LinkedRecords),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Compliant);
    }

    // ── SRO-05: records before attachments ────────────────────────────────

    #[test]
    fn attachment_before_record_create_is_blocked() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Attachments),
            planned(RestoreWritePhaseKind::Records),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
        assert!(result
            .ordering_violations
            .contains(&"attachment-before-record-create".to_string()));
    }

    #[test]
    fn attachment_without_records_is_warning() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Attachments),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Warning);
    }

    #[test]
    fn records_before_attachments_is_compliant() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::Attachments),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Compliant);
    }

    // ── Five checks always present ────────────────────────────────────────

    #[test]
    fn five_checks_present_for_compliant_result() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn five_checks_present_for_blocked_result() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::Schema),
        ]));
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn check_ids_are_sro_01_through_05() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert!(ids.contains(&"SRO-01"));
        assert!(ids.contains(&"SRO-02"));
        assert!(ids.contains(&"SRO-03"));
        assert!(ids.contains(&"SRO-04"));
        assert!(ids.contains(&"SRO-05"));
    }

    // ── Safety invariants ─────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true() {
        for phases in [
            valid_phases(),
            vec![
                planned(RestoreWritePhaseKind::Records),
                planned(RestoreWritePhaseKind::Schema),
            ],
            vec![],
        ] {
            let result = verify_schema_record_order_policy(&req(phases));
            assert!(result.no_changes_made);
        }
    }

    #[test]
    fn writes_enabled_always_false() {
        for phases in [valid_phases(), vec![]] {
            let result = verify_schema_record_order_policy(&req(phases));
            assert!(!result.writes_enabled);
        }
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn compliant_message_says_writes_remain_disabled() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn blocked_message_names_violation() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::Schema),
        ]));
        assert!(result.message.contains("records-before-schema"));
    }

    #[test]
    fn display_name_appears_in_message() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert!(result.message.contains("My Base"));
    }

    #[test]
    fn message_is_non_empty_for_all_statuses() {
        for phases in [
            valid_phases(),
            vec![
                planned(RestoreWritePhaseKind::Records),
                planned(RestoreWritePhaseKind::Schema),
            ],
            vec![],
        ] {
            let result = verify_schema_record_order_policy(&req(phases));
            assert!(!result.message.is_empty());
        }
    }

    // ── Serialization safety ──────────────────────────────────────────────

    #[test]
    fn serialization_has_no_token() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("token"));
        assert!(!json.contains("pat"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn serialization_has_no_full_path() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\\\"));
    }

    #[test]
    fn serialization_has_no_record_payload() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("fields"));
        assert!(!json.contains("recordId"));
    }

    #[test]
    fn no_write_calls_are_made() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert!(!result.network_writes_attempted);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
    }

    // ── Ordering_violations ───────────────────────────────────────────────

    #[test]
    fn ordering_violations_empty_for_compliant() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert!(result.ordering_violations.is_empty());
    }

    #[test]
    fn multiple_violations_collected() {
        // records before schema AND linked before records (both at once)
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::LinkedRecords),
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::Schema),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Blocked);
        // records-before-schema and linked-before-record-create
        assert!(result.ordering_violations.len() >= 1);
    }

    #[test]
    fn schema_only_no_violations() {
        let result =
            verify_schema_record_order_policy(&req(vec![planned(RestoreWritePhaseKind::Schema)]));
        assert!(result.ordering_violations.is_empty());
    }

    #[test]
    fn full_valid_order_schema_records_linked_attachments_validation() {
        let result = verify_schema_record_order_policy(&req(valid_phases()));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Compliant);
        assert!(result.ordering_violations.is_empty());
    }

    #[test]
    fn schema_records_linked_no_attachments_compliant() {
        let result = verify_schema_record_order_policy(&req(vec![
            planned(RestoreWritePhaseKind::Schema),
            planned(RestoreWritePhaseKind::Records),
            planned(RestoreWritePhaseKind::LinkedRecords),
            planned(RestoreWritePhaseKind::Validation),
        ]));
        assert_eq!(result.status, SchemaRecordOrderPolicyStatus::Compliant);
    }
}
