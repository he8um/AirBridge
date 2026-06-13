use crate::credentials::{
    ensure_no_token_in_message, os_keychain_store, CredentialRemoveRequest, CredentialRemoveResult,
    CredentialSaveRequest, CredentialSaveResult, CredentialStatusRequest, CredentialStatusResult,
    CredentialStorageAvailability, CredentialStorageStatus, CredentialStore,
};

/// Returns the storage status for a credential without returning the token.
#[tauri::command]
pub fn get_credential_storage_status(request: CredentialStatusRequest) -> CredentialStatusResult {
    let store = os_keychain_store();
    let availability = store.availability();

    if availability == CredentialStorageAvailability::Unavailable {
        return CredentialStatusResult {
            kind: request.kind,
            status: CredentialStorageStatus::Unavailable,
            availability: CredentialStorageAvailability::Unavailable,
            has_saved_token: false,
            display: "OS keychain is not available on this system.".to_string(),
        };
    }

    match store.exists(&request.kind) {
        Ok(true) => CredentialStatusResult {
            kind: request.kind,
            status: CredentialStorageStatus::Saved,
            availability: CredentialStorageAvailability::Available,
            has_saved_token: true,
            display: "Saved token present".to_string(),
        },
        Ok(false) => CredentialStatusResult {
            kind: request.kind,
            status: CredentialStorageStatus::NotSaved,
            availability: CredentialStorageAvailability::Available,
            has_saved_token: false,
            display: "No saved token".to_string(),
        },
        Err(err) => CredentialStatusResult {
            kind: request.kind,
            status: CredentialStorageStatus::Failed,
            availability: CredentialStorageAvailability::Available,
            has_saved_token: false,
            display: ensure_no_token_in_message(err.safe_message()).to_string(),
        },
    }
}

/// Saves an Airtable token to the OS keychain.
/// The token is accepted from the frontend and forwarded to the keychain only.
/// It is never returned in the result.
#[tauri::command]
pub fn save_airtable_token_to_keychain(request: CredentialSaveRequest) -> CredentialSaveResult {
    // Validate token is not empty
    if request.token.trim().is_empty() {
        return CredentialSaveResult {
            kind: request.kind,
            success: false,
            has_saved_token: false,
            display: "Token must not be empty.".to_string(),
            error_message: Some("Token must not be empty.".to_string()),
        };
    }

    let store = os_keychain_store();
    let availability = store.availability();

    if availability == CredentialStorageAvailability::Unavailable {
        return CredentialSaveResult {
            kind: request.kind,
            success: false,
            has_saved_token: false,
            display: "OS keychain is not available on this system.".to_string(),
            error_message: Some("OS keychain is not available on this system.".to_string()),
        };
    }

    match store.save(&request.kind, &request.token) {
        Ok(()) => CredentialSaveResult {
            kind: request.kind,
            success: true,
            has_saved_token: true,
            display: "Saved token present".to_string(),
            error_message: None,
        },
        Err(err) => CredentialSaveResult {
            kind: request.kind,
            success: false,
            has_saved_token: false,
            display: ensure_no_token_in_message(err.safe_message()).to_string(),
            error_message: Some(ensure_no_token_in_message(err.safe_message()).to_string()),
        },
    }
}

/// Removes a saved token from the OS keychain.
/// Never returns the token.
#[tauri::command]
pub fn remove_airtable_token_from_keychain(
    request: CredentialRemoveRequest,
) -> CredentialRemoveResult {
    let store = os_keychain_store();
    let availability = store.availability();

    if availability == CredentialStorageAvailability::Unavailable {
        return CredentialRemoveResult {
            kind: request.kind,
            success: false,
            has_saved_token: false,
            display: "OS keychain is not available on this system.".to_string(),
            error_message: Some("OS keychain is not available on this system.".to_string()),
        };
    }

    match store.remove(&request.kind) {
        Ok(()) => CredentialRemoveResult {
            kind: request.kind,
            success: true,
            has_saved_token: false,
            display: "No saved token".to_string(),
            error_message: None,
        },
        Err(err) => CredentialRemoveResult {
            kind: request.kind,
            success: false,
            has_saved_token: false,
            display: ensure_no_token_in_message(err.safe_message()).to_string(),
            error_message: Some(ensure_no_token_in_message(err.safe_message()).to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::CredentialKind;

    const SENTINEL: &str = "pat_example_sentinel_0123456789abcdefghijklmnopqrstuvwxyz01234";

    fn status_request() -> CredentialStatusRequest {
        CredentialStatusRequest {
            kind: CredentialKind::AirtablePersonalAccessToken,
        }
    }

    fn remove_request() -> CredentialRemoveRequest {
        CredentialRemoveRequest {
            kind: CredentialKind::AirtablePersonalAccessToken,
        }
    }

    #[test]
    fn status_command_does_not_return_token() {
        let result = get_credential_storage_status(status_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn status_result_has_no_token_field() {
        let result = get_credential_storage_status(status_request());
        // The result type has no token field structurally — serialize and verify
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn save_empty_token_returns_error_without_token() {
        let request = CredentialSaveRequest {
            kind: CredentialKind::AirtablePersonalAccessToken,
            token: "".to_string(),
        };
        let result = save_airtable_token_to_keychain(request);
        assert!(!result.success);
        assert!(!result.has_saved_token);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn save_result_does_not_return_token() {
        let request = CredentialSaveRequest {
            kind: CredentialKind::AirtablePersonalAccessToken,
            token: SENTINEL.to_string(),
        };
        let result = save_airtable_token_to_keychain(request);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn remove_result_does_not_return_token() {
        let result = remove_airtable_token_from_keychain(remove_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn remove_result_has_no_token_field() {
        let result = remove_airtable_token_from_keychain(remove_request());
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"secret\""));
    }

    #[test]
    fn status_display_is_safe() {
        let result = get_credential_storage_status(status_request());
        assert!(!result.display.contains(SENTINEL));
    }

    #[test]
    fn credential_storage_does_not_affect_restore_write_gate() {
        use crate::restore::write_gate::evaluate_write_gate;
        use crate::restore::write_result::RestoreWriteEngineStatus;
        // Saving/removing credentials must not change the write gate
        let gate_before = evaluate_write_gate();
        let _status = get_credential_storage_status(status_request());
        let gate_after = evaluate_write_gate();
        // Gate is always disabled regardless
        assert!(matches!(
            gate_before.status,
            RestoreWriteEngineStatus::Disabled
        ));
        assert!(matches!(
            gate_after.status,
            RestoreWriteEngineStatus::Disabled
        ));
    }
}
