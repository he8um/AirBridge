/// Root URL for the Airtable REST API v0.
pub const API_ROOT: &str = "https://api.airtable.com/v0";

/// Root URL for the Airtable Metadata API.
pub const META_ROOT: &str = "https://api.airtable.com/v0/meta";

/// Returns the path for listing all bases accessible to the token.
pub fn list_bases_path() -> String {
    format!("{META_ROOT}/bases")
}

/// Returns the path for retrieving the schema of a single base.
pub fn base_schema_path(base_id: &str) -> String {
    format!("{META_ROOT}/bases/{base_id}/tables")
}

/// Returns the path for listing records in a table.
pub fn list_records_path(base_id: &str, table_id: &str) -> String {
    format!("{API_ROOT}/{base_id}/{table_id}")
}

/// Returns the path for creating records in a table.
pub fn create_records_path(base_id: &str, table_id: &str) -> String {
    format!("{API_ROOT}/{base_id}/{table_id}")
}

/// Returns the path for updating records in a table (PATCH).
pub fn update_records_path(base_id: &str, table_id: &str) -> String {
    format!("{API_ROOT}/{base_id}/{table_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_bases_path_uses_meta_root() {
        assert!(list_bases_path().starts_with(META_ROOT));
    }

    #[test]
    fn base_schema_path_includes_base_and_tables() {
        let path = base_schema_path("appTestBase001");
        assert!(path.contains("appTestBase001"));
        assert!(path.ends_with("/tables"));
    }

    #[test]
    fn list_records_path_includes_base_and_table() {
        let path = list_records_path("appTestBase001", "tblTestTable01");
        assert!(path.contains("appTestBase001"));
        assert!(path.contains("tblTestTable01"));
    }

    #[test]
    fn create_and_update_paths_match_list_path() {
        let base = "appTestBase001";
        let table = "tblTestTable01";
        assert_eq!(
            create_records_path(base, table),
            list_records_path(base, table)
        );
        assert_eq!(
            update_records_path(base, table),
            list_records_path(base, table)
        );
    }

    #[test]
    fn paths_do_not_contain_real_ids() {
        let path = list_records_path("appTestBase001", "tblTestTable01");
        assert!(!path.contains("appreal"));
    }
}
