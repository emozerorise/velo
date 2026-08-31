//! Write-only storage for summarisation API keys.
//!
//! The account is the destination host, so changing a provider's path or
//! port does not make the user enter the same credential again. The secret
//! never joins `SummarySettings`, which keeps it out of settings JSON and
//! every IPC response built from those settings.

use crate::errors::{Result, VeloError};
use keyring::{Entry, Error as KeyringError};
use std::sync::Arc;

const SERVICE: &str = "com.velo.summary";

pub trait CredentialBackend: Send + Sync {
    fn get(&self, service: &str, account: &str) -> std::result::Result<Option<String>, String>;
    fn set(&self, service: &str, account: &str, key: &str) -> std::result::Result<(), String>;
    fn clear(&self, service: &str, account: &str) -> std::result::Result<(), String>;
}

struct OsKeyring;

impl OsKeyring {
    fn entry(service: &str, account: &str) -> std::result::Result<Entry, String> {
        Entry::new(service, account).map_err(|e| e.to_string())
    }
}

impl CredentialBackend for OsKeyring {
    fn get(&self, service: &str, account: &str) -> std::result::Result<Option<String>, String> {
        match Self::entry(service, account)?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn set(&self, service: &str, account: &str, key: &str) -> std::result::Result<(), String> {
        Self::entry(service, account)?
            .set_password(key)
            .map_err(|e| e.to_string())
    }

    fn clear(&self, service: &str, account: &str) -> std::result::Result<(), String> {
        match Self::entry(service, account)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct ApiKeyStore {
    backend: Arc<dyn CredentialBackend>,
}

impl Default for ApiKeyStore {
    fn default() -> Self {
        Self {
            backend: Arc::new(OsKeyring),
        }
    }
}

impl ApiKeyStore {
    #[cfg(test)]
    fn with_backend(backend: Arc<dyn CredentialBackend>) -> Self {
        Self { backend }
    }

    pub fn get(&self, base_url: &str) -> Result<Option<String>> {
        let account = account_for(base_url)?;
        self.backend
            .get(SERVICE, &account)
            .map_err(|e| VeloError::Summary(format!("Could not read the saved API key: {}", e)))
    }

    pub fn has(&self, base_url: &str) -> Result<bool> {
        Ok(self.get(base_url)?.is_some())
    }

    pub fn set(&self, base_url: &str, key: &str) -> Result<()> {
        if key.trim().is_empty() {
            return Err(VeloError::InvalidParameter(
                "The API key cannot be empty".into(),
            ));
        }

        let account = account_for(base_url)?;
        self.backend
            .set(SERVICE, &account, key)
            .map_err(|e| VeloError::Summary(format!("Could not save the API key: {}", e)))
    }

    pub fn clear(&self, base_url: &str) -> Result<()> {
        let account = account_for(base_url)?;
        self.backend
            .clear(SERVICE, &account)
            .map_err(|e| VeloError::Summary(format!("Could not clear the API key: {}", e)))
    }
}

fn account_for(base_url: &str) -> Result<String> {
    reqwest::Url::parse(base_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .ok_or_else(|| {
            VeloError::InvalidParameter("The model server address must be a URL with a host".into())
        })
}

#[cfg(test)]
mod tests {
    use super::{ApiKeyStore, CredentialBackend};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FakeBackend {
        entries: Mutex<HashMap<(String, String), String>>,
    }

    impl CredentialBackend for FakeBackend {
        fn get(&self, service: &str, account: &str) -> std::result::Result<Option<String>, String> {
            Ok(self
                .entries
                .lock()
                .expect("fake keychain lock poisoned")
                .get(&(service.to_string(), account.to_string()))
                .cloned())
        }

        fn set(&self, service: &str, account: &str, key: &str) -> std::result::Result<(), String> {
            self.entries
                .lock()
                .expect("fake keychain lock poisoned")
                .insert((service.to_string(), account.to_string()), key.to_string());
            Ok(())
        }

        fn clear(&self, service: &str, account: &str) -> std::result::Result<(), String> {
            self.entries
                .lock()
                .expect("fake keychain lock poisoned")
                .remove(&(service.to_string(), account.to_string()));
            Ok(())
        }
    }

    fn store() -> ApiKeyStore {
        ApiKeyStore::with_backend(Arc::new(FakeBackend::default()))
    }

    #[test]
    fn a_key_is_shared_by_urls_on_the_same_host() {
        let store = store();
        store
            .set("https://api.example.com/v1", "secret-key")
            .expect("key should save");

        assert_eq!(
            store
                .get("https://api.example.com:8443/compatible/v1")
                .expect("key should load")
                .as_deref(),
            Some("secret-key")
        );
        assert!(store
            .get("https://other.example.com/v1")
            .expect("other host should be readable")
            .is_none());
    }

    #[test]
    fn clearing_is_idempotent_and_does_not_touch_another_host() {
        let store = store();
        store
            .set("https://first.example.com/v1", "first")
            .expect("first key should save");
        store
            .set("https://second.example.com/v1", "second")
            .expect("second key should save");

        store
            .clear("https://first.example.com/v1")
            .expect("existing key should clear");
        store
            .clear("https://first.example.com/v1")
            .expect("missing key should also clear");

        assert!(!store
            .has("https://first.example.com/v1")
            .expect("first host should be readable"));
        assert!(store
            .has("https://second.example.com/v1")
            .expect("second host should be readable"));
    }

    #[test]
    fn empty_keys_and_urls_without_hosts_are_rejected() {
        let store = store();

        assert!(store
            .set("https://api.example.com/v1", "   ")
            .expect_err("blank key should fail")
            .to_string()
            .contains("empty"));
        assert!(store
            .set("not a URL", "secret")
            .expect_err("invalid URL should fail")
            .to_string()
            .contains("server address"));
    }
}
