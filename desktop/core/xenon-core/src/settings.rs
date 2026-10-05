//! The `settings` namespace: simple key/value settings plus site
//! preferences, all through prepared statements (ENG-04).

use crate::db::Database;
use crate::Result;
use rusqlite::OptionalExtension;

/// Typed access to the `settings` and `site_prefs` tables.
pub struct Settings {
    db: Database,
}

impl Settings {
    /// Take ownership of an opened, migrated database.
    pub fn new(db: Database) -> Settings {
        Settings { db }
    }

    /// Give the database back (used by the binary on shutdown paths).
    pub fn into_db(self) -> Database {
        self.db
    }

    /// Read one setting; `None` when unset.
    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        let value = self
            .db
            .conn()
            .prepare_cached("SELECT value FROM settings WHERE key = ?1")?
            .query_row([key], |row| row.get::<_, String>(0))
            .optional()?;
        Ok(value)
    }

    /// Write one setting (insert or replace).
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.db
            .conn()
            .prepare_cached(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, unixepoch())
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            )?
            .execute(rusqlite::params![key, value])?;
        Ok(())
    }

    /// Delete one setting; returns true when a row was removed.
    pub fn delete(&mut self, key: &str) -> Result<bool> {
        let changed = self
            .db
            .conn()
            .prepare_cached("DELETE FROM settings WHERE key = ?1")?
            .execute([key])?;
        Ok(changed > 0)
    }

    /// List all settings as (key, value) pairs, ordered by key. Settings
    /// values are product configuration, never secrets (passwords live in
    /// their own table in M5.1 and never in `settings`).
    pub fn list(&mut self) -> Result<Vec<(String, String)>> {
        let mut statement = self
            .db
            .conn()
            .prepare_cached("SELECT key, value FROM settings ORDER BY key")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Read the per-site preferences row for `domain`.
    pub fn site_pref(&mut self, domain: &str) -> Result<Option<SitePref>> {
        let row = self
            .db
            .conn()
            .prepare_cached(
                "SELECT zoom, engine, permissions, shield_override
                 FROM site_prefs WHERE domain = ?1",
            )?
            .query_row([domain], |row| {
                Ok(SitePref {
                    domain: domain.to_string(),
                    zoom: row.get(0)?,
                    engine: row.get(1)?,
                    permissions: row.get(2)?,
                    shield_override: row.get(3)?,
                })
            })
            .optional()?;
        Ok(row)
    }

    /// Upsert the per-site preferences row for `domain`.
    pub fn set_site_pref(&mut self, pref: &SitePref) -> Result<()> {
        self.db
            .conn()
            .prepare_cached(
                "INSERT INTO site_prefs (domain, zoom, engine, permissions, shield_override, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, unixepoch())
             ON CONFLICT(domain) DO UPDATE SET
               zoom = excluded.zoom, engine = excluded.engine,
               permissions = excluded.permissions,
               shield_override = excluded.shield_override,
               updated_at = excluded.updated_at",
            )?
            .execute(rusqlite::params![
                pref.domain,
                pref.zoom,
                pref.engine,
                pref.permissions,
                pref.shield_override
            ])?;
        Ok(())
    }
}

/// One site's preferences row. `engine` is `None` unless the user set a
/// per-site engine; `shield_override` is `None` unless set per site
/// (and can only loosen the ad/tracker shield, never the adult shield).
#[derive(Debug, Clone, PartialEq)]
pub struct SitePref {
    pub domain: String,
    pub zoom: f64,
    pub engine: Option<String>,
    /// JSON object of permission name to value.
    pub permissions: String,
    pub shield_override: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbKey;

    fn settings(name: &str) -> Settings {
        let mut db = Database::open(
            &{
                let mut path = std::env::temp_dir();
                path.push(format!("xenon-settings-{name}-{}.db", std::process::id()));
                let _ = std::fs::remove_file(&path);
                path
            },
            &DbKey::generate().expect("key"),
        )
        .expect("open db");
        db.migrate().expect("migrate");
        Settings::new(db)
    }

    #[test]
    fn set_get_delete_round_trip() {
        let mut settings = settings("roundtrip");
        assert_eq!(settings.get("home").expect("get"), None);
        settings.set("home", "xenon://newtab").expect("set");
        assert_eq!(
            settings.get("home").expect("get"),
            Some("xenon://newtab".to_string())
        );
        settings.set("home", "updated").expect("set again");
        assert_eq!(settings.get("home").expect("get"), Some("updated".into()));
        assert!(settings.delete("home").expect("delete"));
        assert!(!settings.delete("home").expect("delete again"));
    }

    #[test]
    fn list_is_ordered_and_complete() {
        let mut settings = settings("list");
        settings.set("b", "2").expect("set b");
        settings.set("a", "1").expect("set a");
        let all = settings.list().expect("list");
        assert_eq!(
            all,
            vec![("a".into(), "1".into()), ("b".into(), "2".into())]
        );
    }

    #[test]
    fn site_pref_round_trip() {
        let mut settings = settings("siteprefs");
        assert_eq!(settings.site_pref("example.com").expect("get"), None);
        settings
            .set_site_pref(&SitePref {
                domain: "example.com".into(),
                zoom: 1.25,
                engine: Some("blink".into()),
                permissions: r#"{"notifications":"block"}"#.into(),
                shield_override: Some(true),
            })
            .expect("set");
        let pref = settings
            .site_pref("example.com")
            .expect("get")
            .expect("row");
        assert_eq!(pref.zoom, 1.25);
        assert_eq!(pref.engine.as_deref(), Some("blink"));
        assert_eq!(pref.shield_override, Some(true));
        assert_eq!(pref.permissions, r#"{"notifications":"block"}"#);
    }
}
