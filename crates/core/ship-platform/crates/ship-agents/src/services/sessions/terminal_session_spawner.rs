//! Terminal sessions (plain `$SHELL`) on the supervisor primitive.
//!
//! Builds a [`ProcessProfile`] for the shell kind, calls
//! [`bridge::spawn_process`], and registers the resulting
//! [`Entry`] in the unified [`Registry`]. The supervisor primitive
//! owns the kernel registration, PTY hold, log file, and `actor.created`
//! emission; this module owns only the shell-specific shape (`$SHELL` binary,
//! work dir, no agent/harness) and the sidecar [`TerminalSessionMeta`].

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use bridge::{ProcessKind, ProcessProfile, TransportKind, spawn_process};
use chrono::Utc;
use tokio::sync::Mutex;

use super::registry::{Entry, Registry, SessionRecord};
use super::types::SessionStatus;

pub fn spawn_shell(
    registry: &Registry,
    work_dir: PathBuf,
    log_dir: PathBuf,
    kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
) -> Result<Entry> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

    let profile = ProcessProfile {
        actor_id_hint: None,
        kind: ProcessKind::Shell,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: shell,
        args: vec!["-l".to_string()],
        env: Default::default(),
        work_dir,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let handle = spawn_process(profile, crate::supervisor_sink::supervisor_context(kernel)?)?;
    let now = Utc::now();
    let meta = SessionRecord {
        id: handle.actor_id.clone(),
        agent_id: None,
        harness: None,
        pid: handle.pid(),
        status: SessionStatus::Idle,
        log_path: handle.child.log_path().display().to_string(),
        spawned_at: now,
        last_activity: now,
        exit_code: None,
        harness_session_id: None,
    };

    Ok(registry.insert(handle, meta))
}

/// Stop a shell session. Terminates the underlying child process (the
/// supervisor handle) and removes the registry entry.
///
/// Returns `true` if the session was found and stopped, `false` if the id
/// wasn't registered.
pub fn stop(registry: &Registry, id: &str) -> bool {
    let Some(entry) = registry.get(id) else {
        return false;
    };
    if let Err(e) = entry.handle.stop() {
        tracing::warn!(session_id = %id, "shell stop: handle.stop failed: {e}");
    }
    registry.remove(id);
    true
}
