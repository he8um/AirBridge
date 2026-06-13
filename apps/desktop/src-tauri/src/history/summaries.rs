use crate::history::models::{
    JobHistoryError, JobHistoryId, JobHistoryItem, JobHistoryKind, JobHistorySource,
    JobHistoryStatus, JobHistorySummary, JobHistoryWarning,
};
use crate::history::redaction::redact_path_to_filename;

/// Generic builder for creating a history item from explicit safe fields.
/// All callers must pass `package_filename` as a filename only (no full path).
pub fn build_history_item(
    id: &str,
    kind: JobHistoryKind,
    status: JobHistoryStatus,
    source: JobHistorySource,
    title: &str,
    detail: Option<&str>,
    package_filename: Option<&str>,
    base_name: Option<&str>,
    warnings: Vec<JobHistoryWarning>,
    errors: Vec<JobHistoryError>,
    validation_status: Option<&str>,
    started_at: Option<&str>,
    finished_at: Option<&str>,
    no_changes_made: bool,
) -> JobHistoryItem {
    let warning_count = warnings.len();
    let error_count = errors.len();
    JobHistoryItem {
        id: JobHistoryId(id.to_string()),
        kind,
        status,
        source,
        started_at: started_at.map(str::to_string),
        finished_at: finished_at.map(str::to_string),
        summary: JobHistorySummary {
            title: title.to_string(),
            detail: detail.map(str::to_string),
            package_filename: package_filename.map(|f| redact_path_to_filename(f)),
            base_name: base_name.map(str::to_string),
            warning_count,
            error_count,
            validation_status: validation_status.map(str::to_string),
        },
        warnings,
        errors,
        no_changes_made,
    }
}

/// Build a history item for a package inspection result.
pub fn from_inspection(
    id: &str,
    package_filename: &str,
    valid: bool,
    warning_count: usize,
    error_count: usize,
    validation_status: &str,
    finished_at: Option<&str>,
) -> JobHistoryItem {
    let status = if !valid {
        JobHistoryStatus::Failed
    } else if warning_count > 0 {
        JobHistoryStatus::SucceededWithWarnings
    } else {
        JobHistoryStatus::Succeeded
    };
    let warnings: Vec<JobHistoryWarning> = Vec::new();
    let errors: Vec<JobHistoryError> = Vec::new();
    JobHistoryItem {
        id: JobHistoryId(id.to_string()),
        kind: JobHistoryKind::PackageInspection,
        status,
        source: JobHistorySource::RestorePage,
        started_at: None,
        finished_at: finished_at.map(str::to_string),
        summary: JobHistorySummary {
            title: "Package inspection".to_string(),
            detail: None,
            package_filename: Some(redact_path_to_filename(package_filename)),
            base_name: None,
            warning_count,
            error_count,
            validation_status: Some(validation_status.to_string()),
        },
        warnings,
        errors,
        no_changes_made: true,
    }
}

/// Build a history item for a restore dry-run plan result.
pub fn from_dry_run_plan(
    id: &str,
    filename: &str,
    status_str: &str,
    warning_count: usize,
    error_count: usize,
    finished_at: Option<&str>,
) -> JobHistoryItem {
    let status = match status_str {
        "ready" => JobHistoryStatus::Succeeded,
        "readyWithWarnings" => JobHistoryStatus::SucceededWithWarnings,
        _ => JobHistoryStatus::Blocked,
    };
    JobHistoryItem {
        id: JobHistoryId(id.to_string()),
        kind: JobHistoryKind::RestoreDryRun,
        status,
        source: JobHistorySource::RestorePage,
        started_at: None,
        finished_at: finished_at.map(str::to_string),
        summary: JobHistorySummary {
            title: "Restore dry-run plan".to_string(),
            detail: None,
            package_filename: Some(redact_path_to_filename(filename)),
            base_name: None,
            warning_count,
            error_count,
            validation_status: None,
        },
        warnings: vec![],
        errors: vec![],
        no_changes_made: true,
    }
}

