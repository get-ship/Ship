//! Daemon lifecycle helpers: status/stop, PID/port files, stale cleanup, shutdown signals.

use anyhow::{Result, anyhow};
use std::path::Path;

/// Check whether the network daemon is running by inspecting ~/.ship/daemon.port.
pub fn network_status() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    match std::fs::read_to_string(global_dir.join("daemon.port")) {
        Ok(port) => println!("shipd: running on port {}", port.trim()),
        Err(_) => println!("shipd: not running"),
    }
    Ok(())
}

/// Send SIGTERM to the network daemon via the stored PID file.
pub fn network_stop() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    let pid_file = global_dir.join("daemon.pid");
    let pid_str = std::fs::read_to_string(&pid_file)
        .map_err(|_| anyhow!("shipd is not running (no pid file)"))?;
    let pid = pid_str.trim().to_string();
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .map_err(|e| anyhow!("failed to send SIGTERM to pid {pid}: {e}"))?;
        println!("shipd: sent SIGTERM to pid {pid}");
    }
    #[cfg(not(unix))]
    anyhow::bail!("shipd stop is not supported on this platform");
    Ok(())
}

/// Clean up stale daemon files from a previous crashed instance.
///
/// If `daemon.pid` exists but the process is dead, remove PID, port, and token
/// files so the new instance starts cleanly.
pub fn cleanup_stale_daemon_files(global_dir: &Path) {
    let pid_file = global_dir.join("daemon.pid");
    let pid_str = match std::fs::read_to_string(&pid_file) {
        Ok(s) => s,
        Err(_) => return, // No PID file — nothing to clean.
    };

    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Corrupt PID file — remove it.
            let _ = std::fs::remove_file(&pid_file);
            return;
        }
    };

    // Check if the old process is still alive.
    #[cfg(unix)]
    let alive = {
        // signal 0 checks existence without sending a real signal.
        unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
    };
    #[cfg(not(unix))]
    let alive = false;

    if alive && pid != std::process::id() {
        tracing::warn!(
            old_pid = pid,
            "shipd: another daemon is still running — aborting stale cleanup"
        );
        return;
    }

    // Old process is dead (or is us from a previous failed start). Clean up.
    tracing::info!(old_pid = pid, "shipd: cleaning up stale daemon files from crashed instance");
    let _ = std::fs::remove_file(global_dir.join("daemon.pid"));
    let _ = std::fs::remove_file(global_dir.join("daemon.port"));
    // Keep daemon.token — reuse across restarts for stability.
}

pub fn write_port_file(global_dir: &Path, port: u16) -> Result<()> {
    std::fs::write(global_dir.join("daemon.port"), port.to_string())
        .map_err(|e| anyhow!("failed to write daemon.port: {e}"))
}

pub fn write_pid_file(global_dir: &Path) -> Result<()> {
    let pid = std::process::id();
    std::fs::write(global_dir.join("daemon.pid"), pid.to_string())
        .map_err(|e| anyhow!("failed to write daemon.pid: {e}"))
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
    use super::cleanup_stale_daemon_files;
    use tempfile::TempDir;

    #[test]
    fn no_pid_file_is_noop() {
        let dir = TempDir::new().unwrap();
        // Should not panic or error.
        cleanup_stale_daemon_files(dir.path());
    }

    #[test]
    fn corrupt_pid_file_gets_removed() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("daemon.pid"), "not-a-number").unwrap();
        cleanup_stale_daemon_files(dir.path());
        assert!(!dir.path().join("daemon.pid").exists());
    }

    #[test]
    fn stale_pid_cleans_up_pid_and_port() {
        let dir = TempDir::new().unwrap();
        // PID 99999999 is almost certainly not running.
        std::fs::write(dir.path().join("daemon.pid"), "99999999").unwrap();
        std::fs::write(dir.path().join("daemon.port"), "9315").unwrap();
        std::fs::write(dir.path().join("daemon.token"), "abc123").unwrap();

        cleanup_stale_daemon_files(dir.path());

        assert!(!dir.path().join("daemon.pid").exists(), "PID file should be removed");
        assert!(!dir.path().join("daemon.port").exists(), "port file should be removed");
        assert!(dir.path().join("daemon.token").exists(), "token should be preserved");
    }
}
