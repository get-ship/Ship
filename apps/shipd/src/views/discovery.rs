//! View directory discovery.
//!
//! User-authored views can live in any of these locations:
//!
//! - `<workspace>/views/<name>/`         — top-level app views (Path A,
//!   matches the `views/` convention used by built-in apps in-tree).
//! - `<workspace>/.ship/views/<name>/`   — project-scoped views colocated
//!   with other `.ship/` config.
//! - `<global>/views/<name>/`            — global views from `data_dir()`.
//!
//! The two project-scoped roots resolve from the daemon's attached
//! workspace, NOT from the Ship repo root. That's the key delta from
//! when views were hardcoded to `<repo-root>/views/` — a user adding
//! `~/my-project/views/my-view/page.sui` can now see it served at
//! `/sui/my-view` on the daemon attached to `~/my-project/`.
//!
//! The same dirs back the `/api/v1/views` listing endpoint, so views
//! authored anywhere on this list show up in the registry.

use std::path::PathBuf;

/// A directory the daemon will scan for view subdirectories, plus a
/// human label identifying its origin (used by the listing endpoint to
/// disambiguate project vs global vs user-app views).
pub struct ViewSearchDir {
    pub path: PathBuf,
    pub source: &'static str,
}

/// All directories the daemon should consult, in priority order.
///
/// Priority matters because the listing + resolver both honor the first
/// hit per view name — workspace `views/` shadows `.ship/views/`, which
/// shadows the global tier. Workspace-scoped paths win so user-authored
/// views can override built-ins by name without editing the original.
pub fn view_search_dirs() -> Vec<ViewSearchDir> {
    let mut out = Vec::with_capacity(3);
    if let Some(workspace) = workspace_root() {
        out.push(ViewSearchDir {
            path: workspace.join("views"),
            source: "workspace",
        });
        // The project tier reads from the composed overlay (local + dep-exported
        // views, namespaced) when the space has deps; the raw `.ship/` otherwise
        // (depless = identical). Backs both the listing endpoint and the
        // resolver, so dep views are discoverable AND servable. Authoring
        // surfaces (source/files/write_api) stay on local `.ship/` — you edit
        // local views, never the composed cache.
        // Marker cutover: composition anchors at the STACK ROOT; the
        // composed cache overlays it when deps exist, the raw root
        // otherwise. (`composed_root_for` takes the content root.)
        out.push(ViewSearchDir {
            path: ship_store::compose::composed_root_for(&workspace).join("views"),
            source: "project",
        });
    }
    if let Ok(global) = ship_substrate::project::get_global_dir() {
        out.push(ViewSearchDir {
            path: global.join("views"),
            source: "global",
        });
    }
    out
}

/// Best-effort workspace root resolution.
///
/// `get_stack_root` returns the stack root (env override or manifest
/// walk-up) — no normalization needed since the marker cutover.
fn workspace_root() -> Option<std::path::PathBuf> {
    ship_substrate::project::get_stack_root(None).ok()
}
