//! Daemon lifecycle helpers: status/stop, PID/port files, stale cleanup, shutdown signals.
//!
//! Helpers operate on a `daemon_dir` argument — typically
//! `<root>/machine/daemon/` (singleton) or `<root>/machine/daemons/<hash>/`
//! (per-worktree, devx-04). The caller picks; helpers just write/read.

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

fn pid_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join("pid")
}

fn port_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join("port")
}

fn ensure_daemon_dir(daemon_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(daemon_dir)
        .map_err(|e| anyhow!("create {}: {e}", daemon_dir.display()))
}

/// Check whether the network daemon is running by discovering its handshake.
/// Honors per-worktree daemons (devx-04) via `discover_running_daemon`.
pub fn network_status() -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match ship_substrate::home::discover_running_daemon(&cwd) {
        Some(handle) => println!("shipd: running on port {}", handle.port),
        None => println!("shipd: not running"),
    }
    Ok(())
}

/// Send SIGTERM to the network daemon serving the current cwd.
pub fn network_stop() -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let handle = ship_substrate::home::discover_running_daemon(&cwd)
        .ok_or_else(|| anyhow!("shipd is not running (no daemon for {})", cwd.display()))?;
    let pid = match handle.pid {
        Some(p) => p,
        None => anyhow::bail!("daemon handshake at {} has no pid file", handle.dir.display()),
    };
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| anyhow!("failed to send SIGTERM to pid {pid}: {e}"))?;
        println!("shipd: sent SIGTERM to pid {pid}");
    }
    #[cfg(not(unix))]
    anyhow::bail!("shipd stop is not supported on this platform");
    Ok(())
}

/// Returns the parent PID on Unix, or `None` on non-Unix / when no parent.
#[cfg(unix)]
fn parent_pid() -> Option<u32> {
    let p = unsafe { libc::getppid() };
    if p > 0 { Some(p as u32) } else { None }
}

#[cfg(not(unix))]
fn parent_pid() -> Option<u32> {
    None
}

/// Returns `Some(pid)` if a daemon other than us is alive per pidfile + `kill(0)`.
///
/// Returns `None` when no pid file exists, the pid file is corrupt, the pid
/// belongs to us or our parent supervisor, or the recorded process is no
/// longer alive. Excluding the parent supervisor prevents the child daemon
/// process from mistaking its own spawning supervisor for a racing daemon.
pub fn another_daemon_alive(daemon_dir: &Path) -> Option<u32> {
    let pid_str = std::fs::read_to_string(pid_path(daemon_dir)).ok()?;
    let pid: u32 = pid_str.trim().parse().ok()?;

    #[cfg(unix)]
    let alive = unsafe { libc::kill(pid as libc::pid_t, 0) == 0 };
    #[cfg(not(unix))]
    let alive = false;

    let is_us = pid == std::process::id();
    let is_our_supervisor = parent_pid() == Some(pid);

    if alive && !is_us && !is_our_supervisor {
        Some(pid)
    } else {
        None
    }
}

/// Clean up stale daemon files from a previous crashed instance.
///
/// If the pid file exists but the process is dead, remove pid, port, and token
/// files so the new instance starts cleanly. If another daemon is alive,
/// refuse to continue and return an actionable error.
pub fn cleanup_stale_daemon_files(daemon_dir: &Path) -> Result<()> {
    if let Some(pid) = another_daemon_alive(daemon_dir) {
        tracing::warn!(
            old_pid = pid,
            "shipd: another daemon is already running — refusing to start"
        );
        return Err(anyhow!(
            "shipd: another daemon is already running (pid {pid}). \
             Use `ship daemon stop` first, or `ship daemon restart` for a clean replace."
        ));
    }

    let pid_file = pid_path(daemon_dir);
    let pid_str = match std::fs::read_to_string(&pid_file) {
        Ok(s) => s,
        Err(_) => return Ok(()), // No pid file — nothing to clean.
    };

    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Corrupt pid file — remove it.
            let _ = std::fs::remove_file(&pid_file);
            return Ok(());
        }
    };

    // Process is either dead or us — safe to clean up.
    tracing::info!(old_pid = pid, "shipd: cleaning up stale daemon files from crashed instance");
    let _ = std::fs::remove_file(pid_path(daemon_dir));
    let _ = std::fs::remove_file(port_path(daemon_dir));
    // Keep the token — reuse across restarts for stability.
    Ok(())
}

pub fn write_port_file(daemon_dir: &Path, port: u16) -> Result<()> {
    ensure_daemon_dir(daemon_dir)?;
    std::fs::write(port_path(daemon_dir), port.to_string())
        .map_err(|e| anyhow!("failed to write daemon port: {e}"))
}

pub fn write_pid_file(daemon_dir: &Path) -> Result<()> {
    ensure_daemon_dir(daemon_dir)?;
    let pid = std::process::id();
    std::fs::write(pid_path(daemon_dir), pid.to_string())
        .map_err(|e| anyhow!("failed to write daemon pid: {e}"))
}

/// Write the worktree path so `just status` / `ship doctor` can report it.
pub fn write_workspace_file(daemon_dir: &Path, workspace: &Path) -> Result<()> {
    ensure_daemon_dir(daemon_dir)?;
    std::fs::write(daemon_dir.join("workspace"), workspace.display().to_string())
        .map_err(|e| anyhow!("failed to write workspace file: {e}"))
}

/// Write the build SHA so `ship doctor` can detect skew between the live
/// daemon and a fresher checkout build.
pub fn write_sha_file(daemon_dir: &Path, sha: &str) -> Result<()> {
    ensure_daemon_dir(daemon_dir)?;
    std::fs::write(daemon_dir.join("sha"), sha)
        .map_err(|e| anyhow!("failed to write sha file: {e}"))
}

/// Remove the port and pid liveness files. Token is preserved so reconnecting
/// CLI clients don't need to re-handshake after a daemon restart.
pub fn remove_liveness_files(daemon_dir: &Path) {
    let _ = std::fs::remove_file(port_path(daemon_dir));
    let _ = std::fs::remove_file(pid_path(daemon_dir));
}

/// Wait for SIGINT or SIGTERM (UNIX) or just SIGINT (other platforms).
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut sigterm) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = sigterm.recv() => {}
                }
            }
            Err(e) => {
                // SIGTERM handler unavailable — fall back to SIGINT only
                tracing::warn!("could not register SIGTERM handler: {e}; SIGINT only");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests;

// Daemon lifecycle helpers — agent-stack startup, leak diagnostics,
// human-gateway bootstrap, listener binding/emit, first-run UX, and
// post-serve drain. Folded from 6 small submodules for layout coherence.
pub mod lifecycle;
mod run;

pub use run::run_network;
