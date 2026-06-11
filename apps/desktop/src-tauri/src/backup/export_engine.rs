use thiserror::Error;

use crate::airtable::client::AirtableClient;
use crate::airtable::errors::AirtableClientError;
use crate::airtable::http::HttpTransport;
use crate::airtable::models::AirtableRecord;
use crate::airtable::pagination::{ListRecordsOptions, PageSize, PaginationOffset};
use crate::backup::attachments::{attachment_metadata_to_jsonl, extract_attachment_metadata};
use crate::backup::export_result::{RecordExportEngineResult, TableExportResult};
use crate::backup::linked_records::{extract_linked_references, references_to_jsonl};
use crate::backup::record_jsonl::records_to_jsonl_lines;

/// Maximum pages the engine will fetch for a single table before stopping.
/// Guards against runaway loops on unexpectedly large tables.
pub const MAX_PAGES_PER_TABLE: usize = 10_000;

/// Errors produced by the record export engine.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ExportEngineError {
    #[error("authentication failed: invalid or expired token")]
    InvalidToken,

    #[error("rate limited by the Airtable API")]
    RateLimited,

    #[error("permission denied for base or table")]
    PermissionDenied,

    #[error("missing required token scope")]
    MissingScope,

    #[error("resource not found")]
    NotFound,

    #[error("malformed response from Airtable API: {0}")]
    MalformedResponse(String),

    #[error("transient server error (HTTP {0})")]
    TransientServerError(u16),

    #[error("record serialisation failed: {0}")]
    SerialisationError(String),

    #[error("page limit reached for table {table_id} after {pages} pages")]
    PageLimitReached { table_id: String, pages: usize },
}

impl From<AirtableClientError> for ExportEngineError {
    fn from(err: AirtableClientError) -> Self {
        match err {
            AirtableClientError::InvalidToken => ExportEngineError::InvalidToken,
            AirtableClientError::RateLimited => ExportEngineError::RateLimited,
            AirtableClientError::PermissionDenied => ExportEngineError::PermissionDenied,
            AirtableClientError::MissingScope => ExportEngineError::MissingScope,
            AirtableClientError::NotFound => ExportEngineError::NotFound,
            AirtableClientError::MalformedResponse(msg) => {
                ExportEngineError::MalformedResponse(msg)
            }
            AirtableClientError::TransientServerError(s) => {
                ExportEngineError::TransientServerError(s)
            }
            AirtableClientError::ValidationError(msg) => ExportEngineError::MalformedResponse(msg),
        }
    }
}

/// Describes a single table to be exported.
#[derive(Debug, Clone)]
pub struct TableExportSpec {
    pub table_id: String,
    pub table_name: String,
    /// Field names classified as `multipleRecordLinks` in the schema.
    pub linked_field_names: Vec<String>,
    /// Field names classified as `multipleAttachments` in the schema.
    pub attachment_field_names: Vec<String>,
}

/// Run the paginated export engine for a set of tables.
///
/// Fetches all records from each table using `client.list_records()` in a loop
/// following the `offset` cursor. Extracts linked-record references and
/// attachment metadata (no URLs) in the same pass.
///
/// No live network calls are made in tests — callers inject a
/// `MockHttpTransport` or `SequentialMockTransport`.
pub fn run_export<T: HttpTransport>(
    client: &AirtableClient<T>,
    base_id: &str,
    base_name: &str,
    tables: &[TableExportSpec],
    page_size: u32,
) -> Result<RecordExportEngineResult, ExportEngineError> {
    let mut table_results = Vec::new();
    let mut all_linked_refs = Vec::new();
    let mut all_attachment_meta = Vec::new();

    for spec in tables {
        let (records, pages_fetched) =
            fetch_all_records(client, base_id, &spec.table_id, page_size)?;

        let linked_names: Vec<&str> = spec.linked_field_names.iter().map(|s| s.as_str()).collect();
        let attach_names: Vec<&str> = spec
            .attachment_field_names
            .iter()
            .map(|s| s.as_str())
            .collect();

        let linked = extract_linked_references(&records, &linked_names);
        let attachments = extract_attachment_metadata(&records, &attach_names);

        all_linked_refs.extend(linked);
        all_attachment_meta.extend(attachments);

        let jsonl_lines = records_to_jsonl_lines(&records)
            .map_err(|e| ExportEngineError::SerialisationError(e.to_string()))?;

        let record_count = jsonl_lines.len();

        table_results.push(TableExportResult {
            table_id: spec.table_id.clone(),
            table_name: spec.table_name.clone(),
            jsonl_lines,
            record_count,
            pages_fetched,
        });
    }

    let linked_records_jsonl = references_to_jsonl(&all_linked_refs);
    let attachment_metadata_jsonl = attachment_metadata_to_jsonl(&all_attachment_meta);

    Ok(RecordExportEngineResult {
        base_id: base_id.to_string(),
        base_name: base_name.to_string(),
        tables: table_results,
        linked_records_jsonl,
        attachment_metadata_jsonl,
    })
}

