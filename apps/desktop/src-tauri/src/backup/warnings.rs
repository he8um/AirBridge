use crate::airtable::models::AirtableField;
use crate::models::backup_plan::{BackupPlanWarning, WarningSeverity};

/// Field types that produce formula/computed warnings.
const COMPUTED_TYPES: &[&str] = &["formula", "rollup", "count", "lookup"];

/// System/audit field types.
const SYSTEM_TYPES: &[&str] = &[
    "createdTime",
    "lastModifiedTime",
    "createdBy",
    "lastModifiedBy",
    "autoNumber",
    "externalSyncSource",
];

/// Attachment field type.
const ATTACHMENT_TYPE: &str = "multipleAttachments";

/// Linked record field type.
const LINKED_RECORD_TYPE: &str = "multipleRecordLinks";

/// Generates all warnings for a single field. Returns zero or more warnings.
pub fn warnings_for_field(table_name: &str, field: &AirtableField) -> Vec<BackupPlanWarning> {
    let mut out = Vec::new();
    let ft = field.field_type.as_str();

    if COMPUTED_TYPES.contains(&ft) {
        out.push(BackupPlanWarning {
            severity: WarningSeverity::Info,
            code: "COMPUTED_FIELD".to_string(),
            message: format!(
                "Table \"{table_name}\", field \"{}\": {} field — schema captured, \
                 computed value cannot be restored.",
                field.name, field.field_type
            ),
            table_name: Some(table_name.to_string()),
            field_name: Some(field.name.clone()),
        });
    } else if SYSTEM_TYPES.contains(&ft) {
        out.push(BackupPlanWarning {
            severity: WarningSeverity::Info,
            code: "SYSTEM_FIELD".to_string(),
            message: format!(
                "Table \"{table_name}\", field \"{}\": {} field — schema captured, \
                 system-managed value cannot be restored.",
                field.name, field.field_type
            ),
            table_name: Some(table_name.to_string()),
            field_name: Some(field.name.clone()),
        });
    } else if ft == ATTACHMENT_TYPE {
        out.push(BackupPlanWarning {
            severity: WarningSeverity::Warning,
            code: "ATTACHMENT_METADATA_ONLY".to_string(),
            message: format!(
                "Table \"{table_name}\", field \"{}\": attachment field — \
                 attachment metadata only; file content is not exported in this version.",
                field.name
            ),
            table_name: Some(table_name.to_string()),
            field_name: Some(field.name.clone()),
        });
    } else if ft == LINKED_RECORD_TYPE {
        out.push(BackupPlanWarning {
            severity: WarningSeverity::Warning,
            code: "LINKED_RECORD_REMAPPING".to_string(),
            message: format!(
                "Table \"{table_name}\", field \"{}\": linked record field — \
                 record ID references are captured; restore will require remapping \
                 to new record IDs.",
                field.name
            ),
            table_name: Some(table_name.to_string()),
            field_name: Some(field.name.clone()),
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::AirtableFieldId;

    fn field(type_str: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId("fldExample0001".to_string()),
            name: "TestField".to_string(),
            field_type: type_str.to_string(),
            options: None,
        }
    }

    #[test]
    fn formula_field_produces_computed_warning() {
        let w = warnings_for_field("Table1", &field("formula"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "COMPUTED_FIELD");
        assert!(matches!(w[0].severity, WarningSeverity::Info));
    }

    #[test]
    fn rollup_field_produces_computed_warning() {
        let w = warnings_for_field("Table1", &field("rollup"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "COMPUTED_FIELD");
    }

    #[test]
    fn count_field_produces_computed_warning() {
        let w = warnings_for_field("Table1", &field("count"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "COMPUTED_FIELD");
    }

    #[test]
    fn lookup_field_produces_computed_warning() {
        let w = warnings_for_field("Table1", &field("lookup"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "COMPUTED_FIELD");
    }

    #[test]
    fn created_time_produces_system_warning() {
        let w = warnings_for_field("Table1", &field("createdTime"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "SYSTEM_FIELD");
    }

    #[test]
    fn last_modified_time_produces_system_warning() {
        let w = warnings_for_field("Table1", &field("lastModifiedTime"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "SYSTEM_FIELD");
    }

    #[test]
    fn attachment_produces_metadata_only_warning() {
        let w = warnings_for_field("Table1", &field("multipleAttachments"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "ATTACHMENT_METADATA_ONLY");
        assert!(matches!(w[0].severity, WarningSeverity::Warning));
    }

    #[test]
    fn linked_record_produces_remapping_warning() {
        let w = warnings_for_field("Table1", &field("multipleRecordLinks"));
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].code, "LINKED_RECORD_REMAPPING");
        assert!(matches!(w[0].severity, WarningSeverity::Warning));
    }

    #[test]
    fn restorable_field_produces_no_warnings() {
        let types = ["singleLineText", "number", "checkbox", "date", "email"];
        for t in types {
            let w = warnings_for_field("Table1", &field(t));
            assert!(w.is_empty(), "expected no warnings for {t}");
        }
    }

    #[test]
    fn warning_message_includes_table_and_field_name() {
        let w = warnings_for_field("MyTable", &field("formula"));
        assert!(w[0].message.contains("MyTable"));
        assert!(w[0].message.contains("TestField"));
    }

    #[test]
    fn warning_table_name_matches_input() {
        let w = warnings_for_field("Projects", &field("rollup"));
        assert_eq!(w[0].table_name.as_deref(), Some("Projects"));
    }
}
