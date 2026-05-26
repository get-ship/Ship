//! Daemon lifecycle — the shell supervises the *existing* `ship` daemon rather
//! than embedding a second copy. One daemon implementation, one IPC handshake;
//! the shell is just another (operator) client. Discovery/handshake reading is
//! reused from `ship-substrate` so the shell never re-expresses those layouts.

use serde::Serialize;
use std::time::Duration;

/// What the webview needs to point its Connect client at the right loopback
/// endpoint and to render daemon status. No token here — the token reaches the
/// webview through the operator-session bootstrap, not this status call.
#[derive(Debug, Clone, Serialize)]
pub struct DaemonInfo {
    /// True when a handshake exists AND its recorded pid is alive.
    pub running: bool,
    /// Loopback port the Connect client should target, when known.
    pub port: Option<u16>,
    pub pid: Option<u32>,
    /// Host socket path, present only when the daemon has bound it (agent/CLI
    /// transport — the shell itself uses loopback, not this).
    pub socket: Option<String>,
    /// IPC directory the daemon serving the shell's cwd reads/writes.
    pub dir: String,
}

fn from_handle(h: ship_substrate::home::DaemonHandle) -> DaemonInfo {
    let sock = h.dir.join("ship.sock");
    DaemonInfo {
        running: h.is_alive(),
        port: Some(h.port),
        pid: h.pid,
        socket: sock.exists().then(|| sock.display().to_string()),
        dir: h.dir.display().to_string(),
    }
}

fn cwd() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Current daemon status for the shell's working directory. Read-only; never
/// starts anything.
#[tauri::command]
pub fn daemon_status() -> DaemonInfo {
    let here = cwd();
    match ship_substrate::home::discover_running_daemon(&here) {
        Some(h) => from_handle(h),
        None => DaemonInfo {
            running: false,
            port: None,
            pid: None,
            socket: None,
            dir: ship_substrate::home::daemon_dir_for(&here).display().to_string(),
        },
    }
}

/// Start the daemon by delegating to the `ship` CLI (`ship daemon start`), then
/// poll the handshake until it's alive or we time out (~5s). Delegating keeps a
/// single daemon-launch path; the shell does not reimplement supervision.
#[tauri::command]
pub fn daemon_start() -> Result<DaemonInfo, String> {
    if let Some(h) = ship_substrate::home::discover_running_daemon(&cwd()) {
        if h.is_alive() {
            return Ok(from_handle(h));
        }
    }
    std::process::Command::new("ship")
        .args(["daemon", "start"])
        .spawn()
        .map_err(|e| format!("spawning `ship daemon start`: {e}"))?;

    for _ in 0..50 {
        if let Some(h) = ship_substrate::home::discover_running_daemon(&cwd()) {
            if h.is_alive() {
                return Ok(from_handle(h));
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("daemon did not report a live handshake within 5s".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::home::DaemonHandle;
    use tempfile::TempDir;

    fn handle_in(dir: &std::path::Path, pid: Option<u32>) -> DaemonHandle {
        DaemonHandle {
            dir: dir.to_path_buf(),
            port: 9315,
            token: "tok".into(),
            pid,
            workspace: None,
            build_sha: None,
        }
    }

    #[test]
    fn from_handle_maps_fields_and_alive_pid() {
        let tmp = TempDir::new().unwrap();
        // Our own pid is alive by construction (we're running this test).
        let info = from_handle(handle_in(tmp.path(), Some(std::process::id())));
        assert!(info.running, "live pid should map to running=true");
        assert_eq!(info.port, Some(9315));
        assert_eq!(info.pid, Some(std::process::id()));
        assert_eq!(info.dir, tmp.path().display().to_string());
    }

    #[test]
    fn from_handle_running_false_without_pid() {
        let tmp = TempDir::new().unwrap();
        let info = from_handle(handle_in(tmp.path(), None));
        assert!(!info.running, "no pid → not running");
        assert_eq!(info.pid, None);
    }

    #[test]
    fn from_handle_socket_reflects_ship_sock_presence() {
        let tmp = TempDir::new().unwrap();
        // Absent until the daemon binds it.
        assert!(from_handle(handle_in(tmp.path(), None)).socket.is_none());
        // Present once `ship.sock` exists in the daemon dir.
        let sock = tmp.path().join("ship.sock");
        std::fs::write(&sock, b"").unwrap();
        assert_eq!(
            from_handle(handle_in(tmp.path(), None)).socket,
            Some(sock.display().to_string())
        );
    }
}
