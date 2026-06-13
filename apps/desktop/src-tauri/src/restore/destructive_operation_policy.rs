use crate::restore::write_gate::evaluate_write_gate;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

/// The overall verdict for a destructive-operation policy check.
///
/// Safety invariants:
/// - `Compliant` does NOT enable restore writes.
/// - `writesEnabled` is always false regardless of status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DestructiveOperationPolicyStatus {
    /// All planned operations are create-only; no destructive operations detected.
    Compliant,
    /// One or more operations could not be classified; manual review recommended.
    Warning,
    /// One or more destructive, overwrite, or blocked operations detected.
    Blocked,
}

/// The result of a single policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DestructiveOperationCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// The category of a declared operation kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DestructiveOperationKind {
    /// Create a new Airtable base.
    DeleteBase,
    /// Drop an existing Airtable table.
    DeleteTable,
    /// Remove an existing Airtable field.
    DeleteField,
    /// Delete an Airtable record.
    DeleteRecord,
    /// Mutate an existing record's field values.
    UpdateExistingRecord,
    /// Overwrite an existing field definition.
    OverwriteField,
    /// Overwrite an existing table's schema.
    OverwriteTable,
    /// Upload attachment bytes to Airtable.
    AttachmentUpload,
    /// Create a new Airtable base.
    CreateBase,
    /// Create a new Airtable table.
    CreateTable,
    /// Create a new Airtable field.
    CreateField,
    /// Create new Airtable records (first-pass batch).
    CreateRecord,
    /// Update linked-record cross-references (second-pass).
    UpdateLinkedRecordReference,
    /// Metadata-only attachment preservation (no file bytes uploaded).
    PreserveAttachmentMetadata,
    /// Checkpoint marker — no Airtable call.
    Checkpoint,
    /// Field skipped; no Airtable call.
    SkipField,
    /// Requires manual intervention; not executed automatically.
    ManualAction,
    /// Deferred linked-field creation; not executed automatically.
    DeferLinkedField,
}

impl DestructiveOperationKind {
    /// Returns `true` if the kind is unconditionally blocked.
    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            DestructiveOperationKind::DeleteBase
                | DestructiveOperationKind::DeleteTable
                | DestructiveOperationKind::DeleteField
                | DestructiveOperationKind::DeleteRecord
                | DestructiveOperationKind::UpdateExistingRecord
                | DestructiveOperationKind::OverwriteField
                | DestructiveOperationKind::OverwriteTable
                | DestructiveOperationKind::AttachmentUpload
        )
    }

    /// Returns `true` if the kind is a safe create-only operation.
    pub fn is_create_only(&self) -> bool {
        matches!(
            self,
            DestructiveOperationKind::CreateBase
                | DestructiveOperationKind::CreateTable
                | DestructiveOperationKind::CreateField
                | DestructiveOperationKind::CreateRecord
                | DestructiveOperationKind::UpdateLinkedRecordReference
                | DestructiveOperationKind::PreserveAttachmentMetadata
                | DestructiveOperationKind::Checkpoint
                | DestructiveOperationKind::SkipField
                | DestructiveOperationKind::ManualAction
                | DestructiveOperationKind::DeferLinkedField
        )
    }
}

// ── Request / result types ────────────────────────────────────────────────────

/// A single declared operation to be policy-checked.
///
/// Safety: no token field, no path field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclaredOperation {
    pub kind: DestructiveOperationKind,
    pub label: String,
}

