//! Ordered schema migrations tracked in `PRAGMA user_version`.
//!
//! Every migration runs inside one transaction and bumps the version.
//! Re-running is a no-op, so startup can always call `run`.

use crate::Result;
use rusqlite::Connection;

/// One schema migration. `sql` may contain several statements.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

/// All migrations, in order. New schema changes append here; existing
/// migrations must never be edited once shipped.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "settings",
        sql: "CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
              ) WITHOUT ROWID;",
    },
    Migration {
        version: 2,
        name: "site_prefs",
        sql: "CREATE TABLE site_prefs (
                domain TEXT PRIMARY KEY,
                zoom REAL NOT NULL DEFAULT 1.0,
                engine TEXT,
                permissions TEXT NOT NULL DEFAULT '{}',
                shield_override INTEGER,
                updated_at INTEGER NOT NULL
              ) WITHOUT ROWID;",
    },
];

/// Apply every migration newer than the database's user_version.
/// Returns the versions applied by this call (empty when up to date).
pub fn run(conn: &mut Connection) -> Result<Vec<i64>> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let mut applied = Vec::new();
    for migration in MIGRATIONS {
        if migration.version <= current {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.pragma_update(None, "user_version", migration.version)?;
        tx.commit()?;
        applied.push(migration.version);
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_db() -> Connection {
        Connection::open_in_memory().expect("in-memory db")
    }

    #[test]
    fn fresh_database_gets_all_migrations() {
        let mut conn = memory_db();
        let applied = run(&mut conn).expect("migrate");
        assert_eq!(applied, vec![1, 2]);
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("user_version");
        assert_eq!(version, 2);
    }

    #[test]
    fn rerun_is_a_no_op() {
        let mut conn = memory_db();
        run(&mut conn).expect("first migrate");
        let applied = run(&mut conn).expect("second migrate");
        assert!(applied.is_empty(), "re-running must apply nothing");
    }

    #[test]
    fn tables_exist_after_migrations() {
        let mut conn = memory_db();
        run(&mut conn).expect("migrate");
        for table in ["settings", "site_prefs"] {
            let count: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("table lookup");
            assert_eq!(count, 1, "table {table} must exist");
        }
    }

    #[test]
    fn site_prefs_defaults_are_sane() {
        let mut conn = memory_db();
        run(&mut conn).expect("migrate");
        conn.execute(
            "INSERT INTO site_prefs (domain, updated_at) VALUES ('example.com', 0)",
            [],
        )
        .expect("insert");
        let (zoom, permissions): (f64, String) = conn
            .query_row(
                "SELECT zoom, permissions FROM site_prefs WHERE domain='example.com'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("select");
        assert_eq!(zoom, 1.0);
        assert_eq!(permissions, "{}");
    }
}
