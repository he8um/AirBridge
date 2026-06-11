/// Integration test: mock two-table pagination → engine → PackageInput →
/// write to tempdir → validate → assert entries and safety constraints.
///
/// No live network calls. No token persistence. Package written to tempdir only.
use airbridge_desktop_lib::airtable::auth::AirtableToken;
use airbridge_desktop_lib::airtable::client::AirtableClient;
use airbridge_desktop_lib::airtable::http::SequentialMockTransport;
use airbridge_desktop_lib::backup::export_engine::{run_export, TableExportSpec};
use airbridge_desktop_lib::backup::export_result::build_package_input;
use airbridge_desktop_lib::backup::validation::{validate_package, ValidationStatus};
use airbridge_desktop_lib::backup::writer::write_package;

const SENTINEL: &str = "pat_integration_test_sentinel_0123456789";

fn make_client(responses: Vec<(u16, &str)>) -> AirtableClient<SequentialMockTransport> {
    AirtableClient::new(
        AirtableToken::new(SENTINEL),
        SequentialMockTransport::new(responses),
    )
}

fn spec(table_id: &str, linked: Vec<&str>, attachments: Vec<&str>) -> TableExportSpec {
    TableExportSpec {
        table_id: table_id.to_string(),
        table_name: format!("Table {table_id}"),
        linked_field_names: linked.into_iter().map(|s| s.to_string()).collect(),
        attachment_field_names: attachments.into_iter().map(|s| s.to_string()).collect(),
    }
}

/// Two-table, two-page test:
/// - Table 1 (tbl01): 2 pages, linked field "Tasks", attachment field "Files"
/// - Table 2 (tbl02): 1 page, no linked/attachment fields
#[test]
fn two_table_paginated_export_writes_valid_package() {
    let tbl01_page1 = serde_json::json!({
        "records": [
            {
                "id": "rec001",
                "fields": {
                    "Name": "Alpha",
                    "Tasks": ["recLink01"],
                    "Files": [{
                        "id": "attAbc01",
                        "filename": "photo.png",
                        "type": "image/png",
                        "size": 1024,
                        "url": "https://dl.airtable.com/REDACTED_DO_NOT_STORE"
                    }]
                },
                "createdTime": "2026-01-01T00:00:00.000Z"
            }
        ],
        "offset": "cursor_tbl01_page2"
    })
    .to_string();

    let tbl01_page2 = serde_json::json!({
        "records": [
            {
                "id": "rec002",
                "fields": { "Name": "Beta", "Tasks": [], "Files": [] },
                "createdTime": "2026-01-01T00:00:00.000Z"
            }
        ]
    })
    .to_string();

    let tbl02_page1 = serde_json::json!({
        "records": [
            {
                "id": "rec101",
                "fields": { "Title": "Item One" },
                "createdTime": "2026-01-01T00:00:00.000Z"
            },
            {
                "id": "rec102",
                "fields": { "Title": "Item Two" },
                "createdTime": "2026-01-01T00:00:00.000Z"
            }
        ]
    })
    .to_string();

    let responses = vec![
        (200, tbl01_page1.as_str()),
        (200, tbl01_page2.as_str()),
        (200, tbl02_page1.as_str()),
    ];
    let client = make_client(responses);

    let tables = vec![
        spec("tbl01", vec!["Tasks"], vec!["Files"]),
        spec("tbl02", vec![], vec![]),
    ];

    let engine_result = run_export(&client, "appSyn01", "Synthetic Base", &tables, 100)
        .expect("export should succeed");

    // Table 1: 2 pages, 2 records
    assert_eq!(engine_result.tables[0].table_id, "tbl01");
    assert_eq!(engine_result.tables[0].record_count, 2);
    assert_eq!(engine_result.tables[0].pages_fetched, 2);

    // Table 2: 1 page, 2 records
    assert_eq!(engine_result.tables[1].table_id, "tbl02");
    assert_eq!(engine_result.tables[1].record_count, 2);
    assert_eq!(engine_result.tables[1].pages_fetched, 1);

    // Linked records extracted
    let linked_text = String::from_utf8(engine_result.linked_records_jsonl.clone()).expect("utf8");
    assert!(
        linked_text.contains("recLink01"),
        "linked ref should be present"
    );

    // Attachment metadata: filename present, URL absent
    let attach_text =
        String::from_utf8(engine_result.attachment_metadata_jsonl.clone()).expect("utf8");
    assert!(
        attach_text.contains("photo.png"),
        "filename should be present"
    );
    assert!(
        !attach_text.contains("dl.airtable.com"),
        "URL must not be stored"
    );
    assert!(!attach_text.contains("https://"), "URL must not be stored");

    // Build PackageInput
    let manifest_json = serde_json::to_vec(&serde_json::json!({
        "format": "airbridge",
        "formatVersion": "0.1.0",
        "appVersion": "0.1.0",
        "createdAt": "2026-06-11T00:00:00Z",
        "source": {
            "provider": "airtable",
            "baseId": "appSyn01",
            "baseName": "Synthetic Base"
        },
        "contents": {
            "tables": 2,
            "fields": 0,
            "records": 4,
            "linkedRecordRelationships": 1,
            "attachments": 1
        },
        "security": {
            "containsRecordData": true,
            "containsAttachmentUrls": false,
            "encrypted": false,
            "redactionsApplied": []
        },
        "package": {
            "generatedByApp": "airbridge",
            "packageId": "00000000-0000-0000-0000-000000000099"
        }
    }))
    .expect("manifest");

    let base_json =
        serde_json::to_vec(&serde_json::json!({"baseId": "appSyn01", "name": "Synthetic Base"}))
            .expect("base");
    let schema_json = serde_json::to_vec(&serde_json::json!({"tables": []})).expect("schema");
    let backup_report_json =
        serde_json::to_vec(&serde_json::json!({"status": "ok"})).expect("report");
    let compat_report_json =
        serde_json::to_vec(&serde_json::json!({"status": "ok"})).expect("compat");

    let package_input = build_package_input(
        &engine_result,
        manifest_json,
        base_json,
        schema_json,
        backup_report_json,
        compat_report_json,
    );

    assert!(
        package_input.is_complete(),
        "package input must be complete"
    );
    assert_eq!(package_input.tables.len(), 2);

    // Write to tempdir — NEVER outside temp
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let pkg_path = tmp_dir.path().join("test_export.airbridge");

    write_package(&pkg_path, &package_input).expect("write_package should succeed");
    assert!(pkg_path.exists(), "package file should exist");

    // Validate the written package
    let report = validate_package(&pkg_path);
    assert_eq!(
        report.status,
        ValidationStatus::Valid,
        "package must be valid; errors: {:?}",
        report.errors
    );
    assert!(report.entry_count > 0, "package must have entries");

    // Assert no token sentinel in JSONL output
    for table in &engine_result.tables {
        for line in &table.jsonl_lines {
            assert!(
                !line.contains(SENTINEL),
                "JSONL line must not contain token sentinel"
            );
        }
    }

    // Assert no absolute paths in JSONL
    for table in &engine_result.tables {
        for line in &table.jsonl_lines {
            assert!(
                !line.contains("/Users/"),
                "JSONL line must not contain absolute paths"
            );
        }
    }
}