/// Request for Gate 4 destructive-operation policy verification.
///
/// Safety: no token field, no path field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestructiveOperationPolicyRequest {
    /// All planned operations from the schema and record write foundations.
    pub declared_operations: Vec<DeclaredOperation>,
    /// Human-readable description of the restore target (optional, for messages).
    pub target_display_name: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestructiveOperationCheck {
    pub check_id: String,
    pub label: String,
    pub status: DestructiveOperationCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Result from `verify_destructive_operation_policy`.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - `Compliant` does NOT enable restore writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestructiveOperationPolicyResult {
    pub status: DestructiveOperationPolicyStatus,
    pub checks: Vec<DestructiveOperationCheck>,
    pub message: String,
    pub blocked_operations: Vec<String>,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies that no destructive operations are present in the declared plan.
///
/// Check IDs:
/// - DOP-01: Write gate is disabled.
/// - DOP-02: No delete operations.
/// - DOP-03: No update/overwrite operations.
/// - DOP-04: No attachment upload operations.
/// - DOP-05: All remaining operations are create-only or safe.
///
/// No Airtable API calls are made.
/// No token is accepted or returned.
/// No filesystem path is accepted or returned.
pub fn verify_destructive_operation_policy(
    request: &DestructiveOperationPolicyRequest,
) -> DestructiveOperationPolicyResult {
    let mut checks: Vec<DestructiveOperationCheck> = Vec::new();
    let mut blocked_operations: Vec<String> = Vec::new();

    // DOP-01: write gate always disabled
    let gate = evaluate_write_gate();
    checks.push(DestructiveOperationCheck {
        check_id: "DOP-01".to_string(),
        label: "write-gate-disabled".to_string(),
        status: DestructiveOperationCheckStatus::Passed,
        message: gate.message.clone(),
        remediation: None,
    });

    // Classify all declared operations
    let mut has_delete = false;
    let mut has_update_overwrite = false;
    let mut has_attachment_upload = false;

    for op in &request.declared_operations {
        match op.kind {
            DestructiveOperationKind::DeleteBase
            | DestructiveOperationKind::DeleteTable
            | DestructiveOperationKind::DeleteField
            | DestructiveOperationKind::DeleteRecord => {
                has_delete = true;
                blocked_operations.push(op.label.clone());
            }
            DestructiveOperationKind::UpdateExistingRecord
            | DestructiveOperationKind::OverwriteField
            | DestructiveOperationKind::OverwriteTable => {
                has_update_overwrite = true;
                blocked_operations.push(op.label.clone());
            }
            DestructiveOperationKind::AttachmentUpload => {
                has_attachment_upload = true;
                blocked_operations.push(op.label.clone());
            }
            _ => {}
        }
    }

    // DOP-02: no delete operations
    if has_delete {
        let names: Vec<&str> = request
            .declared_operations
            .iter()
            .filter(|op| {
                matches!(
                    op.kind,
                    DestructiveOperationKind::DeleteBase
                        | DestructiveOperationKind::DeleteTable
                        | DestructiveOperationKind::DeleteField
                        | DestructiveOperationKind::DeleteRecord
                )
            })
            .map(|op| op.label.as_str())
            .collect();
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-02".to_string(),
            label: "no-delete-operations".to_string(),
            status: DestructiveOperationCheckStatus::Failed,
            message: format!(
                "Delete operations are not permitted during restore: {}.",
                names.join(", ")
            ),
            remediation: Some("Remove all delete operations from the restore plan.".to_string()),
        });
    } else {
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-02".to_string(),
            label: "no-delete-operations".to_string(),
            status: DestructiveOperationCheckStatus::Passed,
            message: "No delete operations declared.".to_string(),
            remediation: None,
        });
    }

    // DOP-03: no update/overwrite operations
    if has_update_overwrite {
        let names: Vec<&str> = request
            .declared_operations
            .iter()
            .filter(|op| {
                matches!(
                    op.kind,
                    DestructiveOperationKind::UpdateExistingRecord
                        | DestructiveOperationKind::OverwriteField
                        | DestructiveOperationKind::OverwriteTable
                )
            })
            .map(|op| op.label.as_str())
            .collect();
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-03".to_string(),
            label: "no-update-overwrite-operations".to_string(),
            status: DestructiveOperationCheckStatus::Failed,
            message: format!(
                "Update and overwrite operations are not permitted during restore: {}.",
                names.join(", ")
            ),
            remediation: Some(
                "Remove all update and overwrite operations from the restore plan.".to_string(),
            ),
        });
    } else {
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-03".to_string(),
            label: "no-update-overwrite-operations".to_string(),
            status: DestructiveOperationCheckStatus::Passed,
            message: "No update or overwrite operations declared.".to_string(),
            remediation: None,
        });
    }

    // DOP-04: no attachment upload operations
    if has_attachment_upload {
        let names: Vec<&str> = request
            .declared_operations
            .iter()
            .filter(|op| matches!(op.kind, DestructiveOperationKind::AttachmentUpload))
            .map(|op| op.label.as_str())
            .collect();
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-04".to_string(),
            label: "no-attachment-upload".to_string(),
            status: DestructiveOperationCheckStatus::Failed,
            message: format!(
                "Attachment upload operations are not permitted in this phase: {}.",
                names.join(", ")
            ),
            remediation: Some(
                "Attachment bytes cannot be uploaded during restore in this version. Only attachment metadata is preserved.".to_string(),
            ),
        });
    } else {
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-04".to_string(),
            label: "no-attachment-upload".to_string(),
            status: DestructiveOperationCheckStatus::Passed,
            message: "No attachment upload operations declared.".to_string(),
            remediation: None,
        });
    }

    // DOP-05: all remaining ops are create-only or safe
    let unknown_ops: Vec<&str> = request
        .declared_operations
        .iter()
        .filter(|op| !op.kind.is_blocked() && !op.kind.is_create_only())
        .map(|op| op.label.as_str())
        .collect();

    let has_unknown = !unknown_ops.is_empty();
    if has_unknown {
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-05".to_string(),
            label: "create-only-operations".to_string(),
            status: DestructiveOperationCheckStatus::Warning,
            message: format!(
                "Some operations could not be classified as create-only: {}.",
                unknown_ops.join(", ")
            ),
            remediation: Some(
                "Review unclassified operations before enabling live writes.".to_string(),
            ),
        });
    } else {
        checks.push(DestructiveOperationCheck {
            check_id: "DOP-05".to_string(),
            label: "create-only-operations".to_string(),
            status: DestructiveOperationCheckStatus::Passed,
            message: "All declared operations are create-only or safe.".to_string(),
            remediation: None,
        });
    }

    // Determine overall status
    let any_hard_fail = has_delete || has_update_overwrite || has_attachment_upload;
    let status = if any_hard_fail {
        DestructiveOperationPolicyStatus::Blocked
    } else if has_unknown {
        DestructiveOperationPolicyStatus::Warning
    } else {
        DestructiveOperationPolicyStatus::Compliant
    };

    let target_label = request
        .target_display_name
        .as_deref()
        .unwrap_or("the target base");

    let message = match status {
        DestructiveOperationPolicyStatus::Compliant => format!(
            "All declared operations for {} are create-only. No destructive operations detected. Restore writes remain disabled.",
            target_label
        ),
        DestructiveOperationPolicyStatus::Warning => format!(
            "Some operations for {} could not be classified. Manual review is required before enabling live writes.",
            target_label
        ),
        DestructiveOperationPolicyStatus::Blocked => format!(
            "Blocked operations detected for {}: {}. Remove all destructive operations before proceeding.",
            target_label,
            blocked_operations.join(", ")
        ),
    };

    DestructiveOperationPolicyResult {
        status,
        checks,
        message,
        blocked_operations,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_op(kind: DestructiveOperationKind, label: &str) -> DeclaredOperation {
        DeclaredOperation {
            kind,
            label: label.to_string(),
        }
    }

    fn request_with_ops(ops: Vec<DeclaredOperation>) -> DestructiveOperationPolicyRequest {
        DestructiveOperationPolicyRequest {
            declared_operations: ops,
            target_display_name: Some("My Base".to_string()),
        }
    }

    fn create_only_ops() -> Vec<DeclaredOperation> {
        vec![
            make_op(
                DestructiveOperationKind::CreateTable,
                "create-table-Projects",
            ),
            make_op(DestructiveOperationKind::CreateField, "create-field-Name"),
            make_op(
                DestructiveOperationKind::CreateRecord,
                "create-record-batch-1",
            ),
            make_op(
                DestructiveOperationKind::UpdateLinkedRecordReference,
                "update-linked-refs",
            ),
            make_op(
                DestructiveOperationKind::PreserveAttachmentMetadata,
                "preserve-attachment",
            ),
            make_op(DestructiveOperationKind::Checkpoint, "checkpoint-1"),
            make_op(DestructiveOperationKind::SkipField, "skip-formula"),
            make_op(DestructiveOperationKind::ManualAction, "manual-link"),
            make_op(DestructiveOperationKind::DeferLinkedField, "defer-linked"),
        ]
    }

    // ── Status paths ──────────────────────────────────────────────────────────

    #[test]
    fn create_only_schema_and_record_plan_is_compliant() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Compliant);
    }

    #[test]
    fn empty_operations_list_is_compliant() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Compliant);
    }

    #[test]
    fn delete_base_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteBase,
            "delete-base",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn delete_table_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteTable,
            "delete-table",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn delete_field_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteField,
            "delete-field",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn delete_record_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteRecord,
            "delete-record",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn update_existing_record_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::UpdateExistingRecord,
            "update-record",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn overwrite_field_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::OverwriteField,
            "overwrite-field",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn overwrite_table_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::OverwriteTable,
            "overwrite-table",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn attachment_upload_is_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::AttachmentUpload,
            "upload-attachment",
        )]));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    #[test]
    fn mixed_create_and_delete_is_blocked() {
        let mut ops = create_only_ops();
        ops.push(make_op(
            DestructiveOperationKind::DeleteTable,
            "delete-table",
        ));
        let result = verify_destructive_operation_policy(&request_with_ops(ops));
        assert_eq!(result.status, DestructiveOperationPolicyStatus::Blocked);
    }

    // ── Check IDs present ────────────────────────────────────────────────────

    #[test]
    fn all_five_check_ids_present_for_compliant() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert!(ids.contains(&"DOP-01"));
        assert!(ids.contains(&"DOP-02"));
        assert!(ids.contains(&"DOP-03"));
        assert!(ids.contains(&"DOP-04"));
        assert!(ids.contains(&"DOP-05"));
    }

    #[test]
    fn all_five_check_ids_present_for_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteTable,
            "delete-table",
        )]));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert!(ids.contains(&"DOP-01"));
        assert!(ids.contains(&"DOP-02"));
        assert!(ids.contains(&"DOP-03"));
        assert!(ids.contains(&"DOP-04"));
        assert!(ids.contains(&"DOP-05"));
    }

    #[test]
    fn dop_01_always_passes() {
        for ops in [vec![], create_only_ops()] {
            let result = verify_destructive_operation_policy(&request_with_ops(ops));
            let dop01 = result
                .checks
                .iter()
                .find(|c| c.check_id == "DOP-01")
                .unwrap();
            assert_eq!(dop01.status, DestructiveOperationCheckStatus::Passed);
        }
    }

    #[test]
    fn dop_02_fails_on_delete() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteRecord,
            "delete-r",
        )]));
        let dop02 = result
            .checks
            .iter()
            .find(|c| c.check_id == "DOP-02")
            .unwrap();
        assert_eq!(dop02.status, DestructiveOperationCheckStatus::Failed);
    }

    #[test]
    fn dop_03_fails_on_overwrite() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::OverwriteField,
            "overwrite-f",
        )]));
        let dop03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "DOP-03")
            .unwrap();
        assert_eq!(dop03.status, DestructiveOperationCheckStatus::Failed);
    }

    #[test]
    fn dop_04_fails_on_attachment_upload() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::AttachmentUpload,
            "upload",
        )]));
        let dop04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "DOP-04")
            .unwrap();
        assert_eq!(dop04.status, DestructiveOperationCheckStatus::Failed);
    }

    #[test]
    fn dop_05_passes_on_create_only() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        let dop05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "DOP-05")
            .unwrap();
        assert_eq!(dop05.status, DestructiveOperationCheckStatus::Passed);
    }

    // ── blocked_operations list ───────────────────────────────────────────────

    #[test]
    fn blocked_operations_empty_for_compliant() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        assert!(result.blocked_operations.is_empty());
    }

    #[test]
    fn blocked_operations_contains_label_for_blocked() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteTable,
            "delete-my-table",
        )]));
        assert!(result
            .blocked_operations
            .contains(&"delete-my-table".to_string()));
    }

    #[test]
    fn multiple_blocked_ops_all_appear_in_list() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![
            make_op(DestructiveOperationKind::DeleteTable, "drop-t"),
            make_op(DestructiveOperationKind::AttachmentUpload, "upload-a"),
        ]));
        assert!(result.blocked_operations.contains(&"drop-t".to_string()));
        assert!(result.blocked_operations.contains(&"upload-a".to_string()));
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_is_always_true() {
        for ops in [
            vec![],
            create_only_ops(),
            vec![make_op(DestructiveOperationKind::DeleteBase, "d")],
        ] {
            let result = verify_destructive_operation_policy(&request_with_ops(ops));
            assert!(result.no_changes_made);
        }
    }

    #[test]
    fn writes_enabled_is_always_false() {
        for ops in [
            vec![],
            create_only_ops(),
            vec![make_op(DestructiveOperationKind::DeleteBase, "d")],
        ] {
            let result = verify_destructive_operation_policy(&request_with_ops(ops));
            assert!(!result.writes_enabled);
        }
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        for ops in [
            vec![],
            create_only_ops(),
            vec![make_op(DestructiveOperationKind::DeleteBase, "d")],
        ] {
            let result = verify_destructive_operation_policy(&request_with_ops(ops));
            assert!(!result.network_writes_attempted);
        }
    }

    #[test]
    fn compliant_result_serialization_has_no_token() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("token"));
        assert!(!json.contains("pat"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn result_serialization_has_no_full_path() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\\\"));
    }

    #[test]
    fn message_does_not_contain_token() {
        for ops in [
            create_only_ops(),
            vec![make_op(DestructiveOperationKind::DeleteBase, "d")],
        ] {
            let result = verify_destructive_operation_policy(&request_with_ops(ops));
            assert!(!result.message.contains("pat"));
            assert!(!result.message.contains("token"));
        }
    }

    #[test]
    fn no_write_calls_are_made() {
        // Structural: verify_destructive_operation_policy takes no token, no HTTP client.
        // Confirmed at compile time — this test documents the intent.
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        assert!(!result.network_writes_attempted);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
    }

    #[test]
    fn message_is_non_empty_for_all_statuses() {
        for ops in [
            vec![],
            create_only_ops(),
            vec![make_op(DestructiveOperationKind::DeleteBase, "d")],
        ] {
            let result = verify_destructive_operation_policy(&request_with_ops(ops));
            assert!(!result.message.is_empty());
        }
    }

    #[test]
    fn compliant_message_says_writes_remain_disabled() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn display_name_appears_in_message() {
        let result = verify_destructive_operation_policy(&request_with_ops(create_only_ops()));
        assert!(result.message.contains("My Base"));
    }

    #[test]
    fn blocked_message_names_blocked_ops() {
        let result = verify_destructive_operation_policy(&request_with_ops(vec![make_op(
            DestructiveOperationKind::DeleteTable,
            "drop-projects",
        )]));
        assert!(result.message.contains("drop-projects"));
    }
}
