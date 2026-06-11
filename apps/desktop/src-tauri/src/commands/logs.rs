use crate::errors::AirBridgeResult;
use crate::models::log::{JobLogEntry, LogLevel};

#[tauri::command]
pub fn list_logs() -> AirBridgeResult<Vec<JobLogEntry>> {
    Ok(vec![
        JobLogEntry {
            id: "log-001".to_string(),
            timestamp: "2025-01-14T14:21:55Z".to_string(),
            level: LogLevel::Debug,
            job_id: Some("job-001".to_string()),
            job_type: Some("backup".to_string()),
            message: "Initializing backup job".to_string(),
            detail: None,
        },
        JobLogEntry {
            id: "log-002".to_string(),
            timestamp: "2025-01-14T14:21:56Z".to_string(),
            level: LogLevel::Info,
            job_id: Some("job-001".to_string()),
            job_type: None,
            message: "Connected to Airtable API".to_string(),
            detail: None,
        },
        JobLogEntry {
            id: "log-003".to_string(),
            timestamp: "2025-01-14T14:22:00Z".to_string(),
            level: LogLevel::Info,
            job_id: Some("job-001".to_string()),
            job_type: None,
            message: "Backing up table: Projects (32 records)".to_string(),
            detail: None,
        },
        JobLogEntry {
            id: "log-004".to_string(),
            timestamp: "2025-01-14T14:22:04Z".to_string(),
            level: LogLevel::Info,
            job_id: Some("job-001".to_string()),
            job_type: None,
            message: "Backing up table: Tasks (15 records)".to_string(),
            detail: None,
        },
        JobLogEntry {
            id: "log-005".to_string(),
            timestamp: "2025-01-14T14:22:06Z".to_string(),
            level: LogLevel::Warning,
            job_id: Some("job-001".to_string()),
            job_type: None,
            message: "Rate limit reached, backing off 30s".to_string(),
            detail: Some("HTTP 429 received. Retrying after delay.".to_string()),
        },
        JobLogEntry {
            id: "log-006".to_string(),
            timestamp: "2025-01-14T14:22:10Z".to_string(),
            level: LogLevel::Info,
            job_id: Some("job-001".to_string()),
            job_type: None,
            message: "Backup complete: 47 records written".to_string(),
            detail: None,
        },
    ])
}
