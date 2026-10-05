//! OS keystore integration (Architecture.md 6.1 `keystore`): the SQLCipher
//! database key lives in the OS credential store — Windows Credential
//! Manager (DPAPI-backed) on Windows, the Secret Service on Linux — with
//! an optional Argon2id master-password wrap in [`wrap`].

pub mod resolve;
pub mod wrap;

use crate::{CoreError, Result};
use std::cell::RefCell;
use std::collections::HashMap;

/// Keystore service name; one account per stored item.
pub const SERVICE: &str = "Xenon Browser";
/// Account name of the database key entry.
pub const DB_KEY_ACCOUNT: &str = "core/db-key";

/// Storage for small secret strings. Implementations must never log or
/// include stored values in error messages (PRIV-05).
pub trait KeyStore {
    /// Read an entry; `None` when absent.
    fn get(&self, account: &str) -> Result<Option<String>>;
    /// Create or overwrite an entry.
    fn set(&self, account: &str, value: &str) -> Result<()>;
    /// Remove an entry; absent entries count as removed.
    fn delete(&self, account: &str) -> Result<()>;
}

/// In-memory keystore for tests and non-persistent runs. Not used by the
/// shipped browser.
#[derive(Default)]
pub struct MemoryKeyStore {
    entries: RefCell<HashMap<String, String>>,
}

impl KeyStore for MemoryKeyStore {
    fn get(&self, account: &str) -> Result<Option<String>> {
        Ok(self.entries.borrow().get(account).cloned())
    }

    fn set(&self, account: &str, value: &str) -> Result<()> {
        self.entries
            .borrow_mut()
            .insert(account.to_string(), value.to_string());
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<()> {
        self.entries.borrow_mut().remove(account);
        Ok(())
    }
}

/// OS-backed keystore: Windows Credential Manager or the Linux Secret
/// Service, via the `keyring` crate.
#[derive(Default)]
pub struct OsKeyStore;

impl OsKeyStore {
    fn entry(&self, account: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(SERVICE, account)
            .map_err(|err| CoreError::Keystore(format!("keystore unavailable: {err}")))
    }
}

impl KeyStore for OsKeyStore {
    fn get(&self, account: &str) -> Result<Option<String>> {
        let entry = self.entry(account)?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(CoreError::Keystore(format!("keystore read failed: {err}"))),
        }
    }

    fn set(&self, account: &str, value: &str) -> Result<()> {
        let entry = self.entry(account)?;
        entry
            .set_password(value)
            .map_err(|err| CoreError::Keystore(format!("keystore write failed: {err}")))
    }

    fn delete(&self, account: &str) -> Result<()> {
        let entry = self.entry(account)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(CoreError::Keystore(format!(
                "keystore delete failed: {err}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip() {
        let store = MemoryKeyStore::default();
        assert_eq!(store.get("a").expect("get"), None);
        store.set("a", "value").expect("set");
        assert_eq!(store.get("a").expect("get"), Some("value".into()));
        store.delete("a").expect("delete");
        assert_eq!(store.get("a").expect("get"), None);
        store.delete("a").expect("delete absent is fine");
    }

    #[test]
    fn os_store_round_trip() {
        // On Linux the Secret Service only exists when CI started the
        // headless gnome-keyring (XENON_FORCE_KEYSTORE_TEST=1); without a
        // daemon the test would measure the environment, not the code, so
        // it skips. Windows always runs it for real.
        let forced = std::env::var("XENON_FORCE_KEYSTORE_TEST").is_ok();
        if !cfg!(windows) && !forced {
            return;
        }
        let store = OsKeyStore;
        let account = format!("core/test-{}", std::process::id());
        // Propagate the set failure itself: a later get() panic would
        // hide the real cause (locked keyring, missing daemon, ...).
        if let Err(err) = store.set(&account, "test-value") {
            if forced {
                panic!("forced keystore test: set failed: {err}");
            }
            eprintln!("skipping: no usable OS keystore here ({err})");
            return;
        }
        assert_eq!(store.get(&account).expect("get"), Some("test-value".into()));
        store.delete(&account).expect("delete");
        assert_eq!(store.get(&account).expect("get"), None);
    }
}
