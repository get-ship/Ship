//! In-process registry of view-edit grants.
//!
//! When a chrome client flips edit-mode ON for a view, the daemon calls
//! [`grant`] to record `(session_id, view_name)`. Subsequent `view.*` MCP
//! tool calls check this map via [`has_grant`] before executing. Grants
//! are intentionally ephemeral — cleared on daemon restart, on `/exit`,
//! and on explicit session end. The contract states "No backend persistence
//! required for the toggle itself."
//!
//! The map keys on `(session_id, view_name)` so a single session can be in
//! edit mode for several views simultaneously without one grant clobbering
//! another.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// A single grant — what view a particular agent session may edit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewEditCapability {
    pub session_id: String,
    pub view_name: String,
}

fn registry() -> &'static Mutex<HashSet<ViewEditCapability>> {
    static REGISTRY: OnceLock<Mutex<HashSet<ViewEditCapability>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Grant `session_id` permission to call `view.*` against `view_name`.
/// Idempotent — re-granting the same pair is a no-op.
pub fn grant(session_id: &str, view_name: &str) {
    let mut guard = registry().lock().expect("view-edit capability poisoned");
    guard.insert(ViewEditCapability {
        session_id: session_id.to_string(),
        view_name: view_name.to_string(),
    });
}

/// Revoke a specific `(session, view)` grant. Returns `true` when a grant
/// existed and was removed. Idempotent: revoking a missing grant is fine.
pub fn revoke(session_id: &str, view_name: &str) -> bool {
    let mut guard = registry().lock().expect("view-edit capability poisoned");
    guard.remove(&ViewEditCapability {
        session_id: session_id.to_string(),
        view_name: view_name.to_string(),
    })
}

/// Check whether `session_id` currently holds a grant for `view_name`.
pub fn has_grant(session_id: &str, view_name: &str) -> bool {
    let guard = registry().lock().expect("view-edit capability poisoned");
    guard.contains(&ViewEditCapability {
        session_id: session_id.to_string(),
        view_name: view_name.to_string(),
    })
}

/// Drop every grant held by `session_id`. Called on session end so a
/// long-running daemon doesn't accumulate stale grants.
pub fn revoke_session(session_id: &str) {
    let mut guard = registry().lock().expect("view-edit capability poisoned");
    guard.retain(|cap| cap.session_id != session_id);
}

/// Snapshot for tests / observability. Returns the current set of grants.
pub fn grants_snapshot() -> Vec<ViewEditCapability> {
    let guard = registry().lock().expect("view-edit capability poisoned");
    guard.iter().cloned().collect()
}

/// Test helper — clear all grants so a fresh test starts from zero state.
#[cfg(test)]
pub fn reset_for_test() {
    let mut guard = registry().lock().expect("view-edit capability poisoned");
    guard.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_and_has_round_trip() {
        // Use unique ids to avoid colliding with parallel tests on the
        // process-wide singleton.
        grant("sess-grhr-1", "view-grhr-1");
        assert!(has_grant("sess-grhr-1", "view-grhr-1"));
        assert!(!has_grant("sess-grhr-1", "other-view"));
        assert!(!has_grant("other-sess", "view-grhr-1"));
        revoke("sess-grhr-1", "view-grhr-1");
    }

    #[test]
    fn revoke_returns_true_when_present_false_otherwise() {
        grant("sess-rev-1", "view-rev-1");
        assert!(revoke("sess-rev-1", "view-rev-1"));
        assert!(!revoke("sess-rev-1", "view-rev-1"));
    }

    #[test]
    fn revoke_session_drops_all_grants_for_that_session() {
        grant("sess-rsa-1", "view-rsa-a");
        grant("sess-rsa-1", "view-rsa-b");
        grant("sess-rsa-other", "view-rsa-c");
        revoke_session("sess-rsa-1");
        assert!(!has_grant("sess-rsa-1", "view-rsa-a"));
        assert!(!has_grant("sess-rsa-1", "view-rsa-b"));
        assert!(has_grant("sess-rsa-other", "view-rsa-c"));
        revoke("sess-rsa-other", "view-rsa-c");
    }
}
