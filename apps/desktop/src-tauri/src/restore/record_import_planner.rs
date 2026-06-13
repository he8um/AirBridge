use crate::restore::attachment_restore_policy::build_attachment_policies;
use crate::restore::linked_record_updates::build_linked_update_plans;
use crate::restore::record_import_batches::{
    batch_count_for, build_checkpoint_plan, build_first_pass_batches, build_second_pass_batches,
    AIRTABLE_WRITE_BATCH_SIZE,
};
use crate::restore::record_import_plan::{
    RecordImportTableInput, RestoreRecordFieldImportPolicy, RestoreRecordFieldPolicy,
    RestoreRecordImportError, RestoreRecordImportPlan, RestoreRecordImportPlanRequest,
    RestoreRecordImportPlanStatus, RestoreRetryPolicy, RestoreTableImportPlan,
};
use crate::restore::record_import_warnings::warnings_for_table_import;
use crate::restore::record_mapping::build_mapping_plan;

/// Creates a record import plan from the request.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files written or extracted.
/// - Filename in the result is never a full path.
/// - no_changes_made is always true.
pub fn create_record_import_plan(
    request: &RestoreRecordImportPlanRequest,
) -> RestoreRecordImportPlan {
    let filename = request.package_filename.clone();

    // Gate: dry-run must be ready.
    let dry_run_ok =
        request.dry_run_status == "ready" || request.dry_run_status == "readyWithWarnings";
    if !dry_run_ok {
        return blocked_plan(
            filename,
            request,
            "DRY_RUN_BLOCKED",
            "The dry-run plan is blocked. Generate a successful restore plan preview before \
            creating a record import plan.",
        );
    }

    // Gate: schema plan must be ready.
    let schema_ok =
        request.schema_plan_status == "ready" || request.schema_plan_status == "readyWithWarnings";
    if !schema_ok {
        return blocked_plan(
            filename,
            request,
            "SCHEMA_PLAN_BLOCKED",
            "The schema creation plan is blocked. Generate a successful schema plan before \
            creating a record import plan.",
        );
    }

    if request.tables.is_empty() {
        return blocked_plan(
            filename,
            request,
            "NO_TABLES",
            "No tables are available for record import planning.",
        );
    }

    let mut table_plans: Vec<RestoreTableImportPlan> = Vec::new();
    let mut all_linked_updates = Vec::new();
    let mut all_warnings = Vec::new();

    for (order, table) in request.tables.iter().enumerate() {
        let table_plan = build_table_import_plan(table, order);

        // Collect per-table warnings.
        let table_warnings = warnings_for_table_import(&table_plan);
        all_warnings.extend(table_warnings);

        // Collect global linked record update plans.
        all_linked_updates.extend(table_plan.linked_record_updates.clone());

        table_plans.push(table_plan);
    }

    let status = if all_warnings.is_empty() {
        RestoreRecordImportPlanStatus::Ready
    } else {
        RestoreRecordImportPlanStatus::ReadyWithWarnings
    };

    RestoreRecordImportPlan {
        filename,
        status,
        target_mode: request.target_mode.clone(),
        target_base_name: request.target_base_name.clone(),
        table_plans,
        linked_record_update_plans: all_linked_updates,
        retry_policy: default_retry_policy(),
        warnings: all_warnings,
        errors: vec![],
        no_changes_made: true,
    }
}

/// Builds the import plan for a single table.
fn build_table_import_plan(
    table: &RecordImportTableInput,
    import_order: usize,
) -> RestoreTableImportPlan {
    let has_linked = table
        .fields
        .iter()
        .any(|f| f.field_type == "multipleRecordLinks");

    let first_pass = build_first_pass_batches(
        table.record_count,
        AIRTABLE_WRITE_BATCH_SIZE,
        &table.table_name,
    );
    let second_pass = build_second_pass_batches(
        table.record_count,
        AIRTABLE_WRITE_BATCH_SIZE,
        has_linked,
        &table.table_name,
    );

    let create_batch_count = table
        .record_count
        .map(|c| batch_count_for(c, AIRTABLE_WRITE_BATCH_SIZE));
    let update_batch_count = if has_linked {
        table
            .record_count
            .map(|c| batch_count_for(c, AIRTABLE_WRITE_BATCH_SIZE))
    } else {
        None
    };

    let field_policies = build_field_policies(table);
    let attachment_policies = build_attachment_policies(table);
    let mapping_plan = build_mapping_plan(table);
    let checkpoint_plan =
        build_checkpoint_plan(&table.table_id, &table.table_name, create_batch_count);
    let linked_record_updates = build_linked_update_plans(table);

    RestoreTableImportPlan {
        table_id: table.table_id.clone(),
        table_name: table.table_name.clone(),
        import_order,
        record_count: table.record_count,
        record_count_known: table.record_count.is_some(),
        batch_size: AIRTABLE_WRITE_BATCH_SIZE,
        create_batch_count,
        update_batch_count,
        first_pass_batches: first_pass,
        second_pass_batches: second_pass,
        field_policies,
        attachment_policies,
        mapping_plan,
        checkpoint_plan,
        linked_record_updates,
    }
}

