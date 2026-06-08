//! Daemon stop — propagate shutdown cleanly to the supervisor.
//!
//! The supervisor (`ship daemon supervise`) is the process we must target.
//! Killing the child shipd alone triggers respawn; the supervisor holds
//! the flock on `shipd.pid` and its exit is what releases that lock for
//! the next start.

use anyhow::{Context, Result};
use std::path::Path;

use super::pidfile;
use super::pid_io::is_process_alive;

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

/// Core stop logic, parameterised on the daemon dir so unit tests can
/// run against a temp directory without mutating the developer's
/// `~/.ship/`. The production entry point (`daemon_stop_inner`) calls
/// `discover_running_daemon` and passes its `dir`.
pub fn daemon_stop_at(daemon_dir: &Path) -> Result<()> {
    let pid = match resolve_supervisor_pid(daemon_dir)? {
        Some(p) => p,
        None => {
            println!("shipd not running");
            return Ok(());
        }
    };

    if !is_process_alive(pid) {
        println!("shipd not running (stale supervisor pid {pid})");
        // A dead supervisor still leaves its pidfile on disk; remove it so
        // the next start is not confused by the artefact.
        let _ = std::fs::remove_file(pidfile::pidfile_path(daemon_dir));
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
    if wait_supervisor_gone(pid, daemon_dir, STOP_GRACE) {
        println!("shipd stopped (supervisor pid {pid})");
        return Ok(());
    }

    // Supervisor refused to exit in time. Escalate to SIGKILL.
    let _ = send_signal(pid, libc::SIGKILL);
    let _ = wait_supervisor_gone(pid, daemon_dir, KILL_SETTLE);
    let _ = std::fs::remove_file(pidfile::pidfile_path(daemon_dir));
    println!("shipd stopped (supervisor pid {pid}, escalated to SIGKILL)");
    Ok(())
}

/// Poll until the supervisor pid is gone and the pidfile lock is released,
/// or the deadline expires. Returns true if both conditions held before
/// the deadline.
fn wait_supervisor_gone(pid: u32, daemon_dir: &Path, grace: std::time::Duration) -> bool {
    let deadline = std::time::Instant::now() + grace;
    while std::time::Instant::now() < deadline {
        if !is_process_alive(pid) && !pidfile_lock_held(daemon_dir) {
            return true;
        }
        std::thread::sleep(STOP_POLL);
    }
    false
}

/// Locate the live supervisor pid via the flock-based pidfile. The lock is
/// the OS-enforced singleton identity — if it's held, the recorded pid is
/// the supervisor.
fn resolve_supervisor_pid(daemon_dir: &Path) -> Result<Option<u32>> {
    if let Some((record, lock_held)) = pidfile::read(daemon_dir)?
        && lock_held {
            return Ok(Some(record.pid));
        }
    Ok(None)
}

/// Non-blocking probe: is the pidfile lock currently held by any process?
fn pidfile_lock_held(daemon_dir: &Path) -> bool {
    matches!(pidfile::read(daemon_dir), Ok(Some((_, true))))
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
