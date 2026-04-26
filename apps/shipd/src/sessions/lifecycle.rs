//! Session lifecycle — spawn, stop, write-stdin, read.
//!
//! Agent sessions dispatch through the provider adapter trait so
//! Claude / Codex / OpenCode all reuse the same registry + stream
//! primitives; only the CLI argv + parsing differ. Shell sessions
//! skip the adapter and spawn `$SHELL` directly.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use runtime::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use runtime::services::provider_adapter::adapter_for;
use tokio::sync::broadcast;

use super::registry::SessionRegistry;
use super::types::{
    private::Inner, PtySession, SessionHandle, SessionId, SessionKind, SessionMeta,
    SessionStatus,
};

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

    let adapter = adapter_for(provider).map_err(|e| anyhow!(e))?;

    // Channels-capable providers (Claude today) get a long-lived PTY-backed
    // process at spawn. User turns are delivered via `studio.chat_turn`
    // kernel events that the MCP `EventRelay` pushes to the connected
    // peer; the agent responds via `chat_reply`. No per-turn subprocess
    // spawn — same provider session for the lifetime of the chat.
    //
    // `spawn_command` is the same primitive `spawn_shell` uses, so chat
    // and shell sessions share the PTY-holding lifecycle. Pipeline
    // executors stay one-shot for now; future consolidation can route
    // them through this primitive too.
    if adapter.caps().channels {
        return spawn_long_lived_agent(
            registry,
            workspace_id,
            agent_id,
            provider,
            adapter.as_ref(),
            work_dir,
            log_dir,
        )
        .map(|session| SpawnOutcome { session, attached: false });
    }

    // Per-turn fallback: agent metadata only. `send_agent_message` spawns
    // a fresh headless provider run per turn and parses stream-json. Used
    // by providers without a channels capability (codex, opencode, etc.).
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("create log dir {}", log_dir.display()))?;
    let id: SessionId = runtime::gen_ulid();
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

/// Long-lived agent spawn for channels-capable providers (J2 Phase 2).
///
/// Mirrors `spawn_shell`: builds a `LaunchCommand` from the adapter's
/// `build_interactive` and hands it to `spawn_command`, which holds the
/// writer + master for the lifetime of the session. The reader thread
/// tees PTY output to a log + raw broadcast channel so subscribers can
/// inspect TUI state during debugging — but the chat UI consumes
/// structured events from `event_tx` (driven by `chat_reply` MCP tool
/// calls), not the raw byte stream.
fn spawn_long_lived_agent(
    registry: &SessionRegistry,
    workspace_id: &str,
    agent_id: &str,
    provider: &str,
    adapter: &dyn runtime::services::provider_adapter::ProviderAdapter,
    work_dir: PathBuf,
    log_dir: PathBuf,
) -> Result<PtySession> {
    let slug = format!("agent-{agent_id}-{}", runtime::gen_ulid());
    let append_system_prompt = load_agent_rules(&work_dir, agent_id);
    let profile = LaunchProfile {
        provider: Some(provider.to_string()),
        persist_session: true,
        structured_output: true,
        append_system_prompt,
        ..LaunchProfile::default()
    };
    let launch = adapter.build_interactive(&slug, &profile);
    let extras = ActorSpawnExtras {
        transport: Some("pty".to_string()),
        persistence: Some("long-lived".to_string()),
        ..Default::default()
    };
    let session = spawn_command(
        registry,
        SessionKind::Agent,
        workspace_id.to_string(),
        Some(agent_id.to_string()),
        Some(provider.to_string()),
        launch,
        work_dir,
        log_dir,
        &slug,
        extras,
    )?;

    // The dev-channels confirmation menu is auto-dismissed by the
    // reader_loop when it sees the menu pattern in the PTY output —
    // state-aware, fires exactly once per session. No timer here.

    Ok(session)
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
        channels = adapter.caps().channels,
        has_long_lived = has_long_lived_process,
        "send_agent_message: routing decision"
    );
    if adapter.caps().channels && has_long_lived_process {
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

    let session_clone = session.clone();
    let adapter_id = adapter.id().to_string();
    std::thread::spawn(move || {
        if let Err(e) = run_agent_turn(&session_clone, &launch, &adapter_id, work_dir) {
            tracing::error!(session = %session_clone.0.meta.lock().unwrap().id, error = %e, "agent turn failed");
            let _ = session_clone.0.event_tx.send(
                runtime::services::provider_adapter::ProviderEvent::Message {
                    text: format!("[ship] turn failed: {e}"),
                },
            );
            set_status(&session_clone.0, SessionStatus::Idle);
        }
    });

    Ok(())
}

