//! Session lifecycle — spawn, stop, write-stdin, read.
//!
//! Agent sessions dispatch through the harness adapter trait so
//! Claude / Codex / OpenCode all reuse the same registry + stream
//! primitives; only the CLI argv + parsing differ. Shell sessions
//! skip the adapter and spawn `$SHELL` directly.

use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use crate::services::dispatch_ports::LaunchProfile;
use crate::services::harness_adapter::adapter_for;
use tokio::sync::broadcast;

use super::registry::SessionRegistry;
use super::types::{
    private::Inner, PtySession, SessionHandle, SessionId, SessionKind, SessionMeta,
    SessionStatus,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::persistence::event_store::SqliteEventStore;
use ship_types::event_types::{SESSION_STATUS_CHANGED, SESSION_TURN_FAILED};
use ship_types::payload::{SessionStatusChanged, SessionTurnFailed};

/// Best-effort emit. The session's in-memory state is the source of
/// truth; the event log is audit/observability scaffolding.
fn emit_session_event(event_type: &'static str, session_id: &str, payload: &impl serde::Serialize) {
    let envelope = match EventEnvelope::new(event_type, session_id, payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("session event {event_type}: build envelope failed: {e}");
            return;
        }
    };
    // session.* is Runtime-scoped → write runtime.db (the session projection's db).
    let store = match SqliteEventStore::new_for_scope(
        &match ship_substrate::project::get_global_dir() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("session event {event_type}: resolve ship dir failed: {e}");
                return;
            }
        },
        ship_types::event_registry::EventScope::Runtime,
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session event {event_type}: open store failed: {e}");
            return;
        }
    };
    // Enforced write: session.* is Runtime-scoped with no space binding at this
    // call site (the service task has no session-minted binding), so stamp +
    // enforce from the daemon-internal binding before persist.
    let binding = ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk);
    if let Err(e) = store.append_enforced(&envelope, &binding) {
        tracing::warn!("session event {event_type}: append failed: {e}");
    }
}

fn status_str(s: SessionStatus) -> &'static str {
    match s {
        SessionStatus::Spawning => "spawning",
        SessionStatus::Idle => "idle",
        SessionStatus::Thinking => "thinking",
        SessionStatus::Ended => "ended",
    }
}

/// Broadcast channel capacity — 1024 chunks ≈ several screens of output
/// at typical PTY burst rates. Lagging subscribers drop old frames.
const OUTPUT_CHANNEL_CAP: usize = 1024;

/// Seconds of output-quiet before a thinking session flips back to idle.
const IDLE_AFTER_SECS: i64 = 3;

pub struct SpawnOutcome {
    pub session: PtySession,
    pub attached: bool,
}

/// Spawn (or attach to) an agent session for `(workspace, agent, harness)`.
///
/// If an existing session matches the triple, return it unchanged with
/// `attached = true`. Otherwise spawn a fresh PTY.
pub fn spawn_agent(
    registry: &SessionRegistry,
    workspace_id: &str,
    agent_id: &str,
    harness: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
) -> Result<SpawnOutcome> {
    if let Some(existing) = registry.find_agent_session(workspace_id, agent_id, harness) {
        return Ok(SpawnOutcome {
            session: existing,
            attached: true,
        });
    }

    let _ = adapter_for(harness).map_err(|e| anyhow!(e))?;

    // Two session paths:
    //   - Persistent harnesses (claude with --resume) are intercepted in
    //     `api::spawn_session` and routed to `agent_actor::spawn_or_attach`
    //     via the unified supervisor primitive.
    //   - Per-turn harnesses (codex, opencode — no channels capability)
    //     fall through here. We allocate session metadata only; the actual
    //     child is spawned fresh per turn by `agent_actor::run_headless_turn`
    //     when `send_agent_message` is called.
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("create log dir {}", log_dir.display()))?;
    let id: SessionId = ship_substrate::gen_ulid();
    let log_path = log_dir.join(format!("agent-{agent_id}-{id}.log"));
    let _ = std::fs::write(&log_path, "");

    let (output_tx, _) = broadcast::channel(OUTPUT_CHANNEL_CAP);
    let (event_tx, _) = broadcast::channel(256);

    let now = Utc::now();
    let meta = SessionMeta {
        id: id.clone(),
        kind: SessionKind::Agent,
        workspace_id: workspace_id.to_string(),
        agent_id: Some(agent_id.to_string()),
        harness: Some(harness.to_string()),
        pid: None,
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

    registry.insert(id.clone(), inner.clone());
    let _ = work_dir;
    Ok(SpawnOutcome {
        session: PtySession(inner),
        attached: false,
    })
}

