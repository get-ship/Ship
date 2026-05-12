use std::path::PathBuf;

/// Worktree base directory. Routes through the canonical
/// `ship_platform::config::worktrees_dir` resolver:
///
///   1. `~/.ship/machine/local.jsonc:worktrees.dir`
///   2. `SHIP_WORKTREE_DIR` env (dev/test override)
///   3. `~/dev/<project_name>-worktrees/`
#[allow(dead_code)]
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
        // With no SHIP_WORKTREE_DIR override AND no local.jsonc setting,
        // the resolver falls back to `<HOME>/dev/<project_name>-worktrees/`.
        // Skip the test when the dev machine has either configured — the
        // resolver correctly ignores the project_name in that case and the
        // assertion would be meaningless.
        let env_set = std::env::var_os("SHIP_WORKTREE_DIR").is_some();
        let local_set = dirs::home_dir()
            .map(|h| {
                ship_substrate::ShipHome::at(h)
                    .machine_dir()
                    .join("local.jsonc")
                    .exists()
            })
            .unwrap_or(false);
        if env_set || local_set {
            eprintln!("skip: SHIP_WORKTREE_DIR or local.jsonc overrides fallback");
            return;
        }
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
