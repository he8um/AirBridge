use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobHistoryId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobHistoryKind {
    ConnectionCheck,
    BackupPlan,
    RecordsExportPlan,
    BackupExecution,
    PackageInspection,
    RestoreDryRun,
    RestoreSchemaplan,
    RestoreRecordImportPlan,
    RestoreExecutionAttempt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobHistoryStatus {
    Planned,
    Running,
    Succeeded,
    SucceededWithWarnings,
    Blocked,
    Failed,
    Cancelled,
}

/// Identifies the originating surface of a history entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobHistorySource {
    BackupPage,
    RestorePage,
    ConnectionsPage,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobHistoryWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobHistoryError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobHistorySummary {
    pub title: String,
    pub detail: Option<String>,
    /// Filename only (never a full path).
    pub package_filename: Option<String>,
    pub base_name: Option<String>,
    pub warning_count: usize,
    pub error_count: usize,
    pub validation_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobHistoryItem {
    pub id: JobHistoryId,
    pub kind: JobHistoryKind,
    pub status: JobHistoryStatus,
    pub source: JobHistorySource,
    /// ISO-8601 timestamp string (UTC). No full path or token content.
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub summary: JobHistorySummary,
    pub warnings: Vec<JobHistoryWarning>,
    pub errors: Vec<JobHistoryError>,
    /// Always true for planning/inspection operations (no Airtable changes possible).
    pub no_changes_made: bool,
}

/// Filter parameters for listing history items.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobHistoryFilter {
    pub kind: Option<JobHistoryKind>,
    pub status: Option<JobHistoryStatus>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobHistoryListResult {
    pub items: Vec<JobHistoryItem>,
    pub total_count: usize,
    pub filtered: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_serializes_no_token_sentinel() {
        let item = JobHistoryItem {
            id: JobHistoryId("hist-001".to_string()),
            kind: JobHistoryKind::ConnectionCheck,
            status: JobHistoryStatus::Succeeded,
            source: JobHistorySource::ConnectionsPage,
            started_at: Some("2026-06-13T00:00:00Z".to_string()),
            finished_at: Some("2026-06-13T00:00:01Z".to_string()),
            summary: JobHistorySummary {
                title: "Connection check".to_string(),
                detail: None,
                package_filename: None,
                base_name: Some("My Base".to_string()),
                warning_count: 0,
                error_count: 0,
                validation_status: None,
            },
            warnings: vec![],
            errors: vec![],
            no_changes_made: true,
        };
        let json = serde_json::to_string(&item).expect("serialization failed");
        assert!(!json.contains("patXXX"));
        assert!(!json.contains("Bearer "));
        assert!(json.contains("hist-001"));
    }

    #[test]
    fn item_serializes_no_full_path() {
        let item = JobHistoryItem {
            id: JobHistoryId("hist-002".to_string()),
            kind: JobHistoryKind::PackageInspection,
            status: JobHistoryStatus::Succeeded,
            source: JobHistorySource::RestorePage,
            started_at: None,
            finished_at: None,
            summary: JobHistorySummary {
                title: "Package inspection".to_string(),
                detail: None,
                package_filename: Some("my-backup.airbridge".to_string()),
                base_name: None,
                warning_count: 0,
                error_count: 0,
                validation_status: Some("valid".to_string()),
            },
            warnings: vec![],
            errors: vec![],
            no_changes_made: true,
        };
        let json = serde_json::to_string(&item).expect("serialization failed");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\"));
        assert!(json.contains("my-backup.airbridge"));
    }

    #[test]
    fn kind_serializes_camel_case() {
        let json = serde_json::to_string(&JobHistoryKind::BackupExecution).unwrap();
        assert_eq!(json, "\"backupExecution\"");
    }

    #[test]
    fn status_serializes_camel_case() {
        let json = serde_json::to_string(&JobHistoryStatus::SucceededWithWarnings).unwrap();
        assert_eq!(json, "\"succeededWithWarnings\"");
    }

    #[test]
    fn filter_default_has_no_constraints() {
        let f = JobHistoryFilter::default();
        assert!(f.kind.is_none());
        assert!(f.status.is_none());
        assert!(f.limit.is_none());
    }
}
