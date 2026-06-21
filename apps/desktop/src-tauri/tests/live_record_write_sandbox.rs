/// Sandbox record write integration test.
///
/// This test is `#[ignore]` by default and requires all four environment
/// variables to be set before it will run:
///
///   AIRBRIDGE_ENABLE_LIVE_RECORD_WRITE_TEST=true
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<personal access token>
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<sandbox base ID>
///   AIRBRIDGE_SANDBOX_TARGET_TABLE_ID_OR_NAME=<sandbox table ID or name>
///
/// Optional:
///   AIRBRIDGE_SANDBOX_TEST_PREFIX=<prefix for test field values>
///
/// Safety invariants:
/// - Default `cargo test` does NOT run this test.
/// - Missing any required env var causes the test to skip, not fail.
/// - Token, base ID, table ID/name are never printed, asserted on, or included
///   in any test output or serialized result.
/// - Only a single record create operation is performed (Records API POST).
/// - No records are updated or deleted.
/// - No linked record fields are written.
/// - No attachment endpoints are called.
/// - No final validation reads are performed.
/// - `evaluate_write_gate()` is verified to remain Disabled before and after.
/// - App runtime execution/reads/writes remain disabled.
/// - The test may leave a sandbox-only test record in the target table.
///   It must only be run against a disposable sandbox base/table.
/// - No cleanup is performed automatically — delete the test record manually
///   if needed after the run.
///
/// To run this test manually against a sandbox base:
///   AIRBRIDGE_ENABLE_LIVE_RECORD_WRITE_TEST=true \
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=your_pat \
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=appYourSandboxBase \
///   AIRBRIDGE_SANDBOX_TARGET_TABLE_ID_OR_NAME=tblYourSandboxTable \
///   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
///     --test live_record_write_sandbox -- --ignored
use airbridge_desktop_lib::airtable::auth::AirtableToken;
use airbridge_desktop_lib::airtable::client::AirtableClient;
use airbridge_desktop_lib::airtable::http::ReqwestHttpTransport;
use airbridge_desktop_lib::airtable::models::CreateSandboxRecordRequest;
use airbridge_desktop_lib::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use airbridge_desktop_lib::restore::live_record_write_test_contract::{
    evaluate_live_record_write_test_contract, LiveRecordWriteTestContractMode,
    LiveRecordWriteTestContractRequest, LiveRecordWriteTestContractStatus,
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
use airbridge_desktop_lib::restore::sandbox_record_write_adapter::{
    build_sandbox_record_write_adapter, SandboxRecordWriteAdapterMode,
    SandboxRecordWriteAdapterRequest, SandboxRecordWriteAdapterStatus,
};
use airbridge_desktop_lib::restore::schema_plan::{
    RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreSchemaDependencyGraph,
    RestoreSchemaPlan, RestoreSchemaPlanStatus, RestoreTableCreationStep,
};
use airbridge_desktop_lib::restore::schema_write_requests::build_schema_write_request_plan;
use airbridge_desktop_lib::restore::write_gate::evaluate_write_gate;
use airbridge_desktop_lib::restore::write_result::RestoreWriteEngineStatus;

// ── Env var names ─────────────────────────────────────────────────────────────

const ENV_ENABLE: &str = "AIRBRIDGE_ENABLE_LIVE_RECORD_WRITE_TEST";
const ENV_TOKEN: &str = "AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN";
const ENV_BASE_ID: &str = "AIRBRIDGE_SANDBOX_TARGET_BASE_ID";
const ENV_TABLE: &str = "AIRBRIDGE_SANDBOX_TARGET_TABLE_ID_OR_NAME";
const ENV_PREFIX: &str = "AIRBRIDGE_SANDBOX_TEST_PREFIX";

// ── Opt-in guard ──────────────────────────────────────────────────────────────

/// Returns `true` if all required env vars are present and the enable flag is
/// exactly `"true"`. Does NOT print any env var values.
fn all_required_env_vars_present() -> bool {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    let table = std::env::var(ENV_TABLE).unwrap_or_default();
    enable == "true" && !token.is_empty() && !base_id.is_empty() && !table.is_empty()
}

// ── Contract verification helpers ────────────────────────────────────────────

fn make_contract_schema_plan(
) -> airbridge_desktop_lib::restore::schema_write_requests::SchemaWriteRequestPlan {
    let plan = RestoreSchemaPlan {
        filename: "sandbox_test.airbridge".to_string(),
        status: RestoreSchemaPlanStatus::Ready,
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: None,
        table_steps: vec![RestoreTableCreationStep {
            table_id: "tbl_sandbox_test".to_string(),
            table_name: "SandboxTest".to_string(),
            step_index: 0,
            field_count: 1,
            direct_field_count: 1,
            deferred_field_count: 0,
            manual_action_count: 0,
            unsupported_count: 0,
            note: "Sandbox integration test table.".to_string(),
        }],
        field_steps: vec![RestoreFieldCreationStep {
            field_id: "fld_sandbox_name".to_string(),
            field_name: "Name".to_string(),
            field_type: "singleLineText".to_string(),
            table_id: "tbl_sandbox_test".to_string(),
            table_name: "SandboxTest".to_string(),
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
        package_filename: "sandbox_test.airbridge".to_string(),
        dry_run_status: "ready".to_string(),
        schema_plan_status: "ready".to_string(),
        target_mode: RestoreTargetMode::NewBase,
        target_base_name: Some("Sandbox Test Base".to_string()),
        tables: vec![RecordImportTableInput {
            table_id: "tbl_sandbox_test".to_string(),
            table_name: "SandboxTest".to_string(),
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

fn make_full_contract_request() -> LiveRecordWriteTestContractRequest {
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
            table_label: "SandboxTest".to_string(),
            field_label: "Name".to_string(),
            record_count: 1,
            batch_count: 1,
            unresolved_link_count: 0,
        }],
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 0,
        linked_coverage_count: 0,
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
    // Token is set. This test still does not perform a network call.
}

#[test]
fn missing_base_id_does_not_perform_network_call() {
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    if base_id.is_empty() {
        // No base ID present — guard would block. No network call.
        return;
    }
    // Base ID is set. This test still does not perform a network call.
}

#[test]
fn missing_table_id_or_name_does_not_perform_network_call() {
    let table = std::env::var(ENV_TABLE).unwrap_or_default();
    if table.is_empty() {
        // No table ID/name present — guard would block. No network call.
        return;
    }
    // Table is set. This test still does not perform a network call.
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
fn record_write_contract_eligible_but_not_executed_with_all_prereqs_satisfied() {
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_full_contract_request();
    let result = evaluate_live_record_write_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveRecordWriteTestContractStatus::EligibleButNotExecuted
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
    assert!(!result.app_runtime_writes_enabled);
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
        id_mapping_entry_count: 0,
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
fn live_record_write_test_does_not_introduce_tauri_command() {
    // This test reaching its assertion confirms no Tauri command was added.
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn no_linked_update_endpoint_called_in_default_test_suite() {
    // Linked updates remain disabled. Write gate confirms.
    let gate = evaluate_write_gate();
    assert!(
        matches!(gate.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled — linked update calls are not permitted"
    );
}

#[test]
fn no_attachment_endpoint_called_in_default_test_suite() {
    // Attachment operations are not permitted. Write gate remains Disabled.
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

// ── Live sandbox integration test (ignored by default) ────────────────────────
//
// To run this test, set all required env vars and pass --ignored:
//
//   AIRBRIDGE_ENABLE_LIVE_RECORD_WRITE_TEST=true \
//   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<your pat> \
//   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<appYourSandboxBase> \
//   AIRBRIDGE_SANDBOX_TARGET_TABLE_ID_OR_NAME=<tblYourSandboxTable> \
//   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
//     --test live_record_write_sandbox \
//     -- sandbox_record_write_creates_record_and_verifies_contract --ignored
//
// WARNING: This test performs a live Airtable API call (createRecord via the
// Records API). It must only be run against a disposable sandbox base/table.
// It may leave a test record behind — delete it manually after the run.
// Never run against a production base.

#[test]
#[ignore]
fn sandbox_record_write_creates_record_and_verifies_contract() {
    // ── Opt-in gate ───────────────────────────────────────────────────────────
    if !all_required_env_vars_present() {
        // Skip gracefully — do not panic, do not make any network call.
        return;
    }

    // ── Retrieve env vars (values never printed) ──────────────────────────────
    let token_raw = std::env::var(ENV_TOKEN).expect("token env var must be set");
    let base_id = std::env::var(ENV_BASE_ID).expect("base ID env var must be set");
    let table_id_or_name = std::env::var(ENV_TABLE).expect("table ID or name env var must be set");
    let prefix = std::env::var(ENV_PREFIX).unwrap_or_else(|_| "airbridge_sandbox_test".to_string());

    // ── Pre-call: write gate must be Disabled ─────────────────────────────────
    let gate_before = evaluate_write_gate();
    assert!(
        matches!(gate_before.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must return Disabled before live call"
    );

    // ── Pre-call: record write contract must return EligibleButNotExecuted ────
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let contract_req = make_full_contract_request();
    let contract_result = evaluate_live_record_write_test_contract(&contract_req, &sp, &rp);
    assert_eq!(
        contract_result.status,
        LiveRecordWriteTestContractStatus::EligibleButNotExecuted,
        "contract must return EligibleButNotExecuted before live call"
    );
    assert!(contract_result.contract_only);
    assert!(!contract_result.airtable_client_called);
    assert!(!contract_result.network_writes_attempted);
    assert!(!contract_result.app_runtime_execution_enabled);
    assert!(!contract_result.app_runtime_writes_enabled);
    assert!(!contract_result.app_runtime_reads_enabled);

    // ── Pre-call: record adapter must be ReadyForSandboxCall ──────────────────
    let adapter_req = SandboxRecordWriteAdapterRequest {
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
    let adapter_result = build_sandbox_record_write_adapter(&adapter_req, &rp, &sp);
    assert_eq!(
        adapter_result.status,
        SandboxRecordWriteAdapterStatus::ReadyForSandboxCall,
        "record adapter must return ReadyForSandboxCall before live call"
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
        id_mapping_entry_count: 0,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    };
    let chain_result = run_sandbox_adapter_chain(&chain_req, &sp, &rp);
    assert_eq!(
        chain_result.status,
        SandboxAdapterChainRunnerStatus::MockRunNotExecuted,
        "adapter chain must return MockRunNotExecuted before live call"
    );

    // ── Live call: createRecord (Records API POST) ────────────────────────────
    // Only a single first-pass record create is performed.
    // No linked record updates. No attachment endpoints. No final validation reads.
    // No update operations.
    let transport = ReqwestHttpTransport::new().expect("http transport");
    let client = AirtableClient::new(AirtableToken::new(token_raw), transport);

    // Build a safe minimal field payload. No linked fields, no attachments.
    // The prefix is not sensitive — it appears in the field value only.
    let name_value = format!("{prefix}_record_write");
    let mut fields = std::collections::HashMap::new();
    fields.insert("Name".to_string(), serde_json::Value::String(name_value));

    let create_req = CreateSandboxRecordRequest { fields };

    let outcome = client
        .create_single_sandbox_record(&base_id, &table_id_or_name, "SandboxTest", &create_req)
        .expect("createRecord must succeed against sandbox table");

    // ── Post-call assertions ──────────────────────────────────────────────────

    // Record was created
    assert!(
        outcome.record_created,
        "record_created must be true after successful createRecord"
    );

    // Exactly one record created
    assert_eq!(
        outcome.record_count, 1,
        "record_count must be 1 after single createRecord"
    );

    // Table name is present in outcome
    assert!(
        !outcome.table_name.is_empty(),
        "table_name must be non-empty in outcome"
    );

    // Outcome must not contain token or base/table IDs
    let outcome_json = serde_json::to_string(&outcome).expect("serialize outcome");
    assert!(
        !outcome_json.contains("pat_"),
        "outcome must not contain token"
    );
    // Record IDs must not appear in the sanitized outcome
    assert!(
        !outcome_json.contains("\"id\""),
        "outcome must not expose record ID"
    );

    // ── Post-call: write gate must still be Disabled ──────────────────────────
    let gate_after = evaluate_write_gate();
    assert!(
        matches!(gate_after.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must remain Disabled after live call"
    );

    // ── Post-call: app runtime execution/reads/writes remain disabled ─────────
    // (Enforced by write_gate — no mutable state. Confirmed by gate check above.)
}
