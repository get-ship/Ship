use anyhow::{Context, Result};
use crate::daemon::paths::{log_file, pid_file, ship_dir, SHIPD_PORT};
use crate::daemon::pidfile;
use crate::daemon::pid_io::{is_process_alive, read_pid, write_pid};

/// Legacy start path wired to the pidfile adapter. Spawns the supervisor
/// process, which in turn owns the actual `shipd` child. Reachable only
/// through `PidfileAdapter::start` (or `ensure_daemon` in-process) —
/// `run_daemon` never calls this directly.
pub(crate) fn daemon_start_inner(host: &str, unsafe_skip_auth: bool) -> Result<()> {
    let pid_path = pid_file()?;
    let log_path = log_file()?;
    let global_dir = ship_dir()?;
    // Dev mode forces localhost bind regardless of --host.
    let host = if unsafe_skip_auth { "127.0.0.1".to_string() } else { host.to_string() };

    // Check if already running. The pidfile lock is the source of truth;
    // a held lock means a live supervisor exists. Falling back to the legacy
    // pid file keeps callers honest if the lock-based pidfile is missing.
    if let Some((record, true)) = pidfile::read(&global_dir)? {
        println!(
            "shipd already running (pid {}, port {}, version {})",
            record.pid, record.port, record.version
        );
        return Ok(());
    }
    if let Some(pid) = read_pid(&pid_path)?
        && is_process_alive(pid)
    {
        println!("shipd already running (pid {}, port {})", pid, SHIPD_PORT);
        return Ok(());
    }
    // Stale legacy pid file — remove it before spawning.
    let _ = std::fs::remove_file(&pid_path);

    let self_exe = std::env::current_exe().context("cannot determine own executable path")?;

    // Ensure the data dir and state/crashes/ exist.
    std::fs::create_dir_all(&global_dir).context("cannot create ~/.ship/ directory")?;
    let crashes_dir = ship_substrate::ShipHome::from_env()
        .state_dir()
        .join("crashes");
    std::fs::create_dir_all(&crashes_dir).context("cannot create state/crashes/ directory")?;
    // Fork the supervisor into the background.
    // The supervisor process watches the daemon child and restarts on crash.
    let log_handle = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("cannot open log file {}", log_path.display()))?;

    let stdout = log_handle.try_clone().context("cannot clone log file handle")?;
    let stderr = log_handle;

    let mut sup_args: Vec<&str> = vec!["daemon", "supervise", "--host", &host];
    if unsafe_skip_auth {
        sup_args.push("--unsafe-skip-auth");
    }
    let supervisor = std::process::Command::new(&self_exe)
        .args(&sup_args)
        .stdout(stdout)
        .stderr(stderr)
        .stdin(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("cannot spawn supervisor: {}", self_exe.display()))?;

    let supervisor_pid = supervisor.id();
    write_pid(&pid_path, supervisor_pid)?;

    if unsafe_skip_auth {
        eprintln!("\x1b[1;33m⚠ SHIPD DEV MODE ENABLED — auth relaxed, localhost only. Not for production.\x1b[0m");
    }
    println!(
        "shipd supervisor started (pid {}, http://{}:{}, log {})",
        supervisor_pid, host, SHIPD_PORT, log_path.display()
    );
    Ok(())
}

/// Legacy stop path wired to the pidfile adapter. Reads the supervisor
/// pid from the flock-based pidfile and sends SIGTERM, then waits for
/// the supervisor to exit so the lock is released before returning.
/// Only the pidfile adapter invokes this — systemd-managed installations
/// go through `systemctl --user stop`.
pub(crate) fn daemon_stop_inner() -> Result<()> {
    // Stop targets the daemon serving cwd. discover_running_daemon walks
    // ProjectRoot::discover → per-worktree → singleton. If nothing is live,
    // fall back to current_daemon_dir so we still tidy stale pidfiles.
    let cwd = std::env::current_dir().context("cannot determine cwd")?;
    let daemon_dir = ship_substrate::home::discover_running_daemon(&cwd)
        .map(|h| h.dir)
        .unwrap_or_else(ship_substrate::home::current_daemon_dir);
    crate::daemon::stop::daemon_stop_at(&daemon_dir)
}
