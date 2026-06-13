use serde::{Deserialize, Serialize};

use crate::restore::schema_plan::{
    RestoreFieldCreateClassification, RestoreSchemaPlan, RestoreSchemaPlanStatus,
};
use crate::restore::schema_steps::classify_field_for_schema;

/// What kind of schema write operation is represented.
///
/// Note: `create_base` is reserved for a future mode where a new base is
/// created from scratch. In v0.1, only table and field operations are planned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteOperationKind {
    CreateBase,
    CreateTable,
    CreateField,
    DeferLinkedField,
    ManualAction,
}

/// Planning-time status for one schema write operation.
///
/// Note: `success` / `succeeded` are intentionally absent.
/// No operation is executed; all statuses are planning-time only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteOperationStatus {
    Planned,
    Blocked,
    Disabled,
}

/// A single planned schema write operation.
///
/// Represents what *would* be sent to the Airtable API when the write engine
/// is enabled. In this version the operation is never executed.
///
/// Safety properties:
/// - No token field.
/// - No base_id / table_id from a live Airtable response (only from backup plan).
/// - status is never `succeeded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteOperation {
    /// Ordering index within the full plan (0-based, tables before fields).
    pub index: usize,
    pub kind: SchemaWriteOperationKind,
    pub status: SchemaWriteOperationStatus,
    /// Source table ID from the backup package (not a live Airtable ID).
    pub source_table_id: String,
    pub table_name: String,
    /// Present for field operations only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_field_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
    /// For deferred linked fields: the target table's source ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_source_table_id: Option<String>,
    pub note: String,
    /// Always true — the operation has not been executed.
    pub no_changes_made: bool,
}

/// Why the schema write request plan is blocked or disabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteBlockedReason {
    DisabledByProductPolicy,
    SchemaPlanNotReady,
    NoTablesInPlan,
}

/// A full schema write request plan built from an existing RestoreSchemaPlan.
///
/// Safety properties:
/// - No token field.
/// - No full paths — filename only.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - status is never `succeeded`.
/// - Operations are ordered: tables first, then fields, then deferred linked
///   fields, then manual actions. This ordering must be preserved because
///   field creation depends on table existence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteRequestPlan {
    /// Filename only — never the full package path.
    pub filename: String,
    pub status: SchemaWriteOperationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<SchemaWriteBlockedReason>,
    /// Ordered list of all planned operations.
    pub operations: Vec<SchemaWriteOperation>,
    pub table_op_count: usize,
    pub field_op_count: usize,
    pub deferred_op_count: usize,
    pub manual_action_count: usize,
    pub total_op_count: usize,
    pub warnings: Vec<String>,
    /// Always true — no Airtable API calls were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
}