fn run_agent_turn(
    session: &PtySession,
    launch: &LaunchCommand,
    provider_id: &str,
    work_dir: PathBuf,
) -> Result<()> {
    use std::io::BufRead;
    use std::process::{Command, Stdio};

    let mut cmd = Command::new(&launch.bin);
    for arg in &launch.args {
        cmd.arg(arg);
    }
    cmd.current_dir(&work_dir);
    for (k, v) in &launch.env {
        cmd.env(k, v);
    }
    // Advertised to the MCP server so `chat_reply` knows which session
    // bubble to target without the agent having to carry the id.
    cmd.env("SHIP_ACTOR_ID", &session.0.meta.lock().unwrap().id);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("spawn {}", launch.bin))?;
    {
        let mut m = session.0.meta.lock().unwrap();
        m.pid = child.id().into();
        m.last_activity = Utc::now();
    }

    let stdout = child.stdout.take().ok_or_else(|| anyhow!("no stdout"))?;
    let log_path = session.0.log_path.clone();
    let adapter = adapter_for(provider_id).map_err(|e| anyhow!(e))?;
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    let reader = std::io::BufReader::new(stdout);
    for line in reader.lines() {
        let Ok(line) = line else { continue };
        if let Some(ref mut f) = log {
            let _ = writeln!(f, "{line}");
        }
        // Broadcast raw for terminal/raw-mode subscribers.
        let _ = session
            .0
            .output_tx
            .send((line.clone() + "\n").into_bytes());
        if let Some(ev) = adapter.parse_output(&line) {
            // Capture provider session id for next-turn --resume.
            if let runtime::services::provider_adapter::ProviderEvent::SessionStarted {
                session_id: sid,
            } = &ev
            {
                let mut m = session.0.meta.lock().unwrap();
                m.provider_session_id = Some(sid.clone());
            }
            let _ = session.0.event_tx.send(ev);
            let mut m = session.0.meta.lock().unwrap();
            m.last_activity = Utc::now();
        }
    }

    let status = child.wait().ok();
    let mut m = session.0.meta.lock().unwrap();
    m.status = SessionStatus::Idle;
    m.last_activity = Utc::now();
    m.pid = None;
    if let Some(s) = status {
        m.exit_code = Some(s.code().unwrap_or(-1));
    }
    Ok(())
}

/// Spawn a plain shell session (dock terminal). No agent, no provider,
/// no parsing — raw PTY.
pub fn spawn_shell(
    registry: &SessionRegistry,
    workspace_id: &str,
    work_dir: PathBuf,
    log_dir: PathBuf,
) -> Result<PtySession> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let launch = LaunchCommand {
        bin: shell,
        args: vec!["-l".to_string()],
        env: Default::default(),
    };
    let slug = format!("shell-{}", runtime::gen_ulid());
    let outcome = spawn_command(
        registry,
        SessionKind::Shell,
        workspace_id.to_string(),
        None,
        None,
        launch,
        work_dir,
        log_dir,
        &slug,
        ActorSpawnExtras::default(),
    )?;
    Ok(outcome)
}

