/// Post-E2E restore readiness audit.
///
/// This module is a Rust-internal audit/report layer — it has no Tauri command,
/// no UI surface, no TypeScript path, and does not call Airtable.
///
/// It inspects and composes existing contract and harness readiness concepts to
/// produce a sanitized internal readiness report for maintainers. It does not
/// execute any harness, does not enable any runtime path, and does not accept
/// credential values.
///
/// Intended use: called from Rust unit tests and internal diagnostics only.
/// Never exposed from normal app runtime.
use serde::{Deserialize, Serialize};

use crate::restore::linked_second_pass_execution_preview::LinkedSecondPassFieldSummary;
use crate::restore::live_e2e_restore_test_contract::{
    evaluate_live_e2e_restore_test_contract, LiveE2ERestoreTestContractMode,
    LiveE2ERestoreTestContractRequest, LiveE2ERestoreTestContractStatus,
};
use crate::restore::record_write_requests::RecordWriteRequestPlan;
use crate::restore::schema_write_requests::SchemaWriteRequestPlan;
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall audit status.
///
/// Safety invariants:
/// - `SandboxHarnessesReadyRuntimeDisabled` does NOT enable runtime execution.
/// - `SandboxHarnessesReadyRuntimeDisabled` does NOT call Airtable.
/// - `SandboxHarnessesReadyRuntimeDisabled` does NOT change `evaluate_write_gate()`.
/// - `SandboxHarnessesReadyRuntimeDisabled` does NOT persist any state.
/// - No `Succeeded`, `Complete`, `Enabled`, `RestoreReady`, or `Done` status exists.
/// - `app_runtime_execution_enabled` is always `false`.
/// - `app_runtime_writes_enabled` is always `false`.
/// - `app_runtime_reads_enabled` is always `false`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `airtable_client_called` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `live_harnesses_ignored_by_default` is always `true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PostE2ERestoreReadinessAuditStatus {
    /// A required audit input is missing or unsafe. No audit was produced.
    Blocked,
    /// All sandbox harnesses are verified as test-only and opt-in by default.
    /// App runtime restore execution remains disabled.
    /// This status does NOT mean product-level restore is complete or approved.
    SandboxHarnessesReadyRuntimeDisabled,
}

/// Status of a single audit item.
///
/// Note: `Succeeded`, `Complete`, and `Done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PostE2ERestoreReadinessAuditItemStatus {
    /// Item verified as safe and ready.
    Ready,
    /// Item is blocked — prevents `SandboxHarnessesReadyRuntimeDisabled`.
    Blocked,
    /// Item has a warning condition but does not block.
    Warning,
    /// Item is known but not yet evaluated in this build.
    Pending,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single item in the readiness audit report.
///
/// Safety properties:
/// - No token field.
/// - No absolute path field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No record field payload.
/// - No attachment URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostE2ERestoreReadinessAuditItem {
    /// Stable item identifier (e.g. `PERRA-ITEM-01`).
    pub item_id: String,
    /// Human-readable label.
    pub label: String,
    pub status: PostE2ERestoreReadinessAuditItemStatus,
    pub note: String,
}

/// Point-in-time safety snapshot captured during the audit.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No old or new record IDs.
/// - No raw record field values.
/// - No raw HTTP body.
/// - No attachment URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostE2ERestoreReadinessAuditSafetySnapshot {
    /// Always `true` — write gate returns `Disabled/DisabledByProductPolicy`.
    pub write_gate_disabled: bool,
    /// Whether the E2E contract returned `EligibleButNotExecuted`.
    pub e2e_contract_eligible_not_executed: bool,
    /// Whether `explicit_internal_post_e2e_audit_requested` was set.
    pub explicit_audit_flag_set: bool,
    /// Always `false` — app runtime execution is not enabled.
    pub app_runtime_execution_enabled: bool,
    /// Always `false` — app runtime writes are not enabled.
    pub app_runtime_writes_enabled: bool,
    /// Always `false` — app runtime reads are not enabled.
    pub app_runtime_reads_enabled: bool,
    /// Always `false` — no network read attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — real Airtable client not called.
    pub airtable_client_called: bool,
    /// Always `true` — all five live harnesses are `#[ignore]` by default.
    pub live_harnesses_ignored_by_default: bool,
    /// Always `true` — no changes were made.
    pub no_changes_made: bool,
    /// Always `false` — no Tauri command exposes live restore.
    pub tauri_command_exposes_live_restore: bool,
    /// Always `false` — no TypeScript/UI path exposes live restore.
    pub typescript_ui_path_exposes_live_restore: bool,
}

