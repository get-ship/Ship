//! Daemon IPC handshake files.
//!
//! The daemon writes all four files atomically on start into `machine/daemon/`
//! and removes them on clean stop. Stale files signal a crash. This module
//! names the files; path resolution lives in `ship_home.rs`.
//!
//! Per devx-04, multi-daemon mode also lays out per-worktree daemons under
//! `machine/daemons/<hash>/` — each worktree gets its own port/token/pid set.
//! `DaemonHandle::load` and `discover_running_daemon` read these layouts.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One of the four well-known daemon handshake files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum DaemonFile {
    /// TCP port the daemon bound.
    Port,
    /// Authentication token for CLI/MCP/hook callers.
    Token,
    /// Daemon process ID.
    Pid,
    /// Optional Unix socket path (when used).
    Socket,
}

impl DaemonFile {
    /// On-disk file name under `machine/daemon/`.
    pub fn as_str(&self) -> &'static str {
        match self {
            DaemonFile::Port => "port",
            DaemonFile::Token => "token",
            DaemonFile::Pid => "pid",
            DaemonFile::Socket => "socket",
        }
    }
}

// ── Multi-daemon discovery (devx-04) ─────────────────────────────────────────

/// First 8 hex chars of `sha256(canonical_path)`. Stable across processes;
/// short enough to type. Used as the directory key under `machine/daemons/`.
pub fn path_hash(path: &Path) -> String {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(8);
    for byte in digest.iter().take(4) {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Loaded view of one daemon's IPC handshake — port, token, pid (and
/// optional workspace path / build SHA on multi-daemon installs).
#[derive(Debug, Clone)]
pub struct DaemonHandle {
    /// Directory the handle was loaded from (`machine/daemon/` or a
    /// `machine/daemons/<hash>/` sibling).
    pub dir: PathBuf,
    pub port: u16,
    pub token: String,
    pub pid: Option<u32>,
    pub workspace: Option<PathBuf>,
    pub build_sha: Option<String>,
}

impl DaemonHandle {
    /// Read `port`, `token`, `pid`, `workspace`, `sha` from `dir`. Returns
    /// `None` when the directory is missing or the handshake is incomplete
    /// (e.g. port file absent).
    pub fn load(dir: &Path) -> Option<Self> {
        let port = read_trim(dir.join("port")).and_then(|s| s.parse::<u16>().ok())?;
        let token = read_trim(dir.join("token"))?;
        let pid = read_trim(dir.join("pid")).and_then(|s| s.parse::<u32>().ok());
        let workspace = read_trim(dir.join("workspace")).map(PathBuf::from);
        let build_sha = read_trim(dir.join("sha"));
        Some(Self {
            dir: dir.to_path_buf(),
            port,
            token,
            pid,
            workspace,
            build_sha,
        })
    }

    /// True when `pid` is recorded AND the process is currently alive.
    /// Best-effort — a `false` result also covers "no pid file at all".
    pub fn is_alive(&self) -> bool {
        match self.pid {
            Some(pid) => pid_alive(pid),
            None => false,
        }
    }
}

/// Discover the running daemon for `from`. Order:
///
/// 1. If `from` resolves to a project root, look for a per-worktree daemon
///    at `machine/daemons/<hash(project_root)>/`.
/// 2. Fall back to the global `machine/daemon/`.
///
/// Returns `None` when neither location holds a complete handshake.
pub fn discover_running_daemon(from: &Path) -> Option<DaemonHandle> {
    let home = crate::ShipHome::from_env();
    if let Some(root) = crate::home::ProjectRoot::discover(from) {
        let hash = path_hash(root.path());
        let dir = home.machine_dir().join("daemons").join(&hash);
        if let Some(handle) = DaemonHandle::load(&dir) {
            return Some(handle);
        }
    }
    DaemonHandle::load(&home.machine_dir().join("daemon"))
}

fn read_trim<P: AsRef<Path>>(path: P) -> Option<String> {
    std::fs::read_to_string(path.as_ref()).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) is the standard liveness probe; signal 0 doesn't
    // deliver, just checks whether the process exists and is signalable.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(windows)]
fn pid_alive(pid: u32) -> bool {
    use std::process::Command;
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}")])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

#[cfg(not(any(unix, windows)))]
fn pid_alive(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn file_names_are_stable() {
        assert_eq!(DaemonFile::Port.as_str(), "port");
        assert_eq!(DaemonFile::Token.as_str(), "token");
        assert_eq!(DaemonFile::Pid.as_str(), "pid");
        assert_eq!(DaemonFile::Socket.as_str(), "socket");
    }

    #[test]
    fn path_hash_is_stable() {
        let tmp = TempDir::new().unwrap();
        let a = path_hash(tmp.path());
        let b = path_hash(tmp.path());
        assert_eq!(a, b);
        assert_eq!(a.len(), 8);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn path_hash_differs_across_paths() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        assert_ne!(path_hash(tmp1.path()), path_hash(tmp2.path()));
    }

    #[test]
    fn handle_load_reads_full_handshake() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("port"), "9315\n").unwrap();
        std::fs::write(tmp.path().join("token"), "tok-abc\n").unwrap();
        std::fs::write(tmp.path().join("pid"), "12345\n").unwrap();
        std::fs::write(tmp.path().join("workspace"), "/proj\n").unwrap();
        std::fs::write(tmp.path().join("sha"), "abc1234\n").unwrap();

        let handle = DaemonHandle::load(tmp.path()).unwrap();
        assert_eq!(handle.port, 9315);
        assert_eq!(handle.token, "tok-abc");
        assert_eq!(handle.pid, Some(12345));
        assert_eq!(handle.workspace.unwrap(), PathBuf::from("/proj"));
        assert_eq!(handle.build_sha.unwrap(), "abc1234");
    }

    #[test]
    fn handle_load_returns_none_when_port_missing() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("token"), "x").unwrap();
        assert!(DaemonHandle::load(tmp.path()).is_none());
    }

    #[test]
    fn handle_is_alive_reflects_pid_state() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("port"), "9315").unwrap();
        std::fs::write(tmp.path().join("token"), "x").unwrap();

        // Stale handle (no pid file) → not alive.
        let handle = DaemonHandle::load(tmp.path()).unwrap();
        assert!(!handle.is_alive());

        // Use our own pid — it must be alive (we're running this code).
        let my_pid = std::process::id();
        std::fs::write(tmp.path().join("pid"), my_pid.to_string()).unwrap();
        let handle = DaemonHandle::load(tmp.path()).unwrap();
        assert!(handle.is_alive());
    }
}
