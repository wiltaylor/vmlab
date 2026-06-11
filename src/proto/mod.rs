//! CLI ↔ daemon wire protocol (PRD §3.1): JSON lines over unix domain
//! sockets, supporting request/response, a subscribable event stream, and
//! streamed output for long operations. Supervisor ↔ lab-daemon control uses
//! the same protocol.

pub mod client;
pub mod server;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One wire message, one JSON line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Client → server command.
    Req { id: u64, cmd: String, #[serde(default)] args: Value },
    /// Server → client final answer for `id`.
    Resp {
        id: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        ok: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        err: Option<String>,
    },
    /// Server → client incremental output for a long-running `id`
    /// (template builds, provision runs). Always followed eventually by a
    /// `Resp` with the same id.
    Stream { id: u64, chunk: String },
    /// Server → client broadcast event (after `subscribe`).
    Event { event: String, data: Value },
}

/// A structured daemon event (PRD §8.1) as carried on the wire and in logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event: String,
    /// Lab the event belongs to; empty for host-scoped events.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub lab: String,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub data: Value,
    pub ts: chrono::DateTime<chrono::Utc>,
}

impl Event {
    pub fn new(event: impl Into<String>, lab: impl Into<String>, data: Value) -> Self {
        Self { event: event.into(), lab: lab.into(), data, ts: chrono::Utc::now() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProtoError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("{0}")]
    Remote(String),
    #[error("connection closed")]
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_round_trip() {
        let m = Message::Req { id: 7, cmd: "status".into(), args: serde_json::json!({"a": 1}) };
        let line = serde_json::to_string(&m).unwrap();
        let back: Message = serde_json::from_str(&line).unwrap();
        match back {
            Message::Req { id, cmd, args } => {
                assert_eq!(id, 7);
                assert_eq!(cmd, "status");
                assert_eq!(args["a"], 1);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn resp_omits_empty_sides() {
        let m = Message::Resp { id: 1, ok: Some(serde_json::json!(true)), err: None };
        let line = serde_json::to_string(&m).unwrap();
        assert!(!line.contains("err"));
    }
}
