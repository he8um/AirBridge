use crate::restore::plan::{RestoreDryRunWarning, RestoreFieldCompatibility, RestoreFieldPlan};

/// Generates dry-run warnings from the list of field plans for a table.
pub fn warnings_for_fields(
    table_name: &str,
    fields: &[RestoreFieldPlan],
) -> Vec<RestoreDryRunWarning> {
    let mut warnings = Vec::new();

    let has_attachments = fields.iter().any(|f| {
        f.compatibility == RestoreFieldCompatibility::MetadataOnly
            && f.field_type == "multipleAttachments"
    });
    let has_linked = fields
        .iter()
        .any(|f| f.compatibility == RestoreFieldCompatibility::PartiallySupported);
    let has_computed = fields.iter().any(|f| {
        matches!(
            f.field_type.as_str(),
            "formula"
                | "rollup"
                | "lookup"
                | "count"
                | "createdTime"
                | "lastModifiedTime"
                | "autoNumber"
        )
    });
    let has_unsupported = fields
        .iter()
        .any(|f| f.compatibility == RestoreFieldCompatibility::Unsupported);
    let has_manual = fields
        .iter()
        .any(|f| f.compatibility == RestoreFieldCompatibility::ManualActionRequired);

    if has_attachments {
        warnings.push(RestoreDryRunWarning {
            code: "ATTACHMENT_METADATA_ONLY".to_string(),
            message: "Attachment fields are present. File content is not re-uploaded; only metadata is restored.".to_string(),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    if has_linked {
        warnings.push(RestoreDryRunWarning {
            code: "LINKED_RECORD_REMAPPING_REQUIRED".to_string(),
            message:
                "Linked record fields require record ID remapping after all records are imported."
                    .to_string(),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    if has_computed {
        warnings.push(RestoreDryRunWarning {
            code: "COMPUTED_FIELD_NOT_RESTORED".to_string(),
            message: "Computed fields (formula, rollup, lookup, auto-number, timestamps) schema is captured but values are not restored.".to_string(),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    if has_unsupported {
        warnings.push(RestoreDryRunWarning {
            code: "UNSUPPORTED_FIELD_MANUAL_RECREATION".to_string(),
            message:
                "One or more fields cannot be restored via the API and must be recreated manually."
                    .to_string(),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    if has_manual {
        warnings.push(RestoreDryRunWarning {
            code: "MANUAL_ACTION_REQUIRED".to_string(),
            message: "One or more fields require manual action during restore (e.g. collaborator fields).".to_string(),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::compatibility::build_field_plan;

    #[test]
    fn attachment_field_generates_warning() {
        let fields = vec![build_field_plan("fld01", "Files", "multipleAttachments")];
        let warns = warnings_for_fields("MyTable", &fields);
        assert!(warns.iter().any(|w| w.code == "ATTACHMENT_METADATA_ONLY"));
    }

    #[test]
    fn linked_record_field_generates_warning() {
        let fields = vec![build_field_plan("fld02", "Related", "multipleRecordLinks")];
        let warns = warnings_for_fields("MyTable", &fields);
        assert!(warns
            .iter()
            .any(|w| w.code == "LINKED_RECORD_REMAPPING_REQUIRED"));
    }

    #[test]
    fn formula_field_generates_unsupported_warning() {
        let fields = vec![build_field_plan("fld03", "Calc", "formula")];
        let warns = warnings_for_fields("MyTable", &fields);
        assert!(warns
            .iter()
            .any(|w| w.code == "UNSUPPORTED_FIELD_MANUAL_RECREATION"));
    }

    #[test]
    fn rollup_field_generates_computed_warning() {
        let fields = vec![build_field_plan("fld04", "Total", "rollup")];
        let warns = warnings_for_fields("MyTable", &fields);
        assert!(warns
            .iter()
            .any(|w| w.code == "COMPUTED_FIELD_NOT_RESTORED"));
    }

    #[test]
    fn plain_text_field_no_warning() {
        let fields = vec![build_field_plan("fld05", "Name", "singleLineText")];
        let warns = warnings_for_fields("MyTable", &fields);
        assert!(warns.is_empty());
    }

    #[test]
    fn collaborator_field_generates_manual_action_warning() {
        let fields = vec![build_field_plan("fld06", "Owner", "singleCollaborator")];
        let warns = warnings_for_fields("MyTable", &fields);
        assert!(warns.iter().any(|w| w.code == "MANUAL_ACTION_REQUIRED"));
    }
}
