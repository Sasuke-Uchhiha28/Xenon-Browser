//! JSON-RPC 2.0 over stdio (Architecture.md 4.2): one message per line,
//! 1 MB line limit, chunking helper for larger payloads, and a versioned
//! `hello` handshake. The dispatch logic lives in [`Service`] so tests can
//! drive it without real pipes; the binary wires it to stdin/stdout.

pub mod framing;

use crate::settings::Settings;
use crate::{chunked, CoreError, Result};
use serde_json::{json, Value};

/// Protocol version of the core RPC surface. Bumped on breaking change;
/// hosts check it during `hello`.
pub const PROTOCOL_VERSION: i64 = 1;

/// Maximum bytes per line, from Architecture.md 4.2.
pub const MAX_LINE_BYTES: usize = 1024 * 1024;

/// JSON-RPC error codes used by core.
pub mod codes {
    pub const PARSE: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL: i32 = -32603;
    pub const TOO_LARGE: i32 = -32000;
}

/// Request dispatcher over the opened database.
pub struct Service {
    settings: Settings,
}

impl Service {
    /// Build a service on top of an opened, migrated database.
    pub fn new(settings: Settings) -> Service {
        Service { settings }
    }

    /// Handle one request line; `None` for notifications (no `id`).
    /// Never panics on bad input: every failure becomes an error response.
    pub fn handle_line(&mut self, line: &str) -> Option<String> {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(err) => {
                return Some(error_response(
                    &Value::Null,
                    codes::PARSE,
                    &format!("parse error: {err}"),
                ))
            }
        };
        self.handle_request(&parsed)
    }

    fn handle_request(&mut self, request: &Value) -> Option<String> {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let notification = request.get("id").is_none();
        if request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Some(error_response(
                &id,
                codes::INVALID_REQUEST,
                "jsonrpc must be exactly \"2.0\"",
            ));
        }
        let method = request.get("method").and_then(Value::as_str);
        let params = request.get("params").cloned().unwrap_or(json!({}));
        let outcome = match method {
            Some("hello") => Ok(self.hello()),
            Some("ping") => Ok(json!("pong")),
            Some("settings.get") => self.settings_get(&params),
            Some("settings.set") => self.settings_set(&params),
            Some("settings.delete") => self.settings_delete(&params),
            Some("settings.list") => self.settings_list(),
            Some(_) => Err(rpc_error(codes::METHOD_NOT_FOUND, "unknown method")),
            None => Err(rpc_error(codes::INVALID_REQUEST, "missing method")),
        };
        if notification {
            return None;
        }
        Some(match outcome {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string(),
            Err(CoreError::Rpc { code, message }) => error_response(&id, code, &message),
            Err(err) => error_response(&id, codes::INTERNAL, &err.to_string()),
        })
    }

    /// Handshake: who we are, the protocol version, and the limits hosts
    /// need to frame and chunk correctly.
    fn hello(&self) -> Value {
        json!({
            "name": "xenon-core",
            "protocolVersion": PROTOCOL_VERSION,
            "implementation": env!("CARGO_PKG_VERSION"),
            "maxLineBytes": MAX_LINE_BYTES,
            "chunkMaxPartBytes": chunked::MAX_PART_BYTES,
        })
    }

    fn require_key<'a>(
        &self,
        params: &'a Value,
        name: &str,
    ) -> std::result::Result<&'a str, CoreError> {
        params.get(name).and_then(Value::as_str).ok_or_else(|| {
            rpc_error(
                codes::INVALID_PARAMS,
                &format!("missing string param {name}"),
            )
        })
    }

    fn settings_get(&mut self, params: &Value) -> Result<Value> {
        let key = self.require_key(params, "key")?;
        let value = self.settings.get(key)?;
        Ok(json!({ "value": value }))
    }

    fn settings_set(&mut self, params: &Value) -> Result<Value> {
        let key = self.require_key(params, "key")?;
        let value = self.require_key(params, "value")?;
        self.settings.set(key, value)?;
        Ok(json!({ "ok": true }))
    }

    fn settings_delete(&mut self, params: &Value) -> Result<Value> {
        let key = self.require_key(params, "key")?;
        let deleted = self.settings.delete(key)?;
        Ok(json!({ "deleted": deleted }))
    }

    fn settings_list(&mut self) -> Result<Value> {
        let items = self.settings.list()?;
        let items: Vec<Value> = items
            .into_iter()
            .map(|(key, value)| json!({"key": key, "value": value}))
            .collect();
        Ok(json!({ "items": items }))
    }
}