/// Caller-supplied actor metadata recorded into `actor.created` payload.
/// `kind` and the basics come from spawn_command; this carries the
/// agent-flavored fields the projection / dashboard cares about.
#[derive(Default, Clone)]
pub struct ActorSpawnExtras {
    pub model: Option<String>,
    pub transport: Option<String>,
    pub persistence: Option<String>,
    pub system_prompt_hash: Option<String>,
    pub capabilities: Vec<String>,
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
    let payload = runtime::events::types::ActorCreated {
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
        pid: m.pid.map(|p| p as u32),
        capabilities: extras.capabilities.clone(),
    };
    let actor_id = m.id.clone();
    let workspace_id = m.workspace_id.clone();
    drop(m);
    std::thread::spawn(move || {
        if let Err(e) = runtime::db::actor_events::emit_actor_created(
            &actor_id,
            &payload,
            Some(&workspace_id),
            None,
        ) {
            tracing::warn!(actor = %actor_id, error = %e, "actor.created emit failed");
        }
    });
}

/// Internal: actually spawn a PTY child and register it.
#[allow(clippy::too_many_arguments)]
fn spawn_command(
    registry: &SessionRegistry,
    kind: SessionKind,
    workspace_id: String,
    agent_id: Option<String>,
    provider: Option<String>,
    launch: LaunchCommand,
    work_dir: PathBuf,
    log_dir: PathBuf,
    slug: &str,
    extras: ActorSpawnExtras,
) -> Result<PtySession> {
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("create log dir {}", log_dir.display()))?;
    let log_path = log_dir.join(format!("{slug}.log"));
    let log_file = std::fs::File::create(&log_path)
        .with_context(|| format!("create log file {}", log_path.display()))?;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| anyhow!("openpty failed: {e}"))?;

    // Generate the registry id up front so SHIP_ACTOR_ID matches the
    // id the daemon and clients use to address this session. The MCP
    // peer registers its `PeerHandle.session_id` from this env var; the
    // relay's session-scoping filter only delivers `studio.chat_turn`
    // events whose `session_id` equals the peer's. Mismatched values
    // (e.g. exporting the slug) silently drop every chat turn.
    let id: SessionId = runtime::gen_ulid();

    let mut cmd = CommandBuilder::new(&launch.bin);
    for arg in &launch.args {
        cmd.arg(arg);
    }
    cmd.cwd(&work_dir);
    for (k, v) in &launch.env {
        cmd.env(k, v);
    }
    if let Some(a) = agent_id.as_deref() {
        cmd.env("SHIP_AGENT_ID", a);
    }
    cmd.env("SHIP_ACTOR_ID", &id);

    let child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| format!("spawn {}", launch.bin))?;
    drop(pair.slave);

    let pid = child.process_id();
    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| anyhow!("pty clone_reader failed: {e}"))?;

    let (output_tx, _) = broadcast::channel(OUTPUT_CHANNEL_CAP);
    let (event_tx, _) = broadcast::channel(256);

    let now = Utc::now();
    let meta = SessionMeta {
        id: id.clone(),
        kind,
        workspace_id,
        agent_id,
        provider,
        pid,
        status: SessionStatus::Spawning,
        log_path: log_path.display().to_string(),
        spawned_at: now,
        last_activity: now,
        exit_code: None,
        provider_session_id: None,
    };

    // portable-pty's `take_writer()` is single-use on the master,
    // so take it once here and hold the resulting writer for every
    // subsequent input call.
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| anyhow!("take pty writer: {e}"))?;

    let agent_id_for_kernel = meta.agent_id.clone();

    let inner = Arc::new(Inner {
        meta: StdMutex::new(meta),
        writer: StdMutex::new(Some(writer)),
        _master: StdMutex::new(Some(pair.master)),
        output_tx: output_tx.clone(),
        event_tx: event_tx.clone(),
        child: StdMutex::new(Some(child)),
        log_path: log_path.clone(),
    });

    registry.insert(id.clone(), inner.clone());

    // Register Shell + Pipeline kinds with the kernel router so they
    // appear on the mesh. Agents register via their MCP child's
    // `actor_spawn` HTTP call (which also captures the mailbox in
    // state.agent_mailboxes for SSE drain) — registering again here
    // would orphan the mailbox between paths and silently drop
    // channel pushes. Daemon-side registration is for kinds that
    // DON'T have an MCP-bridge child of their own.
    if !matches!(kind, SessionKind::Agent) {
        register_with_kernel(&id, kind, agent_id_for_kernel.as_deref());
    }

    // Reader thread: bytes → log file + broadcast. Updates status on
    // activity + quiet-timeout. Cleans up on child exit.
    let reader_inner = inner.clone();
    let reader_registry = registry.clone();
    let reader_id = id.clone();
    let reader_provider = inner.meta.lock().unwrap().provider.clone();
    std::thread::spawn(move || {
        reader_loop(
            reader,
            log_file,
            reader_inner,
            reader_registry,
            reader_id,
            reader_provider,
        );
    });

    Ok(PtySession(inner))
}

