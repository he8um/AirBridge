use crate::credentials::errors::CredentialStorageError;
use crate::credentials::models::{CredentialKind, CredentialStorageAvailability};

/// Abstraction over credential storage backends.
pub trait CredentialStore: Send + Sync {
    /// Returns whether this store's backend is available.
    fn availability(&self) -> CredentialStorageAvailability;

    /// Saves the secret for the given kind. The secret is consumed and not stored in self.
    fn save(&self, kind: &CredentialKind, secret: &str) -> Result<(), CredentialStorageError>;

    /// Returns true if a credential for the given kind exists.
    fn exists(&self, kind: &CredentialKind) -> Result<bool, CredentialStorageError>;

    /// Removes the credential for the given kind. Returns Ok if removed or not present.
    fn remove(&self, kind: &CredentialKind) -> Result<(), CredentialStorageError>;
}

/// In-memory credential store for testing only.
/// Stores secrets in memory (not persisted to disk or keychain).
/// Not for production use.
#[cfg(test)]
pub struct InMemoryCredentialStore {
    inner: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

#[cfg(test)]
impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(test)]
impl CredentialStore for InMemoryCredentialStore {
    fn availability(&self) -> CredentialStorageAvailability {
        CredentialStorageAvailability::Available
    }

    fn save(&self, kind: &CredentialKind, secret: &str) -> Result<(), CredentialStorageError> {
        let mut map = self.inner.lock().unwrap();
        map.insert(kind.account_key().to_string(), secret.to_string());
        Ok(())
    }

    fn exists(&self, kind: &CredentialKind) -> Result<bool, CredentialStorageError> {
        let map = self.inner.lock().unwrap();
        Ok(map.contains_key(kind.account_key()))
    }

    fn remove(&self, kind: &CredentialKind) -> Result<(), CredentialStorageError> {
        let mut map = self.inner.lock().unwrap();
        map.remove(kind.account_key());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind() -> CredentialKind {
        CredentialKind::AirtablePersonalAccessToken
    }

    #[test]
    fn in_memory_store_is_available() {
        let store = InMemoryCredentialStore::new();
        assert_eq!(
            store.availability(),
            CredentialStorageAvailability::Available
        );
    }

    #[test]
    fn in_memory_store_save_and_exists() {
        let store = InMemoryCredentialStore::new();
        assert_eq!(store.exists(&kind()), Ok(false));
        store.save(&kind(), "some_secret").expect("save failed");
        assert_eq!(store.exists(&kind()), Ok(true));
    }

    #[test]
    fn in_memory_store_remove() {
        let store = InMemoryCredentialStore::new();
        store.save(&kind(), "some_secret").expect("save failed");
        store.remove(&kind()).expect("remove failed");
        assert_eq!(store.exists(&kind()), Ok(false));
    }

    #[test]
    fn in_memory_store_remove_nonexistent_ok() {
        let store = InMemoryCredentialStore::new();
        // remove on empty store should not error
        store.remove(&kind()).expect("remove should be ok");
    }

    #[test]
    fn in_memory_store_does_not_expose_secret_via_exists() {
        let store = InMemoryCredentialStore::new();
        let secret = "pat_example_sentinel_abcdef0123456789abcdef0123456789";
        store.save(&kind(), secret).expect("save");
        // exists() only returns bool, never the secret
        let result = store.exists(&kind()).expect("exists");
        // Confirm it's true but we can't get the value out
        assert!(result);
    }
}
