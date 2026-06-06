//! Shell sessions on the supervisor primitive.
//!
//! Replaces `lifecycle::spawn_shell` for the Shell kind. Builds an
//! `ActorProfile`, calls `ship_substrate::supervisor::spawn_actor`, and inserts
//! into `ActorRegistry`. The supervisor primitive owns the kernel
//! registration, PTY hold, log file, and `actor.created` emission — this
//! function's only job is shell-specific shape (env, work dir, $SHELL)
//! plus the session-flavored sidecar metadata.

use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use ship_substrate::supervisor::{spawn_actor, ActorKind, ActorProfile, TransportKind};

use super::actor_registry::{ActorEntry, ActorRegistry};
use super::types::{SessionKind, SessionMeta, SessionStatus};

pub fn spawn_shell(
    registry: &ActorRegistry,
    workspace_id: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
) -> Result<ActorEntry> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

    let profile = ActorProfile {
        actor_id_hint: None,
        kind: ActorKind::Shell,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: shell,
        args: vec!["-l".to_string()],
        env: Default::default(),
        work_dir,
        workspace_id: Some(workspace_id.to_string()),
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let handle = spawn_actor(profile)?;

    let now = Utc::now();
    let meta = SessionMeta {
        id: handle.actor_id.clone(),
        kind: SessionKind::Shell,
        workspace_id: workspace_id.to_string(),
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

pub fn stop(registry: &ActorRegistry, id: &str) -> bool {
    let Some(entry) = registry.remove(id) else {
        return false;
    };
    let _ = entry.handle.stop();
    true
}
