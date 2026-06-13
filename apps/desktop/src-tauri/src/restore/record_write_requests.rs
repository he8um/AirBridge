use serde::{Deserialize, Serialize};

use crate::restore::record_import_plan::{
    RestoreRecordFieldImportPolicy, RestoreRecordImportPlan, RestoreRecordImportPlanStatus,
};

/// What kind of record write operation is represented.
///
/// Note: only planning-time kinds are listed. No live Airtable operation is
/// executed — these describe what *would* happen when the write engine is enabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteOperationKind {
    /// Create a batch of records in a table (first pass).
    CreateRecordBatch,
    /// Update linked record fields in a table (second pass, after ID mapping).
    UpdateLinkedRecordBatch,
    /// Record the checkpoint state after a batch completes.
    Checkpoint,
    /// Field is attachment-typed — metadata preserved, file bytes not uploaded.
    PreserveMetadataOnlyAttachment,
    /// Field is computed or read-only — skipped during record import.
    SkipComputedField,
    /// Field requires manual action after restore.
    ManualAction,
}

/// Planning-time status for one record write operation.
///
/// Note: `success` / `succeeded` / `completed` / `executed` / `applied` are
/// intentionally absent. No operation is executed in this version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteOperationStatus {
    Planned,
    Blocked,
    Disabled,
}

/// Why a record write request plan is blocked or disabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordWriteBlockedReason {
    DisabledByProductPolicy,
    RecordImportPlanNotReady,
    NoTablesInPlan,
}

/// A single planned record write operation.
///
/// Safety properties:
/// - No token field.
/// - Table ID/name are derived from the backup package only.
/// - status is never `succeeded`.
/// - no_changes_made is always true.
/// - Raw record payloads are never included — only counts and summaries.
/// - Old-to-new record ID mapping is unavailable at planning time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteOperation {
    /// Ordering index within the full plan (0-based, create batches before update batches).
    pub index: usize,
    pub kind: RecordWriteOperationKind,
    pub status: RecordWriteOperationStatus,
    /// Table ID derived from the backup package.
    pub table_id: String,
    pub table_name: String,
    /// Batch index within this table's sequence. None for non-batch operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_index: Option<usize>,
    /// Number of records planned for this batch. None if record count is unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_record_count: Option<usize>,
    /// For linked-record update batches: how many linked record fields are involved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_field_count: Option<usize>,
    /// For attachment operations: policy summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_policy: Option<String>,
    /// For skipped fields: field name and type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_field_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_field_type: Option<String>,
    /// Note about this operation.
    pub note: String,
    /// Always true — the operation has not been executed.
    pub no_changes_made: bool,
}

/// A full record write request plan built from a RestoreRecordImportPlan.
///
/// Safety properties:
/// - No token field.
/// - No full paths — filename only.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - status is never `succeeded`.
/// - No raw record payloads — only counts and summaries.
/// - Old-to-new ID mapping is deferred to execution time (not resolved here).
/// - Operations are ordered: create-record batches first, linked-record update
///   batches second, then checkpoints, attachment metadata, and skipped fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteRequestPlan {
    /// Filename only — never the full package path.
    pub filename: String,
    pub status: RecordWriteOperationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<RecordWriteBlockedReason>,
    /// Ordered list of all planned operations.
    pub operations: Vec<RecordWriteOperation>,
    /// Number of create-record batch operations.
    pub create_batch_op_count: usize,
    /// Number of linked-record update batch operations.
    pub linked_update_op_count: usize,
    /// Number of checkpoint operations.
    pub checkpoint_op_count: usize,
    /// Number of attachment metadata/manual operations.
    pub attachment_op_count: usize,
    /// Number of skipped field operations.
    pub skipped_field_op_count: usize,
    /// Total operation count across all kinds.
    pub total_op_count: usize,
    /// Total planned first-pass record create batches across all tables.
    pub total_first_pass_batches: usize,
    /// Total planned second-pass linked-record update batches across all tables.
    pub total_second_pass_batches: usize,
    pub warnings: Vec<String>,
    /// Always true — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
}

