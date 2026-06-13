use crate::history::models::{JobHistoryFilter, JobHistoryItem, JobHistoryListResult};

/// Abstraction over a job history store.
///
/// In this version the only implementation is `InMemoryJobHistoryStore`.
/// The trait is designed so a SQLite-backed implementation can be added later
/// without changing the command layer.
pub trait JobHistoryStore {
    fn add(&mut self, item: JobHistoryItem);
    fn list(&self, filter: &JobHistoryFilter) -> JobHistoryListResult;
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory store. Items are stored in insertion order; `list` returns them
/// most-recent-first (reversed) so callers get the latest activity at the top.
#[derive(Default)]
pub struct InMemoryJobHistoryStore {
    items: Vec<JobHistoryItem>,
}

impl InMemoryJobHistoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl JobHistoryStore for InMemoryJobHistoryStore {
    fn add(&mut self, item: JobHistoryItem) {
        self.items.push(item);
    }

    fn list(&self, filter: &JobHistoryFilter) -> JobHistoryListResult {
        let filtered = filter.kind.is_some() || filter.status.is_some();

        let mut results: Vec<&JobHistoryItem> = self
            .items
            .iter()
            .rev()
            .filter(|item| {
                if let Some(ref kind) = filter.kind {
                    if &item.kind != kind {
                        return false;
                    }
                }
                if let Some(ref status) = filter.status {
                    if &item.status != status {
                        return false;
                    }
                }
                true
            })
            .collect();

        let total_count = results.len();

        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        JobHistoryListResult {
            items: results.into_iter().cloned().collect(),
            total_count,
            filtered,
        }
    }

    fn clear(&mut self) {
        self.items.clear();
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::models::{
        JobHistoryError, JobHistoryId, JobHistoryKind, JobHistorySource, JobHistoryStatus,
        JobHistorySummary, JobHistoryWarning,
    };

    fn make_item(id: &str, kind: JobHistoryKind, status: JobHistoryStatus) -> JobHistoryItem {
        JobHistoryItem {
            id: JobHistoryId(id.to_string()),
            kind,
            status,
            source: JobHistorySource::System,
            started_at: None,
            finished_at: None,
            summary: JobHistorySummary {
                title: format!("Item {id}"),
                detail: None,
                package_filename: None,
                base_name: None,
                warning_count: 0,
                error_count: 0,
                validation_status: None,
            },
            warnings: vec![],
            errors: vec![],
            no_changes_made: true,
        }
    }

    #[test]
    fn add_and_list_returns_all_items() {
        let mut store = InMemoryJobHistoryStore::new();
        store.add(make_item(
            "a",
            JobHistoryKind::ConnectionCheck,
            JobHistoryStatus::Succeeded,
        ));
        store.add(make_item(
            "b",
            JobHistoryKind::BackupExecution,
            JobHistoryStatus::Succeeded,
        ));
        let result = store.list(&JobHistoryFilter::default());
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn list_returns_most_recent_first() {
        let mut store = InMemoryJobHistoryStore::new();
        store.add(make_item(
            "first",
            JobHistoryKind::ConnectionCheck,
            JobHistoryStatus::Succeeded,
        ));
        store.add(make_item(
            "second",
            JobHistoryKind::BackupExecution,
            JobHistoryStatus::Succeeded,
        ));
        let result = store.list(&JobHistoryFilter::default());
        assert_eq!(result.items[0].id, JobHistoryId("second".to_string()));
        assert_eq!(result.items[1].id, JobHistoryId("first".to_string()));
    }

    #[test]
    fn filter_by_kind() {
        let mut store = InMemoryJobHistoryStore::new();
        store.add(make_item(
            "a",
            JobHistoryKind::ConnectionCheck,
            JobHistoryStatus::Succeeded,
        ));
        store.add(make_item(
            "b",
            JobHistoryKind::BackupExecution,
            JobHistoryStatus::Succeeded,
        ));
        store.add(make_item(
            "c",
            JobHistoryKind::ConnectionCheck,
            JobHistoryStatus::Failed,
        ));
        let filter = JobHistoryFilter {
            kind: Some(JobHistoryKind::ConnectionCheck),
            status: None,
            limit: None,
        };
        let result = store.list(&filter);
        assert_eq!(result.items.len(), 2);
        assert!(result.filtered);
    }

    #[test]
    fn filter_by_status() {
        let mut store = InMemoryJobHistoryStore::new();
        store.add(make_item(
            "a",
            JobHistoryKind::ConnectionCheck,
            JobHistoryStatus::Succeeded,
        ));
        store.add(make_item(
            "b",
            JobHistoryKind::BackupExecution,
            JobHistoryStatus::Failed,
        ));
        let filter = JobHistoryFilter {
            kind: None,
            status: Some(JobHistoryStatus::Failed),
            limit: None,
        };
        let result = store.list(&filter);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, JobHistoryId("b".to_string()));
    }

    #[test]
    fn limit_restricts_results() {
        let mut store = InMemoryJobHistoryStore::new();
        for i in 0..10 {
            store.add(make_item(
                &format!("item-{i}"),
                JobHistoryKind::ConnectionCheck,
                JobHistoryStatus::Succeeded,
            ));
        }
        let filter = JobHistoryFilter {
            kind: None,
            status: None,
            limit: Some(3),
        };
        let result = store.list(&filter);
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.total_count, 10);
    }

    #[test]
    fn clear_empties_store() {
        let mut store = InMemoryJobHistoryStore::new();
        store.add(make_item(
            "a",
            JobHistoryKind::BackupExecution,
            JobHistoryStatus::Succeeded,
        ));
        assert_eq!(store.len(), 1);
        store.clear();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn empty_store_list_returns_empty() {
        let store = InMemoryJobHistoryStore::new();
        let result = store.list(&JobHistoryFilter::default());
        assert!(result.items.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.filtered);
    }

    #[test]
    fn warning_and_error_counts_stored() {
        let mut item = make_item(
            "w",
            JobHistoryKind::PackageInspection,
            JobHistoryStatus::SucceededWithWarnings,
        );
        item.warnings.push(JobHistoryWarning {
            code: "W001".to_string(),
            message: "a warning".to_string(),
        });
        item.errors.push(JobHistoryError {
            code: "E001".to_string(),
            message: "an error".to_string(),
        });
        item.summary.warning_count = 1;
        item.summary.error_count = 1;
        let mut store = InMemoryJobHistoryStore::new();
        store.add(item);
        let result = store.list(&JobHistoryFilter::default());
        assert_eq!(result.items[0].summary.warning_count, 1);
        assert_eq!(result.items[0].summary.error_count, 1);
    }
}
