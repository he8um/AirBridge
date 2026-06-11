use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::backup::checksums::sha256_hex;
use crate::backup::format::{FORMAT_NAME, FORMAT_VERSION, PACKAGE_EXTENSION, REQUIRED_ENTRIES};
use crate::backup::reader::BackupPackageReader;

/// Overall validation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidationStatus {
    Valid,
    Invalid,
    Warning,
}

/// A single validation error or warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
}

/// Summary of the manifest for inclusion in the validation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSummary {
    pub format: String,
    pub format_version: String,
    pub app_version: String,
    pub created_at: String,
    pub base_id: String,
    pub base_name: String,
    pub table_count: usize,
    pub record_count: usize,
}

/// Structured report returned by the validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub status: ValidationStatus,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub entry_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_summary: Option<ManifestSummary>,
}

impl ValidationReport {
    fn new() -> Self {
        ValidationReport {
            status: ValidationStatus::Valid,
            errors: vec![],
            warnings: vec![],
            entry_count: 0,
            manifest_summary: None,
        }
    }

    fn add_error(&mut self, code: &str, message: impl Into<String>) {
        self.errors.push(ValidationIssue {
            code: code.to_string(),
            message: message.into(),
        });
        self.status = ValidationStatus::Invalid;
    }

    fn add_warning(&mut self, code: &str, message: impl Into<String>) {
        self.warnings.push(ValidationIssue {
            code: code.to_string(),
            message: message.into(),
        });
        if self.status == ValidationStatus::Valid {
            self.status = ValidationStatus::Warning;
        }
    }
}

