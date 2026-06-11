use serde::{Deserialize, Serialize};

use crate::backup::validation::ValidationStatus;

/// Opaque identifier for a backup job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackupJobId(pub String);

impl BackupJobId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BackupJobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lifecycle status of a backup job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Current pipeline phase of a running or completed job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupJobPhase {
    Planning,
    Schema,
    RecordsExport,
    PackageBuild,
    Validation,
    Completed,
}

/// Scalar progress snapshot for a phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobProgress {
    pub phase: BackupJobPhase,
    /// Human-readable description of current work.
    pub message: String,
    /// Number of tables completed so far.
    pub tables_completed: usize,
    /// Total tables to export, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tables: Option<usize>,
}

/// A non-fatal warning produced during a backup job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_id: Option<String>,
}

/// A fatal or recoverable error produced during a backup job.
///
/// Messages are sanitized — no token values or absolute user paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

/// Per-table result included in `BackupJobResult`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobTableResult {
    pub table_id: String,
    pub table_name: String,
    pub record_count: usize,
    pub pages_fetched: usize,
}

/// Summary of the written package, safe to return to callers.
///
/// Does not include the absolute filesystem path to the package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobPackageSummary {
    pub package_id: String,
    pub format_version: String,
    pub table_count: usize,
    pub record_count: usize,
    pub entry_count: usize,
    pub checksum_count: usize,
    /// Always false for V0.1.
    pub encrypted: bool,
    pub attachment_policy: String,
}

/// Summary of the validation run on the written package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobValidationSummary {
    pub status: ValidationStatus,
    pub error_count: usize,
    pub warning_count: usize,
    pub entry_count: usize,
}

/// Complete result returned after a backup job finishes.
///
/// Safe to serialise and return to the frontend:
/// - No token.
/// - No absolute user paths.
/// - No full attachment URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupJobResult {
    pub job_id: BackupJobId,
    pub status: BackupJobStatus,
    pub base_id: String,
    pub base_name: String,
    pub tables: Vec<BackupJobTableResult>,
    pub warnings: Vec<BackupJobWarning>,
    pub errors: Vec<BackupJobError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_summary: Option<BackupJobPackageSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_summary: Option<BackupJobValidationSummary>,
    /// Ordered event timeline for the job.
    ///
    /// Events contain no token, no absolute paths, and no attachment URLs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<crate::backup::job_events::BackupJobEvent>,
}

impl BackupJobResult {
    pub fn succeeded(&self) -> bool {
        self.status == BackupJobStatus::Succeeded
    }

    pub fn failed(&self) -> bool {
        self.status == BackupJobStatus::Failed
    }

    pub fn cancelled(&self) -> bool {
        self.status == BackupJobStatus::Cancelled
    }
}

/// Input required to run a backup job.
#[derive(Debug, Clone)]
pub struct BackupJobRequest {
    pub job_id: BackupJobId,
    pub base_id: String,
    pub base_name: String,
    /// Pre-serialised base metadata JSON (no token embedded).
    pub base_json: Vec<u8>,
    /// Pre-serialised schema JSON.
    pub schema_json: Vec<u8>,
    /// Table specs for record export.
    pub table_specs: Vec<crate::backup::export_engine::TableExportSpec>,
    /// Page size for the pagination loop.
    pub page_size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result(status: BackupJobStatus) -> BackupJobResult {
        BackupJobResult {
            job_id: BackupJobId("job-syn-001".to_string()),
            status,
            base_id: "appSyn01".to_string(),
            base_name: "Synthetic".to_string(),
            tables: vec![],
            warnings: vec![],
            errors: vec![],
            package_summary: None,
            validation_summary: None,
            events: vec![],
        }
    }

    #[test]
    fn succeeded_result_status_check() {
        assert!(sample_result(BackupJobStatus::Succeeded).succeeded());
        assert!(!sample_result(BackupJobStatus::Succeeded).failed());
        assert!(!sample_result(BackupJobStatus::Succeeded).cancelled());
    }

    #[test]
    fn failed_result_status_check() {
        assert!(sample_result(BackupJobStatus::Failed).failed());
        assert!(!sample_result(BackupJobStatus::Failed).succeeded());
    }

    #[test]
    fn cancelled_result_status_check() {
        assert!(sample_result(BackupJobStatus::Cancelled).cancelled());
        assert!(!sample_result(BackupJobStatus::Cancelled).succeeded());
    }

    #[test]
    fn result_serializes_without_absolute_path() {
        let r = sample_result(BackupJobStatus::Succeeded);
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn result_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_job_test_sentinel_0123456789";
        let r = sample_result(BackupJobStatus::Succeeded);
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn package_summary_encrypted_false_v01() {
        let summary = BackupJobPackageSummary {
            package_id: "00000000-0000-0000-0000-000000000001".to_string(),
            format_version: "0.1.0".to_string(),
            table_count: 2,
            record_count: 10,
            entry_count: 8,
            checksum_count: 7,
            encrypted: false,
            attachment_policy: "metadataOnly".to_string(),
        };
        assert!(!summary.encrypted);
        assert_eq!(summary.attachment_policy, "metadataOnly");
    }

    #[test]
    fn validation_summary_serializes() {
        let vs = BackupJobValidationSummary {
            status: ValidationStatus::Valid,
            error_count: 0,
            warning_count: 0,
            entry_count: 6,
        };
        let json = serde_json::to_string(&vs).expect("serialize");
        assert!(json.contains("valid"));
    }

    #[test]
    fn job_id_display() {
        let id = BackupJobId("job-001".to_string());
        assert_eq!(id.to_string(), "job-001");
    }

    #[test]
    fn backup_job_warning_serializes() {
        let w = BackupJobWarning {
            code: "RATE_LIMITED".to_string(),
            message: "request was rate limited".to_string(),
            table_id: Some("tbl01".to_string()),
        };
        let json = serde_json::to_string(&w).expect("serialize");
        assert!(json.contains("RATE_LIMITED"));
        assert!(json.contains("tbl01"));
    }

    #[test]
    fn backup_job_error_recoverable_field() {
        let e = BackupJobError {
            code: "AUTH_FAILED".to_string(),
            message: "authentication failed".to_string(),
            recoverable: false,
        };
        assert!(!e.recoverable);
    }

    #[test]
    fn result_with_package_and_validation_summary() {
        let mut r = sample_result(BackupJobStatus::Succeeded);
        r.package_summary = Some(BackupJobPackageSummary {
            package_id: "00000000-0000-0000-0000-000000000001".to_string(),
            format_version: "0.1.0".to_string(),
            table_count: 1,
            record_count: 3,
            entry_count: 6,
            checksum_count: 5,
            encrypted: false,
            attachment_policy: "metadataOnly".to_string(),
        });
        r.validation_summary = Some(BackupJobValidationSummary {
            status: ValidationStatus::Valid,
            error_count: 0,
            warning_count: 0,
            entry_count: 6,
        });
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(json.contains("packageSummary"));
        assert!(json.contains("validationSummary"));
    }
}
