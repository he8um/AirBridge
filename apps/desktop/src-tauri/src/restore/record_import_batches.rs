use crate::restore::record_import_plan::{
    RestoreRecordBatchPhase, RestoreRecordBatchPlan, RestoreRecordImportCheckpointPlan,
    RestoreTableImportPlan,
};

/// The batch size used for all Airtable record create/update operations.
pub const AIRTABLE_WRITE_BATCH_SIZE: usize = 10;

/// Computes the number of batches needed for a known record count.
pub fn batch_count_for(record_count: usize, batch_size: usize) -> usize {
    if batch_size == 0 {
        return 0;
    }
    record_count.div_ceil(batch_size)
}

/// Builds first-pass create batches for a table.
///
/// Returns an empty vec if record count is unknown.
pub fn build_first_pass_batches(
    record_count: Option<usize>,
    batch_size: usize,
    table_name: &str,
) -> Vec<RestoreRecordBatchPlan> {
    let Some(count) = record_count else {
        return vec![];
    };
    let n = batch_count_for(count, batch_size);
    (0..n)
        .map(|i| {
            let start = i * batch_size + 1;
            let end = ((i + 1) * batch_size).min(count);
            RestoreRecordBatchPlan {
                batch_index: i,
                phase: RestoreRecordBatchPhase::CreateRecords,
                record_count: end - start + 1,
                note: format!(
                    "First pass — create records {start}–{end} in '{table_name}' (no linked fields)."
                ),
            }
        })
        .collect()
}

/// Builds second-pass linked record update batches for a table.
///
/// Returns an empty vec if record count is unknown or no linked fields exist.
pub fn build_second_pass_batches(
    record_count: Option<usize>,
    batch_size: usize,
    has_linked_fields: bool,
    table_name: &str,
) -> Vec<RestoreRecordBatchPlan> {
    if !has_linked_fields {
        return vec![];
    }
    let Some(count) = record_count else {
        return vec![];
    };
    let n = batch_count_for(count, batch_size);
    (0..n)
        .map(|i| {
            let start = i * batch_size + 1;
            let end = ((i + 1) * batch_size).min(count);
            RestoreRecordBatchPlan {
                batch_index: i,
                phase: RestoreRecordBatchPhase::UpdateLinkedRecords,
                record_count: end - start + 1,
                note: format!(
                    "Second pass — update linked record fields for records {start}–{end} in '{table_name}'."
                ),
            }
        })
        .collect()
}

/// Builds the checkpoint plan for a table.
pub fn build_checkpoint_plan(
    table_id: &str,
    table_name: &str,
    create_batch_count: Option<usize>,
) -> RestoreRecordImportCheckpointPlan {
    let checkpoint_batch_index = create_batch_count.unwrap_or(0);
    RestoreRecordImportCheckpointPlan {
        table_id: table_id.to_string(),
        table_name: table_name.to_string(),
        checkpoint_batch_index,
        source_record_id_offset_placeholder: "<source_record_id_at_checkpoint>".to_string(),
        completed_phase: RestoreRecordBatchPhase::CreateRecords,
        note: format!(
            "After each batch, checkpoint is saved for '{table_name}' so import can resume from \
            the last completed batch without duplicating records."
        ),
    }
}

/// Computes the effective table import plan counts.
pub fn table_import_counts(table: &RestoreTableImportPlan) -> (Option<usize>, Option<usize>) {
    let create = table.create_batch_count;
    let update = table.update_batch_count;
    (create, update)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_count_exact_multiple() {
        assert_eq!(batch_count_for(20, 10), 2);
    }

    #[test]
    fn batch_count_remainder() {
        assert_eq!(batch_count_for(25, 10), 3);
    }

    #[test]
    fn batch_count_less_than_batch_size() {
        assert_eq!(batch_count_for(5, 10), 1);
    }

    #[test]
    fn batch_count_zero_records() {
        assert_eq!(batch_count_for(0, 10), 0);
    }

    #[test]
    fn first_pass_batches_known_count() {
        let batches = build_first_pass_batches(Some(25), 10, "Projects");
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].phase, RestoreRecordBatchPhase::CreateRecords);
        assert_eq!(batches[0].batch_index, 0);
        assert_eq!(batches[0].record_count, 10);
        assert_eq!(batches[2].record_count, 5);
    }

    #[test]
    fn first_pass_batches_unknown_count() {
        let batches = build_first_pass_batches(None, 10, "Projects");
        assert!(batches.is_empty());
    }

    #[test]
    fn second_pass_batches_with_linked_fields() {
        let batches = build_second_pass_batches(Some(15), 10, true, "Projects");
        assert_eq!(batches.len(), 2);
        assert_eq!(
            batches[0].phase,
            RestoreRecordBatchPhase::UpdateLinkedRecords
        );
    }

    #[test]
    fn second_pass_batches_no_linked_fields() {
        let batches = build_second_pass_batches(Some(15), 10, false, "Projects");
        assert!(batches.is_empty());
    }

    #[test]
    fn second_pass_batches_unknown_count() {
        let batches = build_second_pass_batches(None, 10, true, "Projects");
        assert!(batches.is_empty());
    }

    #[test]
    fn checkpoint_plan_has_placeholder() {
        let cp = build_checkpoint_plan("tbl01", "Projects", Some(3));
        assert_eq!(cp.checkpoint_batch_index, 3);
        assert!(cp.source_record_id_offset_placeholder.contains('<'));
        assert_eq!(cp.completed_phase, RestoreRecordBatchPhase::CreateRecords);
    }

    #[test]
    fn checkpoint_plan_unknown_count() {
        let cp = build_checkpoint_plan("tbl01", "Projects", None);
        assert_eq!(cp.checkpoint_batch_index, 0);
    }
}
