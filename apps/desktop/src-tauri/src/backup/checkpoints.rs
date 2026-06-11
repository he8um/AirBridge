use serde::{Deserialize, Serialize};

/// A checkpoint record for resuming a records export job.
///
/// Model-only — no persistence in this phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportCheckpointPlan {
    /// Unique ID for the backup job this checkpoint belongs to.
    pub backup_job_id: String,
    /// ID of the table currently being exported.
    pub table_id: String,
    /// Opaque pagination cursor from the last completed page, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_offset: Option<String>,
    /// Number of records exported so far for this table.
    pub records_exported: usize,
    /// ISO 8601 timestamp of the last checkpoint update.
    pub updated_at: String,
}

impl ExportCheckpointPlan {
    /// Creates a new checkpoint at the start of a table export (no offset yet).
    pub fn new(backup_job_id: &str, table_id: &str, updated_at: &str) -> Self {
        ExportCheckpointPlan {
            backup_job_id: backup_job_id.to_string(),
            table_id: table_id.to_string(),
            last_offset: None,
            records_exported: 0,
            updated_at: updated_at.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ExportCheckpointPlan {
        ExportCheckpointPlan {
            backup_job_id: "job-syn-001".to_string(),
            table_id: "tblSyn01".to_string(),
            last_offset: Some("iterXyz123".to_string()),
            records_exported: 200,
            updated_at: "2026-06-11T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn checkpoint_serializes_to_json() {
        let cp = sample();
        let json = serde_json::to_string(&cp).expect("serialize");
        assert!(json.contains("backupJobId"));
        assert!(json.contains("tableId"));
        assert!(json.contains("recordsExported"));
        assert!(json.contains("updatedAt"));
    }

    #[test]
    fn checkpoint_roundtrips_through_json() {
        let cp = sample();
        let json = serde_json::to_string(&cp).expect("serialize");
        let restored: ExportCheckpointPlan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.backup_job_id, cp.backup_job_id);
        assert_eq!(restored.table_id, cp.table_id);
        assert_eq!(restored.records_exported, cp.records_exported);
        assert_eq!(restored.updated_at, cp.updated_at);
        assert_eq!(restored.last_offset, cp.last_offset);
    }

    #[test]
    fn checkpoint_last_offset_is_optional() {
        let mut cp = sample();
        cp.last_offset = None;
        let json = serde_json::to_string(&cp).expect("serialize");
        // skip_serializing_if = None means it should not appear
        assert!(!json.contains("lastOffset"));
    }

    #[test]
    fn checkpoint_new_starts_with_no_offset_and_zero_records() {
        let cp = ExportCheckpointPlan::new("job-001", "tblSyn01", "2026-06-11T00:00:00Z");
        assert!(cp.last_offset.is_none());
        assert_eq!(cp.records_exported, 0);
        assert_eq!(cp.backup_job_id, "job-001");
    }

    #[test]
    fn checkpoint_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_checkpoint_test_sentinel_0123456789";
        let cp = sample();
        let json = serde_json::to_string(&cp).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }
}
