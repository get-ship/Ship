//! Daemon IPC handshake file names.
//!
//! The three well-known files the daemon writes into `machine/daemon/`. This
//! is the SSOT for the enum; the heavier discovery/handle machinery
//! (DaemonHandle, path_hash, discover_running_daemon) stays in
//! `ship-substrate::home::daemon` and re-exports this type.

use serde::{Deserialize, Serialize};

/// One of the well-known daemon handshake files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DaemonFile {
    /// TCP port the daemon bound.
    Port,
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
        assert_eq!(DaemonFile::Pid.as_str(), "pid");
        assert_eq!(DaemonFile::Socket.as_str(), "socket");
    }
}
