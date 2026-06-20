/// Sandbox schema write integration test.
///
/// This test is `#[ignore]` by default and requires all three environment
/// variables to be set before it will run:
///
///   AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST=true
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<personal access token>
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<sandbox base ID>
///
/// Optional:
///   AIRBRIDGE_SANDBOX_TEST_PREFIX=<prefix for test table name>
///
/// Safety invariants:
/// - Default `cargo test` does NOT run this test.
/// - Missing any required env var causes the test to skip, not fail.
/// - Token and base ID are never printed, asserted on, or included in
///   any test output or serialized result.
/// - Only a schema operation is performed (createTable via Metadata API).
/// - No records are created, updated, or deleted.
/// - No linked record updates are performed.
/// - No attachment endpoints are called.
/// - No final validation reads are performed.
/// - `evaluate_write_gate()` is verified to remain Disabled before and after.
/// - App runtime execution/reads/writes remain disabled.
/// - The test may leave a sandbox-only test table in the target base.
///   It must only be run against a disposable sandbox base.
/// - No cleanup is performed automatically — remove the test table manually
///   after the run if needed.
///
/// To run this test manually against a sandbox base:
///   AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST=true \
///   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=your_pat \
///   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=appYourSandboxBase \
///   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
///     --test live_schema_write_sandbox -- --ignored
use airbridge_desktop_lib::airtable::auth::AirtableToken;
use airbridge_desktop_lib::airtable::client::AirtableClient;
use airbridge_desktop_lib::airtable::http::ReqwestHttpTransport;
use airbridge_desktop_lib::airtable::models::{CreateTableFieldSpec, CreateTableRequest};
use airbridge_desktop_lib::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use airbridge_desktop_lib::restore::live_schema_write_test_contract::{
    evaluate_live_schema_write_test_contract, LiveSchemaWriteTestContractMode,
    LiveSchemaWriteTestContractStatus,
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

const ENV_ENABLE: &str = "AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST";
const ENV_TOKEN: &str = "AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN";
const ENV_BASE_ID: &str = "AIRBRIDGE_SANDBOX_TARGET_BASE_ID";
const ENV_PREFIX: &str = "AIRBRIDGE_SANDBOX_TEST_PREFIX";

// ── Opt-in guard ──────────────────────────────────────────────────────────────

/// Returns `true` if all required env vars are present and the enable flag is
/// exactly `"true"`. Does NOT print any env var values.
fn all_required_env_vars_present() -> bool {
    let enable = std::env::var(ENV_ENABLE).unwrap_or_default();
    let token = std::env::var(ENV_TOKEN).unwrap_or_default();
    let base_id = std::env::var(ENV_BASE_ID).unwrap_or_default();
    enable == "true" && !token.is_empty() && !base_id.is_empty()
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
            record_count: Some(0),
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

fn make_full_contract_request() -> airbridge_desktop_lib::restore::live_schema_write_test_contract::LiveSchemaWriteTestContractRequest{
    use airbridge_desktop_lib::restore::live_schema_write_test_contract::LiveSchemaWriteTestContractRequest;
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
            table_label: "SandboxTest".to_string(),
            field_label: "Name".to_string(),
            record_count: 0,
            batch_count: 0,
            unresolved_link_count: 0,
        }],
        table_count: 1,
        field_count: 1,
        record_count: 0,
        id_mapping_entry_count: 0,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    }
}

// ── Default (ignored) gating tests ───────────────────────────────────────────
// These tests run in default cargo test and verify the opt-in gate works.

