//! Core types for the sandbox registry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

use ship_isolation::tier::IsolationProfile;
use ship_substrate::isolation::SandboxOwner;

/// Opaque, stable id for a sandbox. Minted by the registry; survives
/// shipd restart (Chunk F adds the persistence).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(pub String);

impl SandboxId {
    pub fn new() -> Self {
        Self(format!("sbx_{}", nanoid::nanoid!(10)))
    }
}

/// Human-readable sandbox name within a project (`"default"`,
/// `"untrusted"`, `"gpu"`, ...). Unique within the project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxName(pub String);

impl SandboxName {
    pub fn default_name() -> Self {
        Self("default".into())
    }
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// How long a sandbox lives.
///
/// `Idle { timeout }` — the project-default. Lazy start on first
/// actor; stays alive while any actor is running; reaped after
/// `timeout` of zero actors.
///
/// `PerRun` — ephemeral. Started for one pipeline run or one agent
/// invocation; reaped when that run finishes regardless of activity.
///
/// `Manual` — never reaped automatically. Operator-controlled
/// lifetime. Tier 0/1 in-process default; rarely used for higher tiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SandboxLifetime {
    /// Idle reap after the timeout (expressed as seconds for
    /// serde simplicity; humans configure as minutes in settings).
    Idle { timeout_secs: u64 },
    PerRun,
    Manual,
}

impl SandboxLifetime {
    pub fn idle_secs(timeout_secs: u64) -> Self {
        Self::Idle { timeout_secs }
    }

    pub fn idle_duration(d: Duration) -> Self {
        Self::Idle {
            timeout_secs: d.as_secs(),
        }
    }
}

impl Default for SandboxLifetime {
    fn default() -> Self {
        // 15 minute idle reap — per runtime-architecture.md §6
        Self::Idle {
            timeout_secs: 15 * 60,
        }
    }
}

/// State machine for a sandbox.
///
/// `Pending` — registered, not yet started. Lazy startup target.
/// `Running` — start succeeded; ready to accept actors.
/// `Idle` — no actors currently running; idle reap timer armed.
/// `Reaping` — reap in progress; reject new actor spawns.
/// `Stopped` — sandbox is gone; registry entry awaiting cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxState {
    Pending,
    Running,
    Idle,
    Reaping,
    Stopped,
}

/// Where the supervisor for this sandbox lives. Populated when the
/// sandbox transitions Pending → Running.
///
/// Tier 0/1: `InProcess` — supervisor is a function call away in shipd.
/// Tier 2/3 (later): `Remote { addr, scheme }` — ConnectRPC client
/// against the in-sandbox runtime's v1 listener.
///
/// Today the registry only stores the placeholder; Chunk C wires the
/// actual runtime endpoint when it lands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeEndpoint {
    InProcess,
    Remote { addr: String, scheme: String },
}

/// One sandbox. Holds its identity, isolation profile, lifecycle
/// state, and the actors currently assigned to it.
///
/// `sandbox_owner` holds the optional backend lifecycle binding —
/// the `Arc<SandboxOwner>` whose `Drop` fires the isolator's
/// `shutdown()`. Populated for sandboxes whose backend needs
/// teardown (Docker / Youki / etc.); `None` for in-process /
/// no-backend sandboxes (Tier 0/1).
///
/// Reaping drops the Arc, which fires shutdown on the tokio
/// runtime — that's how the registry's idle-reap loop translates
/// to "docker rm" without ship-platform depending on Docker.
#[derive(Clone)]
pub struct Sandbox {
    pub id: SandboxId,
    pub project: String,
    pub name: SandboxName,
    pub isolation: IsolationProfile,
    pub lifetime: SandboxLifetime,
    pub runtime_endpoint: Option<RuntimeEndpoint>,
    pub actors: Vec<String>,
    pub state: SandboxState,
    pub created_at: DateTime<Utc>,
    /// When the sandbox last had zero actors. `None` while any actor
    /// is running. Used by the reaper to decide if `Idle{timeout}`
    /// has elapsed.
    pub idle_since: Option<DateTime<Utc>>,
    pub sandbox_owner: Option<Arc<SandboxOwner>>,
}

impl std::fmt::Debug for Sandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sandbox")
            .field("id", &self.id)
            .field("project", &self.project)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("actors", &self.actors)
            .field("has_owner", &self.sandbox_owner.is_some())
            .finish()
    }
}

impl Sandbox {
    /// Build a fresh sandbox in `Pending` state. Lifetime defaults to
    /// `Idle { 15min }`; runtime endpoint is `None` until started.
    pub fn new(
        project: impl Into<String>,
        name: SandboxName,
        isolation: IsolationProfile,
    ) -> Self {
        Self {
            id: SandboxId::new(),
            project: project.into(),
            name,
            isolation,
            lifetime: SandboxLifetime::default(),
            runtime_endpoint: None,
            actors: Vec::new(),
            state: SandboxState::Pending,
            created_at: Utc::now(),
            idle_since: None,
            sandbox_owner: None,
        }
    }

    pub fn with_lifetime(mut self, lifetime: SandboxLifetime) -> Self {
        self.lifetime = lifetime;
        self
    }

    /// Idle reap is eligible when state is Idle, no actors are
    /// attached, and the lifetime is Idle with the timeout elapsed.
    pub fn is_ready_to_reap(&self, now: DateTime<Utc>) -> bool {
        if self.state != SandboxState::Idle || !self.actors.is_empty() {
            return false;
        }
        let SandboxLifetime::Idle { timeout_secs } = self.lifetime else {
            return false;
        };
        match self.idle_since {
            Some(since) => {
                let elapsed = now.signed_duration_since(since).to_std().unwrap_or_default();
                elapsed >= Duration::from_secs(timeout_secs)
            }
            None => false,
        }
    }
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SandboxError {
    #[error("no sandbox with id '{0}'")]
    NotFound(String),
    #[error("sandbox '{name}' already exists in project '{project}'")]
    DuplicateName { project: String, name: String },
    #[error(
        "sandbox '{id}' is in state {state:?} which cannot accept actor '{actor}'"
    )]
    InvalidStateForActor {
        id: String,
        state: SandboxState,
        actor: String,
    },
    #[error("sandbox '{id}' is in state {state:?} which cannot transition to {target:?}")]
    InvalidStateTransition {
        id: String,
        state: SandboxState,
        target: SandboxState,
    },
}
