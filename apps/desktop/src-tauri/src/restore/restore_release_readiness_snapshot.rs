/// Restore release readiness snapshot.
///
/// This module is a Rust-internal reporting layer — it has no Tauri command,
/// no UI surface, no TypeScript path, and does not call Airtable.
///
/// It produces a structured release-readiness snapshot for maintainers by
/// composing the post-E2E restore readiness audit and write gate results.
/// It separates backup readiness, restore planning readiness, sandbox harness
/// readiness, runtime restore readiness, and product/security approval status
/// into distinct named areas.
///
/// Intended use: called from Rust unit tests and internal diagnostics only.
/// Never exposed from normal app runtime.
use serde::{Deserialize, Serialize};

use crate::restore::post_e2e_restore_readiness_audit::{
    audit_post_e2e_restore_readiness, PostE2ERestoreReadinessAuditRequest,
    PostE2ERestoreReadinessAuditStatus,
};
use crate::restore::record_write_requests::RecordWriteRequestPlan;
use crate::restore::schema_write_requests::SchemaWriteRequestPlan;
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall release readiness snapshot status.
///
/// Safety invariants:
/// - `AlphaReadyRestoreRuntimeDisabled` does NOT enable runtime execution.
/// - `AlphaReadyRestoreRuntimeDisabled` does NOT call Airtable.
/// - `AlphaReadyRestoreRuntimeDisabled` does NOT modify `evaluate_write_gate()`.
/// - `AlphaReadyRestoreRuntimeDisabled` does NOT persist any state.
/// - No `Succeeded`, `Complete`, `Enabled`, `RestoreReady`, or `Done` variant.
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
pub enum RestoreReleaseReadinessSnapshotStatus {
    /// A required snapshot input is missing or unsafe. No snapshot produced.
    Blocked,
    /// Backup and restore planning capabilities are working. Sandbox harnesses
    /// are verified as test-only and opt-in. App runtime restore execution is
    /// disabled. This status does NOT mean user-facing restore is complete or
    /// approved for production use.
    AlphaReadyRestoreRuntimeDisabled,
}

/// Status of a single readiness area.
///
/// Note: `Succeeded`, `Complete`, and `Done` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreReleaseReadinessAreaStatus {
    /// Area is implemented and verified safe for its stated scope.
    Ready,
    /// Area is partially implemented — core capability present, some items deferred.
    PartiallyReady,
    /// Area is blocked by a missing or unsafe prerequisite.
    Blocked,
    /// Area is intentionally disabled in this version by product policy.
    Disabled,
    /// Area requires explicit product/security approval before it can be enabled.
    PendingApproval,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single readiness area in the snapshot.
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
pub struct RestoreReleaseReadinessArea {
    /// Stable area identifier (e.g. `RRRS-AREA-01`).
    pub area_id: String,
    /// Human-readable name.
    pub name: String,
    pub status: RestoreReleaseReadinessAreaStatus,
    /// Brief summary of the area's current state.
    pub summary: String,
}

/// Point-in-time safety snapshot for the release readiness evaluation.
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
pub struct RestoreReleaseReadinessSafetySnapshot {
    /// Always `true` — write gate returns `Disabled/DisabledByProductPolicy`.
    pub write_gate_disabled: bool,
    /// Whether the post-E2E audit returned `SandboxHarnessesReadyRuntimeDisabled`.
    pub post_e2e_audit_sandbox_ready_runtime_disabled: bool,
    /// Whether `explicit_internal_restore_release_snapshot_requested` was set.
    pub explicit_snapshot_flag_set: bool,
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

/// A pending work item before user-facing restore execution can be approved.
///
/// Safety properties:
/// - No token field.
/// - No absolute path.
/// - No record IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReleaseReadinessPendingWork {
    /// Stable identifier (e.g. `RRRS-PENDING-01`).
    pub pending_id: String,
    /// Human-readable label.
    pub label: String,
    pub note: String,
}

/// A maintainer recommendation from the snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReleaseReadinessRecommendation {
    /// Stable identifier (e.g. `RRRS-REC-01`).
    pub rec_id: String,
    pub label: String,
    pub note: String,
}

