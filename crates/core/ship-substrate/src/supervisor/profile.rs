//! Actor profile — what the actor IS, independent of how it's held.
//!
//! # Isolation
//!
//! No explicit isolation axis on `ActorProfile` today — we trust each harness's
//! own sandboxing (Claude Code's permission system, Codex/Cursor equivalents).
//! When explicit isolation lands (bubblewrap, seatbelt, docker, future VMs),
//! it will be an orthogonal axis on this struct alongside `transport` and the
//! kernel-scope fields, not folded into either. Networking + IPC back to shipd
//! and credential passing are the open design questions blocking that work.

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
    pub harness: Option<String>,
    pub parent_actor_id: Option<String>,

    /// Optional override for log file location. Defaults to
    /// `<global_dir>/logs/actors/{actor_id}.log`.
    pub log_dir: Option<PathBuf>,

    /// Kernel subscription namespaces for this actor. Empty = no
    /// broadcast subscriptions (the actor still receives directed
    /// events via target_actor_id). Agents typically subscribe to
    /// studio.chat_turn / studio.reply / inbox.* / human.* /
    /// mesh.message; shells and pipelines often want nothing.
    pub subscribe_namespaces: Vec<String>,
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
    /// A Tauri shell window hosting a space (process-per-app). Self-registers
    /// with the daemon on launch; the launcher consults the actor registry for
    /// single-instance and the supervisor pid-checks for liveness. A *lifecycle*
    /// actor — identity/pid/lifecycle only, no mailbox (the webview gets events
    /// over its Connect stream, not an actor mailbox).
    Window,
}

impl ActorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActorKind::Agent => "agent",
            ActorKind::PipelineStep => "pipeline-step",
            ActorKind::Shell => "shell",
            ActorKind::Window => "window",
        }
    }
}

/// How a child process is held. Today only PTY. Headless, Tmux, and
/// Server-session arrive as the migration progresses; the enum gates
/// them at the call site so we don't have to break existing consumers
/// when a new transport lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    /// Pseudo-terminal — long-lived interactive children. Used by
    /// shells and channels-capable agents (Claude). Output broadcasts
    /// raw bytes; input writes through the master fd.
    Pty,
    /// Headless one-shot — child with piped stdout/stderr, no PTY.
    /// Used by per-turn agent runs (codex, opencode, cursor)
    /// and pipeline jobs. Output broadcasts merged stdout+stderr;
    /// input is not supported (caller writes via stdin pipe at spawn
    /// time via `ActorProfile.stdin`). Terminates when the child
    /// exits, then auto-deregisters from its registry.
    Headless,
}

impl Transport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Transport::Pty => "pty",
            Transport::Headless => "headless",
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