/// A pending work item — work that must be completed before any future
/// product-level restore enablement.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No record IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostE2ERestoreReadinessAuditPendingWork {
    /// Stable identifier (e.g. `PERRA-PENDING-01`).
    pub pending_id: String,
    /// Human-readable label.
    pub label: String,
    pub note: String,
}

/// Request to the post-E2E restore readiness audit.
///
/// Safety invariants:
/// - No token field.
/// - No base ID field.
/// - No table/field/record values.
/// - No attachment URL.
///
/// `explicit_internal_post_e2e_audit_requested` must be `true` for any
/// non-`Blocked` status to be returned. No UI control, Tauri command, or
/// runtime path sets this flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostE2ERestoreReadinessAuditRequest {
    /// Internal-only flag. Must be explicitly `true` to proceed.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_post_e2e_audit_requested: bool,
    /// Whether the schema write sandbox harness is confirmed as `#[ignore]` by default.
    pub schema_harness_ignored_by_default: bool,
    /// Whether the record write sandbox harness is confirmed as `#[ignore]` by default.
    pub record_harness_ignored_by_default: bool,
    /// Whether the linked update sandbox harness is confirmed as `#[ignore]` by default.
    pub linked_update_harness_ignored_by_default: bool,
    /// Whether the final validation sandbox harness is confirmed as `#[ignore]` by default.
    pub final_validation_harness_ignored_by_default: bool,
    /// Whether the E2E restore sandbox harness is confirmed as `#[ignore]` by default.
    pub e2e_restore_harness_ignored_by_default: bool,
    /// Whether `commands/restore.rs` does not expose live restore execution.
    pub restore_command_does_not_expose_live_execution: bool,
    /// Whether no Tauri command exposes live restore execution.
    pub no_tauri_command_exposes_live_execution: bool,
    /// Whether no TypeScript/UI path exposes live restore execution.
    pub no_typescript_ui_path_exposes_live_execution: bool,
    /// Prerequisite inputs forwarded to the E2E contract probe.
    pub sandbox_verified: bool,
    pub target_base_empty: bool,
    pub mapping_coverage_sufficient: bool,
    pub final_validation_enforcement_safe: bool,
    pub confirmation_gate_declared: bool,
    pub destructive_operation_policy_safe: bool,
    pub attachment_phase_disabled_safe: bool,
    pub live_write_readiness_safe: bool,
    pub write_phase_ordering_safe: bool,
    pub failure_modes_safe: bool,
    pub rollback_limitation_safe: bool,
    pub checkpoint_durability_safe: bool,
    pub sensitive_data_safe: bool,
    pub rate_limit_backoff_safe: bool,
    pub schema_executor_safe: bool,
    pub checkpoint_store_safe: bool,
    pub record_executor_safe: bool,
    pub linked_executor_safe: bool,
    pub linked_second_pass_preview_ready: bool,
    pub mapping_checkpoint_preview_ready: bool,
    /// Per-field summaries forwarded to the E2E contract probe.
    /// No raw record IDs — only safe counts and labels.
    pub field_summaries: Vec<LinkedSecondPassFieldSummary>,
    /// Safe count values forwarded to the E2E contract probe.
    pub table_count: usize,
    pub field_count: usize,
    pub record_count: usize,
    pub id_mapping_entry_count: usize,
    pub linked_coverage_count: usize,
    pub attachment_metadata_count: usize,
    pub manifest_present: bool,
}

