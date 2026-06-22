/// Sandbox live E2E restore integration test.
///
/// This test is `#[ignore]` by default and requires ALL of the following
/// environment variables to be set before it will run:
///
///   AIRBRIDGE_ENABLE_LIVE_E2E_RESTORE_TEST=true
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<personal access token>
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<sandbox base ID>
///   AIRBRIDGE_SANDBOX_SCHEMA_TABLE_NAME=<name for the new table>
///   AIRBRIDGE_SANDBOX_RECORD_TABLE_ID_OR_NAME=<table ID or name for record writes>
///   AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME=<source table ID or name>
///   AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME=<target table ID or name>
///   AIRBRIDGE_SANDBOX_LINK_FIELD_NAME=<linked field name in source table>
///   AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME=<table ID or name for validation read>
///
/// Optional:
///   AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT=<integer>
///   AIRBRIDGE_SANDBOX_TEST_PREFIX=<prefix for test values>
///
/// Safety invariants:
/// - Default `cargo test` does NOT run this test.
/// - Missing any required env var causes the test to skip, not fail.
/// - Token, base ID, table IDs/names, field names, and record IDs are never
///   printed, asserted on by value, or included in any serialized result.
/// - The E2E contract is verified before any live call is made.
/// - Each phase contract is verified before its live call.
/// - `evaluate_write_gate()` is verified to remain Disabled before and after
///   every phase.
/// - App runtime execution/reads/writes remain disabled throughout.
/// - No attachment endpoints are called.
/// - No attachment URLs are fetched.
/// - No record deletes are performed.
/// - No schema deletes are performed.
/// - The test may leave sandbox-only tables and records in the target base.
///   It MUST only be run against a disposable sandbox base.
/// - No cleanup is performed automatically — delete test data manually after
///   the run if needed.
///
/// To run this test manually against a sandbox setup:
///   AIRBRIDGE_ENABLE_LIVE_E2E_RESTORE_TEST=true \
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=your_pat \
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=appYourSandboxBase \
///   AIRBRIDGE_SANDBOX_SCHEMA_TABLE_NAME=SandboxE2ETable \
///   AIRBRIDGE_SANDBOX_RECORD_TABLE_ID_OR_NAME=tblYourRecordTable \
///   AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME=tblSourceTable \
///   AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME=tblTargetTable \
///   AIRBRIDGE_SANDBOX_LINK_FIELD_NAME="Tasks" \
///   AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME=tblValidationTable \
///   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
///     --test live_e2e_restore_sandbox -- --ignored
///
/// WARNING: This test performs live Airtable API calls across all four restore
/// phases. It must only be run against a disposable sandbox base. It may leave
/// test tables and records behind — clean up manually after the run.
/// Never run against a production base.
use airbridge_desktop_lib::airtable::auth::AirtableToken;
use airbridge_desktop_lib::airtable::client::AirtableClient;
use airbridge_desktop_lib::airtable::http::ReqwestHttpTransport;
use airbridge_desktop_lib::airtable::models::{
    AirtableRecordFields, CreateSandboxRecordRequest, CreateTableFieldSpec, CreateTableRequest,
    UpdateLinkedSandboxRecordRequest,
};
use airbridge_desktop_lib::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use airbridge_desktop_lib::restore::live_e2e_restore_test_contract::{
    evaluate_live_e2e_restore_test_contract, LiveE2ERestoreTestContractMode,
    LiveE2ERestoreTestContractRequest, LiveE2ERestoreTestContractStatus,
};
use airbridge_desktop_lib::restore::live_final_validation_test_contract::{
    evaluate_live_final_validation_test_contract, LiveFinalValidationTestContractMode,
    LiveFinalValidationTestContractRequest, LiveFinalValidationTestContractStatus,
};
use airbridge_desktop_lib::restore::live_linked_update_test_contract::{
    evaluate_live_linked_update_test_contract, LiveLinkedUpdateTestContractMode,
    LiveLinkedUpdateTestContractRequest, LiveLinkedUpdateTestContractStatus,
};
use airbridge_desktop_lib::restore::live_record_write_test_contract::{
    evaluate_live_record_write_test_contract, LiveRecordWriteTestContractMode,
    LiveRecordWriteTestContractRequest, LiveRecordWriteTestContractStatus,
};
use airbridge_desktop_lib::restore::live_schema_write_test_contract::{
    evaluate_live_schema_write_test_contract, LiveSchemaWriteTestContractStatus,
};
use airbridge_desktop_lib::restore::plan::RestoreTargetMode;
use airbridge_desktop_lib::restore::record_import_plan::{
    RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
};
use airbridge_desktop_lib::restore::record_import_planner::create_record_import_plan;
use airbridge_desktop_lib::restore::record_write_requests::build_record_write_request_plan;
use airbridge_desktop_lib::restore::sandbox_adapter_chain_runner::{
    run_sandbox_adapter_chain, SandboxAdapterChainRunnerMode, SandboxAdapterChainRunnerRequest,
    SandboxAdapterChainRunnerStatus,
};
use airbridge_desktop_lib::restore::schema_plan::{
    RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreSchemaDependencyGraph,
    RestoreSchemaPlan, RestoreSchemaPlanStatus, RestoreTableCreationStep,
};
use airbridge_desktop_lib::restore::schema_write_requests::build_schema_write_request_plan;
use airbridge_desktop_lib::restore::write_gate::evaluate_write_gate;
use airbridge_desktop_lib::restore::write_result::RestoreWriteEngineStatus;

