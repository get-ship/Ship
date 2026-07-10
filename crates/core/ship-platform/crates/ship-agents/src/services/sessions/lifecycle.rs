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
use harness_adapter::dispatch_ports::LaunchProfile;
use ship_harnesses::harness_adapter::adapter_for;
use tokio::sync::broadcast;

use ship_config::agents_index::AgentDefIndex;
use ship_terminal::SessionRegistry;
use ship_terminal::types::{
    Inner, PtySession, SessionHandle, SessionId, SessionKind, SessionMeta, SessionStatus,
};

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
    agent_id: &str,
    harness: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
) -> Result<SpawnOutcome> {
    if let Some(existing) = registry.find_agent_session(agent_id, harness) {
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
        child: None,
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
pub async fn send_agent_message(
    registry: &SessionRegistry,
    session_id: &str,
    text: &str,
    work_dir: PathBuf,
    index: Option<&AgentDefIndex>,
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
    let append_system_prompt = match agent_id_opt.as_deref() {
        Some(aid) => load_agent_rules(index, &work_dir, aid).await,
        None => None,
    };
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
        let _ = session.0.event_tx.send(
            ship_harnesses::harness_adapter::HarnessEvent::Message {
                text: format!("[ship] turn failed: {e}"),
            },
        );
        set_status(&session.0, SessionStatus::Idle);
    }

    Ok(())
}

fn set_status(inner: &SessionHandle, status: SessionStatus) {
    let (_id, _prior) = {
        let mut m = inner.meta.lock().unwrap();
        let prior = m.status;
        m.status = status;
        m.last_activity = Utc::now();
        (m.id.clone(), prior)
    };
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
                    m.status = SessionStatus::Idle;
                    drop(m);
                }
            }
    }
}

/// Load an agent's `rules.inline`, prepended with a Ship-specific preamble so
/// the agent knows which session id to reply into. Returns `None` when the def
/// is absent or has no inline rules — callers fall back to vanilla harness
/// behaviour.
///
/// When an [`AgentDefIndex`] is supplied, the inline rules are served from the
/// DataStore snapshot (project scope) — so a running spawn does NOT depend on
/// the `.ship/agents/*.jsonc` file. The file reader is the cold / cross-space
/// fallback (no index, or an agent that lives outside the indexed space).
pub async fn load_agent_rules(
    index: Option<&AgentDefIndex>,
    work_dir: &std::path::Path,
    agent_id: &str,
) -> Option<String> {
    let inline = load_inline_rules(index, work_dir, agent_id).await?;
    if inline.trim().is_empty() {
        return None;
    }
    Some(format!(
        "# Ship agent context\n\nYou are the \"{agent_id}\" agent spawned by the Ship daemon. \
         Your chat session id is in the SHIP_ACTOR_ID environment variable — \
         pass it to the chat_reply MCP tool to reply to the user.\n\n{inline}"
    ))
}

/// Resolve `rules.inline` for `agent_id`: index snapshot first, file fallback.
async fn load_inline_rules(
    index: Option<&AgentDefIndex>,
    work_dir: &std::path::Path,
    agent_id: &str,
) -> Option<String> {
    if let Some(idx) = index
        && let Ok(Some(doc)) = idx.get_doc(agent_id, "project").await
    {
        return doc
            .get("rules")
            .and_then(|r| r.get("inline"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
    }
    // Cold / cross-space fallback: THE agent-def reader (FsBroker, ship-config)
    // is the one file door — `rules.inline` lives on the FULL nested profile.
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(work_dir);
    let fs = ship_substrate::ports::LocalFsBroker::new();
    ship_config::agents::read_agent_profile(&fs, &ship_dir, agent_id)
        .ok()
        .flatten()
        .and_then(|p| p.rules.inline)
}

