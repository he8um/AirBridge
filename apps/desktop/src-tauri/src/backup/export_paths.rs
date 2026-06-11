use crate::backup::format::PATH_TABLES_PREFIX;

/// Returns the JSONL records entry path for a given table ID.
///
/// Example: `tables/tblAbc01/records.jsonl`
///
/// Uses the stable table ID, not the human-readable name. No absolute paths.
pub fn records_jsonl_path(table_id: &str) -> String {
    format!("{}{}/records.jsonl", PATH_TABLES_PREFIX, table_id)
}

/// Returns the table metadata JSON entry path for a given table ID.
pub fn table_json_path(table_id: &str) -> String {
    format!("{}{}/table.json", PATH_TABLES_PREFIX, table_id)
}

/// Returns the fields JSON entry path for a given table ID.
pub fn fields_json_path(table_id: &str) -> String {
    format!("{}{}/fields.json", PATH_TABLES_PREFIX, table_id)
}

/// Returns the CSV records entry path for a given table ID.
pub fn records_csv_path(table_id: &str) -> String {
    format!("{}{}/records.csv", PATH_TABLES_PREFIX, table_id)
}

/// Validates that an entry path is safe: no leading slash, no absolute components.
pub fn is_safe_entry_path(path: &str) -> bool {
    !path.starts_with('/')
        && !path.contains("../")
        && !path.contains("Users/")
        && !path.contains("home/")
        && !path.starts_with('\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_jsonl_path_contains_table_id() {
        let path = records_jsonl_path("tblSyn01");
        assert!(path.contains("tblSyn01"));
    }

    #[test]
    fn records_jsonl_path_ends_with_records_jsonl() {
        let path = records_jsonl_path("tblSyn01");
        assert!(path.ends_with("records.jsonl"));
    }

    #[test]
    fn records_jsonl_path_has_no_leading_slash() {
        let path = records_jsonl_path("tblSyn01");
        assert!(!path.starts_with('/'));
    }

    #[test]
    fn records_jsonl_path_uses_tables_prefix() {
        let path = records_jsonl_path("tblSyn01");
        assert!(path.starts_with("tables/"));
    }

    #[test]
    fn table_json_path_structure() {
        let path = table_json_path("tblAbc99");
        assert_eq!(path, "tables/tblAbc99/table.json");
    }

    #[test]
    fn fields_json_path_structure() {
        let path = fields_json_path("tblAbc99");
        assert_eq!(path, "tables/tblAbc99/fields.json");
    }

    #[test]
    fn records_csv_path_structure() {
        let path = records_csv_path("tblAbc99");
        assert_eq!(path, "tables/tblAbc99/records.csv");
    }

    #[test]
    fn is_safe_rejects_absolute_path() {
        assert!(!is_safe_entry_path("/etc/passwd"));
    }

    #[test]
    fn is_safe_rejects_traversal() {
        assert!(!is_safe_entry_path("tables/../secret.json"));
    }

    #[test]
    fn is_safe_rejects_user_path() {
        assert!(!is_safe_entry_path("Users/amir/data.json"));
    }

    #[test]
    fn is_safe_accepts_valid_entry() {
        assert!(is_safe_entry_path("tables/tblSyn01/records.jsonl"));
    }

    #[test]
    fn all_generated_paths_are_safe() {
        let id = "tblSyn01";
        for path in &[
            records_jsonl_path(id),
            table_json_path(id),
            fields_json_path(id),
            records_csv_path(id),
        ] {
            assert!(is_safe_entry_path(path), "path '{}' should be safe", path);
        }
    }
}
