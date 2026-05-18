//! Sidecar wire protocol — request/response/event types.
//!
//! All messages are JSON. Requests are HTTP POST to `/v1/<verb>`. Events
//! stream from `GET /v1/events` as Server-Sent Events with one JSON object
//! per `data:` line. Health probe is `GET /v1/health` → `200 OK` with
//! `{"status":"ok","version":"<sidecar semver>"}`.
//!
//! Endpoints intentionally mirror the trait surface in `ship_substrate::isolation`
//! so the same types serialize to either local transport or cloud HTTPS.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartRequest {
    pub label: String,
    pub policy: ship_substrate::isolation::SandboxPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartResponse {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecRequest {
    pub session_id: String,
    pub bin: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResponse {
    pub exec_id: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRequest {
    pub exec_id: String,
    pub signal: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdinRequest {
    pub exec_id: String,
    pub data_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountRequest {
    pub session_id: String,
    pub mount: ship_substrate::isolation::FsMount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdateRequest {
    pub session_id: String,
    pub policy: ship_substrate::isolation::SandboxPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortPublishRequest {
    pub session_id: String,
    pub internal_port: u16,
    pub external_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortPublishResponse {
    pub external_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SidecarEvent {
    Stdout {
        exec_id: String,
        data_b64: String,
    },
    Stderr {
        exec_id: String,
        data_b64: String,
    },
    Exited {
        exec_id: String,
        code: Option<i32>,
        signal: Option<u8>,
    },
    PolicyDenied {
        session_id: String,
        kind: String,
        detail: String,
    },
    NetworkBlocked {
        session_id: String,
        host: String,
    },
    SidecarShutdown {
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn session_start_roundtrips() {
        let req = SessionStartRequest {
            label: "demo".into(),
            policy: ship_substrate::isolation::SandboxPolicy::workspace_default(PathBuf::from("/tmp/x")),
        };
        let s = serde_json::to_string(&req).unwrap();
        let back: SessionStartRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.label, "demo");
    }

    #[test]
    fn exited_event_serializes_with_type_tag() {
        let ev = SidecarEvent::Exited {
            exec_id: "e1".into(),
            code: Some(0),
            signal: None,
        };
        let s = serde_json::to_string(&ev).unwrap();
        assert!(s.contains("\"type\":\"exited\""));
    }

    #[test]
    fn exec_request_carries_env_and_cwd() {
        let req = ExecRequest {
            session_id: "s1".into(),
            bin: "/usr/bin/claude".into(),
            args: vec!["-p".into()],
            env: HashMap::from([("X".into(), "y".into())]),
            cwd: "/work".into(),
        };
        let s = serde_json::to_string(&req).unwrap();
        let back: ExecRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.env.get("X").unwrap(), "y");
    }
}
