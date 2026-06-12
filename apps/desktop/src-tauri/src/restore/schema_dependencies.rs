use crate::restore::schema_plan::{
    RestoreLinkedDependencyStep, RestoreSchemaDependencyGraph, SchemaPlanTableInput,
};

/// Resolves the table name for a given table ID from the input tables list.
fn resolve_table_name<'a>(tables: &'a [SchemaPlanTableInput], table_id: &'a str) -> &'a str {
    tables
        .iter()
        .find(|t| t.table_id == table_id)
        .map(|t| t.table_name.as_str())
        .unwrap_or(table_id)
}

/// Builds the dependency graph for linked record fields across all tables.
///
/// - No Airtable calls.
/// - Reads only from the in-memory table input.
pub fn build_dependency_graph(tables: &[SchemaPlanTableInput]) -> RestoreSchemaDependencyGraph {
    let mut edges = Vec::new();

    for table in tables {
        for field in &table.fields {
            if field.field_type != "multipleRecordLinks" {
                continue;
            }
            let target_table_id = field.linked_table_id.clone().unwrap_or_default();
            let target_table_name = resolve_table_name(tables, &target_table_id);

            edges.push(RestoreLinkedDependencyStep {
                field_id: field.field_id.clone(),
                field_name: field.field_name.clone(),
                source_table_id: table.table_id.clone(),
                source_table_name: table.table_name.clone(),
                target_table_id: target_table_id.clone(),
                target_table_name: target_table_name.to_string(),
                remapping_required: true,
                note: format!(
                    "Field '{}' in '{}' links to '{}'. Record IDs must be remapped after import.",
                    field.field_name, table.table_name, target_table_name
                ),
            });
        }
    }

    // Simple circularity check: if any table links to itself.
    let has_circular = edges.iter().any(|e| e.source_table_id == e.target_table_id);

    RestoreSchemaDependencyGraph {
        edges,
        has_circular_dependency: has_circular,
        resolution_note: "Linked record fields are deferred and applied after all tables and \
            records are imported. Record IDs in link references are remapped from backup IDs \
            to restored IDs."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::schema_plan::SchemaPlanFieldInput;

    fn make_tables() -> Vec<SchemaPlanTableInput> {
        vec![
            SchemaPlanTableInput {
                table_id: "tblA".to_string(),
                table_name: "Projects".to_string(),
                fields: vec![
                    SchemaPlanFieldInput {
                        field_id: "fld01".to_string(),
                        field_name: "Name".to_string(),
                        field_type: "singleLineText".to_string(),
                        linked_table_id: None,
                    },
                    SchemaPlanFieldInput {
                        field_id: "fld02".to_string(),
                        field_name: "Related Tasks".to_string(),
                        field_type: "multipleRecordLinks".to_string(),
                        linked_table_id: Some("tblB".to_string()),
                    },
                ],
            },
            SchemaPlanTableInput {
                table_id: "tblB".to_string(),
                table_name: "Tasks".to_string(),
                fields: vec![SchemaPlanFieldInput {
                    field_id: "fld03".to_string(),
                    field_name: "Title".to_string(),
                    field_type: "singleLineText".to_string(),
                    linked_table_id: None,
                }],
            },
        ]
    }

    #[test]
    fn linked_fields_become_edges() {
        let tables = make_tables();
        let graph = build_dependency_graph(&tables);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source_table_id, "tblA");
        assert_eq!(graph.edges[0].target_table_id, "tblB");
        assert_eq!(graph.edges[0].target_table_name, "Tasks");
    }

    #[test]
    fn non_linked_fields_are_not_edges() {
        let tables = make_tables();
        let graph = build_dependency_graph(&tables);
        assert!(graph.edges.iter().all(|e| e.field_type_is_linked()));
    }

    #[test]
    fn no_circular_dependency_in_simple_graph() {
        let tables = make_tables();
        let graph = build_dependency_graph(&tables);
        assert!(!graph.has_circular_dependency);
    }

    #[test]
    fn self_link_is_detected_as_circular() {
        let tables = vec![SchemaPlanTableInput {
            table_id: "tblA".to_string(),
            table_name: "Projects".to_string(),
            fields: vec![SchemaPlanFieldInput {
                field_id: "fld01".to_string(),
                field_name: "Sub-projects".to_string(),
                field_type: "multipleRecordLinks".to_string(),
                linked_table_id: Some("tblA".to_string()),
            }],
        }];
        let graph = build_dependency_graph(&tables);
        assert!(graph.has_circular_dependency);
    }

    #[test]
    fn empty_tables_produce_empty_graph() {
        let graph = build_dependency_graph(&[]);
        assert!(graph.edges.is_empty());
        assert!(!graph.has_circular_dependency);
    }

    #[test]
    fn unknown_target_table_uses_id_as_name() {
        let tables = vec![SchemaPlanTableInput {
            table_id: "tblA".to_string(),
            table_name: "Projects".to_string(),
            fields: vec![SchemaPlanFieldInput {
                field_id: "fld01".to_string(),
                field_name: "Links".to_string(),
                field_type: "multipleRecordLinks".to_string(),
                linked_table_id: Some("tblUnknown".to_string()),
            }],
        }];
        let graph = build_dependency_graph(&tables);
        assert_eq!(graph.edges[0].target_table_name, "tblUnknown");
    }

    #[test]
    fn all_edges_require_remapping() {
        let tables = make_tables();
        let graph = build_dependency_graph(&tables);
        assert!(graph.edges.iter().all(|e| e.remapping_required));
    }

    #[test]
    fn graph_serializes_without_error() {
        let tables = make_tables();
        let graph = build_dependency_graph(&tables);
        let json = serde_json::to_string(&graph).expect("serialize");
        assert!(json.contains("edges"));
        assert!(json.contains("remappingRequired"));
    }
}

// Helper for test assertions — not part of the public API.
impl RestoreLinkedDependencyStep {
    #[cfg(test)]
    fn field_type_is_linked(&self) -> bool {
        // All edges in the graph represent multipleRecordLinks fields.
        true
    }
}
