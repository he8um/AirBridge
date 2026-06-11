use std::collections::HashMap;

/// HTTP methods used by the Airtable client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Patch,
    Delete,
}

/// A minimal outbound HTTP request.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        HttpRequest {
            method: HttpMethod::Get,
            url: url.into(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: None,
        }
    }

    pub fn post(url: impl Into<String>, body: impl Into<String>) -> Self {
        HttpRequest {
            method: HttpMethod::Post,
            url: url.into(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: Some(body.into()),
        }
    }

    pub fn patch(url: impl Into<String>, body: impl Into<String>) -> Self {
        HttpRequest {
            method: HttpMethod::Patch,
            url: url.into(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: Some(body.into()),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }
}

/// Inbound HTTP response from the transport layer.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

impl HttpResponse {
    pub fn ok(body: impl Into<String>) -> Self {
        HttpResponse {
            status: 200,
            body: body.into(),
        }
    }

    pub fn with_status(status: u16, body: impl Into<String>) -> Self {
        HttpResponse {
            status,
            body: body.into(),
        }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Transport abstraction — any type that can send an `HttpRequest` and return
/// a response. Implemented by the real reqwest transport (future) and by
/// `MockHttpTransport` in tests.
pub trait HttpTransport: Send + Sync {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, String>;
}

// ── Mock transport ─────────────────────────────────────────────────────────

/// Deterministic mock that returns a pre-configured response.
pub struct MockHttpTransport {
    pub response: HttpResponse,
}

impl MockHttpTransport {
    pub fn ok(body: impl Into<String>) -> Self {
        MockHttpTransport {
            response: HttpResponse::ok(body),
        }
    }

    pub fn with_status(status: u16, body: impl Into<String>) -> Self {
        MockHttpTransport {
            response: HttpResponse::with_status(status, body),
        }
    }
}

impl HttpTransport for MockHttpTransport {
    fn send(&self, _request: HttpRequest) -> Result<HttpResponse, String> {
        Ok(self.response.clone())
    }
}

// ── Reqwest (live) transport ───────────────────────────────────────────────

/// HTTP transport backed by a blocking reqwest client.
///
/// Used in production Tauri commands. Never logs request headers or token.
pub struct ReqwestHttpTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestHttpTransport {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;
        Ok(ReqwestHttpTransport { client })
    }
}

impl Default for ReqwestHttpTransport {
    fn default() -> Self {
        ReqwestHttpTransport::new().expect("reqwest client construction failed")
    }
}

impl HttpTransport for ReqwestHttpTransport {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, String> {
        let method = match request.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Delete => reqwest::Method::DELETE,
        };

        let mut builder = self.client.request(method, &request.url);

        for (k, v) in &request.headers {
            builder = builder.header(k, v);
        }

        if !request.query.is_empty() {
            let pairs: Vec<(&str, &str)> = request
                .query
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            builder = builder.query(&pairs);
        }

        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        let resp = builder.send().map_err(|e| format!("network error: {e}"))?;

        let status = resp.status().as_u16();
        let body = resp
            .text()
            .map_err(|e| format!("response read error: {e}"))?;

        Ok(HttpResponse { status, body })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_returns_configured_200() {
        let transport = MockHttpTransport::ok(r#"{"records":[]}"#);
        let req = HttpRequest::get("https://api.airtable.com/v0/appTest/tblTest");
        let resp = transport.send(req).expect("send failed");
        assert_eq!(resp.status, 200);
        assert!(resp.body.contains("records"));
    }

    #[test]
    fn mock_returns_configured_401() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let req = HttpRequest::get("https://api.airtable.com/v0/appTest/tblTest");
        let resp = transport.send(req).expect("send failed");
        assert_eq!(resp.status, 401);
    }

    #[test]
    fn mock_returns_configured_429() {
        let transport = MockHttpTransport::with_status(429, r#"{"error":"RATE_LIMITED"}"#);
        let req = HttpRequest::get("https://any");
        let resp = transport.send(req).expect("send failed");
        assert_eq!(resp.status, 429);
    }

    #[test]
    fn mock_returns_configured_500() {
        let transport = MockHttpTransport::with_status(500, "Internal Server Error");
        let req = HttpRequest::get("https://any");
        let resp = transport.send(req).expect("send failed");
        assert_eq!(resp.status, 500);
        assert!(!resp.is_success());
    }

    #[test]
    fn http_response_is_success_for_200() {
        assert!(HttpResponse::ok("").is_success());
    }

    #[test]
    fn http_response_is_not_success_for_400() {
        assert!(!HttpResponse::with_status(400, "").is_success());
    }

    #[test]
    fn request_builder_sets_headers_and_query() {
        let req = HttpRequest::get("https://api.airtable.com/v0/app/tbl")
            .with_header("Authorization", "Bearer tok")
            .with_query("pageSize", "50");
        assert_eq!(
            req.headers.get("Authorization").map(|s| s.as_str()),
            Some("Bearer tok")
        );
        assert_eq!(req.query.get("pageSize").map(|s| s.as_str()), Some("50"));
    }
}
