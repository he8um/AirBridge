use crate::errors::AirBridgeResult;
use crate::history::models::{JobHistoryFilter, JobHistoryListResult};
use crate::history::store::{InMemoryJobHistoryStore, JobHistoryStore};
use crate::history::summaries::{
    from_backup_execution, from_dry_run_plan, from_inspection, from_record_import_plan,
    from_restore_execution_blocked, from_schema_plan,
};

/// Build a deterministic in-memory history snapshot suitable for the current
/// phase (no live persistence). Deterministic data allows UI and tests to work
/// without a background service.
fn build_deterministic_store() -> InMemoryJobHistoryStore {
    let mut store = InMemoryJobHistoryStore::new();

    store.add(from_backup_execution(
        "hist-001",
        "my-base-2026-06-10.airbridge",
        Some("My Base"),
        true,
        0,
        0,
        Some("2026-06-10T09:00:00Z"),
        Some("2026-06-10T09:01:12Z"),
    ));

    store.add(from_inspection(
        "hist-002",
        "my-base-2026-06-10.airbridge",
        true,
        0,
        0,
        "valid",
        Some("2026-06-10T09:05:00Z"),
    ));

    store.add(from_dry_run_plan(
        "hist-003",
        "my-base-2026-06-10.airbridge",
        "readyWithWarnings",
        2,
        0,
        Some("2026-06-10T09:06:00Z"),
    ));

    store.add(from_schema_plan(
        "hist-004",
        "my-base-2026-06-10.airbridge",
        "ready",
        0,
        0,
        Some("2026-06-10T09:07:00Z"),
    ));

    store.add(from_record_import_plan(
        "hist-005",
        "my-base-2026-06-10.airbridge",
        "readyWithWarnings",
        3,
        0,
        Some("2026-06-10T09:08:00Z"),
    ));

    store.add(from_restore_execution_blocked(
        "hist-006",
        "my-base-2026-06-10.airbridge",
        "WRITE_ENGINE_DISABLED",
        Some("2026-06-10T09:09:00Z"),
    ));

    store
}

/// List recent job history items.
///
/// - No token in request or response.
/// - No full paths in response.
/// - No record payloads.
/// - In this version returns deterministic in-memory data.
#[tauri::command]
pub fn list_job_history(filter: Option<JobHistoryFilter>) -> AirBridgeResult<JobHistoryListResult> {
    let store = build_deterministic_store();
    let f = filter.unwrap_or_default();
    Ok(store.list(&f))
}

/// Clear job history (no-op in this phase — no persistent store exists).
/// Returns the count that would have been cleared (always 0 in-memory).
#[tauri::command]
pub fn clear_job_history() -> AirBridgeResult<usize> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::models::{JobHistoryKind, JobHistoryStatus};

    #[test]
    fn list_returns_items() {
        let result = list_job_history(None).expect("command failed");
        assert!(!result.items.is_empty());
    }

    #[test]
    fn list_returns_most_recent_first() {
        let result = list_job_history(None).expect("command failed");
        assert_eq!(
            result.items[0].id.0, "hist-006",
            "most recent item should be first"
        );
    }

    #[test]
    fn list_with_kind_filter() {
        let filter = JobHistoryFilter {
            kind: Some(JobHistoryKind::PackageInspection),
            status: None,
            limit: None,
        };
        let result = list_job_history(Some(filter)).expect("command failed");
        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn list_with_limit() {
        let filter = JobHistoryFilter {
            kind: None,
            status: None,
            limit: Some(2),
        };
        let result = list_job_history(Some(filter)).expect("command failed");
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn list_no_token_in_serialized_output() {
        let result = list_job_history(None).expect("command failed");
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("Bearer "));
        assert!(!json.contains("patXXX"));
    }

    #[test]
    fn list_no_full_path_in_output() {
        let result = list_job_history(None).expect("command failed");
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\"));
    }

    #[test]
    fn list_no_changes_made_true_for_planning_items() {
        let result = list_job_history(None).expect("command failed");
        for item in &result.items {
            if matches!(
                item.kind,
                JobHistoryKind::PackageInspection
                    | JobHistoryKind::RestoreDryRun
                    | JobHistoryKind::RestoreSchemaplan
                    | JobHistoryKind::RestoreRecordImportPlan
                    | JobHistoryKind::RestoreExecutionAttempt
            ) {
                assert!(
                    item.no_changes_made,
                    "item {:?} should have no_changes_made: true",
                    item.id
                );
            }
        }
    }

    #[test]
    fn list_succeeded_items_present() {
        let filter = JobHistoryFilter {
            kind: None,
            status: Some(JobHistoryStatus::Succeeded),
            limit: None,
        };
        let result = list_job_history(Some(filter)).expect("command failed");
        assert!(!result.items.is_empty());
    }

    #[test]
    fn clear_returns_zero_in_memory() {
        let cleared = clear_job_history().expect("command failed");
        assert_eq!(cleared, 0);
    }
}
