//! Typed project-root resolver — see ADR `home-layout-and-scope-convention`
//! and spec `.artifacts/specs/devx/03-worktree-model.md`.
//!
//! `ProjectRoot::discover()` is the canonical answer to "given a cwd, what is
//! the project root?". It walks ancestors looking for `.git` (file or dir) or
//! `.ship/`, honoring the `.ship/.boundary` sentinel.

use std::path::{Path, PathBuf};

use crate::project::{BOUNDARY_FILE, resolve_project_ship_dir};

/// A discovered project root — the directory that contains `.ship/` (and
/// usually `.git`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRoot(PathBuf);

impl ProjectRoot {
    /// Walk up from `from` looking for the nearest project root.
    ///
    /// A project root is the directory containing `.ship/`. The walk
    /// honors `.ship/.boundary` and the linked-worktree redirect logic
    /// implemented in `ship_application::project::resolve_project_ship_dir`.
    pub fn discover(from: &Path) -> Option<Self> {
        let ship = resolve_project_ship_dir(from)?;
        let root = ship.parent()?.to_path_buf();
        Some(Self(root))
    }

    /// The absolute path to the project root (the parent of `.ship/`).
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// The project's `.ship/` directory.
    pub fn ship_dir(&self) -> PathBuf {
        crate::ShipHome::project_ship_dir(&self.0)
    }

    /// True when `.git` is a file (linked worktree) rather than a directory.
    pub fn is_worktree(&self) -> bool {
        let dot_git = self.0.join(".git");
        dot_git.is_file()
    }

    /// True when this root carries a `.ship/.boundary` sentinel.
    pub fn has_boundary(&self) -> bool {
        self.ship_dir().join(BOUNDARY_FILE).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discover_returns_root_for_plain_checkout() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(root.join(".ship")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let got = ProjectRoot::discover(&root.join("src")).unwrap();
        assert_eq!(
            fs::canonicalize(got.path()).unwrap(),
            fs::canonicalize(&root).unwrap()
        );
        assert!(!got.is_worktree());
        assert!(!got.has_boundary());
    }

    #[test]
    fn discover_honors_boundary_in_worktree() {
        // Worktree with .boundary should NOT redirect to main.
        let tmp = tempdir().unwrap();
        let main = tmp.path().join("main");
        let common_git = main.join(".git");
        let wt = tmp.path().join("wt");
        let wt_ship = wt.join(".ship");
        let wt_src = wt.join("src");

        fs::create_dir_all(main.join(".ship")).unwrap();
        fs::create_dir_all(common_git.join("worktrees").join("feat")).unwrap();
        fs::create_dir_all(&wt_ship).unwrap();
        fs::create_dir_all(&wt_src).unwrap();
        fs::write(wt_ship.join(BOUNDARY_FILE), "").unwrap();
        fs::write(
            wt.join(".git"),
            format!("gitdir: {}\n", common_git.join("worktrees/feat").display()),
        )
        .unwrap();

        let got = ProjectRoot::discover(&wt_src).unwrap();
        assert_eq!(
            fs::canonicalize(got.path()).unwrap(),
            fs::canonicalize(&wt).unwrap(),
            "boundary must pin the worktree as project root"
        );
        assert!(got.has_boundary());
    }

    #[test]
    fn discover_returns_none_outside_any_project() {
        let tmp = tempdir().unwrap();
        let outside = tmp.path().join("nowhere");
        fs::create_dir_all(&outside).unwrap();
        assert!(ProjectRoot::discover(&outside).is_none());
    }
}
