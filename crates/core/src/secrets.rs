use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};

/// Almacén de secretos del sistema operativo (o un sustituto en pruebas).
pub trait SecretStore: Send + Sync {
    fn get(&self, name: &str) -> Result<Option<String>>;
    fn set(&self, name: &str, value: &str) -> Result<()>;
}

impl<T: SecretStore + ?Sized> SecretStore for Arc<T> {
    fn get(&self, name: &str) -> Result<Option<String>> {
        (**self).get(name)
    }

    fn set(&self, name: &str, value: &str) -> Result<()> {
        (**self).set(name, value)
    }
}

/// Keychain en macOS, Credential Manager en Windows, Secret Service en Linux.
pub struct SystemSecretStore {
    service: String,
}

impl SystemSecretStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self { service: service.into() }
    }

    fn entry(&self, name: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(&self.service, name).map_err(|_| Error::SecretStoreUnavailable)
    }
}

impl SecretStore for SystemSecretStore {
    fn get(&self, name: &str) -> Result<Option<String>> {
        match self.entry(name)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(Error::SecretStoreUnavailable),
        }
    }

    fn set(&self, name: &str, value: &str) -> Result<()> {
        self.entry(name)?.set_password(value).map_err(|_| Error::SecretStoreUnavailable)
    }
}

/// Almacén en memoria, para pruebas.
#[derive(Default)]
pub struct MemorySecretStore {
    values: Mutex<HashMap<String, String>>,
}

impl SecretStore for MemorySecretStore {
    fn get(&self, name: &str) -> Result<Option<String>> {
        Ok(self.values.lock().unwrap().get(name).cloned())
    }

    fn set(&self, name: &str, value: &str) -> Result<()> {
        self.values.lock().unwrap().insert(name.to_string(), value.to_string());
        Ok(())
    }
}
