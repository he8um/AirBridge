use crate::restore::record_import_plan::{
    RecordImportTableInput, RestoreRecordMappingPlan, RestoreRecordMappingStrategy,
};

/// Builds the record ID mapping plan for a table.
///
/// The mapping is an output of execution — IDs cannot be known until records
/// are actually created. This plan describes the strategy that will be applied.
pub fn build_mapping_plan(table: &RecordImportTableInput) -> RestoreRecordMappingPlan {
    let has_linked = table
        .fields
        .iter()
        .any(|f| f.field_type == "multipleRecordLinks");

    RestoreRecordMappingPlan {
        table_id: table.table_id.clone(),
        table_name: table.table_name.clone(),
        strategy: RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId,
        remapping_required: has_linked,
        note: if has_linked {
            format!(
                "Each record created in '{}' receives a new Airtable record ID. The mapping from \
                the source record ID to the new ID is collected at execution time. Linked record \
                fields are updated in a second pass after all records are created.",
                table.table_name
            )
        } else {
            format!(
                "Each record created in '{}' receives a new Airtable record ID. No linked record \
                fields require ID remapping in this table.",
                table.table_name
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::record_import_plan::RecordImportFieldInput;

    fn table_with_linked() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblA".to_string(),
            table_name: "Projects".to_string(),
            record_count: Some(20),
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
            ],
        }
    }

    fn table_without_linked() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblB".to_string(),
            table_name: "Tasks".to_string(),
            record_count: Some(5),
            fields: vec![RecordImportFieldInput {
                field_id: "fld03".to_string(),
                field_name: "Title".to_string(),
                field_type: "singleLineText".to_string(),
                linked_table_id: None,
            }],
        }
    }

    #[test]
    fn mapping_plan_with_linked_fields_requires_remapping() {
        let plan = build_mapping_plan(&table_with_linked());
        assert!(plan.remapping_required);
        assert_eq!(
            plan.strategy,
            RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId
        );
    }

    #[test]
    fn mapping_plan_without_linked_fields_no_remapping() {
        let plan = build_mapping_plan(&table_without_linked());
        assert!(!plan.remapping_required);
        assert_eq!(
            plan.strategy,
            RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId
        );
    }

    #[test]
    fn mapping_plan_note_is_non_empty() {
        let plan = build_mapping_plan(&table_with_linked());
        assert!(!plan.note.is_empty());
    }

    #[test]
    fn mapping_strategy_is_not_unavailable() {
        let plan = build_mapping_plan(&table_with_linked());
        assert_ne!(
            plan.strategy,
            RestoreRecordMappingStrategy::UnavailableUntilExecution
        );
    }

    #[test]
    fn mapping_plan_preserves_table_id_and_name() {
        let plan = build_mapping_plan(&table_with_linked());
        assert_eq!(plan.table_id, "tblA");
        assert_eq!(plan.table_name, "Projects");
    }
}
