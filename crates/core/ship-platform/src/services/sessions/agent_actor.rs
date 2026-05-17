//! Long-lived agent sessions on the supervisor primitive.
//!
//! Replaces `lifecycle::spawn_long_lived_agent` for channels-capable
//! harnesses (Claude). Builds an `ActorProfile` from the harness
//! adapter's `build_interactive`, calls `ship_substrate::supervisor::spawn_actor`,
//! and attaches a reader-middleware thread that:
//!
//! 1. Auto-dismisses Claude's `--dangerously-load-development-channels`
//!    confirmation menu by sending a single CR when the menu pattern
//!    appears in the byte stream. State-aware, fires once per session.
//! 2. Feeds bytes through the harness adapter line-by-line and
//!    publishes parsed `HarnessEvent`s on the entry's `event_tx`.
//! 3. Updates the sidecar `SessionMeta` (status, last_activity).
//! 4. Emits `actor.stopped` when the byte stream closes.
//!
//! Per-turn harnesses (codex, opencode, cursor, gemini) keep their
//! parent metadata on the legacy `SessionRegistry` (no persistent
//! child between turns), but `run_headless_turn` spawns the per-turn
//! child through the supervisor's Headless transport — same primitive,
//! different lifetime.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use chrono::Utc;
use crate::services::dispatch_ports::LaunchProfile;
use crate::services::harness_adapter::{adapter_for, HarnessEvent};
use ship_substrate::supervisor::{spawn_actor, ActorKind, ActorProfile, TransportKind};
use tokio::sync::broadcast;

use super::actor_registry::{ActorEntry, ActorRegistry};
use super::types::{SessionKind, SessionMeta, SessionStatus};

const NEEDLE: &[u8] = b"dangerously-load-development-channels";
const EVENT_CAP: usize = 256;

pub struct SpawnOutcome {
    pub entry: ActorEntry,
    pub attached: bool,
}

pub fn spawn_or_attach(
    registry: &ActorRegistry,
    workspace_id: &str,
    agent_id: &str,
    harness: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
    append_system_prompt: Option<String>,
) -> Result<SpawnOutcome> {
    if let Some(entry) = registry.find_agent_session(workspace_id, agent_id, harness) {
        return Ok(SpawnOutcome {
            entry,
            attached: true,
        });
    }

    let adapter = adapter_for(harness).map_err(|e| anyhow!(e))?;
    if !adapter.caps().persistent_session {
        anyhow::bail!(
            "agent_actor::spawn_or_attach: harness '{harness}' has no persistent-session \
             support; respawn-per-turn is not handled here"
        );
    }

    let slug_prefix = format!("agent-{agent_id}");
    let lp = LaunchProfile {
        harness: Some(harness.to_string()),
        persist_session: true,
        structured_output: true,
        append_system_prompt,
        ..LaunchProfile::default()
    };
    let launch = adapter.build_interactive(&slug_prefix, &lp);

    let profile = ActorProfile {
        actor_id_hint: None,
        kind: ActorKind::Agent,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: launch.bin,
        args: launch.args,
        env: launch.env,
        work_dir,
        workspace_id: Some(workspace_id.to_string()),
        agent_id: Some(agent_id.to_string()),
        harness: Some(harness.to_string()),
        parent_actor_id: None,
        log_dir: Some(log_dir),
        // Channel-listening lives in the MCP-side actor_spawn path
        // (it captures the mailbox for SSE drain). We deliberately
        // leave subscribe_namespaces empty here: the long-lived
        // claude binary has its own MCP child that registers itself
        // as an actor and owns chat_turn delivery.
        subscribe_namespaces: vec![],
    };

    let handle = spawn_actor(profile)?;
    let (event_tx, _) = broadcast::channel::<HarnessEvent>(EVENT_CAP);

    let now = Utc::now();
    let meta = SessionMeta {
        id: handle.actor_id.clone(),
        kind: SessionKind::Agent,
        workspace_id: workspace_id.to_string(),
        agent_id: Some(agent_id.to_string()),
        harness: Some(harness.to_string()),
        pid: handle.pid(),
        status: SessionStatus::Spawning,
        log_path: handle.child.log_path().display().to_string(),
        spawned_at: now,
        last_activity: now,
        exit_code: None,
        harness_session_id: None,
    };

    let entry = registry.insert_with(handle.clone(), meta, Some(event_tx.clone()));

    spawn_reader_middleware(entry.clone(), harness.to_string(), event_tx);

    Ok(SpawnOutcome {
        entry,
        attached: false,
    })
}

