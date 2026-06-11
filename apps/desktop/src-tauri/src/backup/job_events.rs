use serde::{Deserialize, Serialize};

use crate::backup::job::{BackupJobId, BackupJobPhase};

/// A single progress event emitted during a backup job.
///
/// Events must not contain tokens, absolute user-local paths,
/// or full attachment URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum BackupJobEvent {
    JobStarted {
        job_id: BackupJobId,
        base_id: String,
        base_name: String,
        table_count: usize,
    },
    PhaseStarted {
        job_id: BackupJobId,
        phase: BackupJobPhase,
    },
    TableExportStarted {
        job_id: BackupJobId,
        table_id: String,
        table_name: String,
    },
    TableExportCompleted {
        job_id: BackupJobId,
        table_id: String,
        table_name: String,
        record_count: usize,
        pages_fetched: usize,
    },
    PackageWriteStarted {
        job_id: BackupJobId,
    },
    PackageWriteCompleted {
        job_id: BackupJobId,
        entry_count: usize,
    },
    ValidationStarted {
        job_id: BackupJobId,
    },
    ValidationCompleted {
        job_id: BackupJobId,
        /// "valid", "warning", or "invalid"
        status: String,
        error_count: usize,
        warning_count: usize,
    },
    JobSucceeded {
        job_id: BackupJobId,
        total_records: usize,
        table_count: usize,
    },
    JobFailed {
        job_id: BackupJobId,
        error_code: String,
        /// Sanitised message — no token, no absolute paths.
        message: String,
    },
    JobCancelled {
        job_id: BackupJobId,
        at_phase: BackupJobPhase,
    },
}

impl BackupJobEvent {
    /// Returns the job_id associated with this event.
    pub fn job_id(&self) -> &BackupJobId {
        match self {
            BackupJobEvent::JobStarted { job_id, .. } => job_id,
            BackupJobEvent::PhaseStarted { job_id, .. } => job_id,
            BackupJobEvent::TableExportStarted { job_id, .. } => job_id,
            BackupJobEvent::TableExportCompleted { job_id, .. } => job_id,
            BackupJobEvent::PackageWriteStarted { job_id } => job_id,
            BackupJobEvent::PackageWriteCompleted { job_id, .. } => job_id,
            BackupJobEvent::ValidationStarted { job_id } => job_id,
            BackupJobEvent::ValidationCompleted { job_id, .. } => job_id,
            BackupJobEvent::JobSucceeded { job_id, .. } => job_id,
            BackupJobEvent::JobFailed { job_id, .. } => job_id,
            BackupJobEvent::JobCancelled { job_id, .. } => job_id,
        }
    }

    /// Returns a short string identifying the event kind (for logging/ordering assertions).
    pub fn kind_str(&self) -> &'static str {
        match self {
            BackupJobEvent::JobStarted { .. } => "jobStarted",
            BackupJobEvent::PhaseStarted { .. } => "phaseStarted",
            BackupJobEvent::TableExportStarted { .. } => "tableExportStarted",
            BackupJobEvent::TableExportCompleted { .. } => "tableExportCompleted",
            BackupJobEvent::PackageWriteStarted { .. } => "packageWriteStarted",
            BackupJobEvent::PackageWriteCompleted { .. } => "packageWriteCompleted",
            BackupJobEvent::ValidationStarted { .. } => "validationStarted",
            BackupJobEvent::ValidationCompleted { .. } => "validationCompleted",
            BackupJobEvent::JobSucceeded { .. } => "jobSucceeded",
            BackupJobEvent::JobFailed { .. } => "jobFailed",
            BackupJobEvent::JobCancelled { .. } => "jobCancelled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job_id() -> BackupJobId {
        BackupJobId("job-syn-001".to_string())
    }

    #[test]
    fn job_started_serializes_with_tag() {
        let ev = BackupJobEvent::JobStarted {
            job_id: job_id(),
            base_id: "appSyn01".to_string(),
            base_name: "Synthetic".to_string(),
            table_count: 2,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("\"kind\":\"jobStarted\""));
        assert!(json.contains("appSyn01"));
    }

    #[test]
    fn phase_started_serializes_phase() {
        let ev = BackupJobEvent::PhaseStarted {
            job_id: job_id(),
            phase: BackupJobPhase::RecordsExport,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("recordsExport"));
    }

    #[test]
    fn table_export_completed_serializes_counts() {
        let ev = BackupJobEvent::TableExportCompleted {
            job_id: job_id(),
            table_id: "tbl01".to_string(),
            table_name: "Projects".to_string(),
            record_count: 42,
            pages_fetched: 1,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("42"));
        assert!(json.contains("tbl01"));
    }

