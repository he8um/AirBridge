use crate::airtable::auth::AirtableToken;
use crate::airtable::client::AirtableClient;
use crate::airtable::errors::AirtableClientError;
use crate::airtable::http::ReqwestHttpTransport;
use crate::errors::{AirBridgeError, AirBridgeResult, ErrorCode};
use crate::models::connection::{
    ConnectionCheckResult, ConnectionStatus, PermissionCheck, PermissionCheckStatus,
};

/// Validates that a raw token string meets minimum structural requirements.
/// Does not make any network call. Returns an error if the token is clearly
/// unusable (empty or whitespace-only).
fn validate_token_input(raw: &str) -> AirBridgeResult<()> {
    if raw.trim().is_empty() {
        return Err(AirBridgeError::new(
            ErrorCode::AuthInvalidToken,
            "Token must not be empty",
        ));
    }
    Ok(())
}

/// Maps an `AirtableClientError` to an `AirBridgeError`.
/// The token is never included in the returned error message.
fn map_client_error(err: AirtableClientError) -> AirBridgeError {
    match err {
        AirtableClientError::InvalidToken => {
            AirBridgeError::new(ErrorCode::AuthInvalidToken, "Invalid or expired token")
        }
        AirtableClientError::MissingScope => AirBridgeError::new(
            ErrorCode::AuthMissingScope,
            "Token is missing required scopes",
        ),
        AirtableClientError::PermissionDenied => AirBridgeError::new(
            ErrorCode::PermissionDenied,
            "Permission denied for the requested resource",
        ),
        AirtableClientError::RateLimited => {
            AirBridgeError::new(ErrorCode::RateLimited, "Rate limited by Airtable API")
        }
        AirtableClientError::NotFound => {
            AirBridgeError::new(ErrorCode::InternalError, "Resource not found")
        }
        AirtableClientError::ValidationError(msg) => {
            AirBridgeError::new(ErrorCode::InternalError, format!("Validation error: {msg}"))
        }
        AirtableClientError::TransientServerError(_)
        | AirtableClientError::MalformedResponse(_) => {
            AirBridgeError::new(ErrorCode::NetworkUnavailable, "Network or server error")
        }
    }
}

/// Builds the `ConnectionCheckResult` for a successful check.
///
/// Read permissions are marked `Passed` because list-bases succeeded.
/// Write permissions are marked `Unknown` — they are not verified destructively.
fn build_success_result(base_count: usize) -> ConnectionCheckResult {
    let detail = if base_count == 1 {
        Some("1 base accessible".to_string())
    } else {
        Some(format!("{base_count} bases accessible"))
    };

    ConnectionCheckResult {
        connection_id: "conn-live".to_string(),
        status: ConnectionStatus::Connected,
        permissions: vec![
            PermissionCheck {
                key: "schema.bases:read".to_string(),
                label: "Schema read".to_string(),
                status: PermissionCheckStatus::Passed,
                detail,
            },
            PermissionCheck {
                key: "data.records:read".to_string(),
                label: "Records read".to_string(),
                status: PermissionCheckStatus::Passed,
                detail: None,
            },
            PermissionCheck {
                key: "schema.bases:write".to_string(),
                label: "Schema write".to_string(),
                status: PermissionCheckStatus::Unknown,
                detail: Some("Write access not verified".to_string()),
            },
            PermissionCheck {
                key: "data.records:write".to_string(),
                label: "Records write".to_string(),
                status: PermissionCheckStatus::Unknown,
                detail: Some("Write access not verified".to_string()),
            },
        ],
    }
}

/// Builds the `ConnectionCheckResult` for a failed check.
fn build_failure_result(err: AirtableClientError) -> (AirBridgeError, ConnectionCheckResult) {
    let bridge_err = map_client_error(err.clone());
    let detail = match &err {
        AirtableClientError::InvalidToken => Some("Invalid or expired token".to_string()),
        AirtableClientError::MissingScope => Some("Token is missing required scopes".to_string()),
        AirtableClientError::PermissionDenied => Some("Permission denied".to_string()),
        AirtableClientError::RateLimited => Some("Rate limited — try again later".to_string()),
        _ => Some("Network or server error".to_string()),
    };

    let result = ConnectionCheckResult {
        connection_id: "conn-live".to_string(),
        status: ConnectionStatus::Failed,
        permissions: vec![
            PermissionCheck {
                key: "schema.bases:read".to_string(),
                label: "Schema read".to_string(),
                status: PermissionCheckStatus::Failed,
                detail,
            },
            PermissionCheck {
                key: "data.records:read".to_string(),
                label: "Records read".to_string(),
                status: PermissionCheckStatus::Unknown,
                detail: None,
            },
            PermissionCheck {
                key: "schema.bases:write".to_string(),
                label: "Schema write".to_string(),
                status: PermissionCheckStatus::Unknown,
                detail: Some("Write access not verified".to_string()),
            },
            PermissionCheck {
                key: "data.records:write".to_string(),
                label: "Records write".to_string(),
                status: PermissionCheckStatus::Unknown,
                detail: Some("Write access not verified".to_string()),
            },
        ],
    };

    (bridge_err, result)
}

