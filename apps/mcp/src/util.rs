use std::path::PathBuf;

/// Worktree base directory. Routes through the canonical
/// `ship_platform::config::worktrees_dir` resolver:
///
///   1. `~/.ship/machine/local.jsonc:worktrees.dir`
///   2. `SHIP_WORKTREE_DIR` env (dev/test override)
///   3. `~/dev/<project_name>-worktrees/`
pub fn configured_worktree_dir(project_root: &std::path::Path) -> PathBuf {
    let project_name = project_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());
    ship_platform::config::worktrees_dir(&project_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn falls_back_to_project_name_default() {
        // Best-effort: with no SHIP_WORKTREE_DIR override and no
        // local.jsonc setting, the resolver falls back to
        // `<HOME>/dev/<project_name>-worktrees/`. Test only verifies the
        // shape of the project-name component.
        let project_root = std::path::PathBuf::from("/home/user/my-project");
        let result = configured_worktree_dir(&project_root);
        assert_eq!(
            result.file_name(),
            Some(OsStr::new("my-project-worktrees")),
            "expected last component to be `<project>-worktrees`, got {}",
            result.display()
        );
    }
}
