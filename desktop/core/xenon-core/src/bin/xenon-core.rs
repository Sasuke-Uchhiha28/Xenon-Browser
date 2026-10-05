//! xenon-core: the core service process (Architecture.md 4.2).
//!
//! Reads JSON-RPC 2.0, one message per line, from stdin and writes one
//! response line per request to stdout. Exits 0 when stdin closes.
//! Logging goes to stderr only, through the PRIV-05-safe logger.

use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use xenon_core::db::{Database, DbKey};
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
    let key = db_key()?;
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
/// environment), otherwise fetched from the OS keystore, generated and
/// stored on first run.
fn db_key() -> Result<DbKey> {
    if let Ok(hex) = std::env::var("XENON_CORE_DB_KEY") {
        return DbKey::from_hex(&hex);
    }
    let store = OsKeyStore;
    if let Some(hex) = store.get(DB_KEY_ACCOUNT)? {
        return DbKey::from_hex(&hex);
    }
    let key = DbKey::generate()?;
    store.set(DB_KEY_ACCOUNT, &key.as_hex())?;
    Ok(key)
}
