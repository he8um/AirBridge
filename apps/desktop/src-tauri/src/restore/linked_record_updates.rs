use crate::restore::record_import_batches::{batch_count_for, AIRTABLE_WRITE_BATCH_SIZE};
use crate::restore::record_import_plan::{
    RecordImportFieldInput, RecordImportTableInput, RestoreLinkedRecordUpdatePlan,
};

/// Builds the list of second-pass linked record update plans for a table.
///
/// One plan is created per linked record field. Each plan describes the update
/// batch structure for applying remapped record IDs.
pub fn build_linked_update_plans(
    table: &RecordImportTableInput,
) -> Vec<RestoreLinkedRecordUpdatePlan> {
    table
        .fields
        .iter()
        .filter(|f| f.field_type == "multipleRecordLinks")
        .map(|f| build_linked_update_plan(table, f))
        .collect()
}

fn build_linked_update_plan(
    table: &RecordImportTableInput,
    field: &RecordImportFieldInput,
) -> RestoreLinkedRecordUpdatePlan {
    let update_batch_count = table
        .record_count
        .map(|count| batch_count_for(count, AIRTABLE_WRITE_BATCH_SIZE));

    let linked_table_id = field
        .linked_table_id
        .clone()
        .unwrap_or_else(|| "<unknown>".to_string());

    RestoreLinkedRecordUpdatePlan {
        table_id: table.table_id.clone(),
        table_name: table.table_name.clone(),
        field_id: field.field_id.clone(),
        field_name: field.field_name.clone(),
        linked_table_id: linked_table_id.clone(),
        linked_table_name: linked_table_id.clone(),
        update_batch_count,
        note: format!(
            "Second pass: update '{}' in '{}' with remapped record IDs from '{}'. \
            IDs become available only after first-pass creation.",
            field.field_name, table.table_name, linked_table_id
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_with_two_linked_fields() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblA".to_string(),
            table_name: "Projects".to_string(),
            record_count: Some(25),
            fields: vec![
                RecordImportFieldInput {
                    field_id: "fld01".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld02".to_string(),
                    field_name: "Tasks".to_string(),
                    field_type: "multipleRecordLinks".to_string(),
                    linked_table_id: Some("tblB".to_string()),
                },
                RecordImportFieldInput {
                    field_id: "fld03".to_string(),
                    field_name: "Owner".to_string(),
                    field_type: "multipleRecordLinks".to_string(),
                    linked_table_id: Some("tblC".to_string()),
                },
            ],
        }
    }

    fn table_no_linked() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblB".to_string(),
            table_name: "Tasks".to_string(),
            record_count: Some(10),
            fields: vec![RecordImportFieldInput {
                field_id: "fld04".to_string(),
                field_name: "Title".to_string(),
                field_type: "singleLineText".to_string(),
                linked_table_id: None,
            }],
        }
    }

    #[test]
    fn linked_fields_produce_update_plans() {
        let plans = build_linked_update_plans(&table_with_two_linked_fields());
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].field_name, "Tasks");
        assert_eq!(plans[0].linked_table_id, "tblB");
        assert_eq!(plans[1].field_name, "Owner");
    }

    #[test]
    fn no_linked_fields_produces_empty_plans() {
        let plans = build_linked_update_plans(&table_no_linked());
        assert!(plans.is_empty());
    }

    #[test]
    fn update_batch_count_computed_correctly() {
        let plans = build_linked_update_plans(&table_with_two_linked_fields());
        // 25 records / 10 per batch = 3 batches
        assert_eq!(plans[0].update_batch_count, Some(3));
    }

    #[test]
    fn update_batch_count_unknown_when_record_count_unknown() {
        let mut table = table_with_two_linked_fields();
        table.record_count = None;
        let plans = build_linked_update_plans(&table);
        assert_eq!(plans[0].update_batch_count, None);
    }

    #[test]
    fn note_references_linked_table() {
        let plans = build_linked_update_plans(&table_with_two_linked_fields());
        assert!(plans[0].note.contains("tblB"));
    }
}