/// Builds a schema write request plan from an existing schema plan.
///
/// Ordering contract (enforced here):
///   1. CreateTable operations — one per table, in schema plan order.
///   2. CreateField operations — directly-creatable fields, table by table.
///   3. DeferLinkedField operations — linked record fields deferred to second pass.
///   4. ManualAction operations — computed, collaborator, and unsupported fields.
///
/// All operations are `Disabled` because the write gate blocks execution.
/// No Airtable API calls are made. No token is required.
pub fn build_schema_write_request_plan(schema_plan: &RestoreSchemaPlan) -> SchemaWriteRequestPlan {
    let filename = schema_plan.filename.clone();

    // Gate: plan must be ready
    if schema_plan.status == RestoreSchemaPlanStatus::Blocked {
        return SchemaWriteRequestPlan {
            filename,
            status: SchemaWriteOperationStatus::Blocked,
            blocked_reason: Some(SchemaWriteBlockedReason::SchemaPlanNotReady),
            operations: vec![],
            table_op_count: 0,
            field_op_count: 0,
            deferred_op_count: 0,
            manual_action_count: 0,
            total_op_count: 0,
            warnings: schema_plan
                .warnings
                .iter()
                .map(|w| w.message.clone())
                .collect(),
            no_changes_made: true,
            network_writes_attempted: false,
        };
    }

    // Gate: must have tables
    if schema_plan.table_steps.is_empty() {
        return SchemaWriteRequestPlan {
            filename,
            status: SchemaWriteOperationStatus::Blocked,
            blocked_reason: Some(SchemaWriteBlockedReason::NoTablesInPlan),
            operations: vec![],
            table_op_count: 0,
            field_op_count: 0,
            deferred_op_count: 0,
            manual_action_count: 0,
            total_op_count: 0,
            warnings: vec![],
            no_changes_made: true,
            network_writes_attempted: false,
        };
    }

    let mut operations: Vec<SchemaWriteOperation> = Vec::new();
    let mut index = 0usize;

    // Phase 1 — CreateTable operations (must come before any field ops)
    for table_step in &schema_plan.table_steps {
        operations.push(SchemaWriteOperation {
            index,
            kind: SchemaWriteOperationKind::CreateTable,
            status: SchemaWriteOperationStatus::Disabled,
            source_table_id: table_step.table_id.clone(),
            table_name: table_step.table_name.clone(),
            source_field_id: None,
            field_name: None,
            field_type: None,
            linked_source_table_id: None,
            note: format!(
                "Create table '{}' — disabled. Would create table with {} direct field(s).",
                table_step.table_name, table_step.direct_field_count
            ),
            no_changes_made: true,
        });
        index += 1;
    }

    let table_op_count = operations.len();

    // Phase 2 — CreateField operations for directly-creatable fields
    for field_step in &schema_plan.field_steps {
        let classification = classify_field_for_schema(&field_step.field_type);
        let is_direct = matches!(
            classification,
            RestoreFieldCreateClassification::CreateDirectly
                | RestoreFieldCreateClassification::CreateWithAdjustment
        );
        if !is_direct {
            continue;
        }
        operations.push(SchemaWriteOperation {
            index,
            kind: SchemaWriteOperationKind::CreateField,
            status: SchemaWriteOperationStatus::Disabled,
            source_table_id: field_step.table_id.clone(),
            table_name: field_step.table_name.clone(),
            source_field_id: Some(field_step.field_id.clone()),
            field_name: Some(field_step.field_name.clone()),
            field_type: Some(field_step.field_type.clone()),
            linked_source_table_id: None,
            note: format!(
                "Create field '{}' ({}) in table '{}' — disabled.",
                field_step.field_name, field_step.field_type, field_step.table_name
            ),
            no_changes_made: true,
        });
        index += 1;
    }

    let field_op_count = operations.len() - table_op_count;

    // Phase 3 — DeferLinkedField operations
    for deferred in &schema_plan.deferred_steps {
        operations.push(SchemaWriteOperation {
            index,
            kind: SchemaWriteOperationKind::DeferLinkedField,
            status: SchemaWriteOperationStatus::Disabled,
            source_table_id: deferred.table_id.clone(),
            table_name: deferred.table_name.clone(),
            source_field_id: Some(deferred.field_id.clone()),
            field_name: Some(deferred.field_name.clone()),
            field_type: Some(deferred.field_type.clone()),
            linked_source_table_id: deferred.linked_table_id.clone(),
            note: format!(
                "Linked field '{}' in '{}' deferred — all tables must exist first.",
                deferred.field_name, deferred.table_name
            ),
            no_changes_made: true,
        });
        index += 1;
    }

    let deferred_op_count = operations.len() - table_op_count - field_op_count;

    // Phase 4 — ManualAction operations
    for manual in &schema_plan.manual_action_fields {
        operations.push(SchemaWriteOperation {
            index,
            kind: SchemaWriteOperationKind::ManualAction,
            status: SchemaWriteOperationStatus::Disabled,
            source_table_id: manual.table_id.clone(),
            table_name: manual.table_name.clone(),
            source_field_id: Some(manual.field_id.clone()),
            field_name: Some(manual.field_name.clone()),
            field_type: Some(manual.field_type.clone()),
            linked_source_table_id: None,
            note: manual.action_description.clone(),
            no_changes_made: true,
        });
        index += 1;
    }

    let manual_action_count =
        operations.len() - table_op_count - field_op_count - deferred_op_count;
    let total_op_count = operations.len();

    let warnings: Vec<String> = schema_plan
        .warnings
        .iter()
        .map(|w| w.message.clone())
        .collect();

    SchemaWriteRequestPlan {
        filename,
        status: SchemaWriteOperationStatus::Disabled,
        blocked_reason: Some(SchemaWriteBlockedReason::DisabledByProductPolicy),
        operations,
        table_op_count,
        field_op_count,
        deferred_op_count,
        manual_action_count,
        total_op_count,
        warnings,
        no_changes_made: true,
        network_writes_attempted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::schema_plan::{
        RestoreDeferredFieldStep, RestoreFieldCreationStep, RestoreManualActionField,
        RestoreSchemaDependencyGraph, RestoreSchemaPlanStatus, RestoreTableCreationStep,
    };

    fn minimal_plan(status: RestoreSchemaPlanStatus) -> RestoreSchemaPlan {
        RestoreSchemaPlan {
            filename: "backup.airbridge".to_string(),
            status,
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            table_steps: vec![RestoreTableCreationStep {
                table_id: "tbl001".to_string(),
                table_name: "Projects".to_string(),
                step_index: 0,
                field_count: 3,
                direct_field_count: 2,
                deferred_field_count: 1,
                manual_action_count: 0,
                unsupported_count: 0,
                note: "Create table 'Projects'.".to_string(),
            }],
            field_steps: vec![
                RestoreFieldCreationStep {
                    field_id: "fld001".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    table_id: "tbl001".to_string(),
                    table_name: "Projects".to_string(),
                    classification: RestoreFieldCreateClassification::CreateDirectly,
                    note: "Direct field.".to_string(),
                },
                RestoreFieldCreationStep {
                    field_id: "fld002".to_string(),
                    field_name: "Status".to_string(),
                    field_type: "singleSelect".to_string(),
                    table_id: "tbl001".to_string(),
                    table_name: "Projects".to_string(),
                    classification: RestoreFieldCreateClassification::CreateDirectly,
                    note: "Direct field.".to_string(),
                },
            ],
            deferred_steps: vec![RestoreDeferredFieldStep {
                field_id: "fld003".to_string(),
                field_name: "Related".to_string(),
                field_type: "multipleRecordLinks".to_string(),
                table_id: "tbl001".to_string(),
                table_name: "Projects".to_string(),
                reason: "Deferred.".to_string(),
                linked_table_id: Some("tbl002".to_string()),
            }],
            manual_action_fields: vec![RestoreManualActionField {
                field_id: "fld004".to_string(),
                field_name: "Calc".to_string(),
                field_type: "formula".to_string(),
                table_id: "tbl001".to_string(),
                table_name: "Projects".to_string(),
                action_description: "Recreate manually.".to_string(),
            }],
            dependency_graph: RestoreSchemaDependencyGraph {
                edges: vec![],
                has_circular_dependency: false,
                resolution_note: String::new(),
            },
            warnings: vec![],
            errors: vec![],
            no_changes_made: true,
        }
    }

    fn ready_plan() -> RestoreSchemaPlan {
        minimal_plan(RestoreSchemaPlanStatus::Ready)
    }

    #[test]
    fn table_ops_come_before_field_ops() {
        let plan = build_schema_write_request_plan(&ready_plan());
        let ops = &plan.operations;
        // Find the last CreateTable index and the first CreateField index
        let last_table = ops
            .iter()
            .filter(|o| o.kind == SchemaWriteOperationKind::CreateTable)
            .map(|o| o.index)
            .max()
            .expect("should have table ops");
        let first_field = ops
            .iter()
            .filter(|o| o.kind == SchemaWriteOperationKind::CreateField)
            .map(|o| o.index)
            .min()
            .expect("should have field ops");
        assert!(
            last_table < first_field,
            "last CreateTable ({last_table}) must come before first CreateField ({first_field})"
        );
    }

    #[test]
    fn linked_fields_are_deferred_not_create_field() {
        let plan = build_schema_write_request_plan(&ready_plan());
        let linked_ops: Vec<_> = plan
            .operations
            .iter()
            .filter(|o| o.field_type.as_deref() == Some("multipleRecordLinks"))
            .collect();
        assert!(
            !linked_ops.is_empty(),
            "must have deferred linked field ops"
        );
        for op in linked_ops {
            assert_eq!(
                op.kind,
                SchemaWriteOperationKind::DeferLinkedField,
                "multipleRecordLinks must be DeferLinkedField, not CreateField"
            );
        }
    }

    #[test]
    fn deferred_ops_come_after_field_ops() {
        let plan = build_schema_write_request_plan(&ready_plan());
        let ops = &plan.operations;
        let last_field = ops
            .iter()
            .filter(|o| o.kind == SchemaWriteOperationKind::CreateField)
            .map(|o| o.index)
            .max();
        let first_deferred = ops
            .iter()
            .filter(|o| o.kind == SchemaWriteOperationKind::DeferLinkedField)
            .map(|o| o.index)
            .min();
        if let (Some(lf), Some(fd)) = (last_field, first_deferred) {
            assert!(
                lf < fd,
                "CreateField ({lf}) must come before DeferLinkedField ({fd})"
            );
        }
    }

    #[test]
    fn manual_ops_come_last() {
        let plan = build_schema_write_request_plan(&ready_plan());
        let ops = &plan.operations;
        let last_non_manual = ops
            .iter()
            .filter(|o| o.kind != SchemaWriteOperationKind::ManualAction)
            .map(|o| o.index)
            .max()
            .unwrap_or(0);
        let first_manual = ops
            .iter()
            .filter(|o| o.kind == SchemaWriteOperationKind::ManualAction)
            .map(|o| o.index)
            .min();
        if let Some(fm) = first_manual {
            assert!(
                last_non_manual < fm,
                "ManualAction ({fm}) must come after all other op kinds ({last_non_manual})"
            );
        }
    }

    #[test]
    fn unsupported_computed_fields_produce_manual_action_ops() {
        let mut plan = ready_plan();
        // Add a formula field to manual_action_fields
        plan.manual_action_fields.push(RestoreManualActionField {
            field_id: "fld005".to_string(),
            field_name: "Formula".to_string(),
            field_type: "rollup".to_string(),
            table_id: "tbl001".to_string(),
            table_name: "Projects".to_string(),
            action_description: "Rollup — recreate manually.".to_string(),
        });
        let result = build_schema_write_request_plan(&plan);
        let rollup_ops: Vec<_> = result
            .operations
            .iter()
            .filter(|o| o.field_type.as_deref() == Some("rollup"))
            .collect();
        assert!(!rollup_ops.is_empty());
        for op in rollup_ops {
            assert_eq!(op.kind, SchemaWriteOperationKind::ManualAction);
        }
    }

    #[test]
    fn all_ops_are_disabled() {
        let plan = build_schema_write_request_plan(&ready_plan());
        for op in &plan.operations {
            assert_eq!(
                op.status,
                SchemaWriteOperationStatus::Disabled,
                "operation {} must be disabled",
                op.index
            );
        }
    }

    #[test]
    fn all_ops_have_no_changes_made_true() {
        let plan = build_schema_write_request_plan(&ready_plan());
        for op in &plan.operations {
            assert!(
                op.no_changes_made,
                "op {} no_changes_made must be true",
                op.index
            );
        }
    }

    #[test]
    fn plan_no_changes_made_always_true() {
        let plan = build_schema_write_request_plan(&ready_plan());
        assert!(plan.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false() {
        let plan = build_schema_write_request_plan(&ready_plan());
        assert!(!plan.network_writes_attempted);
    }

    #[test]
    fn plan_status_is_disabled_not_succeeded() {
        let plan = build_schema_write_request_plan(&ready_plan());
        assert_eq!(plan.status, SchemaWriteOperationStatus::Disabled);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn no_token_in_serialization() {
        let plan = build_schema_write_request_plan(&ready_plan());
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn no_absolute_path_in_serialization() {
        let plan = build_schema_write_request_plan(&ready_plan());
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn blocked_plan_when_schema_blocked() {
        let blocked = minimal_plan(RestoreSchemaPlanStatus::Blocked);
        let plan = build_schema_write_request_plan(&blocked);
        assert_eq!(plan.status, SchemaWriteOperationStatus::Blocked);
        assert!(plan.operations.is_empty());
        assert!(plan.no_changes_made);
        assert!(!plan.network_writes_attempted);
    }

    #[test]
    fn blocked_plan_when_no_tables() {
        let mut empty = ready_plan();
        empty.table_steps.clear();
        let plan = build_schema_write_request_plan(&empty);
        assert_eq!(plan.status, SchemaWriteOperationStatus::Blocked);
        assert_eq!(
            plan.blocked_reason,
            Some(SchemaWriteBlockedReason::NoTablesInPlan)
        );
        assert!(plan.no_changes_made);
    }

    #[test]
    fn op_counts_match_operations_vec() {
        let plan = build_schema_write_request_plan(&ready_plan());
        assert_eq!(
            plan.table_op_count
                + plan.field_op_count
                + plan.deferred_op_count
                + plan.manual_action_count,
            plan.total_op_count
        );
        assert_eq!(plan.total_op_count, plan.operations.len());
    }

    #[test]
    fn ready_with_warnings_plan_is_disabled_not_blocked() {
        let plan_rw = minimal_plan(RestoreSchemaPlanStatus::ReadyWithWarnings);
        let result = build_schema_write_request_plan(&plan_rw);
        assert_eq!(result.status, SchemaWriteOperationStatus::Disabled);
        assert!(result.no_changes_made);
    }
}
