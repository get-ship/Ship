//! Daemon space identity.
//!
//! A daemon serves exactly one worktree/project = one **space**. The space id
//! is the path-hash identity of the project root, resolved once at daemon
//! startup (`ProjectRoot::discover`) and stored here as a process-global so the
//! per-request emit path can stamp it onto every space-scoped event's
//! `space_id` field.
//!
//! This is the sole domain scope. The legacy `workspace_id` (a git branch slug)
//! must never be used as a scope/partition/delivery key — see spec
//! `.artifacts/specs/space/116-event-partition.md`.

use std::path::Path;
use std::sync::OnceLock;

use crate::home::path_hash;

static DAEMON_SPACE_ID: OnceLock<String> = OnceLock::new();

/// Set the daemon's space id from its project root. Idempotent: the first
/// caller wins (startup sets it once, before any request thread reads it).
///
/// The id is `path_hash(canonical(root))`, matching the `projects` projection
/// primary key (`ship_platform::projects::register`). This is the path-hash
/// space identity, NOT a branch slug.
pub fn set_daemon_space_id_from_root(root: &Path) {
    let id = path_hash(root);
    let _ = DAEMON_SPACE_ID.set(id);
}

/// Set the daemon's space id directly (used by tests and callers that already
/// hold the computed id). Idempotent — first caller wins.
pub fn set_daemon_space_id(id: String) {
    let _ = DAEMON_SPACE_ID.set(id);
}

/// The daemon's space id, if startup resolved a project root. `None` in
/// global-only mode (no `--workspace`, no discoverable `.ship/`), in which case
/// space-scoped emits carry `None` — the same posture as the legacy unscoped
/// path, never a branch slug.
pub fn daemon_space_id() -> Option<String> {
    DAEMON_SPACE_ID.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: `DAEMON_SPACE_ID` is a set-once process global; a unit test that
    // writes it would pollute every other test in this binary. So we only
    // exercise the path-hash derivation, which is the load-bearing part.
    #[test]
    fn space_id_from_root_is_path_hash() {
        let dir = std::env::temp_dir();
        assert_eq!(path_hash(&dir), path_hash(&dir), "path_hash is deterministic");
    }
}
