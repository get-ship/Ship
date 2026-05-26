use std::path::PathBuf;

pub use deck_host::resolver::{ResolvedView, ResolveError, render_resolved};

/// First-party view root, supplied by the ship host via `SHIP_FIRST_PARTY_VIEWS`
/// (e.g. `<repo>/apps/shell/src`). deck never hardcodes the app's layout — when
/// the env is unset (a non-ship host) there is no first-party bucket.
fn first_party_root() -> Option<PathBuf> {
    std::env::var("SHIP_FIRST_PARTY_VIEWS").ok().map(PathBuf::from)
}

pub fn resolve(name: &str, sub_path: Option<&str>) -> Result<ResolvedView, ResolveError> {
    let fp = first_party_root();
    if let Some(dir) = resolve_project_root() {
        deck_host::resolver::resolve(&dir, fp.as_deref(), name, sub_path)
    } else {
        let cwd = match std::env::current_dir() {
            Ok(c) => c,
            Err(_) => {
                return Err(ResolveError {
                    message: format!("view not found: {} — no cwd", join_request(name, sub_path)),
                });
            }
        };
        if let Some(ship_dir) = ship_substrate::project::resolve_project_ship_dir(&cwd) {
            let project_root = ship_dir.parent().unwrap_or(&ship_dir).to_path_buf();
            deck_host::resolver::resolve(&project_root, fp.as_deref(), name, sub_path)
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
/// 1. `SHIP_DEV_PROJECT_DIR` env var — tests and dev tooling.
/// 2. The current daemon's `machine/daemons/<hash>/workspace` file — persisted
///    by `apps/shipd/src/daemon.rs::write_workspace_file` on startup. Lets
///    a per-worktree daemon serve user-authored views from the workspace
///    it was launched against even if its cwd drifts later.
fn resolve_project_root() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("SHIP_DEV_PROJECT_DIR") {
        return Some(PathBuf::from(env_path));
    }
    workspace_from_daemon_file()
}

fn workspace_from_daemon_file() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let daemon_dir = ship_substrate::home::daemon::daemon_dir_for(&cwd);
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
