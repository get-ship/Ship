//! Daemon IPC handshake files.
//!
//! The daemon writes all four files atomically on start into `machine/daemon/`
//! and removes them on clean stop. Stale files signal a crash. This module
//! names the files; path resolution lives in `ship_home.rs`.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_stable() {
        assert_eq!(DaemonFile::Port.as_str(), "port");
        assert_eq!(DaemonFile::Token.as_str(), "token");
        assert_eq!(DaemonFile::Pid.as_str(), "pid");
        assert_eq!(DaemonFile::Socket.as_str(), "socket");
    }
}
