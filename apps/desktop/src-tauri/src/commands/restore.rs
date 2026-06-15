use crate::errors::AirBridgeResult;
use crate::models::restore::{
    RestoreCompatibilityWarning, RestoreMode, RestorePlanStatus, RestorePlanSummary,
    WarningSeverity,
};
use crate::restore::attachment_upload_policy::{
    verify_attachment_upload_policy, AttachmentUploadPolicyRequest, AttachmentUploadPolicyResult,
};
use crate::restore::destructive_operation_policy::{
    verify_destructive_operation_policy, DestructiveOperationPolicyRequest,
    DestructiveOperationPolicyResult,
};
use crate::restore::dry_run::create_dry_run_plan;
use crate::restore::execution::{RestoreExecutionRequest, RestoreExecutionResult};
use crate::restore::execution_gate::validate_restore_execution_gate;
use crate::restore::live_write_confirmation_policy::{
    verify_live_write_confirmation_policy, LiveWriteConfirmationPolicyRequest,
    LiveWriteConfirmationPolicyResult,
};
use crate::restore::plan::{RestoreDryRunPlan, RestoreDryRunRequest};
use crate::restore::rate_limit_backoff_policy::{
    verify_rate_limit_backoff_policy, RateLimitBackoffPolicyRequest, RateLimitBackoffPolicyResult,
};
use crate::restore::record_import_plan::{RestoreRecordImportPlan, RestoreRecordImportPlanRequest};
use crate::restore::record_import_planner::create_record_import_plan;
use crate::restore::record_write_executor::execute_record_write_dry_run;
use crate::restore::record_write_requests::build_record_write_request_plan;
use crate::restore::record_write_result::{
    RecordWriteRequestPlanRequest, RecordWriteRequestPlanResult,
};
use crate::restore::restore_confirmation::{
    validate_restore_confirmation, RestoreConfirmationRequest, RestoreConfirmationResult,
};
use crate::restore::sandbox_verification::{
    verify_sandbox_environment, SandboxVerificationRequest, SandboxVerificationResult,
};
use crate::restore::sandbox_write_testing_policy::{
    verify_sandbox_write_testing_policy, SandboxWriteTestingPolicyRequest,
    SandboxWriteTestingPolicyResult,
};
use crate::restore::schema_plan::{RestoreSchemaPlan, RestoreSchemaPlanRequest};
use crate::restore::schema_planner::create_schema_plan;
use crate::restore::schema_record_order_policy::{
    verify_schema_record_order_policy, SchemaRecordOrderPolicyRequest,
    SchemaRecordOrderPolicyResult,
};
use crate::restore::schema_write_executor::execute_schema_write_dry_run;
use crate::restore::schema_write_requests::build_schema_write_request_plan;
use crate::restore::schema_write_result::{
    SchemaWriteRequestPlanRequest, SchemaWriteRequestPlanResult,
};
use crate::restore::target_empty_verification::{
    verify_target_empty, TargetEmptyVerificationRequest, TargetEmptyVerificationResult,
};
use crate::restore::write_engine::{preview_write_engine, RestoreWriteEngineRequest};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineResult;

#[tauri::command]
pub fn list_restore_plans() -> AirBridgeResult<Vec<RestorePlanSummary>> {
    Ok(vec![RestorePlanSummary {
        id: "plan-001".to_string(),
        package_id: "pkg-001".to_string(),
        connection_id: "conn-002".to_string(),
        target_base_id: None,
        mode: RestoreMode::NewBase,
        status: RestorePlanStatus::Ready,
        warnings: vec![
            RestoreCompatibilityWarning {
                field_id: "fldProjFormula".to_string(),
                field_name: "Formula Result".to_string(),
                field_type: "formula".to_string(),
                message: "Formula fields cannot be restored via the API. The field must be recreated manually.".to_string(),
                severity: WarningSeverity::Warning,
            },
            RestoreCompatibilityWarning {
                field_id: "fldTaskRollup".to_string(),
                field_name: "Rollup Count".to_string(),
                field_type: "rollup".to_string(),
                message: "Rollup configuration is captured in the schema backup but computed values will not be restored.".to_string(),
                severity: WarningSeverity::Info,
            },
        ],
        created_at: "2025-01-14T15:00:00Z".to_string(),
    }])
}

/// Creates a restore dry-run plan from an existing `.airbridge` package.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files extracted to disk.
/// - No write operations of any kind.
/// - Returns filename only — the full path is never included in the result.
#[tauri::command]
pub fn create_restore_dry_run_plan(request: RestoreDryRunRequest) -> RestoreDryRunPlan {
    create_dry_run_plan(&request)
}

/// Creates a schema creation plan from a dry-run result.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files written or extracted.
/// - Filename in the result is never a full path.
/// - no_changes_made is always true.
#[tauri::command]
pub fn create_restore_schema_plan(request: RestoreSchemaPlanRequest) -> RestoreSchemaPlan {
    create_schema_plan(&request)
}

