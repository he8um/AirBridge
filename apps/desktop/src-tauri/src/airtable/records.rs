use super::models::{AirtableRecordFields, AirtableRecordUpdate};

/// Maximum records per batch for create and update operations.
///
/// Airtable enforces a batch size limit for write operations.
pub const MAX_WRITE_BATCH_SIZE: usize = 10;

/// Splits a list of record-create payloads into fixed-size batches.
///
/// Returns an error if `records` is empty.
pub fn split_create_batches(
    records: Vec<AirtableRecordFields>,
) -> Result<Vec<Vec<AirtableRecordFields>>, String> {
    if records.is_empty() {
        return Err("record list must not be empty".to_string());
    }
    Ok(records
        .chunks(MAX_WRITE_BATCH_SIZE)
        .map(|c| c.to_vec())
        .collect())
}

/// Splits a list of record-update payloads into fixed-size batches.
///
/// Returns an error if `records` is empty.
pub fn split_update_batches(
    records: Vec<AirtableRecordUpdate>,
) -> Result<Vec<Vec<AirtableRecordUpdate>>, String> {
    if records.is_empty() {
        return Err("record list must not be empty".to_string());
    }
    Ok(records
        .chunks(MAX_WRITE_BATCH_SIZE)
        .map(|c| c.to_vec())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::AirtableRecordId;
    use std::collections::HashMap;

    fn make_create_records(n: usize) -> Vec<AirtableRecordFields> {
        (0..n)
            .map(|_| AirtableRecordFields {
                fields: HashMap::new(),
            })
            .collect()
    }

    fn make_update_records(n: usize) -> Vec<AirtableRecordUpdate> {
        (0..n)
            .map(|i| AirtableRecordUpdate {
                id: AirtableRecordId(format!("recExample{i:04}")),
                fields: HashMap::new(),
            })
            .collect()
    }

    #[test]
    fn empty_create_list_returns_error() {
        assert!(split_create_batches(vec![]).is_err());
    }

    #[test]
    fn empty_update_list_returns_error() {
        assert!(split_update_batches(vec![]).is_err());
    }

    #[test]
    fn exactly_one_batch_when_under_limit() {
        let batches = split_create_batches(make_create_records(5)).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 5);
    }

    #[test]
    fn exactly_one_batch_when_at_limit() {
        let batches = split_create_batches(make_create_records(MAX_WRITE_BATCH_SIZE)).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), MAX_WRITE_BATCH_SIZE);
    }

    #[test]
    fn two_batches_when_over_limit() {
        let batches = split_create_batches(make_create_records(MAX_WRITE_BATCH_SIZE + 1)).unwrap();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), MAX_WRITE_BATCH_SIZE);
        assert_eq!(batches[1].len(), 1);
    }

    #[test]
    fn total_records_preserved_across_batches() {
        let n = 25;
        let batches = split_create_batches(make_create_records(n)).unwrap();
        let total: usize = batches.iter().map(|b| b.len()).sum();
        assert_eq!(total, n);
    }

    #[test]
    fn update_batching_matches_create_batching_behaviour() {
        let n = 23;
        let batches = split_update_batches(make_update_records(n)).unwrap();
        let total: usize = batches.iter().map(|b| b.len()).sum();
        assert_eq!(total, n);
        assert!(batches.iter().all(|b| b.len() <= MAX_WRITE_BATCH_SIZE));
    }
}
