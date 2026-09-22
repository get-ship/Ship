//! Git hook guards — pre-push (block `refs/heads/job/*` pushes) and
//! pre-commit (refuse `.artifacts/` paths and `.gitignore` weakening).
//!
//! Both hooks install into the repo's **shared git-common-dir** so every
//! worktree inherits them. Static scripts are embedded at compile time;
//! install is idempotent (marker match = no-op) and sets the executable bit.
//!
//! Used by `GitService/Overview` so every repo the SPA opens is guarded on
//! first touch. Dispatch installs the same hooks via this module.

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Marker embedded in the pre-push hook so a re-install is a no-op.
pub const PRE_PUSH_HOOK_MARK: &str = "ship-dispatch:pre-push v1";
/// Marker embedded in the pre-commit hook so a re-install is a no-op.
pub const PRE_COMMIT_HOOK_MARK: &str = "ship-dispatch:pre-commit v1";

const PRE_PUSH_HOOK: &str = include_str!("pre_push_hook.sh");
const PRE_COMMIT_HOOK: &str = include_str!("pre_commit_hook.sh");

/// Install the pre-push guard that blocks pushes of `refs/heads/job/*`.
pub fn install_pre_push_hook(worktree_path: &Path) -> Result<PathBuf> {
    install_hook(worktree_path, "pre-push", PRE_PUSH_HOOK, PRE_PUSH_HOOK_MARK)
}

/// Install the pre-commit guard that refuses `.artifacts/` paths and any
/// `.gitignore` change that weakens the `.artifacts` ignore.
pub fn install_pre_commit_hook(path: &Path) -> Result<PathBuf> {
    install_hook(path, "pre-commit", PRE_COMMIT_HOOK, PRE_COMMIT_HOOK_MARK)
}

/// Install both pre-push and pre-commit guards. Idempotent — safe on every request.
pub fn install_repo_guards(path: &Path) -> Result<(PathBuf, PathBuf)> {
    let pre_push = install_pre_push_hook(path)?;
    let pre_commit = install_pre_commit_hook(path)?;
    Ok((pre_push, pre_commit))
}

fn install_hook(path: &Path, hook_name: &str, content: &str, marker: &str) -> Result<PathBuf> {
    let common_out = std::process::Command::new("git")
        .args([
            "-C",
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("path is not valid UTF-8"))?,
            "rev-parse",
            "--git-common-dir",
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("git rev-parse failed: {e}"))?;
    if !common_out.status.success() {
        return Err(anyhow::anyhow!(
            "git rev-parse --git-common-dir failed: {}",
            String::from_utf8_lossy(&common_out.stderr).trim()
        ));
    }
    let common = PathBuf::from(String::from_utf8_lossy(&common_out.stdout).trim());
    let hook_dir = common.join("hooks");
    std::fs::create_dir_all(&hook_dir)?;
    let hook_file = hook_dir.join(hook_name);
    let needs_install = match std::fs::read_to_string(&hook_file) {
        Ok(existing) => !existing.contains(marker),
        Err(_) => true,
    };
    if needs_install {
        std::fs::write(&hook_file, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hook_file)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hook_file, perms)?;
        }
    }
    Ok(hook_file)
}
