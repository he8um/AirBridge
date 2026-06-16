use serde::{Deserialize, Serialize};

use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::RestoreWriteEngineStatus;

// ── Public enums ──────────────────────────────────────────────────────────────

/// Overall status of the schema write execution preview.
///
/// Safety invariants:
/// - `DryRunReady` does NOT enable live writes.
/// - `writes_enabled` is always `false`.
/// - No Airtable API calls are made by this module.
/// - No token, full path, record payload, raw HTTP, or attachment URL appears
///   in any result field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteExecutionPreviewStatus {
    /// All safety prerequisites are present; a dry-run preview is available.
    /// Live writes remain disabled and no restore execution is started.
    DryRunReady,
    /// At least one required safety prerequisite is missing or unsafe.
    Blocked,
}

/// Status of a single ordered preview step.
///
/// Note: `succeeded` / `completed` are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteExecutionPreviewStepStatus {
    /// The step is planned and would be executed if writes were enabled.
    Pending,
    /// The step is blocked by a safety prerequisite.
    Blocked,
    /// The step is skipped because a prerequisite failed.
    Skipped,
}

/// Execution mode for the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaWriteExecutionPreviewMode {
    /// Dry-run only — no live execution path is reachable.
    DryRunOnly,
    /// Live writes are blocked by product policy.
    LiveBlocked,
}

// ── Public structs ────────────────────────────────────────────────────────────

/// A single ordered step in the schema write execution preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteExecutionPreviewStep {
    pub step_index: usize,
    pub step_id: String,
    pub label: String,
    pub status: SchemaWriteExecutionPreviewStepStatus,
    pub note: String,
}

/// A point-in-time safety snapshot for the preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteSafetySnapshot {
    pub write_gate_disabled: bool,
    pub sandbox_flag_present: bool,
    pub target_empty_verified: bool,
    pub schema_plan_ready: bool,
    pub destructive_policy_safe: bool,
    pub sensitive_data_safe: bool,
    pub attachment_phase_disabled: bool,
    pub final_validation_enforcement_present: bool,
    pub live_write_readiness_satisfied: bool,
}

/// Request for the schema write execution preview.
///
/// No token field — token is not required or accepted.
/// No full path field — filename label only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteExecutionPreviewRequest {
    /// Filename label for the backup package. No directory component.
    pub package_filename: Option<String>,
    /// Whether the sandbox environment check has passed.
    pub sandbox_flag_present: Option<bool>,
    /// Whether the target empty verification has passed.
    pub target_empty_verified: Option<bool>,
    /// Whether the schema plan is ready (not blocked).
    pub schema_plan_ready: Option<bool>,
    /// Number of tables in the schema plan. Used to build ordered steps.
    pub table_count: Option<usize>,
    /// Number of direct fields in the schema plan.
    pub direct_field_count: Option<usize>,
    /// Number of deferred (linked) fields in the schema plan.
    pub deferred_field_count: Option<usize>,
    /// Number of manual-action fields in the schema plan.
    pub manual_action_count: Option<usize>,
    /// Whether the destructive operation policy result is safe (not blocked).
    pub destructive_policy_safe: Option<bool>,
    /// Whether the sensitive data safety policy result is safe (compliant or warning).
    pub sensitive_data_safe: Option<bool>,
    /// Whether the attachment phase disabled policy result shows phase is disabled/metadata-only.
    pub attachment_phase_disabled: Option<bool>,
    /// Whether the final validation enforcement policy result is present (passed or warning).
    pub final_validation_enforcement_present: Option<bool>,
    /// Whether the live-write readiness result is ready or warning (advisory only).
    pub live_write_readiness_satisfied: Option<bool>,
}

