use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

use ship_substrate::events::EventEnvelope;
use ship_substrate::events::daemon::{
    CrashedPayload, PidfileAcquiredPayload, PidfileStaleTakenPayload,
    ShutdownCompletedPayload, event_types as daemon_events,
};
use ship_substrate::events::SqliteEventStore;
use crate::daemon::paths::{pid_file, ship_dir, SHIPD_PORT};
use crate::daemon::pidfile::{self, AcquireResult, PidfileRecord, PidfileRole};

/// Maximum restart attempts within the window before giving up.
const MAX_RESTARTS: u32 = 10;
/// Time window for counting restarts.
const RESTART_WINDOW: std::time::Duration = std::time::Duration::from_secs(600);
/// Initial backoff between restarts.
const INITIAL_BACKOFF: std::time::Duration = std::time::Duration::from_secs(1);
/// Maximum backoff between restarts.
const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(60);

/// The supervisor loop — spawns the daemon, watches it, restarts on crash.
///
/// Called via `ship daemon supervise`. Not meant to be called directly by users.
pub(super) fn daemon_supervise(host: String, dev: bool) -> Result<()> {
    let self_exe = std::env::current_exe().context("cannot determine own executable path")?;
    let global_dir = ship_dir()?;
    // Per devx-04, the supervisor co-locates with its daemon — picks the
    // per-worktree runtime/daemons/<hash>/ when launched from a source
    // directory, else falls back to the singleton. Crashes still live under
    // state/crashes because they're not per-worktree.
    let daemon_dir = ship_substrate::home::current_daemon_dir();
    let crashes_dir = ship_substrate::ShipHome::from_env()
        .state_dir()
        .join("crashes");
    // Dev mode forces localhost bind.
    let host = if dev { "127.0.0.1".to_string() } else { host };

    // Acquire the OS-enforced singleton lock. Another live supervisor
    // holding the lock means we exit cleanly with its identity.
    let record = PidfileRecord {
        pid: std::process::id(),
        port: SHIPD_PORT,
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit_sha: build_info::build_info!().git_sha.to_string(),
        started_at: chrono::Utc::now().to_rfc3339(),
        pidfile_path: String::new(),
        socket_path: String::new(),
        role: PidfileRole::Supervisor,
    };
    let supervisor_pid = record.pid;
    let pidfile_path = pidfile::pidfile_path(&daemon_dir).display().to_string();
    let _pidfile_guard = match pidfile::acquire(&daemon_dir, record)? {
        AcquireResult::Fresh(guard) => {
            emit_supervisor_event(
                &global_dir,
                daemon_events::DAEMON_PIDFILE_ACQUIRED,
                "daemon",
                &PidfileAcquiredPayload {
                    pid: supervisor_pid,
                    pidfile_path: pidfile_path.clone(),
                },
            );
            guard
        }
        AcquireResult::StaleTaken { guard, prior } => {
            emit_supervisor_event(
                &global_dir,
                daemon_events::DAEMON_PIDFILE_STALE_TAKEN,
                "daemon",
                &PidfileStaleTakenPayload {
                    new_pid: supervisor_pid,
                    prior_pid: prior.pid,
                    prior_started_at: prior.started_at,
                    prior_version: prior.version,
                },
            );
            guard
        }
        AcquireResult::Held => {
            if let Some((existing, _)) = pidfile::read(&global_dir)? {
                eprintln!(
                    "shipd already running (pid {}, port {}, version {})",
                    existing.pid, existing.port, existing.version
                );
            } else {
                eprintln!("shipd already running (pidfile lock held, no record readable)");
            }
            return Ok(());
        }
    };

    // Relay SIGTERM/SIGINT to the child, then exit cleanly.
    let child_pid: std::sync::Arc<std::sync::atomic::AtomicU32> =
        std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool> =
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Install signal handlers.
    #[cfg(unix)]
    {
        let cp = child_pid.clone();
        let sf = stop_flag.clone();
        unsafe {
            libc::signal(libc::SIGTERM, signal_handler as *const () as libc::sighandler_t);
            libc::signal(libc::SIGINT, signal_handler as *const () as libc::sighandler_t);
        }
        CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);
        STOP_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
        // We use statics for signal handler since closures can't be signal handlers.
        let _ = (cp, sf); // Suppress unused warnings; we use the statics below.
    }

    let mut restart_count: u32 = 0;
    let mut window_start = std::time::Instant::now();
    let mut backoff = INITIAL_BACKOFF;

    loop {
        // Reset restart count if outside the window.
        if window_start.elapsed() > RESTART_WINDOW {
            restart_count = 0;
            window_start = std::time::Instant::now();
            backoff = INITIAL_BACKOFF;
        }

        if restart_count >= MAX_RESTARTS {
            eprintln!(
                "shipd supervisor: {} restarts in {}s — giving up",
                MAX_RESTARTS, RESTART_WINDOW.as_secs()
            );
            write_crash_report(&crashes_dir, "max restarts exceeded", None);
            break;
        }

        // Spawn the daemon child.
        let port_str = SHIPD_PORT.to_string();
        let mut child_args: Vec<&str> = vec!["daemon", "run", "--host", &host, "--port", &port_str];
        if dev {
            child_args.push("--dev");
        }
        let mut child = std::process::Command::new(&self_exe)
            .args(&child_args)
            .stdin(std::process::Stdio::null())
            .spawn()
            .with_context(|| "supervisor: cannot spawn daemon")?;

        let pid = child.id();
        let child_started_at = std::time::Instant::now();
        eprintln!("shipd supervisor: daemon started (pid {pid})");

        #[cfg(unix)]
        CHILD_PID.store(pid, std::sync::atomic::Ordering::SeqCst);

        // Wait for child to exit.
        let status = child.wait().context("supervisor: failed to wait on child")?;
        let child_uptime_secs = child_started_at.elapsed().as_secs();

        #[cfg(unix)]
        CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);

        // Check if we were asked to stop.
        #[cfg(unix)]
        let should_stop = STOP_FLAG.load(std::sync::atomic::Ordering::SeqCst);
        #[cfg(not(unix))]
        let should_stop = false;

        if should_stop {
            eprintln!("shipd supervisor: clean shutdown (child exit: {status})");
            emit_supervisor_event(
                &global_dir,
                daemon_events::DAEMON_SHUTDOWN_COMPLETED,
                "daemon",
                &ShutdownCompletedPayload {
                    pid,
                    uptime_secs: child_uptime_secs,
                    agents_drained: 0,
                    trigger: "operator".to_string(),
                },
            );
            break;
        }

        if status.success() {
            eprintln!("shipd supervisor: daemon exited cleanly");
            emit_supervisor_event(
                &global_dir,
                daemon_events::DAEMON_SHUTDOWN_COMPLETED,
                "daemon",
                &ShutdownCompletedPayload {
                    pid,
                    uptime_secs: child_uptime_secs,
                    agents_drained: 0,
                    trigger: "child_exited_cleanly".to_string(),
                },
            );
            break;
        }

        // Crash detected.
        restart_count += 1;
        let reason = format_exit_status(&status);
        eprintln!(
            "shipd supervisor: daemon crashed ({reason}), restart {restart_count}/{MAX_RESTARTS} in {}s",
            backoff.as_secs()
        );
        write_crash_report(&crashes_dir, &reason, Some(pid));

        let (exit_code, signal) = exit_status_parts(&status);
        let last_known_event_id =
            ship_substrate::db::event_log_summary(&global_dir).latest_event_id;
        let restart_will_be_attempted = restart_count < MAX_RESTARTS;
        emit_supervisor_event(
            &global_dir,
            daemon_events::DAEMON_CRASHED,
            "daemon",
            &CrashedPayload {
                pid,
                uptime_secs: child_uptime_secs,
                exit_code,
                signal,
                last_known_event_id,
                restart_will_be_attempted,
            },
        );

        std::thread::sleep(backoff);
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }

    // Clean up PID file.
    let _ = std::fs::remove_file(pid_file().unwrap_or_default());
    Ok(())
}

