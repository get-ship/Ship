//! Active-space resolution.
//!
//! There is NO daemon "home space" and NO `global` pseudo-space. A shell runs
//! space-less (account / runtime context); a space becomes active only when one
//! is discoverable AND the runtime has opened it (a row in the `spaces`
//! Runtime-scoped projection).
//!
//! [`resolve_active_space`] is the one seam every caller uses to answer "what is
//! the active space for this probe path?":
//!
//! - a discoverable space whose path is in the `spaces` projection →
//!   `Some(content_id)` — the mint-once `space_id` the runtime assigned when the
//!   space was first opened. Identity is owned by the space; this function NEVER
//!   mints a new id. Minting happens ONLY in
//!   `ship_space::identity::ensure_identity_for_root` so the future "fenced
//!   space-config session" can relocate the on-disk home of identity in one
//!   place.
//! - no discoverable space, or a root the runtime has never opened → `None`
//!   (space-less). Calling the daemon's `Add` RPC opens one. The caller MUST
//!   treat `None` as "no active space" and error cleanly — it must NEVER
//!   fabricate a `"global"` partition.
//!
//! `.ship/` presence answers only "where is the root"; identity is never
//! derived from `.ship/` or a `ship.jsonc` field. The legacy `workspace_id`
//! (a git branch slug) must never be used as a scope/partition/delivery key.

use std::path::Path;

use crate::home::ProjectRoot;

/// Resolve the active space id for `probe`: discover the enclosing project
/// root (presence walk) and look it up in the `spaces` Runtime-scoped
/// projection. `Some(space_id)` for an opened root, else `None` (space-less).
///
/// READ-ONLY: never mints a fresh `space_id` from a path. The mint lives in
/// `ship_space::identity::ensure_identity_for_root` — the single accessor the
/// future fenced space-config session will relocate when moving identity off
/// the runtime projection. This function is the sync lookup that every layer
/// (helmd's CollectionsService, ship-code's agent session factory, shipd's
/// per-request active-space probe) reaches for when it has a probe path but
/// no space_id in hand yet.
///
/// This is the SOLE active-space resolution primitive. `None` means "no
/// active space"; the caller errors cleanly rather than inventing one.
pub fn resolve_active_space(probe: &Path) -> Option<String> {
    let root = ProjectRoot::discover(probe)?;
    // Canonicalize at lookup so a probe through a symlinked alias resolves
    // to the same row the identity accessor minted under its canonical path.
    // Identity accessor at L1 canonicalizes at mint; this read-side
    // canonicalization is symmetric — both sides reach for the canonical
    // path, so one physical directory always resolves to one space.
    let canonical = std::fs::canonicalize(root.path())
        .unwrap_or_else(|_| root.path().to_path_buf())
        .to_string_lossy()
        .into_owned();
    crate::db::spaces::find_by_path(&canonical)
        .ok()
        .flatten()
        .map(|record| record.space_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Seed a `spaces` row directly into the test-isolated runtime.db.
    ///
    /// This is the L0 read-model seam: the production writer is ship-space's
    /// `SpacesProjection` (L1), which this crate cannot depend on. The row
    /// shape is the `spaces` DDL in `db::scope_schema`.
    fn seed_space_row(space_id: &str, path: &str) {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let mut conn = crate::db::open_runtime_db(&ship_dir).unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO spaces (space_id, path, name, opened_at, last_seen_at) \
                 VALUES (?, ?, 'test', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            )
            .bind(space_id)
            .bind(path)
            .execute(&mut conn)
            .await
            .map(|_| ())
        })
        .unwrap();
    }

    #[test]
    fn opened_root_resolves_to_its_identity() {
        // The mint-once model: the `space_id` comes from the `spaces`
        // projection (runtime.db), keyed by the path the daemon opened. The
        // identity was minted by `ship_space::identity::ensure_identity_for`
        // at the runtime's first boot of that tree; `resolve_active_space`
        // NEVER mints (it's a sync lookup, not a creation primitive).
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        let ship_dir = crate::ShipHome::project_ship_dir(&root);
        fs::create_dir_all(&ship_dir).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let canonical = fs::canonicalize(&root).unwrap();
        seed_space_row("identity-id-abc", &canonical.to_string_lossy());

        let got = resolve_active_space(&root.join("src")).expect("opened → Some");
        assert_eq!(got, "identity-id-abc", "id must come from the spaces projection");
        assert!(
            !ship_dir.join("ship.jsonc").exists(),
            "resolution must never write identity into the tree"
        );
    }

    #[test]
    fn unopened_root_resolves_to_none_even_with_tree_id() {
        // Opening is deliberate (Add RPC / boot mint). A discoverable root the
        // projection does not know is space-less — even when a ship.jsonc with
        // an `id` field sits on disk. The tree is not an identity source.
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        let ship_dir = crate::ShipHome::project_ship_dir(&root);
        fs::create_dir_all(&ship_dir).unwrap();
        let jsonc = r#"{"id": "tree-id-must-be-dead"}"#;
        fs::write(ship_dir.join("ship.jsonc"), jsonc).unwrap();

        assert_eq!(
            resolve_active_space(&root),
            None,
            "unopened root → space-less; ship.jsonc#id is never identity"
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
        // The projection stores canonical paths; a probe through a symlinked
        // alias must still resolve to the recorded identity. Canonicalization
        // is owned by the identity accessor at mint-time AND by the
        // read-side resolver — both canonicalize, so one directory always
        // resolves to one space.
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(crate::ShipHome::project_ship_dir(&root)).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let canonical = fs::canonicalize(&root).unwrap();
        seed_space_row("identity-id-canon", &canonical.to_string_lossy());

        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();

        assert_eq!(
            resolve_active_space(&alias.join("src")).as_deref(),
            Some("identity-id-canon"),
            "non-canonical probe must resolve to the same canonical identity"
        );
    }

    #[test]
    fn no_project_resolves_to_no_active_space() {
        // The space-less boot: a probe outside any project must resolve to NO
        // active space (None) — NEVER the fabricated `global` partition. This
        // is the home-space kill (S2): there is no daemon home space to fall
        // back onto, and no global pseudo-space db is ever created.
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
