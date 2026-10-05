//! Integration test: spawn the real `xenon-core` binary and drive it over
//! actual stdin/stdout, exactly like the engine hosts will (Architecture
//! 4.2). Uses a generated key via the dev-only env override and a temp
//! database; nothing here touches the owner's real profile data.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct CoreProc {
    child: Child,
    /// `Some` until shutdown: Drop closes stdin BEFORE waiting, because
    /// the core exits when stdin closes (Architecture.md 4.2).
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl CoreProc {
    fn start(db_path: &std::path::Path, key_hex: &str) -> CoreProc {
        Self::start_with(db_path, "core/db-key", |command| {
            command.env("XENON_CORE_DB_KEY", key_hex);
        })
    }

    /// Spawn with full control over the environment: `configure` adds the
    /// test's variables on top of the standard pipes and db path.
    fn start_with(
        db_path: &std::path::Path,
        keystore_account: &str,
        configure: impl FnOnce(&mut Command),
    ) -> CoreProc {
        let exe = env!("CARGO_BIN_EXE_xenon-core");
        let mut command = Command::new(exe);
        command
            .env("XENON_CORE_DB_PATH", db_path)
            .env("XENON_CORE_KEYSTORE_ACCOUNT", keystore_account)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        configure(&mut command);
        let mut child = command.spawn().expect("spawn xenon-core");
        let stdin = child.stdin.take().expect("take stdin");
        let stdout = BufReader::new(child.stdout.take().expect("take stdout"));
        CoreProc {
            child,
            stdin: Some(stdin),
            stdout,
        }
    }

    fn request(&mut self, line: &str) -> String {
        let stdin = self.stdin.as_mut().expect("stdin still open");
        writeln!(stdin, "{line}").expect("write request");
        stdin.flush().expect("flush request");
        let mut response = String::new();
        self.stdout.read_line(&mut response).expect("read response");
        response.trim_end().to_string()
    }

    fn write_raw(&mut self, line: &str) {
        let stdin = self.stdin.as_mut().expect("stdin still open");
        writeln!(stdin, "{line}").expect("write raw");
        stdin.flush().expect("flush raw");
    }
}

impl Drop for CoreProc {
    fn drop(&mut self) {
        // Close stdin first — the child exits when stdin closes — and
        // only then wait, or wait() would deadlock forever.
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.flush();
        }
        let _ = self.child.wait();
    }
}

