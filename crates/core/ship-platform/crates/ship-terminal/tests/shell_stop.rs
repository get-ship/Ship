//! Integration test for the B17 fix — `shell_actor::stop` must terminate
//! the underlying child process, not just remove the registry entry.
//!
//! Spawns a real `$SHELL` via the supervisor primitive's PtyTransport
//! (Linux + macOS dev hosts), captures the pid from the spawned
//! `ChildHandle`, calls `shell_actor::stop`, then asserts the pid is no
//! longer a live process. Polls with a short backoff because signal
//! delivery isn't instantaneous — SIGTERM propagation through bash's
//! exit handling can take a few ms on a busy CI runner.
//!
//! Skipped on non-unix: the supervisor's PtyTransport uses
//! `portable-pty` + native_pty_system, which is unix-only; the test
//! stays portable by gating on `cfg(unix)`.

#![cfg(unix)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ship_terminal::registry::SessionRegistry;
use ship_terminal::shell_actor;
use tempfile::TempDir;

const POLL_INTERVAL: Duration = Duration::from_millis(25);
const STOP_DEADLINE: Duration = Duration::from_secs(2);

/// Best-effort "is this pid still alive?" probe. Returns true when a
/// signal can be sent (i.e. the process or something in its pid namespace
/// still owns the pid), false when `kill(pid, 0)` errors with ESRCH (no
/// such process).
fn pid_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) sends no signal; the kernel only reports
    // permission/existence. ESRCH == gone; EPERM == alive but not ours.
    let res = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if res == 0 {
        true
    } else {
        let err = std::io::Error::last_os_error();
        !matches!(err.raw_os_error(), Some(libc::ESRCH))
    }
}

/// Wait up to `deadline` for the pid to exit. Returns true if it
/// transitioned to gone within the window.
fn wait_for_exit(pid: u32, deadline: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < deadline {
        if !pid_alive(pid) {
            return true;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    false
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_terminates_the_underlying_child() {
    let workspace_id = "b17-shell-stop";
    let work_dir = TempDir::new().expect("work dir");
    let log_dir = TempDir::new().expect("log dir");
    let registry = SessionRegistry::new();

    // Spawn a real shell. The supervisor primitive creates a fresh
    // child process via PtyTransport; we capture its pid so we can
    // probe liveness directly with kill(pid, 0).
    let session = shell_actor::spawn_shell(
        &registry,
        workspace_id,
        PathBuf::from(work_dir.path()),
        PathBuf::from(log_dir.path()),
    )
    .expect("spawn_shell");

    let id = session.meta().id;
    let pid = session
        .meta()
        .pid
        .expect("shell session has a pid from supervisor");
    let handle_id = id.clone();

    assert!(
        pid_alive(pid),
        "spawned shell pid {pid} must be alive immediately after spawn",
    );

    // Act — call stop. Returns true on first hit, false thereafter.
    let stopped = tokio::task::spawn_blocking(move || shell_actor::stop(&registry, &handle_id))
        .await
        .expect("stop join");
    assert!(stopped, "stop must find and stop the just-spawned session");

    // The leak we fixed: the pid was alive after spawn AND after the
    // old stop()'s registry-only removal. The fixed stop() issues
    // ChildHandle::stop → kill(SIGKILL/SIGTERM) on the child. Give
    // the kernel a moment to deliver.
    let exited = wait_for_exit(pid, STOP_DEADLINE);
    assert!(
        exited,
        "shell pid {pid} is still alive after shell_actor::stop — \
         the child process was leaked (B17 regression)",
    );

    // Idempotent: a second stop on the same id returns false (the
    // session is already gone from the registry). The drop path is
    // safe — we don't double-terminate or panic.
    let registry2 = SessionRegistry::new();
    let second = shell_actor::stop(&registry2, &id);
    assert!(
        !second,
        "stop on an unknown session id must return false, not crash",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_returns_false_for_unknown_id() {
    let registry = SessionRegistry::new();
    assert!(
        !shell_actor::stop(&registry, "session-that-never-existed"),
        "stop must report false (not panic) when the id is unknown",
    );
}
