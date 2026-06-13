use crate::credentials::errors::CredentialStorageError;
use crate::credentials::models::{CredentialKind, CredentialStorageAvailability};
use crate::credentials::store::CredentialStore;

/// OS keychain credential store using the `keyring` crate.
///
/// On macOS, uses the system Keychain. On Windows, uses the Windows Credential Store.
/// On Linux, uses the system secret service (e.g., GNOME Keyring or KWallet).
///
/// The token is passed to the keyring API and never stored in this struct.
/// Errors are mapped to safe variants that do not include the token value.
pub struct OsKeychainStore;

impl OsKeychainStore {
    pub fn new() -> Self {
        OsKeychainStore
    }

    fn entry(kind: &CredentialKind) -> Result<keyring::Entry, CredentialStorageError> {
        keyring::Entry::new(CredentialKind::service_name(), kind.account_key())
            .map_err(|_| CredentialStorageError::KeychainUnavailable)
    }
}

impl CredentialStore for OsKeychainStore {
    fn availability(&self) -> CredentialStorageAvailability {
        // Probe availability by attempting to create an entry.
        // If the keyring backend is absent (e.g., headless Linux), this returns Unavailable.
        match keyring::Entry::new(CredentialKind::service_name(), "availability_probe") {
            Ok(_) => CredentialStorageAvailability::Available,
            Err(_) => CredentialStorageAvailability::Unavailable,
        }
    }

    fn save(&self, kind: &CredentialKind, secret: &str) -> Result<(), CredentialStorageError> {
        let entry = Self::entry(kind)?;
        // The secret is passed directly to the keychain; it is not stored in self.
        entry
            .set_secret(secret.as_bytes())
            .map_err(|_| CredentialStorageError::SaveFailed)
    }

    fn exists(&self, kind: &CredentialKind) -> Result<bool, CredentialStorageError> {
        let entry = Self::entry(kind)?;
        match entry.get_secret() {
            Ok(bytes) => Ok(!bytes.is_empty()),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(_) => Err(CredentialStorageError::StatusCheckFailed),
        }
    }

    fn remove(&self, kind: &CredentialKind) -> Result<(), CredentialStorageError> {
        let entry = Self::entry(kind)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // already gone — not an error
            Err(_) => Err(CredentialStorageError::RemoveFailed),
        }
    }
}

/// Returns the global OS keychain store. Used by commands.
pub fn os_keychain_store() -> OsKeychainStore {
    OsKeychainStore::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "pat_example_sentinel_0123456789abcdefghijklmnopqrstuvwxyz01234";

    #[test]
    fn keychain_store_availability_is_a_valid_variant() {
        let store = OsKeychainStore::new();
        let avail = store.availability();
        // Just verify it returns a valid variant without panicking
        assert!(matches!(
            avail,
            CredentialStorageAvailability::Available | CredentialStorageAvailability::Unavailable
        ));
    }

    #[test]
    fn keychain_error_does_not_expose_token() {
        // Simulate what would happen if save fails — error must not contain token
        let err = CredentialStorageError::SaveFailed;
        let msg = err.safe_message();
        assert!(!msg.contains(SENTINEL));
        let display = format!("{err}");
        assert!(!display.contains(SENTINEL));
    }

    #[test]
    fn keychain_error_variants_are_safe() {
        let errors = vec![
            CredentialStorageError::KeychainUnavailable,
            CredentialStorageError::SaveFailed,
            CredentialStorageError::RemoveFailed,
            CredentialStorageError::StatusCheckFailed,
            CredentialStorageError::NotFound,
        ];
        for err in errors {
            assert!(!err.safe_message().contains(SENTINEL));
            let json = serde_json::to_string(&err).expect("serialize");
            assert!(!json.contains(SENTINEL));
        }
    }
}
