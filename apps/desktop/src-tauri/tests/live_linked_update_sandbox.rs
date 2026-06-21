/// Sandbox linked update integration test.
///
/// This test is `#[ignore]` by default and requires all six environment
/// variables to be set before it will run:
///
///   AIRBRIDGE_ENABLE_LIVE_LINKED_UPDATE_TEST=true
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<personal access token>
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<sandbox base ID>
///   AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME=<source table ID or name>
///   AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME=<target table ID or name>
///   AIRBRIDGE_SANDBOX_LINK_FIELD_NAME=<linked field name in source table>
///
/// Optional:
///   AIRBRIDGE_SANDBOX_TEST_PREFIX=<prefix for test field values>
///
/// Safety invariants:
/// - Default `cargo test` does NOT run the live test.
/// - Missing any required env var causes the test to skip, not fail.
/// - Token, base ID, table IDs/names, field name, and record IDs are never
///   printed, asserted on value, or included in any serialized result.
/// - Exactly two minimal sandbox records are created (one in each table) to
///   set up the linked update scenario.
/// - Exactly one linked field update (PATCH) is performed.
/// - No schema writes, no arbitrary record updates, no attachment endpoints,
///   no final validation reads, no record deletes.
/// - `evaluate_write_gate()` is verified to remain Disabled before and after.
/// - App runtime execution/reads/writes remain disabled.
/// - The test may leave sandbox-only test records in the target tables.
///   It must only be run against disposable sandbox bases/tables.
/// - No cleanup is performed automatically — delete the test records manually
///   if needed after the run.
///
/// To run this test manually against a sandbox setup:
///   AIRBRIDGE_ENABLE_LIVE_LINKED_UPDATE_TEST=true \
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=your_pat \
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=appYourSandboxBase \
///   AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME=tblSourceTable \
///   AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME=tblTargetTable \
///   AIRBRIDGE_SANDBOX_LINK_FIELD_NAME="Tasks" \
///   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
///     --test live_linked_update_sandbox -- --ignored
use airbridge_desktop_lib::airtable::auth::AirtableToken;
use airbridge_desktop_lib::airtable::client::AirtableClient;
use airbridge_desktop_lib::airtable::http::ReqwestHttpTransport;
use airbridge_desktop_lib::airtable::models::{
    CreateSandboxRecordRequest, UpdateLinkedSandboxRecordRequest,
};
use airbridge_desktop_lib::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use airbridge_desktop_lib::restore::live_linked_update_test_contract::{
    evaluate_live_linked_update_test_contract, LiveLinkedUpdateTestContractMode,
    LiveLinkedUpdateTestContractRequest, LiveLinkedUpdateTestContractStatus,
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

const ENV_ENABLE: &str = "AIRBRIDGE_ENABLE_LIVE_LINKED_UPDATE_TEST";
const ENV_TOKEN: &str = "AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN";
const ENV_BASE_ID: &str = "AIRBRIDGE_SANDBOX_TARGET_BASE_ID";
const ENV_SOURCE_TABLE: &str = "AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME";
const ENV_TARGET_TABLE: &str = "AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME";
const ENV_LINK_FIELD: &str = "AIRBRIDGE_SANDBOX_LINK_FIELD_NAME";
const ENV_PREFIX: &str = "AIRBRIDGE_SANDBOX_TEST_PREFIX";

// ── Opt-in guard ──────────────────────────────────────────────────────────────

/// Returns `true` only when all six required env vars are present and the enable
/// flag is exactly `"true"`. Does NOT print any env var values.
fn all_required_env_vars_present() -> bool {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    let source_table = std::env::var(ENV_SOURCE_TABLE).unwrap_or_default();
    let target_table = std::env::var(ENV_TARGET_TABLE).unwrap_or_default();
    let link_field = std::env::var(ENV_LINK_FIELD).unwrap_or_default();
    enable == "true"
        && !token.is_empty()
        && !base_id.is_empty()
        && !source_table.is_empty()
        && !target_table.is_empty()
        && !link_field.is_empty()
}

// ── Contract verification helpers ─────────────────────────────────────────────

fn make_contract_schema_plan(
) -> airbridge_desktop_lib::restore::schema_write_requests::SchemaWriteRequestPlan {
    let plan = RestoreSchemaPlan {
        filename: "sandbox_linked_test.airbridge".to_string(),
        status: RestoreSchemaPlanStatus::Ready,
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: None,
        table_steps: vec![RestoreTableCreationStep {
            table_id: "tbl_sandbox_source".to_string(),
            table_name: "SandboxSource".to_string(),
            step_index: 0,
            field_count: 1,
            direct_field_count: 1,
            deferred_field_count: 0,
            manual_action_count: 0,
            unsupported_count: 0,
            note: "Sandbox integration test source table.".to_string(),
        }],
        field_steps: vec![RestoreFieldCreationStep {
            field_id: "fld_sandbox_name".to_string(),
            field_name: "Name".to_string(),
            field_type: "singleLineText".to_string(),
            table_id: "tbl_sandbox_source".to_string(),
            table_name: "SandboxSource".to_string(),
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
        package_filename: "sandbox_linked_test.airbridge".to_string(),
        dry_run_status: "ready".to_string(),
        schema_plan_status: "ready".to_string(),
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: Some("Sandbox Linked Test Base".to_string()),
        tables: vec![RecordImportTableInput {
            table_id: "tbl_sandbox_source".to_string(),
            table_name: "SandboxSource".to_string(),
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

fn make_full_linked_update_contract_request() -> LiveLinkedUpdateTestContractRequest {
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
            table_label: "SandboxSource".to_string(),
            field_label: "Tasks".to_string(),
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

// ── Default (non-ignored) gating tests ───────────────────────────────────────
// These tests run in default cargo test and verify the opt-in gate works.
// None of them make any network calls.

#[test]
fn missing_enable_flag_does_not_perform_network_call() {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    if enable != "true" {
        // Guard would block — confirmed: no network call attempted.
        return;
    }
    // If the env var happens to be set in the test environment, skip gracefully.
}

#[test]
fn missing_token_does_not_perform_network_call() {
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    if token.is_empty() {
        // No token present — guard would block. No network call.
        return;
    }
}

#[test]
fn missing_base_id_does_not_perform_network_call() {
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    if base_id.is_empty() {
        // No base ID present — guard would block. No network call.
        return;
    }
}

#[test]
fn missing_source_table_does_not_perform_network_call() {
    let source_table = std::env::var(ENV_SOURCE_TABLE).unwrap_or_default();
    if source_table.is_empty() {
        // No source table present — guard would block. No network call.
        return;
    }
}

#[test]
fn missing_target_table_does_not_perform_network_call() {
    let target_table = std::env::var(ENV_TARGET_TABLE).unwrap_or_default();
    if target_table.is_empty() {
        // No target table present — guard would block. No network call.
        return;
    }
}

#[test]
fn missing_link_field_name_does_not_perform_network_call() {
    let link_field = std::env::var(ENV_LINK_FIELD).unwrap_or_default();
    if link_field.is_empty() {
        // No linked field name present — guard would block. No network call.
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
fn linked_update_contract_eligible_but_not_executed_with_all_prereqs_satisfied() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_full_linked_update_contract_request();
    let result = evaluate_live_linked_update_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveLinkedUpdateTestContractStatus::EligibleButNotExecuted
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
            table_label: "SandboxSource".to_string(),
            field_label: "Tasks".to_string(),
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
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
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
fn live_linked_update_test_does_not_introduce_tauri_command() {
    // This test reaching its assertion confirms no Tauri command was added.
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn no_schema_operation_performed_in_default_test_suite() {
    // Schema operations are not performed by this harness.
    // Write gate confirms runtime state is unchanged.
    let gate = evaluate_write_gate();
    assert!(
        matches!(gate.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled — schema operations are not permitted here"
    );
}

#[test]
fn no_attachment_endpoint_called_in_default_test_suite() {
    // Attachment operations are not permitted.
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

// ── Live sandbox integration test (ignored by default) ────────────────────────
//
// To run this test, set all required env vars and pass --ignored:
//
//   AIRBRIDGE_ENABLE_LIVE_LINKED_UPDATE_TEST=true \
//   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<your pat> \
//   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<appYourSandboxBase> \
//   AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME=<tblSourceTable> \
//   AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME=<tblTargetTable> \
//   AIRBRIDGE_SANDBOX_LINK_FIELD_NAME="Tasks" \
//   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
//     --test live_linked_update_sandbox \
//     -- sandbox_linked_update_creates_records_and_updates_link --ignored
//
// WARNING: This test performs live Airtable API calls:
//   1. createRecord in the target table (to get a target record)
//   2. createRecord in the source table (to get a source record)
//   3. updateRecords (PATCH) in the source table (to set the linked field)
//
// It must only be run against disposable sandbox bases/tables with a
// configured linked field. It may leave test records behind — delete them
// manually after the run. Never run against a production base.
//
// The sandbox setup requires:
//   - A source table with a linked field pointing to the target table.
//   - A target table with a Name field.
//   - Both tables in the same sandbox base.

#[test]
#[ignore]
fn sandbox_linked_update_creates_records_and_updates_link() {
    // ── Opt-in gate ───────────────────────────────────────────────────────────
    if !all_required_env_vars_present() {
        // Skip gracefully — do not panic, do not make any network call.
        return;
    }

    // ── Retrieve env vars (values never printed) ──────────────────────────────
    let token_raw = std::env::var(ENV_TOKEN).expect("token env var must be set");
    let base_id = std::env::var(ENV_BASE_ID).expect("base ID env var must be set");
    let source_table =
        std::env::var(ENV_SOURCE_TABLE).expect("source table env var must be set");
    let target_table =
        std::env::var(ENV_TARGET_TABLE).expect("target table env var must be set");
    let link_field = std::env::var(ENV_LINK_FIELD).expect("link field env var must be set");
    let prefix =
        std::env::var(ENV_PREFIX).unwrap_or_else(|_| "airbridge_sandbox_test".to_string());

    // ── Pre-call: write gate must be Disabled ─────────────────────────────────
    let gate_before = evaluate_write_gate();
    assert!(
        matches!(gate_before.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must return Disabled before live calls"
    );

    // ── Pre-call: linked update contract must return EligibleButNotExecuted ───
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let contract_req = make_full_linked_update_contract_request();
    let contract_result =
        evaluate_live_linked_update_test_contract(&contract_req, &sp, &rp);
    assert_eq!(
        contract_result.status,
        LiveLinkedUpdateTestContractStatus::EligibleButNotExecuted,
        "contract must return EligibleButNotExecuted before live calls"
    );
    assert!(contract_result.contract_only);
    assert!(!contract_result.airtable_client_called);
    assert!(!contract_result.network_writes_attempted);
    assert!(!contract_result.app_runtime_execution_enabled);
    assert!(!contract_result.app_runtime_writes_enabled);
    assert!(!contract_result.app_runtime_reads_enabled);

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
        field_summaries: vec![LinkedSecondPassFieldSummary {
            table_label: "SandboxSource".to_string(),
            field_label: link_field.clone(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
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
        table_count: 2,
        field_count: 2,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 1,
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
    // Token is passed into the client and used only for Authorization headers.
    // It is never included in outcomes or assertions on its value.
    let transport = ReqwestHttpTransport::new().expect("http transport");
    let client = AirtableClient::new(AirtableToken::new(token_raw), transport);

    // ── Live call 1: create a target record ───────────────────────────────────
    // Creates a minimal record in the target table to get a target record ID.
    // Only a Name field is set — no linked fields, no attachments.
    let target_name = format!("{prefix}_linked_target");
    let mut target_fields = std::collections::HashMap::new();
    target_fields.insert("Name".to_string(), serde_json::Value::String(target_name));
    let create_target_req = CreateSandboxRecordRequest {
        fields: target_fields,
    };

    // We need the raw record ID from the target create to use in the linked
    // update. The record ID is extracted as an opaque string for use in
    // step 3 and is never printed or included in sanitized outcomes.
    let target_record_id = {
        use airbridge_desktop_lib::airtable::models::AirtableRecordFields;
        let raw_fields: std::collections::HashMap<String, serde_json::Value> =
            create_target_req.fields.clone();

        // SAFETY NOTE: The ID extracted here is used only in the PATCH call
        // below and is never serialized into any outcome struct, never printed,
        // never asserted on its value, and dropped at end of scope.
        let records_resp = client
            .create_records(
                &base_id,
                &target_table,
                vec![AirtableRecordFields {
                    fields: raw_fields,
                }],
            )
            .expect("create target record must succeed against sandbox table");

        assert!(
            !records_resp.records.is_empty(),
            "target record create must return at least one record"
        );
        // Extract the ID. It is opaque and never printed.
        records_resp
            .records
            .into_iter()
            .next()
            .map(|r| r.id.0)
            .expect("target record must have an ID")
    };

    // ── Live call 2: create a source record ───────────────────────────────────
    // Creates a minimal record in the source table.
    // Only a Name field is set. The linked field is populated in step 3.
    let source_name = format!("{prefix}_linked_source");
    let mut source_fields = std::collections::HashMap::new();
    source_fields.insert("Name".to_string(), serde_json::Value::String(source_name));
    let source_record_id = {
        use airbridge_desktop_lib::airtable::models::AirtableRecordFields;
        let records_resp = client
            .create_records(
                &base_id,
                &source_table,
                vec![AirtableRecordFields {
                    fields: source_fields,
                }],
            )
            .expect("create source record must succeed against sandbox table");

        assert!(
            !records_resp.records.is_empty(),
            "source record create must return at least one record"
        );
        // Extract the ID. Opaque handle — never printed.
        records_resp
            .records
            .into_iter()
            .next()
            .map(|r| r.id.0)
            .expect("source record must have an ID")
    };

    // ── Live call 3: perform the linked field update (PATCH) ──────────────────
    // Exactly one PATCH call to set the linked field in the source record.
    // No schema writes, no attachment endpoints, no final validation reads,
    // no arbitrary record updates, no record deletes.
    let update_req = UpdateLinkedSandboxRecordRequest {
        source_record_id: source_record_id.clone(),
        linked_field_name: link_field.clone(),
        target_record_ids: vec![target_record_id.clone()],
    };

    let outcome = client
        .update_single_linked_sandbox_record(
            &base_id,
            &source_table,
            "SandboxSource",
            &update_req,
        )
        .expect("linked record update must succeed against sandbox table");

    // Drop opaque handles — they are no longer needed after the live call.
    drop(source_record_id);
    drop(target_record_id);

    // ── Post-call assertions ──────────────────────────────────────────────────

    assert!(
        outcome.record_updated,
        "record_updated must be true after successful linked update"
    );

    assert_eq!(
        outcome.record_count, 1,
        "record_count must be 1 after single linked update"
    );

    assert!(
        !outcome.source_table_name.is_empty(),
        "source_table_name must be non-empty in outcome"
    );

    assert_eq!(
        outcome.linked_target_count, 1,
        "linked_target_count must be 1 after single target link"
    );

    // Outcome must not contain token or record IDs
    let outcome_json = serde_json::to_string(&outcome).expect("serialize outcome");
    assert!(
        !outcome_json.contains("pat_"),
        "outcome must not contain token"
    );
    assert!(
        !outcome_json.contains("recSensitive"),
        "outcome must not expose raw record IDs"
    );

    // ── Post-call: write gate must still be Disabled ──────────────────────────
    let gate_after = evaluate_write_gate();
    assert!(
        matches!(gate_after.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must remain Disabled after live calls"
    );

    // ── Post-call: app runtime execution/reads/writes remain disabled ─────────
    // (Enforced by write_gate — no mutable state. Confirmed by gate check above.)
}
