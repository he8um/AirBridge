use crate::restore::plan::{
    RestoreAttachmentPlan, RestoreFieldCompatibility, RestoreFieldPlan, RestoreLinkedRecordPlan,
};

/// Classifies a field type into a restore compatibility tier.
pub fn classify_field(field_type: &str) -> RestoreFieldCompatibility {
    match field_type {
        // Fully restorable primitive fields
        "singleLineText" | "multilineText" | "email" | "url" | "phoneNumber" | "number"
        | "currency" | "percent" | "rating" | "checkbox" | "date" | "dateTime" | "duration"
        | "singleSelect" | "multipleSelects" | "barcode" | "count" => {
            RestoreFieldCompatibility::Supported
        }

        // Linked records: partially supported via remapping
        "multipleRecordLinks" => RestoreFieldCompatibility::PartiallySupported,

        // Attachments: metadata only, no file re-upload
        "multipleAttachments" => RestoreFieldCompatibility::MetadataOnly,

        // Computed/auto fields: metadata only, values not restorable
        "rollup" | "createdTime" | "lastModifiedTime" | "autoNumber" | "lookup" => {
            RestoreFieldCompatibility::MetadataOnly
        }

        // Unsupported: must be recreated manually
        "formula" => RestoreFieldCompatibility::Unsupported,

        // User/collaborator fields require manual action
        "singleCollaborator" | "multipleCollaborators" | "createdBy" | "lastModifiedBy" => {
            RestoreFieldCompatibility::ManualActionRequired
        }

        // Unknown field types — treat as manual action required
        _ => RestoreFieldCompatibility::ManualActionRequired,
    }
}

/// Returns a human-readable note for a field type.
pub fn field_note(field_type: &str, compat: &RestoreFieldCompatibility) -> String {
    match compat {
        RestoreFieldCompatibility::Supported => {
            format!("Field type '{field_type}' can be restored.")
        }
        RestoreFieldCompatibility::PartiallySupported => {
            "Linked record references are captured. Record ID remapping is required during restore."
                .to_string()
        }
        RestoreFieldCompatibility::MetadataOnly => match field_type {
            "multipleAttachments" => {
                "Attachment metadata is stored. File content is not re-uploaded; URLs are reference only.".to_string()
            }
            "formula" | "rollup" | "lookup" | "count" => {
                "Computed field — schema is captured. Values are not restored.".to_string()
            }
            _ => {
                format!("Field type '{field_type}' schema is captured but values cannot be restored.")
            }
        },
        RestoreFieldCompatibility::Unsupported => {
            "Formula expressions are stored in the schema backup. The field must be recreated manually.".to_string()
        }
        RestoreFieldCompatibility::ManualActionRequired => {
            format!("Field type '{field_type}' requires manual action during restore.")
        }
    }
}

/// Builds a RestoreFieldPlan for a single schema field entry.
pub fn build_field_plan(field_id: &str, field_name: &str, field_type: &str) -> RestoreFieldPlan {
    let compat = classify_field(field_type);
    let note = field_note(field_type, &compat);
    RestoreFieldPlan {
        field_id: field_id.to_string(),
        field_name: field_name.to_string(),
        field_type: field_type.to_string(),
        compatibility: compat,
        note,
    }
}

/// Builds a linked record plan for a field, if applicable.
pub fn build_linked_record_plan(
    field_id: &str,
    field_name: &str,
    linked_table_id: &str,
) -> RestoreLinkedRecordPlan {
    RestoreLinkedRecordPlan {
        field_id: field_id.to_string(),
        field_name: field_name.to_string(),
        linked_table_id: linked_table_id.to_string(),
        remapping_required: true,
        note: "Record ID references are captured. Restore requires ID remapping after all records are imported.".to_string(),
    }
}

/// Builds an attachment plan for a field.
pub fn build_attachment_plan(field_id: &str, field_name: &str) -> RestoreAttachmentPlan {
    RestoreAttachmentPlan {
        field_id: field_id.to_string(),
        field_name: field_name.to_string(),
        metadata_only: true,
        note: "Attachment metadata (filename, URL, size) is captured. File content is not re-uploaded.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_text_is_supported() {
        assert_eq!(
            classify_field("singleLineText"),
            RestoreFieldCompatibility::Supported
        );
    }

    #[test]
    fn multiple_record_links_is_partially_supported() {
        assert_eq!(
            classify_field("multipleRecordLinks"),
            RestoreFieldCompatibility::PartiallySupported
        );
    }

    #[test]
    fn multiple_attachments_is_metadata_only() {
        assert_eq!(
            classify_field("multipleAttachments"),
            RestoreFieldCompatibility::MetadataOnly
        );
    }

    #[test]
    fn formula_is_unsupported() {
        assert_eq!(
            classify_field("formula"),
            RestoreFieldCompatibility::Unsupported
        );
    }

    #[test]
    fn rollup_is_metadata_only() {
        assert_eq!(
            classify_field("rollup"),
            RestoreFieldCompatibility::MetadataOnly
        );
    }

    #[test]
    fn lookup_is_metadata_only() {
        assert_eq!(
            classify_field("lookup"),
            RestoreFieldCompatibility::MetadataOnly
        );
    }

    #[test]
    fn collaborator_is_manual_action_required() {
        assert_eq!(
            classify_field("singleCollaborator"),
            RestoreFieldCompatibility::ManualActionRequired
        );
    }

    #[test]
    fn unknown_field_type_is_manual_action_required() {
        assert_eq!(
            classify_field("unknownFutureFieldType"),
            RestoreFieldCompatibility::ManualActionRequired
        );
    }

    #[test]
    fn build_field_plan_attachment_metadata_only() {
        let plan = build_field_plan("fld01", "Files", "multipleAttachments");
        assert_eq!(plan.compatibility, RestoreFieldCompatibility::MetadataOnly);
    }

    #[test]
    fn build_field_plan_linked_partially_supported() {
        let plan = build_field_plan("fld02", "Related", "multipleRecordLinks");
        assert_eq!(
            plan.compatibility,
            RestoreFieldCompatibility::PartiallySupported
        );
    }

    #[test]
    fn build_attachment_plan_is_metadata_only() {
        let plan = build_attachment_plan("fld03", "Attachments");
        assert!(plan.metadata_only);
    }

    #[test]
    fn build_linked_record_plan_requires_remapping() {
        let plan = build_linked_record_plan("fld04", "Links", "tblTarget01");
        assert!(plan.remapping_required);
    }
}
