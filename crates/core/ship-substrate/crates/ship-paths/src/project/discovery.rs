//! Stack-root discovery: walk ancestors for the `stack.config.ts` marker.
//!
//! Deliberately dumb: the nearest ancestor directory containing the manifest
//! IS the stack root. A worktree carrying its own manifest resolves itself —
//! correct under the stack model (its compile reads its own branch's config;
//! registry registration is explicit, never minted by discovery). The old
//! `.ship`-dirname walk, the git-worktree redirect, `SHIP_RESPECT_WORKTREE`,
//! and the `.boundary` sentinel died with the marker cutover — worktree
//! awareness is ship-code's concern, expressed as explicit roots.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use super::STACK_CONFIG_FILENAME;

pub(crate) fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn is_stack_root(dir: &Path) -> bool {
    let marker = dir.join(STACK_CONFIG_FILENAME);
    marker.exists() && marker.is_file()
}

/// The nearest enclosing stack root of `path` (the first ancestor — including
/// `path` itself — containing `stack.config.ts`). Pure path/fs scan.
pub fn stack_root_from_path(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| is_stack_root(ancestor))
        .map(|root| canonicalize_lossy(root))
}

/// Resolve the stack root by walking up from `start_dir`, without env
/// overrides. `None` when no ancestor carries the manifest.
pub fn resolve_stack_root(start_dir: &Path) -> Option<PathBuf> {
    stack_root_from_path(start_dir)
}

/// Resolve the stack root, honoring the `SHIP_DEV_PROJECT_DIR` override
/// (`Var::DevProjectDir`): when set to an existing directory, that directory
/// IS the root — explicit beats discovered. Otherwise walk up from
/// `start_dir` (default: cwd).
pub fn get_stack_root(start_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(env_path) = crate::env::read(crate::env::Var::DevProjectDir) {
        let path = PathBuf::from(env_path);
        if path.exists() && path.is_dir() {
            return Ok(canonicalize_lossy(&path));
        }
    }

    let start = match start_dir {
        Some(dir) => dir,
        None => env::current_dir()?,
    };
    resolve_stack_root(&start).ok_or_else(|| {
        anyhow!(
            "no stack found: no ancestor of {} contains {STACK_CONFIG_FILENAME}",
            start.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn mk_stack(root: &Path) {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(STACK_CONFIG_FILENAME), "export default {};\n").unwrap();
    }

    #[test]
    fn nearest_manifest_wins() {
        let tmp = tempdir().unwrap();
        let outer = tmp.path().join("outer");
        let inner = outer.join("packages").join("inner");
        mk_stack(&outer);
        mk_stack(&inner);
        let from = inner.join("src");
        fs::create_dir_all(&from).unwrap();
        assert_eq!(
            stack_root_from_path(&from).unwrap(),
            canonicalize_lossy(&inner),
            "the NEAREST ancestor with a manifest is the root"
        );
    }

    #[test]
    fn no_manifest_resolves_none() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("plain");
        fs::create_dir_all(&dir).unwrap();
        assert!(resolve_stack_root(&dir).is_none());
    }

    #[test]
    fn a_worktree_with_its_own_manifest_resolves_itself() {
        // The old redirect sent linked worktrees to the main checkout. Under
        // the stack model a worktree carrying its own manifest IS a stack
        // root — its compile must read its own branch's config.
        let tmp = tempdir().unwrap();
        let main = tmp.path().join("main");
        let wt = tmp.path().join("worktrees").join("feature");
        mk_stack(&main);
        mk_stack(&wt);
        fs::write(wt.join(".git"), "gitdir: /nonexistent\n").unwrap();
        assert_eq!(
            resolve_stack_root(&wt).unwrap(),
            canonicalize_lossy(&wt),
            "no redirect: the worktree's own manifest wins"
        );
    }

    #[test]
    fn env_override_is_the_root_verbatim() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("explicit");
        mk_stack(&root);
        // SAFETY: test-scoped env mutation; the env seam is the supported
        // override for dev tooling.
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", &root) };
        let got = get_stack_root(None).unwrap();
        unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") };
        assert_eq!(got, canonicalize_lossy(&root));
    }

    #[test]
    fn dir_named_ship_config_ts_is_not_a_marker() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("weird");
        fs::create_dir_all(root.join(STACK_CONFIG_FILENAME)).unwrap();
        assert!(
            resolve_stack_root(&root).is_none(),
            "the marker is a FILE; a directory with the same name does not count"
        );
    }
}