#[tauri::command]
pub fn check_connection(token: String) -> AirBridgeResult<ConnectionCheckResult> {
    validate_token_input(&token)?;

    let airtable_token = AirtableToken::new(&token);
    // Drop the raw token string immediately — the AirtableToken wrapper owns it now.
    drop(token);

    let transport = ReqwestHttpTransport::new().map_err(|_| {
        AirBridgeError::new(
            ErrorCode::NetworkUnavailable,
            "Failed to initialize HTTP client",
        )
    })?;

    let client = AirtableClient::new(airtable_token, transport);

    match client.check_connection_for_token() {
        Ok(outcome) => Ok(build_success_result(outcome.accessible_bases.len())),
        Err(err) => {
            let (bridge_err, _result) = build_failure_result(err);
            Err(bridge_err)
        }
    }
}

// ── Unit-testable helpers ─────────────────────────────────────────────────

/// Internal helper for testing error mapping without constructing a real transport.
#[cfg(test)]
pub fn check_connection_with_outcome(
    raw_token: &str,
    outcome: Result<crate::airtable::models::ConnectionCheckOutcome, AirtableClientError>,
) -> AirBridgeResult<ConnectionCheckResult> {
    validate_token_input(raw_token)?;
    match outcome {
        Ok(o) => Ok(build_success_result(o.accessible_bases.len())),
        Err(err) => {
            let (bridge_err, _) = build_failure_result(err);
            Err(bridge_err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::{AccessibleBase, AirtableBaseId, ConnectionCheckOutcome};

    const SENTINEL: &str = "pat_example_connection_sentinel_abcdef01";

    fn ok_outcome(n: usize) -> ConnectionCheckOutcome {
        ConnectionCheckOutcome {
            accessible_bases: (0..n)
                .map(|i| AccessibleBase {
                    id: AirtableBaseId(format!("appExampleBase{i:02}")),
                    name: format!("Example Base {i}"),
                })
                .collect(),
        }
    }

    #[test]
    fn empty_token_returns_error() {
        let result = check_connection_with_outcome("", Ok(ok_outcome(1)));
        assert!(result.is_err());
    }

    #[test]
    fn whitespace_token_returns_error() {
        let result = check_connection_with_outcome("   ", Ok(ok_outcome(1)));
        assert!(result.is_err());
    }

    #[test]
    fn success_maps_to_connected_status() {
        let result =
            check_connection_with_outcome(SENTINEL, Ok(ok_outcome(2))).expect("should succeed");
        assert!(matches!(result.status, ConnectionStatus::Connected));
    }

    #[test]
    fn success_result_does_not_contain_token() {
        let result =
            check_connection_with_outcome(SENTINEL, Ok(ok_outcome(1))).expect("should succeed");
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn success_marks_read_permissions_passed() {
        let result =
            check_connection_with_outcome(SENTINEL, Ok(ok_outcome(1))).expect("should succeed");
        let schema_read = result
            .permissions
            .iter()
            .find(|p| p.key == "schema.bases:read")
            .expect("schema.bases:read permission missing");
        assert!(matches!(schema_read.status, PermissionCheckStatus::Passed));

        let records_read = result
            .permissions
            .iter()
            .find(|p| p.key == "data.records:read")
            .expect("data.records:read permission missing");
        assert!(matches!(records_read.status, PermissionCheckStatus::Passed));
    }

    #[test]
    fn success_marks_write_permissions_unknown() {
        let result =
            check_connection_with_outcome(SENTINEL, Ok(ok_outcome(1))).expect("should succeed");
        let schema_write = result
            .permissions
            .iter()
            .find(|p| p.key == "schema.bases:write")
            .expect("schema.bases:write missing");
        assert!(matches!(
            schema_write.status,
            PermissionCheckStatus::Unknown
        ));

        let records_write = result
            .permissions
            .iter()
            .find(|p| p.key == "data.records:write")
            .expect("data.records:write missing");
        assert!(matches!(
            records_write.status,
            PermissionCheckStatus::Unknown
        ));
    }

    #[test]
    fn invalid_token_error_maps_correctly() {
        let result =
            check_connection_with_outcome(SENTINEL, Err(AirtableClientError::InvalidToken));
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::AuthInvalidToken));
        assert!(!err.message.contains(SENTINEL));
    }

    #[test]
    fn missing_scope_error_maps_correctly() {
        let result =
            check_connection_with_outcome(SENTINEL, Err(AirtableClientError::MissingScope));
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::AuthMissingScope));
    }

    #[test]
    fn rate_limited_error_maps_correctly() {
        let result = check_connection_with_outcome(SENTINEL, Err(AirtableClientError::RateLimited));
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::RateLimited));
    }

    #[test]
    fn error_message_never_contains_token() {
        let errors = vec![
            AirtableClientError::InvalidToken,
            AirtableClientError::MissingScope,
            AirtableClientError::PermissionDenied,
            AirtableClientError::RateLimited,
            AirtableClientError::TransientServerError(500),
            AirtableClientError::MalformedResponse("bad json".to_string()),
        ];
        for err in errors {
            let result = check_connection_with_outcome(SENTINEL, Err(err));
            let err_msg = result.unwrap_err().message;
            assert!(
                !err_msg.contains(SENTINEL),
                "error message contained token: {err_msg}"
            );
        }
    }
}