/// Creates a record import plan from a dry-run result and schema plan.
///
/// - No Airtable API calls.
/// - No token required.
/// - No files written or extracted.
/// - Filename in the result is never a full path.
/// - no_changes_made is always true.
#[tauri::command]
pub fn create_restore_record_import_plan(
    request: RestoreRecordImportPlanRequest,
) -> RestoreRecordImportPlan {
    create_record_import_plan(&request)
}

/// Validates the restore execution safety gate and returns a blocked/disabled result.
///
/// In this version the write engine is not enabled. The command:
/// - Validates all preconditions (inspection, dry-run, target mode, token, confirmation).
/// - Never calls the Airtable API.
/// - Never writes any file.
/// - Never stores or echoes the token.
/// - Never includes the full package path in the result.
/// - Always sets no_changes_made: true.
#[tauri::command]
pub fn run_restore_execution(request: RestoreExecutionRequest) -> RestoreExecutionResult {
    // Token is forwarded to the gate for presence check only; it is not stored or logged.
    validate_restore_execution_gate(&request)
}

/// Produces a write engine skeleton preview from existing planning outputs.
///
/// - No token field — no Airtable access required.
/// - No Airtable API calls.
/// - No file writes.
/// - Never echoes the package path.
/// - Always sets no_changes_made: true.
/// - Status is always disabled — never succeeded.
#[tauri::command]
pub fn preview_restore_write_engine(
    request: RestoreWriteEngineRequest,
) -> RestoreWriteEngineResult {
    preview_write_engine(&request)
}

/// Builds a schema write request plan from a summary of an existing schema plan.
///
/// No token is accepted. No Airtable calls are made. No schema is written.
/// All operations in the result are `disabled` — the write gate blocks execution.
/// `no_changes_made` is always `true`. `network_writes_attempted` is always `false`.
#[tauri::command]
pub fn preview_schema_write_request_plan(
    request: SchemaWriteRequestPlanRequest,
) -> SchemaWriteRequestPlanResult {
    use crate::restore::schema_write_requests::{
        SchemaWriteBlockedReason, SchemaWriteOperationStatus,
    };
    use crate::restore::schema_write_result::SchemaWriteRequestPlanResult;

    let filename = request.package_filename.clone();

    // Gate: schema plan must be ready
    if request.schema_plan_status == "blocked" {
        return SchemaWriteRequestPlanResult::blocked(
            filename,
            SchemaWriteBlockedReason::SchemaPlanNotReady,
            "Schema plan is not ready — cannot build write request plan.".to_string(),
        );
    }

    // Gate: must have tables
    if request.table_count == 0 {
        return SchemaWriteRequestPlanResult::blocked(
            filename,
            SchemaWriteBlockedReason::NoTablesInPlan,
            "No tables in schema plan — nothing to write.".to_string(),
        );
    }

    // Build a synthetic schema plan from the counts in the request, then run
    // both the request plan builder and the executor skeleton.
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::schema_plan::{
        RestoreFieldCreationStep, RestoreSchemaDependencyGraph, RestoreSchemaPlan,
        RestoreSchemaPlanStatus, RestoreTableCreationStep,
    };
    use crate::restore::schema_steps::classify_field_for_schema;

    // Synthesise a minimal schema plan from the counts so the builder can
    // produce accurate per-operation details without needing full field data.
    let table_steps: Vec<RestoreTableCreationStep> = (0..request.table_count)
        .map(|i| RestoreTableCreationStep {
            table_id: format!("tbl{i:03}"),
            table_name: format!("Table {}", i + 1),
            step_index: i,
            field_count: request.direct_field_count + request.deferred_field_count,
            direct_field_count: request.direct_field_count,
            deferred_field_count: request.deferred_field_count,
            manual_action_count: request.manual_action_count,
            unsupported_count: 0,
            note: format!("Planned table {} of {}.", i + 1, request.table_count),
        })
        .collect();

    // Produce direct field steps (one representative per table for planning purposes)
    let field_steps: Vec<RestoreFieldCreationStep> = if request.direct_field_count > 0 {
        (0..request.direct_field_count)
            .map(|j| RestoreFieldCreationStep {
                field_id: format!("fld_direct_{j:03}"),
                field_name: format!("Field {}", j + 1),
                field_type: "singleLineText".to_string(),
                table_id: "tbl000".to_string(),
                table_name: "Table 1".to_string(),
                classification: classify_field_for_schema("singleLineText"),
                note: "Direct field (planned).".to_string(),
            })
            .collect()
    } else {
        vec![]
    };

    let synthetic_plan = RestoreSchemaPlan {
        filename: filename.clone(),
        status: RestoreSchemaPlanStatus::Ready,
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: None,
        table_steps,
        field_steps,
        deferred_steps: vec![],
        manual_action_fields: vec![],
        dependency_graph: RestoreSchemaDependencyGraph {
            edges: vec![],
            has_circular_dependency: false,
            resolution_note: String::new(),
        },
        warnings: vec![],
        errors: vec![],
        no_changes_made: true,
    };

    let request_plan = build_schema_write_request_plan(&synthetic_plan);
    let dry_run = execute_schema_write_dry_run(&request_plan);

    // Always disabled — gate enforces this
    let gate = evaluate_write_gate();

    let result_status =
        if dry_run.status == crate::restore::write_result::RestoreWriteEngineStatus::Blocked {
            SchemaWriteOperationStatus::Blocked
        } else {
            SchemaWriteOperationStatus::Disabled
        };

    SchemaWriteRequestPlanResult {
        filename,
        status: result_status,
        blocked_reason: None,
        disabled_reason: Some(
            crate::restore::write_result::RestoreWriteDisabledReason::DisabledByProductPolicy,
        ),
        message: gate.message,
        table_op_count: request_plan.table_op_count,
        field_op_count: request_plan.field_op_count,
        deferred_op_count: request_plan.deferred_op_count,
        manual_action_count: request_plan.manual_action_count,
        total_op_count: request_plan.total_op_count,
        warnings: request_plan.warnings,
        no_changes_made: true,
        network_writes_attempted: false,
    }
}

