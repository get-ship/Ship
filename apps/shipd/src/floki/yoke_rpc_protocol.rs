//! Hand-rolled harness session RPC wire types (v1).
//!
//! Mirrors `bridge::rpc::protocol` so shipd can speak NDJSON UDS to
//! `bridge serve` without path-depending the bridge crate. Contract:
//! monorepo `bridge/crates/bridge/docs/HARNESS_SESSION_RPC.md`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Wire protocol version (must match bridge harness RPC v1).
pub const PROTOCOL_VERSION: u32 = 1;

// ── Client → server ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "session.start")]
    SessionStart(SessionStart),
    #[serde(rename = "session.cancel")]
    SessionCancel(SessionCancel),
    #[serde(rename = "session.steer")]
    SessionSteer(SessionSteer),
    #[serde(rename = "op.result")]
    OpResult(OpResult),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientEnvelope {
    pub v: u32,
    #[serde(flatten)]
    pub message: ClientMessage,
}

impl ClientEnvelope {
    pub fn new(message: ClientMessage) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            message,
        }
    }

    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }

    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStart {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub agent_id: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCancel {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSteer {
    pub session_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpResult {
    pub id: String,
    pub session_id: String,
    #[serde(default)]
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Server → client ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "session.started")]
    SessionStarted(SessionStarted),
    #[serde(rename = "session.completed")]
    SessionCompleted(SessionCompleted),
    #[serde(rename = "session.failed")]
    SessionFailed(SessionFailed),
    #[serde(rename = "session.cancelled")]
    SessionCancelled(SessionCancelled),
    #[serde(rename = "session.output")]
    SessionOutput(SessionOutput),
    #[serde(rename = "op.call")]
    OpCall(OpCall),
    #[serde(rename = "error")]
    Error(RpcError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerEnvelope {
    pub v: u32,
    #[serde(flatten)]
    pub message: ServerMessage,
}

impl ServerEnvelope {
    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStarted {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCompleted {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFailed {
    pub session_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCancelled {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionOutput {
    pub session_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpCall {
    pub id: String,
    pub session_id: String,
    pub op: String,
    #[serde(default)]
    pub args: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_start_roundtrip() {
        let env = ClientEnvelope::new(ClientMessage::SessionStart(SessionStart {
            session_id: Some("s1".into()),
            agent_id: "agent-a".into(),
            prompt: "hello".into(),
            provider: Some("nvidia".into()),
            model: Some("meta/llama-3.1-8b-instruct".into()),
            max_turns: Some(3),
            workspace: Some("/tmp/ws".into()),
        }));
        let line = env.to_json_line().expect("encode");
        assert!(line.ends_with('\n'));
        let back = ClientEnvelope::from_json_line(&line).expect("decode");
        assert_eq!(back, env);
        assert_eq!(back.v, PROTOCOL_VERSION);
        let v: Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "session.start");
        assert_eq!(v["v"], 1);
        assert_eq!(v["prompt"], "hello");
    }

    #[test]
    fn session_start_minimal_omits_optionals() {
        let env = ClientEnvelope::new(ClientMessage::SessionStart(SessionStart {
            session_id: None,
            agent_id: "a".into(),
            prompt: "p".into(),
            provider: None,
            model: None,
            max_turns: None,
            workspace: None,
        }));
        let s = serde_json::to_string(&env).unwrap();
        assert!(!s.contains("session_id"));
        assert!(!s.contains("provider"));
        let back: ClientEnvelope = serde_json::from_str(&s).unwrap();
        assert_eq!(back, env);
    }

    #[test]
    fn denied_op_result_roundtrip() {
        let env = ClientEnvelope::new(ClientMessage::OpResult(OpResult {
            id: "op-1".into(),
            session_id: "s1".into(),
            ok: false,
            result: None,
            error: Some("denied".into()),
        }));
        let line = env.to_json_line().unwrap();
        let back = ClientEnvelope::from_json_line(&line).unwrap();
        assert_eq!(back, env);
        let v: Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "op.result");
        assert_eq!(v["ok"], false);
    }

    #[test]
    fn server_lifecycle_roundtrip() {
        let msgs = [
            ServerEnvelope {
                v: PROTOCOL_VERSION,
                message: ServerMessage::SessionStarted(SessionStarted {
                    session_id: "s1".into(),
                    agent_id: Some("a".into()),
                }),
            },
            ServerEnvelope {
                v: PROTOCOL_VERSION,
                message: ServerMessage::SessionOutput(SessionOutput {
                    session_id: "s1".into(),
                    text: "pong".into(),
                }),
            },
            ServerEnvelope {
                v: PROTOCOL_VERSION,
                message: ServerMessage::SessionCompleted(SessionCompleted {
                    session_id: "s1".into(),
                }),
            },
            ServerEnvelope {
                v: PROTOCOL_VERSION,
                message: ServerMessage::SessionFailed(SessionFailed {
                    session_id: "s1".into(),
                    message: "no credentials".into(),
                }),
            },
            ServerEnvelope {
                v: PROTOCOL_VERSION,
                message: ServerMessage::OpCall(OpCall {
                    id: "op-9".into(),
                    session_id: "s1".into(),
                    op: "collections.read".into(),
                    args: serde_json::json!({"id": "x"}),
                }),
            },
            ServerEnvelope {
                v: PROTOCOL_VERSION,
                message: ServerMessage::Error(RpcError {
                    id: None,
                    message: "bad json".into(),
                }),
            },
        ];
        for m in msgs {
            let mut line = serde_json::to_string(&m).unwrap();
            line.push('\n');
            let back = ServerEnvelope::from_json_line(&line).unwrap();
            assert_eq!(back, m);
            assert_eq!(back.v, PROTOCOL_VERSION);
        }
    }

    #[test]
    fn server_type_names_are_dotted() {
        let env = ServerEnvelope {
            v: PROTOCOL_VERSION,
            message: ServerMessage::SessionStarted(SessionStarted {
                session_id: "s".into(),
                agent_id: None,
            }),
        };
        let v: Value = serde_json::from_str(&serde_json::to_string(&env).unwrap()).unwrap();
        assert_eq!(v["type"], "session.started");
    }
}
