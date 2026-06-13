use serde::{Deserialize, Serialize};

use crate::restore::write_result::{
    RestoreWriteEngineStatus, RestoreWritePhase, RestoreWritePhaseSummary,
};

/// A summary of what the schema creation phase *would* do.
///
/// Derived from an existing schema plan. No Airtable calls. No writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaWriteSkeletonPlan {
    pub table_count: usize,
    pub direct_field_count: usize,
    pub deferred_field_count: usize,
    pub manual_action_count: usize,
    pub unsupported_count: usize,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    pub status: RestoreWriteEngineStatus,
    pub note: String,
}

/// Builds the schema write skeleton summary from counts.
///
/// Does not create any base, table, or field.
/// Does not call the Airtable API.
/// Does not require a token.
pub fn build_schema_write_skeleton(
    table_count: usize,
    direct_field_count: usize,
    deferred_field_count: usize,
    manual_action_count: usize,
    unsupported_count: usize,
) -> SchemaWriteSkeletonPlan {
    SchemaWriteSkeletonPlan {
        table_count,
        direct_field_count,
        deferred_field_count,
        manual_action_count,
        unsupported_count,
        no_changes_made: true,
        status: RestoreWriteEngineStatus::Disabled,
        note: "Schema creation is not executed in this version. The write engine is disabled."
            .to_string(),
    }
}

/// Converts the schema skeleton into a phase summary for the engine result.
pub fn schema_skeleton_phase_summary(
    skeleton: &SchemaWriteSkeletonPlan,
) -> RestoreWritePhaseSummary {
    RestoreWritePhaseSummary {
        phase: RestoreWritePhase::SchemaCreation,
        status: RestoreWriteEngineStatus::Disabled,
        no_changes_made: true,
        note: format!(
            "Schema creation disabled. Would create {} table(s), {} direct field(s), {} deferred field(s). {} field(s) require manual action.",
            skeleton.table_count,
            skeleton.direct_field_count,
            skeleton.deferred_field_count,
            skeleton.manual_action_count,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skeleton() -> SchemaWriteSkeletonPlan {
        build_schema_write_skeleton(3, 12, 2, 1, 0)
    }

    #[test]
    fn schema_skeleton_status_is_disabled() {
        let s = sample_skeleton();
        assert_eq!(s.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn schema_skeleton_no_changes_made_is_true() {
        let s = sample_skeleton();
        assert!(s.no_changes_made);
    }

    #[test]
    fn schema_skeleton_summarizes_table_count() {
        let s = build_schema_write_skeleton(5, 20, 3, 2, 1);
        assert_eq!(s.table_count, 5);
    }

    #[test]
    fn schema_skeleton_summarizes_direct_field_count() {
        let s = build_schema_write_skeleton(5, 20, 3, 2, 1);
        assert_eq!(s.direct_field_count, 20);
    }

    #[test]
    fn schema_skeleton_summarizes_deferred_field_count() {
        let s = build_schema_write_skeleton(5, 20, 3, 2, 1);
        assert_eq!(s.deferred_field_count, 3);
    }

    #[test]
    fn schema_skeleton_summarizes_manual_action_count() {
        let s = build_schema_write_skeleton(5, 20, 3, 2, 1);
        assert_eq!(s.manual_action_count, 2);
    }

    #[test]
    fn schema_skeleton_zero_counts_are_valid() {
        let s = build_schema_write_skeleton(0, 0, 0, 0, 0);
        assert_eq!(s.table_count, 0);
        assert!(s.no_changes_made);
        assert_eq!(s.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn schema_skeleton_phase_summary_has_schema_creation_phase() {
        let s = sample_skeleton();
        let summary = schema_skeleton_phase_summary(&s);
        assert_eq!(summary.phase, RestoreWritePhase::SchemaCreation);
    }

    #[test]
    fn schema_skeleton_phase_summary_is_disabled() {
        let s = sample_skeleton();
        let summary = schema_skeleton_phase_summary(&s);
        assert_eq!(summary.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn schema_skeleton_phase_summary_no_changes_made_is_true() {
        let s = sample_skeleton();
        let summary = schema_skeleton_phase_summary(&s);
        assert!(summary.no_changes_made);
    }

    #[test]
    fn schema_skeleton_serialization_has_no_succeeded_status() {
        let s = sample_skeleton();
        let json = serde_json::to_string(&s).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn schema_skeleton_serialization_has_no_token() {
        let s = sample_skeleton();
        let json = serde_json::to_string(&s).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }
}
