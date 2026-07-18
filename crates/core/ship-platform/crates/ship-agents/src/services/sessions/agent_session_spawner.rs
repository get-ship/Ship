//! Long-lived agent sessions on the supervisor primitive.
//!
//! Channels-capable harnesses (Claude) build an [`ProcessProfile`] from the
//! harness adapter's `build_interactive`, call
//! [`process_supervisor::spawn_process`], and attach a reader-middleware
//! thread that:
//!
//! 1. Auto-dismisses Claude's `--dangerously-load-development-channels`
//!    confirmation menu by sending a single CR when the menu pattern
//!    appears in the byte stream. State-aware, fires once per session.
//! 2. Feeds bytes through the harness adapter line-by-line and
//!    publishes parsed `HarnessEvent`s on the entry's `event_tx`.
//! 3. Updates the sidecar `SessionRecord` (status, last_activity).
//! 4. Emits `actor.stopped` when the byte stream closes.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::Utc;
use agent_driver::dispatch_ports::LaunchProfile;
use ship_harnesses::agent_driver::{adapter_for, HarnessEvent};
use parking_lot::Mutex;
use process_supervisor::{spawn_process, ProcessKind, ProcessProfile, TransportKind};
use tokio::sync::broadcast;

use super::registry::{Entry, Registry, SessionRecord};
use super::types::SessionStatus;

const NEEDLE: &[u8] = b"dangerously-load-development-channels";
const EVENT_CAP: usize = 256;

pub struct SpawnOutcome {
    pub entry: Entry,
    pub attached: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_or_attach(
    registry: &Registry,
    agent_id: &str,
    harness: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
    append_system_prompt: Option<String>,
) -> Result<SpawnOutcome> {
    if let Some(entry) = registry.find_agent_session(agent_id, harness) {
        return Ok(SpawnOutcome {
            entry,
            attached: true,
        });
    }

    let adapter = adapter_for(harness).map_err(|e| anyhow!(e))?;
    if !adapter.caps().persistent_session {
        anyhow::bail!(
            "agent_session_spawner::spawn_or_attach: harness '{harness}' has no persistent-session \
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

    let profile = ProcessProfile {
        actor_id_hint: None,
        kind: ProcessKind::Agent,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: launch.bin,
        args: launch.args,
        env: launch.env,
        work_dir: work_dir.clone(),
        agent_id: Some(agent_id.to_string()),
        harness: Some(harness.to_string()),
        parent_actor_id: None,
        log_dir: Some(log_dir),
        // The long-lived Claude binary has its own MCP child actor, so this
        // process does not subscribe through the supervisor mailbox.
        subscribe_namespaces: vec![],
    };

    let handle = spawn_process(profile)?;
    let (event_tx, _) = broadcast::channel::<HarnessEvent>(EVENT_CAP);

    let now = Utc::now();
    let meta = SessionRecord {
        id: handle.actor_id.clone(),
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

    spawn_reader_middleware(
        entry.clone(),
        harness.to_string(),
        agent_id.to_string(),
        work_dir,
        event_tx,
    );

    Ok(SpawnOutcome {
        entry,
        attached: false,
    })
}

fn spawn_reader_middleware(
    entry: Entry,
    harness: String,
    agent_id: String,
    work_dir: PathBuf,
    event_tx: broadcast::Sender<HarnessEvent>,
) {
    let mut rx = entry.handle.subscribe_output();
    let handle = entry.handle.clone();
    let meta = entry.meta.clone();
    let actor_id = handle.actor_id.clone();
    let spawned_at = meta.lock().spawned_at;

    tokio::spawn(async move {
        // Idle on entry — child is up but not yet thinking.
        {
            let mut m = meta.lock();
            m.status = SessionStatus::Idle;
            m.last_activity = Utc::now();
        }
        // Auto-dismiss only applies to Claude — the dev-channels TUI
        // confirmation menu is Claude-specific. Other harnesses don't
        // render it; running the matcher would just burn cycles + the
        // 16K menu_buf budget for nothing.
        let menu_state = Arc::new(Mutex::new(MenuState::default()));
        let adapter = adapter_for(&harness).ok();
        let needs_menu_dismiss = harness == "claude";
        // Bounded accumulator — a claude PTY in interactive (TUI) mode emits
        // a newline-less alternate-screen render stream; an unbounded buffer
        // would grow at the full PTY rate (shipd-leak-193).
        let mut acc = ship_files::line_accumulator::LineAccumulator::new();

        loop {
            match rx.recv().await {
                Ok(chunk) => {
                    if needs_menu_dismiss {
                        try_dismiss_menu(&handle, &menu_state, &chunk);
                    }

                    if let Some(ref adapter) = adapter {
                        for line in acc.push(&String::from_utf8_lossy(&chunk)) {
                            if let Some(ev) = adapter.parse_output(&line) {
                                if let HarnessEvent::SessionStarted { session_id } = &ev {
                                    if let Err(e) = ship_harnesses::harnesses::session_file::write_session_file(
                                        &work_dir,
                                        &harness,
                                        session_id,
                                        &agent_id,
                                    ) {
                                        tracing::warn!(
                                            harness = %harness,
                                            "session_file write failed: {e}"
                                        );
                                    }
                                }
                                let _ = event_tx.send(ev);
                            }
                        }
                    }

                    {
                        let mut m = meta.lock();
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
            let mut m = meta.lock();
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
        std::thread::spawn(move || {
            let _ = ship_substrate::db::actor_events::emit_actor_stopped(
                &stopped_id,
                &payload,
                None,
                None,
            );
        });
    });
}

#[derive(Default)]
struct MenuState {
    dismissed: bool,
    buf: Vec<u8>,
}

fn try_dismiss_menu(
    handle: &process_supervisor::ProcessHandle,
    state: &Arc<Mutex<MenuState>>,
    chunk: &[u8],
) -> bool {
    let mut s = state.lock();
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
