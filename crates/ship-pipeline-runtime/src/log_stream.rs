//! Live agent-log streaming — tails `<worktree>/.artifacts/agent-<slug>.log`.
//!
//! Relocated from `apps/shipd/src/routes/logs_api.rs`. The HTTP transport
//! (Axum SSE wrapper) remains in shipd; this module provides the pure
//! domain logic — slug validation, file discovery, byte-range reads, and
//! ANSI stripping — through a `WorktreesDirPort` so shipd can inject its
//! `ship_agents::runtime::worktrees_dir()` without this crate gaining a
//! direct dependency on the agent-run assembly.
//!
//! SSE event shape (unchanged):
//!   event: log
//!   data: <utf-8 bytes, ansi stripped, may contain newlines>
//! An initial `event: log` fires with the existing tail (last ~64 KiB),
//! then each poll tick yields only new bytes appended since the last tick.

use std::path::PathBuf;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::pipeline_ports::WorktreesDirPort;

/// How often we check the file for new bytes.
pub const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// On first connection, emit at most this many bytes from the tail so
/// a long-running job doesn't dump hundreds of KB into the client at once.
pub const INITIAL_TAIL_BYTES: u64 = 64 * 1024;

/// Slug component must be a safe filename — alphanumeric/dash/underscore
/// only. Without this guard a caller could pass `../../etc/passwd` and
/// escape the worktrees dir via the `worktrees_dir().join(slug)` step.
pub fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 128
        && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Resolve the log file path for `slug` using the injected port.
/// Returns an error string when the slug is invalid or the file does not exist.
pub fn resolve_log_path(
    port: &dyn WorktreesDirPort,
    slug: &str,
) -> Result<PathBuf, String> {
    if !is_valid_slug(slug) {
        return Err("invalid slug".to_string());
    }
    let log_path = port
        .worktrees_dir()
        .join(slug)
        .join(".artifacts")
        .join(format!("agent-{slug}.log"));

    if !log_path.exists() {
        return Err(format!("no log for slug '{slug}' at {}", log_path.display()));
    }
    Ok(log_path)
}

/// Read bytes `[from, to)` from `path`. Returns `None` on IO error or empty
/// range. Caps the initial read (when `from == 0`) at `INITIAL_TAIL_BYTES`
/// starting from the file end.
pub async fn read_chunk(path: &PathBuf, from: u64, to: u64) -> Option<Vec<u8>> {
    if to <= from {
        return None;
    }
    let mut file = tokio::fs::File::open(path).await.ok()?;
    let start = if from == 0 && to > INITIAL_TAIL_BYTES {
        to - INITIAL_TAIL_BYTES
    } else {
        from
    };
    file.seek(std::io::SeekFrom::Start(start)).await.ok()?;
    let len = (to - start) as usize;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf).await.ok()?;
    Some(buf)
}

/// Strip ANSI CSI escape sequences from a byte buffer, returning a
/// UTF-8 string. Invalid bytes are lossy-converted.

pub fn strip_ansi(input: &[u8]) -> String {
    ship_substrate::text::strip_ansi_bytes(input)
}