fn format_exit_status(status: &std::process::ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("signal {signal}");
        }
    }
    match status.code() {
        Some(code) => format!("exit code {code}"),
        None => "unknown".to_string(),
    }
}

/// Split an exit status into (exit_code, signal_name). Either may be None.
/// Used by `daemon.crashed` payloads.
fn exit_status_parts(status: &std::process::ExitStatus) -> (Option<i32>, Option<String>) {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return (None, Some(signal_name(sig)));
        }
    }
    (status.code(), None)
}

#[cfg(unix)]
fn signal_name(sig: i32) -> String {
    match sig {
        libc::SIGHUP => "SIGHUP".into(),
        libc::SIGINT => "SIGINT".into(),
        libc::SIGQUIT => "SIGQUIT".into(),
        libc::SIGILL => "SIGILL".into(),
        libc::SIGABRT => "SIGABRT".into(),
        libc::SIGFPE => "SIGFPE".into(),
        libc::SIGKILL => "SIGKILL".into(),
        libc::SIGSEGV => "SIGSEGV".into(),
        libc::SIGPIPE => "SIGPIPE".into(),
        libc::SIGTERM => "SIGTERM".into(),
        other => format!("signal_{other}"),
    }
}

/// Append a daemon.* lifecycle event from the supervisor process.
///
/// The supervisor lives in its own process and has no in-memory kernel —
/// the platform event store is the source of truth. The daemon child reads
/// from the event log when it comes up; in-memory subscribers are the cache.
/// Failures are logged to stderr (the supervisor's own log channel) and
/// swallowed: telemetry must never block lifecycle decisions.
pub(crate) fn emit_supervisor_event<P: Serialize>(
    global_dir: &Path,
    event_type: &str,
    entity_id: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, entity_id, payload) {
        Ok(e) => e.with_actor_id("daemon"),
        Err(e) => {
            eprintln!("shipd supervisor: cannot build {event_type} envelope: {e}");
            return;
        }
    };
    // daemon.* is Runtime-scoped → write runtime.db.
    let store = match SqliteEventStore::new_for_scope(
        global_dir,
        ship_substrate::events::EventScope::Runtime,
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("shipd supervisor: cannot open event store: {e}");
            return;
        }
    };
    // Enforced write: daemon.* is Runtime-scoped and the supervisor process has
    // no session, so stamp + enforce from the daemon-internal (Runtime) binding
    // before persist.
    let binding = ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk);
    if let Err(e) = store.append_enforced(&envelope, &binding) {
        eprintln!("shipd supervisor: cannot append {event_type}: {e}");
    }
}

pub(crate) fn write_crash_report(crashes_dir: &Path, reason: &str, pid: Option<u32>) {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let filename = format!("crash-{timestamp}.json");
    let report = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "reason": reason,
        "pid": pid,
    });
    let path = crashes_dir.join(&filename);
    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&report).unwrap_or_default()) {
        eprintln!("shipd supervisor: failed to write crash report: {e}");
    }
}

// Signal handler statics (needed because signal handlers can't capture closures).
#[cfg(unix)]
static CHILD_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
#[cfg(unix)]
static STOP_FLAG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn signal_handler(_sig: libc::c_int) {
    STOP_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
    let pid = CHILD_PID.load(std::sync::atomic::Ordering::SeqCst);
    if pid != 0 {
        unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM); }
    }
}