/// Validates a `.airbridge` package at `path`.
pub fn validate_package(path: &Path) -> ValidationReport {
    let mut report = ValidationReport::new();

    // Extension check
    if let Some(ext) = path.extension() {
        if ext != PACKAGE_EXTENSION {
            report.add_warning(
                "WRONG_EXTENSION",
                format!(
                    "expected .{} extension, got .{}",
                    PACKAGE_EXTENSION,
                    ext.to_string_lossy()
                ),
            );
        }
    } else {
        report.add_warning("NO_EXTENSION", "package file has no extension");
    }

    // Open archive
    let mut reader = match BackupPackageReader::open(path) {
        Ok(r) => r,
        Err(e) => {
            report.add_error("CANNOT_OPEN", format!("cannot open package: {e}"));
            return report;
        }
    };

    report.entry_count = reader.entry_count();

    // Required entries check
    let names = reader.entry_names();
    for req in REQUIRED_ENTRIES {
        if !names.contains(&req.to_string()) {
            report.add_error(
                "MISSING_REQUIRED_ENTRY",
                format!("required entry missing: {req}"),
            );
        }
    }

    if report.status == ValidationStatus::Invalid {
        return report;
    }

    // Manifest validation
    let manifest = match reader.read_manifest() {
        Ok(m) => m,
        Err(e) => {
            report.add_error(
                "MANIFEST_PARSE_ERROR",
                format!("cannot parse manifest: {e}"),
            );
            return report;
        }
    };

    if manifest.format != FORMAT_NAME {
        report.add_error(
            "WRONG_FORMAT",
            format!(
                "expected format '{}', got '{}'",
                FORMAT_NAME, manifest.format
            ),
        );
    }

    if manifest.format_version != FORMAT_VERSION {
        report.add_error(
            "UNSUPPORTED_FORMAT_VERSION",
            format!(
                "unsupported format version '{}'; supported: '{}'",
                manifest.format_version, FORMAT_VERSION
            ),
        );
    }

    report.manifest_summary = Some(ManifestSummary {
        format: manifest.format.clone(),
        format_version: manifest.format_version.clone(),
        app_version: manifest.app_version.clone(),
        created_at: manifest.created_at.clone(),
        base_id: manifest.source.base_id.clone(),
        base_name: manifest.source.base_name.clone(),
        table_count: manifest.contents.tables,
        record_count: manifest.contents.records,
    });

    // Checksum validation
    let checksums = match reader.read_checksums() {
        Ok(c) => c,
        Err(e) => {
            report.add_error(
                "CHECKSUMS_PARSE_ERROR",
                format!("cannot read checksums: {e}"),
            );
            return report;
        }
    };

    // Verify each checksummed entry
    for (entry_path, expected_hash) in &checksums {
        match reader.read_entry(entry_path) {
            Ok(data) => {
                let actual_hash = sha256_hex(&data);
                if &actual_hash != expected_hash {
                    report.add_error(
                        "CHECKSUM_MISMATCH",
                        format!("checksum mismatch for entry '{entry_path}'"),
                    );
                }
            }
            Err(_) => {
                report.add_warning(
                    "CHECKSUM_ENTRY_MISSING",
                    format!("checksummed entry '{entry_path}' not present"),
                );
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::format::{
        FORMAT_NAME, PATH_BACKUP_REPORT, PATH_BASE, PATH_CHECKSUMS, PATH_MANIFEST, PATH_SCHEMA,
    };
    use crate::backup::manifest::{
        ManifestContents, ManifestPackage, ManifestSecurity, ManifestSource, PackageManifest,
    };
    use crate::backup::package::{PackageInput, TableRecords};
    use crate::backup::writer::write_package;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_valid_package(dir: &tempfile::TempDir) -> std::path::PathBuf {
        let path = dir.path().join("valid.airbridge");
        let manifest = PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: "appSyn01".to_string(),
                base_name: "Synthetic Base".to_string(),
                workspace_id: None,
            },
            ManifestContents {
                tables: 1,
                fields: 2,
                records: 3,
                linked_record_relationships: 0,
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
                package_id: "00000000-0000-0000-0000-000000000001".to_string(),
            },
        );
        let input = PackageInput {
            manifest_json: serde_json::to_vec(&manifest).unwrap(),
            base_json: br#"{"baseId":"appSyn01"}"#.to_vec(),
            schema_json: br#"{"tables":[]}"#.to_vec(),
            backup_report_json: br#"{"status":"ok"}"#.to_vec(),
            tables: vec![TableRecords {
                table_id: "tblSyn01".to_string(),
                lines: vec![
                    r#"{"id":"rec001","fields":{"Name":"Alpha"}}"#.to_string(),
                    r#"{"id":"rec002","fields":{"Name":"Beta"}}"#.to_string(),
                    r#"{"id":"rec003","fields":{"Name":"Gamma"}}"#.to_string(),
                ],
            }],
            ..Default::default()
        };
        write_package(&path, &input).expect("write");
        path
    }

    #[test]
    fn validator_accepts_valid_package() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let report = validate_package(&path);
        assert_eq!(
            report.status,
            ValidationStatus::Valid,
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn validator_report_has_manifest_summary_on_valid() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let report = validate_package(&path);
        assert!(report.manifest_summary.is_some());
        let summary = report.manifest_summary.unwrap();
        assert_eq!(summary.format, FORMAT_NAME);
        assert_eq!(summary.base_id, "appSyn01");
    }

    #[test]
    fn validator_rejects_missing_manifest() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("no_manifest.airbridge");
        // Write a zip with no manifest.json
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("other.txt", opts).unwrap();
        zip.write_all(b"content").unwrap();
        zip.finish().unwrap();

        let report = validate_package(&path);
        assert_eq!(report.status, ValidationStatus::Invalid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.code == "MISSING_REQUIRED_ENTRY"));
    }

    #[test]
    fn validator_rejects_unsupported_format_version() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("bad_version.airbridge");
        let mut manifest = PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: "appSyn01".to_string(),
                base_name: "Synthetic".to_string(),
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
                package_id: "00000000-0000-0000-0000-000000000002".to_string(),
            },
        );
        manifest.format_version = "99.0.0".to_string();
        let input = PackageInput {
            manifest_json: serde_json::to_vec(&manifest).unwrap(),
            base_json: br#"{"baseId":"appSyn01"}"#.to_vec(),
            schema_json: br#"{"tables":[]}"#.to_vec(),
            backup_report_json: br#"{"status":"ok"}"#.to_vec(),
            ..Default::default()
        };
        write_package(&path, &input).expect("write");
        let report = validate_package(&path);
        assert_eq!(report.status, ValidationStatus::Invalid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.code == "UNSUPPORTED_FORMAT_VERSION"));
    }

    #[test]
    fn validator_rejects_checksum_mismatch() {
        // Build a package where checksums/sha256.json contains a deliberately wrong hash
        // for manifest.json. This simulates post-write tampering.
        use crate::backup::checksums::ChecksumMap;
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("bad_checksum.airbridge");

        let manifest = PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: "appSyn01".to_string(),
                base_name: "Synthetic".to_string(),
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
                package_id: "00000000-0000-0000-0000-000000000003".to_string(),
            },
        );
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();

        // Build a checksum map with a WRONG hash for manifest.json
        let mut bad_checksums: ChecksumMap = ChecksumMap::new();
        bad_checksums.insert(
            PATH_MANIFEST.to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        );
        let base_bytes = br#"{"baseId":"appSyn01"}"#;
        let schema_bytes = br#"{"tables":[]}"#;
        let report_bytes = br#"{"status":"ok"}"#;
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

        // Write a zip manually with the bad checksum file
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

        let report = validate_package(&path);
        assert_eq!(
            report.status,
            ValidationStatus::Invalid,
            "expected Invalid, got {:?}: {:?}",
            report.status,
            report.errors
        );
        assert!(
            report.errors.iter().any(|e| e.code == "CHECKSUM_MISMATCH"),
            "expected CHECKSUM_MISMATCH error, got: {:?}",
            report.errors
        );
    }

    #[test]
    fn validator_warns_on_wrong_extension() {
        let dir = tempdir().expect("tempdir");
        let source = write_valid_package(&dir);
        let wrong_ext = dir.path().join("package.zip");
        std::fs::copy(&source, &wrong_ext).unwrap();
        let report = validate_package(&wrong_ext);
        assert!(report.warnings.iter().any(|w| w.code == "WRONG_EXTENSION"));
    }

    #[test]
    fn validator_report_entry_count_is_nonzero() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let report = validate_package(&path);
        assert!(report.entry_count > 0);
    }

    #[test]
    fn validator_valid_package_has_no_errors() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let report = validate_package(&path);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn validator_report_serializes_to_json() {
        let dir = tempdir().expect("tempdir");
        let path = write_valid_package(&dir);
        let report = validate_package(&path);
        let json = serde_json::to_string(&report).expect("serialize report");
        assert!(json.contains("status"));
    }
}
