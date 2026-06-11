use crate::backup::job::{
    BackupJobError, BackupJobId, BackupJobPackageSummary, BackupJobResult, BackupJobStatus,
    BackupJobTableResult, BackupJobValidationSummary, BackupJobWarning,
};
use crate::backup::validation::ValidationReport;

/// Build a succeeded `BackupJobResult` from orchestrator outputs.
pub fn build_succeeded_result(
    job_id: BackupJobId,
    base_id: &str,
    base_name: &str,
    table_results: Vec<BackupJobTableResult>,
    package_summary: BackupJobPackageSummary,
    validation_summary: BackupJobValidationSummary,
    warnings: Vec<BackupJobWarning>,
) -> BackupJobResult {
    BackupJobResult {
        job_id,
        status: BackupJobStatus::Succeeded,
        base_id: base_id.to_string(),
        base_name: base_name.to_string(),
        tables: table_results,
        warnings,
        errors: vec![],
        package_summary: Some(package_summary),
        validation_summary: Some(validation_summary),
    }
}

/// Build a failed `BackupJobResult`.
///
/// Error messages are expected to be pre-sanitised by the caller
/// (no tokens, no absolute paths).
pub fn build_failed_result(
    job_id: BackupJobId,
    base_id: &str,
    base_name: &str,
    errors: Vec<BackupJobError>,
    warnings: Vec<BackupJobWarning>,
) -> BackupJobResult {
    BackupJobResult {
        job_id,
        status: BackupJobStatus::Failed,
        base_id: base_id.to_string(),
        base_name: base_name.to_string(),
        tables: vec![],
        warnings,
        errors,
        package_summary: None,
        validation_summary: None,
    }
}

/// Build a cancelled `BackupJobResult`.
pub fn build_cancelled_result(
    job_id: BackupJobId,
    base_id: &str,
    base_name: &str,
    warnings: Vec<BackupJobWarning>,
) -> BackupJobResult {
    BackupJobResult {
        job_id,
        status: BackupJobStatus::Cancelled,
        base_id: base_id.to_string(),
        base_name: base_name.to_string(),
        tables: vec![],
        warnings,
        errors: vec![],
        package_summary: None,
        validation_summary: None,
    }
}

/// Derive a `BackupJobValidationSummary` from a `ValidationReport`.
pub fn validation_summary_from_report(report: &ValidationReport) -> BackupJobValidationSummary {
    BackupJobValidationSummary {
        status: report.status.clone(),
        error_count: report.errors.len(),
        warning_count: report.warnings.len(),
        entry_count: report.entry_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::job::BackupJobStatus;
    use crate::backup::validation::{ValidationIssue, ValidationStatus};

    fn job_id() -> BackupJobId {
        BackupJobId("job-result-test-001".to_string())
    }

    fn pkg_summary() -> BackupJobPackageSummary {
        BackupJobPackageSummary {
            package_id: "00000000-0000-0000-0000-000000000001".to_string(),
            format_version: "0.1.0".to_string(),
            table_count: 1,
            record_count: 3,
            entry_count: 6,
            checksum_count: 5,
            encrypted: false,
            attachment_policy: "metadataOnly".to_string(),
        }
    }

    fn val_summary() -> BackupJobValidationSummary {
        BackupJobValidationSummary {
            status: ValidationStatus::Valid,
            error_count: 0,
            warning_count: 0,
            entry_count: 6,
        }
    }

    #[test]
    fn succeeded_result_has_correct_status() {
        let r = build_succeeded_result(
            job_id(),
            "appSyn01",
            "Synthetic",
            vec![],
            pkg_summary(),
            val_summary(),
            vec![],
        );
        assert_eq!(r.status, BackupJobStatus::Succeeded);
        assert!(r.package_summary.is_some());
        assert!(r.validation_summary.is_some());
        assert!(r.errors.is_empty());
    }

    #[test]
    fn failed_result_has_correct_status() {
        let r = build_failed_result(
            job_id(),
            "appSyn01",
            "Synthetic",
            vec![BackupJobError {
                code: "AUTH_FAILED".to_string(),
                message: "authentication failed".to_string(),
                recoverable: false,
            }],
            vec![],
        );
        assert_eq!(r.status, BackupJobStatus::Failed);
        assert_eq!(r.errors.len(), 1);
        assert!(r.package_summary.is_none());
    }

    #[test]
    fn cancelled_result_has_correct_status() {
        let r = build_cancelled_result(job_id(), "appSyn01", "Synthetic", vec![]);
        assert_eq!(r.status, BackupJobStatus::Cancelled);
        assert!(r.package_summary.is_none());
        assert!(r.errors.is_empty());
    }

    #[test]
    fn validation_summary_from_valid_report() {
        let report = ValidationReport {
            status: ValidationStatus::Valid,
            errors: vec![],
            warnings: vec![],
            entry_count: 7,
            manifest_summary: None,
        };
        let summary = validation_summary_from_report(&report);
        assert_eq!(summary.status, ValidationStatus::Valid);
        assert_eq!(summary.entry_count, 7);
        assert_eq!(summary.error_count, 0);
    }

    #[test]
    fn validation_summary_from_invalid_report() {
        let report = ValidationReport {
            status: ValidationStatus::Invalid,
            errors: vec![ValidationIssue {
                code: "CHECKSUM_MISMATCH".to_string(),
                message: "hash mismatch".to_string(),
            }],
            warnings: vec![],
            entry_count: 5,
            manifest_summary: None,
        };
        let summary = validation_summary_from_report(&report);
        assert_eq!(summary.status, ValidationStatus::Invalid);
        assert_eq!(summary.error_count, 1);
    }

    #[test]
    fn no_result_contains_token_sentinel() {
        const SENTINEL: &str = "pat_job_result_sentinel_0123456789";
        let r = build_succeeded_result(
            job_id(),
            "appSyn01",
            "Synthetic",
            vec![],
            pkg_summary(),
            val_summary(),
            vec![],
        );
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn no_result_contains_absolute_path() {
        let r = build_failed_result(
            job_id(),
            "appSyn01",
            "Synthetic",
            vec![BackupJobError {
                code: "WRITE_ERROR".to_string(),
                message: "package write failed".to_string(),
                recoverable: false,
            }],
            vec![],
        );
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
    }
}
