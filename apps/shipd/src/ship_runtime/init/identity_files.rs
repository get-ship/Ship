//! Daemon identity file writers and stale-file cleanup.
//!
//! L0 init — owns the `daemon_dir` contents: pid, port, workspace, sha,
//! and the token (the token is written elsewhere; this module never
//! touches it on cleanup). Callers pass a `daemon_dir` argument —
//! typically `<root>/machine/daemon/` (singleton) or
//! `<root>/machine/daemons/<hash>/` (per-worktree, devx-04). The caller
//! picks; helpers just write/read.

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

pub fn pid_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join("pid")
}

pub fn port_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join("port")
}

pub fn ensure_daemon_dir(daemon_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(daemon_dir)
        .map_err(|e| anyhow!("create {}: {e}", daemon_dir.display()))
}

/// Returns the parent PID on Unix, or `None` on non-Unix / when no parent.
#[cfg(unix)]
pub fn parent_pid() -> Option<u32> {
    let p = unsafe { libc::getppid() };
    if p > 0 { Some(p as u32) } else { None }
}

#[cfg(not(unix))]
pub fn parent_pid() -> Option<u32> {
    None
}

/// Returns `Some(pid)` if a daemon other than us is alive per pidfile + `kill(0)`.
///
/// Returns `None` when no pid file exists, the pid file is corrupt, the pid
/// belongs to us or our parent supervisor, or the recorded process is no
/// longer alive. Excluding the parent supervisor prevents the child daemon
/// process from mistaking its own spawning supervisor for a racing daemon.
pub fn another_daemon_alive(daemon_dir: &Path) -> Option<u32> {
    // hardening-r2 §5.9: NotFound means "no pidfile, no live daemon" →
    // safe to proceed (return None). Any OTHER read error (permission
    // denied, I/O, transient disk) is NOT a "no other daemon" answer —
    // the absence of evidence is not evidence of absence. Surface it as
    // an Indeterminate sentinel below; the caller decides whether to
    // fail-closed.
    let pid_str = match std::fs::read_to_string(pid_path(daemon_dir)) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %pid_path(daemon_dir).display(),
                "shipd: pidfile read failed (non-NotFound) — \
                 unable to determine whether another daemon is alive"
            );
            // Sentinel: u32::MAX means "indeterminate" (a real daemon
            // cannot hold this pid on any platform we target). The caller
            // MUST treat Some(MAX) as a hard refusal per the fail-closed
            // contract.
            return Some(u32::MAX);
        }
    };
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

/// Hardened: returns Ok(()) only when cleanup is safe (no live daemon, no
/// unreadable pidfile, no failed-state). A non-NotFound read error on the
/// pidfile surfaces as `Err` to refuse starting a second daemon onto
/// runtime.db; the loss-of-evidence / transient-disk case is treated as a
/// hostile-state, NOT "no other daemon".
pub fn cleanup_stale_daemon_files(daemon_dir: &Path) -> Result<()> {
    if let Some(pid) = another_daemon_alive(daemon_dir) {
        if pid == u32::MAX {
            // §5.9 sentinel: pidfile unreadable for a non-NotFound reason.
            // We cannot prove no live daemon exists; refuse the boot.
            return Err(anyhow!(
                "shipd: could not read pidfile at {} (non-NotFound error); \
                 refusing to start a second daemon onto runtime.db. \
                 If no daemon is actually running, remove the daemon_dir and retry.",
                pid_path(daemon_dir).display()
            ));
        }
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
        // §5.9: only NotFound is "nothing to clean"; any other error means
        // we can't prove the pidfile is absent (race / perm / IO) and a
        // second daemon must NOT be started.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(anyhow!(
                "shipd: pidfile read failed (non-NotFound: {}); refusing to clean up",
                e
            ));
        }
    };

    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Corrupt pid file — remove it; tolerate non-existence here.
            if let Err(e) = std::fs::remove_file(&pid_file) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(
                        error = %e,
                        path = %pid_file.display(),
                        "shipd: failed to remove corrupt pidfile"
                    );
                }
            }
            return Ok(());
        }
    };

    // Process is either dead or us — safe to clean up.
    tracing::info!(old_pid = pid, "shipd: cleaning up stale daemon files from crashed instance");
    if let Err(e) = std::fs::remove_file(pid_path(daemon_dir)) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(
                error = %e,
                path = %pid_path(daemon_dir).display(),
                "shipd: failed to remove stale pidfile"
            );
        }
    }
    if let Err(e) = std::fs::remove_file(port_path(daemon_dir)) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(
                error = %e,
                path = %port_path(daemon_dir).display(),
                "shipd: failed to remove stale portfile"
            );
        }
    }
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
    for (label, path) in [
        ("port", port_path(daemon_dir)),
        ("pid", pid_path(daemon_dir)),
    ] {
        if let Err(e) = std::fs::remove_file(&path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    error = %e,
                    file = label,
                    path = %path.display(),
                    "shipd: failed to remove liveness file on shutdown"
                );
            }
        }
    }
}
