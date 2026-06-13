use serde::{Deserialize, Serialize};

use crate::restore::schema_write_requests::{
    SchemaWriteBlockedReason, SchemaWriteOperationStatus, SchemaWriteRequestPlan,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::{RestoreWriteDisabledReason, RestoreWriteEngineStatus};

/// Result of the schema write executor.
///
/// Safety properties:
/// - No token field.
/// - No absolute paths — filename only.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - status is never `succeeded`.
/// - No Airtable client is constructed or called.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteDryRunResult {
    /// Filename only — never the full package path.
    pub filename: String,
    pub status: RestoreWriteEngineStatus,
    pub disabled_reason: RestoreWriteDisabledReason,
    pub message: String,
    pub operations_planned: usize,
    pub operations_executed: usize,
    pub table_ops_planned: usize,
    pub field_ops_planned: usize,
    pub deferred_ops_planned: usize,
    pub manual_action_count: usize,
    pub warnings: Vec<String>,
    /// Always true — no Airtable writes were attempted.
    pub no_changes_made: bool,
    /// Always false — the executor skeleton does not call the network.
    pub network_writes_attempted: bool,
}

/// Executes the schema write request plan in dry-run mode.
///
/// Always consults the write gate first. Always returns `Disabled` or `Blocked`.
/// Never calls the Airtable API. Never creates a base, table, or field.
/// No token is required or accepted.
pub fn execute_schema_write_dry_run(
    request_plan: &SchemaWriteRequestPlan,
) -> SchemaWriteDryRunResult {
    let filename = request_plan.filename.clone();

    // Gate check — always disabled
    let gate = evaluate_write_gate();

    // If the request plan itself is blocked (bad input), surface that
    if request_plan.status == SchemaWriteOperationStatus::Blocked {
        let (disabled_reason, message) = match &request_plan.blocked_reason {
            Some(SchemaWriteBlockedReason::SchemaPlanNotReady) => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "Schema plan is not ready — cannot build write request plan.".to_string(),
            ),
            Some(SchemaWriteBlockedReason::NoTablesInPlan) => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "No tables in schema plan — nothing to write.".to_string(),
            ),
            _ => (
                RestoreWriteDisabledReason::BlockedByInvalidPlan,
                "Schema write request plan is blocked.".to_string(),
            ),
        };
        return SchemaWriteDryRunResult {
            filename,
            status: RestoreWriteEngineStatus::Blocked,
            disabled_reason,
            message,
            operations_planned: 0,
            operations_executed: 0,
            table_ops_planned: 0,
            field_ops_planned: 0,
            deferred_ops_planned: 0,
            manual_action_count: 0,
            warnings: request_plan.warnings.clone(),
            no_changes_made: true,
            network_writes_attempted: false,
        };
    }

    // Normal disabled path — write gate is enforced
    SchemaWriteDryRunResult {
        filename,
        status: gate.status,
        disabled_reason: gate.reason,
        message: gate.message,
        operations_planned: request_plan.total_op_count,
        operations_executed: 0, // Always 0 — nothing is executed
        table_ops_planned: request_plan.table_op_count,
        field_ops_planned: request_plan.field_op_count,
        deferred_ops_planned: request_plan.deferred_op_count,
        manual_action_count: request_plan.manual_action_count,
        warnings: request_plan.warnings.clone(),
        no_changes_made: true,
        network_writes_attempted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::schema_plan::RestoreSchemaPlan;
    use crate::restore::schema_plan::{
        RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreManualActionField,
        RestoreSchemaDependencyGraph, RestoreSchemaPlanStatus, RestoreTableCreationStep,
    };
    use crate::restore::schema_write_requests::build_schema_write_request_plan;

    fn simple_schema_plan() -> RestoreSchemaPlan {
        RestoreSchemaPlan {
            filename: "backup.airbridge".to_string(),
            status: RestoreSchemaPlanStatus::Ready,
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            table_steps: vec![RestoreTableCreationStep {
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
                step_index: 0,
                field_count: 2,
                direct_field_count: 2,
                deferred_field_count: 0,
                manual_action_count: 0,
                unsupported_count: 0,
                note: "Create table 'Tasks'.".to_string(),
            }],
            field_steps: vec![RestoreFieldCreationStep {
                field_id: "fld001".to_string(),
                field_name: "Title".to_string(),
                field_type: "singleLineText".to_string(),
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
                classification: RestoreFieldCreateClassification::CreateDirectly,
                note: "Direct field.".to_string(),
            }],
            deferred_steps: vec![],
            manual_action_fields: vec![RestoreManualActionField {
                field_id: "fld002".to_string(),
                field_name: "Formula".to_string(),
                field_type: "formula".to_string(),
                table_id: "tbl001".to_string(),
                table_name: "Tasks".to_string(),
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

    #[test]
    fn executor_always_returns_disabled() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn executor_no_changes_made_always_true() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert!(result.no_changes_made);
    }

    #[test]
    fn executor_network_writes_attempted_always_false() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn executor_operations_executed_always_zero() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.operations_executed, 0);
    }

    #[test]
    fn executor_result_has_no_token() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn executor_result_has_no_succeeded_status() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn executor_result_has_no_absolute_path() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn executor_disabled_reason_is_product_policy() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(
            result.disabled_reason,
            RestoreWriteDisabledReason::DisabledByProductPolicy
        );
    }

    #[test]
    fn executor_filename_is_basename_only() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.filename, "backup.airbridge");
        assert!(!result.filename.contains('/'));
    }

    #[test]
    fn executor_blocked_plan_returns_blocked_status() {
        let mut blocked_plan = build_schema_write_request_plan(&simple_schema_plan());
        blocked_plan.status = SchemaWriteOperationStatus::Blocked;
        blocked_plan.blocked_reason = Some(SchemaWriteBlockedReason::SchemaPlanNotReady);
        let result = execute_schema_write_dry_run(&blocked_plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Blocked);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn executor_no_airtable_client_called() {
        // execute_schema_write_dry_run accepts no HTTP transport parameter.
        // Reaching this assertion confirms no network call was made.
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn executor_operations_planned_matches_request_plan() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let expected_total = request_plan.total_op_count;
        let result = execute_schema_write_dry_run(&request_plan);
        assert_eq!(result.operations_planned, expected_total);
    }

    #[test]
    fn executor_serializes_no_changes_made_key() {
        let request_plan = build_schema_write_request_plan(&simple_schema_plan());
        let result = execute_schema_write_dry_run(&request_plan);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("noChangesMade"));
        assert!(json.contains("networkWritesAttempted"));
    }
}
