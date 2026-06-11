use serde::{Deserialize, Serialize};

use crate::backup::job::{BackupJobId, BackupJobPhase, BackupJobStatus};

/// A read-only snapshot of a backup job's progress at a point in time.
///
/// Returned in the `job_result` after completion in the synchronous model.
/// In a future streaming model this would be emitted during execution.
///
/// Safe to serialise:
/// - No token.
/// - No absolute paths.
/// - No attachment URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobProgressSnapshot {
    pub job_id: BackupJobId,
    pub phase: BackupJobPhase,
    pub status: BackupJobStatus,
    /// Number of tables completed so far.
    pub completed_tables: usize,
    /// Total tables to export, if known at snapshot time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tables: Option<usize>,
    /// True when the total work is not yet known (e.g. record count unknown before first page).
    pub unknown_total: bool,
    /// The table currently being exported, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_table_id: Option<String>,
    /// Human-readable name of the current table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_table_name: Option<String>,
    /// Number of non-fatal warnings accumulated so far.
    pub warning_count: usize,
    /// Number of errors accumulated so far.
    pub error_count: usize,
}

/// Request to cancel a running backup job by ID.
///
/// In V0.1 there is no background job registry, so cancellation always
/// returns `not_running`. The struct exists for future wiring when a registry
/// is added.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobCancellationRequest {
    pub job_id: BackupJobId,
}

/// Result of a cancellation attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobCancellationResult {
    pub job_id: BackupJobId,
    /// True if the job was found and cancellation was signalled.
    /// False if the job was not running (no background registry in V0.1).
    pub was_running: bool,
    /// The status of the job at the moment cancellation was processed.
    /// Always `"not_running"` in V0.1.
    pub status_at_cancellation: String,
}

impl BackupJobCancellationResult {
    /// Placeholder for when no background registry exists.
    pub fn not_running(job_id: BackupJobId) -> Self {
        BackupJobCancellationResult {
            job_id,
            was_running: false,
            status_at_cancellation: "not_running".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job_id() -> BackupJobId {
        BackupJobId("job-progress-test-001".to_string())
    }

    #[test]
    fn progress_snapshot_serializes_with_phase() {
        let snap = BackupJobProgressSnapshot {
            job_id: job_id(),
            phase: BackupJobPhase::RecordsExport,
            status: BackupJobStatus::Running,
            completed_tables: 1,
            total_tables: Some(3),
            unknown_total: false,
            current_table_id: Some("tbl01".to_string()),
            current_table_name: Some("Projects".to_string()),
            warning_count: 0,
            error_count: 0,
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        assert!(json.contains("recordsExport"));
        assert!(json.contains("running"));
        assert!(json.contains("tbl01"));
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn progress_snapshot_unknown_total_omits_total_tables() {
        let snap = BackupJobProgressSnapshot {
            job_id: job_id(),
            phase: BackupJobPhase::RecordsExport,
            status: BackupJobStatus::Running,
            completed_tables: 0,
            total_tables: None,
            unknown_total: true,
            current_table_id: None,
            current_table_name: None,
            warning_count: 0,
            error_count: 0,
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        assert!(!json.contains("totalTables"));
        assert!(json.contains("\"unknownTotal\":true"));
    }

    #[test]
    fn progress_snapshot_no_token_sentinel() {
        const SENTINEL: &str = "pat_progress_test_sentinel_0123456789";
        let snap = BackupJobProgressSnapshot {
            job_id: job_id(),
            phase: BackupJobPhase::Planning,
            status: BackupJobStatus::Running,
            completed_tables: 0,
            total_tables: None,
            unknown_total: true,
            current_table_id: None,
            current_table_name: None,
            warning_count: 0,
            error_count: 0,
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn cancellation_request_serializes_job_id() {
        let req = BackupJobCancellationRequest { job_id: job_id() };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("job-progress-test-001"));
    }

    #[test]
    fn cancellation_result_not_running_constructor() {
        let result = BackupJobCancellationResult::not_running(job_id());
        assert!(!result.was_running);
        assert_eq!(result.status_at_cancellation, "not_running");
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("not_running"));
        assert!(json.contains("false"));
    }

    #[test]
    fn cancellation_result_no_token_or_path() {
        const SENTINEL: &str = "pat_cancel_test_sentinel_0123456789";
        let result = BackupJobCancellationResult::not_running(job_id());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn all_phases_serialize_in_snapshot() {
        let phases = [
            BackupJobPhase::Planning,
            BackupJobPhase::Schema,
            BackupJobPhase::RecordsExport,
            BackupJobPhase::PackageBuild,
            BackupJobPhase::Validation,
            BackupJobPhase::Completed,
        ];
        let expected = [
            "planning",
            "schema",
            "recordsExport",
            "packageBuild",
            "validation",
            "completed",
        ];
        for (phase, expected_str) in phases.iter().zip(expected.iter()) {
            let snap = BackupJobProgressSnapshot {
                job_id: job_id(),
                phase: phase.clone(),
                status: BackupJobStatus::Running,
                completed_tables: 0,
                total_tables: None,
                unknown_total: true,
                current_table_id: None,
                current_table_name: None,
                warning_count: 0,
                error_count: 0,
            };
            let json = serde_json::to_string(&snap).expect("serialize");
            assert!(
                json.contains(expected_str),
                "phase {expected_str} not found in JSON"
            );
        }
    }
}