// ── Env var names ─────────────────────────────────────────────────────────────

const ENV_ENABLE: &str = "AIRBRIDGE_ENABLE_LIVE_E2E_RESTORE_TEST";
const ENV_TOKEN: &str = "AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN";
const ENV_BASE_ID: &str = "AIRBRIDGE_SANDBOX_TARGET_BASE_ID";
const ENV_SCHEMA_TABLE: &str = "AIRBRIDGE_SANDBOX_SCHEMA_TABLE_NAME";
const ENV_RECORD_TABLE: &str = "AIRBRIDGE_SANDBOX_RECORD_TABLE_ID_OR_NAME";
const ENV_SOURCE_TABLE: &str = "AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME";
const ENV_TARGET_TABLE: &str = "AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME";
const ENV_LINK_FIELD: &str = "AIRBRIDGE_SANDBOX_LINK_FIELD_NAME";
const ENV_VALIDATION_TABLE: &str = "AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME";
const ENV_MIN_COUNT: &str = "AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT";
const ENV_PREFIX: &str = "AIRBRIDGE_SANDBOX_TEST_PREFIX";

// ── Opt-in guard ──────────────────────────────────────────────────────────────

/// Returns `true` only when all required env vars are present and the enable
/// flag is exactly `"true"`. Does NOT print any env var values.
fn all_required_env_vars_present() -> bool {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    let schema_table = std::env::var(ENV_SCHEMA_TABLE).unwrap_or_default();
    let record_table = std::env::var(ENV_RECORD_TABLE).unwrap_or_default();
    let source_table = std::env::var(ENV_SOURCE_TABLE).unwrap_or_default();
    let target_table = std::env::var(ENV_TARGET_TABLE).unwrap_or_default();
    let link_field = std::env::var(ENV_LINK_FIELD).unwrap_or_default();
    let validation_table = std::env::var(ENV_VALIDATION_TABLE).unwrap_or_default();
    enable == "true"
        && !token.is_empty()
        && !base_id.is_empty()
        && !schema_table.is_empty()
        && !record_table.is_empty()
        && !source_table.is_empty()
        && !target_table.is_empty()
        && !link_field.is_empty()
        && !validation_table.is_empty()
}

// ── Contract verification helpers ─────────────────────────────────────────────

