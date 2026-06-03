//! Active-space resolution.
//!
//! There is NO daemon "home space" and NO `global` pseudo-space. A shell runs
//! space-less (account / runtime context); a space becomes active only when one
//! is discoverable or has been explicitly activated.
//!
//! [`resolve_active_space`] is the one seam every caller uses to answer "what is
//! the active space for this probe path?":
//!
//! - a discoverable project root → `Some(path_hash(root))` — the path-hash
//!   space identity, matching the `projects` projection primary key
//!   (`ship_platform::projects::register`). NOT a branch slug.
//! - no discoverable project → `None` (space-less). The caller MUST treat this
//!   as "no active space" and error cleanly — it must NEVER fabricate a
//!   `"global"` partition.
//!
//! The legacy `workspace_id` (a git branch slug) must never be used as a
//! scope/partition/delivery key — see spec
//! `.artifacts/specs/space/116-event-partition.md`.

use std::path::Path;

use crate::home::{ProjectRoot, path_hash};

/// Resolve the active space id for `probe`: `Some(path_hash(root))` when a
/// project root is discoverable from it, else `None` (space-less).
///
/// This is the SOLE active-space resolution primitive. It is pure — no process
/// global, no fabricated fallback. `None` means "no active space"; the caller
/// errors cleanly rather than inventing one.
pub fn resolve_active_space(probe: &Path) -> Option<String> {
    // RED step (S2): current behaviour still fabricates the `global`
    // pseudo-space when no project is discoverable. The accompanying test
    // asserts `None`, so it FAILS here on purpose; the GREEN step removes the
    // fabrication.
    Some(
        ProjectRoot::discover(probe)
            .map(|root| path_hash(root.path()))
            .unwrap_or_else(|| "global".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolves_path_hash_inside_a_project() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(root.join(".ship")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let got = resolve_active_space(&root.join("src")).expect("project → Some");
        let expected = ProjectRoot::discover(&root.join("src"))
            .map(|r| path_hash(r.path()))
            .unwrap();
        assert_eq!(got, expected, "active space is the project root's path-hash");
        assert_ne!(got, "global", "must never be the global pseudo-space");
    }

    #[test]
    fn no_project_resolves_to_no_active_space() {
        // The space-less boot: a probe outside any project must resolve to NO
        // active space (None) — NEVER the fabricated `global` partition. This is
        // the home-space kill (S2): there is no daemon home space to fall back
        // onto, and no `spaces/global/space.db` is ever created.
        let tmp = tempdir().unwrap();
        let outside = tmp.path().join("nowhere");
        fs::create_dir_all(&outside).unwrap();

        assert_eq!(
            resolve_active_space(&outside),
            None,
            "no discoverable project → no active space (no `global` fabrication)"
        );
    }
}
