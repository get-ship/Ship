//! Shell sessions on the supervisor primitive.
//!
//! Replaces `lifecycle::spawn_shell` for the Shell kind. Builds an
//! `ActorProfile`, calls `ship_substrate::supervisor::spawn_actor`, and inserts
//! into the PTY `SessionRegistry` (the ship-terminal one). The supervisor
//! primitive owns the kernel registration, PTY hold, log file, and
//! `actor.created` emission — this function's only job is shell-specific
//! shape (env, work dir, $SHELL) plus the session-flavored sidecar metadata.

use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

use anyhow::Result;
use chrono::Utc;
use ship_substrate::supervisor::{spawn_actor, ActorKind, ActorProfile, TransportKind};
use tokio::sync::broadcast;

use crate::registry::SessionRegistry;
use crate::types::{
    private::Inner, PtySession, SessionId, SessionKind, SessionMeta, SessionStatus,
};

/// Broadcast channel capacity — mirrors the per-turn path in
/// `ship_agents::services::sessions::lifecycle`. Shells don't publish
/// here today (the substrate handle's `subscribe_output` is the SSOT for
/// bytes), but the Inner struct needs both channels populated for the
/// `PtySession` API to be uniform across the registry.
const OUTPUT_CHANNEL_CAP: usize = 1024;
const EVENT_CHANNEL_CAP: usize = 256;

pub fn spawn_shell(
    registry: &SessionRegistry,
    workspace_id: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
) -> Result<PtySession> {
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
        log_dir: Some(log_dir.clone()),
        subscribe_namespaces: vec![],
    };

    let handle = spawn_actor(profile)?;
    let id: SessionId = handle.actor_id.clone();
    let log_path = handle.child.log_path().clone();
    let pid = handle.pid();

    let (output_tx, _) = broadcast::channel::<Vec<u8>>(OUTPUT_CHANNEL_CAP);
    let (event_tx, _) =
        broadcast::channel::<ship_harnesses::harness_adapter::HarnessEvent>(EVENT_CHANNEL_CAP);

    let now = Utc::now();
    let meta = SessionMeta {
        id: id.clone(),
        kind: SessionKind::Shell,
        workspace_id: workspace_id.to_string(),
        agent_id: None,
        harness: None,
        pid,
        status: SessionStatus::Idle,
        log_path: log_path.display().to_string(),
        spawned_at: now,
        last_activity: now,
        exit_code: None,
        harness_session_id: None,
    };

    let inner = Arc::new(Inner {
        meta: StdMutex::new(meta),
        output_tx,
        event_tx,
        log_path,
    });

    registry.insert(id, inner.clone());
    Ok(PtySession(inner))
}

pub fn stop(registry: &SessionRegistry, id: &str) -> bool {
    registry.remove(id).is_some()
}
