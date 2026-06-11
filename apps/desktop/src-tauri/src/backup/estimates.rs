use crate::models::backup_plan::{BackupPlanEstimate, RecordReadEstimate};

/// Default Airtable API page size for record listing.
pub const DEFAULT_PAGE_SIZE: usize = 100;

/// Fixed number of API requests for reading a single base schema.
pub const SCHEMA_REQUESTS_PER_BASE: usize = 1;

/// Estimates the number of record-read API pages required for a single table.
///
/// Returns `RecordReadEstimate::Known(pages)` when a record count is provided,
/// or `RecordReadEstimate::Unknown` when the count is not yet known.
/// A table with zero records still requires one request to confirm the list is empty.
pub fn estimate_record_pages(record_count: Option<usize>) -> RecordReadEstimate {
    match record_count {
        None => RecordReadEstimate::Unknown,
        Some(n) if n == 0 => RecordReadEstimate::Known(1),
        Some(n) => {
            let pages = (n + DEFAULT_PAGE_SIZE - 1) / DEFAULT_PAGE_SIZE;
            RecordReadEstimate::Known(pages)
        }
    }
}

/// Builds a `BackupPlanEstimate` from a list of optional per-table record counts.
///
/// If any table count is unknown the total is also unknown.
pub fn build_estimate(per_table_record_counts: &[Option<usize>]) -> BackupPlanEstimate {
    let schema_requests = SCHEMA_REQUESTS_PER_BASE;

    let mut total_record_pages: Option<usize> = Some(0);
    for count in per_table_record_counts {
        match estimate_record_pages(*count) {
            RecordReadEstimate::Known(pages) => {
                if let Some(ref mut total) = total_record_pages {
                    *total += pages;
                }
            }
            RecordReadEstimate::Unknown => {
                total_record_pages = None;
                break;
            }
        }
    }

    BackupPlanEstimate {
        schema_requests,
        record_read_pages: total_record_pages
            .map(RecordReadEstimate::Known)
            .unwrap_or(RecordReadEstimate::Unknown),
        note: "Record counts are unknown until records are fetched. \
               Page estimates are approximate."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_count_returns_unknown_estimate() {
        assert_eq!(estimate_record_pages(None), RecordReadEstimate::Unknown);
    }

    #[test]
    fn zero_records_requires_one_page() {
        assert_eq!(estimate_record_pages(Some(0)), RecordReadEstimate::Known(1));
    }

    #[test]
    fn one_record_requires_one_page() {
        assert_eq!(estimate_record_pages(Some(1)), RecordReadEstimate::Known(1));
    }

    #[test]
    fn exactly_100_records_requires_one_page() {
        assert_eq!(
            estimate_record_pages(Some(100)),
            RecordReadEstimate::Known(1)
        );
    }

    #[test]
    fn one_hundred_one_records_requires_two_pages() {
        assert_eq!(
            estimate_record_pages(Some(101)),
            RecordReadEstimate::Known(2)
        );
    }

    #[test]
    fn two_hundred_records_requires_two_pages() {
        assert_eq!(
            estimate_record_pages(Some(200)),
            RecordReadEstimate::Known(2)
        );
    }

    #[test]
    fn large_record_count_math_is_correct() {
        // 1050 records → ceil(1050/100) = 11 pages
        assert_eq!(
            estimate_record_pages(Some(1050)),
            RecordReadEstimate::Known(11)
        );
    }

    #[test]
    fn build_estimate_all_known_sums_pages() {
        let est = build_estimate(&[Some(100), Some(50), Some(0)]);
        // 100→1, 50→1, 0→1 = 3 pages total
        assert_eq!(est.record_read_pages, RecordReadEstimate::Known(3));
        assert_eq!(est.schema_requests, SCHEMA_REQUESTS_PER_BASE);
    }

    #[test]
    fn build_estimate_any_unknown_yields_unknown_total() {
        let est = build_estimate(&[Some(100), None, Some(50)]);
        assert_eq!(est.record_read_pages, RecordReadEstimate::Unknown);
    }

    #[test]
    fn build_estimate_empty_table_list_is_one_schema_request() {
        let est = build_estimate(&[]);
        assert_eq!(est.schema_requests, 1);
        assert_eq!(est.record_read_pages, RecordReadEstimate::Known(0));
    }

    #[test]
    fn build_estimate_note_is_non_empty() {
        let est = build_estimate(&[Some(10)]);
        assert!(!est.note.is_empty());
    }
}
