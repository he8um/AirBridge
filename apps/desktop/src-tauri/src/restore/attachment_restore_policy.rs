use crate::restore::record_import_plan::{
    RecordImportFieldInput, RecordImportTableInput, RestoreAttachmentImportPolicy,
    RestoreAttachmentRestorePolicy,
};

/// Builds the attachment restore policies for a table.
///
/// In V0.1, all attachment fields are handled as metadata-only with a note
/// that manual re-attachment is required after restore.
pub fn build_attachment_policies(
    table: &RecordImportTableInput,
) -> Vec<RestoreAttachmentImportPolicy> {
    table
        .fields
        .iter()
        .filter(|f| f.field_type == "multipleAttachments")
        .map(|f| build_attachment_policy(table, f))
        .collect()
}

fn build_attachment_policy(
    table: &RecordImportTableInput,
    field: &RecordImportFieldInput,
) -> RestoreAttachmentImportPolicy {
    RestoreAttachmentImportPolicy {
        table_id: table.table_id.clone(),
        table_name: table.table_name.clone(),
        field_id: field.field_id.clone(),
        field_name: field.field_name.clone(),
        policy: RestoreAttachmentRestorePolicy::MetadataOnly,
        note: format!(
            "Attachment field '{}' in '{}': attachment metadata (filename, MIME type, size) \
            is captured in the backup. File bytes are not downloaded or re-uploaded. \
            Files must be manually re-attached after the restore is complete.",
            field.field_name, table.table_name
        ),
    }
}

/// Returns true if the field type is an attachment field.
pub fn is_attachment_field(field_type: &str) -> bool {
    field_type == "multipleAttachments"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_with_attachments() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblA".to_string(),
            table_name: "Projects".to_string(),
            record_count: Some(10),
            fields: vec![
                RecordImportFieldInput {
                    field_id: "fld01".to_string(),
                    field_name: "Name".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld02".to_string(),
                    field_name: "Files".to_string(),
                    field_type: "multipleAttachments".to_string(),
                    linked_table_id: None,
                },
                RecordImportFieldInput {
                    field_id: "fld03".to_string(),
                    field_name: "Docs".to_string(),
                    field_type: "multipleAttachments".to_string(),
                    linked_table_id: None,
                },
            ],
        }
    }

    fn table_no_attachments() -> RecordImportTableInput {
        RecordImportTableInput {
            table_id: "tblB".to_string(),
            table_name: "Tasks".to_string(),
            record_count: Some(5),
            fields: vec![RecordImportFieldInput {
                field_id: "fld04".to_string(),
                field_name: "Title".to_string(),
                field_type: "singleLineText".to_string(),
                linked_table_id: None,
            }],
        }
    }

    #[test]
    fn attachment_fields_produce_policies() {
        let policies = build_attachment_policies(&table_with_attachments());
        assert_eq!(policies.len(), 2);
        assert_eq!(policies[0].field_name, "Files");
        assert_eq!(
            policies[0].policy,
            RestoreAttachmentRestorePolicy::MetadataOnly
        );
    }

    #[test]
    fn no_attachment_fields_produces_empty_policies() {
        let policies = build_attachment_policies(&table_no_attachments());
        assert!(policies.is_empty());
    }

    #[test]
    fn policy_note_mentions_manual_reattachment() {
        let policies = build_attachment_policies(&table_with_attachments());
        assert!(policies[0].note.to_lowercase().contains("manual"));
    }

    #[test]
    fn is_attachment_field_detection() {
        assert!(is_attachment_field("multipleAttachments"));
        assert!(!is_attachment_field("singleLineText"));
        assert!(!is_attachment_field("multipleRecordLinks"));
    }
}