/// Result of the schema write execution preview.
///
/// Safety invariants (enforced):
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - No token field.
/// - No full filesystem path field.
/// - No record payload field.
/// - No raw HTTP request or response body.
/// - No attachment URL.
/// - Status is never `succeeded` or any completion state.
/// - `DryRunReady` does NOT enable live writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteExecutionPreviewResult {
    pub status: SchemaWriteExecutionPreviewStatus,
    pub mode: SchemaWriteExecutionPreviewMode,
    pub message: String,
    pub steps: Vec<SchemaWriteExecutionPreviewStep>,
    pub safety_snapshot: SchemaWriteSafetySnapshot,
    pub table_step_count: usize,
    pub field_step_count: usize,
    pub deferred_step_count: usize,
    pub manual_step_count: usize,
    pub total_step_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    /// Always false — no network write operations were attempted.
    pub network_writes_attempted: bool,
    /// Always false — live writes are not enabled.
    pub writes_enabled: bool,
}

// ── Prerequisite IDs (stable labels, not Airtable IDs) ────────────────────────

const PREREQ_WRITE_GATE: &str = "SWEP-PRE-01";
const PREREQ_SANDBOX: &str = "SWEP-PRE-02";
const PREREQ_TARGET_EMPTY: &str = "SWEP-PRE-03";
const PREREQ_SCHEMA_PLAN: &str = "SWEP-PRE-04";
const PREREQ_DESTRUCTIVE_POLICY: &str = "SWEP-PRE-05";
const PREREQ_SENSITIVE_DATA: &str = "SWEP-PRE-06";
const PREREQ_ATTACHMENT_PHASE: &str = "SWEP-PRE-07";
const PREREQ_FINAL_VALIDATION: &str = "SWEP-PRE-08";
const PREREQ_LWR: &str = "SWEP-PRE-09";

// ── Core function ─────────────────────────────────────────────────────────────

