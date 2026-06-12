use crate::restore::schema_plan::{
    RestoreDeferredFieldStep, RestoreFieldCreateClassification, RestoreFieldCreationStep,
    RestoreManualActionField, RestoreTableCreationStep,
};

/// Classifies a field type into a schema creation classification.
pub fn classify_field_for_schema(field_type: &str) -> RestoreFieldCreateClassification {
    match field_type {
        // Primitive fields — create directly via the API.
        "singleLineText" | "multilineText" | "email" | "url" | "phoneNumber" | "number"
        | "currency" | "percent" | "rating" | "checkbox" | "date" | "dateTime" | "duration"
        | "barcode" => RestoreFieldCreateClassification::CreateDirectly,

        // Select fields — create directly; choices are set in the field config.
        "singleSelect" | "multipleSelects" => RestoreFieldCreateClassification::CreateDirectly,

        // Linked records — must be deferred until target tables are created.
        "multipleRecordLinks" => RestoreFieldCreateClassification::DeferUntilTablesExist,

        // Attachments — metadata only; no file re-upload.
        "multipleAttachments" => RestoreFieldCreateClassification::MetadataOnly,

        // Auto-number and timestamps — metadata captured; cannot be set via API.
        "autoNumber" | "createdTime" | "lastModifiedTime" | "count" => {
            RestoreFieldCreateClassification::MetadataOnly
        }

        // Computed fields — schema captured; expressions not restorable via API.
        "formula" | "rollup" | "lookup" => RestoreFieldCreateClassification::Unsupported,

        // Collaborator fields — require manual setup.
        "singleCollaborator" | "multipleCollaborators" | "createdBy" | "lastModifiedBy" => {
            RestoreFieldCreateClassification::ManualActionRequired
        }

        // Unknown — conservative manual action.
        _ => RestoreFieldCreateClassification::ManualActionRequired,
    }
}

/// Returns a description note for a field classification.
pub fn classification_note(field_type: &str, class: &RestoreFieldCreateClassification) -> String {
    match class {
        RestoreFieldCreateClassification::CreateDirectly => {
            format!("'{field_type}' can be created directly via the Airtable API.")
        }
        RestoreFieldCreateClassification::CreateWithAdjustment => {
            format!("'{field_type}' can be created with minor configuration adjustments.")
        }
        RestoreFieldCreateClassification::DeferUntilTablesExist => {
            "Linked record field — must be created after all target tables exist.".to_string()
        }
        RestoreFieldCreateClassification::MetadataOnly => match field_type {
            "multipleAttachments" => {
                "Attachment metadata is captured. File content is not re-uploaded.".to_string()
            }
            _ => format!("'{field_type}' schema is captured but values cannot be set via the API."),
        },
        RestoreFieldCreateClassification::Unsupported => {
            format!("'{field_type}' cannot be created via the Airtable API. Must be recreated manually.")
        }
        RestoreFieldCreateClassification::ManualActionRequired => {
            format!("'{field_type}' requires manual action during restore.")
        }
    }
}

/// Builds a table creation step.
pub fn build_table_step(
    table_id: &str,
    table_name: &str,
    step_index: usize,
    field_count: usize,
    direct: usize,
    deferred: usize,
    manual: usize,
    unsupported: usize,
) -> RestoreTableCreationStep {
    RestoreTableCreationStep {
        table_id: table_id.to_string(),
        table_name: table_name.to_string(),
        step_index,
        field_count,
        direct_field_count: direct,
        deferred_field_count: deferred,
        manual_action_count: manual,
        unsupported_count: unsupported,
        note: format!(
            "Create table '{table_name}': {direct} direct, {deferred} deferred, {manual} manual, {unsupported} unsupported."
        ),
    }
}

/// Builds a field creation step for a directly-creatable field.
pub fn build_field_step(
    field_id: &str,
    field_name: &str,
    field_type: &str,
    table_id: &str,
    table_name: &str,
    classification: RestoreFieldCreateClassification,
) -> RestoreFieldCreationStep {
    let note = classification_note(field_type, &classification);
    RestoreFieldCreationStep {
        field_id: field_id.to_string(),
        field_name: field_name.to_string(),
        field_type: field_type.to_string(),
        table_id: table_id.to_string(),
        table_name: table_name.to_string(),
        classification,
        note,
    }
}

/// Builds a deferred field step.
pub fn build_deferred_step(
    field_id: &str,
    field_name: &str,
    field_type: &str,
    table_id: &str,
    table_name: &str,
    linked_table_id: Option<String>,
) -> RestoreDeferredFieldStep {
    RestoreDeferredFieldStep {
        field_id: field_id.to_string(),
        field_name: field_name.to_string(),
        field_type: field_type.to_string(),
        table_id: table_id.to_string(),
        table_name: table_name.to_string(),
        reason: "Linked record field — deferred until all tables and records exist.".to_string(),
        linked_table_id,
    }
}

