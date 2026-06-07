//! `runtime::supervisor` — the single spawn primitive.
//!
//! All supervised processes flow through `spawn_actor(profile)` and one
//! transport trait. The legacy `spawn_command` path in
//! `apps/shipd/src/sessions/lifecycle.rs` was removed in mr-05; shell
//! sessions go via `shell_actor::spawn_shell`, agent sessions via
//! `agent_actor::*`, and pipeline jobs via `ProcessExecutor` — every
//! one of them eventually calls `spawn_actor` here. Currently exposed:
//!
//!   - [`Transport`] — how a child process is held. PTY today; headless,
//!     tmux, server-session arrive as the migration progresses.
//!   - [`ActorProfile`] — what the actor IS. Kind, transport, agent id,
//!     harness, env, work dir. The orchestrator wires Transport +
//!     metadata + lifecycle events together.
//!   - [`spawn_actor`] — the orchestrator. Mints the actor id, opens
//!     the transport, holds the child, emits `actor.created`, returns
//!     a handle the caller can write input to / observe output from.
//!
//! The mesh is the source of truth for "alive and reachable." The
//! supervisor's only job at runtime is owning OS-level child processes
//! and translating their lifecycle into kernel events.

pub mod process_stats;
pub mod profile;
pub mod runtime;
pub mod transport;

pub use profile::{ActorKind, ActorProfile, Transport as TransportKind};
pub use runtime::{ActorRuntime, InProcessActorRuntime};
pub use transport::{
    ChildBackend, ChildHandle, HeadlessChildBackend, HeadlessTransport, PtyChildBackend,
    PtyTransport, Transport,
};

use std::path::Path;
use std::sync::OnceLock;

/// Optional resolver that maps a working-directory path to a project id.
/// Installed by the platform crate (`ship_platform::projects`); used by
/// `spawn_actor` to attribute the emitted `actor.created` event to a
/// project. When unset, space_id is left None.
type ProjectIdResolver = Box<dyn Fn(&Path) -> Option<String> + Send + Sync>;
static PROJECT_ID_RESOLVER: OnceLock<ProjectIdResolver> = OnceLock::new();

pub fn install_space_id_resolver(resolver: ProjectIdResolver) {
    let _ = PROJECT_ID_RESOLVER.set(resolver);
}

fn resolve_space_id(path: &Path) -> Option<String> {
    PROJECT_ID_RESOLVER.get().and_then(|f| f(path))
}

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

use crate::events::Mailbox;

/// Stash of Mailboxes for actors that substrate pre-registered with the
/// kernel router before any MCP child connected. The MCP `actor_spawn`
/// HTTP endpoint drains this on its refine path so the per-actor SSE
/// stream (`/mesh/events/{actor_id}`) has a receiver to drain. Cleared
/// when the consumer takes it.
static PRE_REGISTERED_MAILBOXES: OnceLock<StdMutex<HashMap<String, Mailbox>>> = OnceLock::new();

fn mailbox_stash() -> &'static StdMutex<HashMap<String, Mailbox>> {
    PRE_REGISTERED_MAILBOXES.get_or_init(|| StdMutex::new(HashMap::new()))
}

/// Remove and return the Mailbox stashed by `spawn_actor` for `actor_id`.
/// Returns `None` if there isn't one (already taken, or the actor was
/// registered outside substrate).
pub fn take_pre_registered_mailbox(actor_id: &str) -> Option<Mailbox> {
    mailbox_stash().lock().ok()?.remove(actor_id)
}

/// leak-193 diagnostics: count of mailboxes pre-registered by `spawn_actor`
/// but not yet drained by an MCP refine. A monotonic climb here means
/// Agent-kind spawns whose MCP child never connected are stranding kernel
/// mailboxes.
pub fn pre_registered_mailbox_count() -> usize {
    mailbox_stash().lock().map(|s| s.len()).unwrap_or(0)
}

/// Drop any stashed Mailbox for `actor_id` without consuming it. Called
/// when an actor is being torn down before its consumer has arrived.
pub fn discard_pre_registered_mailbox(actor_id: &str) {
    if let Ok(mut stash) = mailbox_stash().lock() {
        let _ = stash.remove(actor_id);
    }
}

