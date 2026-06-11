use std::path::Path;

use serde::Deserialize;

use crate::backup::reader::BackupPackageReader;
use crate::backup::validation::{validate_package, ValidationStatus};
use crate::restore::compatibility::{
    build_attachment_plan, build_field_plan, build_linked_record_plan,
};
use crate::restore::ordering::build_ordering_plan;
use crate::restore::plan::{
    RestoreDryRunError, RestoreDryRunPlan, RestoreDryRunRequest, RestoreFieldCompatibility,
    RestorePackageSummary, RestorePlanStatus, RestoreTablePlan,
};
use crate::restore::warnings::warnings_for_fields;

/// Minimal schema structures for deserializing schema.json from the package.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaField {
    id: Option<String>,
    name: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<String>,
    /// For linked record fields, the linked table ID.
    #[serde(default)]
    linked_table_id: Option<String>,
    /// Alternative key used by some Airtable schema formats.
    #[serde(default)]
    options: Option<LinkedFieldOptions>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinkedFieldOptions {
    #[serde(default)]
    linked_table_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaTable {
    id: Option<String>,
    name: Option<String>,
    #[serde(default)]
    fields: Vec<SchemaField>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageSchema {
    #[serde(default)]
    tables: Vec<SchemaTable>,
}

/// Creates a restore dry-run plan from a `.airbridge` package.
///
/// - No files are extracted to disk.
/// - No Airtable API calls.
/// - No token required.
/// - Returns filename only — never the full path.
pub fn create_dry_run_plan(request: &RestoreDryRunRequest) -> RestoreDryRunPlan {
    let path = Path::new(&request.path);

    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    // Validate the package first.
    let validation = validate_package(path);
    if validation.status == ValidationStatus::Invalid {
        let errors: Vec<RestoreDryRunError> = validation
            .errors
            .iter()
            .map(|e| RestoreDryRunError {
                code: e.code.clone(),
                message: e.message.clone(),
            })
            .collect();
        return RestoreDryRunPlan {
            filename,
            status: RestorePlanStatus::Blocked,
            target_mode: request.target_mode.clone(),
            target_base_name: request.target_base_name.clone(),
            package_summary: None,
            tables: vec![],
            ordering: None,
            warnings: vec![],
            errors,
            no_changes_made: true,
        };
    }

    // Open package in memory.
    let mut reader = match BackupPackageReader::open(path) {
        Ok(r) => r,
        Err(e) => {
            return blocked_plan(
                filename,
                request,
                "CANNOT_OPEN",
                &format!("cannot open package: {e}"),
            );
        }
    };

    // Read manifest.
    let manifest = match reader.read_manifest() {
        Ok(m) => m,
        Err(e) => {
            return blocked_plan(
                filename,
                request,
                "MANIFEST_PARSE_ERROR",
                &format!("cannot parse manifest: {e}"),
            );
        }
    };

    // Read schema.
    let schema_bytes = match reader.read_schema() {
        Ok(b) => b,
        Err(e) => {
            return blocked_plan(
                filename,
                request,
                "SCHEMA_READ_ERROR",
                &format!("cannot read schema: {e}"),
            );
        }
    };

    let schema: PackageSchema = match serde_json::from_slice(&schema_bytes) {
        Ok(s) => s,
        Err(e) => {
            return blocked_plan(
                filename,
                request,
                "SCHEMA_PARSE_ERROR",
                &format!("cannot parse schema: {e}"),
            );
        }
    };

    // Build package summary.
    let package_summary = RestorePackageSummary {
        filename: filename.clone(),
        format: manifest.format.clone(),
        format_version: manifest.format_version.clone(),
        app_version: manifest.app_version.clone(),
        created_at: manifest.created_at.clone(),
        provider: manifest.source.provider.clone(),
        base_id: manifest.source.base_id.clone(),
        base_name: manifest.source.base_name.clone(),
        table_count: manifest.contents.tables,
        field_count: manifest.contents.fields,
        record_count: manifest.contents.records,
        contains_record_data: manifest.security.contains_record_data,
        contains_attachment_urls: manifest.security.contains_attachment_urls,
        encrypted: manifest.security.encrypted,
    };

    // Build table plans from schema.
    let mut all_warnings = Vec::new();
    let mut table_plans = Vec::new();

    for table in &schema.tables {
        let table_id = table.id.clone().unwrap_or_default();
        let table_name = table.name.clone().unwrap_or_else(|| table_id.clone());

        let mut field_plans = Vec::new();
        let mut linked_record_plans = Vec::new();
        let mut attachment_plans = Vec::new();

        for field in &table.fields {
            let field_id = field.id.clone().unwrap_or_default();
            let field_name = field.name.clone().unwrap_or_else(|| field_id.clone());
            let field_type = field.field_type.clone().unwrap_or_default();

            let fp = build_field_plan(&field_id, &field_name, &field_type);

            if fp.compatibility == RestoreFieldCompatibility::PartiallySupported
                && field_type == "multipleRecordLinks"
            {
                let linked_id = field
                    .linked_table_id
                    .clone()
                    .or_else(|| {
                        field
                            .options
                            .as_ref()
                            .and_then(|o| o.linked_table_id.clone())
                    })
                    .unwrap_or_default();
                linked_record_plans.push(build_linked_record_plan(
                    &field_id,
                    &field_name,
                    &linked_id,
                ));
            }

            if fp.compatibility == RestoreFieldCompatibility::MetadataOnly
                && field_type == "multipleAttachments"
            {
                attachment_plans.push(build_attachment_plan(&field_id, &field_name));
            }

            field_plans.push(fp);
        }

        let restorable_count = field_plans
            .iter()
            .filter(|f| f.compatibility == RestoreFieldCompatibility::Supported)
            .count();
        let partial_count = field_plans
            .iter()
            .filter(|f| {
                f.compatibility == RestoreFieldCompatibility::PartiallySupported
                    || f.compatibility == RestoreFieldCompatibility::MetadataOnly
            })
            .count();
        let unsupported_count = field_plans
            .iter()
            .filter(|f| {
                f.compatibility == RestoreFieldCompatibility::Unsupported
                    || f.compatibility == RestoreFieldCompatibility::ManualActionRequired
            })
            .count();

        let mut table_warnings = warnings_for_fields(&table_name, &field_plans);
        all_warnings.append(&mut table_warnings);

        table_plans.push(RestoreTablePlan {
            table_id,
            table_name,
            field_count: field_plans.len(),
            record_count: 0, // record count is not available from schema alone
            fields: field_plans,
            linked_record_plans,
            attachment_plans,
            restorable_field_count: restorable_count,
            partial_field_count: partial_count,
            unsupported_field_count: unsupported_count,
        });
    }

    // Pass through any validation warnings.
    for vw in &validation.warnings {
        all_warnings.push(crate::restore::plan::RestoreDryRunWarning {
            code: format!("VALIDATION_{}", vw.code),
            message: vw.message.clone(),
            table_name: None,
            field_name: None,
        });
    }

    let status = if all_warnings.is_empty() {
        RestorePlanStatus::Ready
    } else {
        RestorePlanStatus::ReadyWithWarnings
    };

    RestoreDryRunPlan {
        filename,
        status,
        target_mode: request.target_mode.clone(),
        target_base_name: request.target_base_name.clone(),
        package_summary: Some(package_summary),
        tables: table_plans,
        ordering: Some(build_ordering_plan()),
        warnings: all_warnings,
        errors: vec![],
        no_changes_made: true,
    }
}

fn blocked_plan(
    filename: String,
    request: &RestoreDryRunRequest,
    code: &str,
    message: &str,
) -> RestoreDryRunPlan {
    RestoreDryRunPlan {
        filename,
        status: RestorePlanStatus::Blocked,
        target_mode: request.target_mode.clone(),
        target_base_name: request.target_base_name.clone(),
        package_summary: None,
        tables: vec![],
        ordering: None,
        warnings: vec![],
        errors: vec![RestoreDryRunError {
            code: code.to_string(),
            message: message.to_string(),
        }],
        no_changes_made: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::manifest::{
        ManifestContents, ManifestPackage, ManifestSecurity, ManifestSource, PackageManifest,
    };
    use crate::backup::package::{PackageInput, TableRecords};
    use crate::backup::writer::write_package;
    use crate::restore::plan::RestoreTargetMode;
    use tempfile::tempdir;

    fn write_package_with_schema(
        dir: &tempfile::TempDir,
        name: &str,
        schema_json: serde_json::Value,
        tables: usize,
        fields: usize,
        records: usize,
    ) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let manifest = PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: "appDryRun01".to_string(),
                base_name: "Dry Run Test Base".to_string(),
                workspace_id: None,
            },
            ManifestContents {
                tables,
                fields,
                records,
                linked_record_relationships: 0,
                attachments: 0,
            },
            ManifestSecurity {
                contains_record_data: records > 0,
                contains_attachment_urls: false,
                encrypted: false,
                redactions_applied: vec![],
            },
            ManifestPackage {
                generated_by_app: "airbridge".to_string(),
                package_id: "00000000-0000-0000-0000-000000000020".to_string(),
            },
        );
        let input = PackageInput {
            manifest_json: serde_json::to_vec(&manifest).unwrap(),
            base_json: br#"{"baseId":"appDryRun01"}"#.to_vec(),
            schema_json: serde_json::to_vec(&schema_json).unwrap(),
            backup_report_json: br#"{"status":"ok"}"#.to_vec(),
            tables: if records > 0 {
                vec![TableRecords {
                    table_id: "tblDR01".to_string(),
                    lines: vec![r#"{"id":"rec001","fields":{"Name":"Row 1"}}"#.to_string()],
                }]
            } else {
                vec![]
            },
            ..Default::default()
        };
        write_package(&path, &input).expect("write");
        path
    }

    fn simple_schema() -> serde_json::Value {
        serde_json::json!({
            "tables": [{
                "id": "tblDR01",
                "name": "Projects",
                "fields": [
                    {"id": "fld01", "name": "Name", "type": "singleLineText"},
                    {"id": "fld02", "name": "Status", "type": "singleSelect"},
                    {"id": "fld03", "name": "Notes", "type": "multilineText"}
                ]
            }]
        })
    }

    fn complex_schema() -> serde_json::Value {
        serde_json::json!({
            "tables": [
                {
                    "id": "tblDR01",
                    "name": "Projects",
                    "fields": [
                        {"id": "fld01", "name": "Name", "type": "singleLineText"},
                        {"id": "fld02", "name": "Calc", "type": "formula"},
                        {"id": "fld03", "name": "Rollup", "type": "rollup"},
                        {"id": "fld04", "name": "Files", "type": "multipleAttachments"},
                        {"id": "fld05", "name": "Related", "type": "multipleRecordLinks",
                         "options": {"linkedTableId": "tblDR02"}}
                    ]
                },
                {
                    "id": "tblDR02",
                    "name": "Tasks",
                    "fields": [
                        {"id": "fld06", "name": "Title", "type": "singleLineText"}
                    ]
                }
            ]
        })
    }

    fn make_request(path: &str) -> RestoreDryRunRequest {
        RestoreDryRunRequest {
            path: path.to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: Some("My Restored Base".to_string()),
        }
    }

    #[test]
    fn valid_package_produces_ready_or_ready_with_warnings() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", simple_schema(), 1, 3, 1);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        assert!(
            plan.status == RestorePlanStatus::Ready
                || plan.status == RestorePlanStatus::ReadyWithWarnings,
            "expected ready/ready_with_warnings, got {:?}: {:?}",
            plan.status,
            plan.errors
        );
    }

    #[test]
    fn nonexistent_file_produces_blocked_plan() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("does_not_exist.airbridge");
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        assert_eq!(plan.status, RestorePlanStatus::Blocked);
        assert!(!plan.errors.is_empty());
    }

    #[test]
    fn result_contains_filename_only_not_full_path() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "mybackup.airbridge", simple_schema(), 1, 3, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        assert_eq!(plan.filename, "mybackup.airbridge");
        assert!(!plan.filename.contains('/'));
        assert!(!plan.filename.contains('\\'));
    }

    #[test]
    fn result_does_not_expose_full_path() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", simple_schema(), 1, 3, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        let dir_str = dir.path().to_string_lossy();
        assert!(
            !json.contains(dir_str.as_ref()),
            "full path leaked in serialized plan"
        );
    }

    #[test]
    fn result_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_dry_run_test_sentinel_0123456789";
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", simple_schema(), 1, 3, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn attachment_fields_become_metadata_only() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", complex_schema(), 2, 6, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        let projects = plan
            .tables
            .iter()
            .find(|t| t.table_id == "tblDR01")
            .unwrap();
        let files_field = projects
            .fields
            .iter()
            .find(|f| f.field_type == "multipleAttachments")
            .unwrap();
        assert_eq!(
            files_field.compatibility,
            RestoreFieldCompatibility::MetadataOnly
        );
        assert!(!projects.attachment_plans.is_empty());
    }

    #[test]
    fn linked_record_fields_require_remapping() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", complex_schema(), 2, 6, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        let projects = plan
            .tables
            .iter()
            .find(|t| t.table_id == "tblDR01")
            .unwrap();
        assert!(!projects.linked_record_plans.is_empty());
        assert!(projects.linked_record_plans[0].remapping_required);
    }

    #[test]
    fn computed_fields_generate_warnings() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", complex_schema(), 2, 6, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        assert!(plan
            .warnings
            .iter()
            .any(|w| w.code == "COMPUTED_FIELD_NOT_RESTORED"
                || w.code == "UNSUPPORTED_FIELD_MANUAL_RECREATION"));
    }

    #[test]
    fn ordering_plan_tables_before_fields_before_records_before_links() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", simple_schema(), 1, 3, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        let ord = plan.ordering.expect("ordering present");
        assert!(ord.create_tables_first);
        assert!(ord.create_fields_after_tables);
        assert!(ord.import_records_without_links);
        assert!(ord.apply_links_after_records);
    }

    #[test]
    fn new_base_target_mode_serializes() {
        let req = RestoreDryRunRequest {
            path: "/tmp/test.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let json = serde_json::to_string(&req.target_mode).expect("serialize");
        assert!(json.contains("newBase"));
    }

    #[test]
    fn empty_existing_base_target_mode_serializes() {
        let req = RestoreDryRunRequest {
            path: "/tmp/test.airbridge".to_string(),
            target_mode: RestoreTargetMode::EmptyExistingBase,
            target_base_name: None,
        };
        let json = serde_json::to_string(&req.target_mode).expect("serialize");
        assert!(json.contains("emptyExistingBase"));
    }

    #[test]
    fn package_not_extracted_to_disk() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", simple_schema(), 1, 3, 0);
        let req = make_request(path.to_str().unwrap());
        let _plan = create_dry_run_plan(&req);
        // Only the original .airbridge file should exist — no extracted entries.
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1, "no files should be extracted");
        assert_eq!(entries[0].file_name().to_string_lossy(), "test.airbridge");
    }

    #[test]
    fn command_does_not_require_token_field() {
        let req = RestoreDryRunRequest {
            path: "/tmp/x.airbridge".to_string(),
            target_mode: RestoreTargetMode::NewBase,
            target_base_name: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("token"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn missing_manifest_blocks_planning() {
        use std::io::Write;
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("no_manifest.airbridge");
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("other.txt", opts).unwrap();
        zip.write_all(b"dummy").unwrap();
        zip.finish().unwrap();
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        assert_eq!(plan.status, RestorePlanStatus::Blocked);
    }

    #[test]
    fn checksum_mismatch_propagates_to_plan() {
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
                base_id: "appDryRun02".to_string(),
                base_name: "Checksum Test".to_string(),
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
                package_id: "00000000-0000-0000-0000-000000000021".to_string(),
            },
        );
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        let base_bytes = br#"{"baseId":"appDryRun02"}"#;
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

        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        // Checksum mismatch → Invalid → Blocked
        assert_eq!(plan.status, RestorePlanStatus::Blocked);
    }

    #[test]
    fn no_changes_made_is_always_true() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", simple_schema(), 1, 3, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        assert!(plan.no_changes_made);
    }

    #[test]
    fn plan_serializes_to_json() {
        let dir = tempdir().expect("tempdir");
        let path = write_package_with_schema(&dir, "test.airbridge", simple_schema(), 1, 3, 0);
        let req = make_request(path.to_str().unwrap());
        let plan = create_dry_run_plan(&req);
        let json = serde_json::to_string(&plan).expect("serialize");
        assert!(json.contains("noChangesMade"));
        assert!(json.contains("status"));
    }
}