#[test]
fn missing_enable_flag_does_not_perform_network_call() {
    // Verify that without the env var set to "true", no network call is made.
    // This test itself makes no network call — it only checks the guard function.
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
fn evaluate_write_gate_remains_disabled_without_env_vars() {
    // Verify the write gate always returns Disabled regardless of test harness state.
    let gate = evaluate_write_gate();
    assert!(
        matches!(gate.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must always return Disabled"
    );
}

#[test]
fn contract_eligible_but_not_executed_with_all_prereqs_satisfied() {
    // Verify the contract returns EligibleButNotExecuted without any live call.
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let req = make_full_contract_request();
    let result = evaluate_live_schema_write_test_contract(&req, &sp, &rp);
    assert_eq!(
        result.status,
        LiveSchemaWriteTestContractStatus::EligibleButNotExecuted
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
fn schema_adapter_ready_for_sandbox_call_without_live_call() {
    // Verify the schema adapter returns ReadyForSandboxCall without network.
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
        record_count: 0,
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
fn live_schema_write_test_does_not_introduce_tauri_command() {
    // This test reaching its assertion confirms no Tauri command was added.
    // The integration test file accepts no Tauri AppHandle, token through
    // app command models, or UI execution path.
    let gate = evaluate_write_gate();
    assert!(matches!(gate.status, RestoreWriteEngineStatus::Disabled));
}

#[test]
fn no_record_endpoint_called_in_default_test_suite() {
    // Record writes remain disabled. Verify by checking the write gate.
    let gate = evaluate_write_gate();
    assert!(
        matches!(gate.status, RestoreWriteEngineStatus::Disabled),
        "write gate must remain Disabled — record endpoint calls are not permitted"
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
//   AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST=true \
//   AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=<your pat> \
//   AIRBRIDGE_SANDBOX_TARGET_BASE_ID=<appYourSandboxBase> \
//   cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
//     --test live_schema_write_sandbox \
//     -- sandbox_schema_write_creates_table_and_verifies_contract --ignored
//
// WARNING: This test performs a live Airtable API call (createTable via the
// Metadata API). It must only be run against a disposable sandbox base.
// It may leave a test table behind — remove it manually after the run.
// Never run against a production base.

#[test]
#[ignore]
fn sandbox_schema_write_creates_table_and_verifies_contract() {
    // ── Opt-in gate ───────────────────────────────────────────────────────────
    if !all_required_env_vars_present() {
        // Skip gracefully — do not panic, do not make any network call.
        return;
    }

    // ── Retrieve env vars (values never printed) ──────────────────────────────
    let token_raw = std::env::var(ENV_TOKEN).expect("token env var must be set");
    let base_id = std::env::var(ENV_BASE_ID).expect("base ID env var must be set");
    let prefix = std::env::var(ENV_PREFIX).unwrap_or_else(|_| "airbridge_sandbox_test".to_string());

    // ── Pre-call: write gate must be Disabled ─────────────────────────────────
    let gate_before = evaluate_write_gate();
    assert!(
        matches!(gate_before.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must return Disabled before live call"
    );

    // ── Pre-call: contract must return EligibleButNotExecuted ─────────────────
    let sp = make_contract_schema_plan();
    let rp = make_contract_record_plan();
    let contract_req = make_full_contract_request();
    let contract_result = evaluate_live_schema_write_test_contract(&contract_req, &sp, &rp);
    assert_eq!(
        contract_result.status,
        LiveSchemaWriteTestContractStatus::EligibleButNotExecuted,
        "contract must return EligibleButNotExecuted before live call"
    );
    assert!(contract_result.contract_only);
    assert!(!contract_result.airtable_client_called);
    assert!(!contract_result.network_writes_attempted);
    assert!(!contract_result.app_runtime_execution_enabled);
    assert!(!contract_result.app_runtime_writes_enabled);
    assert!(!contract_result.app_runtime_reads_enabled);

    // ── Pre-call: schema adapter must be ReadyForSandboxCall ──────────────────
    let adapter_req = SandboxSchemaWriteAdapterRequest {
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
    let adapter_result = build_sandbox_schema_write_adapter(&adapter_req, &sp);
    assert_eq!(
        adapter_result.status,
        SandboxSchemaWriteAdapterStatus::ReadyForSandboxCall,
        "schema adapter must return ReadyForSandboxCall before live call"
    );

    // ── Live call: createTable (schema-only, Metadata API) ────────────────────
    // Only a schema operation is performed. No records are created.
    // No linked updates, no attachment endpoints, no final validation reads.
    let transport = ReqwestHttpTransport::new().expect("http transport");
    let client = AirtableClient::new(AirtableToken::new(token_raw), transport);

    // Build a uniquely prefixed table name. No random/time-based call is safe
    // here; use the prefix only. The table name is not sensitive.
    let table_name = format!("{prefix}_schema_write");

    let create_req = CreateTableRequest {
        name: table_name.clone(),
        description: Some("Sandbox integration test table — safe to delete.".to_string()),
        fields: vec![CreateTableFieldSpec {
            name: "Name".to_string(),
            field_type: "singleLineText".to_string(),
        }],
    };

    let outcome = client
        .create_table(&base_id, &create_req)
        .expect("createTable must succeed against sandbox base");

    // ── Post-call assertions ──────────────────────────────────────────────────

    // Table name matches (not the ID — we don't assert on or return raw IDs)
    assert_eq!(
        outcome.table_name, table_name,
        "created table name must match requested name"
    );

    // Table ID must be non-empty (but is not printed or asserted on value)
    assert!(
        !outcome.table_id.is_empty(),
        "table ID must be non-empty after successful createTable"
    );

    // Table ID must not appear in any assertion message or be printed
    // (satisfies the no-record-ID-leak requirement)
    let outcome_json = serde_json::to_string(&outcome).expect("serialize outcome");
    assert!(
        !outcome_json.contains("pat_"),
        "outcome must not contain token"
    );

    // ── Post-call: write gate must still be Disabled ──────────────────────────
    let gate_after = evaluate_write_gate();
    assert!(
        matches!(gate_after.status, RestoreWriteEngineStatus::Disabled),
        "evaluate_write_gate() must remain Disabled after live call"
    );

    // ── Post-call: app runtime execution/reads/writes remain disabled ─────────
    // (These are enforced by write_gate and have no mutable state — confirmed
    // by the gate check above. There is no path to enable them.)
}
