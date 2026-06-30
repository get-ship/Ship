//! `.ship/session.json` — per-worktree record of the active harness session.
//!
//! Written by the dispatch path when the harness first emits its
//! `SessionStarted` event so subsequent `resume: true` requests have
//! a deterministic place to read the harness-native session id from.
//!
//! Schema (versioned only by additive fields):
//! ```json
//! {
//!   "harness": "claude",
//!   "session_id": "<harness-native id>",
//!   "agent": "<agent_id>",
//!   "started_at": "<UTC RFC3339>"
//! }
//! ```
//!
//! Last-session-wins: each launch overwrites the file. Cross-harness
//! resume is rejected at dispatch by comparing the stored `harness`
//! against the requested one.

use std::path::Path;

use serde::{Deserialize, Serialize};

pub const SESSION_FILE_REL: &str = ".ship/session.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionFile {
    pub harness: String,
    pub session_id: String,
    pub agent: String,
    /// UTC RFC3339 timestamp captured when the file was written.
    pub started_at: String,
}

/// Write `<work_dir>/.ship/session.json`, creating `.ship/` if missing.
///
/// Overwrites any previous file (last-session-wins). Returns the path
/// that was written so callers can log it.
pub fn write_session_file(
    work_dir: &Path,
    harness: &str,
    session_id: &str,
    agent: &str,
) -> std::io::Result<std::path::PathBuf> {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(work_dir);
    std::fs::create_dir_all(&ship_dir)?;
    let path = ship_dir.join("session.json");
    let file = SessionFile {
        harness: harness.to_string(),
        session_id: session_id.to_string(),
        agent: agent.to_string(),
        started_at: chrono::Utc::now().to_rfc3339(),
    };
    let body = serde_json::to_string_pretty(&file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&path, body)?;
    Ok(path)
}

/// Read `<work_dir>/.ship/session.json`. Returns `Ok(None)` when the
/// file is absent (cold worktree); `Err` when present but malformed.
pub fn read_session_file(work_dir: &Path) -> anyhow::Result<Option<SessionFile>> {
    let path = work_dir.join(SESSION_FILE_REL);
    let body = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let file: SessionFile = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("malformed {}: {e}", path.display()))?;
    Ok(Some(file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_then_read_round_trip() {
        let tmp = TempDir::new().unwrap();
        let path = write_session_file(tmp.path(), "claude", "sess-1", "rust-lane").unwrap();
        assert!(path.ends_with(".ship/session.json"));

        let read = read_session_file(tmp.path()).unwrap().unwrap();
        assert_eq!(read.harness, "claude");
        assert_eq!(read.session_id, "sess-1");
        assert_eq!(read.agent, "rust-lane");
        assert!(!read.started_at.is_empty());
    }

    #[test]
    fn read_returns_none_when_absent() {
        let tmp = TempDir::new().unwrap();
        assert!(read_session_file(tmp.path()).unwrap().is_none());
    }

    #[test]
    fn read_errors_on_malformed_file() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(ship_substrate::ShipHome::project_ship_dir(tmp.path())).unwrap();
        std::fs::write(tmp.path().join(SESSION_FILE_REL), "not json {").unwrap();
        let err = read_session_file(tmp.path()).unwrap_err();
        assert!(err.to_string().contains("malformed"), "got: {err}");
    }

    #[test]
    fn write_overwrites_existing_file() {
        let tmp = TempDir::new().unwrap();
        write_session_file(tmp.path(), "claude", "old", "agent").unwrap();
        write_session_file(tmp.path(), "codex", "new", "agent").unwrap();
        let read = read_session_file(tmp.path()).unwrap().unwrap();
        assert_eq!(read.harness, "codex");
        assert_eq!(read.session_id, "new");
    }
}