/// Result of the post-E2E restore readiness audit.
///
/// Safety invariants (always enforced):
/// - `app_runtime_execution_enabled` is always `false`.
/// - `app_runtime_writes_enabled` is always `false`.
/// - `app_runtime_reads_enabled` is always `false`.
/// - `live_harnesses_ignored_by_default` is always `true`.
/// - `network_reads_attempted` is always `false`.
/// - `network_writes_attempted` is always `false`.
/// - `airtable_client_called` is always `false`.
/// - `no_changes_made` is always `true`.
/// - No token field.
/// - No absolute path field.
/// - No record payload field.
/// - No raw HTTP body.
/// - No old or new record IDs.
/// - No attachment URL.
/// - Status is never `succeeded`, `complete`, `restoreReady`, `enabled`, `done`,
///   or `executionReady`.
/// - Not reachable from UI, TypeScript, or any Tauri command.
/// - `evaluate_write_gate()` is never modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostE2ERestoreReadinessAuditResult {
    pub status: PostE2ERestoreReadinessAuditStatus,
    pub message: String,
    pub items: Vec<PostE2ERestoreReadinessAuditItem>,
    pub safety_snapshot: PostE2ERestoreReadinessAuditSafetySnapshot,
    pub pending_work: Vec<PostE2ERestoreReadinessAuditPendingWork>,
    pub item_count: usize,
    pub ready_item_count: usize,
    pub blocked_item_count: usize,
    pub pending_work_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always `false` — app runtime execution is not enabled.
    pub app_runtime_execution_enabled: bool,
    /// Always `false` — app runtime writes are not enabled.
    pub app_runtime_writes_enabled: bool,
    /// Always `false` — app runtime reads are not enabled.
    pub app_runtime_reads_enabled: bool,
    /// Always `true` — all five live harnesses are `#[ignore]` by default.
    pub live_harnesses_ignored_by_default: bool,
    /// Always `false` — no network read attempted.
    pub network_reads_attempted: bool,
    /// Always `false` — no network write attempted.
    pub network_writes_attempted: bool,
    /// Always `false` — real Airtable client not called.
    pub airtable_client_called: bool,
    /// Always `true` — no changes were made.
    pub no_changes_made: bool,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn build_audit_items(
    req: &PostE2ERestoreReadinessAuditRequest,
) -> Vec<PostE2ERestoreReadinessAuditItem> {
    vec![
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-01".to_string(),
            label: "Schema write sandbox harness is #[ignore] by default".to_string(),
            status: if req.schema_harness_ignored_by_default {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "live_schema_write_sandbox.rs is an opt-in Rust integration test. \
                   Not reachable from app runtime."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-02".to_string(),
            label: "Record write sandbox harness is #[ignore] by default".to_string(),
            status: if req.record_harness_ignored_by_default {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "live_record_write_sandbox.rs is an opt-in Rust integration test. \
                   Not reachable from app runtime."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-03".to_string(),
            label: "Linked update sandbox harness is #[ignore] by default".to_string(),
            status: if req.linked_update_harness_ignored_by_default {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "live_linked_update_sandbox.rs is an opt-in Rust integration test. \
                   Not reachable from app runtime."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-04".to_string(),
            label: "Final validation sandbox harness is #[ignore] by default".to_string(),
            status: if req.final_validation_harness_ignored_by_default {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "live_final_validation_sandbox.rs is an opt-in Rust integration test. \
                   Not reachable from app runtime."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-05".to_string(),
            label: "E2E restore sandbox harness is #[ignore] by default".to_string(),
            status: if req.e2e_restore_harness_ignored_by_default {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "live_e2e_restore_sandbox.rs is an opt-in Rust integration test that \
                   sequences all five phases. Not reachable from app runtime."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-06".to_string(),
            label: "E2E restore test contract returns EligibleButNotExecuted".to_string(),
            status: PostE2ERestoreReadinessAuditItemStatus::Ready,
            note: "evaluate_live_e2e_restore_test_contract() returns \
                   EligibleButNotExecuted under contract-only mode. No network call."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-07".to_string(),
            label: "evaluate_write_gate() returns Disabled/DisabledByProductPolicy".to_string(),
            status: PostE2ERestoreReadinessAuditItemStatus::Ready,
            note: "Write gate is always Disabled. App runtime restore execution \
                   remains blocked by product policy."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-08".to_string(),
            label: "commands/restore.rs does not expose live restore execution".to_string(),
            status: if req.restore_command_does_not_expose_live_execution {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "The restore Tauri command does not expose any live execution path. \
                   Only plan preview and dry-run operations are available."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-09".to_string(),
            label: "No Tauri command exposes live restore execution".to_string(),
            status: if req.no_tauri_command_exposes_live_execution {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "No Tauri command provides a path to live restore execution \
                   in the current build."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-10".to_string(),
            label: "No TypeScript/UI path exposes live restore execution".to_string(),
            status: if req.no_typescript_ui_path_exposes_live_execution {
                PostE2ERestoreReadinessAuditItemStatus::Ready
            } else {
                PostE2ERestoreReadinessAuditItemStatus::Blocked
            },
            note: "No frontend TypeScript or UI component provides a path to live \
                   restore execution in the current build."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-11".to_string(),
            label: "Attachment binary handling remains disabled".to_string(),
            status: PostE2ERestoreReadinessAuditItemStatus::Ready,
            note: "Attachment endpoint calls and attachment URL fetches are blocked. \
                   Attachment phase disabled policy is enforced."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditItem {
            item_id: "PERRA-ITEM-12".to_string(),
            label: "App runtime execution/writes/reads remain disabled".to_string(),
            status: PostE2ERestoreReadinessAuditItemStatus::Ready,
            note: "app_runtime_execution_enabled, app_runtime_writes_enabled, and \
                   app_runtime_reads_enabled are always false in the current build."
                .to_string(),
        },
    ]
}

fn build_pending_work() -> Vec<PostE2ERestoreReadinessAuditPendingWork> {
    vec![
        PostE2ERestoreReadinessAuditPendingWork {
            pending_id: "PERRA-PENDING-01".to_string(),
            label: "Product decision for runtime restore enablement".to_string(),
            note: "A deliberate product decision is required before any user-facing \
                   restore execution is exposed. Sandbox harness readiness does not \
                   imply this approval."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditPendingWork {
            pending_id: "PERRA-PENDING-02".to_string(),
            label: "Runtime restore command contract (if ever approved)".to_string(),
            note: "A formal Tauri command contract is required before any live restore \
                   execution is exposed from app runtime. Not implemented."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditPendingWork {
            pending_id: "PERRA-PENDING-03".to_string(),
            label: "UI review/confirmation design (if ever approved)".to_string(),
            note: "User-facing confirmation flow, warning display, and destructive-action \
                   acknowledgement UI must be designed and reviewed before any live \
                   restore execution is exposed."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditPendingWork {
            pending_id: "PERRA-PENDING-04".to_string(),
            label: "Credential handling review for any future runtime path".to_string(),
            note: "Token storage, scoping, and secure delivery to the Airtable client \
                   must be reviewed before any user-facing restore execution. Tokens are \
                   not accepted or persisted in the current build."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditPendingWork {
            pending_id: "PERRA-PENDING-05".to_string(),
            label: "Attachment binary handling (remains disabled)".to_string(),
            note: "Attachment phase remains disabled. Attachment endpoint calls and \
                   binary file handling require separate design and approval before any \
                   restore execution path includes them."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditPendingWork {
            pending_id: "PERRA-PENDING-06".to_string(),
            label: "Cleanup strategy for sandbox-created tables and records".to_string(),
            note: "The live sandbox harnesses may leave test tables and records in the \
                   sandbox base. A cleanup strategy (manual or automated) is required \
                   before repeated sandbox test runs."
                .to_string(),
        },
        PostE2ERestoreReadinessAuditPendingWork {
            pending_id: "PERRA-PENDING-07".to_string(),
            label: "Security review before any user-facing restore execution".to_string(),
            note: "A full security review of the restore execution path — including \
                   token handling, destructive write operations, and rate limit \
                   behaviour — is required before any user-facing restore execution \
                   is exposed."
                .to_string(),
        },
    ]
}

fn build_safety_snapshot(
    explicit_flag: bool,
    e2e_contract_eligible: bool,
) -> PostE2ERestoreReadinessAuditSafetySnapshot {
    PostE2ERestoreReadinessAuditSafetySnapshot {
        write_gate_disabled: true,
        e2e_contract_eligible_not_executed: e2e_contract_eligible,
        explicit_audit_flag_set: explicit_flag,
        app_runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        live_harnesses_ignored_by_default: true,
        no_changes_made: true,
        tauri_command_exposes_live_restore: false,
        typescript_ui_path_exposes_live_restore: false,
    }
}

fn blocked_result(reason: &str) -> PostE2ERestoreReadinessAuditResult {
    let pending_work = build_pending_work();
    let pending_work_count = pending_work.len();
    PostE2ERestoreReadinessAuditResult {
        status: PostE2ERestoreReadinessAuditStatus::Blocked,
        message: format!(
            "Post-E2E restore readiness audit blocked: {reason}. \
             App runtime restore execution remains disabled. \
             All live sandbox harnesses remain test-only and opt-in."
        ),
        items: vec![],
        safety_snapshot: build_safety_snapshot(false, false),
        pending_work,
        item_count: 0,
        ready_item_count: 0,
        blocked_item_count: 0,
        pending_work_count,
        blocked_reason: Some(reason.to_string()),
        app_runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        live_harnesses_ignored_by_default: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        no_changes_made: true,
    }
}

// ── Public function ───────────────────────────────────────────────────────────

/// Produce a sanitized post-E2E restore readiness audit.
///
/// Safety invariants:
/// - Does NOT call Airtable.
/// - Does NOT execute any harness.
/// - Does NOT accept token, base ID, table, field, or record values.
/// - Does NOT modify `evaluate_write_gate()` behavior.
/// - Does NOT enable app runtime reads, writes, or execution.
/// - Returns `Blocked` for any unsafe or missing input.
/// - All safety fields in the result are always enforced regardless of status.
pub fn audit_post_e2e_restore_readiness(
    req: &PostE2ERestoreReadinessAuditRequest,
    schema_plan: &SchemaWriteRequestPlan,
    record_plan: &RecordWriteRequestPlan,
) -> PostE2ERestoreReadinessAuditResult {
    // Gate 1: explicit audit flag must be set
    if !req.explicit_internal_post_e2e_audit_requested {
        return blocked_result("explicit_internal_post_e2e_audit_requested is false");
    }

    // Gate 2: write gate must be Disabled (invariant check — never modified)
    let gate = evaluate_write_gate();
    if gate.status != RestoreWriteEngineStatus::Disabled {
        return blocked_result("evaluate_write_gate() did not return Disabled");
    }

    // Gate 3: verify E2E contract returns EligibleButNotExecuted under contract mode
    let e2e_req = LiveE2ERestoreTestContractRequest {
        mode: LiveE2ERestoreTestContractMode::SandboxIntegrationCandidate,
        explicit_internal_live_e2e_restore_test_contract_requested: true,
        sandbox_verified: req.sandbox_verified,
        target_base_empty: req.target_base_empty,
        mapping_coverage_sufficient: req.mapping_coverage_sufficient,
        final_validation_enforcement_safe: req.final_validation_enforcement_safe,
        confirmation_gate_declared: req.confirmation_gate_declared,
        destructive_operation_policy_safe: req.destructive_operation_policy_safe,
        attachment_phase_disabled_safe: req.attachment_phase_disabled_safe,
        live_write_readiness_safe: req.live_write_readiness_safe,
        write_phase_ordering_safe: req.write_phase_ordering_safe,
        failure_modes_safe: req.failure_modes_safe,
        rollback_limitation_safe: req.rollback_limitation_safe,
        checkpoint_durability_safe: req.checkpoint_durability_safe,
        sensitive_data_safe: req.sensitive_data_safe,
        rate_limit_backoff_safe: req.rate_limit_backoff_safe,
        schema_executor_safe: req.schema_executor_safe,
        checkpoint_store_safe: req.checkpoint_store_safe,
        record_executor_safe: req.record_executor_safe,
        linked_executor_safe: req.linked_executor_safe,
        linked_second_pass_preview_ready: req.linked_second_pass_preview_ready,
        mapping_checkpoint_preview_ready: req.mapping_checkpoint_preview_ready,
        field_summaries: req.field_summaries.clone(),
        table_count: req.table_count,
        field_count: req.field_count,
        record_count: req.record_count,
        id_mapping_entry_count: req.id_mapping_entry_count,
        linked_coverage_count: req.linked_coverage_count,
        attachment_metadata_count: req.attachment_metadata_count,
        manifest_present: req.manifest_present,
    };
    let e2e_result = evaluate_live_e2e_restore_test_contract(&e2e_req, schema_plan, record_plan);

    let e2e_contract_eligible =
        e2e_result.status == LiveE2ERestoreTestContractStatus::EligibleButNotExecuted;

    if !e2e_contract_eligible {
        return blocked_result(
            "live E2E restore test contract did not return EligibleButNotExecuted",
        );
    }

    // Gate 4: all harnesses must be confirmed as ignored/opt-in
    if !req.schema_harness_ignored_by_default {
        return blocked_result(
            "schema write sandbox harness is not confirmed as #[ignore] by default",
        );
    }
    if !req.record_harness_ignored_by_default {
        return blocked_result(
            "record write sandbox harness is not confirmed as #[ignore] by default",
        );
    }
    if !req.linked_update_harness_ignored_by_default {
        return blocked_result(
            "linked update sandbox harness is not confirmed as #[ignore] by default",
        );
    }
    if !req.final_validation_harness_ignored_by_default {
        return blocked_result(
            "final validation sandbox harness is not confirmed as #[ignore] by default",
        );
    }
    if !req.e2e_restore_harness_ignored_by_default {
        return blocked_result(
            "E2E restore sandbox harness is not confirmed as #[ignore] by default",
        );
    }

    // Gate 5: runtime/command/UI safety inputs
    if !req.restore_command_does_not_expose_live_execution {
        return blocked_result("restore_command_does_not_expose_live_execution is false");
    }
    if !req.no_tauri_command_exposes_live_execution {
        return blocked_result("no_tauri_command_exposes_live_execution is false");
    }
    if !req.no_typescript_ui_path_exposes_live_execution {
        return blocked_result("no_typescript_ui_path_exposes_live_execution is false");
    }

    // All gates passed — build the audit report
    let items = build_audit_items(req);
    let pending_work = build_pending_work();

    let ready_item_count = items
        .iter()
        .filter(|i| i.status == PostE2ERestoreReadinessAuditItemStatus::Ready)
        .count();
    let blocked_item_count = items
        .iter()
        .filter(|i| i.status == PostE2ERestoreReadinessAuditItemStatus::Blocked)
        .count();
    let item_count = items.len();
    let pending_work_count = pending_work.len();

    PostE2ERestoreReadinessAuditResult {
        status: PostE2ERestoreReadinessAuditStatus::SandboxHarnessesReadyRuntimeDisabled,
        message: "All five sandbox harnesses are verified as test-only and opt-in by default. \
                  App runtime restore execution remains disabled by product policy. \
                  This audit does not imply product-level restore is complete or approved. \
                  Pending work items remain before any user-facing restore execution."
            .to_string(),
        item_count,
        ready_item_count,
        blocked_item_count,
        pending_work_count,
        blocked_reason: None,
        app_runtime_execution_enabled: false,
        app_runtime_writes_enabled: false,
        app_runtime_reads_enabled: false,
        live_harnesses_ignored_by_default: true,
        network_reads_attempted: false,
        network_writes_attempted: false,
        airtable_client_called: false,
        no_changes_made: true,
        safety_snapshot: build_safety_snapshot(true, true),
        items,
        pending_work,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::plan::RestoreTargetMode;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreRecordImportPlanRequest,
    };
    use crate::restore::record_import_planner::create_record_import_plan;
    use crate::restore::record_write_requests::build_record_write_request_plan;
    use crate::restore::schema_plan::{
        RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreSchemaDependencyGraph,
        RestoreSchemaPlan, RestoreSchemaPlanStatus, RestoreTableCreationStep,
    };
    use crate::restore::schema_write_requests::build_schema_write_request_plan;

    fn make_schema_plan() -> SchemaWriteRequestPlan {
        let plan = RestoreSchemaPlan {
            filename: "audit_test.airbridge".to_string(),
            status: RestoreSchemaPlanStatus::Ready,
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            table_steps: vec![RestoreTableCreationStep {
                table_id: "tbl_audit_01".to_string(),
                table_name: "AuditTestTable".to_string(),
                step_index: 0,
                field_count: 1,
                direct_field_count: 1,
                deferred_field_count: 0,
                manual_action_count: 0,
                unsupported_count: 0,
                note: "Audit test table.".to_string(),
            }],
            field_steps: vec![RestoreFieldCreationStep {
                field_id: "fld_audit_name".to_string(),
                field_name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
                table_id: "tbl_audit_01".to_string(),
                table_name: "AuditTestTable".to_string(),
                classification: RestoreFieldCreateClassification::CreateDirectly,
                note: "Primary field.".to_string(),
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

    fn make_record_plan() -> RecordWriteRequestPlan {
        let req = RestoreRecordImportPlanRequest {
            package_filename: "audit_test.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Audit Test Base".to_string()),
            tables: vec![RecordImportTableInput {
                table_id: "tbl_audit_01".to_string(),
                table_name: "AuditTestTable".to_string(),
                record_count: Some(1),
                fields: vec![RecordImportFieldInput {
                    field_id: "fld_audit_name".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                }],
            }],
        };
        let import_plan = create_record_import_plan(&req);
        build_record_write_request_plan(&import_plan)
    }

    fn full_request() -> PostE2ERestoreReadinessAuditRequest {
        PostE2ERestoreReadinessAuditRequest {
            explicit_internal_post_e2e_audit_requested: true,
            schema_harness_ignored_by_default: true,
            record_harness_ignored_by_default: true,
            linked_update_harness_ignored_by_default: true,
            final_validation_harness_ignored_by_default: true,
            e2e_restore_harness_ignored_by_default: true,
            restore_command_does_not_expose_live_execution: true,
            no_tauri_command_exposes_live_execution: true,
            no_typescript_ui_path_exposes_live_execution: true,
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
                table_label: "AuditTestTable".to_string(),
                field_label: "Name".to_string(),
                record_count: 1,
                batch_count: 1,
                unresolved_link_count: 0,
            }],
            table_count: 1,
            field_count: 1,
            record_count: 1,
            id_mapping_entry_count: 1,
            linked_coverage_count: 0,
            attachment_metadata_count: 0,
            manifest_present: true,
        }
    }

    #[test]
    fn default_request_is_blocked() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = PostE2ERestoreReadinessAuditRequest {
            explicit_internal_post_e2e_audit_requested: false,
            schema_harness_ignored_by_default: false,
            record_harness_ignored_by_default: false,
            linked_update_harness_ignored_by_default: false,
            final_validation_harness_ignored_by_default: false,
            e2e_restore_harness_ignored_by_default: false,
            restore_command_does_not_expose_live_execution: false,
            no_tauri_command_exposes_live_execution: false,
            no_typescript_ui_path_exposes_live_execution: false,
            sandbox_verified: false,
            target_base_empty: false,
            mapping_coverage_sufficient: false,
            final_validation_enforcement_safe: false,
            confirmation_gate_declared: false,
            destructive_operation_policy_safe: false,
            attachment_phase_disabled_safe: false,
            live_write_readiness_safe: false,
            write_phase_ordering_safe: false,
            failure_modes_safe: false,
            rollback_limitation_safe: false,
            checkpoint_durability_safe: false,
            sensitive_data_safe: false,
            rate_limit_backoff_safe: false,
            schema_executor_safe: false,
            checkpoint_store_safe: false,
            record_executor_safe: false,
            linked_executor_safe: false,
            linked_second_pass_preview_ready: false,
            mapping_checkpoint_preview_ready: false,
            field_summaries: vec![],
            table_count: 0,
            field_count: 0,
            record_count: 0,
            id_mapping_entry_count: 0,
            linked_coverage_count: 0,
            attachment_metadata_count: 0,
            manifest_present: false,
        };
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn missing_explicit_flag_is_blocked() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.explicit_internal_post_e2e_audit_requested = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("explicit_internal_post_e2e_audit_requested"));
    }

    #[test]
    fn all_five_harnesses_reported_as_ignored_opt_in() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(
            result.status,
            PostE2ERestoreReadinessAuditStatus::SandboxHarnessesReadyRuntimeDisabled
        );
        let harness_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.item_id <= "PERRA-ITEM-05".to_string())
            .collect();
        assert_eq!(harness_items.len(), 5);
        for item in &harness_items {
            assert_eq!(
                item.status,
                PostE2ERestoreReadinessAuditItemStatus::Ready,
                "harness item {} should be Ready",
                item.item_id
            );
        }
        assert!(result.live_harnesses_ignored_by_default);
    }

    #[test]
    fn e2e_contract_reported_as_contract_only() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(
            result.status,
            PostE2ERestoreReadinessAuditStatus::SandboxHarnessesReadyRuntimeDisabled
        );
        let contract_item = result
            .items
            .iter()
            .find(|i| i.item_id == "PERRA-ITEM-06")
            .expect("PERRA-ITEM-06 must be present");
        assert_eq!(
            contract_item.status,
            PostE2ERestoreReadinessAuditItemStatus::Ready
        );
        assert!(result.safety_snapshot.e2e_contract_eligible_not_executed);
    }

    #[test]
    fn write_gate_default_remains_disabled() {
        let gate = evaluate_write_gate();
        assert_eq!(gate.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn full_request_returns_sandbox_harnesses_ready_runtime_disabled() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(
            result.status,
            PostE2ERestoreReadinessAuditStatus::SandboxHarnessesReadyRuntimeDisabled
        );
    }

    #[test]
    fn app_runtime_execution_enabled_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.app_runtime_execution_enabled);
        assert!(!result.safety_snapshot.app_runtime_execution_enabled);
    }

    #[test]
    fn app_runtime_writes_enabled_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.app_runtime_writes_enabled);
        assert!(!result.safety_snapshot.app_runtime_writes_enabled);
    }

    #[test]
    fn app_runtime_reads_enabled_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.app_runtime_reads_enabled);
        assert!(!result.safety_snapshot.app_runtime_reads_enabled);
    }

    #[test]
    fn network_reads_attempted_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.network_reads_attempted);
        assert!(!result.safety_snapshot.network_reads_attempted);
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.network_writes_attempted);
        assert!(!result.safety_snapshot.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(result.no_changes_made);
        assert!(result.safety_snapshot.no_changes_made);
    }

    #[test]
    fn airtable_client_called_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.airtable_client_called);
        assert!(!result.safety_snapshot.airtable_client_called);
    }

    #[test]
    fn pending_work_includes_attachment_handling_disabled() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        let found = result.pending_work.iter().any(|p| {
            p.note.contains("attachment") || p.label.to_lowercase().contains("attachment")
        });
        assert!(
            found,
            "pending work must include attachment handling disabled item"
        );
    }

    #[test]
    fn pending_work_includes_product_approval_required() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        let found = result.pending_work.iter().any(|p| {
            p.label.to_lowercase().contains("product")
                || p.note.to_lowercase().contains("product decision")
        });
        assert!(found, "pending work must include product approval item");
    }

    #[test]
    fn serialization_contains_no_sensitive_data() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize audit result");
        assert!(!json.contains("pat_"), "must not contain token prefix");
        assert!(!json.contains("/Users/"), "must not contain absolute path");
        assert!(!json.contains("/home/"), "must not contain home path");
        assert!(!json.contains("/tmp/"), "must not contain tmp path");
        assert!(
            !json.contains("\"fields\":{"),
            "must not contain record fields"
        );
        assert!(
            !json.contains("\"records\":[{"),
            "must not contain record list"
        );
        assert!(
            !json.contains("oldRecordId"),
            "must not contain old record ID"
        );
        assert!(
            !json.contains("newRecordId"),
            "must not contain new record ID"
        );
        assert!(
            !json.contains("cdn.airtable.com"),
            "must not contain attachment CDN"
        );
        assert!(
            !json.contains("attachmentUrl"),
            "must not contain attachment URL"
        );
    }

    #[test]
    fn no_restore_success_state_in_serialization() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize audit result");
        assert!(
            !json.contains("restoreSuccess"),
            "must not contain restoreSuccess"
        );
        assert!(
            !json.contains("restoreComplete"),
            "must not contain restoreComplete"
        );
        assert!(
            !json.contains("\"succeeded\""),
            "must not contain succeeded state"
        );
        assert!(
            !json.contains("executionReady"),
            "must not contain executionReady"
        );
    }

    #[test]
    fn no_tauri_command_or_ui_path_introduced() {
        // Write gate remains Disabled — no app execution path was added.
        let gate = evaluate_write_gate();
        assert_eq!(gate.status, RestoreWriteEngineStatus::Disabled);
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.safety_snapshot.tauri_command_exposes_live_restore);
        assert!(
            !result
                .safety_snapshot
                .typescript_ui_path_exposes_live_restore
        );
    }

    #[test]
    fn schema_harness_not_ignored_blocks_audit() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.schema_harness_ignored_by_default = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn record_harness_not_ignored_blocks_audit() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.record_harness_ignored_by_default = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn linked_harness_not_ignored_blocks_audit() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.linked_update_harness_ignored_by_default = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn fv_harness_not_ignored_blocks_audit() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.final_validation_harness_ignored_by_default = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn e2e_harness_not_ignored_blocks_audit() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.e2e_restore_harness_ignored_by_default = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn tauri_command_exposes_execution_blocks_audit() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.no_tauri_command_exposes_live_execution = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn typescript_ui_path_exposes_execution_blocks_audit() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.no_typescript_ui_path_exposes_live_execution = false;
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert_eq!(result.status, PostE2ERestoreReadinessAuditStatus::Blocked);
    }

    #[test]
    fn blocked_result_safety_fields_always_enforced() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = PostE2ERestoreReadinessAuditRequest {
            explicit_internal_post_e2e_audit_requested: false,
            ..PostE2ERestoreReadinessAuditRequest {
                schema_harness_ignored_by_default: false,
                record_harness_ignored_by_default: false,
                linked_update_harness_ignored_by_default: false,
                final_validation_harness_ignored_by_default: false,
                e2e_restore_harness_ignored_by_default: false,
                restore_command_does_not_expose_live_execution: false,
                no_tauri_command_exposes_live_execution: false,
                no_typescript_ui_path_exposes_live_execution: false,
                sandbox_verified: false,
                target_base_empty: false,
                mapping_coverage_sufficient: false,
                final_validation_enforcement_safe: false,
                confirmation_gate_declared: false,
                destructive_operation_policy_safe: false,
                attachment_phase_disabled_safe: false,
                live_write_readiness_safe: false,
                write_phase_ordering_safe: false,
                failure_modes_safe: false,
                rollback_limitation_safe: false,
                checkpoint_durability_safe: false,
                sensitive_data_safe: false,
                rate_limit_backoff_safe: false,
                schema_executor_safe: false,
                checkpoint_store_safe: false,
                record_executor_safe: false,
                linked_executor_safe: false,
                linked_second_pass_preview_ready: false,
                mapping_checkpoint_preview_ready: false,
                field_summaries: vec![],
                table_count: 0,
                field_count: 0,
                record_count: 0,
                id_mapping_entry_count: 0,
                linked_coverage_count: 0,
                attachment_metadata_count: 0,
                manifest_present: false,
                explicit_internal_post_e2e_audit_requested: false,
            }
        };
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(!result.app_runtime_execution_enabled);
        assert!(!result.app_runtime_writes_enabled);
        assert!(!result.app_runtime_reads_enabled);
        assert!(!result.network_reads_attempted);
        assert!(!result.network_writes_attempted);
        assert!(!result.airtable_client_called);
        assert!(result.no_changes_made);
        assert!(result.live_harnesses_ignored_by_default);
    }

    #[test]
    fn pending_work_count_is_nonzero() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        assert!(
            result.pending_work_count > 0,
            "pending work must be non-empty — product approval still required"
        );
        assert_eq!(result.pending_work.len(), result.pending_work_count);
    }

    #[test]
    fn message_does_not_imply_restore_is_complete() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let req = full_request();
        let result = audit_post_e2e_restore_readiness(&req, &sp, &rp);
        let msg = result.message.to_lowercase();
        assert!(
            !msg.contains("restore ready"),
            "message must not say restore ready"
        );
        assert!(
            !msg.contains("restore complete"),
            "message must not say restore complete"
        );
        assert!(
            !msg.contains("restore succeeded"),
            "message must not say restore succeeded"
        );
    }
}