/// Send a user turn to an agent session. Spawns the harness headless
/// with stream-json output, parses events line-by-line, broadcasts
/// `HarnessEvent`s on the session's event_tx, and captures the
/// harness's session id for subsequent `--resume` calls.
pub fn send_agent_message(
    registry: &SessionRegistry,
    session_id: &str,
    text: &str,
    work_dir: PathBuf,
) -> Result<()> {
    let session = registry
        .get(session_id)
        .ok_or_else(|| anyhow!("session '{session_id}' not found"))?;
    let (harness, resume, slug) = {
        let m = session.0.meta.lock().unwrap();
        if m.kind != SessionKind::Agent {
            bail!("send_agent_message: session is not an agent session");
        }
        let p = m
            .harness
            .clone()
            .ok_or_else(|| anyhow!("agent session missing harness"))?;
        (p, m.harness_session_id.clone(), m.id.clone())
    };
    let adapter = adapter_for(&harness).map_err(|e| anyhow!(e))?;

    // Persistent-session harnesses (claude with --resume) live on
    // `ActorRegistry`, not here — they're routed earlier in
    // `api::spawn_session` to `agent_actor::spawn_or_attach` and their
    // chat turns flow over the channel event from the HTTP layer.
    // Anything reaching this function is a per-turn harness (codex,
    // opencode, cursor) and gets a fresh headless child below.
    tracing::info!(session = %slug, harness = %harness, "send_agent_message: per-turn headless");

    let agent_id_opt = session.0.meta.lock().unwrap().agent_id.clone();
    let append_system_prompt = agent_id_opt
        .as_deref()
        .and_then(|aid| load_agent_rules(&work_dir, aid));
    let profile = LaunchProfile {
        harness: Some(harness.clone()),
        resume_session: resume,
        autostart_prompt: Some(text.to_string()),
        persist_session: true,
        structured_output: true,
        append_system_prompt,
        ..LaunchProfile::default()
    };
    let launch = adapter.build_headless(&slug, &profile);

    // Mark thinking and kick the child off-thread so this call returns
    // fast to the HTTP handler; the spawned process + reader continue.
    set_status(&session.0, SessionStatus::Thinking);

    let adapter_id = adapter.id().to_string();
    if let Err(e) = super::agent_actor::run_headless_turn(
        session.clone(),
        launch,
        adapter_id,
        work_dir,
    ) {
        let session_id = session.0.meta.lock().unwrap().id.clone();
        tracing::error!(session = %session_id, error = %e, "headless turn spawn failed");
        emit_session_event(
            SESSION_TURN_FAILED,
            &session_id,
            &SessionTurnFailed { error: e.to_string() },
        );
        let _ = session.0.event_tx.send(
            crate::services::harness_adapter::HarnessEvent::Message {
                text: format!("[ship] turn failed: {e}"),
            },
        );
        set_status(&session.0, SessionStatus::Idle);
    }

    Ok(())
}

/// Caller-supplied actor metadata recorded into `actor.created` payload.
/// `kind` and the basics come from the spawn primitive; this carries the
/// agent-flavored fields the projection / dashboard cares about.
#[derive(Default, Clone)]
pub struct ActorSpawnExtras {
    pub model: Option<String>,
    pub transport: Option<String>,
    pub persistence: Option<String>,
    pub system_prompt_hash: Option<String>,
    pub capabilities: Vec<String>,
    pub space_id: Option<String>,
    pub cwd: Option<String>,
}

