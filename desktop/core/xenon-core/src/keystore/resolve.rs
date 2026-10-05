//! Database-key lifecycle over a [`KeyStore`]: raw hex storage by
//! default, or Argon2id-wrapped storage when the user opts into a master
//! password. This is the only place that decides how a stored entry is
//! interpreted, so the binary and future hosts cannot disagree.

use super::wrap::{self, WrappedKey};
use super::{KeyStore, DB_KEY_ACCOUNT};
use crate::db::DbKey;
use crate::{CoreError, Result};
use serde_json::from_str;

/// Store `key` under the standard account, wrapped when a master
/// password is given, raw hex otherwise.
pub fn store_db_key(
    store: &dyn KeyStore,
    key: &DbKey,
    master_password: Option<&str>,
) -> Result<()> {
    match master_password {
        Some(password) => {
            let wrapped = wrap::wrap(key.as_bytes(), password)?;
            let blob = serde_json::to_string(&wrapped)
                .map_err(|err| CoreError::Crypto(format!("wrap serialization failed: {err}")))?;
            store.set(DB_KEY_ACCOUNT, &blob)
        }
        None => store.set(DB_KEY_ACCOUNT, &key.as_hex()),
    }
}

/// Read the stored key. `Ok(None)` when nothing is stored yet.
/// A wrapped entry requires `master_password`; a wrong password and a
/// corrupt entry both fail without revealing which one happened.
pub fn load_db_key(store: &dyn KeyStore, master_password: Option<&str>) -> Result<Option<DbKey>> {
    let stored = match store.get(DB_KEY_ACCOUNT)? {
        Some(stored) => stored,
        None => return Ok(None),
    };
    if is_wrapped_blob(&stored) {
        let wrapped: WrappedKey = from_str(&stored)
            .map_err(|_| CoreError::Keystore("stored key entry is corrupt".into()))?;
        let password = master_password.ok_or_else(|| {
            CoreError::Keystore("master password required to unlock the database key".into())
        })?;
        let key = wrap::unwrap(&wrapped, password)?;
        Ok(Some(DbKey::from_hex(&crate::hex_encode(key.as_slice()))?))
    } else {
        // A raw entry with a password supplied is still usable; the
        // password simply does not apply until convert_to_wrapped runs.
        Ok(Some(DbKey::from_hex(&stored)?))
    }
}

/// Load the key, generating and storing a fresh one on first run. The
/// storage format follows `master_password` for the new key; an existing
/// entry keeps its format (use [`convert_to_wrapped`] to migrate).
pub fn ensure_db_key(store: &dyn KeyStore, master_password: Option<&str>) -> Result<DbKey> {
    if let Some(key) = load_db_key(store, master_password)? {
        return Ok(key);
    }
    let key = DbKey::generate()?;
    store_db_key(store, &key, master_password)?;
    Ok(key)
}

/// Migrate an existing raw entry to master-password-wrapped storage.
/// Fails if nothing is stored yet or the entry is already wrapped.
pub fn convert_to_wrapped(store: &dyn KeyStore, master_password: &str) -> Result<DbKey> {
    let stored = store
        .get(DB_KEY_ACCOUNT)?
        .ok_or_else(|| CoreError::Keystore("no key stored yet".into()))?;
    if is_wrapped_blob(&stored) {
        return Err(CoreError::Keystore(
            "the stored key is already wrapped".into(),
        ));
    }
    let key = DbKey::from_hex(&stored)?;
    store_db_key(store, &key, Some(master_password))?;
    Ok(key)
}

/// Normalize the entry to `key` (raw) and immediately wrap it under
/// `master_password`. Used when a harness provides the key explicitly
/// and wants wrapped storage in one step.
pub fn reset_and_wrap(store: &dyn KeyStore, key: &DbKey, master_password: &str) -> Result<()> {
    store_db_key(store, key, None)?;
    convert_to_wrapped(store, master_password).map(|_| ())
}

/// A wrapped entry is a JSON object; raw entries are 64 hex characters.
fn is_wrapped_blob(stored: &str) -> bool {
    stored.trim_start().starts_with('{')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::MemoryKeyStore;

    const KEY_HEX: &str = "1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd";
    const PASSWORD: &str = "test master password";

    fn stored_value(store: &MemoryKeyStore) -> String {
        store.get(DB_KEY_ACCOUNT).expect("get").expect("stored")
    }

    #[test]
    fn raw_storage_round_trip() {
        let store = MemoryKeyStore::default();
        let key = DbKey::from_hex(KEY_HEX).expect("key");
        store_db_key(&store, &key, None).expect("store");
        assert_eq!(stored_value(&store), KEY_HEX);
        let loaded = load_db_key(&store, None).expect("load").expect("some");
        assert_eq!(loaded.as_hex(), KEY_HEX);
    }

    #[test]
    fn wrapped_storage_round_trip() {
        let store = MemoryKeyStore::default();
        let key = DbKey::from_hex(KEY_HEX).expect("key");
        store_db_key(&store, &key, Some(PASSWORD)).expect("store wrapped");
        assert!(stored_value(&store).starts_with('{'), "stored wrapped");
        let loaded = load_db_key(&store, Some(PASSWORD))
            .expect("load")
            .expect("some");
        assert_eq!(loaded.as_hex(), KEY_HEX);
    }

    #[test]
    fn wrapped_entry_requires_the_password() {
        let store = MemoryKeyStore::default();
        let key = DbKey::from_hex(KEY_HEX).expect("key");
        store_db_key(&store, &key, Some(PASSWORD)).expect("store wrapped");
        let error = load_db_key(&store, None).expect_err("must require password");
        assert!(error.to_string().contains("master password required"));
        assert!(load_db_key(&store, Some("wrong")).is_err());
    }

    #[test]
    fn ensure_creates_and_reuses() {
        let store = MemoryKeyStore::default();
        let first = ensure_db_key(&store, None).expect("ensure");
        let second = ensure_db_key(&store, None).expect("ensure again");
        assert_eq!(first.as_hex(), second.as_hex(), "must not regenerate");
    }

    #[test]
    fn convert_to_wrapped_migrates_raw_entries() {
        let store = MemoryKeyStore::default();
        let key = DbKey::from_hex(KEY_HEX).expect("key");
        store_db_key(&store, &key, None).expect("store raw");
        convert_to_wrapped(&store, PASSWORD).expect("convert");
        // Raw load no longer applies (it is wrapped now).
        assert!(load_db_key(&store, None).is_err());
        let loaded = load_db_key(&store, Some(PASSWORD))
            .expect("load wrapped")
            .expect("some");
        assert_eq!(loaded.as_hex(), KEY_HEX);
        assert!(
            convert_to_wrapped(&store, PASSWORD).is_err(),
            "already wrapped"
        );
    }

    #[test]
    fn corrupt_entry_is_an_error() {
        let store = MemoryKeyStore::default();
        store.set(DB_KEY_ACCOUNT, "{not json").expect("set");
        assert!(load_db_key(&store, Some(PASSWORD)).is_err());
    }
}