/// Build a history item for a restore schema creation plan.
pub fn from_schema_plan(
    id: &str,
    filename: &str,
    status_str: &str,
    warning_count: usize,
    error_count: usize,
    finished_at: Option<&str>,
) -> JobHistoryItem {
    let status = match status_str {
        "ready" => JobHistoryStatus::Succeeded,
        "readyWithWarnings" => JobHistoryStatus::SucceededWithWarnings,
        _ => JobHistoryStatus::Blocked,
    };
    JobHistoryItem {
        id: JobHistoryId(id.to_string()),
        kind: JobHistoryKind::RestoreSchemaplan,
        status,
        source: JobHistorySource::RestorePage,
        started_at: None,
        finished_at: finished_at.map(str::to_string),
        summary: JobHistorySummary {
            title: "Restore schema creation plan".to_string(),
            detail: None,
            package_filename: Some(redact_path_to_filename(filename)),
            base_name: None,
            warning_count,
            error_count,
            validation_status: None,
        },
        warnings: vec![],
        errors: vec![],
        no_changes_made: true,
    }
}

/// Build a history item for a restore record import plan.
pub fn from_record_import_plan(
    id: &str,
    filename: &str,
    status_str: &str,
    warning_count: usize,
    error_count: usize,
    finished_at: Option<&str>,
) -> JobHistoryItem {
    let status = match status_str {
        "ready" => JobHistoryStatus::Succeeded,
        "readyWithWarnings" => JobHistoryStatus::SucceededWithWarnings,
        _ => JobHistoryStatus::Blocked,
    };
    JobHistoryItem {
        id: JobHistoryId(id.to_string()),
        kind: JobHistoryKind::RestoreRecordImportPlan,
        status,
        source: JobHistorySource::RestorePage,
        started_at: None,
        finished_at: finished_at.map(str::to_string),
        summary: JobHistorySummary {
            title: "Restore record import plan".to_string(),
            detail: None,
            package_filename: Some(redact_path_to_filename(filename)),
            base_name: None,
            warning_count,
            error_count,
            validation_status: None,
        },
        warnings: vec![],
        errors: vec![],
        no_changes_made: true,
    }
}

/// Build a history item for a blocked restore execution attempt.
pub fn from_restore_execution_blocked(
    id: &str,
    filename: &str,
    error_code: &str,
    finished_at: Option<&str>,
) -> JobHistoryItem {
    JobHistoryItem {
        id: JobHistoryId(id.to_string()),
        kind: JobHistoryKind::RestoreExecutionAttempt,
        status: JobHistoryStatus::Blocked,
        source: JobHistorySource::RestorePage,
        started_at: None,
        finished_at: finished_at.map(str::to_string),
        summary: JobHistorySummary {
            title: "Restore execution attempt (blocked)".to_string(),
            detail: Some(format!("Blocked: {error_code}")),
            package_filename: Some(redact_path_to_filename(filename)),
            base_name: None,
            warning_count: 0,
            error_count: 1,
            validation_status: None,
        },
        warnings: vec![],
        errors: vec![JobHistoryError {
            code: error_code.to_string(),
            message: format!("Restore execution blocked: {error_code}"),
        }],
        no_changes_made: true,
    }
}