#[test]
fn rpc_round_trip_over_real_stdio() {
    let mut db_path = std::env::temp_dir();
    db_path.push(format!("xenon-stdio-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&db_path);
    let key_hex = "ab".repeat(32);

    let mut core = CoreProc::start(&db_path, &key_hex);

    // Handshake: version and limits come back.
    let hello = core.request(r#"{"jsonrpc":"2.0","id":1,"method":"hello"}"#);
    let parsed: serde_json::Value = serde_json::from_str(&hello).expect("hello is json");
    assert_eq!(parsed["result"]["protocolVersion"], 1);
    assert_eq!(parsed["result"]["name"], "xenon-core");
    assert_eq!(parsed["result"]["maxLineBytes"], 1_048_576);

    // Settings round trip through the real binary and real SQLCipher file.
    let set = core.request(
        r#"{"jsonrpc":"2.0","id":2,"method":"settings.set","params":{"key":"engine","value":"gecko"}}"#,
    );
    assert!(set.contains(r#""ok":true"#), "set response was {set}");
    let get = core
        .request(r#"{"jsonrpc":"2.0","id":3,"method":"settings.get","params":{"key":"engine"}}"#);
    assert!(get.contains(r#""value":"gecko""#), "get response was {get}");

    // Unknown method -> JSON-RPC error, connection stays usable.
    let unknown = core.request(r#"{"jsonrpc":"2.0","id":4,"method":"nope"}"#);
    let parsed: serde_json::Value = serde_json::from_str(&unknown).expect("error is json");
    assert_eq!(parsed["error"]["code"], -32601);

    // Oversized line -> too-large error with null id, stream stays in sync.
    let big = "x".repeat(1_048_577);
    core.write_raw(&big);
    let mut response = String::new();
    core.stdout
        .read_line(&mut response)
        .expect("read error response");
    let parsed: serde_json::Value =
        serde_json::from_str(response.trim_end()).expect("error is json");
    assert_eq!(parsed["error"]["code"], -32000);
    assert_eq!(parsed["id"], serde_json::Value::Null);

    // Framing survived: the next request still works.
    let after = core.request(r#"{"jsonrpc":"2.0","id":5,"method":"ping"}"#);
    assert!(
        after.contains("pong"),
        "ping after oversized line was {after}"
    );

    drop(core);
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn core_exits_cleanly_when_stdin_closes() {
    let mut db_path = std::env::temp_dir();
    db_path.push(format!("xenon-stdio-exit-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&db_path);

    let mut child = Command::new(env!("CARGO_BIN_EXE_xenon-core"))
        .env("XENON_CORE_DB_PATH", &db_path)
        .env("XENON_CORE_DB_KEY", "cd".repeat(32))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn xenon-core");
    // Closing stdin immediately (before any request) must end the core
    // with exit code 0 (Architecture.md 4.2: core exits when stdin closes).
    drop(child.stdin.take());
    let status = child.wait().expect("wait after stdin closed");
    assert!(
        status.success(),
        "core must exit 0 when stdin closes, got {status:?}"
    );
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn master_password_wrap_round_trip_through_the_real_binary() {
    // Requires a usable OS keystore: Windows everywhere, Linux only when
    // CI started the headless gnome-keyring (XENON_FORCE_KEYSTORE_TEST=1).
    let forced = std::env::var("XENON_FORCE_KEYSTORE_TEST").is_ok();
    if !cfg!(windows) && !forced {
        return;
    }
    let unique = std::process::id();
    let mut db_path = std::env::temp_dir();
    db_path.push(format!("xenon-stdio-wrap-{unique}.db"));
    let _ = std::fs::remove_file(&db_path);
    // A per-test keystore account, so the real "core/db-key" profile
    // entry is never touched; it is removed again at the end.
    let account = format!("core/test-wrap-{unique}");

    // Run 1: the key is provided as a dev key and migrated to wrapped
    // storage under the master password.
    {
        let mut core = CoreProc::start_with(&db_path, &account, |command| {
            command
                .env("XENON_CORE_DB_KEY", "ab".repeat(32))
                .env("XENON_CORE_SETUP_MASTER_PASSWORD", "wrap test password");
        });
        let hello = core.request(r#"{"jsonrpc":"2.0","id":1,"method":"hello"}"#);
        assert!(
            hello.contains(r#""protocolVersion":1"#),
            "run 1 was {hello}"
        );
        core.request(
            r#"{"jsonrpc":"2.0","id":2,"method":"settings.set","params":{"key":"marker","value":"wrapped"}}"#,
        );
    } // stdin closes: core exits.

    // Run 2: no dev key; the wrapped entry is unwrapped with the password
    // and must open the very same database file.
    {
        let mut core = CoreProc::start_with(&db_path, &account, |command| {
            command.env("XENON_CORE_MASTER_PASSWORD", "wrap test password");
        });
        let hello = core.request(r#"{"jsonrpc":"2.0","id":1,"method":"hello"}"#);
        assert!(
            hello.contains(r#""protocolVersion":1"#),
            "run 2 was {hello}"
        );
        let get = core.request(
            r#"{"jsonrpc":"2.0","id":2,"method":"settings.get","params":{"key":"marker"}}"#,
        );
        assert!(
            get.contains(r#""value":"wrapped""#),
            "the unwrapped key must open the same database, got {get}"
        );
    }

    // Run 3: without the correct password the wrapped entry is refused
    // and the core exits with a failure code before serving anything.
    {
        let mut core = CoreProc::start_with(&db_path, &account, |command| {
            command.env("XENON_CORE_MASTER_PASSWORD", "wrong password");
        });
        let status = core.child.wait().expect("wait for refusal");
        assert!(
            !status.success(),
            "a wrong password must be a failure exit, got {status:?}"
        );
    }

    // Remove the per-test keystore entry.
    use xenon_core::keystore::{KeyStore, OsKeyStore};
    OsKeyStore.delete(&account).expect("delete test entry");
    let _ = std::fs::remove_file(&db_path);
}
