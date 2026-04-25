//! Actor profile — what the actor IS, independent of how it's held.

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ActorProfile {
    /// Optional id hint. ULID-shape values are honoured; anything else
    /// is ignored and a fresh ULID is minted. The daemon is the
    /// authoritative minter.
    pub actor_id_hint: Option<String>,

    pub kind: ActorKind,
    pub transport: Transport,

    /// `None` = persist as long as the connection is alive (chat,
    /// shells, services). `Some(true)` = one-shot (pipeline steps).
    pub one_shot: bool,

    pub bin: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub work_dir: PathBuf,

    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub provider: Option<String>,
    pub parent_actor_id: Option<String>,

    /// Optional override for log file location. Defaults to
    /// `<global_dir>/logs/actors/{actor_id}.log`.
    pub log_dir: Option<PathBuf>,
}

impl ActorProfile {
    pub fn resolve_actor_id(&self) -> String {
        match self.actor_id_hint.as_deref() {
            Some(id) if is_ulid_shape(id) => id.to_string(),
            _ => crate::gen_ulid(),
        }
    }

    pub fn persistence_label(&self) -> &'static str {
        if self.one_shot { "one-shot" } else { "long-lived" }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorKind {
    Agent,
    PipelineStep,
    Shell,
    Service,
}

impl ActorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActorKind::Agent => "agent",
            ActorKind::PipelineStep => "pipeline-step",
            ActorKind::Shell => "shell",
            ActorKind::Service => "service",
        }
    }
}

/// How a child process is held. Today only PTY. Headless, Tmux, and
/// Server-session arrive as the migration progresses; the enum gates
/// them at the call site so we don't have to break existing consumers
/// when a new transport lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Pty,
}

impl Transport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Transport::Pty => "pty",
        }
    }
}

fn is_ulid_shape(s: &str) -> bool {
    s.len() == 26 && s.bytes().all(|b| {
        matches!(
            b,
            b'0'..=b'9' | b'A'..=b'H' | b'J' | b'K' | b'M'..=b'N' | b'P'..=b'T' | b'V'..=b'Z'
        )
    })
}
