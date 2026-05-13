//! View directory discovery.
//!
//! User-authored views can live in any of these locations:
//!
//! - `<workspace>/views/<name>/`         — top-level app views (Path A,
//!   matches the `views/` convention used by built-in apps in-tree).
//! - `<workspace>/.ship/views/<name>/`   — project-scoped views colocated
//!   with other `.ship/` config.
//! - `<global>/views/<name>/`            — global views from `~/.ship/`.
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
        out.push(ViewSearchDir {
            path: workspace.join(".ship").join("views"),
            source: "project",
        });
    }
    if let Ok(global) = ship_application::project::get_global_dir() {
        out.push(ViewSearchDir {
            path: global.join("views"),
            source: "global",
        });
    }
    out
}

/// Best-effort workspace root resolution.
///
/// `get_project_dir` returns either the workspace root (env override
/// path: `SHIP_DEV_PROJECT_DIR` is the workspace, set by shipd's
/// `--workspace` flag) or the `<workspace>/.ship` directory (cwd walk).
/// We normalize by stripping a trailing `.ship` segment so callers can
/// reliably compose `<workspace>/views/` and `<workspace>/.ship/views/`.
fn workspace_root() -> Option<std::path::PathBuf> {
    let raw = ship_application::project::get_project_dir(None).ok()?;
    if raw.file_name().and_then(|s| s.to_str()) == Some(".ship") {
        raw.parent().map(|p| p.to_path_buf())
    } else {
        Some(raw)
    }
}
