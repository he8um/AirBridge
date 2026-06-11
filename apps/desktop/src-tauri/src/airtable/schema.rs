use super::models::AirtableField;

/// Field types that are fully restorable via the Airtable API.
const RESTORABLE_TYPES: &[&str] = &[
    "singleLineText",
    "multilineText",
    "number",
    "currency",
    "percent",
    "singleSelect",
    "multipleSelects",
    "checkbox",
    "date",
    "dateTime",
    "duration",
    "email",
    "url",
    "phoneNumber",
    "rating",
];

/// Field types that are captured in schema backups but whose computed values
/// cannot be restored via the API.
const METADATA_ONLY_TYPES: &[&str] = &[
    "formula",
    "rollup",
    "count",
    "lookup",
    "createdTime",
    "lastModifiedTime",
    "createdBy",
    "lastModifiedBy",
    "autoNumber",
    "externalSyncSource",
];

/// Compatibility classification for a single field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldCompatibility {
    /// Field can be fully backed up and restored.
    Restorable,
    /// Field schema is captured but values cannot be restored.
    MetadataOnly,
    /// Field type is not recognized; treat conservatively.
    Unknown,
}

/// Classify a field's restore compatibility based on its type string.
pub fn classify_field(field: &AirtableField) -> FieldCompatibility {
    if RESTORABLE_TYPES.contains(&field.field_type.as_str()) {
        return FieldCompatibility::Restorable;
    }
    if METADATA_ONLY_TYPES.contains(&field.field_type.as_str()) {
        return FieldCompatibility::MetadataOnly;
    }
    FieldCompatibility::Unknown
}

/// Returns `true` if every field in `fields` is fully restorable.
pub fn all_fields_restorable(fields: &[AirtableField]) -> bool {
    fields
        .iter()
        .all(|f| classify_field(f) == FieldCompatibility::Restorable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::AirtableFieldId;

    fn field(type_str: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId("fldExample0001".to_string()),
            name: "Test Field".to_string(),
            field_type: type_str.to_string(),
            options: None,
        }
    }

    #[test]
    fn single_line_text_is_restorable() {
        assert_eq!(
            classify_field(&field("singleLineText")),
            FieldCompatibility::Restorable
        );
    }

    #[test]
    fn formula_is_metadata_only() {
        assert_eq!(
            classify_field(&field("formula")),
            FieldCompatibility::MetadataOnly
        );
    }

    #[test]
    fn unknown_type_is_unknown() {
        assert_eq!(
            classify_field(&field("someNewFutureType")),
            FieldCompatibility::Unknown
        );
    }

    #[test]
    fn all_restorable_when_all_simple_types() {
        let fields = vec![field("singleLineText"), field("number"), field("checkbox")];
        assert!(all_fields_restorable(&fields));
    }

    #[test]
    fn not_all_restorable_when_formula_present() {
        let fields = vec![field("singleLineText"), field("formula")];
        assert!(!all_fields_restorable(&fields));
    }
}
