use crate::restore::schema_plan::{
    RestoreFieldCreateClassification, RestoreFieldCreationStep, RestoreSchemaWarning,
};

/// Generates schema planning warnings from the classified field steps.
pub fn warnings_for_schema_steps(
    table_name: &str,
    field_steps: &[RestoreFieldCreationStep],
    deferred_count: usize,
    manual_count: usize,
    unsupported_count: usize,
) -> Vec<RestoreSchemaWarning> {
    let mut warnings = Vec::new();

    let has_attachments = field_steps.iter().any(|f| {
        f.classification == RestoreFieldCreateClassification::MetadataOnly
            && f.field_type == "multipleAttachments"
    });

    if has_attachments {
        warnings.push(RestoreSchemaWarning {
            code: "ATTACHMENT_METADATA_ONLY".to_string(),
            message: "Attachment fields are present. File content is not re-uploaded; only metadata is captured.".to_string(),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    if deferred_count > 0 {
        warnings.push(RestoreSchemaWarning {
            code: "LINKED_FIELDS_DEFERRED".to_string(),
            message: format!(
                "{deferred_count} linked record field(s) in '{table_name}' will be deferred until all tables exist."
            ),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    if unsupported_count > 0 {
        warnings.push(RestoreSchemaWarning {
            code: "UNSUPPORTED_FIELDS_REQUIRE_MANUAL_RECREATION".to_string(),
            message: format!(
                "{unsupported_count} field(s) in '{table_name}' cannot be created via the API and must be recreated manually."
            ),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    if manual_count > 0 {
        warnings.push(RestoreSchemaWarning {
            code: "MANUAL_ACTION_REQUIRED".to_string(),
            message: format!(
                "{manual_count} field(s) in '{table_name}' require manual action during restore."
            ),
            table_name: Some(table_name.to_string()),
            field_name: None,
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::schema_plan::RestoreFieldCreateClassification;
    use crate::restore::schema_steps::build_field_step;

    fn attachment_step() -> RestoreFieldCreationStep {
        build_field_step(
            "fld01",
            "Files",
            "multipleAttachments",
            "tbl01",
            "MyTable",
            RestoreFieldCreateClassification::MetadataOnly,
        )
    }

    fn direct_step() -> RestoreFieldCreationStep {
        build_field_step(
            "fld02",
            "Name",
            "singleLineText",
            "tbl01",
            "MyTable",
            RestoreFieldCreateClassification::CreateDirectly,
        )
    }

    #[test]
    fn attachment_produces_metadata_only_warning() {
        let steps = vec![attachment_step()];
        let warns = warnings_for_schema_steps("MyTable", &steps, 0, 0, 0);
        assert!(warns.iter().any(|w| w.code == "ATTACHMENT_METADATA_ONLY"));
    }

    #[test]
    fn deferred_count_produces_linked_warning() {
        let steps = vec![direct_step()];
        let warns = warnings_for_schema_steps("MyTable", &steps, 2, 0, 0);
        assert!(warns.iter().any(|w| w.code == "LINKED_FIELDS_DEFERRED"));
    }

    #[test]
    fn unsupported_produces_manual_recreation_warning() {
        let steps = vec![direct_step()];
        let warns = warnings_for_schema_steps("MyTable", &steps, 0, 0, 1);
        assert!(warns
            .iter()
            .any(|w| w.code == "UNSUPPORTED_FIELDS_REQUIRE_MANUAL_RECREATION"));
    }

    #[test]
    fn manual_count_produces_manual_action_warning() {
        let steps = vec![direct_step()];
        let warns = warnings_for_schema_steps("MyTable", &steps, 0, 1, 0);
        assert!(warns.iter().any(|w| w.code == "MANUAL_ACTION_REQUIRED"));
    }

    #[test]
    fn direct_only_fields_produce_no_warnings() {
        let steps = vec![direct_step()];
        let warns = warnings_for_schema_steps("MyTable", &steps, 0, 0, 0);
        assert!(warns.is_empty());
    }

    #[test]
    fn multiple_issues_produce_multiple_warnings() {
        let steps = vec![attachment_step()];
        let warns = warnings_for_schema_steps("MyTable", &steps, 1, 1, 1);
        assert!(warns.len() >= 3);
    }
}