/// Emit `actor.created` for a freshly spawned PtySession. Reads metadata
/// from the session's meta lock, then dispatches the emit on a fresh
/// std::thread so the underlying `block_on` in the kernel-route path
/// doesn't deadlock the tokio runtime that called us. Fire-and-forget;
/// emit failures only warn.
pub fn emit_actor_created_for(session: &PtySession, extras: &ActorSpawnExtras) {
    let m = session.0.meta.lock().unwrap();
    let kind_str = match m.kind {
        SessionKind::Agent => "agent",
        SessionKind::Pipeline => "pipeline-step",
        SessionKind::Shell => "shell",
    };
    let payload = ship_substrate::events::types::ActorCreated {
        kind: kind_str.to_string(),
        environment_type: "local".to_string(),
        workspace_id: Some(m.workspace_id.clone()),
        parent_actor_id: None,
        restart_count: 0,
        agent_id: m.agent_id.clone(),
        harness: m.harness.clone(),
        model: extras.model.clone(),
        transport: extras.transport.clone().or(Some("pty".to_string())),
        persistence: extras.persistence.clone().or(Some("long-lived".to_string())),
        system_prompt_hash: extras.system_prompt_hash.clone(),
        pid: m.pid,
        capabilities: extras.capabilities.clone(),
        space_id: extras.space_id.clone(),
        cwd: extras.cwd.clone(),
    };
    let actor_id = m.id.clone();
    let workspace_id = m.workspace_id.clone();
    drop(m);
    std::thread::spawn(move || {
        if let Err(e) = ship_substrate::db::actor_events::emit_actor_created(
            &actor_id,
            &payload,
            Some(&workspace_id),
            None,
        ) {
            tracing::warn!(actor = %actor_id, error = %e, "actor.created emit failed");
        }
    });
}


fn set_status(inner: &SessionHandle, status: SessionStatus) {
    let (id, prior) = {
        let mut m = inner.meta.lock().unwrap();
        let prior = m.status;
        m.status = status;
        m.last_activity = Utc::now();
        (m.id.clone(), prior)
    };
    if prior != status {
        emit_session_event(
            SESSION_STATUS_CHANGED,
            &id,
            &SessionStatusChanged {
                from: status_str(prior).to_string(),
                to: status_str(status).to_string(),
            },
        );
    }
}

/// Idle-reaper: flips Thinking → Idle after a quiet window. Run this
/// on a timer in the daemon's main tick, or expose as a helper the
/// API can call before serving `meta()`. Cheap — just compares clock.
pub fn sweep_idle(registry: &SessionRegistry) {
    let now = Utc::now();
    let list = registry.list();
    for meta in list {
        if !matches!(meta.status, SessionStatus::Thinking) {
            continue;
        }
        if (now - meta.last_activity).num_seconds() >= IDLE_AFTER_SECS
            && let Some(session) = registry.get(&meta.id) {
                let mut m = session.0.meta.lock().unwrap();
                if matches!(m.status, SessionStatus::Thinking)
                    && (Utc::now() - m.last_activity).num_seconds() >= IDLE_AFTER_SECS
                {
                    let session_id = m.id.clone();
                    m.status = SessionStatus::Idle;
                    drop(m);
                    emit_session_event(
                        SESSION_STATUS_CHANGED,
                        &session_id,
                        &SessionStatusChanged {
                            from: "thinking".to_string(),
                            to: "idle".to_string(),
                        },
                    );
                }
            }
    }
}

/// Drop a per-turn agent session from the registry. The underlying
/// per-turn child is owned by the substrate supervisor and exits on its
/// own when the turn finishes; this just clears the metadata sidecar.
pub fn stop(registry: &SessionRegistry, id: &str) -> bool {
    registry.remove(id).is_some()
}

/// Load an agent's `rules.inline` from `.ship/agents/<agent_id>.jsonc` in
/// the work directory. Prepended with a Ship-specific preamble so the agent
/// knows which session id to reply into. Returns `None` if the file is
/// absent, unreadable, or has no inline rules — callers fall back to vanilla
/// harness behaviour.
pub fn load_agent_rules(work_dir: &std::path::Path, agent_id: &str) -> Option<String> {
    // THE agent-def reader (FsBroker, ship-config) is the one door — no raw
    // `std::fs` read of the def here. `rules.inline` lives on the FULL nested
    // profile.
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(work_dir);
    let fs = ship_substrate::ports::LocalFsBroker::new();
    let profile = ship_config::agents::read_agent_profile(&fs, &ship_dir, agent_id)
        .ok()
        .flatten()?;
    let inline = profile.rules.inline.as_deref()?;
    if inline.trim().is_empty() {
        return None;
    }
    Some(format!(
        "# Ship agent context\n\nYou are the \"{agent_id}\" agent spawned by the Ship daemon. \
         Your chat session id is in the SHIP_ACTOR_ID environment variable — \
         pass it to the chat_reply MCP tool to reply to the user.\n\n{inline}"
    ))
}