/// Builds a manual action field entry.
pub fn build_manual_action_field(
    field_id: &str,
    field_name: &str,
    field_type: &str,
    table_id: &str,
    table_name: &str,
) -> RestoreManualActionField {
    let description = match field_type {
        "formula" | "rollup" | "lookup" => {
            format!(
                "'{field_type}' must be recreated manually — cannot be set via the Airtable API."
            )
        }
        "singleCollaborator" | "multipleCollaborators" | "createdBy" | "lastModifiedBy" => {
            format!("'{field_type}' requires manual collaborator assignment.")
        }
        _ => format!("'{field_type}' requires manual action."),
    };
    RestoreManualActionField {
        field_id: field_id.to_string(),
        field_name: field_name.to_string(),
        field_type: field_type.to_string(),
        table_id: table_id.to_string(),
        table_name: table_name.to_string(),
        action_description: description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_text_is_create_directly() {
        assert_eq!(
            classify_field_for_schema("singleLineText"),
            RestoreFieldCreateClassification::CreateDirectly
        );
    }

    #[test]
    fn single_select_is_create_directly() {
        assert_eq!(
            classify_field_for_schema("singleSelect"),
            RestoreFieldCreateClassification::CreateDirectly
        );
    }

    #[test]
    fn linked_record_is_deferred() {
        assert_eq!(
            classify_field_for_schema("multipleRecordLinks"),
            RestoreFieldCreateClassification::DeferUntilTablesExist
        );
    }

    #[test]
    fn attachment_is_metadata_only() {
        assert_eq!(
            classify_field_for_schema("multipleAttachments"),
            RestoreFieldCreateClassification::MetadataOnly
        );
    }

    #[test]
    fn formula_is_unsupported() {
        assert_eq!(
            classify_field_for_schema("formula"),
            RestoreFieldCreateClassification::Unsupported
        );
    }

    #[test]
    fn rollup_is_unsupported() {
        assert_eq!(
            classify_field_for_schema("rollup"),
            RestoreFieldCreateClassification::Unsupported
        );
    }

    #[test]
    fn lookup_is_unsupported() {
        assert_eq!(
            classify_field_for_schema("lookup"),
            RestoreFieldCreateClassification::Unsupported
        );
    }

    #[test]
    fn collaborator_is_manual_action() {
        assert_eq!(
            classify_field_for_schema("singleCollaborator"),
            RestoreFieldCreateClassification::ManualActionRequired
        );
    }

    #[test]
    fn unknown_type_is_manual_action() {
        assert_eq!(
            classify_field_for_schema("futureMysteryField"),
            RestoreFieldCreateClassification::ManualActionRequired
        );
    }

    #[test]
    fn auto_number_is_metadata_only() {
        assert_eq!(
            classify_field_for_schema("autoNumber"),
            RestoreFieldCreateClassification::MetadataOnly
        );
    }

    #[test]
    fn created_time_is_metadata_only() {
        assert_eq!(
            classify_field_for_schema("createdTime"),
            RestoreFieldCreateClassification::MetadataOnly
        );
    }

    #[test]
    fn build_table_step_fields_match() {
        let step = build_table_step("tbl01", "Projects", 0, 5, 3, 1, 1, 0);
        assert_eq!(step.table_id, "tbl01");
        assert_eq!(step.direct_field_count, 3);
        assert_eq!(step.deferred_field_count, 1);
        assert_eq!(step.manual_action_count, 1);
    }

    #[test]
    fn build_field_step_note_is_non_empty() {
        let step = build_field_step(
            "fld01",
            "Name",
            "singleLineText",
            "tbl01",
            "Projects",
            RestoreFieldCreateClassification::CreateDirectly,
        );
        assert!(!step.note.is_empty());
    }

    #[test]
    fn build_deferred_step_has_reason() {
        let step = build_deferred_step(
            "fld02",
            "Related",
            "multipleRecordLinks",
            "tbl01",
            "Projects",
            Some("tblTarget01".to_string()),
        );
        assert!(!step.reason.is_empty());
        assert_eq!(step.linked_table_id, Some("tblTarget01".to_string()));
    }

    #[test]
    fn build_manual_action_field_formula_describes_recreation() {
        let field = build_manual_action_field("fld03", "Calc", "formula", "tbl01", "Projects");
        assert!(
            field.action_description.contains("recreated manually")
                || field.action_description.contains("API")
        );
    }
}
