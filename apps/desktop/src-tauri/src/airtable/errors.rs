use thiserror::Error;

/// Errors produced by the Airtable API client.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AirtableClientError {
    #[error("invalid or expired token")]
    InvalidToken,

    #[error("token is missing one or more required scopes")]
    MissingScope,

    #[error("permission denied for the requested resource")]
    PermissionDenied,

    #[error("resource not found")]
    NotFound,

    #[error("rate limited by the Airtable API (HTTP 429)")]
    RateLimited,

    #[error("request validation failed: {0}")]
    ValidationError(String),

    #[error("transient server error (HTTP {0})")]
    TransientServerError(u16),

    #[error("malformed response from Airtable API: {0}")]
    MalformedResponse(String),
}

/// Map an HTTP status code and optional body to a typed client error.
pub fn map_http_error(status: u16, body: &str) -> AirtableClientError {
    match status {
        401 => AirtableClientError::InvalidToken,
        403 => {
            if body.contains("scope") {
                AirtableClientError::MissingScope
            } else {
                AirtableClientError::PermissionDenied
            }
        }
        404 => AirtableClientError::NotFound,
        429 => AirtableClientError::RateLimited,
        400 => AirtableClientError::ValidationError(body.to_string()),
        500..=599 => AirtableClientError::TransientServerError(status),
        _ => AirtableClientError::TransientServerError(status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_401_to_invalid_token() {
        assert_eq!(map_http_error(401, ""), AirtableClientError::InvalidToken);
    }

    #[test]
    fn maps_403_with_scope_to_missing_scope() {
        assert_eq!(
            map_http_error(403, "{\"error\":\"missing required scope\"}"),
            AirtableClientError::MissingScope
        );
    }

    #[test]
    fn maps_403_without_scope_to_permission_denied() {
        assert_eq!(
            map_http_error(403, "{\"error\":\"forbidden\"}"),
            AirtableClientError::PermissionDenied
        );
    }

    #[test]
    fn maps_404_to_not_found() {
        assert_eq!(map_http_error(404, ""), AirtableClientError::NotFound);
    }

    #[test]
    fn maps_429_to_rate_limited() {
        assert_eq!(map_http_error(429, ""), AirtableClientError::RateLimited);
    }

    #[test]
    fn maps_500_to_transient_server_error() {
        assert_eq!(
            map_http_error(500, ""),
            AirtableClientError::TransientServerError(500)
        );
    }

    #[test]
    fn maps_503_to_transient_server_error() {
        assert_eq!(
            map_http_error(503, ""),
            AirtableClientError::TransientServerError(503)
        );
    }

    #[test]
    fn maps_400_to_validation_error() {
        let body = "{\"error\":\"INVALID_RECORDS\"}";
        match map_http_error(400, body) {
            AirtableClientError::ValidationError(msg) => assert!(msg.contains("INVALID_RECORDS")),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn error_display_does_not_expose_internals() {
        let err = AirtableClientError::InvalidToken;
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(!msg.contains("Bearer"));
    }
}
