use super::models::{
    AirtableField, AirtableTable, BaseSchemaSummary, FieldTypeCount, SchemaCompatibilitySummary,
    TableSchemaSummary,
};
use std::collections::HashMap;

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

/// Builds a `SchemaCompatibilitySummary` from a slice of fields.
fn summarize_field_compatibility(fields: &[AirtableField]) -> SchemaCompatibilitySummary {
    let mut restorable_count = 0usize;
    let mut metadata_only_count = 0usize;
    let mut unknown_count = 0usize;
    for f in fields {
        match classify_field(f) {
            FieldCompatibility::Restorable => restorable_count += 1,
            FieldCompatibility::MetadataOnly => metadata_only_count += 1,
            FieldCompatibility::Unknown => unknown_count += 1,
        }
    }
    SchemaCompatibilitySummary {
        restorable_count,
        metadata_only_count,
        unknown_count,
        total_count: fields.len(),
    }
}

/// Builds a `TableSchemaSummary` from a single `AirtableTable`.
fn summarize_table(table: &AirtableTable) -> TableSchemaSummary {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for f in &table.fields {
        *counts.entry(f.field_type.clone()).or_insert(0) += 1;
    }
    let mut field_type_counts: Vec<FieldTypeCount> = counts
        .into_iter()
        .map(|(field_type, count)| FieldTypeCount { field_type, count })
        .collect();
    field_type_counts.sort_by(|a, b| a.field_type.cmp(&b.field_type));

    TableSchemaSummary {
        id: table.id.as_str().to_string(),
        name: table.name.clone(),
        field_count: table.fields.len(),
        field_type_counts,
        compatibility: summarize_field_compatibility(&table.fields),
    }
}

/// Builds a `BaseSchemaSummary` from a base id and its tables as returned by
/// the Airtable schema endpoint.
pub fn summarize_schema(base_id: &str, tables: &[AirtableTable]) -> BaseSchemaSummary {
    let table_summaries: Vec<TableSchemaSummary> = tables.iter().map(summarize_table).collect();

    let totals = SchemaCompatibilitySummary {
        restorable_count: table_summaries
            .iter()
            .map(|t| t.compatibility.restorable_count)
            .sum(),
        metadata_only_count: table_summaries
            .iter()
            .map(|t| t.compatibility.metadata_only_count)
            .sum(),
        unknown_count: table_summaries
            .iter()
            .map(|t| t.compatibility.unknown_count)
            .sum(),
        total_count: table_summaries
            .iter()
            .map(|t| t.compatibility.total_count)
            .sum(),
    };

    BaseSchemaSummary {
        base_id: base_id.to_string(),
        table_count: table_summaries.len(),
        tables: table_summaries,
        compatibility: totals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::{AirtableFieldId, AirtableTableId};

    fn field(type_str: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId("fldExample0001".to_string()),
            name: "Test Field".to_string(),
            field_type: type_str.to_string(),
            options: None,
        }
    }

    fn named_field(id: &str, name: &str, type_str: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId(id.to_string()),
            name: name.to_string(),
            field_type: type_str.to_string(),
            options: None,
        }
    }

    fn table(id: &str, name: &str, fields: Vec<AirtableField>) -> AirtableTable {
        AirtableTable {
            id: AirtableTableId(id.to_string()),
            name: name.to_string(),
            primary_field_id: None,
            fields,
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

    // ── summarize_schema tests ─────────────────────────────────────────────

    #[test]
    fn summarize_schema_empty_tables_returns_zero_counts() {
        let summary = summarize_schema("appExampleBase01", &[]);
        assert_eq!(summary.table_count, 0);
        assert_eq!(summary.compatibility.total_count, 0);
        assert_eq!(summary.base_id, "appExampleBase01");
    }

    #[test]
    fn summarize_schema_counts_tables_and_fields() {
        let t1 = table(
            "tblTestTable01",
            "Projects",
            vec![
                named_field("fld001", "Name", "singleLineText"),
                named_field("fld002", "Status", "singleSelect"),
                named_field("fld003", "Formula", "formula"),
            ],
        );
        let t2 = table(
            "tblTestTable02",
            "Tasks",
            vec![
                named_field("fld004", "Title", "singleLineText"),
                named_field("fld005", "Done", "checkbox"),
            ],
        );
        let summary = summarize_schema("appExampleBase01", &[t1, t2]);
        assert_eq!(summary.table_count, 2);
        assert_eq!(summary.compatibility.total_count, 5);
        assert_eq!(summary.compatibility.restorable_count, 4);
        assert_eq!(summary.compatibility.metadata_only_count, 1);
        assert_eq!(summary.compatibility.unknown_count, 0);
    }

    #[test]
    fn summarize_schema_field_type_counts_are_sorted() {
        let t = table(
            "tblTestTable01",
            "Mixed",
            vec![
                named_field("fld001", "A", "singleLineText"),
                named_field("fld002", "B", "checkbox"),
                named_field("fld003", "C", "singleLineText"),
            ],
        );
        let summary = summarize_schema("appExampleBase01", &[t]);
        let tc = &summary.tables[0].field_type_counts;
        assert_eq!(tc.len(), 2);
        assert_eq!(tc[0].field_type, "checkbox");
        assert_eq!(tc[0].count, 1);
        assert_eq!(tc[1].field_type, "singleLineText");
        assert_eq!(tc[1].count, 2);
    }

    #[test]
    fn summarize_schema_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_example_schema_sentinel_0123456789";
        let t = table(
            "tblTestTable01",
            "Table",
            vec![named_field("fld001", "Name", "singleLineText")],
        );
        let summary = summarize_schema("appExampleBase01", &[t]);
        let serialized = serde_json::to_string(&summary).expect("serialize");
        assert!(!serialized.contains(SENTINEL));
    }

    #[test]
    fn summarize_table_field_count_matches_field_vec() {
        let t = table(
            "tblTestTable01",
            "Contacts",
            vec![
                named_field("fld001", "Name", "singleLineText"),
                named_field("fld002", "Email", "email"),
                named_field("fld003", "Phone", "phoneNumber"),
                named_field("fld004", "Notes", "multilineText"),
                named_field("fld005", "Computed", "formula"),
            ],
        );
        let summary = summarize_schema("appExampleBase01", &[t]);
        assert_eq!(summary.tables[0].field_count, 5);
        assert_eq!(summary.tables[0].compatibility.restorable_count, 4);
        assert_eq!(summary.tables[0].compatibility.metadata_only_count, 1);
    }
}
