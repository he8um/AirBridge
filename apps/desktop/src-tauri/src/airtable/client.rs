use super::auth::AirtableToken;
use super::endpoints;
use super::errors::{map_http_error, AirtableClientError};
use super::http::{HttpRequest, HttpTransport};
use super::models::{
    AirtableListRecordsResponse, AirtableRecordFields, AirtableRecordUpdate, AirtableTable,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::auth::AirtableToken;
    use crate::airtable::http::MockHttpTransport;
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
}
