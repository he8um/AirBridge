use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::backup::validation::{validate_package, ValidationStatus};

/// Filename-only display name (no directory component, never the full path).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPackageInspectionResult {
    /// Filename only — never includes directory path.
    pub filename: String,
    pub validation_status: String,
    pub manifest: Option<BackupPackageManifestSummary>,
    pub contents: Option<BackupPackageContentsSummary>,
    pub security: Option<BackupPackageSecuritySummary>,
    pub checksums: Option<BackupPackageChecksumSummary>,
    pub entry_count: usize,
    pub warnings: Vec<BackupPackageInspectionIssue>,
    pub errors: Vec<BackupPackageInspectionIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPackageManifestSummary {
    pub format: String,
    pub format_version: String,
    pub app_version: String,
    pub created_at: String,
    pub provider: String,
    pub base_id: String,
    pub base_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPackageContentsSummary {
    pub table_count: usize,
    pub field_count: usize,
    pub record_count: usize,
    pub linked_record_relationship_count: usize,
    pub attachment_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPackageSecuritySummary {
    pub encrypted: bool,
    pub contains_record_data: bool,
    pub contains_attachment_urls: bool,
    pub redactions_applied: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPackageChecksumSummary {
    pub checksum_count: usize,
    pub all_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPackageInspectionIssue {
    pub code: String,
    pub message: String,
}

/// Inspects a `.airbridge` package at `path`.
///
/// - No files are extracted to disk.
/// - No writes of any kind.
/// - Returns filename only — never the full path.
/// - Errors from the path itself are sanitized before returning.
pub fn inspect_backup_package(path: &Path) -> BackupPackageInspectionResult {
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    let validation_report = validate_package(path);

    let validation_status = match validation_report.status {
        ValidationStatus::Valid => "valid",
        ValidationStatus::Invalid => "invalid",
        ValidationStatus::Warning => "warning",
    }
    .to_string();

    let warnings = validation_report
        .warnings
        .iter()
        .map(|i| BackupPackageInspectionIssue {
            code: i.code.clone(),
            message: i.message.clone(),
        })
        .collect();

    let errors = validation_report
        .errors
        .iter()
        .map(|i| BackupPackageInspectionIssue {
            code: i.code.clone(),
            message: i.message.clone(),
        })
        .collect();

    // If we have no manifest summary the package cannot be opened or parsed — return early.
    if validation_report.manifest_summary.is_none()
        && validation_report.status == ValidationStatus::Invalid
    {
        return BackupPackageInspectionResult {
            filename,
            validation_status,
            manifest: None,
            contents: None,
            security: None,
            checksums: None,
            entry_count: validation_report.entry_count,
            warnings,
            errors,
        };
    }

    // Collect richer manifest/contents/security from the raw manifest (requires a second open).
    let (manifest_summary, contents_summary, security_summary, checksum_summary) =
        enrich_from_package(path, &validation_report);

    BackupPackageInspectionResult {
        filename,
        validation_status,
        manifest: manifest_summary,
        contents: contents_summary,
        security: security_summary,
        checksums: checksum_summary,
        entry_count: validation_report.entry_count,
        warnings,
        errors,
    }
}

fn enrich_from_package(
    path: &Path,
    validation_report: &crate::backup::validation::ValidationReport,
) -> (
    Option<BackupPackageManifestSummary>,
    Option<BackupPackageContentsSummary>,
    Option<BackupPackageSecuritySummary>,
    Option<BackupPackageChecksumSummary>,
) {
    use crate::backup::reader::BackupPackageReader;

    let mut reader = match BackupPackageReader::open(path) {
        Ok(r) => r,
        Err(_) => return (None, None, None, None),
    };

    let manifest = match reader.read_manifest() {
        Ok(m) => m,
        Err(_) => {
            // Reconstruct minimal manifest summary from validation report if available
            if let Some(vs) = &validation_report.manifest_summary {
                let manifest_summary = BackupPackageManifestSummary {
                    format: vs.format.clone(),
                    format_version: vs.format_version.clone(),
                    app_version: vs.app_version.clone(),
                    created_at: vs.created_at.clone(),
                    provider: String::new(),
                    base_id: vs.base_id.clone(),
                    base_name: vs.base_name.clone(),
                };
                return (Some(manifest_summary), None, None, None);
            }
            return (None, None, None, None);
        }
    };

    let manifest_summary = BackupPackageManifestSummary {
        format: manifest.format.clone(),
        format_version: manifest.format_version.clone(),
        app_version: manifest.app_version.clone(),
        created_at: manifest.created_at.clone(),
        provider: manifest.source.provider.clone(),
        base_id: manifest.source.base_id.clone(),
        base_name: manifest.source.base_name.clone(),
    };

    let contents_summary = BackupPackageContentsSummary {
        table_count: manifest.contents.tables,
        field_count: manifest.contents.fields,
        record_count: manifest.contents.records,
        linked_record_relationship_count: manifest.contents.linked_record_relationships,
        attachment_count: manifest.contents.attachments,
    };

    let security_summary = BackupPackageSecuritySummary {
        encrypted: manifest.security.encrypted,
        contains_record_data: manifest.security.contains_record_data,
        contains_attachment_urls: manifest.security.contains_attachment_urls,
        redactions_applied: manifest.security.redactions_applied.clone(),
    };

    let checksum_count = reader
        .read_checksums()
        .map(|c| c.len())
        .unwrap_or(0);

    let checksum_valid = validation_report
        .errors
        .iter()
        .all(|e| e.code != "CHECKSUM_MISMATCH");

    let checksum_summary = BackupPackageChecksumSummary {
        checksum_count,
        all_valid: checksum_valid,
    };

    (
        Some(manifest_summary),
        Some(contents_summary),
        Some(security_summary),
        Some(checksum_summary),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::manifest::{
        ManifestContents, ManifestPackage, ManifestSecurity, ManifestSource, PackageManifest,
    };
    use crate::backup::package::{PackageInput, TableRecords};
    use crate::backup::writer::write_package;
    use tempfile::tempdir;

    fn write_valid_package(dir: &tempfile::TempDir) -> std::path::PathBuf {
        let path = dir.path().join("test.airbridge");
        let manifest = PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: "appInsp01".to_string(),
                base_name: "Inspection Test Base".to_string(),
                workspace_id: None,
            },
            ManifestContents {
                tables: 2,
                fields: 5,
                records: 10,
                linked_record_relationships: 1,
                attachments: 0,
            },
            ManifestSecurity {
                contains_record_data: true,
                contains_attachment_urls: false,
                encrypted: false,
                redactions_applied: vec![],
            },
            ManifestPackage {
                generated_by_app: "airbridge".to_string(),
                package_id: "00000000-0000-0000-0000-000000000010".to_string(),
            },
        );
        let input = PackageInput {
            manifest_json: serde_json::to_vec(&manifest).unwrap(),
            base_json: br#"{"baseId":"appInsp01"}"#.to_vec(),
            schema_json: br#"{"tables":[]}"#.to_vec(),
            backup_report_json: br#"{"status":"ok"}"#.to_vec(),
            tables: vec![TableRecords {
                table_id: "tblInsp01".to_string(),
                lines: vec![
                    r#"{"id":"recA01","fields":{"Name":"Row 1"}}"#.to_string(),
                    r#"{"id":"recA02","fields":{"Name":"Row 2"}}"#.to_string(),
                ],
            }],
            ..Default::default()
        };
        write_package(&path, &input).expect("write");
        path
    }

    #[test]
    fn inspect_valid_package_returns_valid_status() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        assert_eq!(result.validation_status, "valid");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn inspect_returns_filename_not_full_path() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        assert_eq!(result.filename, "test.airbridge");
        assert!(!result.filename.contains('/'));
        assert!(!result.filename.contains('\\'));
    }

    #[test]
    fn inspect_returns_manifest_summary() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        let manifest = result.manifest.expect("manifest present");
        assert_eq!(manifest.base_id, "appInsp01");
        assert_eq!(manifest.base_name, "Inspection Test Base");
        assert_eq!(manifest.provider, "airtable");
        assert_eq!(manifest.format, "airbridge");
        assert_eq!(manifest.format_version, "0.1.0");
    }

    #[test]
    fn inspect_returns_contents_summary() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        let contents = result.contents.expect("contents present");
        assert_eq!(contents.table_count, 2);
        assert_eq!(contents.field_count, 5);
        assert_eq!(contents.record_count, 10);
        assert_eq!(contents.linked_record_relationship_count, 1);
        assert_eq!(contents.attachment_count, 0);
    }

    #[test]
    fn inspect_returns_security_summary() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        let security = result.security.expect("security present");
        assert!(!security.encrypted);
        assert!(security.contains_record_data);
        assert!(!security.contains_attachment_urls);
        assert!(security.redactions_applied.is_empty());
    }

    #[test]
    fn inspect_returns_checksum_summary() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        let checksums = result.checksums.expect("checksums present");
        assert!(checksums.checksum_count > 0);
        assert!(checksums.all_valid);
    }

    #[test]
    fn inspect_returns_nonzero_entry_count() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        assert!(result.entry_count > 0);
    }

    #[test]
    fn inspect_nonexistent_file_returns_error_status() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("does_not_exist.airbridge");
        let result = inspect_backup_package(&path);
        assert_eq!(result.validation_status, "invalid");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn inspect_result_does_not_expose_full_path() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        let json = serde_json::to_string(&result).expect("serialize");
        // The absolute directory should not appear in any serialized field.
        let dir_str = dir.path().to_string_lossy();
        assert!(
            !json.contains(dir_str.as_ref()),
            "full path leaked in JSON output"
        );
    }

    #[test]
    fn inspect_result_serializes_to_json() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("validationStatus"));
        assert!(json.contains("filename"));
    }

    #[test]
    fn inspect_valid_package_has_no_checksum_mismatch_error() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let result = inspect_backup_package(&path);
        assert!(!result
            .errors
            .iter()
            .any(|e| e.code == "CHECKSUM_MISMATCH"));
    }

    #[test]
    fn inspect_checksum_all_valid_false_on_mismatch() {
        use crate::backup::checksums::ChecksumMap;
        use crate::backup::format::{
            PATH_BACKUP_REPORT, PATH_BASE, PATH_CHECKSUMS, PATH_MANIFEST, PATH_SCHEMA,
        };
        use std::io::Write;

        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("bad_checksum.airbridge");
        let manifest = PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: "appInsp02".to_string(),
                base_name: "Bad Checksum Base".to_string(),
                workspace_id: None,
            },
            ManifestContents {
                tables: 0,
                fields: 0,
                records: 0,
                linked_record_relationships: 0,
                attachments: 0,
            },
            ManifestSecurity {
                contains_record_data: false,
                contains_attachment_urls: false,
                encrypted: false,
                redactions_applied: vec![],
            },
            ManifestPackage {
                generated_by_app: "airbridge".to_string(),
                package_id: "00000000-0000-0000-0000-000000000011".to_string(),
            },
        );
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        let base_bytes = br#"{"baseId":"appInsp02"}"#;
        let schema_bytes = br#"{"tables":[]}"#;
        let report_bytes = br#"{"status":"ok"}"#;

        let mut bad_checksums: ChecksumMap = ChecksumMap::new();
        bad_checksums.insert(
            PATH_MANIFEST.to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        );
        bad_checksums.insert(
            PATH_BASE.to_string(),
            crate::backup::checksums::sha256_hex(base_bytes),
        );
        bad_checksums.insert(
            PATH_SCHEMA.to_string(),
            crate::backup::checksums::sha256_hex(schema_bytes),
        );
        bad_checksums.insert(
            PATH_BACKUP_REPORT.to_string(),
            crate::backup::checksums::sha256_hex(report_bytes),
        );

        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file(PATH_MANIFEST, opts).unwrap();
        zip.write_all(&manifest_bytes).unwrap();
        zip.start_file(PATH_BASE, opts).unwrap();
        zip.write_all(base_bytes).unwrap();
        zip.start_file(PATH_SCHEMA, opts).unwrap();
        zip.write_all(schema_bytes).unwrap();
        zip.start_file(PATH_BACKUP_REPORT, opts).unwrap();
        zip.write_all(report_bytes).unwrap();
        let checksum_json = crate::backup::checksums::checksums_to_json(&bad_checksums);
        zip.start_file(PATH_CHECKSUMS, opts).unwrap();
        zip.write_all(&checksum_json).unwrap();
        zip.finish().unwrap();

        let result = inspect_backup_package(&path);
        assert_eq!(result.validation_status, "invalid");
        assert!(result.checksums.map(|c| !c.all_valid).unwrap_or(true));
    }
}
