use crate::airtable::models::{AirtableField, AirtableTable};
use crate::airtable::schema::classify_field;
use crate::backup::estimates::build_estimate;
use crate::backup::warnings::warnings_for_field;
use crate::models::backup_plan::{
    AttachmentPolicy, BackupPlan, BackupPlanCompatibilitySummary, BackupPlanField, BackupPlanTable,
    BackupPlanWarning, BackupScope, LinkedRecordPolicy,
};

/// Classifies a field and returns the appropriate attachment/linked-record policy.
fn attachment_policy_for(field: &AirtableField) -> Option<AttachmentPolicy> {
    if field.field_type == "multipleAttachments" {
        Some(AttachmentPolicy::MetadataOnly)
    } else {
        None
    }
}

fn linked_record_policy_for(field: &AirtableField) -> Option<LinkedRecordPolicy> {
    if field.field_type == "multipleRecordLinks" {
        Some(LinkedRecordPolicy::RemappingRequiredForRestore)
    } else {
        None
    }
}

/// Builds a `BackupPlanField` for a single Airtable field.
fn plan_field(field: &AirtableField) -> BackupPlanField {
    use crate::airtable::schema::FieldCompatibility;

    let compat = classify_field(field);
    let compatibility_label = match compat {
        FieldCompatibility::Restorable => "restorable".to_string(),
        FieldCompatibility::MetadataOnly => "metadataOnly".to_string(),
        FieldCompatibility::Unknown => "unknown".to_string(),
    };

    BackupPlanField {
        id: field.id.0.clone(),
        name: field.name.clone(),
        field_type: field.field_type.clone(),
        compatibility: compatibility_label,
        attachment_policy: attachment_policy_for(field),
        linked_record_policy: linked_record_policy_for(field),
    }
}

/// Builds a `BackupPlanTable` from an `AirtableTable`.
fn plan_table(table: &AirtableTable, record_count: Option<usize>) -> BackupPlanTable {
    let fields: Vec<BackupPlanField> = table.fields.iter().map(plan_field).collect();

    let mut restorable = 0usize;
    let mut metadata_only = 0usize;
    let mut unknown = 0usize;
    for f in &fields {
        match f.compatibility.as_str() {
            "restorable" => restorable += 1,
            "metadataOnly" => metadata_only += 1,
            _ => unknown += 1,
        }
    }

    let warnings: Vec<BackupPlanWarning> = table
        .fields
        .iter()
        .flat_map(|f| warnings_for_field(&table.name, f))
        .collect();

    BackupPlanTable {
        id: table.id.as_str().to_string(),
        name: table.name.clone(),
        field_count: fields.len(),
        record_count,
        fields,
        warnings,
        compatibility: BackupPlanCompatibilitySummary {
            restorable_count: restorable,
            metadata_only_count: metadata_only,
            unknown_count: unknown,
            total_count: restorable + metadata_only + unknown,
        },
    }
}

