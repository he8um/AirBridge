use serde::{Deserialize, Serialize};

/// The kind of credential being stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CredentialKind {
    AirtablePersonalAccessToken,
}

impl CredentialKind {
    /// Returns the safe non-secret account key used as the keychain account name.
    pub fn account_key(&self) -> &'static str {
        match self {
            Self::AirtablePersonalAccessToken => "airtable_personal_access_token",
        }
    }

    /// Returns the service name used in the keychain entry.
    pub fn service_name() -> &'static str {
        "AirBridge"
    }
}

/// Whether the OS keychain is available for use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CredentialStorageAvailability {
    Available,
    Unavailable,
}

/// The current storage status of a credential.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CredentialStorageStatus {
    Saved,
    NotSaved,
    Unavailable,
    Failed,
}

/// Request to check the storage status of a credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatusRequest {
    pub kind: CredentialKind,
}

/// Result of a credential status check.
/// Never contains the credential value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatusResult {
    pub kind: CredentialKind,
    pub status: CredentialStorageStatus,
    pub availability: CredentialStorageAvailability,
    pub has_saved_token: bool,
    /// Safe display string — never the token value.
    pub display: String,
}

/// Request to save a credential to the keychain.
/// The token field is accepted here but must never appear in results or logs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSaveRequest {
    pub kind: CredentialKind,
    /// The raw token value — accepted from the frontend, never returned.
    pub token: String,
}

/// Result of saving a credential.
/// Never contains the token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSaveResult {
    pub kind: CredentialKind,
    pub success: bool,
    pub has_saved_token: bool,
    /// Safe display string.
    pub display: String,
    pub error_message: Option<String>,
}

/// Request to remove a saved credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialRemoveRequest {
    pub kind: CredentialKind,
}

/// Result of removing a credential.
/// Never contains the token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialRemoveResult {
    pub kind: CredentialKind,
    pub success: bool,
    pub has_saved_token: bool,
    /// Safe display string.
    pub display: String,
    pub error_message: Option<String>,
}

/// A safe redacted summary of a stored credential for display purposes.
/// Never contains the token value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactedCredentialSummary {
    pub kind: CredentialKind,
    pub has_saved_token: bool,
    pub display: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "pat_example_sentinel_0123456789abcdef";

    #[test]
    fn save_result_serialization_excludes_token() {
        let result = CredentialSaveResult {
            kind: CredentialKind::AirtablePersonalAccessToken,
            success: true,
            has_saved_token: true,
            display: "Saved token present".to_string(),
            error_message: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
        assert!(!json.contains("token_value"));
    }

    #[test]
    fn status_result_serialization_excludes_token() {
        let result = CredentialStatusResult {
            kind: CredentialKind::AirtablePersonalAccessToken,
            status: CredentialStorageStatus::Saved,
            availability: CredentialStorageAvailability::Available,
            has_saved_token: true,
            display: "Saved token present".to_string(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn remove_result_serialization_excludes_token() {
        let result = CredentialRemoveResult {
            kind: CredentialKind::AirtablePersonalAccessToken,
            success: true,
            has_saved_token: false,
            display: "No saved token".to_string(),
            error_message: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn credential_kind_account_key_is_not_secret() {
        let kind = CredentialKind::AirtablePersonalAccessToken;
        // account_key is used as the keychain account name — must be safe to expose
        let key = kind.account_key();
        assert!(!key.contains(SENTINEL));
        assert_eq!(key, "airtable_personal_access_token");
    }

    #[test]
    fn service_name_is_product_name() {
        assert_eq!(CredentialKind::service_name(), "AirBridge");
    }
}
