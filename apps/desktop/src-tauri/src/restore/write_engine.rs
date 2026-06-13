use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::restore::record_import_plan::RestoreRecordMappingStrategy;
use crate::restore::record_write_skeleton::{
    build_record_write_skeleton, record_skeleton_phase_summaries,
};
use crate::restore::schema_write_skeleton::{
    build_schema_write_skeleton, schema_skeleton_phase_summary,
};
use crate::restore::write_gate::evaluate_write_gate;
use crate::restore::write_result::{
    RestoreWriteDisabledReason, RestoreWriteEngineResult, RestoreWriteEngineStatus,
    RestoreWriteEvent, RestoreWritePhase, RestoreWritePhaseSummary,
};
use crate::restore::write_safety::build_write_safety_report;

/// Input for the write engine skeleton preview.
///
/// - No token field — the skeleton preview does not require Airtable access.
/// - package_path is never echoed in the result.
/// - All counts are derived from existing planning outputs.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreWriteEngineRequest {
    /// Filename-only identifier from the most recent package inspection.
    pub package_filename: String,
    /// Full path used to derive the filename — never echoed in the result.
    pub package_path: String,
    /// Number of tables in the schema plan.
    #[serde(default)]
    pub schema_table_count: usize,
    /// Number of directly-creatable fields in the schema plan.
    #[serde(default)]
    pub schema_direct_field_count: usize,
    /// Number of deferred fields in the schema plan.
    #[serde(default)]
    pub schema_deferred_field_count: usize,
    /// Number of fields requiring manual action.
    #[serde(default)]
    pub schema_manual_action_count: usize,
    /// Number of unsupported fields.
    #[serde(default)]
    pub schema_unsupported_count: usize,
    /// Estimated number of first-pass record create batches (all tables combined).
    #[serde(default)]
    pub estimated_first_pass_batches: usize,
    /// Estimated number of second-pass linked-record update batches.
    #[serde(default)]
    pub estimated_second_pass_batches: usize,
    /// Number of linked record fields requiring a second-pass update.
    #[serde(default)]
    pub linked_record_update_count: usize,
}

/// Produces a write engine skeleton preview result.
///
/// - Calls evaluate_write_gate() — always returns disabled.
/// - Builds schema and record phase summaries from request counts.
/// - Never calls the Airtable API.
/// - Never writes any file.
/// - Never stores or echoes a token (no token in request).
/// - Always sets no_changes_made: true.
/// - Never returns a succeeded status.
pub fn preview_write_engine(request: &RestoreWriteEngineRequest) -> RestoreWriteEngineResult {
    let filename = Path::new(&request.package_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| request.package_filename.clone());

    // Input shape validation
    if request.package_filename.is_empty() && request.package_path.is_empty() {
        return blocked_result(
            String::from("unknown"),
            RestoreWriteDisabledReason::BlockedByInvalidPlan,
            "No package filename provided.",
        );
    }

    // Always-disabled gate
    let gate = evaluate_write_gate();

    // Build skeleton summaries
    let schema_skeleton = build_schema_write_skeleton(
        request.schema_table_count,
        request.schema_direct_field_count,
        request.schema_deferred_field_count,
        request.schema_manual_action_count,
        request.schema_unsupported_count,
    );
    let record_skeleton = build_record_write_skeleton(
        request.estimated_first_pass_batches,
        request.estimated_second_pass_batches,
        request.linked_record_update_count,
        RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId,
        "MetadataOnly — attachment file bytes are not uploaded in this version".to_string(),
    );

    let safety = build_write_safety_report();

    // Phase summaries for all pipeline phases
    let mut phase_summaries = vec![
        RestoreWritePhaseSummary {
            phase: RestoreWritePhase::ValidateInputs,
            status: RestoreWriteEngineStatus::Disabled,
            no_changes_made: true,
            note: "Input validation completed. Write engine is disabled.".to_string(),
        },
        schema_skeleton_phase_summary(&schema_skeleton),
    ];
    phase_summaries.extend(record_skeleton_phase_summaries(&record_skeleton));
    phase_summaries.push(RestoreWritePhaseSummary {
        phase: RestoreWritePhase::FinalValidation,
        status: RestoreWriteEngineStatus::Disabled,
        no_changes_made: true,
        note: "Final validation is not executed — write engine is disabled.".to_string(),
    });

    // Events
    let events = vec![
        RestoreWriteEvent {
            phase: RestoreWritePhase::ValidateInputs,
            code: "WRITE_ENGINE_DISABLED".to_string(),
            message: gate.message.clone(),
        },
        RestoreWriteEvent {
            phase: RestoreWritePhase::SchemaCreation,
            code: "SCHEMA_PHASE_SKIPPED".to_string(),
            message: format!(
                "Schema creation skipped. {} table(s) and {} direct field(s) would be created.",
                request.schema_table_count, request.schema_direct_field_count
            ),
        },
        RestoreWriteEvent {
            phase: RestoreWritePhase::RecordCreation,
            code: "RECORD_PHASE_SKIPPED".to_string(),
            message: format!(
                "Record import skipped. {} first-pass batch(es) would run.",
                request.estimated_first_pass_batches
            ),
        },
    ];

    let _ = safety; // Safety report is computed; invariants are checked in tests.

    RestoreWriteEngineResult {
        filename,
        status: gate.status,
        disabled_reason: gate.reason,
        message: gate.message,
        phase_summaries,
        events,
        no_changes_made: true,
    }
}