/// Builds a record write request plan from a summary of an existing record import plan.
///
/// No token is accepted. No Airtable calls are made. No records are created, updated, or deleted.
/// All operations in the result are `disabled` — the write gate blocks execution.
/// `no_changes_made` is always `true`. `network_writes_attempted` is always `false`.
/// Raw record payloads are never included — only counts and summaries.
/// Old-to-new record ID mapping is execution-time only and is not resolved here.
#[tauri::command]
pub fn preview_record_write_request_plan(
    request: RecordWriteRequestPlanRequest,
) -> RecordWriteRequestPlanResult {
    use crate::restore::record_write_requests::RecordWriteBlockedReason;

    let filename = request.package_filename.clone();

    // Gate: record import plan must be ready
    if request.record_import_plan_status == "blocked" {
        return RecordWriteRequestPlanResult::blocked(
            filename,
            RecordWriteBlockedReason::RecordImportPlanNotReady,
            "Record import plan is not ready — cannot build record write request plan.".to_string(),
        );
    }

    // Gate: must have tables
    if request.table_count == 0 {
        return RecordWriteRequestPlanResult::blocked(
            filename,
            RecordWriteBlockedReason::NoTablesInPlan,
            "No tables in record import plan — nothing to write.".to_string(),
        );
    }

    // Build a synthetic record import plan from the counts in the request, then run
    // both the request plan builder and the executor skeleton.
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::record_import_batches::{
        build_checkpoint_plan, build_first_pass_batches, build_second_pass_batches,
        AIRTABLE_WRITE_BATCH_SIZE,
    };
    use crate::restore::record_import_plan::RestoreRecordImportPlanRequest;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlan,
        RestoreRecordImportPlanStatus, RestoreRetryPolicy,
    };
    use crate::restore::record_import_planner::create_record_import_plan;

    // Synthesise a minimal per-table batch distribution from the counts.
    // Distribute first-pass batches evenly across tables; remaining go to last.
    let batches_per_table = request.total_first_pass_batches / request.table_count.max(1);
    let remainder = request.total_first_pass_batches % request.table_count.max(1);

    let tables: Vec<RecordImportTableInput> = (0..request.table_count)
        .map(|i| {
            let table_batches = batches_per_table
                + if i + 1 == request.table_count {
                    remainder
                } else {
                    0
                };
            // Derive a representative record count from the planned batch count
            let record_count = if table_batches > 0 {
                Some(table_batches * AIRTABLE_WRITE_BATCH_SIZE)
            } else {
                None
            };
            // Add synthetic linked field for tables that would have second-pass batches
            let second_pass_per_table =
                request.total_second_pass_batches / request.table_count.max(1);
            let has_linked = second_pass_per_table > 0;

            let mut fields = vec![RecordImportFieldInput {
                field_id: format!("fld_name_{i:03}"),
                field_name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
                linked_table_id: None,
            }];
            if has_linked {
                fields.push(RecordImportFieldInput {
                    field_id: format!("fld_link_{i:03}"),
                    field_name: "Linked".to_string(),
                    field_type: "multipleRecordLinks".to_string(),
                    linked_table_id: Some(format!("tbl_linked_{i:03}")),
                });
            }
            // Spread attachment and skipped fields across tables
            if i == 0 {
                for j in 0..request.attachment_field_count {
                    fields.push(RecordImportFieldInput {
                        field_id: format!("fld_att_{j:03}"),
                        field_name: format!("Attachment {}", j + 1),
                        field_type: "multipleAttachments".to_string(),
                        linked_table_id: None,
                    });
                }
                for j in 0..request.skipped_field_count {
                    fields.push(RecordImportFieldInput {
                        field_id: format!("fld_skip_{j:03}"),
                        field_name: format!("Computed {}", j + 1),
                        field_type: "formula".to_string(),
                        linked_table_id: None,
                    });
                }
            }

            RecordImportTableInput {
                table_id: format!("tbl{i:03}"),
                table_name: format!("Table {}", i + 1),
                record_count,
                fields,
            }
        })
        .collect();

    let synthetic_request = RestoreRecordImportPlanRequest {
        package_filename: filename.clone(),
        dry_run_status: "ready".to_string(),
        schema_plan_status: "ready".to_string(),
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: None,
        tables,
    };

    let import_plan = create_record_import_plan(&synthetic_request);
    let request_plan = build_record_write_request_plan(&import_plan);
    let dry_run = execute_record_write_dry_run(&request_plan);

    // Always disabled — gate enforces this
    let gate = evaluate_write_gate();

    use crate::restore::write_result::RestoreWriteEngineStatus;
    let result_status = if dry_run.status == RestoreWriteEngineStatus::Blocked {
        crate::restore::record_write_requests::RecordWriteOperationStatus::Blocked
    } else {
        crate::restore::record_write_requests::RecordWriteOperationStatus::Disabled
    };

    RecordWriteRequestPlanResult {
        filename,
        status: result_status,
        blocked_reason: None,
        disabled_reason: Some(
            crate::restore::write_result::RestoreWriteDisabledReason::DisabledByProductPolicy,
        ),
        message: gate.message,
        create_batch_op_count: request_plan.create_batch_op_count,
        linked_update_op_count: request_plan.linked_update_op_count,
        checkpoint_op_count: request_plan.checkpoint_op_count,
        attachment_op_count: request_plan.attachment_op_count,
        skipped_field_op_count: request_plan.skipped_field_op_count,
        total_op_count: request_plan.total_op_count,
        total_first_pass_batches: request_plan.total_first_pass_batches,
        total_second_pass_batches: request_plan.total_second_pass_batches,
        warnings: request_plan.warnings,
        no_changes_made: true,
        network_writes_attempted: false,
    }
}

