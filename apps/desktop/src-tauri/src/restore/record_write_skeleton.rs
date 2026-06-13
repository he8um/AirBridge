use serde::{Deserialize, Serialize};

use crate::restore::record_import_plan::RestoreRecordMappingStrategy;
use crate::restore::write_result::{
    RestoreWriteEngineStatus, RestoreWritePhase, RestoreWritePhaseSummary,
};

/// A summary of what the record import phase *would* do.
///
/// Derived from an existing record import plan. No Airtable calls. No writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWriteSkeletonPlan {
    pub estimated_first_pass_batches: usize,
    pub estimated_second_pass_batches: usize,
    pub linked_record_update_count: usize,
    pub mapping_strategy: RestoreRecordMappingStrategy,
    /// Short description of the attachment policy applied to all tables.
    pub attachment_policy_summary: String,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    pub status: RestoreWriteEngineStatus,
    pub note: String,
}

/// Builds the record write skeleton summary from plan counts.
///
/// Does not create or update any Airtable records.
/// Does not download or upload attachment files.
/// Does not require a token.
pub fn build_record_write_skeleton(
    estimated_first_pass_batches: usize,
    estimated_second_pass_batches: usize,
    linked_record_update_count: usize,
    mapping_strategy: RestoreRecordMappingStrategy,
    attachment_policy_summary: String,
) -> RecordWriteSkeletonPlan {
    RecordWriteSkeletonPlan {
        estimated_first_pass_batches,
        estimated_second_pass_batches,
        linked_record_update_count,
        mapping_strategy,
        attachment_policy_summary,
        no_changes_made: true,
        status: RestoreWriteEngineStatus::Disabled,
        note: "Record import is not executed in this version. The write engine is disabled."
            .to_string(),
    }
}

/// Converts the record skeleton into phase summaries for the engine result.
pub fn record_skeleton_phase_summaries(
    skeleton: &RecordWriteSkeletonPlan,
) -> Vec<RestoreWritePhaseSummary> {
    vec![
        RestoreWritePhaseSummary {
            phase: RestoreWritePhase::RecordCreation,
            status: RestoreWriteEngineStatus::Disabled,
            no_changes_made: true,
            note: format!(
                "Record import disabled. Would run {} first-pass create batch(es) at batch size 10.",
                skeleton.estimated_first_pass_batches,
            ),
        },
        RestoreWritePhaseSummary {
            phase: RestoreWritePhase::LinkedRecordUpdates,
            status: RestoreWriteEngineStatus::Disabled,
            no_changes_made: true,
            note: format!(
                "Linked record updates disabled. Would run {} second-pass update batch(es) for {} linked record field(s).",
                skeleton.estimated_second_pass_batches,
                skeleton.linked_record_update_count,
            ),
        },
        RestoreWritePhaseSummary {
            phase: RestoreWritePhase::AttachmentHandling,
            status: RestoreWriteEngineStatus::Disabled,
            no_changes_made: true,
            note: format!(
                "Attachment handling disabled. Policy: {}.",
                skeleton.attachment_policy_summary,
            ),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skeleton() -> RecordWriteSkeletonPlan {
        build_record_write_skeleton(
            4,
            2,
            3,
            RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId,
            "MetadataOnly — attachment file bytes are not uploaded".to_string(),
        )
    }

    #[test]
    fn record_skeleton_status_is_disabled() {
        let s = sample_skeleton();
        assert_eq!(s.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn record_skeleton_no_changes_made_is_true() {
        let s = sample_skeleton();
        assert!(s.no_changes_made);
    }

    #[test]
    fn record_skeleton_summarizes_first_pass_batch_count() {
        let s = sample_skeleton();
        assert_eq!(s.estimated_first_pass_batches, 4);
    }

    #[test]
    fn record_skeleton_summarizes_second_pass_batch_count() {
        let s = sample_skeleton();
        assert_eq!(s.estimated_second_pass_batches, 2);
    }

    #[test]
    fn record_skeleton_summarizes_linked_record_update_count() {
        let s = sample_skeleton();
        assert_eq!(s.linked_record_update_count, 3);
    }

    #[test]
    fn record_skeleton_summarizes_mapping_strategy() {
        let s = sample_skeleton();
        assert_eq!(
            s.mapping_strategy,
            RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId
        );
    }

    #[test]
    fn record_skeleton_zero_counts_are_valid() {
        let s = build_record_write_skeleton(
            0,
            0,
            0,
            RestoreRecordMappingStrategy::UnavailableUntilExecution,
            "none".to_string(),
        );
        assert_eq!(s.estimated_first_pass_batches, 0);
        assert!(s.no_changes_made);
        assert_eq!(s.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn record_skeleton_phase_summaries_include_record_creation() {
        let s = sample_skeleton();
        let summaries = record_skeleton_phase_summaries(&s);
        assert!(summaries
            .iter()
            .any(|p| p.phase == RestoreWritePhase::RecordCreation));
    }

    #[test]
    fn record_skeleton_phase_summaries_include_linked_record_updates() {
        let s = sample_skeleton();
        let summaries = record_skeleton_phase_summaries(&s);
        assert!(summaries
            .iter()
            .any(|p| p.phase == RestoreWritePhase::LinkedRecordUpdates));
    }

    #[test]
    fn record_skeleton_phase_summaries_include_attachment_handling() {
        let s = sample_skeleton();
        let summaries = record_skeleton_phase_summaries(&s);
        assert!(summaries
            .iter()
            .any(|p| p.phase == RestoreWritePhase::AttachmentHandling));
    }

    #[test]
    fn record_skeleton_phase_summaries_all_disabled() {
        let s = sample_skeleton();
        let summaries = record_skeleton_phase_summaries(&s);
        for summary in &summaries {
            assert_eq!(summary.status, RestoreWriteEngineStatus::Disabled);
            assert!(summary.no_changes_made);
        }
    }

    #[test]
    fn record_skeleton_serialization_has_no_succeeded_status() {
        let s = sample_skeleton();
        let json = serde_json::to_string(&s).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("Succeeded"));
    }

    #[test]
    fn record_skeleton_serialization_has_no_token() {
        let s = sample_skeleton();
        let json = serde_json::to_string(&s).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }
}