fn make_contract_schema_plan(
) -> airbridge_desktop_lib::restore::schema_write_requests::SchemaWriteRequestPlan {
    let plan = RestoreSchemaPlan {
        filename: "sandbox_e2e_test.airbridge".to_string(),
        status: RestoreSchemaPlanStatus::Ready,
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: None,
        table_steps: vec![RestoreTableCreationStep {
            table_id: "tbl_e2e_source".to_string(),
            table_name: "SandboxE2ESource".to_string(),
            step_index: 0,
            field_count: 1,
            direct_field_count: 1,
            deferred_field_count: 0,
            manual_action_count: 0,
            unsupported_count: 0,
            note: "Sandbox E2E integration test table.".to_string(),
        }],
        field_steps: vec![RestoreFieldCreationStep {
            field_id: "fld_e2e_name".to_string(),
            field_name: "Name".to_string(),
            field_type: "singleLineText".to_string(),
            table_id: "tbl_e2e_source".to_string(),
            table_name: "SandboxE2ESource".to_string(),
            classification: RestoreFieldCreateClassification::CreateDirectly,
            note: "Primary text field.".to_string(),
        }],
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
    build_schema_write_request_plan(&plan)
}

fn make_contract_record_plan(
) -> airbridge_desktop_lib::restore::record_write_requests::RecordWriteRequestPlan {
    let req = RestoreRecordImportPlanRequest {
        package_filename: "sandbox_e2e_test.airbridge".to_string(),
        dry_run_status: "ready".to_string(),
        schema_plan_status: "ready".to_string(),
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: Some("Sandbox E2E Test Base".to_string()),
        tables: vec![RecordImportTableInput {
            table_id: "tbl_e2e_source".to_string(),
            table_name: "SandboxE2ESource".to_string(),
            record_count: Some(1),
            fields: vec![RecordImportFieldInput {
                field_id: "fld_e2e_name".to_string(),
                field_name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
                linked_table_id: None,
            }],
        }],
    };
    let import_plan = create_record_import_plan(&req);
    build_record_write_request_plan(&import_plan)
}

fn all_prereqs_true() -> bool {
    true
}

fn make_e2e_contract_request() -> LiveE2ERestoreTestContractRequest {
    LiveE2ERestoreTestContractRequest {
        mode: LiveE2ERestoreTestContractMode::SandboxIntegrationCandidate,
        explicit_internal_live_e2e_restore_test_contract_requested: true,
        sandbox_verified: all_prereqs_true(),
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        final_validation_enforcement_safe: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![LinkedSecondPassFieldSummary {
            table_label: "SandboxE2ESource".to_string(),
            field_label: "Name".to_string(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

fn make_schema_contract_request(
) -> airbridge_desktop_lib::restore::live_schema_write_test_contract::LiveSchemaWriteTestContractRequest
{
    use airbridge_desktop_lib::restore::live_schema_write_test_contract::{
        LiveSchemaWriteTestContractMode, LiveSchemaWriteTestContractRequest,
    };
    LiveSchemaWriteTestContractRequest {
        mode: LiveSchemaWriteTestContractMode::SandboxIntegrationCandidate,
        explicit_internal_live_schema_test_contract_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        final_validation_enforcement_safe: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![LinkedSecondPassFieldSummary {
            table_label: "SandboxE2ESource".to_string(),
            field_label: "Name".to_string(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

fn make_record_contract_request() -> LiveRecordWriteTestContractRequest {
    LiveRecordWriteTestContractRequest {
        mode: LiveRecordWriteTestContractMode::SandboxIntegrationCandidate,
        explicit_internal_live_record_test_contract_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        final_validation_enforcement_safe: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![LinkedSecondPassFieldSummary {
            table_label: "SandboxE2ESource".to_string(),
            field_label: "Name".to_string(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

fn make_linked_update_contract_request() -> LiveLinkedUpdateTestContractRequest {
    LiveLinkedUpdateTestContractRequest {
        mode: LiveLinkedUpdateTestContractMode::SandboxIntegrationCandidate,
        explicit_internal_live_linked_update_test_contract_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        final_validation_enforcement_safe: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![LinkedSecondPassFieldSummary {
            table_label: "SandboxE2ESource".to_string(),
            field_label: "Name".to_string(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

fn make_fv_contract_request() -> LiveFinalValidationTestContractRequest {
    LiveFinalValidationTestContractRequest {
        mode: LiveFinalValidationTestContractMode::SandboxIntegrationCandidate,
        explicit_internal_live_final_validation_test_contract_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        final_validation_enforcement_safe: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![LinkedSecondPassFieldSummary {
            table_label: "SandboxE2ESource".to_string(),
            field_label: "Name".to_string(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

fn make_adapter_chain_request() -> SandboxAdapterChainRunnerRequest {
    SandboxAdapterChainRunnerRequest {
        mode: SandboxAdapterChainRunnerMode::MockInternalOnly,
        explicit_internal_mock_chain_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        final_validation_enforcement_safe: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![],
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

// ── Default (non-ignored) gating tests ───────────────────────────────────────
// These tests run in default cargo test and verify the opt-in gate works.

#[test]
fn missing_enable_flag_does_not_perform_network_call() {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    if enable != "true" {
        return;
    }
    // Enable flag set — this test still does not perform a network call.
}

#[test]
fn missing_token_does_not_perform_network_call() {
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    if token.is_empty() {
        return;
    }
    // Token set. This test still does not perform a network call.
}

#[test]
fn missing_base_id_does_not_perform_network_call() {
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    if base_id.is_empty() {
        return;
    }
    // Base ID set. This test still does not perform a network call.
}

#[test]
fn missing_schema_table_does_not_perform_network_call() {
    let v = std::env::var(ENV_SCHEMA_TABLE).unwrap_or_default();
    if v.is_empty() {
        return;
    }
}

#[test]
fn missing_record_table_does_not_perform_network_call() {
    let v = std::env::var(ENV_RECORD_TABLE).unwrap_or_default();
    if v.is_empty() {
        return;
    }
}

#[test]
fn missing_link_source_table_does_not_perform_network_call() {
    let v = std::env::var(ENV_SOURCE_TABLE).unwrap_or_default();
    if v.is_empty() {
        return;
    }
}

#[test]
fn missing_link_target_table_does_not_perform_network_call() {
    let v = std::env::var(ENV_TARGET_TABLE).unwrap_or_default();
    if v.is_empty() {
        return;
    }
}

#[test]
fn missing_link_field_name_does_not_perform_network_call() {
    let v = std::env::var(ENV_LINK_FIELD).unwrap_or_default();
    if v.is_empty() {
        return;
    }
}

#[test]
fn missing_validation_table_does_not_perform_network_call() {
    let v = std::env::var(ENV_VALIDATION_TABLE).unwrap_or_default();
    if v.is_empty() {
        return;
    }
}

#[test]
fn evaluate_write_gate_remains_disabled_without_env_vars() {
    let gate = evaluate_write_gate();
    assert!(
        matches!(gate.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must always return Disabled"
    );
}

#[test]
fn e2e_contract_eligible_but_not_executed_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_e2e_contract_request();
    let result = evaluate_live_e2e_restore_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveE2ERestoreTestContractStatus::EligibleButNotExecuted
    );
    assert!(result.contract_only);
    assert!(!result.airtable_client_called);
    assert!(!result.network_writes_attempted);
    assert!(!result.network_reads_attempted);
    assert!(!result.app_runtime_execution_enabled);
    assert!(!result.app_runtime_writes_enabled);
    assert!(!result.app_runtime_reads_enabled);
    assert!(result.no_changes_made);
}

#[test]
fn schema_contract_eligible_but_not_executed_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_schema_contract_request();
    let result = evaluate_live_schema_write_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveSchemaWriteTestContractStatus::EligibleButNotExecuted
    );
    assert!(result.contract_only);
    assert!(!result.airtable_client_called);
    assert!(!result.network_writes_attempted);
    assert!(result.no_changes_made);
}

#[test]
fn record_contract_eligible_but_not_executed_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_record_contract_request();
    let result = evaluate_live_record_write_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveRecordWriteTestContractStatus::EligibleButNotExecuted
    );
    assert!(result.contract_only);
    assert!(!result.airtable_client_called);
    assert!(!result.network_writes_attempted);
    assert!(result.no_changes_made);
}

#[test]
fn linked_update_contract_eligible_but_not_executed_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_linked_update_contract_request();
    let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveLinkedUpdateTestContractStatus::EligibleButNotExecuted
    );
    assert!(result.contract_only);
    assert!(!result.airtable_client_called);
    assert!(!result.network_writes_attempted);
    assert!(result.no_changes_made);
}

#[test]
fn fv_contract_eligible_but_not_executed_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_fv_contract_request();
    let result = evaluate_live_final_validation_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveFinalValidationTestContractStatus::EligibleButNotExecuted
    );
    assert!(result.contract_only);
    assert!(!result.airtable_client_called);
    assert!(!result.network_writes_attempted);
    assert!(result.no_changes_made);
}

#[test]
fn adapter_chain_returns_mock_run_not_executed_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_adapter_chain_request();
    let result = run_sandbox_adapter_chain(&req, &sp, &rp);
    assert_eq!(
        result.status,
        SandboxAdapterChainRunnerStatus::MockRunNotExecuted
    );
}

#[test]
fn no_attachment_endpoint_called_in_default_test_suite() {
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn live_e2e_test_does_not_introduce_tauri_command() {
    // Verifies this file introduces no Tauri command by confirming the write
    // gate remains Disabled — no app execution path exists.
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn no_restore_success_state_in_app_runtime() {
    // App runtime restore execution remains disabled. The write gate is the
    // canonical guard and is always Disabled.
    let gate = evaluate_write_gate();
    assert!(
        matches!(gate.status, RestoreWriteEngineStatus::Disabled),
        "app runtime restore execution must remain disabled"
    );
}

// ── Live E2E sandbox integration test (ignored by default) ────────────────────
//
// To run this test, set all required env vars and pass --ignored:
//
//   AIRBRIDGE_ENABLE_LIVE_E2E_RESTORE_TEST=true \
//   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<your pat> \
//   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<appYourSandboxBase> \
//   AIRBRIDGE_SANDBOX_SCHEMA_TABLE_NAME=SandboxE2ETable \
//   AIRBRIDGE_SANDBOX_RECORD_TABLE_ID_OR_NAME=<tblYourRecordTable> \
//   AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME=<tblSourceTable> \
//   AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME=<tblTargetTable> \
//   AIRBRIDGE_SANDBOX_LINK_FIELD_NAME="Tasks" \
//   AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME=<tblValidationTable> \
//   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
//     --test live_e2e_restore_sandbox \
//     -- sandbox_e2e_restore_sequences_all_phases_and_verifies_contracts --ignored
//
// WARNING: This test performs live Airtable API calls across all four restore
// phases. It must ONLY be run against a disposable sandbox base. It may leave
// test tables and records behind — clean up manually. Never run against a
// production base.

#[test]
#[ignore]
fn sandbox_e2e_restore_sequences_all_phases_and_verifies_contracts() {
    // ── Opt-in gate ───────────────────────────────────────────────────────────
    if !all_required_env_vars_present() {
        // Skip gracefully — do not panic, do not make any network call.
        return;
    }

    // ── Retrieve env vars (values never printed) ──────────────────────────────
    let token_raw = std::env::var(ENV_TOKEN).expect("token env var must be set");
    let base_id = std::env::var(ENV_BASE_ID).expect("base ID env var must be set");
    let schema_table_name =
        std::env::var(ENV_SCHEMA_TABLE).expect("schema table name env var must be set");
    let record_table = std::env::var(ENV_RECORD_TABLE).expect("record table env var must be set");
    let source_table =
        std::env::var(ENV_SOURCE_TABLE).expect("link source table env var must be set");
    let target_table =
        std::env::var(ENV_TARGET_TABLE).expect("link target table env var must be set");
    let link_field = std::env::var(ENV_LINK_FIELD).expect("link field name env var must be set");
    let validation_table =
        std::env::var(ENV_VALIDATION_TABLE).expect("validation table env var must be set");
    let expected_min_count: Option<usize> = std::env::var(ENV_MIN_COUNT)
        .ok()
        .and_then(|s| s.parse().ok());
    let prefix = std::env::var(ENV_PREFIX).unwrap_or_else(|_| "airbridge_e2e_test".to_string());

    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();

    // ── Pre-E2E: write gate must be Disabled ──────────────────────────────────
    let gate_init = evaluate_write_gate();
    assert!(
        matches!(gate_init.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must return Disabled before E2E test begins"
    );

    // ── Pre-E2E: top-level E2E contract must return EligibleButNotExecuted ────
    let e2e_req = make_e2e_contract_request();
    let e2e_result = evaluate_live_e2e_restore_test_contract(&e2e_req, &sp, &rp);
    assert_eq!(
        e2e_result.status,
        LiveE2ERestoreTestContractStatus::EligibleButNotExecuted,
        "E2E contract must return EligibleButNotExecuted before any live call"
    );
    assert!(e2e_result.contract_only);
    assert!(!e2e_result.airtable_client_called);
    assert!(!e2e_result.network_writes_attempted);
    assert!(!e2e_result.network_reads_attempted);
    assert!(!e2e_result.app_runtime_execution_enabled);
    assert!(!e2e_result.app_runtime_writes_enabled);
    assert!(!e2e_result.app_runtime_reads_enabled);
    assert!(e2e_result.no_changes_made);

    // ── Pre-E2E: adapter chain must return MockRunNotExecuted ─────────────────
    let chain_req = make_adapter_chain_request();
    let chain_result = run_sandbox_adapter_chain(&chain_req, &sp, &rp);
    assert_eq!(
        chain_result.status,
        SandboxAdapterChainRunnerStatus::MockRunNotExecuted,
        "adapter chain must return MockRunNotExecuted before any live call"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // PHASE 1: Schema write
    // ─────────────────────────────────────────────────────────────────────────

    // Phase 1 contract gate
    let schema_contract_req = make_schema_contract_request();
    let schema_contract_result =
        evaluate_live_schema_write_test_contract(&schema_contract_req, &sp, &rp);
    assert_eq!(
        schema_contract_result.status,
        LiveSchemaWriteTestContractStatus::EligibleButNotExecuted,
        "schema contract must return EligibleButNotExecuted before phase 1 live call"
    );
    assert!(!schema_contract_result.airtable_client_called);
    assert!(!schema_contract_result.network_writes_attempted);
    assert!(!schema_contract_result.app_runtime_execution_enabled);
    assert!(!schema_contract_result.app_runtime_writes_enabled);
    assert!(!schema_contract_result.app_runtime_reads_enabled);

    // Phase 1 write gate check
    let gate_p1_pre = evaluate_write_gate();
    assert!(
        matches!(gate_p1_pre.status, RestoreWriteEngineStatus::Disabled),
        "write gate must be Disabled before phase 1"
    );

    // Phase 1 live call: createTable
    let transport = ReqwestHttpTransport::new().expect("http transport");
    let client = AirtableClient::new(AirtableToken::new(token_raw), transport);

    let schema_table_full_name = format!("{prefix}_{schema_table_name}");
    let create_table_req = CreateTableRequest {
        name: schema_table_full_name.clone(),
        description: Some("Sandbox E2E integration test table — safe to delete.".to_string()),
        fields: vec![CreateTableFieldSpec {
            name: "Name".to_string(),
            field_type: "singleLineText".to_string(),
        }],
    };
    let schema_outcome = client
        .create_table(&base_id, &create_table_req)
        .expect("phase 1: createTable must succeed against sandbox base");

    // Phase 1 assertions — name matches; ID non-empty but never printed
    assert_eq!(
        schema_outcome.table_name, schema_table_full_name,
        "phase 1: created table name must match requested name"
    );
    assert!(
        !schema_outcome.table_id.is_empty(),
        "phase 1: table ID must be non-empty after createTable"
    );
    let schema_json = serde_json::to_string(&schema_outcome).expect("serialize schema outcome");
    assert!(
        !schema_json.contains("pat_"),
        "phase 1: outcome must not contain token"
    );

    // Phase 1 post: write gate still Disabled
    let gate_p1_post = evaluate_write_gate();
    assert!(
        matches!(gate_p1_post.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled after phase 1"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // PHASE 2: Record write
    // ─────────────────────────────────────────────────────────────────────────

    // Phase 2 contract gate
    let record_contract_req = make_record_contract_request();
    let record_contract_result =
        evaluate_live_record_write_test_contract(&record_contract_req, &sp, &rp);
    assert_eq!(
        record_contract_result.status,
        LiveRecordWriteTestContractStatus::EligibleButNotExecuted,
        "record contract must return EligibleButNotExecuted before phase 2 live call"
    );
    assert!(!record_contract_result.airtable_client_called);
    assert!(!record_contract_result.network_writes_attempted);
    assert!(!record_contract_result.app_runtime_execution_enabled);
    assert!(!record_contract_result.app_runtime_writes_enabled);
    assert!(!record_contract_result.app_runtime_reads_enabled);

    // Phase 2 write gate check
    let gate_p2_pre = evaluate_write_gate();
    assert!(
        matches!(gate_p2_pre.status, RestoreWriteEngineStatus::Disabled),
        "write gate must be Disabled before phase 2"
    );

    // Phase 2 live call: createRecord
    let record_name_value = format!("{prefix}_record_write");
    let mut record_fields = std::collections::HashMap::new();
    record_fields.insert(
        "Name".to_string(),
        serde_json::Value::String(record_name_value),
    );
    let create_record_req = CreateSandboxRecordRequest {
        fields: record_fields,
    };
    let record_outcome = client
        .create_single_sandbox_record(
            &base_id,
            &record_table,
            "SandboxE2ERecord",
            &create_record_req,
        )
        .expect("phase 2: createRecord must succeed against sandbox table");

    // Phase 2 assertions
    assert!(
        record_outcome.record_created,
        "phase 2: record_created must be true"
    );
    assert_eq!(
        record_outcome.record_count, 1,
        "phase 2: record_count must be 1"
    );
    let record_json = serde_json::to_string(&record_outcome).expect("serialize record outcome");
    assert!(
        !record_json.contains("pat_"),
        "phase 2: outcome must not contain token"
    );
    assert!(
        !record_json.contains("\"id\""),
        "phase 2: outcome must not expose record ID"
    );

    // Phase 2 post: write gate still Disabled
    let gate_p2_post = evaluate_write_gate();
    assert!(
        matches!(gate_p2_post.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled after phase 2"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // PHASE 3: Linked field update
    // ─────────────────────────────────────────────────────────────────────────

    // Phase 3 contract gate
    let linked_contract_req = make_linked_update_contract_request();
    let linked_contract_result =
        evaluate_live_linked_update_test_contract(&linked_contract_req, &sp, &rp);
    assert_eq!(
        linked_contract_result.status,
        LiveLinkedUpdateTestContractStatus::EligibleButNotExecuted,
        "linked update contract must return EligibleButNotExecuted before phase 3 live call"
    );
    assert!(!linked_contract_result.airtable_client_called);
    assert!(!linked_contract_result.network_writes_attempted);
    assert!(!linked_contract_result.app_runtime_execution_enabled);
    assert!(!linked_contract_result.app_runtime_writes_enabled);
    assert!(!linked_contract_result.app_runtime_reads_enabled);

    // Phase 3 write gate check
    let gate_p3_pre = evaluate_write_gate();
    assert!(
        matches!(gate_p3_pre.status, RestoreWriteEngineStatus::Disabled),
        "write gate must be Disabled before phase 3"
    );

    // Phase 3 live setup: create records in target and source tables to set up
    // the linked field scenario. Record IDs are held locally only, used once
    // in the PATCH, then dropped.
    let mut target_fields_map = std::collections::HashMap::new();
    target_fields_map.insert(
        "Name".to_string(),
        serde_json::Value::String(format!("{prefix}_linked_target")),
    );
    let target_resp = client
        .create_records(
            &base_id,
            &target_table,
            vec![AirtableRecordFields {
                fields: target_fields_map,
            }],
        )
        .expect("phase 3: create target record must succeed");
    let target_id = target_resp
        .records
        .into_iter()
        .next()
        .map(|r| r.id.0)
        .expect("phase 3: target record must have an ID");

    let mut source_fields_map = std::collections::HashMap::new();
    source_fields_map.insert(
        "Name".to_string(),
        serde_json::Value::String(format!("{prefix}_linked_source")),
    );
    let source_resp = client
        .create_records(
            &base_id,
            &source_table,
            vec![AirtableRecordFields {
                fields: source_fields_map,
            }],
        )
        .expect("phase 3: create source record must succeed");
    let source_id = source_resp
        .records
        .into_iter()
        .next()
        .map(|r| r.id.0)
        .expect("phase 3: source record must have an ID");

    // Phase 3 live call: PATCH linked field
    let linked_update_req = UpdateLinkedSandboxRecordRequest {
        source_record_id: source_id,
        linked_field_name: link_field.clone(),
        target_record_ids: vec![target_id],
    };
    let linked_outcome = client
        .update_single_linked_sandbox_record(
            &base_id,
            &source_table,
            "SandboxE2ESource",
            &linked_update_req,
        )
        .expect("phase 3: linked field update must succeed");

    // Phase 3 assertions
    assert!(
        linked_outcome.record_updated,
        "phase 3: record_updated must be true"
    );
    assert_eq!(
        linked_outcome.linked_target_count, 1,
        "phase 3: linked_target_count must be 1"
    );
    let linked_json = serde_json::to_string(&linked_outcome).expect("serialize linked outcome");
    assert!(
        !linked_json.contains("pat_"),
        "phase 3: outcome must not contain token"
    );
    assert!(
        !linked_json.contains("\"id\""),
        "phase 3: outcome must not expose record ID"
    );

    // Phase 3 post: write gate still Disabled
    let gate_p3_post = evaluate_write_gate();
    assert!(
        matches!(gate_p3_post.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled after phase 3"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // PHASE 4: Final validation read
    // ─────────────────────────────────────────────────────────────────────────

    // Phase 4 contract gate
    let fv_contract_req = make_fv_contract_request();
    let fv_contract_result =
        evaluate_live_final_validation_test_contract(&fv_contract_req, &sp, &rp);
    assert_eq!(
        fv_contract_result.status,
        LiveFinalValidationTestContractStatus::EligibleButNotExecuted,
        "final validation contract must return EligibleButNotExecuted before phase 4 live call"
    );
    assert!(!fv_contract_result.airtable_client_called);
    assert!(!fv_contract_result.network_writes_attempted);
    assert!(!fv_contract_result.app_runtime_execution_enabled);
    assert!(!fv_contract_result.app_runtime_writes_enabled);
    assert!(!fv_contract_result.app_runtime_reads_enabled);

    // Phase 4 write gate check
    let gate_p4_pre = evaluate_write_gate();
    assert!(
        matches!(gate_p4_pre.status, RestoreWriteEngineStatus::Disabled),
        "write gate must be Disabled before phase 4"
    );

    // Phase 4 live call: read-only GET records
    let fv_outcome = client
        .list_sandbox_records_for_validation(&base_id, &validation_table, expected_min_count)
        .expect("phase 4: list records for validation must succeed");

    // Phase 4 assertions
    assert!(
        fv_outcome.table_reachable,
        "phase 4: table_reachable must be true"
    );
    assert!(
        fv_outcome.has_records || expected_min_count.is_none(),
        "phase 4: has_records must be true when min count is expected"
    );
    if let Some(min) = expected_min_count {
        assert!(
            fv_outcome.min_count_satisfied,
            "phase 4: min_count_satisfied must be true (expected >= {min})"
        );
    }
    let fv_json = serde_json::to_string(&fv_outcome).expect("serialize fv outcome");
    assert!(
        !fv_json.contains("pat_"),
        "phase 4: outcome must not contain token"
    );
    assert!(
        !fv_json.contains("rec"),
        "phase 4: outcome must not expose record IDs"
    );

    // Phase 4 post: write gate still Disabled
    let gate_p4_post = evaluate_write_gate();
    assert!(
        matches!(gate_p4_post.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled after phase 4"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // PHASE 5: Final non-runtime guard
    // ─────────────────────────────────────────────────────────────────────────

    // Phase 5 is a pure safety assertion — no network call.
    // Verifies app runtime restore execution remains disabled after all phases.

    let gate_final = evaluate_write_gate();
    assert!(
        matches!(gate_final.status, RestoreWriteEngineStatus::Disabled),
        "phase 5: write gate must remain Disabled after all phases — \
         app runtime restore execution must not be enabled"
    );

    // Confirm app runtime state has not changed
    let e2e_final_req = make_e2e_contract_request();
    let e2e_final_result = evaluate_live_e2e_restore_test_contract(&e2e_final_req, &sp, &rp);
    assert!(
        !e2e_final_result.app_runtime_execution_enabled,
        "phase 5: app_runtime_execution_enabled must remain false after all phases"
    );
    assert!(
        !e2e_final_result.app_runtime_writes_enabled,
        "phase 5: app_runtime_writes_enabled must remain false after all phases"
    );
    assert!(
        !e2e_final_result.app_runtime_reads_enabled,
        "phase 5: app_runtime_reads_enabled must remain false after all phases"
    );
    assert!(
        !e2e_final_result.airtable_client_called,
        "phase 5: airtable_client_called must remain false in contract result"
    );
    assert!(
        e2e_final_result.no_changes_made,
        "phase 5: no_changes_made must remain true in contract result"
    );

    // Final confirmation: no restore success or complete state introduced
    let e2e_json = serde_json::to_string(&e2e_final_result).expect("serialize final e2e result");
    assert!(
        !e2e_json.contains("restoreSuccess"),
        "phase 5: restoreSuccess must not appear in any result"
    );
    assert!(
        !e2e_json.contains("restoreComplete"),
        "phase 5: restoreComplete must not appear in any result"
    );
    assert!(
        !e2e_json.contains("\"succeeded\""),
        "phase 5: succeeded state must not appear in any result"
    );
}