/// Verifies local sandbox safety conditions for Gate 1 of the live restore write safety checklist.
///
/// - No Airtable API calls.
/// - No token accepted or returned.
/// - No files written.
/// - No full paths in result.
/// - No write operations of any kind.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
/// - Returns blocked for unsafe target configurations.
#[tauri::command]
pub fn verify_restore_sandbox_environment(
    request: SandboxVerificationRequest,
) -> SandboxVerificationResult {
    verify_sandbox_environment(&request)
}

/// Validates the user's restore confirmation text for Gate 2.
///
/// - No Airtable API calls.
/// - No token accepted or returned.
/// - No files written.
/// - No full paths in result.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
/// - Confirmed status does NOT enable restore writes.
#[tauri::command]
pub fn validate_restore_confirmation_gate(
    request: RestoreConfirmationRequest,
) -> RestoreConfirmationResult {
    validate_restore_confirmation(&request)
}

/// Verifies that the restore target base is empty before any live writes begin (Gate 3).
///
/// - No Airtable write API calls.
/// - No token accepted or returned.
/// - No files written.
/// - No full paths in result.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
/// - Verified status does NOT enable restore writes.
#[tauri::command]
pub fn verify_restore_target_empty(
    request: TargetEmptyVerificationRequest,
) -> TargetEmptyVerificationResult {
    verify_target_empty(&request)
}

/// Verifies that no destructive operations exist in the declared restore plan (Gate 4).
///
/// Safety:
/// - No Airtable API calls.
/// - No token accepted or returned.
/// - No filesystem path accepted or returned.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
/// - Compliant status does NOT enable restore writes.
#[tauri::command]
pub fn verify_destructive_operation_policy_gate(
    request: DestructiveOperationPolicyRequest,
) -> DestructiveOperationPolicyResult {
    verify_destructive_operation_policy(&request)
}

/// Verifies the attachment upload policy for all declared attachment fields (Gate 5).
///
/// Safety:
/// - No Airtable API calls.
/// - No token accepted or returned.
/// - No filesystem path accepted or returned.
/// - No full attachment URL accepted or returned.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
/// - Compliant status does NOT enable restore writes.
/// - Attachment file bytes are never uploaded.
#[tauri::command]
pub fn verify_attachment_upload_policy_gate(
    request: AttachmentUploadPolicyRequest,
) -> AttachmentUploadPolicyResult {
    verify_attachment_upload_policy(&request)
}