/// Request to the restore release readiness snapshot.
///
/// Safety invariants:
/// - No token field.
/// - No base ID field.
/// - No table/field/record values.
/// - No attachment URL.
///
/// `explicit_internal_restore_release_snapshot_requested` must be `true` for
/// any non-`Blocked` status to be returned. No UI control, Tauri command, or
/// runtime path sets this flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReleaseReadinessSnapshotRequest {
    /// Internal-only flag. Must be explicitly `true` to proceed.
    /// No UI control, Tauri command, or runtime path sets this flag.
    pub explicit_internal_restore_release_snapshot_requested: bool,
    /// Whether backup package creation is confirmed working.
    pub backup_package_creation_ready: bool,
    /// Whether package inspection is confirmed working.
    pub package_inspection_ready: bool,
    /// Whether restore dry-run planning is confirmed working.
    pub restore_dry_run_planning_ready: bool,
    /// Whether restore execution preview is confirmed working.
    pub restore_execution_preview_ready: bool,
    /// Whether `commands/restore.rs` does not expose live restore execution.
    pub restore_command_does_not_expose_live_execution: bool,
    /// Whether no Tauri command exposes live restore execution.
    pub no_tauri_command_exposes_live_execution: bool,
    /// Whether no TypeScript/UI path exposes live restore execution.
    pub no_typescript_ui_path_exposes_live_execution: bool,
    /// Forwarded to the post-E2E audit probe.
    pub schema_harness_ignored_by_default: bool,
    pub record_harness_ignored_by_default: bool,
    pub linked_update_harness_ignored_by_default: bool,
    pub final_validation_harness_ignored_by_default: bool,
    pub e2e_restore_harness_ignored_by_default: bool,
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
}

/// Result of the restore release readiness snapshot.
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
/// - Status is never `succeeded`, `complete`, `restoreReady`, `enabled`, `done`,
///   or `executionReady`.
/// - Not reachable from UI, TypeScript, or any Tauri command.
/// - `evaluate_write_gate()` is never modified.
/// - No token, base ID, table/field/record values in any field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReleaseReadinessSnapshotResult {
    pub status: RestoreReleaseReadinessSnapshotStatus,
    pub message: String,
    pub areas: Vec<RestoreReleaseReadinessArea>,
    pub safety_snapshot: RestoreReleaseReadinessSafetySnapshot,
    pub pending_work: Vec<RestoreReleaseReadinessPendingWork>,
    pub recommendations: Vec<RestoreReleaseReadinessRecommendation>,
    pub area_count: usize,
    pub ready_area_count: usize,
    pub disabled_area_count: usize,
    pub pending_approval_area_count: usize,
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