/// Distinctive unbroken byte sequence in Claude's dev-channels
/// warning. The visible "Enter to confirm" prompt has cursor-movement
/// escape sequences between words; this string is contiguous in the
/// byte stream and uniquely identifies the menu's presence.
const NEEDLE: &[u8] = b"dangerously-load-development-channels";

fn reader_loop(
    mut reader: Box<dyn Read + Send>,
    mut log_file: std::fs::File,
    inner: SessionHandle,
    registry: SessionRegistry,
    id: SessionId,
    provider: Option<String>,
) {
    set_status(&inner, SessionStatus::Idle);

    // Optional provider adapter for parsing agent output line-by-line.
    // Shell sessions leave this None and skip the parse path.
    let adapter = provider
        .as_deref()
        .and_then(|p| runtime::services::provider_adapter::adapter_for(p).ok());

    let mut buf = [0u8; 4096];
    let mut line_buf = String::new();
    // Detect Claude's `--dangerously-load-development-channels` menu
    // pattern in the rendered TUI bytes. When seen, send a single CR
    // to dismiss it. State-aware: fires once per session; we don't
    // brute-force keystrokes that can land at the main prompt and
    // submit empty inputs (which under some conditions causes early
    // exit).
    let mut menu_dismissed = false;
    let mut menu_buf: Vec<u8> = Vec::with_capacity(4096);
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = &buf[..n];
                let _ = log_file.write_all(chunk);
                let _ = log_file.flush();
                let _ = inner.output_tx.send(chunk.to_vec());

                if !menu_dismissed {
                    menu_buf.extend_from_slice(chunk);
                    if menu_buf.len() > 16_384 {
                        // Bound: if we haven't seen the menu in ~16KB
                        // of TUI output, claude isn't going to show it.
                        menu_dismissed = true;
                    } else if menu_buf.windows(NEEDLE.len()).any(|w| w == NEEDLE) {
                        // Menu rendered. Single CR accepts the
                        // pre-selected first option. Match against the
                        // unbroken token in the warning header — the
                        // visible "Enter to confirm" prompt has cursor-
                        // movement escapes between words and isn't a
                        // contiguous byte match.
                        let _ = write_input(&PtySession(inner.clone()), b"\r");
                        menu_dismissed = true;
                    }
                }
                if let Some(ref adapter) = adapter {
                    // Accumulate into a line buffer and flush on \n so
                    // parse_output sees whole lines. ANSI codes pass
                    // through — adapters know their own provider's
                    // output formatting.
                    line_buf.push_str(&String::from_utf8_lossy(chunk));
                    while let Some(nl) = line_buf.find('\n') {
                        let line: String = line_buf.drain(..=nl).collect();
                        let trimmed = line.trim_end_matches(['\n', '\r']);
                        if !trimmed.is_empty() {
                            if let Some(ev) = adapter.parse_output(trimmed) {
                                let _ = inner.event_tx.send(ev);
                            }
                        }
                    }
                }
                {
                    let mut m = inner.meta.lock().unwrap();
                    m.last_activity = Utc::now();
                    // Any output → assume thinking; idle-reaper below
                    // will flip back after a quiet window.
                    m.status = SessionStatus::Thinking;
                }
            }
            Err(e) => {
                tracing::warn!(session = %id, error = %e, "pty read error — ending session");
                break;
            }
        }
    }

    // Child exited. Capture exit code and mark Ended.
    let exit_code = {
        let mut guard = inner.child.lock().unwrap();
        guard.as_mut().and_then(|c| c.try_wait().ok().flatten()).map(|s| s.exit_code() as i32)
    };
    let (workspace_id, spawned_at) = {
        let mut m = inner.meta.lock().unwrap();
        m.status = SessionStatus::Ended;
        m.exit_code = exit_code;
        m.last_activity = Utc::now();
        (m.workspace_id.clone(), m.spawned_at)
    };
    tracing::info!(session = %id, exit = ?exit_code, "session ended");

    // Emit lifecycle event so the actors projection flips status and
    // captures duration + exit code.
    let duration_ms = (Utc::now() - spawned_at).num_milliseconds().max(0) as u64;
    let payload = runtime::events::types::ActorStopped {
        reason: "process_exit".to_string(),
        exit_code,
        duration_ms: Some(duration_ms),
    };
    let stopped_id = id.clone();
    let stopped_ws = workspace_id.clone();
    std::thread::spawn(move || {
        let _ = runtime::db::actor_events::emit_actor_stopped(
            &stopped_id,
            &payload,
            Some(&stopped_ws),
            None,
        );
    });
    // Keep in registry for history/list purposes; caller can remove
    // explicitly. A future eviction pass can clean up Ended entries
    // older than N.
    let _ = registry;
}

