use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CredentialStorageError {
    #[error("Keychain is not available on this system")]
    KeychainUnavailable,

    #[error("Failed to save credential to keychain")]
    SaveFailed,

    #[error("Failed to remove credential from keychain")]
    RemoveFailed,

    #[error("Failed to check credential status")]
    StatusCheckFailed,

    #[error("Credential not found")]
    NotFound,

    #[error("Invalid credential kind")]
    InvalidKind,
}

impl CredentialStorageError {
    /// Returns a safe user-facing message that never includes a secret value.
    pub fn safe_message(&self) -> &'static str {
        match self {
            Self::KeychainUnavailable => "OS keychain is not available on this system.",
            Self::SaveFailed => "Failed to save credential to the keychain.",
            Self::RemoveFailed => "Failed to remove credential from the keychain.",
            Self::StatusCheckFailed => "Failed to check credential status.",
            Self::NotFound => "No saved credential found.",
            Self::InvalidKind => "Unknown credential type.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "pat_example_sentinel_0123456789abcdef";

    #[test]
    fn safe_message_never_contains_token() {
        let errors = vec![
            CredentialStorageError::KeychainUnavailable,
            CredentialStorageError::SaveFailed,
            CredentialStorageError::RemoveFailed,
            CredentialStorageError::StatusCheckFailed,
            CredentialStorageError::NotFound,
            CredentialStorageError::InvalidKind,
        ];
        for err in errors {
            assert!(!err.safe_message().contains(SENTINEL));
        }
    }

    #[test]
    fn error_display_never_contains_token() {
        let err = CredentialStorageError::SaveFailed;
        let display = format!("{err}");
        assert!(!display.contains(SENTINEL));
    }

    #[test]
    fn error_serializes_without_token() {
        let err = CredentialStorageError::SaveFailed;
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }
}