/// Single-page empty table export writes a valid package.
#[test]
fn empty_table_export_writes_valid_package() {
    let responses = vec![(200, r#"{"records":[]}"#)];
    let client = make_client(responses);

    let engine_result = run_export(
        &client,
        "appEmpty01",
        "Empty Base",
        &[spec("tbl01", vec![], vec![])],
        100,
    )
    .expect("export should succeed");

    assert_eq!(engine_result.total_records(), 0);

    let pkg_input = build_package_input(
        &engine_result,
        serde_json::to_vec(&serde_json::json!({"format":"airbridge","formatVersion":"0.1.0","appVersion":"0.1.0","createdAt":"2026-06-11T00:00:00Z","source":{"provider":"airtable","baseId":"appEmpty01","baseName":"Empty Base"},"contents":{"tables":1,"fields":0,"records":0,"linkedRecordRelationships":0,"attachments":0},"security":{"containsRecordData":false,"containsAttachmentUrls":false,"encrypted":false,"redactionsApplied":[]},"package":{"generatedByApp":"airbridge","packageId":"00000000-0000-0000-0000-000000000001"}})).expect("manifest"),
        serde_json::to_vec(&serde_json::json!({"baseId":"appEmpty01"})).expect("base"),
        serde_json::to_vec(&serde_json::json!({"tables":[]})).expect("schema"),
        serde_json::to_vec(&serde_json::json!({"status":"ok"})).expect("report"),
        serde_json::to_vec(&serde_json::json!({"status":"ok"})).expect("compat"),
    );

    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let pkg_path = tmp_dir.path().join("empty_export.airbridge");

    write_package(&pkg_path, &pkg_input).expect("write_package should succeed");

    let report = validate_package(&pkg_path);
    assert_eq!(
        report.status,
        ValidationStatus::Valid,
        "empty package must be valid; errors: {:?}",
        report.errors
    );
}

/// Package path stays within temp directory.
#[test]
fn package_path_is_in_temp_directory() {
    let responses = vec![(200, r#"{"records":[]}"#)];
    let client = make_client(responses);
    let engine_result = run_export(&client, "appSyn01", "Synthetic", &[], 100).expect("export");

    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let pkg_path = tmp_dir.path().join("package.airbridge");

    // Verify the path is within the system temp directory
    let tmp_root = std::env::temp_dir();
    assert!(
        pkg_path.starts_with(&tmp_root),
        "package path must be under temp dir, got: {}",
        pkg_path.display()
    );

    let _ = engine_result; // not writing, just checking path safety
}
