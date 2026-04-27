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

fn token_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join("token")
}

fn ensure_daemon_dir(daemon_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(daemon_dir)
        .map_err(|e| anyhow!("create {}: {e}", daemon_dir.display()))
}

/// Check whether the network daemon is running by discovering its handshake.
/// Honors per-worktree daemons (devx-04) via `discover_running_daemon`.
pub fn network_status() -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match runtime::home::discover_running_daemon(&cwd) {
        Some(handle) => println!("shipd: running on port {}", handle.port),
        None => println!("shipd: not running"),
    }
    Ok(())
}

/// Send SIGTERM to the network daemon serving the current cwd.
pub fn network_stop() -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let handle = runtime::home::discover_running_daemon(&cwd)
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
mod stale_cleanup_tests {
    use super::*;
    use tempfile::TempDir;

    fn write_pid(dir: &Path, pid: &str) {
        let path = pid_path(dir);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, pid).unwrap();
    }

    fn write_port(dir: &Path, port: &str) {
        let path = port_path(dir);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, port).unwrap();
    }

    fn write_token(dir: &Path, token: &str) {
        let path = token_path(dir);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, token).unwrap();
    }

    #[test]
    fn no_pid_file_is_noop() {
        let dir = TempDir::new().unwrap();
        cleanup_stale_daemon_files(dir.path()).expect("no pid file should be Ok");
    }

    #[test]
    fn corrupt_pid_file_gets_removed() {
        let dir = TempDir::new().unwrap();
        write_pid(dir.path(), "not-a-number");
        cleanup_stale_daemon_files(dir.path()).expect("corrupt pid should be Ok");
        assert!(!pid_path(dir.path()).exists());
    }

    #[test]
    fn stale_pid_cleans_up_pid_and_port() {
        let dir = TempDir::new().unwrap();
        // pid 99999999 is almost certainly not running.
        write_pid(dir.path(), "99999999");
        write_port(dir.path(), "9315");
        write_token(dir.path(), "abc123");

        cleanup_stale_daemon_files(dir.path()).expect("stale pid should be Ok");

        assert!(!pid_path(dir.path()).exists(), "pid file should be removed");
        assert!(!port_path(dir.path()).exists(), "port file should be removed");
        assert!(token_path(dir.path()).exists(), "token should be preserved");
    }

    /// If the recorded pid matches our own process, treat it as a leftover from
    /// a prior failed start and clean up rather than refusing to boot.
    #[test]
    fn own_pid_is_treated_as_leftover() {
        let dir = TempDir::new().unwrap();
        let own = std::process::id().to_string();
        write_pid(dir.path(), &own);
        write_port(dir.path(), "9315");

        cleanup_stale_daemon_files(dir.path())
            .expect("own pid should not be treated as a live conflict");
        assert!(!pid_path(dir.path()).exists());
        assert!(!port_path(dir.path()).exists());
    }

    /// When another process is confirmed alive and it is not us, startup
    /// must refuse by returning Err.
    #[cfg(unix)]
    #[test]
    fn live_daemon_detected_returns_err() {
        let dir = TempDir::new().unwrap();

        // Spawn a long-running child; its pid is guaranteed alive for the test.
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        let child_pid = child.id();
        write_pid(dir.path(), &child_pid.to_string());
        write_port(dir.path(), "9315");

        let detected = another_daemon_alive(dir.path());
        assert_eq!(detected, Some(child_pid), "helper must detect live foreign pid");

        let err = cleanup_stale_daemon_files(dir.path())
            .expect_err("live conflict must return Err");
        let msg = format!("{err}");
        assert!(msg.contains("another daemon is already running"), "msg: {msg}");
        assert!(msg.contains(&child_pid.to_string()), "msg: {msg}");
        assert!(
            pid_path(dir.path()).exists(),
            "pid file must remain when refusing"
        );

        let _ = child.kill();
        let _ = child.wait();
    }

    /// The supervisor process writes the pid file with its own PID before
    /// spawning the child daemon. The child must not treat the live
    /// parent-supervisor as a racing daemon.
    #[cfg(unix)]
    #[test]
    fn live_supervisor_parent_does_not_block_child_startup() {
        let dir = TempDir::new().unwrap();
        let ppid = super::parent_pid().expect("test must run with a parent");
        write_pid(dir.path(), &ppid.to_string());

        let detected = another_daemon_alive(dir.path());
        assert_eq!(
            detected, None,
            "parent supervisor (pid {ppid}) must not be flagged as a racing daemon"
        );

        let result = cleanup_stale_daemon_files(dir.path());
        assert!(
            result.is_ok(),
            "live supervisor parent (pid {ppid}) wrongly treated as racing daemon: {:?}",
            result.err()
        );
    }

    /// Pin sup-restart-01's contract: a live third-party PID that is
    /// neither us nor our parent still refuses startup. Uses a spawned
    /// child process (whose pid is alive, not us, and not our parent).
    #[cfg(unix)]
    #[test]
    fn live_unrelated_daemon_still_blocks_startup() {
        let dir = TempDir::new().unwrap();
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        let child_pid = child.id();

        assert_ne!(child_pid, std::process::id(), "child pid must not be ours");
        assert_ne!(
            Some(child_pid),
            super::parent_pid(),
            "child pid must not be our parent"
        );

        write_pid(dir.path(), &child_pid.to_string());
        let result = cleanup_stale_daemon_files(dir.path());
        let err = result.expect_err("live unrelated daemon should block startup");
        let msg = format!("{err}");
        assert!(msg.contains("already running"), "msg: {msg}");
        assert!(msg.contains(&child_pid.to_string()), "msg: {msg}");

        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(test)]