/// Build a history item for a backup execution.
pub fn from_backup_execution(
    id: &str,
    package_filename: &str,
    base_name: Option<&str>,
    succeeded: bool,
    warning_count: usize,
    error_count: usize,
    started_at: Option<&str>,
    finished_at: Option<&str>,
) -> JobHistoryItem {
    let status = if succeeded {
        if warning_count > 0 {
            JobHistoryStatus::SucceededWithWarnings
        } else {
            JobHistoryStatus::Succeeded
        }
    } else {
        JobHistoryStatus::Failed
    };
    JobHistoryItem {
        id: JobHistoryId(id.to_string()),
        kind: JobHistoryKind::BackupExecution,
        status,
        source: JobHistorySource::BackupPage,
        started_at: started_at.map(str::to_string),
        finished_at: finished_at.map(str::to_string),
        summary: JobHistorySummary {
            title: "Backup execution".to_string(),
            detail: None,
            package_filename: Some(redact_path_to_filename(package_filename)),
            base_name: base_name.map(str::to_string),
            warning_count,
            error_count,
            validation_status: None,
        },
        warnings: vec![],
        errors: vec![],
        no_changes_made: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_item_redacts_path_in_filename() {
        let item = build_history_item(
            "h-001",
            JobHistoryKind::PackageInspection,
            JobHistoryStatus::Succeeded,
            JobHistorySource::RestorePage,
            "Package inspection",
            None,
            Some("/Users/alice/backups/my-backup.airbridge"),
            None,
            vec![],
            vec![],
            Some("valid"),
            None,
            None,
            true,
        );
        assert_eq!(
            item.summary.package_filename,
            Some("my-backup.airbridge".to_string())
        );
    }

    #[test]
    fn from_inspection_valid_no_warnings_is_succeeded() {
        let item = from_inspection("i1", "backup.airbridge", true, 0, 0, "valid", None);
        assert_eq!(item.status, JobHistoryStatus::Succeeded);
        assert!(item.no_changes_made);
    }

    #[test]
    fn from_inspection_with_warnings_is_succeeded_with_warnings() {
        let item = from_inspection("i2", "backup.airbridge", true, 2, 0, "valid", None);
        assert_eq!(item.status, JobHistoryStatus::SucceededWithWarnings);
    }

    #[test]
    fn from_inspection_invalid_is_failed() {
        let item = from_inspection("i3", "backup.airbridge", false, 0, 1, "invalid", None);
        assert_eq!(item.status, JobHistoryStatus::Failed);
    }

    #[test]
    fn from_inspection_path_redacted() {
        let item = from_inspection(
            "i4",
            "/Users/alice/backup.airbridge",
            true,
            0,
            0,
            "valid",
            None,
        );
        assert_eq!(
            item.summary.package_filename,
            Some("backup.airbridge".to_string())
        );
    }

    #[test]
    fn from_dry_run_plan_ready_is_succeeded() {
        let item = from_dry_run_plan("d1", "backup.airbridge", "ready", 0, 0, None);
        assert_eq!(item.status, JobHistoryStatus::Succeeded);
        assert!(item.no_changes_made);
    }

    #[test]
    fn from_dry_run_plan_blocked_is_blocked() {
        let item = from_dry_run_plan("d2", "backup.airbridge", "blocked", 0, 1, None);
        assert_eq!(item.status, JobHistoryStatus::Blocked);
    }

    #[test]
    fn from_schema_plan_ready_with_warnings() {
        let item = from_schema_plan("s1", "backup.airbridge", "readyWithWarnings", 1, 0, None);
        assert_eq!(item.status, JobHistoryStatus::SucceededWithWarnings);
        assert!(item.no_changes_made);
    }

    #[test]
    fn from_record_import_plan_is_no_changes_made() {
        let item = from_record_import_plan("r1", "backup.airbridge", "ready", 0, 0, None);
        assert!(item.no_changes_made);
    }

    #[test]
    fn from_restore_execution_blocked_has_error() {
        let item =
            from_restore_execution_blocked("e1", "backup.airbridge", "WRITE_ENGINE_DISABLED", None);
        assert_eq!(item.status, JobHistoryStatus::Blocked);
        assert_eq!(item.errors.len(), 1);
        assert_eq!(item.errors[0].code, "WRITE_ENGINE_DISABLED");
        assert!(item.no_changes_made);
    }

    #[test]
    fn from_backup_execution_succeeded_no_warnings() {
        let item = from_backup_execution(
            "b1",
            "backup.airbridge",
            Some("My Base"),
            true,
            0,
            0,
            None,
            None,
        );
        assert_eq!(item.status, JobHistoryStatus::Succeeded);
        assert!(!item.no_changes_made);
    }

    #[test]
    fn from_backup_execution_path_redacted() {
        let item = from_backup_execution(
            "b2",
            "/Users/alice/backup.airbridge",
            None,
            true,
            0,
            0,
            None,
            None,
        );
        assert_eq!(
            item.summary.package_filename,
            Some("backup.airbridge".to_string())
        );
    }

    #[test]
    fn summary_no_token_in_serialized_output() {
        let item = from_backup_execution(
            "b3",
            "backup.airbridge",
            Some("My Base"),
            true,
            0,
            0,
            None,
            None,
        );
        let json = serde_json::to_string(&item).unwrap();
        assert!(!json.contains("Bearer "));
        assert!(!json.contains("patXXX"));
    }
}
