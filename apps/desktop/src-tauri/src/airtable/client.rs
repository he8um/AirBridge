use super::auth::AirtableToken;
use super::endpoints;
use super::errors::{map_http_error, AirtableClientError};
use super::http::{HttpRequest, HttpTransport};
use super::models::{
    AccessibleBase, AccessibleBaseSummary, AirtableListRecordsResponse, AirtableRecordFields,
    AirtableRecordUpdate, AirtableTable, ConnectionCheckOutcome, CreateSandboxRecordOutcome,
    CreateSandboxRecordRequest, CreateTableOutcome, CreateTableRequest, ListBasesResponse,
};
use super::pagination::ListRecordsOptions;

type ClientResult<T> = Result<T, AirtableClientError>;

/// Airtable API client, generic over its HTTP transport.
///
/// The transport abstraction allows tests to inject a `MockHttpTransport`
/// without making any network calls.
pub struct AirtableClient<T: HttpTransport> {
    token: AirtableToken,
    transport: T,
}

impl<T: HttpTransport> AirtableClient<T> {
    pub fn new(token: AirtableToken, transport: T) -> Self {
        AirtableClient { token, transport }
    }

    // ── Internal helpers ───────────────────────────────────────────────────

    fn auth_header(&self) -> String {
        self.token.authorization_header_value()
    }

    fn send_get(&self, url: String, query: Vec<(String, String)>) -> ClientResult<String> {
        let mut req = HttpRequest::get(url).with_header("Authorization", self.auth_header());
        for (k, v) in query {
            req = req.with_query(k, v);
        }
        self.execute(req)
    }

    fn send_post(&self, url: String, body: String) -> ClientResult<String> {
        let req = HttpRequest::post(url, body)
            .with_header("Authorization", self.auth_header())
            .with_header("Content-Type", "application/json");
        self.execute(req)
    }

    fn send_patch(&self, url: String, body: String) -> ClientResult<String> {
        let req = HttpRequest::patch(url, body)
            .with_header("Authorization", self.auth_header())
            .with_header("Content-Type", "application/json");
        self.execute(req)
    }

    fn execute(&self, request: HttpRequest) -> ClientResult<String> {
        let resp = self
            .transport
            .send(request)
            .map_err(|_| AirtableClientError::TransientServerError(0))?;

        if resp.is_success() {
            Ok(resp.body)
        } else {
            Err(map_http_error(resp.status, &resp.body))
        }
    }

    // ── Public API methods ─────────────────────────────────────────────────