mod path_layout_tests {
    //! Validate that daemon writes land in the directory the caller passes —
    //! either the singleton `<root>/machine/daemon/` or a per-worktree
    //! `<root>/machine/daemons/<hash>/` (devx-04). The helpers treat
    //! `daemon_dir` as opaque; policy lives in the caller.
    use super::*;
    use runtime::{DaemonFile, ShipHome};
    use tempfile::TempDir;

    #[test]
    fn write_pid_lands_in_caller_provided_dir() {
        let tmp = TempDir::new().unwrap();
        let daemon_dir = ShipHome::for_test(tmp.path()).daemon_dir();
        write_pid_file(&daemon_dir).expect("write pid");
        assert!(daemon_dir.join("pid").exists());
    }

    #[test]
    fn write_port_lands_in_caller_provided_dir() {
        let tmp = TempDir::new().unwrap();
        let daemon_dir = ShipHome::for_test(tmp.path()).daemon_dir();
        write_port_file(&daemon_dir, 9315).expect("write port");
        let body = std::fs::read_to_string(daemon_dir.join("port")).unwrap();
        assert_eq!(body.trim(), "9315");
    }

    #[test]
    fn remove_liveness_files_removes_pid_and_port() {
        let tmp = TempDir::new().unwrap();
        let daemon_dir = ShipHome::for_test(tmp.path()).daemon_dir();
        write_port_file(&daemon_dir, 9315).unwrap();
        write_pid_file(&daemon_dir).unwrap();

        remove_liveness_files(&daemon_dir);

        assert!(!daemon_dir.join("port").exists());
        assert!(!daemon_dir.join("pid").exists());
    }

    #[test]
    fn singleton_dir_matches_typed_accessor() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let daemon_dir = home.daemon_dir();
        assert_eq!(daemon_dir.join("pid"), home.daemon_file(DaemonFile::Pid));
        assert_eq!(daemon_dir.join("port"), home.daemon_file(DaemonFile::Port));
        assert_eq!(daemon_dir.join("token"), home.daemon_file(DaemonFile::Token));
    }

    #[test]
    fn worktree_dir_writes_under_per_worktree_path() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let hash = "abcd1234";
        let daemon_dir = home.worktree_daemon_dir(hash);
        write_pid_file(&daemon_dir).expect("write pid");
        write_workspace_file(&daemon_dir, std::path::Path::new("/proj/wt"))
            .expect("write workspace");

        // Verify writes land under the typed accessor's path — checking the
        // accessor itself round-trips through fs prevents the lint from
        // having to whitelist test code that joins ".ship/" literals.
        assert!(home.worktree_daemon_dir(hash).join("pid").exists());
        assert!(home.worktree_daemon_dir(hash).join("workspace").exists());
    }
}