/// Builds a schema write execution preview from a request.
///
/// This function:
/// - Never calls any Airtable API endpoint.
/// - Never creates a base, table, or field.
/// - Never returns a token, full path, record payload, raw HTTP body, or attachment URL.
/// - Always returns `writes_enabled: false`, `no_changes_made: true`,
///   `network_writes_attempted: false`.
/// - Always consults `evaluate_write_gate()` to confirm writes remain disabled.
/// - A `DryRunReady` result is advisory only — it does NOT enable write execution.
pub fn preview_schema_write_execution(
    request: &SchemaWriteExecutionPreviewRequest,
) -> SchemaWriteExecutionPreviewResult {
    // Always consult the write gate first.
    let gate = evaluate_write_gate();
    let write_gate_disabled = matches!(gate.status, RestoreWriteEngineStatus::Disabled);

    let sandbox_flag_present = request.sandbox_flag_present.unwrap_or(false);
    let target_empty_verified = request.target_empty_verified.unwrap_or(false);
    let schema_plan_ready = request.schema_plan_ready.unwrap_or(false);
    let destructive_policy_safe = request.destructive_policy_safe.unwrap_or(false);
    let sensitive_data_safe = request.sensitive_data_safe.unwrap_or(false);
    let attachment_phase_disabled = request.attachment_phase_disabled.unwrap_or(false);
    let final_validation_enforcement_present = request
        .final_validation_enforcement_present
        .unwrap_or(false);
    let live_write_readiness_satisfied = request.live_write_readiness_satisfied.unwrap_or(false);

    let snapshot = SchemaWriteSafetySnapshot {
        write_gate_disabled,
        sandbox_flag_present,
        target_empty_verified,
        schema_plan_ready,
        destructive_policy_safe,
        sensitive_data_safe,
        attachment_phase_disabled,
        final_validation_enforcement_present,
        live_write_readiness_satisfied,
    };

    // Check all prerequisites in order; collect the first blocking reason.
    let mut blocked_reason: Option<String> = None;

    if !write_gate_disabled {
        blocked_reason = Some(format!(
            "{PREREQ_WRITE_GATE}: Write gate is not disabled. Live schema writes must not be attempted."
        ));
    } else if !sandbox_flag_present {
        blocked_reason = Some(format!(
            "{PREREQ_SANDBOX}: Sandbox environment check has not passed. Run the sandbox environment gate before requesting a preview."
        ));
    } else if !target_empty_verified {
        blocked_reason = Some(format!(
            "{PREREQ_TARGET_EMPTY}: Target empty verification has not passed. Verify the target base is empty before requesting a preview."
        ));
    } else if !schema_plan_ready {
        blocked_reason = Some(format!(
            "{PREREQ_SCHEMA_PLAN}: Schema plan is not ready. Complete the schema plan before requesting a preview."
        ));
    } else if !destructive_policy_safe {
        blocked_reason = Some(format!(
            "{PREREQ_DESTRUCTIVE_POLICY}: Destructive operation policy is not safe. All planned operations must be create-only."
        ));
    } else if !sensitive_data_safe {
        blocked_reason = Some(format!(
            "{PREREQ_SENSITIVE_DATA}: Sensitive data safety policy is not satisfied. All exposure surfaces must be covered."
        ));
    } else if !attachment_phase_disabled {
        blocked_reason = Some(format!(
            "{PREREQ_ATTACHMENT_PHASE}: Attachment phase is not disabled or metadata-only. Attachment writes must be disabled."
        ));
    } else if !final_validation_enforcement_present {
        blocked_reason = Some(format!(
            "{PREREQ_FINAL_VALIDATION}: Final validation enforcement policy has not been verified. Run the final validation enforcement gate."
        ));
    } else if !live_write_readiness_satisfied {
        blocked_reason = Some(format!(
            "{PREREQ_LWR}: Live write readiness policy is not satisfied. All 17 required safety gates must be declared."
        ));
    }

    if let Some(ref reason) = blocked_reason {
        return SchemaWriteExecutionPreviewResult {
            status: SchemaWriteExecutionPreviewStatus::Blocked,
            mode: SchemaWriteExecutionPreviewMode::LiveBlocked,
            message: format!(
                "Schema write execution preview is blocked. {reason} \
                 Live schema writes remain disabled."
            ),
            steps: blocked_steps(),
            safety_snapshot: snapshot,
            table_step_count: 0,
            field_step_count: 0,
            deferred_step_count: 0,
            manual_step_count: 0,
            total_step_count: 0,
            blocked_reason: Some(reason.clone()),
            no_changes_made: true,
            network_writes_attempted: false,
            writes_enabled: false,
        };
    }

    // All prerequisites satisfied — build the dry-run preview steps.
    let table_count = request.table_count.unwrap_or(0);
    let direct_field_count = request.direct_field_count.unwrap_or(0);
    let deferred_field_count = request.deferred_field_count.unwrap_or(0);
    let manual_count = request.manual_action_count.unwrap_or(0);

    let steps = build_preview_steps(
        table_count,
        direct_field_count,
        deferred_field_count,
        manual_count,
    );
    let total = steps.len();

    SchemaWriteExecutionPreviewResult {
        status: SchemaWriteExecutionPreviewStatus::DryRunReady,
        mode: SchemaWriteExecutionPreviewMode::DryRunOnly,
        message: format!(
            "Schema write execution preview is ready (dry-run only). {} table(s), {} direct field(s), \
             {} deferred field(s), {} manual action(s) planned. \
             Live schema writes remain disabled. This preview does not start any restore execution.",
            table_count, direct_field_count, deferred_field_count, manual_count
        ),
        steps,
        safety_snapshot: snapshot,
        table_step_count: table_count,
        field_step_count: direct_field_count,
        deferred_step_count: deferred_field_count,
        manual_step_count: manual_count,
        total_step_count: total,
        blocked_reason: None,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn blocked_steps() -> Vec<SchemaWriteExecutionPreviewStep> {
    vec![SchemaWriteExecutionPreviewStep {
        step_index: 0,
        step_id: "SWEP-STEP-BLOCKED".to_string(),
        label: "Preview blocked".to_string(),
        status: SchemaWriteExecutionPreviewStepStatus::Blocked,
        note: "Safety prerequisites not satisfied. No steps can be previewed.".to_string(),
    }]
}

fn build_preview_steps(
    table_count: usize,
    direct_field_count: usize,
    deferred_field_count: usize,
    manual_count: usize,
) -> Vec<SchemaWriteExecutionPreviewStep> {
    let mut steps = Vec::new();
    let mut idx = 0usize;

    // Phase 1 — validate inputs
    steps.push(SchemaWriteExecutionPreviewStep {
        step_index: idx,
        step_id: "SWEP-STEP-VAL".to_string(),
        label: "Validate schema plan inputs".to_string(),
        status: SchemaWriteExecutionPreviewStepStatus::Pending,
        note: "Validates that the schema plan is complete and all required fields are present. No API calls.".to_string(),
    });
    idx += 1;

    // Phase 2 — create tables (one step per table)
    for t in 0..table_count {
        steps.push(SchemaWriteExecutionPreviewStep {
            step_index: idx,
            step_id: format!("SWEP-STEP-TBL-{:03}", t),
            label: format!("Create table {} of {}", t + 1, table_count),
            status: SchemaWriteExecutionPreviewStepStatus::Pending,
            note: "Would call Airtable create-table endpoint. Disabled — no network call made."
                .to_string(),
        });
        idx += 1;
    }

    // Phase 3 — create direct fields (aggregated, not per-field)
    if direct_field_count > 0 {
        steps.push(SchemaWriteExecutionPreviewStep {
            step_index: idx,
            step_id: "SWEP-STEP-FLD-DIRECT".to_string(),
            label: format!("Create {} direct field(s)", direct_field_count),
            status: SchemaWriteExecutionPreviewStepStatus::Pending,
            note: "Would call Airtable create-field endpoint for each directly-creatable field. Disabled — no network calls made.".to_string(),
        });
        idx += 1;
    }

    // Phase 4 — deferred linked fields
    if deferred_field_count > 0 {
        steps.push(SchemaWriteExecutionPreviewStep {
            step_index: idx,
            step_id: "SWEP-STEP-FLD-DEFERRED".to_string(),
            label: format!("Defer {} linked field(s) to second pass", deferred_field_count),
            status: SchemaWriteExecutionPreviewStepStatus::Pending,
            note: "Linked fields require all tables to exist first. Would be created in a second pass. Disabled — no network calls made.".to_string(),
        });
        idx += 1;
    }

    // Phase 5 — manual action fields
    if manual_count > 0 {
        steps.push(SchemaWriteExecutionPreviewStep {
            step_index: idx,
            step_id: "SWEP-STEP-MANUAL".to_string(),
            label: format!("{} field(s) require manual action", manual_count),
            status: SchemaWriteExecutionPreviewStepStatus::Pending,
            note: "Computed, collaborator, and unsupported fields cannot be created via the API and require manual setup.".to_string(),
        });
        idx += 1;
    }

    // Phase 6 — post-schema safety check
    steps.push(SchemaWriteExecutionPreviewStep {
        step_index: idx,
        step_id: "SWEP-STEP-POST".to_string(),
        label: "Post-schema safety verification".to_string(),
        status: SchemaWriteExecutionPreviewStepStatus::Pending,
        note: "Would verify that created schema matches the backup plan before proceeding. Disabled — no API calls made.".to_string(),
    });

    steps
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_request() -> SchemaWriteExecutionPreviewRequest {
        SchemaWriteExecutionPreviewRequest {
            package_filename: Some("test-backup.airbridge".to_string()),
            sandbox_flag_present: Some(true),
            target_empty_verified: Some(true),
            schema_plan_ready: Some(true),
            table_count: Some(2),
            direct_field_count: Some(4),
            deferred_field_count: Some(1),
            manual_action_count: Some(0),
            destructive_policy_safe: Some(true),
            sensitive_data_safe: Some(true),
            attachment_phase_disabled: Some(true),
            final_validation_enforcement_present: Some(true),
            live_write_readiness_satisfied: Some(true),
        }
    }

    fn missing_request() -> SchemaWriteExecutionPreviewRequest {
        SchemaWriteExecutionPreviewRequest {
            package_filename: None,
            sandbox_flag_present: None,
            target_empty_verified: None,
            schema_plan_ready: None,
            table_count: None,
            direct_field_count: None,
            deferred_field_count: None,
            manual_action_count: None,
            destructive_policy_safe: None,
            sensitive_data_safe: None,
            attachment_phase_disabled: None,
            final_validation_enforcement_present: None,
            live_write_readiness_satisfied: None,
        }
    }

    // ── Basic safety invariants ────────────────────────────────────────────────

    #[test]
    fn writes_enabled_always_false_for_safe_request() {
        let result = preview_schema_write_execution(&safe_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_for_safe_request() {
        let result = preview_schema_write_execution(&safe_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_for_safe_request() {
        let result = preview_schema_write_execution(&safe_request());
        assert!(!result.network_writes_attempted);
    }

    #[test]
    fn writes_enabled_always_false_for_missing_request() {
        let result = preview_schema_write_execution(&missing_request());
        assert!(!result.writes_enabled);
    }

    #[test]
    fn no_changes_made_always_true_for_missing_request() {
        let result = preview_schema_write_execution(&missing_request());
        assert!(result.no_changes_made);
    }

    #[test]
    fn network_writes_attempted_always_false_for_missing_request() {
        let result = preview_schema_write_execution(&missing_request());
        assert!(!result.network_writes_attempted);
    }

    // ── Blocked when prerequisites missing ────────────────────────────────────

    #[test]
    fn missing_all_prerequisites_returns_blocked() {
        let result = preview_schema_write_execution(&missing_request());
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn missing_schema_plan_returns_blocked() {
        let mut req = safe_request();
        req.schema_plan_ready = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn missing_sandbox_flag_returns_blocked() {
        let mut req = safe_request();
        req.sandbox_flag_present = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn missing_target_empty_returns_blocked() {
        let mut req = safe_request();
        req.target_empty_verified = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn unsafe_destructive_policy_returns_blocked() {
        let mut req = safe_request();
        req.destructive_policy_safe = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn sensitive_data_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.sensitive_data_safe = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn attachment_phase_unsafe_returns_blocked() {
        let mut req = safe_request();
        req.attachment_phase_disabled = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn final_validation_enforcement_missing_returns_blocked() {
        let mut req = safe_request();
        req.final_validation_enforcement_present = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    #[test]
    fn live_write_readiness_missing_returns_blocked() {
        let mut req = safe_request();
        req.live_write_readiness_satisfied = Some(false);
        let result = preview_schema_write_execution(&req);
        assert_eq!(result.status, SchemaWriteExecutionPreviewStatus::Blocked);
    }

    // ── DryRunReady for safe request ───────────────────────────────────────────

    #[test]
    fn safe_request_returns_dry_run_ready() {
        let result = preview_schema_write_execution(&safe_request());
        assert_eq!(
            result.status,
            SchemaWriteExecutionPreviewStatus::DryRunReady
        );
    }

    #[test]
    fn dry_run_ready_mode_is_dry_run_only() {
        let result = preview_schema_write_execution(&safe_request());
        assert_eq!(result.mode, SchemaWriteExecutionPreviewMode::DryRunOnly);
    }

    #[test]
    fn dry_run_ready_has_no_blocked_reason() {
        let result = preview_schema_write_execution(&safe_request());
        assert!(result.blocked_reason.is_none());
    }

    #[test]
    fn dry_run_ready_write_gate_disabled_in_snapshot() {
        let result = preview_schema_write_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── Step ordering ──────────────────────────────────────────────────────────

    #[test]
    fn step_ordering_is_deterministic() {
        let r1 = preview_schema_write_execution(&safe_request());
        let r2 = preview_schema_write_execution(&safe_request());
        let ids1: Vec<_> = r1.steps.iter().map(|s| &s.step_id).collect();
        let ids2: Vec<_> = r2.steps.iter().map(|s| &s.step_id).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn step_indices_are_sequential() {
        let result = preview_schema_write_execution(&safe_request());
        for (i, step) in result.steps.iter().enumerate() {
            assert_eq!(step.step_index, i, "step_index must be sequential");
        }
    }

    #[test]
    fn table_steps_come_before_field_steps() {
        let result = preview_schema_write_execution(&safe_request());
        let tbl_last = result
            .steps
            .iter()
            .filter(|s| s.step_id.starts_with("SWEP-STEP-TBL"))
            .map(|s| s.step_index)
            .max()
            .unwrap_or(0);
        let fld_first = result
            .steps
            .iter()
            .find(|s| s.step_id == "SWEP-STEP-FLD-DIRECT")
            .map(|s| s.step_index)
            .unwrap_or(usize::MAX);
        assert!(tbl_last < fld_first, "table steps must precede field steps");
    }

    #[test]
    fn step_counts_match_request_counts() {
        let result = preview_schema_write_execution(&safe_request());
        assert_eq!(result.table_step_count, 2);
        assert_eq!(result.field_step_count, 4);
        assert_eq!(result.deferred_step_count, 1);
        assert_eq!(result.manual_step_count, 0);
    }

    #[test]
    fn total_step_count_equals_steps_len() {
        let result = preview_schema_write_execution(&safe_request());
        assert_eq!(result.total_step_count, result.steps.len());
    }

    // ── Safety serialization checks ────────────────────────────────────────────

    #[test]
    fn no_token_in_dry_run_ready_serialization() {
        let result = preview_schema_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
        assert!(!json.contains("pat_"));
    }

    #[test]
    fn no_absolute_path_in_dry_run_ready_serialization() {
        let result = preview_schema_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn no_succeeded_in_dry_run_ready_serialization() {
        let result = preview_schema_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn no_attachment_url_in_serialization() {
        let result = preview_schema_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("cdn.airtable.com"));
        assert!(!json.contains("attachmentUrl"));
    }

    #[test]
    fn no_record_payload_in_serialization() {
        let result = preview_schema_write_execution(&safe_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"fields\":{"));
        assert!(!json.contains("\"records\":["));
    }

    #[test]
    fn dry_run_ready_message_does_not_imply_restore_complete() {
        let result = preview_schema_write_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(!lower.contains("restore complete"));
        assert!(!lower.contains("restore succeeded"));
        assert!(!lower.contains("restore success"));
    }

    #[test]
    fn dry_run_ready_message_states_writes_remain_disabled() {
        let result = preview_schema_write_execution(&safe_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("disabled"));
    }

    // ── Write gate not bypassed ────────────────────────────────────────────────

    #[test]
    fn write_gate_not_bypassed_by_preview() {
        let gate_before = evaluate_write_gate();
        let _result = preview_schema_write_execution(&safe_request());
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

    #[test]
    fn write_gate_snapshot_always_disabled() {
        let result = preview_schema_write_execution(&safe_request());
        assert!(result.safety_snapshot.write_gate_disabled);
    }

    // ── Blocked mode ───────────────────────────────────────────────────────────

    #[test]
    fn blocked_result_has_blocked_reason() {
        let result = preview_schema_write_execution(&missing_request());
        assert!(result.blocked_reason.is_some());
    }

    #[test]
    fn blocked_result_mode_is_live_blocked() {
        let result = preview_schema_write_execution(&missing_request());
        assert_eq!(result.mode, SchemaWriteExecutionPreviewMode::LiveBlocked);
    }

    #[test]
    fn blocked_result_steps_show_blocked_step() {
        let result = preview_schema_write_execution(&missing_request());
        assert!(!result.steps.is_empty());
        assert_eq!(
            result.steps[0].status,
            SchemaWriteExecutionPreviewStepStatus::Blocked
        );
    }

    #[test]
    fn blocked_result_message_contains_live_writes_disabled() {
        let result = preview_schema_write_execution(&missing_request());
        let lower = result.message.to_lowercase();
        assert!(lower.contains("disabled"));
    }

    // ── No success state ───────────────────────────────────────────────────────

    #[test]
    fn no_success_state_introduced() {
        let result = preview_schema_write_execution(&safe_request());
        assert_ne!(
            result.status,
            // DryRunReady is NOT a success state — but assert writes_enabled=false
            // as the definitive check
            SchemaWriteExecutionPreviewStatus::Blocked
        );
        assert!(!result.writes_enabled);
        // DryRunReady must not imply completion
        assert!(result
            .message
            .to_lowercase()
            .contains("does not start any restore execution"));
    }
}
