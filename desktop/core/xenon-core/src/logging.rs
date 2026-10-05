//! PRIV-05-safe structured logging.
//!
//! Core logs events with typed fields. The logger rejects — never
//! silently passes through — anything that could carry user data:
//! field names like `url`, `query`, `password`, `cookie`, `token`,
//! `title`, `note`, and string values that look like URLs. Hostnames go
//! through [`Field::Domain`], which validates that the value is a bare
//! domain, because the live interception log is domain-only. Rejections
//! are counted; the rejected content is never written anywhere.

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Log severity, ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
        }
    }
}

/// One structured field value. Strings are screened for URL-like
/// content; use [`Field::Domain`] for hostnames.
#[derive(Debug, Clone)]
pub enum Field<'a> {
    /// Free text; rejected when it looks like a URL or carries a path.
    Str(&'a str),
    /// A bare hostname such as `example.com` — the only user-related
    /// data PRIV-05 allows in the in-memory log.
    Domain(&'a str),
    Int(i64),
    Bool(bool),
}

/// Banned tokens: a field is rejected when its normalized name contains
/// any of these substrings (`page_title` -> `pagetitle` matches `title`).
/// Conservative on purpose — a false rejection is cheaper than a leak.
const BANNED_TOKENS: &[&str] = &[
    "url", "uri", "query", "search", "password", "passwd", "pwd", "cookie", "token", "secret",
    "key", "title", "note", "referrer", "auth", "history",
];

fn normalize_key(key: &str) -> String {
    key.chars()
        .filter(|c| *c != '-' && *c != '_')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn key_is_banned(key: &str) -> bool {
    let normalized = normalize_key(key);
    BANNED_TOKENS.iter().any(|token| normalized.contains(token))
}

/// True for values that would reveal navigation or content data.
fn looks_url_like(value: &str) -> bool {
    let lower = value.to_lowercase();
    lower.contains("://")
        || lower.starts_with("data:")
        || lower.starts_with("mailto:")
        || lower.starts_with("www.")
        || lower.contains('/')
        || lower.contains('\\')
}

/// True for long hex or base64 blobs, which are almost always key or
/// token material even when someone gives them an innocent field name.
fn looks_like_secret(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() >= 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    trimmed.len() >= 40
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

/// True when `value` is a bare domain (labels and dots, optionally a
/// leading `xn--` label; no scheme, path, port or credentials).
pub fn is_bare_domain(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 253
        && !value.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-'))
        && !value.starts_with('.')
        && !value.ends_with('.')
        && value.split('.').all(|label| !label.is_empty())
}

/// Structured logger writing JSON lines to a sink.
pub struct Logger {
    sink: Mutex<Box<dyn Write + Send>>,
    rejected: AtomicU64,
}

impl Logger {
    /// Log to standard error (the RPC channel on stdout must stay clean).
    pub fn to_stderr() -> Logger {
        Logger {
            sink: Mutex::new(Box::new(std::io::stderr())),
            rejected: AtomicU64::new(0),
        }
    }

    /// Log into memory (tests and the later in-memory interception log).
    pub fn in_memory() -> (Logger, std::sync::Arc<Mutex<Vec<u8>>>) {
        let buffer = std::sync::Arc::new(Mutex::new(Vec::new()));
        let logger = Logger {
            sink: Mutex::new(Box::new(SharedSink {
                buffer: buffer.clone(),
            })),
            rejected: AtomicU64::new(0),
        };
        (logger, buffer)
    }

    /// Record one event. Fields whose key is banned, or whose value
    /// looks like a URL, are dropped and counted — nothing about the
    /// rejected content is written.
    pub fn event(&self, level: Level, event: &str, fields: &[(&str, Field)]) {
        let mut record = serde_json::Map::new();
        record.insert("level".into(), serde_json::json!(level.as_str()));
        record.insert("event".into(), serde_json::json!(event));
        for (key, value) in fields {
            if key_is_banned(key) {
                self.rejected.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            match value {
                Field::Str(text) if looks_url_like(text) || looks_like_secret(text) => {
                    self.rejected.fetch_add(1, Ordering::Relaxed);
                }
                Field::Str(text) => {
                    record.insert((*key).into(), serde_json::json!(text));
                }
                Field::Domain(host) if is_bare_domain(host) => {
                    record.insert((*key).into(), serde_json::json!(host));
                }
                Field::Domain(_) => {
                    self.rejected.fetch_add(1, Ordering::Relaxed);
                }
                Field::Int(number) => {
                    record.insert((*key).into(), serde_json::json!(number));
                }
                Field::Bool(flag) => {
                    record.insert((*key).into(), serde_json::json!(flag));
                }
            }
        }
        let line = serde_json::to_string(&serde_json::Value::Object(record))
            .unwrap_or_else(|_| "{\"level\":\"error\",\"event\":\"log.serialize\"}".into());
        if let Ok(mut sink) = self.sink.lock() {
            let _ = writeln!(sink, "{line}");
        }
    }

    /// How many fields have been rejected so far.
    pub fn rejected_count(&self) -> u64 {
        self.rejected.load(Ordering::Relaxed)
    }
}

/// Sink adapter sharing an in-memory buffer between the logger and the
/// test/harness that reads it.
struct SharedSink {
    buffer: std::sync::Arc<Mutex<Vec<u8>>>,
}

impl Write for SharedSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.buffer.lock() {
            Ok(mut buffer) => {
                buffer.extend_from_slice(buf);
                Ok(buf.len())
            }
            Err(_) => Err(std::io::Error::other("log buffer poisoned")),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn captured() -> (Logger, std::sync::Arc<Mutex<Vec<u8>>>) {
        Logger::in_memory()
    }

    fn contents(buffer: &std::sync::Arc<Mutex<Vec<u8>>>) -> String {
        match buffer.lock() {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(_) => String::new(),
        }
    }

    #[test]
    fn plain_events_are_written() {
        let (logger, buffer) = captured();
        logger.event(Level::Info, "core.started", &[("pid", Field::Int(123))]);
        let text = contents(&buffer);
        assert!(text.contains("core.started"));
        assert!(text.contains("\"pid\":123"));
    }

    #[test]
    fn url_values_are_rejected() {
        let (logger, buffer) = captured();
        logger.event(
            Level::Info,
            "nav.attempt",
            &[("target", Field::Str("https://example.com/secret-path"))],
        );
        assert_eq!(logger.rejected_count(), 1);
        assert!(!contents(&buffer).contains("example.com"));
    }

    #[test]
    fn banned_keys_are_rejected_whatever_the_value() {
        let (logger, _buffer) = captured();
        for key in ["url", "URL", "page_title", "cookie", "auth-token", "db_key"] {
            logger.event(Level::Info, "e", &[(key, Field::Str("innocent"))]);
        }
        assert_eq!(logger.rejected_count(), 6);
    }

    #[test]
    fn domains_pass_and_paths_fail() {
        let (logger, buffer) = captured();
        logger.event(
            Level::Info,
            "net.host",
            &[("host", Field::Domain("example.com"))],
        );
        logger.event(
            Level::Info,
            "net.host",
            &[("host", Field::Domain("example.com/path"))],
        );
        assert_eq!(logger.rejected_count(), 1);
        assert!(contents(&buffer).contains("example.com"));
    }

    #[test]
    fn database_key_never_reaches_the_log() {
        // Acceptance check (M1.3): the SQLCipher key must never appear in
        // logs, whatever innocent-looking field name someone uses for it.
        let (logger, buffer) = captured();
        let key_hex = "3f2a".repeat(16); // stand-in 32-byte key as hex
        logger.event(Level::Info, "db.opened", &[("key", Field::Str(&key_hex))]);
        logger.event(
            Level::Info,
            "db.opened",
            &[("keyvalue", Field::Str(&key_hex))],
        );
        logger.event(
            Level::Info,
            "db.opened",
            &[("db-key", Field::Str(&key_hex))],
        );
        logger.event(
            Level::Info,
            "db.opened",
            &[("masterkey", Field::Str(&key_hex))],
        );
        let text = contents(&buffer);
        assert!(!text.contains(&key_hex), "the key hex must never be logged");
        // And the key material cannot hide in a URL-shaped value either.
        logger.event(Level::Info, "db.opened", &[("label", Field::Str(&key_hex))]);
        assert!(!contents(&buffer).contains(&key_hex));
        assert!(logger.rejected_count() >= 4);
    }
}