/// A live spawn — the supervisor owns the child process; the caller
/// holds this handle for input/observation/lifecycle. Drop semantics:
/// dropping does NOT kill the child; explicit `stop()` does. This
/// matches the `lifecycle.rs::PtySession` contract so migration is a
/// drop-in replacement.
///
/// `sandbox` carries the isolation owner when the actor runs inside
/// one. It's an `Arc` so cloning the handle is cheap; the underlying
/// `SandboxOwner::Drop` fires the isolator's `shutdown()` when the
/// last clone goes away. Stays `None` for unsandboxed actors.
#[derive(Clone)]
pub struct ActorHandle {
    pub actor_id: String,
    pub kind: ActorKind,
    pub child: Arc<ChildHandle>,
    pub sandbox: Option<Arc<crate::isolation::SandboxOwner>>,
}

impl ActorHandle {
    pub fn id(&self) -> &str {
        &self.actor_id
    }

    pub fn write_input(&self, bytes: &[u8]) -> Result<()> {
        self.child.write_input(bytes)
    }

    /// Subscribe to raw byte output from the child. Each subscriber
    /// gets a fresh broadcast receiver. Used by xterm.js attaches and
    /// stream-json parsers.
    pub fn subscribe_output(&self) -> tokio::sync::broadcast::Receiver<Vec<u8>> {
        self.child.subscribe_output()
    }

    pub fn stop(&self) -> Result<()> {
        self.child.stop()
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.pid()
    }

    pub fn is_alive(&self) -> bool {
        self.child.is_alive()
    }

    pub fn try_exit(&self) -> Option<i32> {
        self.child.try_exit()
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.child.resize(cols, rows)
    }
}

/// Spawn an actor according to `profile`. The daemon mints the actor
/// id (or `profile.actor_id_hint` is honoured if it's a ULID), the
/// transport opens the child, and `actor.created` is emitted with rich
/// metadata. Returns the handle the caller drives.
///
/// ALL actor kinds are pre-registered with the kernel router before the
/// child process starts. This removes the load-bearing dependency on the
/// MCP child for Agent visibility — previously, if the MCP child failed
/// to start (port conflict, crash, network blip), the agent's OS process
/// was running but invisible to the kernel and mesh. Now substrate is
/// the single source of "registered." For agents, the MCP child later
/// refines the config (subscribe_namespaces, namespace) via the
/// `actor_spawn` HTTP endpoint, which detects an existing actor and
/// calls `refine_actor` instead of re-registering.
///
/// Side effects: kernel actor registered, child process started, kernel
/// actor event emitted. On error: no child process is left running,
/// error returned.
/// Rewrite an agent profile's `bin`/`args` to spawn inside the agent's process
/// container (`ship_host::containerize`). No-op for non-agent profiles and on
/// hosts without a container backend. Stdio/env/cwd are untouched — only the
/// program and its leading args change (Linux: a `systemd-run --scope` prefix).
fn containerize_agent(profile: ActorProfile) -> ActorProfile {
    let Some(agent_id) = profile.agent_id.clone() else {
        return profile;
    };
    let (bin, args) = ship_host::host().containerize(&agent_id, &profile.bin, &profile.args);
    if bin != profile.bin {
        tracing::info!(agent_id = %agent_id, "supervisor: spawning agent into its process container");
    }
    ActorProfile { bin, args, ..profile }
}