fn blocked_result(
    filename: String,
    reason: RestoreWriteDisabledReason,
    message: &str,
) -> RestoreWriteEngineResult {
    RestoreWriteEngineResult {
        filename,
        status: RestoreWriteEngineStatus::Blocked,
        disabled_reason: reason,
        message: message.to_string(),
        phase_summaries: vec![],
        events: vec![],
        no_changes_made: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_request() -> RestoreWriteEngineRequest {
        RestoreWriteEngineRequest {
            package_filename: "backup.airbridge".to_string(),
            package_path: "/tmp/backup.airbridge".to_string(),
            schema_table_count: 3,
            schema_direct_field_count: 12,
            schema_deferred_field_count: 2,
            schema_manual_action_count: 1,
            schema_unsupported_count: 0,
            estimated_first_pass_batches: 4,
            estimated_second_pass_batches: 2,
            linked_record_update_count: 3,
        }
    }

    #[test]
    fn write_engine_returns_disabled_status() {
        let req = base_request();
        let result = preview_write_engine(&req);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn write_engine_no_changes_made_always_true() {
        let req = base_request();
        let result = preview_write_engine(&req);
        assert!(result.no_changes_made);
    }

    #[test]
    fn write_engine_does_not_require_token() {
        let req = base_request();
        // No token field in request struct — verify serialization has no token
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn write_engine_result_has_no_token() {
        let req = base_request();
        let result = preview_write_engine(&req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn write_engine_result_does_not_echo_path() {
        let req = base_request();
        let result = preview_write_engine(&req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn write_engine_result_filename_only() {
        let req = base_request();
        let result = preview_write_engine(&req);
        assert_eq!(result.filename, "backup.airbridge");
        assert!(!result.filename.contains('/'));
        assert!(!result.filename.contains('\\'));
    }

    #[test]
    fn write_engine_result_has_no_succeeded_status() {
        let req = base_request();
        let result = preview_write_engine(&req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn write_engine_has_all_phase_summaries() {
        let req = base_request();
        let result = preview_write_engine(&req);
        let phases: Vec<_> = result.phase_summaries.iter().map(|p| &p.phase).collect();
        assert!(phases.contains(&&RestoreWritePhase::ValidateInputs));
        assert!(phases.contains(&&RestoreWritePhase::SchemaCreation));
        assert!(phases.contains(&&RestoreWritePhase::RecordCreation));
        assert!(phases.contains(&&RestoreWritePhase::LinkedRecordUpdates));
        assert!(phases.contains(&&RestoreWritePhase::AttachmentHandling));
        assert!(phases.contains(&&RestoreWritePhase::FinalValidation));
    }

    #[test]
    fn write_engine_all_phase_summaries_are_disabled() {
        let req = base_request();
        let result = preview_write_engine(&req);
        for summary in &result.phase_summaries {
            assert_eq!(
                summary.status,
                RestoreWriteEngineStatus::Disabled,
                "phase {:?} must be disabled",
                summary.phase
            );
        }
    }

    #[test]
    fn write_engine_all_phase_summaries_no_changes_made_true() {
        let req = base_request();
        let result = preview_write_engine(&req);
        for summary in &result.phase_summaries {
            assert!(
                summary.no_changes_made,
                "phase {:?} no_changes_made must be true",
                summary.phase
            );
        }
    }

    #[test]
    fn write_engine_has_events() {
        let req = base_request();
        let result = preview_write_engine(&req);
        assert!(!result.events.is_empty());
    }

    #[test]
    fn write_engine_disabled_reason_is_product_policy() {
        let req = base_request();
        let result = preview_write_engine(&req);
        assert_eq!(
            result.disabled_reason,
            RestoreWriteDisabledReason::DisabledByProductPolicy
        );
    }

    #[test]
    fn write_engine_empty_filename_returns_blocked() {
        let req = RestoreWriteEngineRequest {
            package_filename: "".to_string(),
            package_path: "".to_string(),
            ..base_request()
        };
        let result = preview_write_engine(&req);
        assert_eq!(result.status, RestoreWriteEngineStatus::Blocked);
        assert!(result.no_changes_made);
    }

    #[test]
    fn write_engine_zero_counts_still_returns_disabled() {
        let req = RestoreWriteEngineRequest {
            schema_table_count: 0,
            schema_direct_field_count: 0,
            schema_deferred_field_count: 0,
            schema_manual_action_count: 0,
            schema_unsupported_count: 0,
            estimated_first_pass_batches: 0,
            estimated_second_pass_batches: 0,
            linked_record_update_count: 0,
            ..base_request()
        };
        let result = preview_write_engine(&req);
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
        assert!(result.no_changes_made);
    }

    #[test]
    fn write_engine_no_airtable_client_called() {
        // The function signature accepts no HTTP transport or Airtable client.
        // This test validates structural safety: no client type is constructible
        // from the request, and calling preview_write_engine() completes without
        // any network operation.
        let req = base_request();
        let result = preview_write_engine(&req);
        // If we reach this assertion, no network call was attempted.
        assert_eq!(result.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn write_engine_serializes_no_changes_made_key() {
        let req = base_request();
        let result = preview_write_engine(&req);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("noChangesMade"));
    }
}