    /// Retrieve the table schema for a base.
    pub fn get_base_schema(&self, base_id: &str) -> ClientResult<Vec<AirtableTable>> {
        let url = endpoints::base_schema_path(base_id);
        let body = self.send_get(url, vec![])?;

        #[derive(serde::Deserialize)]
        struct TablesResponse {
            tables: Vec<AirtableTable>,
        }

        serde_json::from_str::<TablesResponse>(&body)
            .map(|r| r.tables)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))
    }

    /// List records from a table, with pagination support.
    pub fn list_records(
        &self,
        base_id: &str,
        table_id: &str,
        opts: &ListRecordsOptions,
    ) -> ClientResult<AirtableListRecordsResponse> {
        let url = endpoints::list_records_path(base_id, table_id);
        let params = super::pagination::build_list_query_params(opts);
        let query: Vec<(String, String)> = params.into_iter().collect();
        let body = self.send_get(url, query)?;

        serde_json::from_str::<AirtableListRecordsResponse>(&body)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))
    }

    /// Create records in a table. The caller is responsible for splitting
    /// into batches via `records::split_create_batches` before calling this.
    pub fn create_records(
        &self,
        base_id: &str,
        table_id: &str,
        records: Vec<AirtableRecordFields>,
    ) -> ClientResult<AirtableListRecordsResponse> {
        let url = endpoints::create_records_path(base_id, table_id);
        let payload = super::models::AirtableCreateRecordsRequest { records };
        let body_str = serde_json::to_string(&payload)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))?;
        let body = self.send_post(url, body_str)?;

        serde_json::from_str::<AirtableListRecordsResponse>(&body)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))
    }

    /// Update records in a table (PATCH). The caller is responsible for
    /// batching via `records::split_update_batches`.
    pub fn update_records(
        &self,
        base_id: &str,
        table_id: &str,
        records: Vec<AirtableRecordUpdate>,
    ) -> ClientResult<AirtableListRecordsResponse> {
        let url = endpoints::update_records_path(base_id, table_id);
        let payload = super::models::AirtableUpdateRecordsRequest { records };
        let body_str = serde_json::to_string(&payload)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))?;
        let body = self.send_patch(url, body_str)?;

        serde_json::from_str::<AirtableListRecordsResponse>(&body)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))
    }

    /// List all bases accessible to the token as lightweight summaries.
    ///
    /// Returns only id and name — no token, no record data.
    pub fn list_accessible_bases(&self) -> ClientResult<Vec<AccessibleBaseSummary>> {
        let url = endpoints::list_bases_path();
        let body = self.send_get(url, vec![])?;

        let response = serde_json::from_str::<ListBasesResponse>(&body)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))?;

        Ok(response
            .bases
            .into_iter()
            .map(|b| AccessibleBaseSummary {
                id: b.id,
                name: b.name,
            })
            .collect())
    }

    /// Performs a read-only connection check by calling the list-bases endpoint.
    ///
    /// This is the only method used for live connection verification. It does
    /// not perform any write operations. The token is used for this call only
    /// and is never included in the returned `ConnectionCheckOutcome`.
    pub fn check_connection_for_token(&self) -> ClientResult<ConnectionCheckOutcome> {
        let url = endpoints::list_bases_path();
        let body = self.send_get(url, vec![])?;

        let response = serde_json::from_str::<ListBasesResponse>(&body)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))?;

        let accessible_bases = response
            .bases
            .into_iter()
            .map(|b| AccessibleBase {
                id: super::models::AirtableBaseId(b.id),
                name: b.name,
            })
            .collect();

        Ok(ConnectionCheckOutcome { accessible_bases })
    }

    /// Creates a single minimal record in a sandbox table via the Records API.
    ///
    /// Safety invariants:
    /// - Never called from app runtime or Tauri commands.
    /// - Used only in the sandbox record write integration test.
    /// - Returns only a sanitized outcome (record_created, record_count, table_name).
    /// - No record ID is included in the returned outcome.
    /// - Token is used for the Authorization header and is not returned.
    /// - No linked fields, no attachments, no update operations.
    pub fn create_single_sandbox_record(
        &self,
        base_id: &str,
        table_id_or_name: &str,
        table_name: &str,
        request: &CreateSandboxRecordRequest,
    ) -> ClientResult<CreateSandboxRecordOutcome> {
        let url = endpoints::create_records_path(base_id, table_id_or_name);
        let payload = super::models::AirtableCreateRecordsRequest {
            records: vec![AirtableRecordFields {
                fields: request.fields.clone(),
            }],
        };
        let body_str = serde_json::to_string(&payload)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))?;
        let body = self.send_post(url, body_str)?;

        #[derive(serde::Deserialize)]
        struct CreateRecordsResponse {
            records: Vec<serde_json::Value>,
        }

        let resp = serde_json::from_str::<CreateRecordsResponse>(&body)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))?;

        let record_created = !resp.records.is_empty();
        Ok(CreateSandboxRecordOutcome {
            record_created,
            record_count: resp.records.len(),
            table_name: table_name.to_string(),
        })
    }

    /// Creates a table in a base via the Airtable Metadata API.
    ///
    /// Safety invariants:
    /// - Never called from app runtime or Tauri commands.
    /// - Used only in the sandbox schema write integration test.
    /// - Returns only table id and name — no raw HTTP body, no record payload,
    ///   no attachment URLs, no old/new record IDs.
    /// - Token is used for the Authorization header and is not returned.
    pub fn create_table(
        &self,
        base_id: &str,
        request: &CreateTableRequest,
    ) -> ClientResult<CreateTableOutcome> {
        let url = endpoints::create_table_path(base_id);
        let body_str = serde_json::to_string(request)
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))?;
        let body = self.send_post(url, body_str)?;

        #[derive(serde::Deserialize)]
        struct CreateTableResponse {
            id: String,
            name: String,
        }

        serde_json::from_str::<CreateTableResponse>(&body)
            .map(|r| CreateTableOutcome {
                table_id: r.id,
                table_name: r.name,
            })
            .map_err(|e| AirtableClientError::MalformedResponse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::auth::AirtableToken;
    use crate::airtable::http::MockHttpTransport;
    use crate::airtable::models::{
        CreateSandboxRecordRequest, CreateTableFieldSpec, CreateTableRequest,
    };
    use crate::airtable::pagination::ListRecordsOptions;

    const SENTINEL: &str = "pat_example_client_sentinel_0123456789";

    fn client_with(transport: MockHttpTransport) -> AirtableClient<MockHttpTransport> {
        AirtableClient::new(AirtableToken::new(SENTINEL), transport)
    }

    #[test]
    fn list_records_parses_empty_page() {
        let transport = MockHttpTransport::ok(r#"{"records":[]}"#);
        let client = client_with(transport);
        let result = client.list_records(
            "appTestBase001",
            "tblTestTable01",
            &ListRecordsOptions::default(),
        );
        let resp = result.expect("should succeed");
        assert_eq!(resp.records.len(), 0);
        assert!(resp.offset.is_none());
    }

    #[test]
    fn list_records_parses_records_and_offset() {
        let body = r#"{
            "records": [
                {"id": "recExample0001", "fields": {"Name": "Row A"}, "createdTime": "2025-01-01T00:00:00.000Z"}
            ],
            "offset": "cursor_next_page"
        }"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_with(transport);
        let resp = client
            .list_records(
                "appTestBase001",
                "tblTestTable01",
                &ListRecordsOptions::default(),
            )
            .expect("should succeed");
        assert_eq!(resp.records.len(), 1);
        assert_eq!(resp.offset.as_deref(), Some("cursor_next_page"));
    }

    #[test]
    fn list_records_maps_401_to_invalid_token() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let client = client_with(transport);
        let err = client
            .list_records(
                "appTestBase001",
                "tblTestTable01",
                &ListRecordsOptions::default(),
            )
            .unwrap_err();
        assert_eq!(err, AirtableClientError::InvalidToken);
    }

    #[test]
    fn list_records_maps_429_to_rate_limited() {
        let transport = MockHttpTransport::with_status(429, r#"{"error":"RATE_LIMITED"}"#);
        let client = client_with(transport);
        let err = client
            .list_records(
                "appTestBase001",
                "tblTestTable01",
                &ListRecordsOptions::default(),
            )
            .unwrap_err();
        assert_eq!(err, AirtableClientError::RateLimited);
    }

    #[test]
    fn list_records_maps_malformed_json_to_malformed_response() {
        let transport = MockHttpTransport::ok("not valid json {{{{");
        let client = client_with(transport);
        let err = client
            .list_records(
                "appTestBase001",
                "tblTestTable01",
                &ListRecordsOptions::default(),
            )
            .unwrap_err();
        match err {
            AirtableClientError::MalformedResponse(_) => {}
            other => panic!("expected MalformedResponse, got {other:?}"),
        }
    }

    #[test]
    fn list_records_response_does_not_contain_token() {
        let transport = MockHttpTransport::ok(r#"{"records":[]}"#);
        let client = client_with(transport);
        let resp = client
            .list_records(
                "appTestBase001",
                "tblTestTable01",
                &ListRecordsOptions::default(),
            )
            .expect("should succeed");
        let serialized = serde_json::to_string(&resp).expect("serialize");
        assert!(!serialized.contains(SENTINEL));
    }

    // ── list_accessible_bases tests ───────────────────────────────────────

    #[test]
    fn list_accessible_bases_parses_empty_list() {
        let transport = MockHttpTransport::ok(r#"{"bases":[]}"#);
        let client = client_with(transport);
        let bases = client.list_accessible_bases().expect("should succeed");
        assert_eq!(bases.len(), 0);
    }

    #[test]
    fn list_accessible_bases_returns_id_and_name() {
        let body = r#"{"bases":[
            {"id":"appExampleBase01","name":"Example Base One","permissionLevel":"create"},
            {"id":"appExampleBase02","name":"Example Base Two","permissionLevel":"read"}
        ]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_with(transport);
        let bases = client.list_accessible_bases().expect("should succeed");
        assert_eq!(bases.len(), 2);
        assert_eq!(bases[0].id, "appExampleBase01");
        assert_eq!(bases[0].name, "Example Base One");
    }

    #[test]
    fn list_accessible_bases_401_maps_to_invalid_token() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let client = client_with(transport);
        let err = client.list_accessible_bases().unwrap_err();
        assert_eq!(err, AirtableClientError::InvalidToken);
    }

    #[test]
    fn list_accessible_bases_result_does_not_contain_token() {
        let body = r#"{"bases":[{"id":"appExampleBase01","name":"Example Base One","permissionLevel":"create"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_with(transport);
        let bases = client.list_accessible_bases().expect("should succeed");
        let serialized = serde_json::to_string(&bases).expect("serialize");
        assert!(!serialized.contains(SENTINEL));
    }

    #[test]
    fn get_base_schema_parses_empty_tables() {
        let transport = MockHttpTransport::ok(r#"{"tables":[]}"#);
        let client = client_with(transport);
        let tables = client
            .get_base_schema("appTestBase001")
            .expect("should succeed");
        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn get_base_schema_maps_404_to_not_found() {
        let transport = MockHttpTransport::with_status(404, r#"{"error":"NOT_FOUND"}"#);
        let client = client_with(transport);
        let err = client.get_base_schema("appTestBase001").unwrap_err();
        assert_eq!(err, AirtableClientError::NotFound);
    }

    // ── check_connection_for_token tests ──────────────────────────────────

    #[test]
    fn check_connection_success_returns_accessible_bases() {
        let body = r#"{"bases":[{"id":"appExampleBase01","name":"Example Base","permissionLevel":"create"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_with(transport);
        let outcome = client.check_connection_for_token().expect("should succeed");
        assert_eq!(outcome.accessible_bases.len(), 1);
        assert_eq!(outcome.accessible_bases[0].name, "Example Base");
    }

    #[test]
    fn check_connection_empty_bases_is_valid() {
        let transport = MockHttpTransport::ok(r#"{"bases":[]}"#);
        let client = client_with(transport);
        let outcome = client.check_connection_for_token().expect("should succeed");
        assert_eq!(outcome.accessible_bases.len(), 0);
    }

    #[test]
    fn check_connection_401_maps_to_invalid_token() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let client = client_with(transport);
        let err = client.check_connection_for_token().unwrap_err();
        assert_eq!(err, AirtableClientError::InvalidToken);
    }

    #[test]
    fn check_connection_403_maps_to_permission_denied() {
        let transport = MockHttpTransport::with_status(403, r#"{"error":"forbidden"}"#);
        let client = client_with(transport);
        let err = client.check_connection_for_token().unwrap_err();
        assert_eq!(err, AirtableClientError::PermissionDenied);
    }

    #[test]
    fn check_connection_403_scope_maps_to_missing_scope() {
        let transport =
            MockHttpTransport::with_status(403, r#"{"error":"missing required scope"}"#);
        let client = client_with(transport);
        let err = client.check_connection_for_token().unwrap_err();
        assert_eq!(err, AirtableClientError::MissingScope);
    }

    #[test]
    fn check_connection_429_maps_to_rate_limited() {
        let transport = MockHttpTransport::with_status(429, r#"{"error":"RATE_LIMITED"}"#);
        let client = client_with(transport);
        let err = client.check_connection_for_token().unwrap_err();
        assert_eq!(err, AirtableClientError::RateLimited);
    }

    #[test]
    fn check_connection_malformed_json_maps_to_malformed_response() {
        let transport = MockHttpTransport::ok("this is not json");
        let client = client_with(transport);
        let err = client.check_connection_for_token().unwrap_err();
        match err {
            AirtableClientError::MalformedResponse(_) => {}
            other => panic!("expected MalformedResponse, got {other:?}"),
        }
    }

    #[test]
    fn check_connection_transport_error_maps_to_transient() {
        struct FailingTransport;
        impl crate::airtable::http::HttpTransport for FailingTransport {
            fn send(
                &self,
                _r: crate::airtable::http::HttpRequest,
            ) -> Result<crate::airtable::http::HttpResponse, String> {
                Err("network error: connection refused".to_string())
            }
        }
        let client = AirtableClient::new(AirtableToken::new(SENTINEL), FailingTransport);
        let err = client.check_connection_for_token().unwrap_err();
        assert_eq!(err, AirtableClientError::TransientServerError(0));
    }

    #[test]
    fn check_connection_result_does_not_contain_token() {
        let body = r#"{"bases":[{"id":"appExampleBase01","name":"Example Base","permissionLevel":"create"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_with(transport);
        let outcome = client.check_connection_for_token().expect("should succeed");
        let serialized = serde_json::to_string(&outcome).expect("serialize");
        assert!(!serialized.contains(SENTINEL));
    }

    #[test]
    fn check_connection_write_permissions_not_verified_in_outcome() {
        let body = r#"{"bases":[{"id":"appExampleBase01","name":"Example Base","permissionLevel":"create"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = client_with(transport);
        let outcome = client.check_connection_for_token().expect("should succeed");
        // ConnectionCheckOutcome has no write-permission fields — write checks
        // are not performed at this stage.
        let serialized = serde_json::to_string(&outcome).expect("serialize");
        assert!(!serialized.contains("write"));
    }

    // ── create_table tests ────────────────────────────────────────────────────

    // ── create_single_sandbox_record tests ───────────────────────────────────

    #[test]
    fn create_single_sandbox_record_returns_sanitized_outcome() {
        let body = r#"{"records":[{"id":"recNewRecord001","fields":{"Name":"Test"},"createdTime":"2025-01-01T00:00:00.000Z"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = AirtableClient::new(AirtableToken::new("pat_sentinel"), transport);
        let req = CreateSandboxRecordRequest {
            fields: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "Name".to_string(),
                    serde_json::Value::String("Test".to_string()),
                );
                m
            },
        };
        let outcome = client
            .create_single_sandbox_record("appTestBase001", "tblTest01", "SandboxTest", &req)
            .expect("should succeed");
        assert!(outcome.record_created);
        assert_eq!(outcome.record_count, 1);
        assert_eq!(outcome.table_name, "SandboxTest");
    }

    #[test]
    fn create_single_sandbox_record_outcome_does_not_contain_token() {
        let body = r#"{"records":[{"id":"recNewRecord001","fields":{"Name":"Test"},"createdTime":"2025-01-01T00:00:00.000Z"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let sentinel = "pat_sandbox_record_sentinel_0123456789";
        let client = AirtableClient::new(AirtableToken::new(sentinel), transport);
        let req = CreateSandboxRecordRequest {
            fields: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "Name".to_string(),
                    serde_json::Value::String("Test".to_string()),
                );
                m
            },
        };
        let outcome = client
            .create_single_sandbox_record("appTestBase001", "tblTest01", "SandboxTest", &req)
            .expect("should succeed");
        let serialized = serde_json::to_string(&outcome).expect("serialize");
        assert!(!serialized.contains(sentinel));
    }

    #[test]
    fn create_single_sandbox_record_outcome_does_not_contain_record_id() {
        let body = r#"{"records":[{"id":"recNewRecord001","fields":{"Name":"Test"},"createdTime":"2025-01-01T00:00:00.000Z"}]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = AirtableClient::new(AirtableToken::new("pat_sentinel"), transport);
        let req = CreateSandboxRecordRequest {
            fields: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "Name".to_string(),
                    serde_json::Value::String("Test".to_string()),
                );
                m
            },
        };
        let outcome = client
            .create_single_sandbox_record("appTestBase001", "tblTest01", "SandboxTest", &req)
            .expect("should succeed");
        let serialized = serde_json::to_string(&outcome).expect("serialize");
        // Record ID must not appear in the sanitized outcome
        assert!(!serialized.contains("recNewRecord001"));
        assert!(!serialized.contains("\"id\""));
    }

    #[test]
    fn create_single_sandbox_record_empty_records_returns_not_created() {
        let body = r#"{"records":[]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = AirtableClient::new(AirtableToken::new("pat_sentinel"), transport);
        let req = CreateSandboxRecordRequest {
            fields: std::collections::HashMap::new(),
        };
        let outcome = client
            .create_single_sandbox_record("appTestBase001", "tblTest01", "SandboxTest", &req)
            .expect("should succeed");
        assert!(!outcome.record_created);
        assert_eq!(outcome.record_count, 0);
    }

    #[test]
    fn create_single_sandbox_record_401_maps_to_invalid_token() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let client = AirtableClient::new(AirtableToken::new("pat_sentinel"), transport);
        let req = CreateSandboxRecordRequest {
            fields: std::collections::HashMap::new(),
        };
        let err = client
            .create_single_sandbox_record("appTestBase001", "tblTest01", "SandboxTest", &req)
            .unwrap_err();
        assert_eq!(err, AirtableClientError::InvalidToken);
    }

    // ── create_table tests ────────────────────────────────────────────────────

    #[test]
    fn create_table_parses_response() {
        let body = r#"{"id":"tblNewTable001","name":"Test Table","fields":[]}"#;
        let transport = MockHttpTransport::ok(body);
        let client = AirtableClient::new(AirtableToken::new("pat_sentinel"), transport);
        let req = CreateTableRequest {
            name: "Test Table".to_string(),
            description: None,
            fields: vec![CreateTableFieldSpec {
                name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
            }],
        };
        let outcome = client
            .create_table("appTestBase001", &req)
            .expect("should succeed");
        assert_eq!(outcome.table_id, "tblNewTable001");
        assert_eq!(outcome.table_name, "Test Table");
    }

    #[test]
    fn create_table_outcome_does_not_contain_token() {
        let body = r#"{"id":"tblNewTable001","name":"Test Table","fields":[]}"#;
        let transport = MockHttpTransport::ok(body);
        let sentinel = "pat_create_table_sentinel_0123456789";
        let client = AirtableClient::new(AirtableToken::new(sentinel), transport);
        let req = CreateTableRequest {
            name: "Test Table".to_string(),
            description: None,
            fields: vec![CreateTableFieldSpec {
                name: "Name".to_string(),
                field_type: "singleLineText".to_string(),
            }],
        };
        let outcome = client
            .create_table("appTestBase001", &req)
            .expect("should succeed");
        let serialized = serde_json::to_string(&outcome).expect("serialize");
        assert!(!serialized.contains(sentinel));
    }

    #[test]
    fn create_table_422_maps_to_error() {
        let transport = MockHttpTransport::with_status(422, r#"{"error":"INVALID_REQUEST_BODY"}"#);
        let client = AirtableClient::new(AirtableToken::new("pat_sentinel"), transport);
        let req = CreateTableRequest {
            name: "Bad".to_string(),
            description: None,
            fields: vec![],
        };
        let err = client.create_table("appTestBase001", &req).unwrap_err();
        // 422 is not a 2xx so map_http_error returns TransientServerError(422)
        assert!(matches!(
            err,
            AirtableClientError::TransientServerError(422)
        ));
    }
}