fn build_areas(req: &RestoreReleaseReadinessSnapshotRequest) -> Vec<RestoreReleaseReadinessArea> {
    vec![
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-01".to_string(),
            name: "Backup package creation".to_string(),
            status: if req.backup_package_creation_ready {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "Create a .airbridge ZIP backup package from a connected Airtable base. \
                      Requires explicit file picker and confirmation text. Token consumed \
                      for the HTTP client; never stored."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-02".to_string(),
            name: "Package inspection".to_string(),
            status: if req.package_inspection_ready {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "Select and inspect a .airbridge package. Validates manifest, schema, \
                      checksums, and record entries. Read-only; no Airtable calls, no token."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-03".to_string(),
            name: "Restore dry-run planning".to_string(),
            status: if req.restore_dry_run_planning_ready {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "Generate a restore plan preview from an inspected package. Shows tables, \
                      field compatibility, record counts, and warnings. Read-only; no Airtable \
                      calls, no token."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-04".to_string(),
            name: "Restore execution preview".to_string(),
            status: if req.restore_execution_preview_ready {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "Preview schema write plan, record write plan, linked second-pass plan, \
                      and mapping checkpoint plan. All previews are read-only; no Airtable \
                      calls, no token."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-05".to_string(),
            name: "Sandbox schema write harness".to_string(),
            status: if req.schema_harness_ignored_by_default {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "live_schema_write_sandbox.rs — opt-in Rust integration test \
                      (#[ignore] by default). Not reachable from app runtime."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-06".to_string(),
            name: "Sandbox record write harness".to_string(),
            status: if req.record_harness_ignored_by_default {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "live_record_write_sandbox.rs — opt-in Rust integration test \
                      (#[ignore] by default). Not reachable from app runtime."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-07".to_string(),
            name: "Sandbox linked update harness".to_string(),
            status: if req.linked_update_harness_ignored_by_default {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "live_linked_update_sandbox.rs — opt-in Rust integration test \
                      (#[ignore] by default). Not reachable from app runtime."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-08".to_string(),
            name: "Sandbox final validation read harness".to_string(),
            status: if req.final_validation_harness_ignored_by_default {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "live_final_validation_sandbox.rs — opt-in Rust integration test \
                      (#[ignore] by default). Not reachable from app runtime."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-09".to_string(),
            name: "Sandbox E2E restore harness".to_string(),
            status: if req.e2e_restore_harness_ignored_by_default {
                RestoreReleaseReadinessAreaStatus::Ready
            } else {
                RestoreReleaseReadinessAreaStatus::Blocked
            },
            summary: "live_e2e_restore_sandbox.rs — opt-in Rust integration test \
                      (#[ignore] by default) that sequences all five restore phases. \
                      Not reachable from app runtime."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-10".to_string(),
            name: "Runtime restore execution".to_string(),
            status: RestoreReleaseReadinessAreaStatus::Disabled,
            summary: "Disabled by product policy. evaluate_write_gate() always returns \
                      Disabled/DisabledByProductPolicy. No Tauri command exposes live \
                      restore execution. No TypeScript/UI path exists for live execution. \
                      Requires explicit product/security approval before enablement."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-11".to_string(),
            name: "Attachment handling".to_string(),
            status: RestoreReleaseReadinessAreaStatus::Disabled,
            summary: "Attachment endpoint calls and attachment URL fetches are blocked. \
                      Attachment phase disabled policy is enforced. Requires separate \
                      design and approval before enablement."
                .to_string(),
        },
        RestoreReleaseReadinessArea {
            area_id: "RRRS-AREA-12".to_string(),
            name: "Product/security approval for user-facing restore".to_string(),
            status: RestoreReleaseReadinessAreaStatus::PendingApproval,
            summary: "No product or security approval has been granted for user-facing \
                      restore execution. Sandbox harness readiness does not imply this \
                      approval. All pending work items must be addressed before approval \
                      can be considered."
                .to_string(),
        },
    ]
}

fn build_pending_work() -> Vec<RestoreReleaseReadinessPendingWork> {
    vec![
        RestoreReleaseReadinessPendingWork {
            pending_id: "RRRS-PENDING-01".to_string(),
            label: "Product/security decision for runtime restore enablement".to_string(),
            note: "A deliberate product and security decision is required before any \
                   user-facing restore execution is exposed. Alpha readiness does not \
                   imply this decision."
                .to_string(),
        },
        RestoreReleaseReadinessPendingWork {
            pending_id: "RRRS-PENDING-02".to_string(),
            label: "Runtime restore command contract (if ever approved)".to_string(),
            note: "A formal Tauri command contract is required before any live restore \
                   execution is exposed from app runtime. Not implemented."
                .to_string(),
        },
        RestoreReleaseReadinessPendingWork {
            pending_id: "RRRS-PENDING-03".to_string(),
            label: "UI confirmation and failure-state design (if ever approved)".to_string(),
            note: "User-facing confirmation flow, warning display, destructive-action \
                   acknowledgement, and failure/partial-failure UI must be designed and \
                   reviewed before any live restore execution is exposed."
                .to_string(),
        },
        RestoreReleaseReadinessPendingWork {
            pending_id: "RRRS-PENDING-04".to_string(),
            label: "Credential handling review for future runtime restore".to_string(),
            note: "Token storage, scoping, and secure delivery to the Airtable client \
                   must be reviewed before any user-facing restore execution. Tokens are \
                   not accepted or persisted in the current build."
                .to_string(),
        },
        RestoreReleaseReadinessPendingWork {
            pending_id: "RRRS-PENDING-05".to_string(),
            label: "Cleanup strategy for sandbox-created tables and records".to_string(),
            note: "The live sandbox harnesses may leave test tables and records in the \
                   sandbox base. A cleanup strategy is required before repeated sandbox \
                   test runs."
                .to_string(),
        },
        RestoreReleaseReadinessPendingWork {
            pending_id: "RRRS-PENDING-06".to_string(),
            label: "Attachment binary handling (remains disabled)".to_string(),
            note: "Attachment phase remains disabled. Attachment endpoint calls and \
                   binary file handling require separate design and approval."
                .to_string(),
        },
        RestoreReleaseReadinessPendingWork {
            pending_id: "RRRS-PENDING-07".to_string(),
            label: "User-facing restore documentation".to_string(),
            note: "End-user restore guide, limitations disclosure, and support documentation \
                   are pending. Current docs describe the planning and sandbox layers only."
                .to_string(),
        },
    ]
}

fn build_recommendations() -> Vec<RestoreReleaseReadinessRecommendation> {
    vec![
        RestoreReleaseReadinessRecommendation {
            rec_id: "RRRS-REC-01".to_string(),
            label: "Do not ship user-facing restore execution without product/security approval"
                .to_string(),
            note: "The current build is alpha-ready for backup and planning. Runtime restore \
                   execution requires explicit approval, a formal command contract, and \
                   UI confirmation design before being exposed to users."
                .to_string(),
        },
        RestoreReleaseReadinessRecommendation {
            rec_id: "RRRS-REC-02".to_string(),
            label: "Run sandbox E2E harness against a disposable base before any approval review"
                .to_string(),
            note: "The live_e2e_restore_sandbox.rs harness sequences all five restore phases \
                   against a sandbox base. Run it against a disposable base and review \
                   outcomes before requesting product/security approval."
                .to_string(),
        },
        RestoreReleaseReadinessRecommendation {
            rec_id: "RRRS-REC-03".to_string(),
            label: "Address all pending work items before requesting approval".to_string(),
            note: "Seven pending work items remain (RRRS-PENDING-01 through RRRS-PENDING-07). \
                   Each must be addressed or explicitly deferred before any approval review."
                .to_string(),
        },
    ]
}

fn build_safety_snapshot(
    explicit_flag: bool,
    post_e2e_audit_ready: bool,
) -> RestoreReleaseReadinessSafetySnapshot {
    RestoreReleaseReadinessSafetySnapshot {
        write_gate_disabled: true,
        post_e2e_audit_sandbox_ready_runtime_disabled: post_e2e_audit_ready,
        explicit_snapshot_flag_set: explicit_flag,
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

fn blocked_result(reason: &str) -> RestoreReleaseReadinessSnapshotResult {
    let pending_work = build_pending_work();
    let pending_work_count = pending_work.len();
    RestoreReleaseReadinessSnapshotResult {
        status: RestoreReleaseReadinessSnapshotStatus::Blocked,
        message: format!(
            "Restore release readiness snapshot blocked: {reason}. \
             App runtime restore execution remains disabled. \
             All live sandbox harnesses remain test-only and opt-in."
        ),
        areas: vec![],
        safety_snapshot: build_safety_snapshot(false, false),
        pending_work,
        recommendations: vec![],
        area_count: 0,
        ready_area_count: 0,
        disabled_area_count: 0,
        pending_approval_area_count: 0,
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

/// Produce a sanitized restore release readiness snapshot.
///
/// Safety invariants:
/// - Does NOT call Airtable.
/// - Does NOT execute any harness.
/// - Does NOT accept token, base ID, table, field, or record values.
/// - Does NOT modify `evaluate_write_gate()` behavior.
/// - Does NOT enable app runtime reads, writes, or execution.
/// - Returns `Blocked` for any unsafe or missing input.
/// - All safety fields in the result are always enforced regardless of status.
pub fn build_restore_release_readiness_snapshot(
    req: &RestoreReleaseReadinessSnapshotRequest,
    schema_plan: &SchemaWriteRequestPlan,
    record_plan: &RecordWriteRequestPlan,
) -> RestoreReleaseReadinessSnapshotResult {
    // Gate 1: explicit snapshot flag must be set
    if !req.explicit_internal_restore_release_snapshot_requested {
        return blocked_result("explicit_internal_restore_release_snapshot_requested is false");
    }

    // Gate 2: write gate must be Disabled (invariant check)
    let gate = evaluate_write_gate();
    if gate.status != RestoreWriteEngineStatus::Disabled {
        return blocked_result("evaluate_write_gate() did not return Disabled");
    }

    // Gate 3: runtime/command/UI safety inputs
    if !req.restore_command_does_not_expose_live_execution {
        return blocked_result("restore_command_does_not_expose_live_execution is false");
    }
    if !req.no_tauri_command_exposes_live_execution {
        return blocked_result("no_tauri_command_exposes_live_execution is false");
    }
    if !req.no_typescript_ui_path_exposes_live_execution {
        return blocked_result("no_typescript_ui_path_exposes_live_execution is false");
    }

    // Gate 4: post-E2E audit must return SandboxHarnessesReadyRuntimeDisabled
    let audit_req = PostE2ERestoreReadinessAuditRequest {
        explicit_internal_post_e2e_audit_requested: true,
        schema_harness_ignored_by_default: req.schema_harness_ignored_by_default,
        record_harness_ignored_by_default: req.record_harness_ignored_by_default,
        linked_update_harness_ignored_by_default: req.linked_update_harness_ignored_by_default,
        final_validation_harness_ignored_by_default: req
            .final_validation_harness_ignored_by_default,
        e2e_restore_harness_ignored_by_default: req.e2e_restore_harness_ignored_by_default,
        restore_command_does_not_expose_live_execution: req
            .restore_command_does_not_expose_live_execution,
        no_tauri_command_exposes_live_execution: req.no_tauri_command_exposes_live_execution,
        no_typescript_ui_path_exposes_live_execution: req
            .no_typescript_ui_path_exposes_live_execution,
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
        field_summaries: vec![],
        table_count: 1,
        field_count: 1,
        record_count: 1,
        id_mapping_entry_count: 1,
        linked_coverage_count: 0,
        attachment_metadata_count: 0,
        manifest_present: true,
    };
    let audit_result = audit_post_e2e_restore_readiness(&audit_req, schema_plan, record_plan);

    let post_e2e_audit_ready = audit_result.status
        == PostE2ERestoreReadinessAuditStatus::SandboxHarnessesReadyRuntimeDisabled;

    if !post_e2e_audit_ready {
        return blocked_result(
            "post-E2E restore readiness audit did not return \
             SandboxHarnessesReadyRuntimeDisabled",
        );
    }

    // All gates passed — build the snapshot
    let areas = build_areas(req);
    let pending_work = build_pending_work();
    let recommendations = build_recommendations();

    let ready_area_count = areas
        .iter()
        .filter(|a| a.status == RestoreReleaseReadinessAreaStatus::Ready)
        .count();
    let disabled_area_count = areas
        .iter()
        .filter(|a| a.status == RestoreReleaseReadinessAreaStatus::Disabled)
        .count();
    let pending_approval_area_count = areas
        .iter()
        .filter(|a| a.status == RestoreReleaseReadinessAreaStatus::PendingApproval)
        .count();
    let area_count = areas.len();
    let pending_work_count = pending_work.len();

    RestoreReleaseReadinessSnapshotResult {
        status: RestoreReleaseReadinessSnapshotStatus::AlphaReadyRestoreRuntimeDisabled,
        message: "Backup and restore planning capabilities are working. All five sandbox \
                  harnesses are verified as test-only and opt-in. App runtime restore \
                  execution is disabled by product policy. This snapshot does NOT imply \
                  user-facing restore execution is complete or approved for production use."
            .to_string(),
        area_count,
        ready_area_count,
        disabled_area_count,
        pending_approval_area_count,
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
        areas,
        pending_work,
        recommendations,
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
            filename: "snapshot_test.airbridge".to_string(),
            status: RestoreSchemaPlanStatus::Ready,
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
            table_steps: vec![RestoreTableCreationStep {
                table_id: "tbl_snap_01".to_string(),
                table_name: "SnapshotTestTable".to_string(),
                step_index: 0,
                field_count: 1,
                direct_field_count: 1,
                deferred_field_count: 0,
                manual_action_count: 0,
                unsupported_count: 0,
                note: "Snapshot test table.".to_string(),
            }],
            field_steps: vec![RestoreFieldCreationStep {
                field_id: "fld_snap_name".to_string(),
                field_name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
                table_id: "tbl_snap_01".to_string(),
                table_name: "SnapshotTestTable".to_string(),
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
            package_filename: "snapshot_test.airbridge".to_string(),
            dry_run_status: "ready".to_string(),
            schema_plan_status: "ready".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("Snapshot Test Base".to_string()),
            tables: vec![RecordImportTableInput {
                table_id: "tbl_snap_01".to_string(),
                table_name: "SnapshotTestTable".to_string(),
                record_count: Some(1),
                fields: vec![RecordImportFieldInput {
                    field_id: "fld_snap_name".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                }],
            }],
        };
        let import_plan = create_record_import_plan(&req);
        build_record_write_request_plan(&import_plan)
    }

    fn full_request() -> RestoreReleaseReadinessSnapshotRequest {
        RestoreReleaseReadinessSnapshotRequest {
            explicit_internal_restore_release_snapshot_requested: true,
            backup_package_creation_ready: true,
            package_inspection_ready: true,
            restore_dry_run_planning_ready: true,
            restore_execution_preview_ready: true,
            restore_command_does_not_expose_live_execution: true,
            no_tauri_command_exposes_live_execution: true,
            no_typescript_ui_path_exposes_live_execution: true,
            schema_harness_ignored_by_default: true,
            record_harness_ignored_by_default: true,
            linked_update_harness_ignored_by_default: true,
            final_validation_harness_ignored_by_default: true,
            e2e_restore_harness_ignored_by_default: true,
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
        }
    }

    fn all_false_request() -> RestoreReleaseReadinessSnapshotRequest {
        RestoreReleaseReadinessSnapshotRequest {
            explicit_internal_restore_release_snapshot_requested: false,
            backup_package_creation_ready: false,
            package_inspection_ready: false,
            restore_dry_run_planning_ready: false,
            restore_execution_preview_ready: false,
            restore_command_does_not_expose_live_execution: false,
            no_tauri_command_exposes_live_execution: false,
            no_typescript_ui_path_exposes_live_execution: false,
            schema_harness_ignored_by_default: false,
            record_harness_ignored_by_default: false,
            linked_update_harness_ignored_by_default: false,
            final_validation_harness_ignored_by_default: false,
            e2e_restore_harness_ignored_by_default: false,
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
        }
    }

    #[test]
    fn default_request_is_blocked() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&all_false_request(), &sp, &rp);
        assert_eq!(
            result.status,
            RestoreReleaseReadinessSnapshotStatus::Blocked
        );
    }

    #[test]
    fn missing_explicit_flag_is_blocked() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.explicit_internal_restore_release_snapshot_requested = false;
        let result = build_restore_release_readiness_snapshot(&req, &sp, &rp);
        assert_eq!(
            result.status,
            RestoreReleaseReadinessSnapshotStatus::Blocked
        );
        assert!(result
            .blocked_reason
            .as_deref()
            .unwrap_or("")
            .contains("explicit_internal_restore_release_snapshot_requested"));
    }

    #[test]
    fn post_e2e_audit_unsafe_blocks_snapshot() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        // Cause the post-E2E audit to return Blocked by unsetting a harness flag
        req.schema_harness_ignored_by_default = false;
        let result = build_restore_release_readiness_snapshot(&req, &sp, &rp);
        assert_eq!(
            result.status,
            RestoreReleaseReadinessSnapshotStatus::Blocked
        );
    }

    #[test]
    fn write_gate_default_remains_disabled() {
        let gate = evaluate_write_gate();
        assert_eq!(gate.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn full_request_returns_alpha_ready_runtime_disabled() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            RestoreReleaseReadinessSnapshotStatus::AlphaReadyRestoreRuntimeDisabled
        );
    }

    #[test]
    fn snapshot_reports_backup_and_planning_separately_from_runtime_restore() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert_eq!(
            result.status,
            RestoreReleaseReadinessSnapshotStatus::AlphaReadyRestoreRuntimeDisabled
        );
        // Backup and planning areas should be Ready
        let backup = result
            .areas
            .iter()
            .find(|a| a.area_id == "RRRS-AREA-01")
            .expect("RRRS-AREA-01 must exist");
        assert_eq!(backup.status, RestoreReleaseReadinessAreaStatus::Ready);

        // Runtime restore execution must be Disabled
        let runtime = result
            .areas
            .iter()
            .find(|a| a.area_id == "RRRS-AREA-10")
            .expect("RRRS-AREA-10 must exist");
        assert_eq!(runtime.status, RestoreReleaseReadinessAreaStatus::Disabled);
    }

    #[test]
    fn snapshot_reports_all_live_harnesses_as_ignored_opt_in() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        for area_id in &[
            "RRRS-AREA-05",
            "RRRS-AREA-06",
            "RRRS-AREA-07",
            "RRRS-AREA-08",
            "RRRS-AREA-09",
        ] {
            let area = result
                .areas
                .iter()
                .find(|a| a.area_id == *area_id)
                .unwrap_or_else(|| panic!("{area_id} must exist"));
            assert_eq!(
                area.status,
                RestoreReleaseReadinessAreaStatus::Ready,
                "harness area {area_id} must be Ready (confirmed as #[ignore])"
            );
        }
        assert!(result.live_harnesses_ignored_by_default);
    }

    #[test]
    fn snapshot_reports_runtime_restore_as_disabled() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        let area = result
            .areas
            .iter()
            .find(|a| a.area_id == "RRRS-AREA-10")
            .expect("RRRS-AREA-10 must exist");
        assert_eq!(area.status, RestoreReleaseReadinessAreaStatus::Disabled);
        assert_eq!(result.disabled_area_count, 2); // runtime + attachment
    }

    #[test]
    fn snapshot_reports_attachment_handling_as_disabled() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        let area = result
            .areas
            .iter()
            .find(|a| a.area_id == "RRRS-AREA-11")
            .expect("RRRS-AREA-11 must exist");
        assert_eq!(area.status, RestoreReleaseReadinessAreaStatus::Disabled);
    }

    #[test]
    fn snapshot_reports_product_approval_as_pending() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        let area = result
            .areas
            .iter()
            .find(|a| a.area_id == "RRRS-AREA-12")
            .expect("RRRS-AREA-12 must exist");
        assert_eq!(
            area.status,
            RestoreReleaseReadinessAreaStatus::PendingApproval
        );
        assert_eq!(result.pending_approval_area_count, 1);
    }

    #[test]
    fn app_runtime_execution_enabled_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(!result.app_runtime_execution_enabled);
        assert!(!result.safety_snapshot.app_runtime_execution_enabled);
    }

    #[test]
    fn app_runtime_writes_enabled_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(!result.app_runtime_writes_enabled);
        assert!(!result.safety_snapshot.app_runtime_writes_enabled);
    }

    #[test]
    fn app_runtime_reads_enabled_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(!result.app_runtime_reads_enabled);
        assert!(!result.safety_snapshot.app_runtime_reads_enabled);
    }

    #[test]
    fn network_reads_attempted_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(!result.network_reads_attempted);
        assert!(!result.safety_snapshot.network_reads_attempted);
    }

    #[test]
    fn network_writes_attempted_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(!result.network_writes_attempted);
        assert!(!result.safety_snapshot.network_writes_attempted);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(result.no_changes_made);
        assert!(result.safety_snapshot.no_changes_made);
    }

    #[test]
    fn airtable_client_called_is_always_false() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(!result.airtable_client_called);
        assert!(!result.safety_snapshot.airtable_client_called);
    }

    #[test]
    fn serialization_contains_no_sensitive_data() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize snapshot result");
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
            "must not contain CDN URL"
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
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        let json = serde_json::to_string(&result).expect("serialize snapshot result");
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
        let gate = evaluate_write_gate();
        assert_eq!(gate.status, RestoreWriteEngineStatus::Disabled);
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(!result.safety_snapshot.tauri_command_exposes_live_restore);
        assert!(
            !result
                .safety_snapshot
                .typescript_ui_path_exposes_live_restore
        );
    }

    #[test]
    fn tauri_command_exposes_execution_blocks_snapshot() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.no_tauri_command_exposes_live_execution = false;
        let result = build_restore_release_readiness_snapshot(&req, &sp, &rp);
        assert_eq!(
            result.status,
            RestoreReleaseReadinessSnapshotStatus::Blocked
        );
    }

    #[test]
    fn typescript_ui_path_exposes_execution_blocks_snapshot() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let mut req = full_request();
        req.no_typescript_ui_path_exposes_live_execution = false;
        let result = build_restore_release_readiness_snapshot(&req, &sp, &rp);
        assert_eq!(
            result.status,
            RestoreReleaseReadinessSnapshotStatus::Blocked
        );
    }

    #[test]
    fn blocked_result_safety_fields_always_enforced() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&all_false_request(), &sp, &rp);
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
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(result.pending_work_count > 0);
        assert_eq!(result.pending_work.len(), result.pending_work_count);
    }

    #[test]
    fn message_does_not_imply_restore_is_complete() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
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

    #[test]
    fn recommendations_are_present() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert!(
            !result.recommendations.is_empty(),
            "recommendations must be present"
        );
    }

    #[test]
    fn twelve_areas_reported() {
        let sp = make_schema_plan();
        let rp = make_record_plan();
        let result = build_restore_release_readiness_snapshot(&full_request(), &sp, &rp);
        assert_eq!(result.area_count, 12);
        assert_eq!(result.areas.len(), 12);
    }
}
