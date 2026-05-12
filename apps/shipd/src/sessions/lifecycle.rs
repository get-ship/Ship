//! Session lifecycle — spawn, stop, write-stdin, read.
//!
//! Agent sessions dispatch through the provider adapter trait so
//! Claude / Codex / OpenCode all reuse the same registry + stream
//! primitives; only the CLI argv + parsing differ. Shell sessions
//! skip the adapter and spawn `$SHELL` directly.

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use ship_platform::services::dispatch_ports::LaunchProfile;
use ship_platform::services::provider_adapter::adapter_for;
use tokio::sync::broadcast;

use super::registry::SessionRegistry;
use super::types::{
    private::Inner, PtySession, SessionHandle, SessionId, SessionKind, SessionMeta,
    SessionStatus,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
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
    let store = match SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session event {event_type}: open store failed: {e}");
            return;
        }
    };
    if let Err(e) = store.append(&envelope) {
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

/// Writable byte budget per `write_input` call — generous cap to stop
/// a runaway caller from filling a provider's input buffer.
const MAX_INPUT_BYTES: usize = 1 << 20;

/// Seconds of output-quiet before a thinking session flips back to idle.
const IDLE_AFTER_SECS: i64 = 3;

pub struct SpawnOutcome {
    pub session: PtySession,
    pub attached: bool,
}

/// Spawn (or attach to) an agent session for `(workspace, agent, provider)`.
///
/// If an existing session matches the triple, return it unchanged with
/// `attached = true`. Otherwise spawn a fresh PTY.
pub fn spawn_agent(
    registry: &SessionRegistry,
    workspace_id: &str,
    agent_id: &str,
    provider: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
) -> Result<SpawnOutcome> {
    if let Some(existing) = registry.find_agent_session(workspace_id, agent_id, provider) {
        return Ok(SpawnOutcome {
            session: existing,
            attached: true,
        });
    }

    let _ = adapter_for(provider).map_err(|e| anyhow!(e))?;

    // Persistent-session providers are intercepted in `api::spawn_session`
    // and routed to `agent_actor::spawn_or_attach`, which uses the
    // unified supervisor primitive. By the time we get here, this is the
    // per-turn fallback for non-persistent providers — agent metadata
    // only, no spawn. `send_agent_message` will spawn the headless turn
    // child via `agent_actor::run_headless_turn` when a turn arrives.

    // Per-turn fallback: agent metadata only. `send_agent_message` spawns
    // a fresh headless provider run per turn and parses stream-json. Used
    // by providers without a channels capability (codex, opencode, etc.).
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
        provider: Some(provider.to_string()),
        pid: None,
        status: SessionStatus::Idle,
        log_path: log_path.display().to_string(),
        spawned_at: now,
        last_activity: now,
        exit_code: None,
        provider_session_id: None,
    };

    let inner = Arc::new(Inner {
        meta: StdMutex::new(meta),
        writer: StdMutex::new(None),
        _master: StdMutex::new(None),
        output_tx,
        event_tx,
        child: StdMutex::new(None),
        log_path,
    });

    registry.insert(id.clone(), inner.clone());
    let _ = work_dir;
    Ok(SpawnOutcome {
        session: PtySession(inner),
        attached: false,
    })
}