    #[test]
    fn package_write_completed_serializes_entry_count() {
        let ev = BackupJobEvent::PackageWriteCompleted {
            job_id: job_id(),
            entry_count: 8,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("8"));
    }

    #[test]
    fn validation_completed_serializes_status() {
        let ev = BackupJobEvent::ValidationCompleted {
            job_id: job_id(),
            status: "valid".to_string(),
            error_count: 0,
            warning_count: 0,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("valid"));
    }

    #[test]
    fn job_failed_sanitized_message_no_token() {
        const SENTINEL: &str = "pat_event_test_sentinel_0123456789";
        let ev = BackupJobEvent::JobFailed {
            job_id: job_id(),
            error_code: "AUTH_FAILED".to_string(),
            message: "authentication failed".to_string(),
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(!json.contains(SENTINEL));
        assert!(!json.contains("/Users/"));
    }

    #[test]
    fn job_cancelled_includes_at_phase() {
        let ev = BackupJobEvent::JobCancelled {
            job_id: job_id(),
            at_phase: BackupJobPhase::RecordsExport,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("jobCancelled"));
        assert!(json.contains("recordsExport"));
    }

    #[test]
    fn job_succeeded_serializes() {
        let ev = BackupJobEvent::JobSucceeded {
            job_id: job_id(),
            total_records: 100,
            table_count: 2,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("jobSucceeded"));
        assert!(json.contains("100"));
    }

    #[test]
    fn all_events_have_job_id() {
        let events = vec![
            BackupJobEvent::JobStarted {
                job_id: job_id(),
                base_id: "app01".to_string(),
                base_name: "B".to_string(),
                table_count: 1,
            },
            BackupJobEvent::PhaseStarted {
                job_id: job_id(),
                phase: BackupJobPhase::Planning,
            },
            BackupJobEvent::TableExportStarted {
                job_id: job_id(),
                table_id: "tbl01".to_string(),
                table_name: "T".to_string(),
            },
            BackupJobEvent::TableExportCompleted {
                job_id: job_id(),
                table_id: "tbl01".to_string(),
                table_name: "T".to_string(),
                record_count: 0,
                pages_fetched: 1,
            },
            BackupJobEvent::PackageWriteStarted { job_id: job_id() },
            BackupJobEvent::PackageWriteCompleted {
                job_id: job_id(),
                entry_count: 5,
            },
            BackupJobEvent::ValidationStarted { job_id: job_id() },
            BackupJobEvent::ValidationCompleted {
                job_id: job_id(),
                status: "valid".to_string(),
                error_count: 0,
                warning_count: 0,
            },
            BackupJobEvent::JobSucceeded {
                job_id: job_id(),
                total_records: 0,
                table_count: 0,
            },
            BackupJobEvent::JobFailed {
                job_id: job_id(),
                error_code: "ERR".to_string(),
                message: "failed".to_string(),
            },
            BackupJobEvent::JobCancelled {
                job_id: job_id(),
                at_phase: BackupJobPhase::RecordsExport,
            },
        ];

        for ev in &events {
            assert_eq!(ev.job_id().as_str(), "job-syn-001");
        }
    }

    #[test]
    fn kind_str_matches_expected_values() {
        assert_eq!(
            BackupJobEvent::JobStarted {
                job_id: job_id(),
                base_id: "a".to_string(),
                base_name: "b".to_string(),
                table_count: 0,
            }
            .kind_str(),
            "jobStarted"
        );
        assert_eq!(
            BackupJobEvent::PackageWriteStarted { job_id: job_id() }.kind_str(),
            "packageWriteStarted"
        );
        assert_eq!(
            BackupJobEvent::ValidationCompleted {
                job_id: job_id(),
                status: "valid".to_string(),
                error_count: 0,
                warning_count: 0,
            }
            .kind_str(),
            "validationCompleted"
        );
    }

    #[test]
    fn no_event_contains_absolute_path() {
        let events = vec![
            BackupJobEvent::JobFailed {
                job_id: job_id(),
                error_code: "ERR".to_string(),
                message: "failed".to_string(),
            },
            BackupJobEvent::JobSucceeded {
                job_id: job_id(),
                total_records: 5,
                table_count: 1,
            },
        ];
        for ev in &events {
            let json = serde_json::to_string(ev).expect("serialize");
            assert!(!json.contains("/Users/"), "event contains absolute path");
            assert!(!json.contains("/home/"), "event contains home path");
        }
    }
}
