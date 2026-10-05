//! SQLCipher database access. Every query in core goes through here with
//! prepared statements (ENG-04); hosts and UI never open the database.

pub mod migrations;

use crate::{hex_encode, CoreError, Result};
use rusqlite::Connection;
use zeroize::Zeroizing;

/// The SQLCipher key. Zeroized on drop so the bytes do not linger in
/// memory (Rules.md 4.3).
#[derive(Clone)]
pub struct DbKey(Zeroizing<Vec<u8>>);

impl DbKey {
    /// Generate a fresh random 256-bit key.
    pub fn generate() -> Result<DbKey> {
        let mut bytes = vec![0u8; 32];
        getrandom::fill(&mut bytes)
            .map_err(|err| CoreError::Crypto(format!("rng failed: {err}")))?;
        Ok(DbKey(Zeroizing::new(bytes)))
    }

    /// Build a key from hex (dev/test override and keystore round trips).
    pub fn from_hex(text: &str) -> Result<DbKey> {
        let bytes = crate::hex_decode(text)?;
        if bytes.len() != 32 {
            return Err(CoreError::Crypto(format!(
                "db key must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        Ok(DbKey(Zeroizing::new(bytes)))
    }

    /// Lowercase hex form. Only for SQLCipher's key literal and the dev
    /// override — never for logging (PRIV-05).
    pub fn as_hex(&self) -> String {
        hex_encode(&self.0)
    }

    /// Raw bytes, for the master-password wrap.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An open, keyed database connection.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open `path` with the SQLCipher key and verify the key works by
    /// forcing a read. The raw key never appears in errors.
    pub fn open(path: &std::path::Path, key: &DbKey) -> Result<Database> {
        let conn = Connection::open(path)?;
        // Raw key material as an x'' literal, per SQLCipher's documented
        // raw-key form; the statement text with the key is built here and
        // passed to execute_batch without ever being logged.
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key.as_hex()))?;
        // Force SQLCipher to actually derive the page key now, so a wrong
        // key surfaces as an error here instead of on first use.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            let _: i64 = row.get(0)?;
            Ok(())
        })?;
        Ok(Database { conn })
    }

    /// Run all pending migrations (idempotent).
    pub fn migrate(&mut self) -> Result<Vec<i64>> {
        migrations::run(&mut self.conn)
    }

    /// Prepared-statement access for feature modules (settings, later
    /// session, filter, and so on).
    pub fn conn(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("xenon-core-test-{name}-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[test]
    fn open_with_correct_key_and_probe() {
        let path = temp_db_path("open-ok");
        let key = DbKey::generate().expect("generate key");
        let opened = Database::open(&path, &key);
        let _ = std::fs::remove_file(&path);
        assert!(opened.is_ok(), "opening with the right key must succeed");
    }

    #[test]
    fn open_without_key_fails() {
        let path = temp_db_path("open-nokey");
        let key = DbKey::generate().expect("generate key");
        {
            let mut db = Database::open(&path, &key).expect("open with key");
            db.migrate().expect("migrate");
        }
        // Reopen with no key at all: the probe must fail.
        let plain = Connection::open(&path);
        let result = plain.and_then(|conn| {
            conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
                let _: i64 = row.get(0)?;
                Ok(())
            })
        });
        let _ = std::fs::remove_file(&path);
        assert!(result.is_err(), "an unkeyed open must not read the file");
    }

    #[test]
    fn open_with_wrong_key_fails() {
        let path = temp_db_path("open-wrongkey");
        let key = DbKey::generate().expect("generate key");
        {
            let mut db = Database::open(&path, &key).expect("open with key");
            db.migrate().expect("migrate");
        }
        let wrong = DbKey::generate().expect("generate wrong key");
        let result = Database::open(&path, &wrong);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_err(), "a wrong key must be rejected");
    }

    #[test]
    fn file_on_disk_has_no_plaintext() {
        let path = temp_db_path("plaintext");
        let key = DbKey::generate().expect("generate key");
        let mut db = Database::open(&path, &key).expect("open with key");
        db.migrate().expect("migrate");
        db.conn()
            .execute(
                "INSERT INTO settings (key, value, updated_at) VALUES ('marker', 'XENON-PLAINTEXT-MARKER', 0)",
                [],
            )
            .expect("insert marker");
        drop(db);
        let raw = std::fs::read(&path).expect("read db file");
        let marker = b"XENON-PLAINTEXT-MARKER";
        let contains = raw.windows(marker.len()).any(|window| window == marker);
        let _ = std::fs::remove_file(&path);
        assert!(!contains, "plaintext must not appear in the database file");
    }

    #[test]
    fn key_from_hex_enforces_length() {
        assert!(DbKey::from_hex("00ff").is_err());
        assert!(DbKey::from_hex(&"ab".repeat(32)).is_ok());
    }
}
