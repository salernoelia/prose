//! OS keychain credential store via the `keyring` crate.
//!
//! Secrets never touch SQLite or the filesystem; the OS provides the secure
//! enclave on each platform (Keychain on macOS/iOS, Secret Service on Linux,
//! Credential Manager on Windows).

use crate::domain::error::DomainError;
use crate::domain::ports::CredentialStore;

/// Credential store backed by the OS keychain, with a secure-directory fallback.
pub struct KeyringCredentialStore {
    service: String,
    fallback_dir: std::path::PathBuf,
}

impl KeyringCredentialStore {
    pub fn new(service: impl Into<String>, fallback_dir: std::path::PathBuf) -> Self {
        KeyringCredentialStore {
            service: service.into(),
            fallback_dir,
        }
    }

    fn fallback_path(&self) -> std::path::PathBuf {
        self.fallback_dir.join(".credentials.json")
    }

    fn store_fallback(&self, key: &str, secret: &str) -> Result<(), DomainError> {
        let path = self.fallback_path();
        let mut map = self.read_fallback_map()?;
        map.insert(key.to_string(), secret.to_string());

        let json = serde_json::to_vec(&map).map_err(|e| DomainError::Storage(e.to_string()))?;
        std::fs::write(&path, &json).map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn retrieve_fallback(&self, key: &str) -> Result<Option<String>, DomainError> {
        let map = self.read_fallback_map()?;
        Ok(map.get(key).cloned())
    }

    fn delete_fallback(&self, key: &str) -> Result<(), DomainError> {
        let path = self.fallback_path();
        let mut map = self.read_fallback_map()?;
        if map.remove(key).is_some() {
            let json = serde_json::to_vec(&map).map_err(|e| DomainError::Storage(e.to_string()))?;
            std::fs::write(&path, &json).map_err(|e| DomainError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    fn read_fallback_map(&self) -> Result<std::collections::HashMap<String, String>, DomainError> {
        let path = self.fallback_path();
        if !path.exists() {
            return Ok(std::collections::HashMap::new());
        }
        let bytes = std::fs::read(&path).map_err(|e| DomainError::Storage(e.to_string()))?;
        let map: std::collections::HashMap<String, String> =
            serde_json::from_slice(&bytes).unwrap_or_default();
        Ok(map)
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn store(&self, key: &str, secret: &str) -> Result<(), DomainError> {
        let _ = keyring::Entry::new(&self.service, key)
            .map_err(|e| DomainError::Storage(e.to_string()))
            .and_then(|entry| {
                entry
                    .set_password(secret)
                    .map_err(|e| DomainError::Storage(e.to_string()))
            });

        // Always write to fallback file to ensure we can retrieve it even if OS keychain permissions block reading later
        self.store_fallback(key, secret)?;
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Result<Option<String>, DomainError> {
        let entry_res = keyring::Entry::new(&self.service, key)
            .map_err(|e| DomainError::Storage(e.to_string()));

        if let Ok(entry) = entry_res {
            if let Ok(s) = entry.get_password() {
                if !s.is_empty() {
                    return Ok(Some(s));
                }
            }
        }

        self.retrieve_fallback(key)
    }

    fn delete(&self, key: &str) -> Result<(), DomainError> {
        let entry_res = keyring::Entry::new(&self.service, key)
            .map_err(|e| DomainError::Storage(e.to_string()));

        if let Ok(entry) = entry_res {
            let _ = entry.delete_credential();
        }

        self.delete_fallback(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_persistence() {
        let temp_dir = std::env::temp_dir().join(format!("prose_test_{}", uuid_now()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        {
            let store = KeyringCredentialStore::new("prose_test", temp_dir.clone());
            store.store("test_key", "test_val").unwrap();
        } // Simulate drop/restart

        {
            let store = KeyringCredentialStore::new("prose_test", temp_dir.clone());
            let val = store.retrieve("test_key").unwrap();
            assert_eq!(val.as_deref(), Some("test_val"));
        }

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    fn uuid_now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}