/// Builds a record write request plan from an existing record import plan.
///
/// Ordering contract (enforced here):
///   1. CreateRecordBatch operations — first-pass create batches, table by table.
///   2. UpdateLinkedRecordBatch operations — second-pass linked updates, table by table.
///   3. Checkpoint operations — one per table after its create batches.
///   4. PreserveMetadataOnlyAttachment operations — per-table attachment summaries.
///   5. SkipComputedField operations — per-table skipped field summaries.
///
/// All operations are `Disabled` because the write gate blocks execution.
/// No Airtable API calls are made. No token is required.
/// Raw record payloads are never included — only counts and summaries.
pub fn build_record_write_request_plan(
    import_plan: &RestoreRecordImportPlan,
) -> RecordWriteRequestPlan {
    let filename = import_plan.filename.clone();

    // If the source plan is blocked, produce a blocked result immediately.
    if import_plan.status == RestoreRecordImportPlanStatus::Blocked {
        return RecordWriteRequestPlan {
            filename,
            status: RecordWriteOperationStatus::Blocked,
            blocked_reason: Some(RecordWriteBlockedReason::RecordImportPlanNotReady),
            operations: vec![],
            create_batch_op_count: 0,
            linked_update_op_count: 0,
            checkpoint_op_count: 0,
            attachment_op_count: 0,
            skipped_field_op_count: 0,
            total_op_count: 0,
            total_first_pass_batches: 0,
            total_second_pass_batches: 0,
            warnings: vec![
                "Record import plan is blocked — no operations can be planned.".to_string(),
            ],
            no_changes_made: true,
            network_writes_attempted: false,
        };
    }

    if import_plan.table_plans.is_empty() {
        return RecordWriteRequestPlan {
            filename,
            status: RecordWriteOperationStatus::Blocked,
            blocked_reason: Some(RecordWriteBlockedReason::NoTablesInPlan),
            operations: vec![],
            create_batch_op_count: 0,
            linked_update_op_count: 0,
            checkpoint_op_count: 0,
            attachment_op_count: 0,
            skipped_field_op_count: 0,
            total_op_count: 0,
            total_first_pass_batches: 0,
            total_second_pass_batches: 0,
            warnings: vec!["No tables in record import plan — nothing to plan.".to_string()],
            no_changes_made: true,
            network_writes_attempted: false,
        };
    }

    let mut operations: Vec<RecordWriteOperation> = Vec::new();
    let mut idx: usize = 0;

    // Phase 1: CreateRecordBatch — one operation per first-pass batch, table by table.
    let mut total_first_pass_batches: usize = 0;
    for table_plan in &import_plan.table_plans {
        let batch_count = table_plan
            .create_batch_count
            .unwrap_or(table_plan.first_pass_batches.len());
        total_first_pass_batches += batch_count;

        if batch_count == 0 && table_plan.record_count.is_none() {
            // Unknown record count — plan a single representative operation
            operations.push(RecordWriteOperation {
                index: idx,
                kind: RecordWriteOperationKind::CreateRecordBatch,
                status: RecordWriteOperationStatus::Disabled,
                table_id: table_plan.table_id.clone(),
                table_name: table_plan.table_name.clone(),
                batch_index: None,
                planned_record_count: None,
                linked_field_count: None,
                attachment_policy: None,
                skipped_field_name: None,
                skipped_field_type: None,
                note: format!(
                    "Record count unknown for '{}' — batch count cannot be determined at planning time.",
                    table_plan.table_name
                ),
                no_changes_made: true,
            });
            idx += 1;
        } else {
            for batch in &table_plan.first_pass_batches {
                operations.push(RecordWriteOperation {
                    index: idx,
                    kind: RecordWriteOperationKind::CreateRecordBatch,
                    status: RecordWriteOperationStatus::Disabled,
                    table_id: table_plan.table_id.clone(),
                    table_name: table_plan.table_name.clone(),
                    batch_index: Some(batch.batch_index),
                    planned_record_count: Some(batch.record_count),
                    linked_field_count: None,
                    attachment_policy: None,
                    skipped_field_name: None,
                    skipped_field_type: None,
                    note: format!(
                        "Create batch {} of {} for table '{}': {} record(s) (batch size {}).",
                        batch.batch_index + 1,
                        batch_count,
                        table_plan.table_name,
                        batch.record_count,
                        table_plan.batch_size,
                    ),
                    no_changes_made: true,
                });
                idx += 1;
            }
        }
    }

    // Phase 2: UpdateLinkedRecordBatch — one operation per second-pass batch, table by table.
    let mut total_second_pass_batches: usize = 0;
    for table_plan in &import_plan.table_plans {
        let linked_field_count = table_plan.linked_record_updates.len();
        if linked_field_count == 0 {
            continue;
        }

        let batch_count = table_plan
            .update_batch_count
            .unwrap_or(table_plan.second_pass_batches.len());
        total_second_pass_batches += batch_count;

        if batch_count == 0 && table_plan.record_count.is_none() {
            operations.push(RecordWriteOperation {
                index: idx,
                kind: RecordWriteOperationKind::UpdateLinkedRecordBatch,
                status: RecordWriteOperationStatus::Disabled,
                table_id: table_plan.table_id.clone(),
                table_name: table_plan.table_name.clone(),
                batch_index: None,
                planned_record_count: None,
                linked_field_count: Some(linked_field_count),
                attachment_policy: None,
                skipped_field_name: None,
                skipped_field_type: None,
                note: format!(
                    "Linked record update count unknown for '{}' — {} linked field(s) to update after ID mapping.",
                    table_plan.table_name, linked_field_count
                ),
                no_changes_made: true,
            });
            idx += 1;
        } else {
            for batch in &table_plan.second_pass_batches {
                operations.push(RecordWriteOperation {
                    index: idx,
                    kind: RecordWriteOperationKind::UpdateLinkedRecordBatch,
                    status: RecordWriteOperationStatus::Disabled,
                    table_id: table_plan.table_id.clone(),
                    table_name: table_plan.table_name.clone(),
                    batch_index: Some(batch.batch_index),
                    planned_record_count: Some(batch.record_count),
                    linked_field_count: Some(linked_field_count),
                    attachment_policy: None,
                    skipped_field_name: None,
                    skipped_field_type: None,
                    note: format!(
                        "Linked update batch {} for table '{}': {} record(s), {} linked field(s). ID mapping unavailable until execution.",
                        batch.batch_index + 1,
                        table_plan.table_name,
                        batch.record_count,
                        linked_field_count,
                    ),
                    no_changes_made: true,
                });
                idx += 1;
            }
        }
    }

    // Phase 3: Checkpoint — one per table.
    for table_plan in &import_plan.table_plans {
        let cp = &table_plan.checkpoint_plan;
        operations.push(RecordWriteOperation {
            index: idx,
            kind: RecordWriteOperationKind::Checkpoint,
            status: RecordWriteOperationStatus::Disabled,
            table_id: table_plan.table_id.clone(),
            table_name: table_plan.table_name.clone(),
            batch_index: Some(cp.checkpoint_batch_index),
            planned_record_count: None,
            linked_field_count: None,
            attachment_policy: None,
            skipped_field_name: None,
            skipped_field_type: None,
            note: format!(
                "Checkpoint for table '{}' at batch index {}.",
                table_plan.table_name, cp.checkpoint_batch_index,
            ),
            no_changes_made: true,
        });
        idx += 1;
    }

    // Phase 4: PreserveMetadataOnlyAttachment — one per attachment field per table.
    let mut attachment_op_count: usize = 0;
    for table_plan in &import_plan.table_plans {
        for att in &table_plan.attachment_policies {
            operations.push(RecordWriteOperation {
                index: idx,
                kind: RecordWriteOperationKind::PreserveMetadataOnlyAttachment,
                status: RecordWriteOperationStatus::Disabled,
                table_id: table_plan.table_id.clone(),
                table_name: table_plan.table_name.clone(),
                batch_index: None,
                planned_record_count: None,
                linked_field_count: None,
                attachment_policy: Some(format!("{:?}", att.policy)),
                skipped_field_name: None,
                skipped_field_type: None,
                note: format!(
                    "Attachment field '{}' in '{}': metadata preserved, file bytes not uploaded. Manual re-attachment required after restore.",
                    att.field_name, table_plan.table_name,
                ),
                no_changes_made: true,
            });
            idx += 1;
            attachment_op_count += 1;
        }
    }

    // Phase 5: SkipComputedField — one per skipped field per table.
    let mut skipped_field_op_count: usize = 0;
    for table_plan in &import_plan.table_plans {
        for fp in &table_plan.field_policies {
            if fp.policy == RestoreRecordFieldImportPolicy::Skip {
                operations.push(RecordWriteOperation {
                    index: idx,
                    kind: RecordWriteOperationKind::SkipComputedField,
                    status: RecordWriteOperationStatus::Disabled,
                    table_id: table_plan.table_id.clone(),
                    table_name: table_plan.table_name.clone(),
                    batch_index: None,
                    planned_record_count: None,
                    linked_field_count: None,
                    attachment_policy: None,
                    skipped_field_name: Some(fp.field_name.clone()),
                    skipped_field_type: Some(fp.field_type.clone()),
                    note: format!(
                        "Field '{}' ({}) in '{}' is computed or read-only — skipped during record import.",
                        fp.field_name, fp.field_type, table_plan.table_name,
                    ),
                    no_changes_made: true,
                });
                idx += 1;
                skipped_field_op_count += 1;
            }
        }
    }

    let create_batch_op_count = operations
        .iter()
        .filter(|o| o.kind == RecordWriteOperationKind::CreateRecordBatch)
        .count();
    let linked_update_op_count = operations
        .iter()
        .filter(|o| o.kind == RecordWriteOperationKind::UpdateLinkedRecordBatch)
        .count();
    let checkpoint_op_count = operations
        .iter()
        .filter(|o| o.kind == RecordWriteOperationKind::Checkpoint)
        .count();
    let total_op_count = operations.len();

    // Collect warnings from the source plan
    let warnings: Vec<String> = import_plan
        .warnings
        .iter()
        .map(|w| format!("[{}] {}", w.code, w.message))
        .collect();

    RecordWriteRequestPlan {
        filename,
        status: RecordWriteOperationStatus::Disabled,
        blocked_reason: None,
        operations,
        create_batch_op_count,
        linked_update_op_count,
        checkpoint_op_count,
        attachment_op_count,
        skipped_field_op_count,
        total_op_count,
        total_first_pass_batches,
        total_second_pass_batches,
        warnings,
        no_changes_made: true,
        network_writes_attempted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
    };
    use crate::restore::record_import_planner::create_record_import_plan;

    fn make_import_request(tables: Vec<RecordImportTableInput>) -> RestoreRecordImportPlanRequest {
        RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables,
        }
    }

    fn simple_table(id: &str, name: &str, count: Option<usize>) -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: id.to_string(),
            table_name: name.to_string(),
            record_count: count,
            fields: vec![
                RecordImportFieldInput {
                    field_id: "fld01".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld02".to_string(),
                    field_name: "Status".to_string(),
                    field_type: "singleSelect".to_string(),
                    linked_table_id: None,
                },
            ],
        }
    }

    fn complex_table() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblA".to_string(),
            table_name: "Projects".to_string(),
            record_count: Some(25),
            fields: vec![
                RecordImportFieldInput {
                    field_id: "fld01".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld02".to_string(),
                    field_name: "Tasks".to_string(),
                    field_type: "multipleRecordLinks".to_string(),
                    linked_table_id: Some("tblB".to_string()),
                },
                RecordImportFieldInput {
                    field_id: "fld03".to_string(),
                    field_name: "Files".to_string(),
                    field_type: "multipleAttachments".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld04".to_string(),
                    field_name: "Calc".to_string(),
                    field_type: "formula".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld05".to_string(),
                    field_name: "ID".to_string(),
                    field_type: "autoNumber".to_string(),
                    linked_table_id: None,
                },
            ],
        }
    }

    fn ready_import_plan() -> RestoreRecordImportPlan {
        let req = make_import_request(vec![simple_table("tbl01", "Projects", Some(20))]);
        create_record_import_plan(&req)
    }

    fn complex_import_plan() -> RestoreRecordImportPlan {
        let req = make_import_request(vec![complex_table()]);
        create_record_import_plan(&req)
    }

    fn blocked_import_plan() -> RestoreRecordImportPlan {
        let req = RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "blocked".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            tables: vec![simple_table("tbl01", "T", Some(5))],
        };
        create_record_import_plan(&req)
    }

    // ── Status tests ───────────────────────────────────────────────────────

    #[test]
    fn plan_status_is_disabled_for_ready_input() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.status, RecordWriteOperationStatus::Disabled);
    }

    #[test]
    fn plan_status_is_blocked_for_blocked_input() {
        let import_plan = blocked_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.status, RecordWriteOperationStatus::Blocked);
        assert_eq!(
            plan.blocked_reason,
            Some(RecordWriteBlockedReason::RecordImportPlanNotReady)
        );
    }

    #[test]
    fn plan_status_is_blocked_for_empty_tables() {
        let import_plan = RestoreRecordImportPlan {
            filename: "backup.airbridge".to_string(),
            status: RestoreRecordImportPlanStatus::Ready,
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            table_plans: vec![],
            linked_record_update_plans: vec![],
            retry_policy: crate::restore::record_import_plan::RestoreRetryPolicy {
                max_retries_on_rate_limit: 5,
                initial_backoff_ms: 1000,
                backoff_multiplier: 2.0,
                note: "".to_string(),
            },
            warnings: vec![],
            errors: vec![],
            no_changes_made: true,
        };
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.status, RecordWriteOperationStatus::Blocked);
        assert_eq!(
            plan.blocked_reason,
            Some(RecordWriteBlockedReason::NoTablesInPlan)
        );
    }

    // ── Safety invariant tests ─────────────────────────────────────────────

    #[test]
    fn no_changes_made_is_always_true() {
        for plan in &[
            build_record_write_request_plan(&ready_import_plan()),
            build_record_write_request_plan(&blocked_import_plan()),
            build_record_write_request_plan(&complex_import_plan()),
        ] {
            assert!(plan.no_changes_made, "no_changes_made must be true");
        }
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        for plan in &[
            build_record_write_request_plan(&ready_import_plan()),
            build_record_write_request_plan(&blocked_import_plan()),
        ] {
            assert!(
                !plan.network_writes_attempted,
                "network_writes_attempted must be false"
            );
        }
    }

    #[test]
    fn plan_has_no_token_field() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn plan_has_no_succeeded_status() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn plan_has_no_absolute_path() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn plan_has_no_raw_record_payloads() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("\"records\":"));
        assert!(!json.contains("\"payload\":"));
        assert!(!json.contains("\"fields\":"));
    }

    #[test]
    fn plan_has_no_new_record_ids() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("newRecordId"));
        assert!(!json.contains("rec_new_"));
    }

    #[test]
    fn filename_is_basename_only() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.filename, "backup.airbridge");
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
    }

    // ── Ordering tests ─────────────────────────────────────────────────────

    #[test]
    fn create_batches_come_before_linked_update_batches() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);

        let last_create = plan
            .operations
            .iter()
            .rposition(|o| o.kind == RecordWriteOperationKind::CreateRecordBatch);
        let first_update = plan
            .operations
            .iter()
            .position(|o| o.kind == RecordWriteOperationKind::UpdateLinkedRecordBatch);

        if let (Some(last_c), Some(first_u)) = (last_create, first_update) {
            assert!(
                last_c < first_u,
                "all create batches must come before any linked update batches"
            );
        }
    }

    #[test]
    fn linked_update_batches_come_before_checkpoints() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);

        let last_update = plan
            .operations
            .iter()
            .rposition(|o| o.kind == RecordWriteOperationKind::UpdateLinkedRecordBatch);
        let first_checkpoint = plan
            .operations
            .iter()
            .position(|o| o.kind == RecordWriteOperationKind::Checkpoint);

        if let (Some(last_u), Some(first_cp)) = (last_update, first_checkpoint) {
            assert!(
                last_u < first_cp,
                "linked update batches must come before checkpoints"
            );
        }
    }

    #[test]
    fn checkpoints_come_before_attachment_and_skipped_ops() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);

        let last_checkpoint = plan
            .operations
            .iter()
            .rposition(|o| o.kind == RecordWriteOperationKind::Checkpoint);
        let first_attachment = plan
            .operations
            .iter()
            .position(|o| o.kind == RecordWriteOperationKind::PreserveMetadataOnlyAttachment);
        let first_skip = plan
            .operations
            .iter()
            .position(|o| o.kind == RecordWriteOperationKind::SkipComputedField);

        if let (Some(last_cp), Some(first_att)) = (last_checkpoint, first_attachment) {
            assert!(
                last_cp < first_att,
                "checkpoints must precede attachment ops"
            );
        }
        if let (Some(last_cp), Some(first_sk)) = (last_checkpoint, first_skip) {
            assert!(last_cp < first_sk, "checkpoints must precede skip ops");
        }
    }

    #[test]
    fn operation_indices_are_contiguous_and_increasing() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        for (i, op) in plan.operations.iter().enumerate() {
            assert_eq!(op.index, i, "operation index must be contiguous");
        }
    }

    // ── Batch count / size tests ───────────────────────────────────────────

    #[test]
    fn batch_count_matches_import_plan() {
        let import_plan = ready_import_plan();
        let expected_batches = import_plan.table_plans[0].create_batch_count.unwrap_or(0);
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.create_batch_op_count, expected_batches);
    }

    #[test]
    fn create_batch_op_count_is_non_zero_for_known_record_count() {
        let import_plan = ready_import_plan(); // 20 records = 2 batches
        let plan = build_record_write_request_plan(&import_plan);
        assert!(plan.create_batch_op_count > 0);
    }

    #[test]
    fn total_first_pass_batches_matches_sum() {
        let import_plan = ready_import_plan();
        let expected: usize = import_plan
            .table_plans
            .iter()
            .map(|t| t.create_batch_count.unwrap_or(0))
            .sum();
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.total_first_pass_batches, expected);
    }

    #[test]
    fn unknown_record_count_produces_single_representative_create_op() {
        let req = make_import_request(vec![simple_table("tbl01", "Unknown", None)]);
        let import_plan = create_record_import_plan(&req);
        let plan = build_record_write_request_plan(&import_plan);
        // Should have exactly one representative create op
        let create_ops: Vec<_> = plan
            .operations
            .iter()
            .filter(|o| o.kind == RecordWriteOperationKind::CreateRecordBatch)
            .collect();
        assert_eq!(
            create_ops.len(),
            1,
            "unknown count should produce one representative op"
        );
        assert!(create_ops[0].planned_record_count.is_none());
        assert!(create_ops[0].batch_index.is_none());
    }

    // ── Linked record tests ────────────────────────────────────────────────

    #[test]
    fn linked_update_ops_are_planned_for_linked_fields() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert!(plan.linked_update_op_count > 0);
    }

    #[test]
    fn no_linked_update_ops_for_table_without_linked_fields() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.linked_update_op_count, 0);
    }

    #[test]
    fn linked_update_ops_note_id_mapping_unavailable() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let linked_ops: Vec<_> = plan
            .operations
            .iter()
            .filter(|o| o.kind == RecordWriteOperationKind::UpdateLinkedRecordBatch)
            .collect();
        for op in &linked_ops {
            assert!(
                op.note.contains("ID mapping unavailable"),
                "linked update op must note that ID mapping is unavailable until execution"
            );
        }
    }

    #[test]
    fn total_second_pass_batches_is_zero_for_no_linked_fields() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.total_second_pass_batches, 0);
    }

    // ── Attachment tests ───────────────────────────────────────────────────

    #[test]
    fn attachment_ops_are_planned_for_attachment_fields() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert!(plan.attachment_op_count > 0);
    }

    #[test]
    fn attachment_ops_note_no_file_upload() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let att_ops: Vec<_> = plan
            .operations
            .iter()
            .filter(|o| o.kind == RecordWriteOperationKind::PreserveMetadataOnlyAttachment)
            .collect();
        for op in &att_ops {
            assert!(
                op.note.contains("file bytes not uploaded")
                    || op.note.contains("Manual re-attachment"),
                "attachment op must note that file bytes are not uploaded"
            );
        }
    }

    // ── Skipped field tests ────────────────────────────────────────────────

    #[test]
    fn skipped_field_ops_are_planned_for_computed_fields() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert!(plan.skipped_field_op_count > 0);
    }

    #[test]
    fn skipped_field_ops_have_field_name_and_type() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let skip_ops: Vec<_> = plan
            .operations
            .iter()
            .filter(|o| o.kind == RecordWriteOperationKind::SkipComputedField)
            .collect();
        for op in &skip_ops {
            assert!(op.skipped_field_name.is_some());
            assert!(op.skipped_field_type.is_some());
        }
    }

    // ── Checkpoint tests ───────────────────────────────────────────────────

    #[test]
    fn checkpoint_ops_are_planned_one_per_table() {
        let req = make_import_request(vec![
            simple_table("tbl01", "A", Some(10)),
            simple_table("tbl02", "B", Some(20)),
        ]);
        let import_plan = create_record_import_plan(&req);
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.checkpoint_op_count, 2);
    }

    // ── Count consistency tests ────────────────────────────────────────────

    #[test]
    fn total_op_count_equals_operations_len() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        assert_eq!(plan.total_op_count, plan.operations.len());
    }

    #[test]
    fn op_count_components_sum_to_total() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let sum = plan.create_batch_op_count
            + plan.linked_update_op_count
            + plan.checkpoint_op_count
            + plan.attachment_op_count
            + plan.skipped_field_op_count;
        assert_eq!(
            plan.total_op_count, sum,
            "total_op_count must equal sum of component counts"
        );
    }

    // ── Serialization tests ────────────────────────────────────────────────

    #[test]
    fn plan_serializes_no_changes_made_key() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
        assert!(json.contains("networkWritesAttempted"));
    }

    #[test]
    fn operations_each_serialize_no_changes_made() {
        let import_plan = ready_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        let json = serde_json::to_string(&plan).expect("serialize");
        // Every operation has no_changes_made — at least one must appear
        assert!(json.contains("noChangesMade"));
    }

    #[test]
    fn all_operation_statuses_are_disabled_or_blocked() {
        let import_plan = complex_import_plan();
        let plan = build_record_write_request_plan(&import_plan);
        for op in &plan.operations {
            assert!(
                op.status == RecordWriteOperationStatus::Disabled
                    || op.status == RecordWriteOperationStatus::Blocked,
                "operation status must be disabled or blocked, not succeeded"
            );
        }
    }
}
