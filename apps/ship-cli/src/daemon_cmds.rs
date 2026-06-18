//! CLI subcommand handlers for `ship network status` / `ship network stop`.
//!
//! These are the user-facing daemon introspectors the ship CLI dispatches
//! from its `network` subcommand. They talk to a running daemon by
//! discovering its handshake via `discover_running_daemon`; they do NOT
//! touch the local PID file directly. Daemon process control is delegated
//! to `ship network` only when the daemon is reachable on its IPC port.

use anyhow::{Result, anyhow};
use std::path::PathBuf;

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