fn set_status(inner: &SessionHandle, status: SessionStatus) {
    let mut m = inner.meta.lock().unwrap();
    m.status = status;
    m.last_activity = Utc::now();
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
        if (now - meta.last_activity).num_seconds() >= IDLE_AFTER_SECS {
            if let Some(session) = registry.get(&meta.id) {
                let mut m = session.0.meta.lock().unwrap();
                if matches!(m.status, SessionStatus::Thinking)
                    && (Utc::now() - m.last_activity).num_seconds() >= IDLE_AFTER_SECS
                {
                    m.status = SessionStatus::Idle;
                }
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

/// Register the freshly-spawned PtySession with the kernel router so
/// it appears on the mesh. Different session kinds get different
/// subscription sets — agents listen for chat turns, shells/pipelines
/// don't subscribe to anything by default (still receive directed
/// events via target_actor_id). Failures are logged, not propagated;
/// process visibility doesn't depend on mesh registration succeeding.
fn register_with_kernel(actor_id: &str, kind: SessionKind, agent_id: Option<&str>) {
    let subscribe_namespaces: Vec<String> = match kind {
        SessionKind::Agent => vec![
            "studio.chat_turn".to_string(),
            "studio.reply".to_string(),
            "inbox.".to_string(),
            "human.".to_string(),
            "mesh.message".to_string(),
        ],
        SessionKind::Pipeline | SessionKind::Shell => vec![],
    };
    let config = runtime::events::ActorConfig {
        namespace: actor_id.to_string(),
        write_namespaces: vec!["".to_string()],
        read_namespaces: vec!["agent.".to_string()],
        subscribe_namespaces,
    };
    let actor_id_owned = actor_id.to_string();
    let _ = agent_id; // reserved for future per-agent subscription extensions
    if let Some(kr) = runtime::events::kernel_router() {
        let res = runtime::db::block_on_anyhow(async move {
            let mut kernel = kr.lock().await;
            if !kernel.has_actor(&actor_id_owned) {
                kernel
                    .spawn_actor(&actor_id_owned, config)
                    .map(|_| ())
                    .map_err(|e| anyhow!("{e}"))
            } else {
                Ok(())
            }
        });
        if let Err(e) = res {
            tracing::warn!(actor_id, "register_with_kernel failed: {e}");
        }
    }
}

/// Load an agent's `rules.inline` from `.ship/agents/<agent_id>.jsonc` in
/// the work directory. Prepended with a Ship-specific preamble so the agent
/// knows which session id to reply into. Returns `None` if the file is
/// absent, unreadable, or has no inline rules — callers fall back to vanilla
/// provider behaviour.
pub fn load_agent_rules(work_dir: &std::path::Path, agent_id: &str) -> Option<String> {
    let path = runtime::ShipHome::project_ship_dir(work_dir).join("agents").join(format!("{agent_id}.jsonc"));
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

