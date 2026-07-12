//! Typed stack-root resolver.
//!
//! `ProjectRoot::discover()` is the canonical answer to "given a cwd, what
//! is the stack root?": the nearest ancestor containing `ship.config.ts`.
//! No git awareness — worktree flows pass explicit roots (ship-code's
//! concern); the `.boundary` sentinel died with the marker cutover.

use std::path::{Path, PathBuf};

use crate::project::resolve_stack_root;

/// A discovered stack root — the directory containing `ship.config.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRoot(PathBuf);

impl ProjectRoot {
    /// Walk up from `from` to the nearest stack root.
    pub fn discover(from: &Path) -> Option<Self> {
        resolve_stack_root(from).map(Self)
    }

    /// The absolute path to the stack root.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discover_returns_nearest_manifest_root() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("ship.config.ts"), "export default {};\n").unwrap();

        let got = ProjectRoot::discover(&root.join("src")).unwrap();
        assert_eq!(
            fs::canonicalize(got.path()).unwrap(),
            fs::canonicalize(&root).unwrap()
        );
    }

    #[test]
    fn discover_none_without_manifest() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("plain");
        fs::create_dir_all(&dir).unwrap();
        assert!(ProjectRoot::discover(&dir).is_none());
    }
}
