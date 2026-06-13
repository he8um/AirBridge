use crate::restore::record_import_plan::{RestoreRecordImportWarning, RestoreTableImportPlan};

/// Generates warnings for a table's record import plan.
pub fn warnings_for_table_import(
    table: &RestoreTableImportPlan,
) -> Vec<RestoreRecordImportWarning> {
    let mut warnings = Vec::new();

    if !table.record_count_known {
        warnings.push(RestoreRecordImportWarning {
            code: "RECORD_COUNT_UNKNOWN".to_string(),
            message: format!(
                "Record count for '{}' is not available in the package manifest. \
                Batch count cannot be calculated until import begins.",
                table.table_name
            ),
            table_name: Some(table.table_name.clone()),
            field_name: None,
        });
    }

    if !table.attachment_policies.is_empty() {
        warnings.push(RestoreRecordImportWarning {
            code: "ATTACHMENT_METADATA_ONLY".to_string(),
            message: format!(
                "'{}' contains attachment fields. Attachment file bytes are not restored — \
                only metadata is captured. Files must be manually re-attached.",
                table.table_name
            ),
            table_name: Some(table.table_name.clone()),
            field_name: None,
        });
    }

    let skipped_count = table
        .field_policies
        .iter()
        .filter(|p| {
            p.policy == crate::restore::record_import_plan::RestoreRecordFieldImportPolicy::Skip
        })
        .count();

    if skipped_count > 0 {
        warnings.push(RestoreRecordImportWarning {
            code: "COMPUTED_FIELDS_SKIPPED".to_string(),
            message: format!(
                "{skipped_count} field(s) in '{}' will be skipped during import — they are \
                computed or read-only and cannot be set via the Airtable API.",
                table.table_name
            ),
            table_name: Some(table.table_name.clone()),
            field_name: None,
        });
    }

    if !table.linked_record_updates.is_empty() {
        warnings.push(RestoreRecordImportWarning {
            code: "LINKED_RECORD_SECOND_PASS_REQUIRED".to_string(),
            message: format!(
                "'{}' has {} linked record field(s) that require a second pass after all \
                records are created and ID mapping is complete.",
                table.table_name,
                table.linked_record_updates.len()
            ),
            table_name: Some(table.table_name.clone()),
            field_name: None,
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::record_import_plan::{
        RecordImportFieldInput, RecordImportTableInput, RestoreAttachmentImportPolicy,
        RestoreAttachmentRestorePolicy, RestoreLinkedRecordUpdatePlan, RestoreRecordBatchPhase,
        RestoreRecordFieldImportPolicy, RestoreRecordFieldPolicy,
        RestoreRecordImportCheckpointPlan, RestoreRecordMappingPlan, RestoreRecordMappingStrategy,
    };

    fn minimal_checkpoint() -> RestoreRecordImportCheckpointPlan {
        RestoreRecordImportCheckpointPlan {
            table_id: "tbl01".to_string(),
            table_name: "T".to_string(),
            checkpoint_batch_index: 0,
            source_record_id_offset_placeholder: "<x>".to_string(),
            completed_phase: RestoreRecordBatchPhase::CreateRecords,
            note: String::new(),
        }
    }

    fn minimal_mapping() -> RestoreRecordMappingPlan {
        RestoreRecordMappingPlan {
            table_id: "tbl01".to_string(),
            table_name: "T".to_string(),
            strategy: RestoreRecordMappingStrategy::MapSourceRecordIdToCreatedRecordId,
            remapping_required: false,
            note: String::new(),
        }
    }

    fn make_table(
        record_count_known: bool,
        has_attachments: bool,
        skipped_count: usize,
        linked_count: usize,
    ) -> RestoreTableImportPlan {
        let field_policies = (0..skipped_count)
            .map(|i| RestoreRecordFieldPolicy {
                field_id: format!("fld_skip_{i}"),
                field_name: format!("Computed {i}"),
                field_type: "formula".to_string(),
                policy: RestoreRecordFieldImportPolicy::Skip,
                note: String::new(),
            })
            .collect();

        let attachment_policies = if has_attachments {
            vec![RestoreAttachmentImportPolicy {
                table_id: "tbl01".to_string(),
                table_name: "T".to_string(),
                field_id: "fld_att".to_string(),
                field_name: "Files".to_string(),
                policy: RestoreAttachmentRestorePolicy::MetadataOnly,
                note: String::new(),
            }]
        } else {
            vec![]
        };

        let linked_record_updates = (0..linked_count)
            .map(|i| RestoreLinkedRecordUpdatePlan {
                table_id: "tbl01".to_string(),
                table_name: "T".to_string(),
                field_id: format!("fld_link_{i}"),
                field_name: format!("Link {i}"),
                linked_table_id: "tblX".to_string(),
                linked_table_name: "tblX".to_string(),
                update_batch_count: None,
                note: String::new(),
            })
            .collect();

        RestoreTableImportPlan {
            table_id: "tbl01".to_string(),
            table_name: "T".to_string(),
            import_order: 0,
            record_count: if record_count_known { Some(10) } else { None },
            record_count_known,
            batch_size: 10,
            create_batch_count: if record_count_known { Some(1) } else { None },
            update_batch_count: None,
            first_pass_batches: vec![],
            second_pass_batches: vec![],
            field_policies,
            attachment_policies,
            mapping_plan: minimal_mapping(),
            checkpoint_plan: minimal_checkpoint(),
            linked_record_updates,
        }
    }

    #[test]
    fn unknown_record_count_produces_warning() {
        let table = make_table(false, false, 0, 0);
        let warns = warnings_for_table_import(&table);
        assert!(warns.iter().any(|w| w.code == "RECORD_COUNT_UNKNOWN"));
    }

    #[test]
    fn known_record_count_no_count_warning() {
        let table = make_table(true, false, 0, 0);
        let warns = warnings_for_table_import(&table);
        assert!(!warns.iter().any(|w| w.code == "RECORD_COUNT_UNKNOWN"));
    }

    #[test]
    fn attachment_fields_produce_warning() {
        let table = make_table(true, true, 0, 0);
        let warns = warnings_for_table_import(&table);
        assert!(warns.iter().any(|w| w.code == "ATTACHMENT_METADATA_ONLY"));
    }

    #[test]
    fn skipped_fields_produce_warning() {
        let table = make_table(true, false, 2, 0);
        let warns = warnings_for_table_import(&table);
        assert!(warns.iter().any(|w| w.code == "COMPUTED_FIELDS_SKIPPED"));
    }

    #[test]
    fn linked_fields_produce_second_pass_warning() {
        let table = make_table(true, false, 0, 2);
        let warns = warnings_for_table_import(&table);
        assert!(warns
            .iter()
            .any(|w| w.code == "LINKED_RECORD_SECOND_PASS_REQUIRED"));
    }

    #[test]
    fn no_issues_produces_no_warnings() {
        let table = make_table(true, false, 0, 0);
        let warns = warnings_for_table_import(&table);
        assert!(warns.is_empty());
    }
}
