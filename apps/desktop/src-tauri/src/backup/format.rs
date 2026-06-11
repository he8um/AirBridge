/// Identifies the package format in manifest.json.
pub const FORMAT_NAME: &str = "airbridge";

/// Current package format version. Incremented on breaking layout changes.
pub const FORMAT_VERSION: &str = "0.1.0";

/// File extension for AirBridge packages (without dot).
pub const PACKAGE_EXTENSION: &str = "airbridge";

// ── Required archive entry paths ──────────────────────────────────────────

pub const PATH_MANIFEST: &str = "manifest.json";
pub const PATH_BASE: &str = "base.json";
pub const PATH_SCHEMA: &str = "schema.json";
pub const PATH_LINKED_RECORDS: &str = "links/linked-records.jsonl";
pub const PATH_ATTACHMENT_METADATA: &str = "attachments/metadata.jsonl";
pub const PATH_BACKUP_REPORT: &str = "reports/backup-report.json";
pub const PATH_COMPATIBILITY_REPORT: &str = "reports/compatibility-report.json";
pub const PATH_VALIDATION_REPORT: &str = "reports/validation-report.json";
pub const PATH_CHECKSUMS: &str = "checksums/sha256.json";

/// Prefix for per-table record files, e.g. `tables/tbl_xxx/records.jsonl`.
pub const PATH_TABLES_PREFIX: &str = "tables/";

/// All required top-level entries that every valid package must contain.
pub const REQUIRED_ENTRIES: &[&str] = &[
    PATH_MANIFEST,
    PATH_BASE,
    PATH_SCHEMA,
    PATH_CHECKSUMS,
    PATH_BACKUP_REPORT,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_name_is_airbridge() {
        assert_eq!(FORMAT_NAME, "airbridge");
    }

    #[test]
    fn format_version_is_semver_like() {
        let parts: Vec<&str> = FORMAT_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "format version should be major.minor.patch");
    }

    #[test]
    fn required_entries_contains_manifest() {
        assert!(REQUIRED_ENTRIES.contains(&PATH_MANIFEST));
    }

    #[test]
    fn required_entries_contains_checksums() {
        assert!(REQUIRED_ENTRIES.contains(&PATH_CHECKSUMS));
    }

    #[test]
    fn path_constants_have_no_leading_slash() {
        let all = [
            PATH_MANIFEST,
            PATH_BASE,
            PATH_SCHEMA,
            PATH_LINKED_RECORDS,
            PATH_ATTACHMENT_METADATA,
            PATH_BACKUP_REPORT,
            PATH_COMPATIBILITY_REPORT,
            PATH_VALIDATION_REPORT,
            PATH_CHECKSUMS,
        ];
        for p in &all {
            assert!(!p.starts_with('/'), "path '{}' must not start with /", p);
        }
    }
}
