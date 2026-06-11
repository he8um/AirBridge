use crate::restore::plan::RestoreRecordOrderingPlan;

/// Returns the standard record import ordering plan.
///
/// The order is always:
/// 1. Create table schemas
/// 2. Create fields within each table
/// 3. Import records without linked-record references
/// 4. Apply linked-record references after all records exist and ID remapping is complete
pub fn build_ordering_plan() -> RestoreRecordOrderingPlan {
    RestoreRecordOrderingPlan {
        create_tables_first: true,
        create_fields_after_tables: true,
        import_records_without_links: true,
        apply_links_after_records: true,
        note: "Tables and fields are created first. Records are imported without linked references. Linked references are applied in a second pass after all record IDs are remapped.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_plan_tables_before_fields() {
        let plan = build_ordering_plan();
        assert!(plan.create_tables_first);
        assert!(plan.create_fields_after_tables);
    }

    #[test]
    fn ordering_plan_records_before_links() {
        let plan = build_ordering_plan();
        assert!(plan.import_records_without_links);
        assert!(plan.apply_links_after_records);
    }

    #[test]
    fn ordering_plan_serializes() {
        let plan = build_ordering_plan();
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("createTablesFirst"));
        assert!(json.contains("applyLinksAfterRecords"));
    }
}