/// Verifies that write phases observe schema-before-record ordering (Gate 6).
///
/// Safety:
/// - No Airtable API calls.
/// - No token accepted or returned.
/// - No filesystem path accepted or returned.
/// - No record payload accepted or returned.
/// - no_changes_made is always true.
/// - network_writes_attempted is always false.
/// - writes_enabled is always false.
/// - Compliant status does NOT enable restore writes.
#[tauri::command]
pub fn verify_schema_record_order_policy_gate(
    request: SchemaRecordOrderPolicyRequest,
) -> SchemaRecordOrderPolicyResult {
    verify_schema_record_order_policy(&request)
}

#[tauri::command]
pub fn verify_sandbox_write_testing_policy_gate(
    request: SandboxWriteTestingPolicyRequest,
) -> SandboxWriteTestingPolicyResult {
    verify_sandbox_write_testing_policy(&request)
}

#[tauri::command]
pub fn verify_live_write_confirmation_policy_gate(
    request: LiveWriteConfirmationPolicyRequest,
) -> LiveWriteConfirmationPolicyResult {
    verify_live_write_confirmation_policy(&request)
}

#[tauri::command]
pub fn verify_rate_limit_backoff_policy_gate(
    request: RateLimitBackoffPolicyRequest,
) -> RateLimitBackoffPolicyResult {
    verify_rate_limit_backoff_policy(&request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::execution::{RestoreExecutionStatus, RESTORE_CONFIRMATION_PHRASE};
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::schema_plan::{
        RestoreSchemaPlanStatus, SchemaPlanFieldInput, SchemaPlanTableInput,
    };

    // ── schema plan command tests ──────────────────────────────────────────

    fn schema_plan_request(dry_run_status: &str) -> RestoreSchemaPlanRequest {
        RestoreSchemaPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: dry_run_status.to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables: vec![SchemaPlanTableInput {
                table_id: "tbl01".to_string(),
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
                        field_name: "Status".to_string(),
                        field_type: "singleSelect".to_string(),
                        linked_table_id: None,
                    },
                ],
            }],
        }
    }

    #[test]
    fn schema_plan_command_does_not_require_token() {
        let req = schema_plan_request("ready");
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn schema_plan_command_ready_status_produces_plan() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(
            plan.status == RestoreSchemaPlanStatus::Ready
                || plan.status == RestoreSchemaPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn schema_plan_command_blocked_status_produces_blocked_plan() {
        let req = schema_plan_request("blocked");
        let plan = create_restore_schema_plan(req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Blocked);
        assert!(!plan.errors.is_empty());
    }

    #[test]
    fn schema_plan_result_no_changes_made_always_true() {
        for status in &["ready", "readyWithWarnings", "blocked"] {
            let req = schema_plan_request(status);
            let plan = create_restore_schema_plan(req);
            assert!(plan.no_changes_made);
        }
    }

    #[test]
    fn schema_plan_result_does_not_contain_absolute_path() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn schema_plan_result_has_no_token() {
        const SENTINEL: &str = "pat_command_test_sentinel_9999999999";
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn schema_plan_filename_is_not_a_path() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
        assert_eq!(plan.filename, "backup.airbridge");
    }

    #[test]
    fn schema_plan_result_has_table_steps() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(!plan.table_steps.is_empty());
        assert_eq!(plan.table_steps[0].table_name, "Projects");
    }

    #[test]
    fn schema_plan_result_has_field_steps() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        assert!(!plan.field_steps.is_empty());
    }

    #[test]
    fn schema_plan_ready_with_warnings_dry_run_status_accepted() {
        let req = schema_plan_request("readyWithWarnings");
        let plan = create_restore_schema_plan(req);
        assert!(
            plan.status == RestoreSchemaPlanStatus::Ready
                || plan.status == RestoreSchemaPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn schema_plan_empty_dry_run_status_produces_blocked() {
        let req = schema_plan_request("");
        let plan = create_restore_schema_plan(req);
        assert_eq!(plan.status, RestoreSchemaPlanStatus::Blocked);
    }

    #[test]
    fn schema_plan_serializes_with_no_changes_made_key() {
        let req = schema_plan_request("ready");
        let plan = create_restore_schema_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
    }

    #[test]
    fn schema_plan_has_no_succeeded_status() {
        for status in &["ready", "blocked"] {
            let req = schema_plan_request(status);
            let plan = create_restore_schema_plan(req);
            let json = serde_json::to_string(&plan).expect("serialize");
            assert!(!json.contains("succeeded"));
            assert!(!json.contains("Succeeded"));
        }
    }

    // ── dry-run command tests ──────────────────────────────────────────────

    #[test]
    fn command_does_not_require_token() {
        let req = RestoreDryRunRequest {
            path: "/nonexistent.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("token"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn nonexistent_path_returns_blocked_plan() {
        let req = RestoreDryRunRequest {
            path: "/tmp/does_not_exist_command_test.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let plan = create_restore_dry_run_plan(req);
        assert_eq!(
            plan.status,
            crate::restore::plan::RestorePlanStatus::Blocked
        );
        assert!(!plan.errors.is_empty());
    }

    #[test]
    fn blocked_plan_always_has_no_changes_made() {
        let req = RestoreDryRunRequest {
            path: "/tmp/does_not_exist_no_changes.airbridge".to_string(),
            target_mode: RestoreTargetMode::EmptyExistingBase,
            target_base_name: None,
        };
        let plan = create_restore_dry_run_plan(req);
        assert!(plan.no_changes_made);
    }

    #[test]
    fn result_does_not_contain_absolute_path() {
        let req = RestoreDryRunRequest {
            path: "/Users/testuser/backups/test.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let plan = create_restore_dry_run_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/testuser/"));
        assert!(!json.contains("/backups/"));
    }

    // ── execution command tests ────────────────────────────────────────────

    fn exec_request_missing_confirmation() -> RestoreExecutionRequest {
        RestoreExecutionRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            package_validation_status: "valid".to_string(),
            dry_run_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            token: "tok-test-value".to_string(),
            confirmation: "".to_string(),
        }
    }

    fn exec_request_valid() -> RestoreExecutionRequest {
        RestoreExecutionRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            package_validation_status: "valid".to_string(),
            dry_run_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            token: "tok-test-value".to_string(),
            confirmation: RESTORE_CONFIRMATION_PHRASE.to_string(),
        }
    }

    #[test]
    fn execution_command_missing_confirmation_returns_blocked() {
        let req = exec_request_missing_confirmation();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
        assert!(result.no_changes_made);
    }

    #[test]
    fn execution_command_valid_gate_returns_ready_but_disabled() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::ReadyButDisabled);
        assert!(result.no_changes_made);
    }

    #[test]
    fn execution_result_does_not_contain_token() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("tok-test-value"));
    }

    #[test]
    fn execution_result_does_not_contain_absolute_path() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn execution_result_has_no_succeeded_status() {
        let req = exec_request_valid();
        let result = run_restore_execution(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("succeeded"));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn execution_missing_token_returns_blocked() {
        let mut req = exec_request_valid();
        req.token = "".to_string();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
    }

    #[test]
    fn execution_invalid_package_returns_blocked() {
        let mut req = exec_request_valid();
        req.package_validation_status = "invalid".to_string();
        let result = run_restore_execution(req);
        assert_eq!(result.status, RestoreExecutionStatus::Blocked);
    }

    #[test]
    fn execution_no_changes_made_always_true() {
        // blocked case
        let req = exec_request_missing_confirmation();
        let result = run_restore_execution(req);
        assert!(result.no_changes_made);
        // ready-but-disabled case
        let req2 = exec_request_valid();
        let result2 = run_restore_execution(req2);
        assert!(result2.no_changes_made);
    }

    // ── record import plan command tests ───────────────────────────────────

    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
        RestoreRecordImportPlanStatus,
    };

    fn record_import_plan_request(
        dry_run_status: &str,
        schema_plan_status: &str,
    ) -> RestoreRecordImportPlanRequest {
        RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: dry_run_status.to_string(),
            schema_plan_status: schema_plan_status.to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Restored Base".to_string()),
            tables: vec![RecordImportTableInput {
                table_id: "tbl01".to_string(),
                table_name: "Projects".to_string(),
                record_count: Some(20),
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
            }],
        }
    }

    #[test]
    fn record_import_plan_command_does_not_require_token() {
        let req = record_import_plan_request("ready", "ready");
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn record_import_plan_command_ready_status_produces_plan() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(
            plan.status == RestoreRecordImportPlanStatus::Ready
                || plan.status == RestoreRecordImportPlanStatus::ReadyWithWarnings
        );
    }

    #[test]
    fn record_import_plan_command_blocked_dry_run_produces_blocked() {
        let req = record_import_plan_request("blocked", "ready");
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert!(!plan.errors.is_empty());
    }

    #[test]
    fn record_import_plan_command_blocked_schema_plan_produces_blocked() {
        let req = record_import_plan_request("ready", "blocked");
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert_eq!(plan.errors[0].code, "SCHEMA_PLAN_BLOCKED");
    }

    #[test]
    fn record_import_plan_command_no_changes_made_always_true() {
        for (dry, schema) in &[
            ("ready", "ready"),
            ("readyWithWarnings", "readyWithWarnings"),
            ("blocked", "ready"),
            ("ready", "blocked"),
        ] {
            let req = record_import_plan_request(dry, schema);
            let plan = create_restore_record_import_plan(req);
            assert!(
                plan.no_changes_made,
                "no_changes_made must be true for dry={dry}, schema={schema}"
            );
        }
    }

    #[test]
    fn record_import_plan_result_does_not_contain_absolute_path() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn record_import_plan_result_does_not_contain_token() {
        const SENTINEL: &str = "pat_command_test_import_sentinel_9999999999";
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn record_import_plan_filename_is_not_a_path() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
        assert_eq!(plan.filename, "backup.airbridge");
    }

    #[test]
    fn record_import_plan_result_has_table_plans() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(!plan.table_plans.is_empty());
        assert_eq!(plan.table_plans[0].table_name, "Projects");
    }

    #[test]
    fn record_import_plan_result_has_batch_size() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.table_plans[0].batch_size, 10);
    }

    #[test]
    fn record_import_plan_result_has_retry_policy() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        assert!(plan.retry_policy.max_retries_on_rate_limit > 0);
        assert!(plan.retry_policy.initial_backoff_ms > 0);
    }

    #[test]
    fn record_import_plan_ready_with_warnings_dry_run_status_accepted() {
        let req = record_import_plan_request("readyWithWarnings", "readyWithWarnings");
        let plan = create_restore_record_import_plan(req);
        assert_ne!(plan.status, RestoreRecordImportPlanStatus::Blocked);
    }

    #[test]
    fn record_import_plan_empty_tables_produces_blocked() {
        let req = RestoreRecordImportPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            tables: vec![],
        };
        let plan = create_restore_record_import_plan(req);
        assert_eq!(plan.status, RestoreRecordImportPlanStatus::Blocked);
        assert_eq!(plan.errors[0].code, "NO_TABLES");
    }

    #[test]
    fn record_import_plan_has_no_succeeded_status() {
        for (dry, schema) in &[("ready", "ready"), ("blocked", "ready")] {
            let req = record_import_plan_request(dry, schema);
            let plan = create_restore_record_import_plan(req);
            let json = serde_json::to_string(&plan).expect("serialize");
            assert!(!json.contains("succeeded"), "dry={dry}, schema={schema}");
        }
    }

    #[test]
    fn record_import_plan_serializes_no_changes_made_key() {
        let req = record_import_plan_request("ready", "ready");
        let plan = create_restore_record_import_plan(req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
    }

    // ── write engine command tests ─────────────────────────────────────────

    use crate::restore::write_result::RestoreWriteEngineStatus;

    fn write_engine_request() -> RestoreWriteEngineRequest {
        RestoreWriteEngineRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            schema_table_count: 2,
            schema_direct_field_count: 8,
            schema_deferred_field_count: 1,
            schema_manual_action_count: 0,
            schema_unsupported_count: 0,
            estimated_first_pass_batches: 3,
            estimated_second_pass_batches: 1,
            linked_record_update_count: 2,
        }
    }

    #[test]
    fn write_engine_command_returns_disabled() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn write_engine_command_no_changes_made_is_true() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn write_engine_command_does_not_require_token() {
        let req = write_engine_request();
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn write_engine_command_result_has_no_token() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn write_engine_command_result_has_no_absolute_path() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn write_engine_command_result_has_no_succeeded_status() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn write_engine_command_frontend_contract_is_safe() {
        let req = write_engine_request();
        let result = preview_restore_write_engine(req);
        // status must be disabled or blocked — never succeeded
        assert!(
            result.status == RestoreWriteEngineStatus::Disabled
                || result.status == RestoreWriteEngineStatus::Blocked
        );
        assert!(result.no_changes_made);
        assert!(!result.filename.contains('/'));
    }

    // ── schema write request plan command tests ────────────────────────────

    fn schema_write_request() -> SchemaWriteRequestPlanRequest {
        SchemaWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            schema_plan_status: "ready".to_string(),
            table_count: 3,
            direct_field_count: 6,
            deferred_field_count: 1,
            manual_action_count: 1,
        }
    }

    #[test]
    fn schema_write_command_returns_disabled() {
        use crate::restore::schema_write_requests::SchemaWriteOperationStatus;
        let req = schema_write_request();
        let result = preview_schema_write_request_plan(req);
        assert_eq!(result.status, SchemaWriteOperationStatus::Disabled);
    }

    #[test]
    fn schema_write_command_no_changes_made_true() {
        let req = schema_write_request();
        let result = preview_schema_write_request_plan(req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn schema_write_command_network_writes_attempted_false() {
        let req = schema_write_request();
        let result = preview_schema_write_request_plan(req);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn schema_write_command_no_token_in_result() {
        let req = schema_write_request();
        let result = preview_schema_write_request_plan(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn schema_write_command_no_succeeded_status() {
        let req = schema_write_request();
        let result = preview_schema_write_request_plan(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn schema_write_command_filename_basename_only() {
        let req = schema_write_request();
        let result = preview_schema_write_request_plan(req);
        assert_eq!(result.filename, "backup.airbridge");
        assert!(!result.filename.contains('/'));
    }

    #[test]
    fn schema_write_command_blocked_when_schema_blocked() {
        use crate::restore::schema_write_requests::SchemaWriteOperationStatus;
        let req = SchemaWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            schema_plan_status: "blocked".to_string(),
            table_count: 2,
            direct_field_count: 4,
            deferred_field_count: 0,
            manual_action_count: 0,
        };
        let result = preview_schema_write_request_plan(req);
        assert_eq!(result.status, SchemaWriteOperationStatus::Blocked);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn schema_write_command_blocked_when_no_tables() {
        use crate::restore::schema_write_requests::{
            SchemaWriteBlockedReason, SchemaWriteOperationStatus,
        };
        let req = SchemaWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            schema_plan_status: "ready".to_string(),
            table_count: 0,
            direct_field_count: 0,
            deferred_field_count: 0,
            manual_action_count: 0,
        };
        let result = preview_schema_write_request_plan(req);
        assert_eq!(result.status, SchemaWriteOperationStatus::Blocked);
        assert_eq!(
            result.blocked_reason,
            Some(SchemaWriteBlockedReason::NoTablesInPlan)
        );
        assert!(result.no_changes_made);
    }

    #[test]
    fn schema_write_command_op_counts_are_non_zero_for_valid_request() {
        let req = schema_write_request();
        let result = preview_schema_write_request_plan(req);
        assert!(result.table_op_count > 0, "must have table ops");
        assert!(result.total_op_count >= result.table_op_count);
    }

    #[test]
    fn schema_write_command_does_not_affect_restore_write_gate() {
        // Running the schema write command must leave the write gate unchanged.
        let gate_before = evaluate_write_gate();
        let req = schema_write_request();
        let _result = preview_schema_write_request_plan(req);
        let gate_after = evaluate_write_gate();
        assert!(matches!(
            gate_before.status,
            RestoreWriteEngineStatus::Disabled
        ));
        assert!(matches!(
            gate_after.status,
            RestoreWriteEngineStatus::Disabled
        ));
    }

    // ── record write request plan command tests ────────────────────────────

    use crate::restore::record_write_result::RecordWriteRequestPlanRequest;

    fn record_write_request() -> RecordWriteRequestPlanRequest {
        RecordWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            record_import_plan_status: "ready".to_string(),
            table_count: 2,
            total_first_pass_batches: 4,
            total_second_pass_batches: 2,
            attachment_field_count: 1,
            skipped_field_count: 2,
        }
    }

    #[test]
    fn record_write_command_returns_disabled() {
        use crate::restore::record_write_requests::RecordWriteOperationStatus;
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        assert_eq!(result.status, RecordWriteOperationStatus::Disabled);
    }

    #[test]
    fn record_write_command_no_changes_made_true() {
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn record_write_command_network_writes_attempted_false() {
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn record_write_command_no_token_in_result() {
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn record_write_command_no_succeeded_status() {
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn record_write_command_filename_basename_only() {
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        assert_eq!(result.filename, "backup.airbridge");
        assert!(!result.filename.contains('/'));
    }

    #[test]
    fn record_write_command_blocked_when_import_plan_blocked() {
        use crate::restore::record_write_requests::RecordWriteOperationStatus;
        let req = RecordWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            record_import_plan_status: "blocked".to_string(),
            table_count: 2,
            total_first_pass_batches: 4,
            total_second_pass_batches: 0,
            attachment_field_count: 0,
            skipped_field_count: 0,
        };
        let result = preview_record_write_request_plan(req);
        assert_eq!(result.status, RecordWriteOperationStatus::Blocked);
        assert!(result.no_changes_made);
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn record_write_command_blocked_when_no_tables() {
        use crate::restore::record_write_requests::{
            RecordWriteBlockedReason, RecordWriteOperationStatus,
        };
        let req = RecordWriteRequestPlanRequest {
            package_filename: "backup.airbridge".to_string(),
            record_import_plan_status: "ready".to_string(),
            table_count: 0,
            total_first_pass_batches: 0,
            total_second_pass_batches: 0,
            attachment_field_count: 0,
            skipped_field_count: 0,
        };
        let result = preview_record_write_request_plan(req);
        assert_eq!(result.status, RecordWriteOperationStatus::Blocked);
        assert_eq!(
            result.blocked_reason,
            Some(RecordWriteBlockedReason::NoTablesInPlan)
        );
        assert!(result.no_changes_made);
    }

    #[test]
    fn record_write_command_op_counts_are_non_zero_for_valid_request() {
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        assert!(result.total_op_count > 0, "must have ops");
        assert!(
            result.create_batch_op_count > 0,
            "must have create batch ops"
        );
    }

    #[test]
    fn record_write_command_no_absolute_path_in_result() {
        let req = record_write_request();
        let result = preview_record_write_request_plan(req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn record_write_command_does_not_affect_restore_write_gate() {
        let gate_before = evaluate_write_gate();
        let req = record_write_request();
        let _result = preview_record_write_request_plan(req);
        let gate_after = evaluate_write_gate();
        assert!(matches!(
            gate_before.status,
            RestoreWriteEngineStatus::Disabled
        ));
        assert!(matches!(
            gate_after.status,
            RestoreWriteEngineStatus::Disabled
        ));
    }
}