/// Send a user turn to an agent session. Spawns the provider headless
/// with stream-json output, parses events line-by-line, broadcasts
/// `ProviderEvent`s on the session's event_tx, and captures the
/// provider's session id for subsequent `--resume` calls.
pub fn send_agent_message(
    registry: &SessionRegistry,
    session_id: &str,
    text: &str,
    work_dir: PathBuf,
) -> Result<()> {
    let session = registry
        .get(session_id)
        .ok_or_else(|| anyhow!("session '{session_id}' not found"))?;
    let (provider, resume, slug) = {
        let m = session.0.meta.lock().unwrap();
        if m.kind != SessionKind::Agent {
            bail!("send_agent_message: session is not an agent session");
        }
        let p = m
            .provider
            .clone()
            .ok_or_else(|| anyhow!("agent session missing provider"))?;
        (p, m.provider_session_id.clone(), m.id.clone())
    };
    let adapter = adapter_for(&provider).map_err(|e| anyhow!(e))?;

    // Channels path: long-lived process is already running. The HTTP
    // handler emits a `studio.chat_turn` kernel event scoped to this
    // session id; the MCP `EventRelay` pushes it to the connected peer
    // as a `notifications/claude/channel`. The agent reads the channel
    // event and replies via `chat_reply`. No per-turn subprocess spawn,
    // no `--resume` flag thrash. If the long-lived process died, fall
    // through to the headless per-turn fallback below so the chat
    // remains usable while we add restart logic.
    let has_long_lived_process = {
        let writer = session.0.writer.lock().unwrap();
        writer.is_some()
    };
    tracing::info!(
        session = %slug,
        provider = %provider,
        channels = adapter.caps().persistent_session,
        has_long_lived = has_long_lived_process,
        "send_agent_message: routing decision"
    );
    if adapter.caps().persistent_session && has_long_lived_process {
        let _ = text; // Delivered via the channel event from the HTTP layer.
        set_status(&session.0, SessionStatus::Thinking);
        return Ok(());
    }
    let agent_id_opt = session.0.meta.lock().unwrap().agent_id.clone();
    let append_system_prompt = agent_id_opt
        .as_deref()
        .and_then(|aid| load_agent_rules(&work_dir, aid));
    let profile = LaunchProfile {
        provider: Some(provider.clone()),
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
            ship_platform::services::provider_adapter::ProviderEvent::Message {
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
    pub project_id: Option<String>,
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
        provider: m.provider.clone(),
        model: extras.model.clone(),
        transport: extras.transport.clone().or(Some("pty".to_string())),
        persistence: extras.persistence.clone().or(Some("long-lived".to_string())),
        system_prompt_hash: extras.system_prompt_hash.clone(),
        pid: m.pid.map(|p| p),
        capabilities: extras.capabilities.clone(),
        project_id: extras.project_id.clone(),
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

/// Write bytes to a session's PTY master (stdin from the child's view).
/// Agent sessions don't hold a PTY writer; callers should use
/// `send_agent_message` instead.
pub fn write_input(session: &PtySession, bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_INPUT_BYTES {
        bail!("input exceeds {MAX_INPUT_BYTES} byte limit");
    }
    let mut guard = session.0.writer.lock().unwrap();
    let writer = guard
        .as_mut()
        .ok_or_else(|| anyhow!("session has no stdin writer (agent sessions use /message)"))?;
    writer
        .write_all(bytes)
        .map_err(|e| anyhow!("write pty: {e}"))?;
    writer
        .flush()
        .map_err(|e| anyhow!("flush pty: {e}"))?;
    Ok(())
}

/// Gracefully stop a session: SIGTERM via `kill()`, wait briefly, then
/// drop from the registry. Returns true if a session was present.
pub fn stop(registry: &SessionRegistry, id: &str) -> bool {
    let Some(session) = registry.get(id) else {
        return false;
    };
    {
        let mut guard = session.0.child.lock().unwrap();
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
    }
    // Give the reader thread a moment to observe EOF + set Ended.
    std::thread::sleep(Duration::from_millis(50));
    registry.remove(id);
    true
}

/// Load an agent's `rules.inline` from `.ship/agents/<agent_id>.jsonc` in
/// the work directory. Prepended with a Ship-specific preamble so the agent
/// knows which session id to reply into. Returns `None` if the file is
/// absent, unreadable, or has no inline rules — callers fall back to vanilla
/// provider behaviour.
pub fn load_agent_rules(work_dir: &std::path::Path, agent_id: &str) -> Option<String> {
    let path = ship_substrate::ShipHome::project_ship_dir(work_dir).join("agents").join(format!("{agent_id}.jsonc"));
    let raw = std::fs::read_to_string(&path).ok()?;
    let stripped = strip_jsonc(&raw);
    let v: serde_json::Value = serde_json::from_str(&stripped).ok()?;
    let inline = v
        .get("rules")
        .and_then(|r| r.get("inline"))
        .and_then(|s| s.as_str())?;
    if inline.trim().is_empty() {
        return None;
    }
    Some(format!(
        "# Ship agent context\n\nYou are the \"{agent_id}\" agent spawned by the Ship daemon. \
         Your chat session id is in the SHIP_ACTOR_ID environment variable — \
         pass it to the chat_reply MCP tool to reply to the user.\n\n{inline}"
    ))
}

/// Minimal JSONC stripper: removes `// line` and `/* block */` comments.
/// Does not handle comments inside string literals; good enough for hand-
/// written agent configs.
fn strip_jsonc(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            out.push(b as char);
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_str = true;
            out.push('"');
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() {
            let n = bytes[i + 1];
            if n == b'/' {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if n == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