/// Classifies each field into a record import policy.
fn build_field_policies(table: &RecordImportTableInput) -> Vec<RestoreRecordFieldPolicy> {
    table
        .fields
        .iter()
        .map(|f| {
            let (policy, note) = classify_field_import_policy(&f.field_type);
            RestoreRecordFieldPolicy {
                field_id: f.field_id.clone(),
                field_name: f.field_name.clone(),
                field_type: f.field_type.clone(),
                policy,
                note,
            }
        })
        .collect()
}

/// Classifies a field type into an import policy.
fn classify_field_import_policy(field_type: &str) -> (RestoreRecordFieldImportPolicy, String) {
    match field_type {
        // Directly writable primitive fields.
        "singleLineText" | "multilineText" | "email" | "url" | "phoneNumber" | "number"
        | "currency" | "percent" | "rating" | "checkbox" | "date" | "dateTime" | "duration"
        | "barcode" | "singleSelect" | "multipleSelects" => (
            RestoreRecordFieldImportPolicy::Include,
            format!("'{field_type}' is included in the record create payload."),
        ),

        // Linked record fields — deferred to second pass.
        "multipleRecordLinks" => (
            RestoreRecordFieldImportPolicy::DeferToLinkedRecordPass,
            "Linked record field — excluded from first-pass create, applied in second pass \
            after ID remapping."
                .to_string(),
        ),

        // Attachment fields — metadata included but no file bytes.
        "multipleAttachments" => (
            RestoreRecordFieldImportPolicy::MetadataOnly,
            "Attachment metadata is included; file bytes are not re-uploaded.".to_string(),
        ),

        // Computed / read-only / system fields — skip.
        "formula"
        | "rollup"
        | "lookup"
        | "autoNumber"
        | "createdTime"
        | "lastModifiedTime"
        | "count"
        | "createdBy"
        | "lastModifiedBy"
        | "singleCollaborator"
        | "multipleCollaborators" => (
            RestoreRecordFieldImportPolicy::Skip,
            format!("'{field_type}' is computed or read-only — skipped during record import."),
        ),

        // Unknown — conservative skip.
        _ => (
            RestoreRecordFieldImportPolicy::Skip,
            format!("'{field_type}' is an unknown field type — skipped during record import."),
        ),
    }
}

/// Returns the default retry/backoff policy for record import.
fn default_retry_policy() -> RestoreRetryPolicy {
    RestoreRetryPolicy {
        max_retries_on_rate_limit: 5,
        initial_backoff_ms: 1000,
        backoff_multiplier: 2.0,
        note: "On a 429 rate-limit response, the import waits for the Retry-After header duration \
            (or the initial backoff if absent) and retries. Backoff doubles each retry up to the \
            maximum. No progress is lost — the current batch is retried from the beginning."
            .to_string(),
    }
}