fn rpc_error(code: i32, message: &str) -> CoreError {
    CoreError::Rpc {
        code,
        message: message.to_string(),
    }
}

fn error_response(id: &Value, code: i32, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, DbKey};

    fn service(name: &str) -> Service {
        let mut path = std::env::temp_dir();
        path.push(format!("xenon-rpc-{name}-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut db = Database::open(&path, &DbKey::generate().expect("key")).expect("open");
        db.migrate().expect("migrate");
        Service::new(Settings::new(db))
    }

    fn call(service: &mut Service, request: &str) -> Option<Value> {
        service
            .handle_line(request)
            .map(|line| serde_json::from_str(&line).expect("response is json"))
    }

    #[test]
    fn hello_reports_protocol_and_limits() {
        let mut service = service("hello");
        let response =
            call(&mut service, r#"{"jsonrpc":"2.0","id":1,"method":"hello"}"#).expect("response");
        assert_eq!(response["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(response["result"]["maxLineBytes"], MAX_LINE_BYTES as u64);
        assert_eq!(response["id"], 1);
    }

    #[test]
    fn settings_round_trip_over_rpc() {
        let mut service = service("settings");
        let response = call(
            &mut service,
            r#"{"jsonrpc":"2.0","id":2,"method":"settings.set","params":{"key":"engine","value":"gecko"}}"#,
        )
        .expect("response");
        assert_eq!(response["result"]["ok"], true);
        let response = call(
            &mut service,
            r#"{"jsonrpc":"2.0","id":3,"method":"settings.get","params":{"key":"engine"}}"#,
        )
        .expect("response");
        assert_eq!(response["result"]["value"], "gecko");
        let response = call(
            &mut service,
            r#"{"jsonrpc":"2.0","id":4,"method":"settings.list"}"#,
        )
        .expect("response");
        assert_eq!(
            response["result"]["items"].as_array().expect("items").len(),
            1
        );
    }

    #[test]
    fn parse_error_returns_id_null() {
        let mut service = service("parse");
        let response = call(&mut service, "{not json").expect("response");
        assert_eq!(response["error"]["code"], codes::PARSE);
        assert_eq!(response["id"], Value::Null);
    }

    #[test]
    fn wrong_protocol_version_is_rejected() {
        let mut service = service("version");
        let response =
            call(&mut service, r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#).expect("response");
        assert_eq!(response["error"]["code"], codes::INVALID_REQUEST);
    }

    #[test]
    fn unknown_method_and_missing_params() {
        let mut service = service("errors");
        let response =
            call(&mut service, r#"{"jsonrpc":"2.0","id":1,"method":"nope"}"#).expect("response");
        assert_eq!(response["error"]["code"], codes::METHOD_NOT_FOUND);
        let response = call(
            &mut service,
            r#"{"jsonrpc":"2.0","id":2,"method":"settings.get"}"#,
        )
        .expect("response");
        assert_eq!(response["error"]["code"], codes::INVALID_PARAMS);
    }

    #[test]
    fn notifications_get_no_response() {
        let mut service = service("notify");
        assert!(service
            .handle_line(
                r#"{"jsonrpc":"2.0","method":"settings.set","params":{"key":"a","value":"b"}}"#
            )
            .is_none());
        assert_eq!(
            service.settings.get("a").expect("get"),
            Some("b".into()),
            "notifications still execute"
        );
    }
}
