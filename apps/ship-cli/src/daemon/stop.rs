//! Daemon stop — propagate shutdown cleanly to the supervisor.
//!
//! The supervisor (`ship daemon supervise`) is the process we must target.
//! Killing the child shipd alone triggers respawn; the supervisor holds
//! the flock on `shipd.pid` and its exit is what releases that lock for
//! the next start.
//!
//! Source of truth is the flock-based pidfile, not the legacy
//! `daemon.pid`. The shipd child overwrites `daemon.pid` with its own pid
//! during startup (see `apps/shipd/src/lib.rs::write_pid_file`), so trusting
//! that file produced the original bug: `ship daemon stop` would SIGTERM
//! the child, the supervisor would respawn a replacement, and the next
//! `ship daemon start` would refuse with "already running" because the
//! supervisor still held the pidfile lock.

use anyhow::{Context, Result};
use std::path::Path;

use super::pidfile;
use super::{is_process_alive, read_pid};

/// Deadline for waiting on the supervisor to exit after SIGTERM before
/// escalating to SIGKILL. The supervisor installs a signal handler that
/// forwards SIGTERM to its child and waits for the child to drain, so a
/// few seconds is ample for a well-behaved shipd.
const STOP_GRACE: std::time::Duration = std::time::Duration::from_secs(5);
/// Poll interval while waiting for the supervisor to release the pidfile
/// lock. Tight enough that stop feels immediate, loose enough to not spin.
const STOP_POLL: std::time::Duration = std::time::Duration::from_millis(50);
/// How long to wait for the kernel to reap the process and release the
/// flock after SIGKILL — essentially "one scheduling tick, rounded up".
const KILL_SETTLE: std::time::Duration = std::time::Duration::from_secs(1);

/// Core stop logic, parameterised on the global dir so unit tests can
/// run against a temp directory without mutating the developer's
/// `~/.ship/`. The production entry point (`daemon_stop_inner`) passes
/// `ship_dir()`.
pub fn daemon_stop_at(global_dir: &Path) -> Result<()> {
    let legacy_pid_path = global_dir.join("daemon.pid");

    let (pid, source) = match resolve_supervisor_pid(global_dir)? {
        Some(v) => v,
        None => {
            println!("shipd not running");
            let _ = std::fs::remove_file(&legacy_pid_path);
            return Ok(());
        }
    };

    if !is_process_alive(pid) {
        println!("shipd not running (stale {source} pid {pid})");
        let _ = std::fs::remove_file(&legacy_pid_path);
        // A dead supervisor still leaves its pidfile on disk; remove it so
        // the next start is not confused by the artefact.
        let _ = std::fs::remove_file(pidfile::pidfile_path(global_dir));
        return Ok(());
    }

    // SIGTERM the supervisor. Its signal handler forwards SIGTERM to the
    // live child, then the supervise loop observes the stop flag and exits,
    // at which point the kernel releases the flock on shipd.pid.
    send_signal(pid, libc::SIGTERM).with_context(|| format!("SIGTERM to pid {pid}"))?;

    // Wait for the supervisor to actually exit. This is the load-bearing
    // step: returning before the flock is released lets a follow-up
    // `ship daemon start` race the dying supervisor and refuse with
    // "already running".
    if wait_supervisor_gone(pid, global_dir, STOP_GRACE) {
        let _ = std::fs::remove_file(&legacy_pid_path);
        println!("shipd stopped (supervisor pid {pid})");
        return Ok(());
    }

    // Supervisor refused to exit in time. Escalate to SIGKILL on the
    // supervisor and any still-live child to stop the respawn loop.
    let child_pid = pidfile_child_pid(global_dir);
    let _ = send_signal(pid, libc::SIGKILL);
    if let Some(cp) = child_pid {
        if cp != pid && is_process_alive(cp) {
            let _ = send_signal(cp, libc::SIGKILL);
        }
    }

    let _ = wait_supervisor_gone(pid, global_dir, KILL_SETTLE);

    let _ = std::fs::remove_file(&legacy_pid_path);
    let _ = std::fs::remove_file(pidfile::pidfile_path(global_dir));
    println!("shipd stopped (supervisor pid {pid}, escalated to SIGKILL)");
    Ok(())
}

/// Poll until the supervisor pid is gone and the pidfile lock is released,
/// or the deadline expires. Returns true if both conditions held before
/// the deadline.
fn wait_supervisor_gone(pid: u32, global_dir: &Path, grace: std::time::Duration) -> bool {
    let deadline = std::time::Instant::now() + grace;
    while std::time::Instant::now() < deadline {
        if !is_process_alive(pid) && !pidfile_lock_held(global_dir) {
            return true;
        }
        std::thread::sleep(STOP_POLL);
    }
    false
}

/// Locate the live supervisor pid. Prefers the flock-based pidfile when
/// the lock is held, because that record is the OS-enforced singleton
/// identity; falls back to the legacy `daemon.pid` only when no pidfile
/// record or lock is present (e.g. partial install or pre-upgrade state).
fn resolve_supervisor_pid(global_dir: &Path) -> Result<Option<(u32, &'static str)>> {
    if let Some((record, lock_held)) = pidfile::read(global_dir)? {
        if lock_held {
            return Ok(Some((record.pid, "supervisor")));
        }
    }
    let legacy_pid_path = global_dir.join("daemon.pid");
    if let Some(pid) = read_pid(&legacy_pid_path)? {
        return Ok(Some((pid, "daemon.pid")));
    }
    Ok(None)
}

/// Non-blocking probe: is the pidfile lock currently held by any process?
fn pidfile_lock_held(global_dir: &Path) -> bool {
    matches!(pidfile::read(global_dir), Ok(Some((_, true))))
}

/// Read the child pid that shipd wrote to `daemon.pid`, if it is still alive.
/// Used only by the SIGKILL escalation path to ensure we do not leave
/// orphaned shipd processes behind when the supervisor refuses to exit.
fn pidfile_child_pid(global_dir: &Path) -> Option<u32> {
    let pid = read_pid(&global_dir.join("daemon.pid")).ok().flatten()?;
    if is_process_alive(pid) { Some(pid) } else { None }
}

#[cfg(unix)]
fn send_signal(pid: u32, signal: i32) -> Result<()> {
    // SAFETY: `kill(2)` is a pure syscall wrapper. A negative return value
    // is translated into an error; no shared state is involved.
    let rc = unsafe { libc::kill(pid as libc::pid_t, signal) };
    if rc == 0 {
        return Ok(());
    }
    let err = std::io::Error::last_os_error();
    // ESRCH means the process is already gone — treat that as success
    // since the caller's intent (make this pid go away) is satisfied.
    if err.raw_os_error() == Some(libc::ESRCH) {
        return Ok(());
    }
    Err(err).with_context(|| format!("kill({pid}, {signal})"))
}

#[cfg(not(unix))]
fn send_signal(_pid: u32, _signal: i32) -> Result<()> {
    anyhow::bail!("signal delivery is only supported on Unix")
}

#[cfg(test)]
#[path = "stop_tests.rs"]
mod tests;