/// Produces a `BackupPlan` from a base id, name, list of tables, and optional scope.
///
/// `record_counts` must be the same length as `tables` when provided; each entry
/// corresponds to the matching table. Pass an empty slice to mark all counts unknown.
pub fn create_plan(
    base_id: &str,
    base_name: &str,
    tables: &[AirtableTable],
    record_counts: &[Option<usize>],
    scope: BackupScope,
) -> BackupPlan {
    // Pair each table with its optional record count; default to unknown.
    let plan_tables: Vec<BackupPlanTable> = tables
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let rc = record_counts.get(i).copied().flatten();
            plan_table(
                t,
                if record_counts.is_empty() {
                    None
                } else {
                    Some(rc).flatten().or(None)
                },
            )
        })
        .collect();

    // Re-collect per-table counts for estimate (respecting unknown).
    let per_table_counts: Vec<Option<usize>> = if record_counts.is_empty() {
        tables.iter().map(|_| None).collect()
    } else {
        tables
            .iter()
            .enumerate()
            .map(|(i, _)| record_counts.get(i).copied().flatten())
            .collect()
    };

    let estimate = build_estimate(&per_table_counts);

    let total_fields: usize = plan_tables.iter().map(|t| t.field_count).sum();
    let all_warnings: Vec<BackupPlanWarning> = plan_tables
        .iter()
        .flat_map(|t| t.warnings.clone())
        .collect();

    let global_compat = BackupPlanCompatibilitySummary {
        restorable_count: plan_tables
            .iter()
            .map(|t| t.compatibility.restorable_count)
            .sum(),
        metadata_only_count: plan_tables
            .iter()
            .map(|t| t.compatibility.metadata_only_count)
            .sum(),
        unknown_count: plan_tables
            .iter()
            .map(|t| t.compatibility.unknown_count)
            .sum(),
        total_count: total_fields,
    };

    BackupPlan {
        base_id: base_id.to_string(),
        base_name: base_name.to_string(),
        scope,
        table_count: plan_tables.len(),
        total_field_count: total_fields,
        tables: plan_tables,
        compatibility: global_compat,
        warnings: all_warnings,
        estimate,
        dry_run: true,
        output_package_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::{AirtableFieldId, AirtableTableId};

    fn field(id: &str, name: &str, type_str: &str) -> AirtableField {
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
    fn plan_includes_all_tables_by_default() {
        let tables = vec![
            table(
                "tbl01",
                "Projects",
                vec![field("f01", "Name", "singleLineText")],
            ),
            table(
                "tbl02",
                "Tasks",
                vec![field("f02", "Title", "singleLineText")],
            ),
        ];
        let plan = create_plan("appEx01", "MyBase", &tables, &[], BackupScope::Full);
        assert_eq!(plan.table_count, 2);
        assert_eq!(plan.tables.len(), 2);
    }

    #[test]
    fn plan_includes_all_fields_by_default() {
        let t = table(
            "tbl01",
            "Projects",
            vec![
                field("f01", "Name", "singleLineText"),
                field("f02", "Status", "singleSelect"),
                field("f03", "Formula", "formula"),
            ],
        );
        let plan = create_plan("appEx01", "MyBase", &[t], &[], BackupScope::Full);
        assert_eq!(plan.tables[0].field_count, 3);
        assert_eq!(plan.total_field_count, 3);
    }

    #[test]
    fn compatibility_summary_counts_correctly() {
        let t = table(
            "tbl01",
            "Projects",
            vec![
                field("f01", "Name", "singleLineText"),
                field("f02", "Status", "singleSelect"),
                field("f03", "Formula", "formula"),
                field("f04", "Linked", "multipleRecordLinks"),
            ],
        );
        let plan = create_plan("appEx01", "MyBase", &[t], &[], BackupScope::Full);
        // singleLineText + singleSelect = 2 restorable
        // formula = 1 metadataOnly
        // multipleRecordLinks = 1 unknown (not in restorable or metadata-only lists)
        assert_eq!(plan.compatibility.restorable_count, 2);
        assert_eq!(plan.compatibility.metadata_only_count, 1);
        assert_eq!(plan.compatibility.total_count, 4);
    }

    #[test]
    fn warnings_generated_for_formula_field() {
        let t = table("tbl01", "T", vec![field("f01", "Computed", "formula")]);
        let plan = create_plan("appEx01", "B", &[t], &[], BackupScope::Full);
        assert!(plan.warnings.iter().any(|w| w.code == "COMPUTED_FIELD"));
    }

    #[test]
    fn warnings_generated_for_attachment_field() {
        let t = table(
            "tbl01",
            "T",
            vec![field("f01", "Files", "multipleAttachments")],
        );
        let plan = create_plan("appEx01", "B", &[t], &[], BackupScope::Full);
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.code == "ATTACHMENT_METADATA_ONLY"));
    }

    #[test]
    fn warnings_generated_for_linked_record_field() {
        let t = table(
            "tbl01",
            "T",
            vec![field("f01", "Link", "multipleRecordLinks")],
        );
        let plan = create_plan("appEx01", "B", &[t], &[], BackupScope::Full);
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.code == "LINKED_RECORD_REMAPPING"));
    }

    #[test]
    fn attachment_policy_is_metadata_only() {
        let t = table(
            "tbl01",
            "T",
            vec![field("f01", "Files", "multipleAttachments")],
        );
        let plan = create_plan("appEx01", "B", &[t], &[], BackupScope::Full);
        let f = &plan.tables[0].fields[0];
        assert!(matches!(
            f.attachment_policy,
            Some(AttachmentPolicy::MetadataOnly)
        ));
    }

    #[test]
    fn linked_record_policy_indicates_remapping_required() {
        let t = table(
            "tbl01",
            "T",
            vec![field("f01", "Link", "multipleRecordLinks")],
        );
        let plan = create_plan("appEx01", "B", &[t], &[], BackupScope::Full);
        let f = &plan.tables[0].fields[0];
        assert!(matches!(
            f.linked_record_policy,
            Some(LinkedRecordPolicy::RemappingRequiredForRestore)
        ));
    }

    #[test]
    fn dry_run_is_always_true() {
        let plan = create_plan("appEx01", "B", &[], &[], BackupScope::Full);
        assert!(plan.dry_run);
    }

    #[test]
    fn output_package_path_is_always_none() {
        let plan = create_plan("appEx01", "B", &[], &[], BackupScope::Full);
        assert!(plan.output_package_path.is_none());
    }

    #[test]
    fn plan_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_example_planner_sentinel_0123456789";
        let t = table("tbl01", "T", vec![field("f01", "Name", "singleLineText")]);
        let plan = create_plan("appEx01", "B", &[t], &[], BackupScope::Full);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn record_counts_unknown_when_not_provided() {
        use crate::models::backup_plan::RecordReadEstimate;
        let t = table("tbl01", "T", vec![field("f01", "Name", "singleLineText")]);
        let plan = create_plan("appEx01", "B", &[t], &[], BackupScope::Full);
        assert_eq!(plan.estimate.record_read_pages, RecordReadEstimate::Unknown);
    }

    #[test]
    fn record_counts_known_when_provided() {
        use crate::models::backup_plan::RecordReadEstimate;
        let t = table("tbl01", "T", vec![field("f01", "Name", "singleLineText")]);
        let plan = create_plan("appEx01", "B", &[t], &[Some(50)], BackupScope::Full);
        // 50 records → 1 page
        assert_eq!(
            plan.estimate.record_read_pages,
            RecordReadEstimate::Known(1)
        );
    }
}
