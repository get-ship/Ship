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
    let slug = format!("chat-{}-{}", agent_id, runtime::gen_ulid());

    let profile = LaunchProfile {
        provider: Some(provider.to_string()),
        persist_session: true,
        ..LaunchProfile::default()
    };
    let launch = adapter.build_interactive(&slug, &profile);

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
    )?;

    Ok(SpawnOutcome {
        session,
        attached: false,
    })
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
    )?;
    Ok(outcome)
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
    cmd.env("SHIP_SESSION_ID", slug);

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

    let id: SessionId = runtime::gen_ulid();
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
    };

    let inner = Arc::new(Inner {
        meta: StdMutex::new(meta),
        master: StdMutex::new(pair.master),
        output_tx: output_tx.clone(),
        event_tx: event_tx.clone(),
        child: StdMutex::new(Some(child)),
        log_path: log_path.clone(),
    });

    registry.insert(id.clone(), inner.clone());

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
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = &buf[..n];
                let _ = log_file.write_all(chunk);
                let _ = log_file.flush();
                let _ = inner.output_tx.send(chunk.to_vec());
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
    {
        let mut m = inner.meta.lock().unwrap();
        m.status = SessionStatus::Ended;
        m.exit_code = exit_code;
        m.last_activity = Utc::now();
    }
    tracing::info!(session = %id, exit = ?exit_code, "session ended");
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
pub fn write_input(session: &PtySession, bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_INPUT_BYTES {
        bail!("input exceeds {MAX_INPUT_BYTES} byte limit");
    }
    let mut master = session.0.master.lock().unwrap();
    master
        .take_writer()
        .map_err(|e| anyhow!("take pty writer: {e}"))?
        .write_all(bytes)
        .map_err(|e| anyhow!("write pty: {e}"))?;
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