fn fetch_all_records<T: HttpTransport>(
    client: &AirtableClient<T>,
    base_id: &str,
    table_id: &str,
    page_size: u32,
) -> Result<(Vec<AirtableRecord>, usize), ExportEngineError> {
    let mut all_records: Vec<AirtableRecord> = Vec::new();
    let mut offset: Option<String> = None;
    let mut pages_fetched: usize = 0;

    loop {
        let opts = ListRecordsOptions {
            page_size: Some(PageSize::new(page_size)),
            offset: offset.as_deref().map(|s| PaginationOffset(s.to_string())),
            ..Default::default()
        };

        let resp = client.list_records(base_id, table_id, &opts)?;
        pages_fetched += 1;
        all_records.extend(resp.records);
        offset = resp.offset;

        if offset.is_none() {
            break;
        }

        if pages_fetched >= MAX_PAGES_PER_TABLE {
            return Err(ExportEngineError::PageLimitReached {
                table_id: table_id.to_string(),
                pages: pages_fetched,
            });
        }
    }

    Ok((all_records, pages_fetched))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::auth::AirtableToken;
    use crate::airtable::http::{MockHttpTransport, SequentialMockTransport};

    const SENTINEL: &str = "pat_export_engine_test_sentinel_0123456789";

    fn client_mock(transport: MockHttpTransport) -> AirtableClient<MockHttpTransport> {
        AirtableClient::new(AirtableToken::new(SENTINEL), transport)
    }

    fn client_seq(transport: SequentialMockTransport) -> AirtableClient<SequentialMockTransport> {
        AirtableClient::new(AirtableToken::new(SENTINEL), transport)
    }

    fn spec(table_id: &str) -> TableExportSpec {
        TableExportSpec {
            table_id: table_id.to_string(),
            table_name: format!("Table {table_id}"),
            linked_field_names: vec![],
            attachment_field_names: vec![],
        }
    }

    fn spec_with_fields(
        table_id: &str,
        linked: Vec<&str>,
        attachments: Vec<&str>,
    ) -> TableExportSpec {
        TableExportSpec {
            table_id: table_id.to_string(),
            table_name: format!("Table {table_id}"),
            linked_field_names: linked.into_iter().map(|s| s.to_string()).collect(),
            attachment_field_names: attachments.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn empty_table_returns_zero_records() {
        let transport = MockHttpTransport::ok(r#"{"records":[]}"#);
        let client = client_mock(transport);
        let result = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100)
            .expect("should succeed");
        assert_eq!(result.tables[0].record_count, 0);
        assert_eq!(result.tables[0].pages_fetched, 1);
    }

    #[test]
    fn single_page_with_records() {
        let body = r#"{"records":[
            {"id":"rec001","fields":{"Name":"Alpha"},"createdTime":"2026-01-01T00:00:00.000Z"},
            {"id":"rec002","fields":{"Name":"Beta"},"createdTime":"2026-01-01T00:00:00.000Z"}
        ]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_mock(transport);
        let result = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100)
            .expect("should succeed");
        assert_eq!(result.tables[0].record_count, 2);
        assert_eq!(result.tables[0].pages_fetched, 1);
    }

    #[test]
    fn two_page_pagination_fetches_all_records() {
        let page1 = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}],"offset":"cursor_page2"}"#;
        let page2 = r#"{"records":[{"id":"rec002","fields":{},"createdTime":null}]}"#;
        let transport = SequentialMockTransport::new(vec![(200, page1), (200, page2)]);
        let client = client_seq(transport);
        let result = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100)
            .expect("should succeed");
        assert_eq!(result.tables[0].record_count, 2);
        assert_eq!(result.tables[0].pages_fetched, 2);
    }

    #[test]
    fn three_page_pagination_accumulates_records() {
        let page1 = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}],"offset":"c2"}"#;
        let page2 = r#"{"records":[{"id":"rec002","fields":{},"createdTime":null}],"offset":"c3"}"#;
        let page3 = r#"{"records":[{"id":"rec003","fields":{},"createdTime":null}]}"#;
        let transport =
            SequentialMockTransport::new(vec![(200, page1), (200, page2), (200, page3)]);
        let client = client_seq(transport);
        let result = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100)
            .expect("should succeed");
        assert_eq!(result.tables[0].record_count, 3);
        assert_eq!(result.tables[0].pages_fetched, 3);
    }

    #[test]
    fn two_tables_both_exported() {
        let body = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_mock(transport);
        let result = run_export(
            &client,
            "appSyn01",
            "Synthetic",
            &[spec("tbl01"), spec("tbl02")],
            100,
        )
        .expect("should succeed");
        assert_eq!(result.tables.len(), 2);
        assert_eq!(result.tables[0].table_id, "tbl01");
        assert_eq!(result.tables[1].table_id, "tbl02");
    }

    #[test]
    fn result_total_records_sums_tables() {
        let body_one = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}]}"#;
        let transport = MockHttpTransport::ok(body_one);
        let client = client_mock(transport);
        let result = run_export(
            &client,
            "appSyn01",
            "Synthetic",
            &[spec("tbl01"), spec("tbl02")],
            100,
        )
        .expect("should succeed");
        assert_eq!(result.total_records(), 2);
    }

    #[test]
    fn jsonl_lines_are_valid_json_objects() {
        let body = r#"{"records":[{"id":"rec001","fields":{"Name":"Alice"},"createdTime":"2026-01-01T00:00:00.000Z"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_mock(transport);
        let result = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100)
            .expect("should succeed");
        for line in &result.tables[0].jsonl_lines {
            let _: serde_json::Value = serde_json::from_str(line).expect("valid json");
        }
    }

    #[test]
    fn jsonl_lines_contain_record_id() {
        let body = r#"{"records":[{"id":"recSpecificId01","fields":{},"createdTime":null}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_mock(transport);
        let result = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100)
            .expect("should succeed");
        assert!(result.tables[0].jsonl_lines[0].contains("recSpecificId01"));
    }

    #[test]
    fn error_401_maps_to_invalid_token() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let client = client_mock(transport);
        let err = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100).unwrap_err();
        assert_eq!(err, ExportEngineError::InvalidToken);
    }

    #[test]
    fn error_429_maps_to_rate_limited() {
        let transport = MockHttpTransport::with_status(429, r#"{"error":"RATE_LIMITED"}"#);
        let client = client_mock(transport);
        let err = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100).unwrap_err();
        assert_eq!(err, ExportEngineError::RateLimited);
    }

    #[test]
    fn error_403_maps_to_permission_denied() {
        let transport = MockHttpTransport::with_status(403, r#"{"error":"forbidden"}"#);
        let client = client_mock(transport);
        let err = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100).unwrap_err();
        assert_eq!(err, ExportEngineError::PermissionDenied);
    }

    #[test]
    fn error_500_maps_to_transient_server_error() {
        let transport = MockHttpTransport::with_status(500, "Internal Server Error");
        let client = client_mock(transport);
        let err = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100).unwrap_err();
        assert_eq!(err, ExportEngineError::TransientServerError(500));
    }

    #[test]
    fn linked_records_extracted_and_in_result() {
        use serde_json::json;
        let body = serde_json::to_string(&serde_json::json!({
            "records": [{
                "id": "recSrc01",
                "fields": {
                    "Tasks": ["recLink01", "recLink02"]
                },
                "createdTime": null
            }]
        }))
        .unwrap();
        let transport = MockHttpTransport::ok(body);
        let client = client_mock(transport);
        let result = run_export(
            &client,
            "appSyn01",
            "Synthetic",
            &[spec_with_fields("tbl01", vec!["Tasks"], vec![])],
            100,
        )
        .expect("should succeed");
        assert!(!result.linked_records_jsonl.is_empty());
        let text = String::from_utf8(result.linked_records_jsonl).expect("utf8");
        assert!(text.contains("recLink01"));
    }

    #[test]
    fn attachment_metadata_extracted_without_url() {
        let body = serde_json::to_string(&serde_json::json!({
            "records": [{
                "id": "recSrc01",
                "fields": {
                    "Files": [{
                        "id": "attAbc01",
                        "filename": "photo.png",
                        "type": "image/png",
                        "size": 1024,
                        "url": "https://dl.airtable.com/REDACTED"
                    }]
                },
                "createdTime": null
            }]
        }))
        .unwrap();
        let transport = MockHttpTransport::ok(body);
        let client = client_mock(transport);
        let result = run_export(
            &client,
            "appSyn01",
            "Synthetic",
            &[spec_with_fields("tbl01", vec![], vec!["Files"])],
            100,
        )
        .expect("should succeed");
        assert!(!result.attachment_metadata_jsonl.is_empty());
        let text = String::from_utf8(result.attachment_metadata_jsonl).expect("utf8");
        assert!(text.contains("photo.png"));
        assert!(!text.contains("dl.airtable.com"));
        assert!(!text.contains("https://"));
    }

    #[test]
    fn result_does_not_contain_token_sentinel() {
        let transport = MockHttpTransport::ok(r#"{"records":[]}"#);
        let client = client_mock(transport);
        let result = run_export(&client, "appSyn01", "Synthetic", &[spec("tbl01")], 100)
            .expect("should succeed");
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn base_id_and_name_preserved_in_result() {
        let transport = MockHttpTransport::ok(r#"{"records":[]}"#);
        let client = client_mock(transport);
        let result = run_export(&client, "appSyn01", "My Base", &[], 100).expect("should succeed");
        assert_eq!(result.base_id, "appSyn01");
        assert_eq!(result.base_name, "My Base");
    }

    #[test]
    fn no_tables_returns_empty_result() {
        let transport = MockHttpTransport::ok(r#"{"records":[]}"#);
        let client = client_mock(transport);
        let result =
            run_export(&client, "appSyn01", "Synthetic", &[], 100).expect("should succeed");
        assert!(result.tables.is_empty());
        assert!(result.linked_records_jsonl.is_empty());
        assert!(result.attachment_metadata_jsonl.is_empty());
    }
}