pub fn spawn_actor(profile: ActorProfile) -> Result<ActorHandle> {
    let actor_id = profile.resolve_actor_id();

    // Place agent processes in their own kernel process container so the daemon
    // can identify them by peer credentials over the host socket (the inverse
    // of `peer_identity::resolve`). Keyed on `agent_id`: any spawn that belongs
    // to an agent gets containerized, regardless of ActorKind. Best-effort — on
    // a host with no container backend `containerize` returns the command
    // unchanged and the process resolves as the operator.
    let profile = containerize_agent(profile);

    // Register with the kernel router so the actor appears on the
    // mesh (kernel.list_actors()) — the source-of-truth contract.
    // For Agent kind, this is a hard error: a spawn that can't be
    // observed is worse than no spawn — return without starting the
    // child so the caller sees the failure. For other kinds we keep
    // the historical best-effort warning so a missing global kernel
    // (e.g. in tests) doesn't fail their spawn.
    let is_agent = profile.kind == ActorKind::Agent;
    if let Some(kr) = crate::events::kernel_router() {
        let kernel_config = crate::events::ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec!["".to_string()],
            read_namespaces: vec!["agent.".to_string()],
            subscribe_namespaces: profile.subscribe_namespaces.clone(),
            see_directed_events: false,
        };
        let actor_id_for_kr = actor_id.clone();
        let res: Result<()> = crate::db::block_on_anyhow(async move {
            let mut kernel = kr.lock().await;
            if !kernel.has_actor(&actor_id_for_kr) {
                let (_store, mailbox) = kernel
                    .spawn_actor(&actor_id_for_kr, kernel_config)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                if is_agent {
                    // Stash so the MCP-side `actor_spawn` refine path can
                    // drain it into `agent_mailboxes` for SSE delivery.
                    // Non-Agent kinds don't consume their mailbox today —
                    // drop it (same as before this change).
                    mailbox_stash()
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .insert(actor_id_for_kr.clone(), mailbox);
                }
                Ok(())
            } else {
                Ok(())
            }
        });
        if let Err(e) = res {
            if is_agent {
                return Err(anyhow::anyhow!(
                    "kernel pre-register failed for agent {actor_id}: {e}"
                ));
            }
            tracing::warn!(actor_id = %actor_id, "kernel spawn_actor failed: {e}");
        }
    }

    // For now there is exactly one transport. The match here gates
    // future transports without changing the call site.
    let child = match profile.transport {
        TransportKind::Pty => {
            PtyTransport.spawn(&actor_id, &profile)
                .with_context(|| format!("pty spawn for {actor_id}"))?
        }
        TransportKind::Headless => {
            HeadlessTransport.spawn(&actor_id, &profile)
                .with_context(|| format!("headless spawn for {actor_id}"))?
        }
    };
    let child = Arc::new(child);

    // Emit actor.created. Best-effort — the spawn succeeded, projection
    // catch-up is independent. Fire on a std::thread to avoid the
    // block_on-inside-tokio deadlock the older path papered over.
    let emit_actor_id = actor_id.clone();
    let emit_profile = profile.clone();
    let emit_pid = child.pid();
    std::thread::spawn(move || {
        let cwd = emit_profile.work_dir.to_string_lossy().to_string();
        let space_id = resolve_space_id(&emit_profile.work_dir);
        let payload = crate::events::types::ActorCreated {
            kind: emit_profile.kind.as_str().to_string(),
            environment_type: "local".to_string(),
            workspace_id: emit_profile.workspace_id.clone(),
            space_id,
            cwd: Some(cwd),
            agent_id: emit_profile.agent_id.clone(),
            harness: emit_profile.harness.clone(),
            transport: Some(emit_profile.transport.as_str().to_string()),
            persistence: Some(emit_profile.persistence_label().to_string()),
            pid: emit_pid,
            ..Default::default()
        };
        if let Err(e) = crate::db::actor_events::emit_actor_created(
            &emit_actor_id,
            &payload,
            emit_profile.workspace_id.as_deref(),
            emit_profile.parent_actor_id.as_deref(),
        ) {
            tracing::warn!(actor_id = %emit_actor_id, "actor.created emit failed: {e}");
        }
    });

    Ok(ActorHandle {
        actor_id,
        kind: profile.kind,
        child,
        sandbox: None,
    })
}

impl ActorHandle {
    /// Attach a sandbox to this handle. Called by the supervisor right
    /// after `spawn_actor` returns when the agent runs inside an
    /// isolator. Returns a new handle (shares the same child + actor
    /// id) with the sandbox owner attached. The owner's `Drop` fires
    /// shutdown when the last clone of this handle is gone.
    pub fn with_sandbox(mut self, sandbox: Arc<crate::isolation::SandboxOwner>) -> Self {
        self.sandbox = Some(sandbox);
        self
    }
}

/// Convenience: where children write their logs by default. Callers
/// can override via `ActorProfile::log_dir`.
pub fn default_log_dir() -> Result<PathBuf> {
    Ok(crate::project::get_global_dir()?.join("logs/actors"))
}

#[cfg(test)]
mod tests;
