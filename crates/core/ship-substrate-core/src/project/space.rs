//! Active-space resolution.
//!
//! There is NO daemon "home space" and NO `global` pseudo-space. A shell runs
//! space-less (account / runtime context); a space becomes active only when one
//! is discoverable AND registered.
//!
//! [`resolve_active_space`] is the one seam every caller uses to answer "what is
//! the active space for this probe path?":
//!
//! - a discoverable, REGISTERED project root → `Some(content_id)` — the id in
//!   the `projects` pointer (runtime.db), minted only by `registry::add` and
//!   keyed by the canonical project path. This is the single stable identity
//!   all operational paths (event-partition, space.db) key on. NOT a
//!   path-hash, NOT a branch slug, and NOT a tree file.
//! - no discoverable project, or a root the pointer does not know → `None`
//!   (space-less). Registration is explicit (`registry::add`; the daemon
//!   registers its workspace at boot). The caller MUST treat `None` as "no
//!   active space" and error cleanly — it must NEVER fabricate a `"global"`
//!   partition.
//!
//! `.ship/` presence answers only "where is the root"; `ship.jsonc` is never
//! read or written for identity. The legacy `workspace_id` (a git branch slug)
//! must never be used as a scope/partition/delivery key — see spec
//! `.artifacts/specs/space/116-event-partition.md`.

use std::path::Path;

use crate::home::ProjectRoot;

/// Resolve the active space id for `probe`: discover the enclosing project
/// root (presence walk), canonicalize it, and look it up in the `projects`
/// pointer. `Some(content_id)` for a registered root, else `None`
/// (space-less).
///
/// This is the SOLE active-space resolution primitive. `None` means "no active
/// space"; the caller errors cleanly rather than inventing one.
pub fn resolve_active_space(probe: &Path) -> Option<String> {
    let root = ProjectRoot::discover(probe)?;
    let key = crate::db::projects::canonical_path_key(root.path());
    crate::db::projects::find_by_path(&key)
        .ok()
        .flatten()
        .map(|record| record.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Seed a `projects` row directly into the test-isolated runtime.db.
    ///
    /// This is the L0 read-model seam: the production writer is ship-space's
    /// `ProjectsProjection` (L1), which this crate cannot depend on. The row
    /// shape is the `projects` DDL in `db::scope_schema`.
    fn seed_project_row(id: &str, path: &str) {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let mut conn = crate::db::open_runtime_db(&ship_dir).unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO projects (id, path, name, created_at, last_seen_at) \
                 VALUES (?, ?, 'test', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            )
            .bind(id)
            .bind(path)
            .execute(&mut conn)
            .await
            .map(|_| ())
        })
        .unwrap();
    }

    #[test]
    fn registered_root_resolves_to_pointer_id() {
        // The pointer model: the id comes from the `projects` table
        // (runtime.db), keyed by the canonical project path. `.ship/`
        // presence only locates the root; no tree file is read or written.
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        let ship_dir = crate::ShipHome::project_ship_dir(&root);
        fs::create_dir_all(&ship_dir).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let canonical = fs::canonicalize(&root).unwrap();
        seed_project_row("pointer-id-abc", &canonical.to_string_lossy());

        let got = resolve_active_space(&root.join("src")).expect("registered → Some");
        assert_eq!(got, "pointer-id-abc", "id must come from the projects pointer");
        assert!(
            !ship_dir.join("ship.jsonc").exists(),
            "resolution must never write identity into the tree"
        );
    }

    #[test]
    fn unregistered_root_resolves_to_none_even_with_tree_id() {
        // Registration is explicit (`registry::add`). A discoverable root the
        // pointer does not know is space-less — even when a ship.jsonc with an
        // `id` field sits on disk. The tree is not an identity source.
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        let ship_dir = crate::ShipHome::project_ship_dir(&root);
        fs::create_dir_all(&ship_dir).unwrap();
        let jsonc = r#"{"id": "tree-id-must-be-dead"}"#;
        fs::write(ship_dir.join("ship.jsonc"), jsonc).unwrap();

        assert_eq!(
            resolve_active_space(&root),
            None,
            "unregistered root → space-less; ship.jsonc#id is never identity"
        );
        assert_eq!(
            fs::read_to_string(ship_dir.join("ship.jsonc")).unwrap(),
            jsonc,
            "resolution must not touch ship.jsonc"
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolution_canonicalizes_the_probe() {
        // The pointer stores canonical paths; a probe through a symlinked
        // alias must still resolve to the registered id (read-path
        // canonicalization matches `registry::add`'s write-path key).
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(crate::ShipHome::project_ship_dir(&root)).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let canonical = fs::canonicalize(&root).unwrap();
        seed_project_row("pointer-id-canon", &canonical.to_string_lossy());

        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();

        assert_eq!(
            resolve_active_space(&alias.join("src")).as_deref(),
            Some("pointer-id-canon"),
            "non-canonical probe must resolve via the canonical pointer key"
        );
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
}