fn spawn_reader_middleware(
    entry: ActorEntry,
    harness: String,
    event_tx: broadcast::Sender<HarnessEvent>,
) {
    let mut rx = entry.handle.subscribe_output();
    let handle = entry.handle.clone();
    let meta = entry.meta.clone();
    let actor_id = handle.actor_id.clone();
    let workspace_id = meta.lock().unwrap().workspace_id.clone();
    let spawned_at = meta.lock().unwrap().spawned_at;

    tokio::spawn(async move {
        // Idle on entry — child is up but not yet thinking.
        {
            let mut m = meta.lock().unwrap();
            m.status = SessionStatus::Idle;
            m.last_activity = Utc::now();
        }
        let menu_state = Arc::new(Mutex::new(MenuState::default()));
        let adapter = adapter_for(&harness).ok();
        // Auto-dismiss only applies to Claude — the dev-channels TUI
        // confirmation menu is Claude-specific. Other harnesses don't
        // render it; running the matcher would just burn cycles + the
        // 16K menu_buf budget for nothing.
        let needs_menu_dismiss = harness == "claude";
        let mut line_buf = String::new();

        loop {
            match rx.recv().await {
                Ok(chunk) => {
                    if needs_menu_dismiss {
                        try_dismiss_menu(&handle, &menu_state, &chunk);
                    }

                    // Harness parse → event_tx.
                    if let Some(ref adapter) = adapter {
                        line_buf.push_str(&String::from_utf8_lossy(&chunk));
                        while let Some(nl) = line_buf.find('\n') {
                            let line: String = line_buf.drain(..=nl).collect();
                            let trimmed = line.trim_end_matches(['\n', '\r']);
                            if !trimmed.is_empty()
                                && let Some(ev) = adapter.parse_output(trimmed) {
                                    // Studio.reply parity: long-lived
                                    // non-Claude harnesses (codex,
                                    // opencode, cursor, gemini via stdin
                                    // push) emit Message events here.
                                    // Claude has its own chat_reply
                                    // path; emitting again would be a
                                    // double — but parse_output rarely
                                    // produces Message from a Claude
                                    // TUI rendering, so the duplicate
                                    // case is unlikely in practice.
                                    if let HarnessEvent::Message { text } = &ev {
                                        emit_studio_reply(&actor_id, text);
                                    }
                                    let _ = event_tx.send(ev);
                                }
                        }
                    }

                    {
                        let mut m = meta.lock().unwrap();
                        m.last_activity = Utc::now();
                        m.status = SessionStatus::Thinking;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }

        // Stream closed = child exited.
        {
            let mut m = meta.lock().unwrap();
            m.status = SessionStatus::Ended;
            m.last_activity = Utc::now();
        }

        let duration_ms = (Utc::now() - spawned_at).num_milliseconds().max(0) as u64;
        let payload = ship_substrate::events::types::ActorStopped {
            reason: "process_exit".to_string(),
            exit_code: None,
            duration_ms: Some(duration_ms),
        };
        let stopped_id = actor_id.clone();
        let stopped_ws = workspace_id.clone();
        std::thread::spawn(move || {
            let _ = ship_substrate::db::actor_events::emit_actor_stopped(
                &stopped_id,
                &payload,
                Some(&stopped_ws),
                None,
            );
        });
    });
}

/// Run a per-turn headless agent through the supervisor.
///
/// Per-turn "sessions" (metadata + event_tx) keep living on the legacy
/// `SessionRegistry` — they have no persistent child between turns, so
/// they don't fit ActorRegistry's "live ActorHandle" model. But the
/// child for each turn now spawns through the supervisor: fresh actor
/// id per turn, Headless transport, lifetime bounded by the turn.
///
/// Stream-json on stdout drains through the harness adapter and the
/// parsed `HarnessEvent`s forward to the parent session's event_tx
/// (so SSE consumers tailing /events keep working). Child exit
/// captures exit code into the parent session meta and flips status
/// back to Idle.
pub fn run_headless_turn(
    session: super::types::PtySession,
    launch: crate::services::dispatch_ports::LaunchCommand,
    harness_id: String,
    work_dir: PathBuf,
) -> Result<()> {
    let profile = ActorProfile {
        actor_id_hint: None,
        kind: ActorKind::Agent,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: launch.bin,
        args: launch.args,
        env: launch.env,
        work_dir,
        workspace_id: None,
        agent_id: None,
        harness: Some(harness_id.clone()),
        parent_actor_id: Some(session.0.meta.lock().unwrap().id.clone()),
        log_dir: None,
        subscribe_namespaces: vec![],
    };

    let handle = spawn_actor(profile)?;
    let mut rx = handle.subscribe_output();
    let session_clone = session.clone();
    let handle_for_wait = handle.clone();

    tokio::spawn(async move {
        let adapter = match adapter_for(&harness_id) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!(harness = %harness_id, "adapter lookup failed: {e}");
                return;
            }
        };
        let mut line_buf = String::new();
        loop {
            match rx.recv().await {
                Ok(chunk) => {
                    let _ = session_clone.0.output_tx.send(chunk.clone());
                    line_buf.push_str(&String::from_utf8_lossy(&chunk));
                    while let Some(nl) = line_buf.find('\n') {
                        let line: String = line_buf.drain(..=nl).collect();
                        let trimmed = line.trim_end_matches(['\n', '\r']);
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Some(ev) = adapter.parse_output(trimmed) {
                            if let HarnessEvent::SessionStarted { session_id } = &ev {
                                let mut m = session_clone.0.meta.lock().unwrap();
                                m.harness_session_id = Some(session_id.clone());
                            }
                            // Studio.reply parity: emit a kernel event on
                            // every Message HarnessEvent so the chat
                            // thread sees codex/opencode/cursor/gemini
                            // replies on the same rail Claude uses via
                            // chat_reply. Best-effort; in-memory event_tx
                            // broadcast still fires below for SSE
                            // consumers tailing /events.
                            if let HarnessEvent::Message { text: msg } = &ev {
                                let parent_id = session_clone.0.meta.lock().unwrap().id.clone();
                                emit_studio_reply(&parent_id, msg);
                            }
                            let _ = session_clone.0.event_tx.send(ev);
                            let mut m = session_clone.0.meta.lock().unwrap();
                            m.last_activity = chrono::Utc::now();
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }

        // Stream closed = child done. Poll briefly for exit code (the
        // reader threads exit slightly before the OS marks the child
        // reapable).
        let exit = (0..40).find_map(|_| {
            if let Some(c) = handle_for_wait.try_exit() {
                Some(c)
            } else {
                std::thread::sleep(std::time::Duration::from_millis(50));
                None
            }
        });
        let mut m = session_clone.0.meta.lock().unwrap();
        m.status = super::types::SessionStatus::Idle;
        m.last_activity = chrono::Utc::now();
        m.pid = None;
        if let Some(code) = exit {
            m.exit_code = Some(code);
        }
    });

    Ok(())
}

/// Emit a `studio.reply` kernel event for `text` keyed on `actor_id`.
/// Fire-and-forget — we're inside a non-async parse loop and don't
/// want to block on kernel availability. Spawned on a std::thread so
/// the block_on inside the kernel router doesn't deadlock the tokio
/// runtime that's driving the reader.
fn emit_studio_reply(actor_id: &str, text: &str) {
    let actor_id = actor_id.to_string();
    let text = text.to_string();
    std::thread::spawn(move || {
        let Some(kr) = ship_substrate::events::kernel_router() else { return };
        let payload = serde_json::json!({ "actor_id": &actor_id, "text": text });
        let envelope = match ship_substrate::events::EventEnvelope::new(
            ship_substrate::events::types::event_types::STUDIO_REPLY,
            &actor_id,
            &payload,
        ) {
            Ok(e) => e.with_actor_id(&actor_id).with_target(&actor_id),
            Err(e) => {
                tracing::warn!(actor = %actor_id, "studio.reply envelope failed: {e}");
                return;
            }
        };
        let ctx = ship_substrate::events::EmitContext {
            caller_kind: ship_substrate::events::CallerKind::Runtime,
            skill_id: None,
            project_id: None,
            session_id: None,
        };
        if let Err(e) = ship_substrate::db::block_on_anyhow(async move {
            kr.lock().await.emit(envelope, &ctx).await
        }) {
            tracing::warn!(actor = %actor_id, "studio.reply emit failed: {e}");
        }
    });
}

#[derive(Default)]
struct MenuState {
    dismissed: bool,
    buf: Vec<u8>,
}

fn try_dismiss_menu(
    handle: &ship_substrate::supervisor::ActorHandle,
    state: &Arc<Mutex<MenuState>>,
    chunk: &[u8],
) -> bool {
    let mut s = state.lock().unwrap();
    if s.dismissed {
        return false;
    }
    s.buf.extend_from_slice(chunk);
    if s.buf.len() > 16_384 {
        s.dismissed = true;
        return false;
    }
    if s.buf.windows(NEEDLE.len()).any(|w| w == NEEDLE) {
        let _ = handle.write_input(b"\r");
        s.dismissed = true;
        return true;
    }
    false
}

