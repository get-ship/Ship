//! Dev-time path resolution: locate a freshly-built tool binary, or the
//! current Cargo workspace root from a given cwd. Both are project-internal
//! monorepo concerns — they walk for `Cargo.toml` markers and don't run on
//! user machines. (Production project-root discovery — what the daemon uses
//! to serve a user's `.deck/` tree — is `deck_core::resolver::resolve_project_root`,
//! which goes through `SHIP_DEV_PROJECT_DIR` + the daemon's workspace file.
//! That's a deployment-aware path; this module is dev tooling only.)
//!
//! L0 substrate: every dev-tooling crate (xtask, deck-cli, sui-compiler,
//! ship-platform, deck) can depend on it without picking up host concepts.

use std::path::{Path, PathBuf};

/// Walk up from `start` to the topmost ancestor that holds a `Cargo.toml`.
/// The topmost (not the nearest) Cargo.toml is the workspace root in a
/// nested-crate layout — the inner Cargo.toml's are member manifests.
///
/// Returns `None` when no Cargo.toml is found within 16 ancestor levels;
/// callers should treat that as "not in a Cargo workspace."
pub fn workspace_root_from(start: &Path) -> Option<PathBuf> {
    let mut cur = start;
    let mut topmost: Option<PathBuf> = None;
    for _ in 0..16 {
        if cur.join("Cargo.toml").exists() {
            topmost = Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    topmost
}

/// Convenience: resolve the workspace root from the current `cwd`.
pub fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    workspace_root_from(&cwd).ok_or_else(|| {
        format!("no Cargo workspace found above {}", cwd.display())
    })
}

/// Walk up from cwd to the topmost Cargo workspace, then return the freshest
/// `target/{release,debug}/<name>` (release preferred when present).
pub fn walk_for_target_binary(name: &str) -> Result<PathBuf, String> {
    let root = workspace_root()?;
    let release = root.join("target").join("release").join(name);
    if release.exists() {
        return Ok(release);
    }
    let debug = root.join("target").join("debug").join(name);
    if debug.exists() {
        return Ok(debug);
    }
    Err(format!(
        "neither {}/target/release/{name} nor target/debug/{name} found",
        root.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_from_finds_topmost() {
        // Build a fake nested-workspace layout under tempdir.
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\nmembers=[]").unwrap();
        let inner = root.path().join("crates/inner");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("Cargo.toml"), "[package]\nname=\"inner\"").unwrap();

        // From the inner crate, we want the OUTER workspace root, not the inner.
        let found = workspace_root_from(&inner).unwrap();
        assert_eq!(found, root.path());
    }

    #[test]
    fn workspace_root_from_returns_none_outside_workspace() {
        let root = tempfile::tempdir().unwrap();
        assert!(workspace_root_from(root.path()).is_none());
    }
}
