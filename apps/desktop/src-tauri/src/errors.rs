use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    AuthInvalidToken,
    AuthMissingScope,
    PermissionDenied,
    NetworkUnavailable,
    RateLimited,
    PackageInvalid,
    SchemaUnsupported,
    RestorePlanInvalid,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirBridgeError {
    pub code: ErrorCode,
    pub message: String,
}

impl AirBridgeError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        AirBridgeError {
            code,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        AirBridgeError::new(ErrorCode::InternalError, message)
    }
}

impl std::fmt::Display for AirBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<AirBridgeError> for String {
    fn from(err: AirBridgeError) -> String {
        serde_json::to_string(&err).unwrap_or_else(|_| err.message.clone())
    }
}

pub type AirBridgeResult<T> = Result<T, AirBridgeError>;
