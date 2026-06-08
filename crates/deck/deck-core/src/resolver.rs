use std::path::PathBuf;
use deck_host::resolver::ViewLayout;
pub use deck_host::resolver::{ResolvedView, ResolveError, render_resolved};
/// First-party view root, in resolution order:
///
/// 1. `DeckConfig.first_party_app.root` from `<project_root>/deck.jsonc`
///    (the canonical config-driven source — deck never names the app).
/// 2. `DECK_FIRST_PARTY_VIEWS` env var (canonical, replaces the legacy
///    `SHIP_FIRST_PARTY_VIEWS`).
/// 3. `SHIP_FIRST_PARTY_VIEWS` env var (legacy fallback while hosts migrate).
///
/// When none of the above is set, the host is non-ship and there is no
/// first-party bucket. See .artifacts/asset-architecture.md.
fn first_party_root() -> Option<PathBuf> {
    if let Some(root) = resolve_project_root() {
        let cfg = deck_config::DeckConfig::load(&root);
        if let Some(fpa) = cfg.first_party_app {
            // `fpa.root` is repo-relative (e.g. `apps/shell/src`); the host
            // resolver expects an absolute path so its `canonicalize`
            // containment guard has a stable root. The relative-path form
            // would resolve against cwd, which is not what callers mean.
            return Some(root.join(fpa.root));
        }
    }
    std::env::var("DECK_FIRST_PARTY_VIEWS")
        .or_else(|_| std::env::var("SHIP_FIRST_PARTY_VIEWS"))
        .ok()
        .map(PathBuf::from)
}
/// Build a [`ViewLayout`] from the project-root `deck.jsonc` (when present)
/// or fall back to the conventional `["views", ".ship/views"]` buckets with
/// a single `page.sui` leaf. The leaf is unified across all buckets
/// (fixes the #254 `index.sui` vs `page.sui` divergence); the bucket
/// list is config-driven so a host with a different layout can override
/// it via `viewDirs` in `deck.jsonc`.
fn view_layout() -> ViewLayout {
    let mut layout = ViewLayout::conventional();
    if let Some(root) = resolve_project_root() {
        let cfg = deck_config::DeckConfig::load(&root);
        if !cfg.view_dirs.is_empty() {
            layout.view_dirs = cfg.view_dirs.clone();
        }
        if let Some(leaf) = cfg.view_leaf {
            layout.view_leaf = leaf;
        }
    }
    layout
}
pub fn resolve(name: &str, sub_path: Option<&str>) -> Result<ResolvedView, ResolveError> {
    let mut layout = view_layout();
    layout.first_party_root = first_party_root();
    if let Some(dir) = resolve_project_root() {
        deck_host::resolver::resolve(&dir, &layout, name, sub_path)
    } else {
        let cwd = match std::env::current_dir() {
            Ok(c) => c,
            Err(_) => {
                return Err(ResolveError {
                    message: format!("view not found: {} — no cwd", join_request(name, sub_path)),
                });
            }
        };
        if let Some(ship_dir) = ship_substrate::project::resolve_project_ship_dir(&cwd) { // LAYER_EXCEPTION: deck-core requires project-root resolution to locate user views on disk; no higher-layer indirection is available without a circular dep
            let project_root = ship_dir.parent().unwrap_or(&ship_dir).to_path_buf();
            deck_host::resolver::resolve(&project_root, &layout, name, sub_path)
        } else {
            Err(ResolveError {
                message: format!(
                    "view not found: {} — no project root",
                    join_request(name, sub_path),
                ),
            })
        }
    }
}

/// Project root for locating the `.deck` build-output tree. deck reads app +
/// user view/chrome bundles from `<project_root>/.deck/` — never from its own
/// crate (that holds only deck's baked runtime). Same resolution as view
/// source, so the built bundles sit beside the source tree they came from.
pub(crate) fn deck_out_root() -> Option<PathBuf> {
    resolve_project_root().map(|p| p.join(".deck"))
}

/// Resolution order:
/// 1. `DECK_PROJECT_DIR` (canonical) or legacy `SHIP_DEV_PROJECT_DIR` env var —
///    tests and dev tooling.
/// 2. The current daemon's `machine/daemons/<hash>/workspace` file — persisted
///    by `apps/shipd/src/daemon.rs::write_workspace_file` on startup. Lets
///    a per-worktree daemon serve user-authored views from the workspace
///    it was launched against even if its cwd drifts later.
pub(crate) fn resolve_project_root() -> Option<PathBuf> {
    // `DECK_PROJECT_DIR` is canonical; `SHIP_DEV_PROJECT_DIR` is the legacy
    // name, kept as a fallback until hosts migrate.
    if let Ok(env_path) =
        std::env::var("DECK_PROJECT_DIR").or_else(|_| std::env::var("SHIP_DEV_PROJECT_DIR"))
    {
        return Some(PathBuf::from(env_path));
    }
    workspace_from_daemon_file()
}

fn workspace_from_daemon_file() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let daemon_dir = ship_substrate::home::daemon::daemon_dir_for(&cwd); // LAYER_EXCEPTION: deck-core reads the daemon workspace file to resolve the project root; see Open Call #1 in shared/deck.md
    let workspace_path = daemon_dir.join("workspace");
    let raw = std::fs::read_to_string(&workspace_path).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() { return None; }
    Some(PathBuf::from(trimmed))
}

fn join_request(name: &str, sub_path: Option<&str>) -> String {
    match sub_path.filter(|s| !s.is_empty()) {
        Some(sub) => format!("{name}/{sub}"),
        None => name.to_string(),
    }
}

/// Best-effort list of top-level view names servable on this daemon, for the
/// human-facing not-found page. Scans the same on-disk roots the resolver
/// consults — the first-party bucket (config-driven via
/// `first_party_app.root` or the legacy env-var fallback) plus every
/// bucket in the host's `view_dirs` (default `["views", ".ship/views"]`).
/// Cheap (one shallow `read_dir` per root, no compilation) and graceful:
/// any missing root contributes nothing, and with no project root at
/// all the list is simply empty. Names are deduped and sorted so the
/// page renders deterministically.
pub fn list_view_names() -> Vec<String> {
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let layout = view_layout();
    if let Some(fp) = layout.first_party_root.as_deref() {
        collect_view_dirs(fp, &mut names);
    }
    if let Some(root) = resolve_project_root() {
        for bucket in &layout.view_dirs {
            collect_view_dirs(&root.join(bucket), &mut names);
        }
    }
    names.into_iter().collect()
}

/// Add each immediate child directory of `dir` whose name is a valid view name
/// to `out`. A non-directory or unreadable `dir` contributes nothing.
fn collect_view_dirs(dir: &std::path::Path, out: &mut std::collections::BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str()
            && crate::view::is_valid_view_name(name)
        {
            out.insert(name.to_string());
        }
    }
}
