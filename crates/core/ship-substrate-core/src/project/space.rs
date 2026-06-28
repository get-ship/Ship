//! Active-space resolution.
//!
//! There is NO daemon "home space" and NO `global` pseudo-space. A shell runs
//! space-less (account / runtime context); a space becomes active only when one
//! is discoverable or has been explicitly activated.
//!
//! [`resolve_active_space`] is the one seam every caller uses to answer "what is
//! the active space for this probe path?":
//!
//! - a discoverable project root → `Some(content_id)` — the CONTENT-ID minted
//!   (or read) from `<root>/.ship/ship.jsonc#id`. This is the single stable
//!   identity that all operational paths (registry, event-partition, space.db)
//!   key on. NOT a path-hash and NOT a branch slug.
//! - no discoverable project, or a project whose ship.jsonc is missing/unreadable
//!   → `None` (space-less). The caller MUST treat this as "no active space" and
//!   error cleanly — it must NEVER fabricate a `"global"` partition.
//!
//! The legacy `workspace_id` (a git branch slug) must never be used as a
//! scope/partition/delivery key — see spec
//! `.artifacts/specs/space/116-event-partition.md`.

use std::path::Path;

use crate::home::ProjectRoot;
use crate::project::ensure_space_id;

/// Resolve the active space id for `probe`: `Some(content_id)` when a
/// project root with a readable `ship.jsonc` is discoverable from it,
/// else `None` (space-less).
///
/// The content-id is the `id` field in `<root>/.ship/ship.jsonc`, minted on
/// first access if absent. It is stable across renames and shared by every
/// linked worktree that redirects to the same `.ship/` tree — making it the
/// correct umbrella identity for multi-repo/multi-worktree layouts.
///
/// This is the SOLE active-space resolution primitive. `None` means "no active
/// space"; the caller errors cleanly rather than inventing one.
pub fn resolve_active_space(probe: &Path) -> Option<String> {
    let root = ProjectRoot::discover(probe)?;
    ensure_space_id(&root.ship_dir()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::path_hash;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolves_content_id_inside_a_project() {
        // The clean model: resolve_active_space must return the CONTENT-ID
        // (ship.jsonc#id), NOT the path-hash.
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        let ship_dir = root.join(".ship");
        fs::create_dir_all(&ship_dir).unwrap();
        fs::write(ship_dir.join("ship.jsonc"), r#"{"id": "test-content-id-abc"}"#).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let got = resolve_active_space(&root.join("src")).expect("project → Some");
        assert_eq!(got, "test-content-id-abc", "active space must be the content-id from ship.jsonc");
        assert_ne!(got, path_hash(&root), "must NOT be the path-hash (content-id model)");
        assert_ne!(got, "global", "must never be the global pseudo-space");
    }

    #[test]
    fn mints_and_persists_id_when_absent() {
        // When ship.jsonc has no id field, resolve_active_space mints a new
        // nanoid, writes it to ship.jsonc, and returns it.
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        let ship_dir = root.join(".ship");
        fs::create_dir_all(&ship_dir).unwrap();
        fs::write(ship_dir.join("ship.jsonc"), "{}").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let got = resolve_active_space(&root.join("src")).expect("should mint an id");
        assert!(!got.is_empty(), "minted id must be non-empty");

        // The id must now be persisted in ship.jsonc.
        let raw = fs::read_to_string(ship_dir.join("ship.jsonc")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let persisted = parsed.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        assert_eq!(got, persisted, "minted id must be persisted to ship.jsonc");

        // A second call must return the SAME id (stable after mint).
        let got2 = resolve_active_space(&root.join("src")).expect("stable id");
        assert_eq!(got, got2, "content-id is stable across calls");
    }

    #[test]
    fn no_project_resolves_to_no_active_space() {
        // The space-less boot: a probe outside any project must resolve to NO
        // active space (None) — NEVER the fabricated `global` partition. This is
        // the home-space kill (S2): there is no daemon home space to fall back
        // onto, and no global pseudo-space db is ever created.
        let tmp = tempdir().unwrap();
        let outside = tmp.path().join("nowhere");
        fs::create_dir_all(&outside).unwrap();

        assert_eq!(
            resolve_active_space(&outside),
            None,
            "no discoverable project → no active space (no `global` fabrication)"
        );
    }

    #[test]
    fn missing_ship_jsonc_resolves_to_no_active_space() {
        // A .ship/ dir with no ship.jsonc is uninitialized — the probe must
        // return None (space-less) rather than panic or use the path-hash.
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(root.join(".ship")).unwrap();
        // Deliberately no ship.jsonc.

        assert_eq!(
            resolve_active_space(&root),
            None,
            "uninitialized .ship/ (no ship.jsonc) → space-less"
        );
    }
}