/// Returns a blocked plan with a single error.
fn blocked_plan(
    filename: String,
    request: &RestoreRecordImportPlanRequest,
    code: &str,
    message: &str,
) -> RestoreRecordImportPlan {
    RestoreRecordImportPlan {
        filename,
        status: RestoreRecordImportPlanStatus::Blocked,
        target_mode: request.target_mode.clone(),
        target_base_name: request.target_base_name.clone(),
        table_plans: vec![],
        linked_record_update_plans: vec![],
        retry_policy: default_retry_policy(),
        warnings: vec![],
        errors: vec![RestoreRecordImportError {
            code: code.to_string(),
            message: message.to_string(),
        }],
        no_changes_made: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordBatchPhase,
        RestoreRecordImportPlanRequest,
    };

    fn make_request(
        dry_run_status: &str,
        schema_plan_status: &str,
        tables: Vec<RecordImportTableInput>,
    ) -> RestoreRecordImportPlanRequest {
        RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: dry_run_status.to_string(),
            schema_plan_status: schema_plan_status.to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables,
        }
    }

    fn simple_table(id: &str, name: &str, record_count: Option<usize>) -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: id.to_string(),
            table_name: name.to_string(),
            record_count,
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
                    field_name: "Autonumber".to_string(),
                    field_type: "autoNumber".to_string(),
                    linked_table_id: None,
                },
            ],
        }
    }

    // ── Gate tests ─────────────────────────────────────────────────────────

    #[test]
    fn ready_dry_run_and_schema_plan_produces_import_plan() {
        let req = make_request(
            "ready",
            "ready",
            vec![simple_table("tbl01", "Projects", Some(10))],
        );
        let plan = create_record_import_plan(&req);
        assert!(
            plan.status == RestoreRecordImportPlanStatus::Ready
                || plan.status == RestoreRecordImportPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn ready_with_warnings_statuses_accepted() {
        let req = make_request(
            "readyWithWarnings",
            "readyWithWarnings",
            vec![simple_table("tbl01", "Projects", Some(10))],
        );
        let plan = create_record_import_plan(&req);
        assert_ne!(plan.status, RestoreRecordImportPlanStatus::Blocked);
    }

    #[test]
    fn blocked_dry_run_produces_blocked_plan() {
        let req = make_request(
            "blocked",
            "ready",
            vec![simple_table("tbl01", "Projects", Some(10))],
        );
        let plan = create_record_import_plan(&req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert!(!plan.errors.is_empty());
        assert_eq!(plan.errors[0].code, "DRY_RUN_BLOCKED");
    }

    #[test]
    fn blocked_schema_plan_produces_blocked_plan() {
        let req = make_request(
            "ready",
            "blocked",
            vec![simple_table("tbl01", "Projects", Some(10))],
        );
        let plan = create_record_import_plan(&req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert_eq!(plan.errors[0].code, "SCHEMA_PLAN_BLOCKED");
    }

    #[test]
    fn empty_tables_produces_blocked_plan() {
        let req = make_request("ready", "ready", vec![]);
        let plan = create_record_import_plan(&req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert_eq!(plan.errors[0].code, "NO_TABLES");
    }

    // ── Batch count tests ──────────────────────────────────────────────────

    #[test]
    fn known_record_count_produces_correct_batch_count() {
        // 25 records, batch size 10 → 3 batches
        let req = make_request(
            "ready",
            "ready",
            vec![simple_table("tbl01", "Projects", Some(25))],
        );
        let plan = create_record_import_plan(&req);
        assert_eq!(plan.table_plans[0].create_batch_count, Some(3));
        assert_eq!(plan.table_plans[0].first_pass_batches.len(), 3);
    }

    #[test]
    fn known_record_count_exact_multiple() {
        // 20 records, batch size 10 → 2 batches
        let req = make_request(
            "ready",
            "ready",
            vec![simple_table("tbl01", "Projects", Some(20))],
        );
        let plan = create_record_import_plan(&req);
        assert_eq!(plan.table_plans[0].create_batch_count, Some(2));
    }

    #[test]
    fn unknown_record_count_stays_unknown() {
        let req = make_request(
            "ready",
            "ready",
            vec![simple_table("tbl01", "Projects", None)],
        );
        let plan = create_record_import_plan(&req);
        let tp = &plan.table_plans[0];
        assert!(tp.create_batch_count.is_none());
        assert!(!tp.record_count_known);
        assert!(tp.first_pass_batches.is_empty());
    }

    // ── Linked record tests ────────────────────────────────────────────────

    #[test]
    fn linked_record_fields_create_second_pass_plan() {
        let req = make_request("ready", "ready", vec![complex_table()]);
        let plan = create_record_import_plan(&req);
        assert!(!plan.table_plans[0].linked_record_updates.is_empty());
        assert!(!plan.linked_record_update_plans.is_empty());
    }

    #[test]
    fn linked_record_second_pass_batches_are_planned() {
        let req = make_request("ready", "ready", vec![complex_table()]);
        let plan = create_record_import_plan(&req);
        assert!(!plan.table_plans[0].second_pass_batches.is_empty());
        assert_eq!(
            plan.table_plans[0].second_pass_batches[0].phase,
            RestoreRecordBatchPhase::UpdateLinkedRecords
        );
    }

    // ── Attachment tests ───────────────────────────────────────────────────

    #[test]
    fn attachment_fields_create_metadata_policy() {
        let req = make_request("ready", "ready", vec![complex_table()]);
        let plan = create_record_import_plan(&req);
        let att = &plan.table_plans[0].attachment_policies;
        assert!(!att.is_empty());
        assert_eq!(
            att[0].policy,
            crate::restore::record_import_plan::RestoreAttachmentRestorePolicy::MetadataOnly
        );
    }

    // ── Skipped field tests ────────────────────────────────────────────────

    #[test]
    fn computed_fields_are_skipped() {
        let req = make_request("ready", "ready", vec![complex_table()]);
        let plan = create_record_import_plan(&req);
        let skipped = plan.table_plans[0]
            .field_policies
            .iter()
            .filter(|p| p.policy == RestoreRecordFieldImportPolicy::Skip)
            .count();
        assert!(
            skipped >= 2,
            "formula and autoNumber should both be skipped"
        );
    }

    // ── Mapping tests ──────────────────────────────────────────────────────

    #[test]
    fn mapping_strategy_is_execution_time() {
        let req = make_request(
            "ready",
            "ready",
            vec![simple_table("tbl01", "Projects", Some(10))],
        );
        let plan = create_record_import_plan(&req);
        // The strategy should not be UnavailableUntilExecution — it is planned,
        // but the actual IDs are only known at execution time.
        assert_eq!(
            plan.table_plans[0].mapping_plan.strategy,
            crate::restore::record_import_plan::RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId
        );
    }

    #[test]
    fn serialized_plan_does_not_contain_fake_record_ids() {
        let req = make_request("ready", "ready", vec![complex_table()]);
        let plan = create_record_import_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        // Should not contain any fake pre-assigned new record IDs.
        assert!(!json.contains("\"newRecordId\""));
        assert!(!json.contains("rec_new_"));
    }

    // ── Checkpoint tests ───────────────────────────────────────────────────

    #[test]
    fn checkpoint_plan_serializes_correctly() {
        let req = make_request("ready", "ready", vec![complex_table()]);
        let plan = create_record_import_plan(&req);
        let cp = &plan.table_plans[0].checkpoint_plan;
        let json = serde_json::to_string(cp).expect("serialize");
        assert!(json.contains("checkpointBatchIndex"));
        assert!(json.contains("sourceRecordIdOffsetPlaceholder"));
        assert!(json.contains("completedPhase"));
    }

    // ── Safety tests ───────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_is_always_true() {
        for (dry, schema) in &[
            ("ready", "ready"),
            ("readyWithWarnings", "readyWithWarnings"),
            ("blocked", "ready"),
            ("ready", "blocked"),
        ] {
            let req = make_request(dry, schema, vec![complex_table()]);
            let plan = create_record_import_plan(&req);
            assert!(
                plan.no_changes_made,
                "no_changes_made must be true for dry={dry}, schema={schema}"
            );
        }
    }

    #[test]
    fn request_has_no_token_field() {
        let req = make_request("ready", "ready", vec![simple_table("tbl01", "T", Some(5))]);
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn serialized_result_contains_no_token_sentinel() {
        const SENTINEL: &str = "pat_import_planner_test_sentinel_9999999";
        let req = make_request("ready", "ready", vec![simple_table("tbl01", "T", Some(5))]);
        let plan = create_record_import_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn filename_is_not_a_path() {
        let req = make_request("ready", "ready", vec![simple_table("tbl01", "T", Some(5))]);
        let plan = create_record_import_plan(&req);
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
        assert_eq!(plan.filename, "backup.airbridge");
    }

    #[test]
    fn result_has_no_succeeded_status() {
        let req = make_request("ready", "ready", vec![simple_table("tbl01", "T", Some(5))]);
        let plan = create_record_import_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("succeeded"));
    }

    #[test]
    fn result_has_no_changes_made_key() {
        let req = make_request("ready", "ready", vec![simple_table("tbl01", "T", Some(5))]);
        let plan = create_record_import_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
    }

    #[test]
    fn ready_with_warnings_status_when_issues_exist() {
        let req = make_request("ready", "ready", vec![complex_table()]);
        let plan = create_record_import_plan(&req);
        assert_eq!(
            plan.status,
            RestoreRecordImportPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn ready_status_when_no_issues() {
        let req = make_request(
            "ready",
            "ready",
            vec![simple_table("tbl01", "Projects", Some(10))],
        );
        let plan = create_record_import_plan(&req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Ready);
    }

    #[test]
    fn retry_policy_is_present() {
        let req = make_request("ready", "ready", vec![simple_table("tbl01", "T", Some(5))]);
        let plan = create_record_import_plan(&req);
        assert!(plan.retry_policy.max_retries_on_rate_limit > 0);
        assert!(plan.retry_policy.initial_backoff_ms > 0);
        assert!(plan.retry_policy.backoff_multiplier > 1.0);
    }
}
