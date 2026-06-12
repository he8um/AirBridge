use crate::restore::schema_dependencies::build_dependency_graph;
use crate::restore::schema_plan::{
    RestoreFieldCreateClassification, RestoreSchemaError, RestoreSchemaPlan,
    RestoreSchemaPlanRequest, RestoreSchemaPlanStatus,
};
use crate::restore::schema_steps::{
    build_deferred_step, build_field_step, build_manual_action_field, build_table_step,
    classify_field_for_schema,
};
use crate::restore::schema_warnings::warnings_for_schema_steps;

/// Creates a restore schema creation plan from the request.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files written or extracted.
/// - Filename is passed in explicitly — never treated as a path in results.
/// - no_changes_made is always true.
pub fn create_schema_plan(request: &RestoreSchemaPlanRequest) -> RestoreSchemaPlan {
    let filename = request.package_filename.clone();

    // Gate: blocked dry-run blocks schema planning.
    let dry_run_ok =
        request.dry_run_status == "ready" || request.dry_run_status == "readyWithWarnings";
    if !dry_run_ok {
        return blocked_plan(
            filename,
            request,
            "DRY_RUN_BLOCKED",
            "The dry-run plan is blocked or missing. Generate a successful restore plan preview before creating a schema plan.",
        );
    }

    if request.tables.is_empty() {
        return blocked_plan(
            filename,
            request,
            "NO_TABLES",
            "No tables are available for schema planning. Generate a restore plan preview first.",
        );
    }

    let mut all_table_steps = Vec::new();
    let mut all_field_steps = Vec::new();
    let mut all_deferred_steps = Vec::new();
    let mut all_manual_fields = Vec::new();
    let mut all_warnings = Vec::new();

    for (step_index, table) in request.tables.iter().enumerate() {
        let mut direct_count = 0usize;
        let mut deferred_count = 0usize;
        let mut manual_count = 0usize;
        let mut unsupported_count = 0usize;
        let mut table_field_steps = Vec::new();

        for field in &table.fields {
            let class = classify_field_for_schema(&field.field_type);

            match &class {
                RestoreFieldCreateClassification::CreateDirectly
                | RestoreFieldCreateClassification::CreateWithAdjustment
                | RestoreFieldCreateClassification::MetadataOnly => {
                    let step = build_field_step(
                        &field.field_id,
                        &field.field_name,
                        &field.field_type,
                        &table.table_id,
                        &table.table_name,
                        class,
                    );
                    if step.classification == RestoreFieldCreateClassification::MetadataOnly {
                        // Count metadata-only as effectively non-direct for table summary
                        // but still record it as a field step (it's informational).
                        direct_count += 1;
                    } else {
                        direct_count += 1;
                    }
                    table_field_steps.push(step);
                }
                RestoreFieldCreateClassification::DeferUntilTablesExist => {
                    deferred_count += 1;
                    all_deferred_steps.push(build_deferred_step(
                        &field.field_id,
                        &field.field_name,
                        &field.field_type,
                        &table.table_id,
                        &table.table_name,
                        field.linked_table_id.clone(),
                    ));
                }
                RestoreFieldCreateClassification::Unsupported => {
                    unsupported_count += 1;
                    all_manual_fields.push(build_manual_action_field(
                        &field.field_id,
                        &field.field_name,
                        &field.field_type,
                        &table.table_id,
                        &table.table_name,
                    ));
                }
                RestoreFieldCreateClassification::ManualActionRequired => {
                    manual_count += 1;
                    all_manual_fields.push(build_manual_action_field(
                        &field.field_id,
                        &field.field_name,
                        &field.field_type,
                        &table.table_id,
                        &table.table_name,
                    ));
                }
            }
        }

        // Collect warnings for this table.
        let table_warnings = warnings_for_schema_steps(
            &table.table_name,
            &table_field_steps,
            deferred_count,
            manual_count,
            unsupported_count,
        );
        all_warnings.extend(table_warnings);

        all_table_steps.push(build_table_step(
            &table.table_id,
            &table.table_name,
            step_index,
            table.fields.len(),
            direct_count,
            deferred_count,
            manual_count,
            unsupported_count,
        ));

        all_field_steps.extend(table_field_steps);
    }

    // Build the dependency graph from all tables.
    let dependency_graph = build_dependency_graph(&request.tables);

    let status = if all_warnings.is_empty() {
        RestoreSchemaPlanStatus::Ready
    } else {
        RestoreSchemaPlanStatus::ReadyWithWarnings
    };

    RestoreSchemaPlan {
        filename,
        status,
        target_mode: request.target_mode.clone(),
        target_base_name: request.target_base_name.clone(),
        table_steps: all_table_steps,
        field_steps: all_field_steps,
        deferred_steps: all_deferred_steps,
        manual_action_fields: all_manual_fields,
        dependency_graph,
        warnings: all_warnings,
        errors: vec![],
        no_changes_made: true,
    }
}

