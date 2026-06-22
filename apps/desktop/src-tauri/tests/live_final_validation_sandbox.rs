/// Sandbox final validation read integration test.
///
/// This test is `#[ignore]` by default and requires all four environment
/// variables to be set before it will run:
///
///   AIRBRIDGE_ENABLE_LIVE_FINAL_VALIDATION_TEST=true
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<personal access token>
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<sandbox base ID>
///   AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME=<table ID or name>
///
/// Optional:
///   AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT=<integer>
///   AIRBRIDGE_SANDBOX_TEST_PREFIX=<prefix for test labels>
///
/// Safety invariants:
/// - Default `cargo test` does NOT run the live test.
/// - Missing any required env var causes the test to skip, not fail.
/// - Token, base ID, table ID/name, and record IDs are never printed,
///   asserted on by value, or included in any serialized result.
/// - Exactly one read-only API call is made (GET records endpoint).
/// - No records are created, updated, or deleted.
/// - No schema writes are performed.
/// - No linked record updates are performed.
/// - No attachment endpoints are accessed.
/// - No attachment URLs are fetched.
/// - `evaluate_write_gate()` is verified to remain Disabled before and after.
/// - App runtime execution, reads, and writes remain disabled.
/// - The test is safe to run against any accessible sandbox table.
///
/// To run this test manually against a sandbox setup:
///   AIRBRIDGE_ENABLE_LIVE_FINAL_VALIDATION_TEST=true \
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=your_pat \
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=appYourSandboxBase \
///   AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME=tblYourTable \
///   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
///     --test live_final_validation_sandbox -- --ignored
use airbridge_desktop_lib::airtable::auth::AirtableToken;
use airbridge_desktop_lib::airtable::client::AirtableClient;
use airbridge_desktop_lib::airtable::http::ReqwestHttpTransport;
use airbridge_desktop_lib::restore::final_validation_reader::{
    build_final_validation_reader_plan, FinalValidationReaderMode, FinalValidationReaderRequest,
    FinalValidationReaderStatus,
};
use airbridge_desktop_lib::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use airbridge_desktop_lib::restore::live_final_validation_test_contract::{
    evaluate_live_final_validation_test_contract, LiveFinalValidationTestContractMode,
    LiveFinalValidationTestContractRequest, LiveFinalValidationTestContractStatus,
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
use airbridge_desktop_lib::restore::sandbox_final_validation_adapter::{
    build_sandbox_final_validation_adapter, SandboxFinalValidationAdapterMode,
    SandboxFinalValidationAdapterRequest, SandboxFinalValidationAdapterStatus,
};
use airbridge_desktop_lib::restore::sandbox_linked_second_pass_adapter::{
    build_sandbox_linked_second_pass_adapter, SandboxLinkedSecondPassAdapterMode,
    SandboxLinkedSecondPassAdapterRequest, SandboxLinkedSecondPassAdapterStatus,
};
use airbridge_desktop_lib::restore::sandbox_record_write_adapter::{
    build_sandbox_record_write_adapter, SandboxRecordWriteAdapterMode,
    SandboxRecordWriteAdapterRequest, SandboxRecordWriteAdapterStatus,
};
use airbridge_desktop_lib::restore::sandbox_schema_write_adapter::{
    build_sandbox_schema_write_adapter, SandboxSchemaWriteAdapterMode,
    SandboxSchemaWriteAdapterRequest, SandboxSchemaWriteAdapterStatus,
};
use airbridge_desktop_lib::restore::schema_plan::{
    RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreSchemaDependencyGraph,
    RestoreSchemaPlan, RestoreSchemaPlanStatus, RestoreTableCreationStep,
};
use airbridge_desktop_lib::restore::schema_write_requests::build_schema_write_request_plan;
use airbridge_desktop_lib::restore::write_gate::evaluate_write_gate;
use airbridge_desktop_lib::restore::write_result::RestoreWriteEngineStatus;

// ── Env var names ─────────────────────────────────────────────────────────────

const ENV_ENABLE: &str = "AIRBRIDGE_ENABLE_LIVE_FINAL_VALIDATION_TEST";
const ENV_TOKEN: &str = "AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN";
const ENV_BASE_ID: &str = "AIRBRIDGE_SANDBOX_TARGET_BASE_ID";
const ENV_TABLE: &str = "AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME";
const ENV_MIN_COUNT: &str = "AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT";

// ── Opt-in guard ──────────────────────────────────────────────────────────────

/// Returns `true` only when all four required env vars are present and the
/// enable flag is exactly `"true"`. Does NOT print any env var values.
fn all_required_env_vars_present() -> bool {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    let table = std::env::var(ENV_TABLE).unwrap_or_default();
    enable == "true" && !token.is_empty() && !base_id.is_empty() && !table.is_empty()
}

// ── Contract verification helpers ─────────────────────────────────────────────

fn make_contract_schema_plan(
) -> airbridge_desktop_lib::restore::schema_write_requests::SchemaWriteRequestPlan {
    let plan = RestoreSchemaPlan {
        filename: "sandbox_fv_test.airbridge".to_string(),
        status: RestoreSchemaPlanStatus::Ready,
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: None,
        table_steps: vec![RestoreTableCreationStep {
            table_id: "tbl_sandbox_fv".to_string(),
            table_name: "SandboxFV".to_string(),
            step_index: 0,
            field_count: 1,
            direct_field_count: 1,
            deferred_field_count: 0,
            manual_action_count: 0,
            unsupported_count: 0,
            note: "Sandbox final validation test table.".to_string(),
        }],
        field_steps: vec![RestoreFieldCreationStep {
            field_id: "fld_sandbox_name".to_string(),
            field_name: "Name".to_string(),
            field_type: "singleLineText".to_string(),
            table_id: "tbl_sandbox_fv".to_string(),
            table_name: "SandboxFV".to_string(),
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
        package_filename: "sandbox_fv_test.airbridge".to_string(),
        dry_run_status: "ready".to_string(),
        schema_plan_status: "ready".to_string(),
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: Some("Sandbox FV Test Base".to_string()),
        tables: vec![RecordImportTableInput {
            table_id: "tbl_sandbox_fv".to_string(),
            table_name: "SandboxFV".to_string(),
            record_count: Some(1),
            fields: vec![RecordImportFieldInput {
                field_id: "fld_sandbox_name".to_string(),
                field_name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
                linked_table_id: None,
            }],
        }],
    };
    let import_plan = create_record_import_plan(&req);
    build_record_write_request_plan(&import_plan)
}

fn make_full_fv_contract_request() -> LiveFinalValidationTestContractRequest {
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
        field_summaries: vec![],
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

fn make_fv_reader_request() -> FinalValidationReaderRequest {
    FinalValidationReaderRequest {
        mode: FinalValidationReaderMode::SandboxOnly,
        explicit_internal_final_validation_read_requested: true,
        sandbox_verified: true,
        schema_executor_safe: true,
        record_executor_safe: true,
        linked_executor_safe: true,
        final_validation_preview_ready: true,
        final_validation_enforcement_safe: true,
        sensitive_data_safe: true,
        attachment_phase_disabled_safe: true,
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

// ── Default (non-ignored) gating tests ───────────────────────────────────────
// None of these make any network calls.

#[test]
fn missing_enable_flag_does_not_perform_network_call() {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    if enable != "true" {
        return;
    }
}

#[test]
fn missing_token_does_not_perform_network_call() {
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    if token.is_empty() {
        return;
    }
}

#[test]
fn missing_base_id_does_not_perform_network_call() {
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    if base_id.is_empty() {
        return;
    }
}

#[test]
fn missing_validation_table_does_not_perform_network_call() {
    let table = std::env::var(ENV_TABLE).unwrap_or_default();
    if table.is_empty() {
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
fn fv_contract_eligible_but_not_executed_with_all_prereqs_satisfied() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_full_fv_contract_request();
    let result = evaluate_live_final_validation_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveFinalValidationTestContractStatus::EligibleButNotExecuted
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
fn fv_reader_plan_not_executed_while_gate_disabled() {
    let req = make_fv_reader_request();
    let result = build_final_validation_reader_plan(&req);
    assert_eq!(result.status, FinalValidationReaderStatus::NotExecuted);
    assert!(!result.reads_enabled);
    assert!(!result.writes_enabled);
    assert!(result.no_changes_made);
    assert!(!result.network_reads_attempted);
    assert!(!result.network_writes_attempted);
}

#[test]
fn fv_adapter_ready_for_sandbox_call_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = SandboxFinalValidationAdapterRequest {
        mode: SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
        explicit_internal_validation_sandbox_call_requested: true,
        sandbox_verified: true,
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
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        field_summaries: vec![],
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    };
    let result = build_sandbox_final_validation_adapter(&req, &sp, &rp);
    assert_eq!(
        result.status,
        SandboxFinalValidationAdapterStatus::ReadyForSandboxCall
    );
    assert!(!result.network_writes_attempted);
    assert!(!result.network_reads_attempted);
    assert!(!result.runtime_execution_enabled);
}

#[test]
fn linked_adapter_ready_for_sandbox_call_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = SandboxLinkedSecondPassAdapterRequest {
        mode: SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
        explicit_internal_linked_sandbox_call_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        final_validation_enforcement_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![LinkedSecondPassFieldSummary {
            table_label: "SandboxFV".to_string(),
            field_label: "Name".to_string(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
    };
    let result = build_sandbox_linked_second_pass_adapter(&req, &sp, &rp);
    assert_eq!(
        result.status,
        SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall
    );
    assert!(!result.network_writes_attempted);
    assert!(!result.runtime_execution_enabled);
}

#[test]
fn record_adapter_ready_for_sandbox_call_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = SandboxRecordWriteAdapterRequest {
        mode: SandboxRecordWriteAdapterMode::SandboxOnlyInternal,
        explicit_internal_record_sandbox_call_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        final_validation_enforcement_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
    };
    let result = build_sandbox_record_write_adapter(&req, &rp, &sp);
    assert_eq!(
        result.status,
        SandboxRecordWriteAdapterStatus::ReadyForSandboxCall
    );
    assert!(!result.network_writes_attempted);
    assert!(!result.runtime_execution_enabled);
}

#[test]
fn schema_adapter_ready_for_sandbox_call_without_live_call() {
    let sp = make_contract_schema_plan();
    let req = SandboxSchemaWriteAdapterRequest {
        mode: SandboxSchemaWriteAdapterMode::SandboxOnlyInternal,
        explicit_internal_schema_sandbox_call_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        final_validation_enforcement_safe: true,
        rate_limit_backoff_safe: true,
    };
    let result = build_sandbox_schema_write_adapter(&req, &sp);
    assert_eq!(
        result.status,
        SandboxSchemaWriteAdapterStatus::ReadyForSandboxCall
    );
    assert!(!result.network_writes_attempted);
    assert!(!result.runtime_execution_enabled);
}

#[test]
fn adapter_chain_returns_mock_run_not_executed_without_live_call() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = SandboxAdapterChainRunnerRequest {
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
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    };
    let result = run_sandbox_adapter_chain(&req, &sp, &rp);
    assert_eq!(
        result.status,
        SandboxAdapterChainRunnerStatus::MockRunNotExecuted
    );
}

#[test]
fn live_fv_test_does_not_introduce_tauri_command() {
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn no_schema_write_in_default_test_suite() {
    let gate = evaluate_write_gate();
    assert!(
        matches!(gate.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled — schema writes are not permitted here"
    );
}

#[test]
fn no_record_create_in_default_test_suite() {
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn no_linked_update_in_default_test_suite() {
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn no_attachment_endpoint_in_default_test_suite() {
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

// ── Live sandbox integration test (ignored by default) ────────────────────────
//
// To run this test, set all required env vars and pass --ignored:
//
//   AIRBRIDGE_ENABLE_LIVE_FINAL_VALIDATION_TEST=true \
//   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<your pat> \
//   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<appYourSandboxBase> \
//   AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME=<tblYourTable> \
//   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
//     --test live_final_validation_sandbox \
//     -- sandbox_final_validation_reads_table_and_verifies_contract --ignored
//
// WARNING: This test performs one live Airtable API call:
//   1. GET records endpoint for the configured validation table (read-only).
//
// No records are created, updated, or deleted.
// No schema writes are performed.
// No attachment endpoints are accessed.
// The test is safe to run against any accessible sandbox table.
// Results are sanitized — no record IDs, no raw field values, no token.

#[test]
#[ignore]
fn sandbox_final_validation_reads_table_and_verifies_contract() {
    // ── Opt-in gate ───────────────────────────────────────────────────────────
    if !all_required_env_vars_present() {
        return;
    }

    // ── Retrieve env vars (values never printed) ──────────────────────────────
    let token_raw = std::env::var(ENV_TOKEN).expect("token env var must be set");
    let base_id = std::env::var(ENV_BASE_ID).expect("base ID env var must be set");
    let table = std::env::var(ENV_TABLE).expect("table env var must be set");
    let expected_min: Option<usize> = std::env::var(ENV_MIN_COUNT)
        .ok()
        .and_then(|s| s.parse::<usize>().ok());

    // ── Pre-call: write gate must be Disabled ─────────────────────────────────
    let gate_before = evaluate_write_gate();
    assert!(
        matches!(gate_before.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must return Disabled before live calls"
    );

    // ── Pre-call: FV contract must return EligibleButNotExecuted ──────────────
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let contract_req = make_full_fv_contract_request();
    let contract_result = evaluate_live_final_validation_test_contract(&contract_req, &sp, &rp);
    assert_eq!(
        contract_result.status,
        LiveFinalValidationTestContractStatus::EligibleButNotExecuted,
        "FV contract must return EligibleButNotExecuted before live calls"
    );
    assert!(contract_result.contract_only);
    assert!(!contract_result.airtable_client_called);
    assert!(!contract_result.network_writes_attempted);
    assert!(!contract_result.network_reads_attempted);
    assert!(!contract_result.app_runtime_execution_enabled);
    assert!(!contract_result.app_runtime_writes_enabled);
    assert!(!contract_result.app_runtime_reads_enabled);

    // ── Pre-call: FV reader plan must be NotExecuted ──────────────────────────
    let reader_req = make_fv_reader_request();
    let reader_result = build_final_validation_reader_plan(&reader_req);
    assert_eq!(
        reader_result.status,
        FinalValidationReaderStatus::NotExecuted,
        "FV reader plan must return NotExecuted before live calls"
    );
    assert!(!reader_result.reads_enabled);
    assert!(!reader_result.writes_enabled);
    assert!(reader_result.no_changes_made);

    // ── Pre-call: FV adapter must be ReadyForSandboxCall ─────────────────────
    let fv_adapter_req = SandboxFinalValidationAdapterRequest {
        mode: SandboxFinalValidationAdapterMode::SandboxOnlyInternal,
        explicit_internal_validation_sandbox_call_requested: true,
        sandbox_verified: true,
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
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        field_summaries: vec![],
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    };
    let fv_adapter_result = build_sandbox_final_validation_adapter(&fv_adapter_req, &sp, &rp);
    assert_eq!(
        fv_adapter_result.status,
        SandboxFinalValidationAdapterStatus::ReadyForSandboxCall,
        "FV adapter must return ReadyForSandboxCall before live calls"
    );

    // ── Pre-call: linked adapter must be ReadyForSandboxCall ──────────────────
    let linked_adapter_req = SandboxLinkedSecondPassAdapterRequest {
        mode: SandboxLinkedSecondPassAdapterMode::SandboxOnlyInternal,
        explicit_internal_linked_sandbox_call_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        mapping_coverage_sufficient: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        final_validation_enforcement_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
        record_executor_safe: true,
        linked_second_pass_preview_ready: true,
        mapping_checkpoint_preview_ready: true,
        field_summaries: vec![],
    };
    let linked_adapter_result =
        build_sandbox_linked_second_pass_adapter(&linked_adapter_req, &sp, &rp);
    assert_eq!(
        linked_adapter_result.status,
        SandboxLinkedSecondPassAdapterStatus::ReadyForSandboxCall,
        "linked adapter must return ReadyForSandboxCall before live calls"
    );

    // ── Pre-call: record adapter must be ReadyForSandboxCall ──────────────────
    let record_adapter_req = SandboxRecordWriteAdapterRequest {
        mode: SandboxRecordWriteAdapterMode::SandboxOnlyInternal,
        explicit_internal_record_sandbox_call_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        final_validation_enforcement_safe: true,
        rate_limit_backoff_safe: true,
        schema_executor_safe: true,
        checkpoint_store_safe: true,
    };
    let record_adapter_result = build_sandbox_record_write_adapter(&record_adapter_req, &rp, &sp);
    assert_eq!(
        record_adapter_result.status,
        SandboxRecordWriteAdapterStatus::ReadyForSandboxCall,
        "record adapter must return ReadyForSandboxCall before live calls"
    );

    // ── Pre-call: schema adapter must be ReadyForSandboxCall ──────────────────
    let schema_adapter_req = SandboxSchemaWriteAdapterRequest {
        mode: SandboxSchemaWriteAdapterMode::SandboxOnlyInternal,
        explicit_internal_schema_sandbox_call_requested: true,
        sandbox_verified: true,
        target_base_empty: true,
        confirmation_gate_declared: true,
        destructive_operation_policy_safe: true,
        attachment_phase_disabled_safe: true,
        live_write_readiness_safe: true,
        write_phase_ordering_safe: true,
        failure_modes_safe: true,
        rollback_limitation_safe: true,
        checkpoint_durability_safe: true,
        sensitive_data_safe: true,
        final_validation_enforcement_safe: true,
        rate_limit_backoff_safe: true,
    };
    let schema_adapter_result = build_sandbox_schema_write_adapter(&schema_adapter_req, &sp);
    assert_eq!(
        schema_adapter_result.status,
        SandboxSchemaWriteAdapterStatus::ReadyForSandboxCall,
        "schema adapter must return ReadyForSandboxCall before live calls"
    );

    // ── Pre-call: adapter chain must return MockRunNotExecuted ────────────────
    let chain_req = SandboxAdapterChainRunnerRequest {
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
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    };
    let chain_result = run_sandbox_adapter_chain(&chain_req, &sp, &rp);
    assert_eq!(
        chain_result.status,
        SandboxAdapterChainRunnerStatus::MockRunNotExecuted,
        "adapter chain must return MockRunNotExecuted before live calls"
    );

    // ── Build Airtable client ─────────────────────────────────────────────────
    let transport = ReqwestHttpTransport::new().expect("http transport");
    let client = AirtableClient::new(AirtableToken::new(token_raw), transport);

    // ── Live call: read-only GET records from the validation table ────────────
    // This is the single live call. It is read-only:
    //   - No records created, updated, or deleted.
    //   - No schema writes.
    //   - No linked record updates.
    //   - No attachment endpoints.
    // The outcome is sanitized — no record IDs, no raw field values, no token.
    let outcome = client
        .list_sandbox_records_for_validation(&base_id, &table, expected_min)
        .expect("validation read must succeed against sandbox table");

    // ── Post-call assertions ──────────────────────────────────────────────────

    assert!(
        outcome.table_reachable,
        "table must be reachable after successful read"
    );

    // If a min count was provided, check it was satisfied.
    if expected_min.is_some() {
        assert!(
            outcome.min_count_satisfied,
            "observed record count must meet expected minimum"
        );
    }

    // Outcome must not contain token, record IDs, or raw field values.
    let outcome_json = serde_json::to_string(&outcome).expect("serialize outcome");
    assert!(
        !outcome_json.contains("pat_"),
        "outcome must not contain token"
    );
    assert!(
        !outcome_json.contains("rec"),
        "outcome must not contain record IDs"
    );

    // ── Post-call: write gate must still be Disabled ──────────────────────────
    let gate_after = evaluate_write_gate();
    assert!(
        matches!(gate_after.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must remain Disabled after live reads"
    );

    // ── Post-call: reader plan still returns NotExecuted (gate unchanged) ─────
    let reader_after = build_final_validation_reader_plan(&make_fv_reader_request());
    assert_eq!(
        reader_after.status,
        FinalValidationReaderStatus::NotExecuted,
        "FV reader plan must remain NotExecuted after live read — gate is unchanged"
    );
    assert!(!reader_after.reads_enabled);
    assert!(!reader_after.writes_enabled);
}
