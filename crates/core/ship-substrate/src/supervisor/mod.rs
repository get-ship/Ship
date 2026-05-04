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
//!     provider, env, work dir. The orchestrator wires Transport +
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
pub mod transport;

pub use profile::{ActorKind, ActorProfile, Transport as TransportKind};
pub use transport::{ChildHandle, HeadlessTransport, PtyTransport, Transport};

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

/// A live spawn — the supervisor owns the child process; the caller
/// holds this handle for input/observation/lifecycle. Drop semantics:
/// dropping does NOT kill the child; explicit `stop()` does. This
/// matches the `lifecycle.rs::PtySession` contract so migration is a
/// drop-in replacement.
#[derive(Clone)]
pub struct ActorHandle {
    pub actor_id: String,
    pub kind: ActorKind,
    pub child: Arc<ChildHandle>,
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
/// Side effects: child process started, kernel actor event emitted.
/// On error: child is reaped, no event fires, error returned.
pub fn spawn_actor(profile: ActorProfile) -> Result<ActorHandle> {
    let actor_id = profile.resolve_actor_id();

    // Register with the kernel router so the actor appears on the
    // mesh (kernel.list_actors()) — the source-of-truth contract.
    //
    // EXCEPT for Agents: agents have an MCP child that calls
    // actor_spawn over HTTP, which both registers the actor AND
    // captures the mailbox handle for SSE drain. Pre-registering
    // here makes the MCP-side call a no-op (has_actor=true) and
    // orphans the mailbox, silently dropping channel events.
    // For Shells / Pipelines / Services there is no MCP child, so
    // the supervisor is the only one who can register them.
    let pre_register = !matches!(profile.kind, ActorKind::Agent);
    if pre_register {
        if let Some(kr) = crate::events::kernel_router() {
        let kernel_config = crate::events::ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec!["".to_string()],
            read_namespaces: vec!["agent.".to_string()],
            subscribe_namespaces: profile.subscribe_namespaces.clone(),
        };
        let actor_id_for_kr = actor_id.clone();
        let res = crate::db::block_on_anyhow(async move {
            let mut kernel = kr.lock().await;
            if !kernel.has_actor(&actor_id_for_kr) {
                kernel.spawn_actor(&actor_id_for_kr, kernel_config)
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!("{e}"))
            } else {
                Ok(())
            }
        });
        if let Err(e) = res {
            tracing::warn!(actor_id = %actor_id, "kernel spawn_actor failed: {e}");
        }
        }
    }

    // For now there is exactly one transport. The match here gates
    // future transports without changing the call site.
    let child = match profile.transport {
        TransportKind::Pty => {
            PtyTransport::default().spawn(&actor_id, &profile)
                .with_context(|| format!("pty spawn for {actor_id}"))?
        }
        TransportKind::Headless => {
            HeadlessTransport::default().spawn(&actor_id, &profile)
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
        let project_id = crate::projects::find_by_path(&emit_profile.work_dir)
            .ok()
            .flatten()
            .map(|p| p.id);
        let payload = crate::events::types::ActorCreated {
            kind: emit_profile.kind.as_str().to_string(),
            environment_type: "local".to_string(),
            workspace_id: emit_profile.workspace_id.clone(),
            project_id,
            cwd: Some(cwd),
            agent_id: emit_profile.agent_id.clone(),
            provider: emit_profile.provider.clone(),
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
            tracing::debug!(actor_id = %emit_actor_id, "actor.created emit failed: {e}");
        }
    });

    Ok(ActorHandle {
        actor_id,
        kind: profile.kind,
        child,
    })
}

/// Convenience: where children write their logs by default. Callers
/// can override via `ActorProfile::log_dir`.
pub fn default_log_dir() -> Result<PathBuf> {
    Ok(crate::project::get_global_dir()?.join("logs/actors"))
}

#[cfg(test)]
mod tests;