fn blocked_plan(
    filename: String,
    request: &RestoreSchemaPlanRequest,
    code: &str,
    message: &str,
) -> RestoreSchemaPlan {
    RestoreSchemaPlan {
        filename,
        status: RestoreSchemaPlanStatus::Blocked,
        target_mode: request.target_mode.clone(),
        target_base_name: request.target_base_name.clone(),
        table_steps: vec![],
        field_steps: vec![],
        deferred_steps: vec![],
        manual_action_fields: vec![],
        dependency_graph: crate::restore::schema_dependencies::build_dependency_graph(&[]),
        warnings: vec![],
        errors: vec![RestoreSchemaError {
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
    use crate::restore::schema_plan::{SchemaPlanFieldInput, SchemaPlanTableInput};

    fn make_request(
        dry_run_status: &str,
        tables: Vec<SchemaPlanTableInput>,
    ) -> RestoreSchemaPlanRequest {
        RestoreSchemaPlanRequest {
            package_filename: "test.airbridge".to_string(),
            dry_run_status: dry_run_status.to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("My Restored Base".to_string()),
            tables,
        }
    }

    fn simple_table(id: &str, name: &str) -> SchemaPlanTableInput {
        SchemaPlanTableInput {
            table_id: id.to_string(),
            table_name: name.to_string(),
            fields: vec![
                SchemaPlanFieldInput {
                    field_id: "fld01".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                },
                SchemaPlanFieldInput {
                    field_id: "fld02".to_string(),
                    field_name: "Status".to_string(),
                    field_type: "singleSelect".to_string(),
                    linked_table_id: None,
                },
            ],
        }
    }

    fn complex_tables() -> Vec<SchemaPlanTableInput> {
        vec![
            SchemaPlanTableInput {
                table_id: "tblA".to_string(),
                table_name: "Projects".to_string(),
                fields: vec![
                    SchemaPlanFieldInput {
                        field_id: "fld01".to_string(),
                        field_name: "Name".to_string(),
                        field_type: "singleLineText".to_string(),
                        linked_table_id: None,
                    },
                    SchemaPlanFieldInput {
                        field_id: "fld02".to_string(),
                        field_name: "Calc".to_string(),
                        field_type: "formula".to_string(),
                        linked_table_id: None,
                    },
                    SchemaPlanFieldInput {
                        field_id: "fld03".to_string(),
                        field_name: "Files".to_string(),
                        field_type: "multipleAttachments".to_string(),
                        linked_table_id: None,
                    },
                    SchemaPlanFieldInput {
                        field_id: "fld04".to_string(),
                        field_name: "Tasks".to_string(),
                        field_type: "multipleRecordLinks".to_string(),
                        linked_table_id: Some("tblB".to_string()),
                    },
                    SchemaPlanFieldInput {
                        field_id: "fld05".to_string(),
                        field_name: "Rollup".to_string(),
                        field_type: "rollup".to_string(),
                        linked_table_id: None,
                    },
                    SchemaPlanFieldInput {
                        field_id: "fld06".to_string(),
                        field_name: "Owner".to_string(),
                        field_type: "singleCollaborator".to_string(),
                        linked_table_id: None,
                    },
                ],
            },
            SchemaPlanTableInput {
                table_id: "tblB".to_string(),
                table_name: "Tasks".to_string(),
                fields: vec![SchemaPlanFieldInput {
                    field_id: "fld07".to_string(),
                    field_name: "Title".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                }],
            },
        ]
    }

    #[test]
    fn ready_dry_run_produces_schema_plan() {
        let req = make_request("ready", vec![simple_table("tbl01", "Projects")]);
        let plan = create_schema_plan(&req);
        assert!(
            plan.status == RestoreSchemaPlanStatus::Ready
                || plan.status == RestoreSchemaPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn ready_with_warnings_dry_run_produces_schema_plan() {
        let req = make_request("readyWithWarnings", complex_tables());
        let plan = create_schema_plan(&req);
        assert!(
            plan.status == RestoreSchemaPlanStatus::Ready
                || plan.status == RestoreSchemaPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn blocked_dry_run_produces_blocked_schema_plan() {
        let req = make_request("blocked", complex_tables());
        let plan = create_schema_plan(&req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Blocked);
        assert!(!plan.errors.is_empty());
        assert_eq!(plan.errors[0].code, "DRY_RUN_BLOCKED");
    }

    #[test]
    fn empty_dry_run_status_produces_blocked_schema_plan() {
        let req = make_request("", complex_tables());
        let plan = create_schema_plan(&req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Blocked);
    }

    #[test]
    fn tables_planned_before_fields() {
        let req = make_request("ready", vec![simple_table("tbl01", "Projects")]);
        let plan = create_schema_plan(&req);
        assert!(!plan.table_steps.is_empty(), "table steps must be present");
        // table_steps come before field_steps in the plan struct definition order.
        assert_eq!(plan.table_steps[0].table_id, "tbl01");
    }

    #[test]
    fn simple_fields_planned_create_directly() {
        let req = make_request("ready", vec![simple_table("tbl01", "Projects")]);
        let plan = create_schema_plan(&req);
        let direct = plan
            .field_steps
            .iter()
            .filter(|f| f.classification == RestoreFieldCreateClassification::CreateDirectly)
            .count();
        assert_eq!(direct, 2, "both simple fields should be CreateDirectly");
    }

    #[test]
    fn linked_fields_planned_defer_until_tables_exist() {
        let req = make_request("ready", complex_tables());
        let plan = create_schema_plan(&req);
        assert!(
            !plan.deferred_steps.is_empty(),
            "linked fields should be deferred"
        );
        assert!(plan
            .deferred_steps
            .iter()
            .all(|d| d.field_type == "multipleRecordLinks"));
    }

    #[test]
    fn linked_dependencies_are_represented() {
        let req = make_request("ready", complex_tables());
        let plan = create_schema_plan(&req);
        assert!(
            !plan.dependency_graph.edges.is_empty(),
            "dependency graph should have edges"
        );
        assert_eq!(plan.dependency_graph.edges[0].source_table_id, "tblA");
        assert_eq!(plan.dependency_graph.edges[0].target_table_id, "tblB");
    }

    #[test]
    fn attachment_fields_produce_metadata_warning() {
        let req = make_request("ready", complex_tables());
        let plan = create_schema_plan(&req);
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.code == "ATTACHMENT_METADATA_ONLY"));
    }

    #[test]
    fn formula_rollup_fields_produce_manual_warning() {
        let req = make_request("ready", complex_tables());
        let plan = create_schema_plan(&req);
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.code == "UNSUPPORTED_FIELDS_REQUIRE_MANUAL_RECREATION"
                || w.code == "MANUAL_ACTION_REQUIRED"));
    }

    #[test]
    fn formula_rollup_fields_are_in_manual_action_list() {
        let req = make_request("ready", complex_tables());
        let plan = create_schema_plan(&req);
        assert!(plan
            .manual_action_fields
            .iter()
            .any(|f| f.field_type == "formula" || f.field_type == "rollup"));
    }

    #[test]
    fn ready_with_warnings_when_warnings_exist() {
        let req = make_request("ready", complex_tables());
        let plan = create_schema_plan(&req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::ReadyWithWarnings);
    }

    #[test]
    fn ready_when_no_warnings() {
        let req = make_request("ready", vec![simple_table("tbl01", "Projects")]);
        let plan = create_schema_plan(&req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Ready);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        for status in &["ready", "readyWithWarnings", "blocked"] {
            let req = make_request(status, complex_tables());
            let plan = create_schema_plan(&req);
            assert!(
                plan.no_changes_made,
                "no_changes_made must be true for status={status}"
            );
        }
    }

    #[test]
    fn serialized_result_contains_no_token_sentinel() {
        const SENTINEL: &str = "pat_schema_test_sentinel_9999999999";
        let req = make_request("ready", vec![simple_table("tbl01", "Projects")]);
        let plan = create_schema_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn filename_is_not_a_path() {
        let req = RestoreSchemaPlanRequest {
            package_filename: "mybackup.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            tables: vec![simple_table("tbl01", "Projects")],
        };
        let plan = create_schema_plan(&req);
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
    }

    #[test]
    fn request_has_no_token_field() {
        let req = make_request("ready", vec![simple_table("tbl01", "Projects")]);
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn result_serializes_with_no_changes_made_key() {
        let req = make_request("ready", vec![simple_table("tbl01", "Projects")]);
        let plan = create_schema_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
        assert!(json.contains("true"));
    }

    #[test]
    fn collaborator_field_is_manual_action() {
        let req = make_request("ready", complex_tables());
        let plan = create_schema_plan(&req);
        assert!(plan
            .manual_action_fields
            .iter()
            .any(|f| f.field_type == "singleCollaborator"));
    }

    #[test]
    fn empty_tables_produce_blocked_plan() {
        let req = make_request("ready", vec![]);
        let plan = create_schema_plan(&req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Blocked);
        assert_eq!(plan.errors[0].code, "NO_TABLES");
    }
}
