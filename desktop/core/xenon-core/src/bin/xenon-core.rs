//! xenon-core: the core service process (Architecture.md 4.2).
//!
//! Reads JSON-RPC 2.0, one message per line, from stdin and writes one
//! response line per request to stdout. Exits 0 when stdin closes.
//! Logging goes to stderr only, through the PRIV-05-safe logger.

use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use xenon_core::db::{Database, DbKey};
use xenon_core::keystore::resolve;
use xenon_core::keystore::{KeyStore, OsKeyStore, DB_KEY_ACCOUNT};
use xenon_core::logging::{Field, Level, Logger};
use xenon_core::rpc::framing::FrameReader;
use xenon_core::rpc::{Service, MAX_LINE_BYTES};
use xenon_core::settings::Settings;
use xenon_core::{CoreError, Result};

fn main() -> ExitCode {
    let logger = Logger::to_stderr();
    match run(&logger) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            logger.event(
                Level::Error,
                "core.fatal",
                &[("message", Field::Str(&err.to_string()))],
            );
            ExitCode::FAILURE
        }
    }
}

fn run(logger: &Logger) -> Result<()> {
    let db_path = db_path()?;
    let key = db_key(logger)?;
    let mut db = Database::open(&db_path, &key)?;
    let applied = db.migrate()?;
    logger.event(
        Level::Info,
        "core.started",
        &[
            ("migrationsApplied", Field::Int(applied.len() as i64)),
            (
                "protocolVersion",
                Field::Int(xenon_core::rpc::PROTOCOL_VERSION),
            ),
        ],
    );

    let mut service = Service::new(Settings::new(db));
    let stdin = std::io::stdin();
    let mut reader = FrameReader::new(stdin.lock(), MAX_LINE_BYTES);
    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    loop {
        match reader.read_message() {
            Ok(None) => break, // stdin closed: core exits (Architecture.md 4.2)
            Ok(Some(line)) => {
                if let Some(response) = service.handle_line(&line) {
                    writeln!(writer, "{response}")?;
                    writer.flush()?;
                }
            }
            Err(CoreError::Rpc { code, message }) => {
                // Framing-level failures answer with a null id and keep
                // serving the next line.
                writeln!(
                    writer,
                    r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":{code},"message":{:?}}}}}"#,
                    message
                )?;
                writer.flush()?;
            }
            Err(err) => return Err(err),
        }
    }
    logger.event(Level::Info, "core.stopped", &[]);
    Ok(())
}

/// Database location: `--db <path>` argument, else `XENON_CORE_DB_PATH`,
/// else `xenon.db` in the current directory (development only).
fn db_path() -> Result<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(position) = args.iter().position(|arg| arg == "--db") {
        let path = args
            .get(position + 1)
            .ok_or_else(|| CoreError::Other("--db needs a path argument".to_string()))?;
        return Ok(PathBuf::from(path));
    }
    if let Ok(path) = std::env::var("XENON_CORE_DB_PATH") {
        return Ok(PathBuf::from(path));
    }
    Ok(PathBuf::from("xenon.db"))
}

/// The SQLCipher key: from `XENON_CORE_DB_KEY` (hex; dev and test
/// harnesses only — the shipped browser never passes keys through the
/// environment), otherwise resolved through the OS keystore: generated on
/// first run, optionally wrapped under a master password.
///
/// Master-password environment variables (dev/test plumbing until hosts
/// prompt through the Bridge in a later milestone):
/// - `XENON_CORE_SETUP_MASTER_PASSWORD`: one-shot migration of a raw
///   stored key to wrapped storage.
/// - `XENON_CORE_MASTER_PASSWORD`: unlocks a wrapped key.
/// - `XENON_CORE_KEYSTORE_ACCOUNT`: keystore account override so tests
///   never touch the real profile entry.
fn db_key(logger: &Logger) -> Result<DbKey> {
    let account =
        std::env::var("XENON_CORE_KEYSTORE_ACCOUNT").unwrap_or_else(|_| DB_KEY_ACCOUNT.to_string());
    let store = AccountStore {
        inner: OsKeyStore,
        account: &account,
    };
    let explicit = match std::env::var("XENON_CORE_DB_KEY") {
        Ok(hex) => Some(DbKey::from_hex(&hex)?),
        Err(_) => None,
    };
    if let Ok(setup) = std::env::var("XENON_CORE_SETUP_MASTER_PASSWORD") {
        // One-shot migration to wrapped storage; the explicit dev key is
        // wrapped when given, otherwise the stored (raw) key is wrapped.
        let key = match explicit {
            Some(key) => {
                resolve::reset_and_wrap(&store, &key, &setup)?;
                key
            }
            None => {
                let key = resolve::ensure_db_key(&store, None)?;
                resolve::convert_to_wrapped(&store, &setup)?;
                key
            }
        };
        logger.event(Level::Info, "core.keywrap.enabled", &[]);
        return Ok(key);
    }
    match explicit {
        Some(key) => Ok(key),
        None => {
            let master = std::env::var("XENON_CORE_MASTER_PASSWORD").ok();
            resolve::ensure_db_key(&store, master.as_deref()).map_err(|err| match err {
                CoreError::Keystore(message) if message.contains("master password required") => {
                    CoreError::Keystore(
                        "the database key is wrapped; set XENON_CORE_MASTER_PASSWORD \
                         (the UI will prompt in a later milestone)"
                            .into(),
                    )
                }
                other => other,
            })
        }
    }
}

/// A keystore view pinned to one account name, so callers cannot mix up
/// entries and tests can avoid the real profile entry.
struct AccountStore<'a> {
    inner: OsKeyStore,
    account: &'a str,
}

impl KeyStore for AccountStore<'_> {
    fn get(&self, _account: &str) -> Result<Option<String>> {
        self.inner.get(self.account)
    }
    fn set(&self, _account: &str, value: &str) -> Result<()> {
        self.inner.set(self.account, value)
    }
    fn delete(&self, _account: &str) -> Result<()> {
        self.inner.delete(self.account)
    }
}
