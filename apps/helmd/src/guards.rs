//! Git hook guards — pre-push (block `refs/heads/job/*` pushes) and
//! pre-commit (refuse `.artifacts/` paths and `.gitignore` weakening).
//!
//! Both hooks install into the repo's **shared git-common-dir** so every
//! worktree inherits them. They are static scripts embedded at compile
//! time; the only `ship`-side logic is idempotency (re-running on a hook
//! that already carries the marker is a no-op) and the executable bit.
//!
//! This module has no feature gate — the guards are helmd's contract with
//! the operator regardless of which feature flags are on. `dispatch.rs`
//! installs the guards on every dispatch; `main.rs` installs them on every
//! `GitService/Overview` call so the main checkout is guarded on first
//! SPA touch, even before any dispatch has happened.

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Marker embedded in the pre-push hook so a re-install is a no-op.
pub const PRE_PUSH_HOOK_MARK: &str = "ship-dispatch:pre-push v1";
/// Marker embedded in the pre-commit hook so a re-install is a no-op.
pub const PRE_COMMIT_HOOK_MARK: &str = "ship-dispatch:pre-commit v1";

const PRE_PUSH_HOOK: &str = include_str!("pre_push_hook.sh");
const PRE_COMMIT_HOOK: &str = include_str!("pre_commit_hook.sh");

/// Install the pre-push guard that blocks pushes of `refs/heads/job/*`
/// from any worktree in this repo. Returns the path to the installed hook.
pub fn install_pre_push_hook(worktree_path: &Path) -> Result<PathBuf> {
    install_hook(
        worktree_path,
        "pre-push",
        PRE_PUSH_HOOK,
        PRE_PUSH_HOOK_MARK,
    )
}

/// Install the pre-commit guard that refuses `.artifacts/` paths and any
/// `.gitignore` change that weakens the `.artifacts` ignore. Returns the
/// path to the installed hook.
pub fn install_pre_commit_hook(path: &Path) -> Result<PathBuf> {
    install_hook(
        path,
        "pre-commit",
        PRE_COMMIT_HOOK,
        PRE_COMMIT_HOOK_MARK,
    )
}

/// Install BOTH the pre-push and pre-commit guards. Idempotent — safe to
/// call on every request. Used by `dispatch.rs` on each dispatch, by
/// `main.rs` on every `GitService/Overview` call, and by the
/// `--install-hooks <path>` CLI flag.
pub fn install_repo_guards(path: &Path) -> Result<(PathBuf, PathBuf)> {
    let pre_push = install_pre_push_hook(path)?;
    let pre_commit = install_pre_commit_hook(path)?;
    Ok((pre_push, pre_commit))
}

/// Resolve the shared git-common-dir for the repo containing `path`, then
/// ensure `hooks/<hook_name>` exists with the supplied `content` and the
/// executable bit set. Idempotent: a hook that already carries `marker`
/// is left alone (a future re-install never clobbers a hand-edited hook).
fn install_hook(
    path: &Path,
    hook_name: &str,
    content: &str,
    marker: &str,
) -> Result<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::{install_pre_commit_hook, install_pre_push_hook, install_repo_guards};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Build a one-commit repo with a worktree at `<tmp>/<wt>` on a
    /// `job/<slug>` branch. Returns the worktree path.
    fn setup_repo_with_worktree(tmp: &Path, wt_subdir: &str, slug: &str) -> PathBuf {
        let main = tmp;
        let run = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(main)
                .output()
                .expect("git failed");
            assert!(
                out.status.success(),
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        run(&["init", "--initial-branch=main"]);
        run(&["config", "user.email", "t@example.com"]);
        run(&["config", "user.name", "test"]);
        std::fs::write(main.join("README.md"), "x").unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", "init"]);
        let wt = main.join(wt_subdir);
        run(&[
            "worktree",
            "add",
            "-b",
            &format!("job/{slug}"),
            wt.to_str().unwrap(),
        ]);
        wt
    }

    /// After `install_pre_push_hook` runs, the hook file exists at the
    /// shared git-common-dir/hooks/pre-push, is executable, and contains
    /// the guard that blocks pushes of `refs/heads/job/*`.
    #[test]
    fn install_pre_push_hook_writes_executable_guard() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wt = setup_repo_with_worktree(tmp.path(), "wt", "auth");
        let hook_path = install_pre_push_hook(&wt).expect("install");

        assert!(hook_path.exists(), "hook file should exist at {hook_path:?}");
        let content = std::fs::read_to_string(&hook_path).expect("read hook");
        assert!(
            content.contains("ship-dispatch:pre-push v1"),
            "marker missing in hook content"
        );
        assert!(
            content.contains("refs/heads/job/*"),
            "ref guard missing in hook content"
        );
        assert!(
            content.contains("SHIP_ALLOW_PUSH"),
            "bypass env guard missing"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&hook_path)
                .expect("metadata")
                .permissions()
                .mode();
            assert!(
                mode & 0o111 != 0,
                "hook should be executable, got mode {mode:o}"
            );
        }
    }

    /// Re-running the installer on an already-marked hook is a no-op: the
    /// content is preserved verbatim (no clobbering of unrelated edits).
    #[test]
    fn install_pre_push_hook_is_idempotent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wt = setup_repo_with_worktree(tmp.path(), "wt", "idem");
        install_pre_push_hook(&wt).expect("first install");
        let hook_path = install_pre_push_hook(&wt).expect("second install");

        let content = std::fs::read_to_string(&hook_path).expect("read hook");
        assert!(content.contains("ship-dispatch:pre-push v1"));
        let len = content.len();
        // third call should be a no-op
        install_pre_push_hook(&wt).expect("third install");
        let content2 = std::fs::read_to_string(&hook_path).expect("read hook");
        assert_eq!(
            content2.len(),
            len,
            "hook should not grow on re-install (would clobber user edits)"
        );
    }

    /// Companion: pre-commit hook installs in the same shared hooks dir,
    /// executable, with the `.artifacts/`-refusal marker.
    #[test]
    fn install_pre_commit_hook_writes_executable_guard() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wt = setup_repo_with_worktree(tmp.path(), "wt", "guard");
        let hook_path = install_pre_commit_hook(&wt).expect("install");

        assert!(hook_path.exists(), "hook file should exist at {hook_path:?}");
        let content = std::fs::read_to_string(&hook_path).expect("read hook");
        assert!(
            content.contains("ship-dispatch:pre-commit v1"),
            "marker missing in hook content"
        );
        assert!(
            content.contains(r"^\.artifacts/"),
            ".artifacts path guard missing in hook content"
        );
        assert!(
            content.contains("un-ignores") || content.contains(".gitignore"),
            ".gitignore-weakening guard missing in hook content"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&hook_path)
                .expect("metadata")
                .permissions()
                .mode();
            assert!(
                mode & 0o111 != 0,
                "hook should be executable, got mode {mode:o}"
            );
        }
    }

    /// `install_repo_guards` writes BOTH the pre-push and pre-commit hooks
    /// in one call. Both hooks must be installed and executable.
    #[test]
    fn install_repo_guards_writes_both_hooks() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wt = setup_repo_with_worktree(tmp.path(), "wt", "both");
        let (pre_push, pre_commit) = install_repo_guards(&wt).expect("install");

        assert!(pre_push.exists(), "pre-push should exist at {pre_push:?}");
        assert!(
            pre_commit.exists(),
            "pre-commit should exist at {pre_commit:?}"
        );
        assert!(
            pre_push
                .parent()
                .and_then(|p| pre_commit.parent().map(|q| p == q))
                .unwrap_or(false),
            "both hooks should live in the same shared hooks dir"
        );
        let push = std::fs::read_to_string(&pre_push).expect("read pre-push");
        let commit = std::fs::read_to_string(&pre_commit).expect("read pre-commit");
        assert!(push.contains("ship-dispatch:pre-push v1"));
        assert!(commit.contains("ship-dispatch:pre-commit v1"));
    }

    /// Behavioral end-to-end: with the pre-commit hook installed, a real
    /// `git commit` of an `.artifacts/` path is refused. This is the fence
    /// the hook exists to enforce — the marker test above proves the file
    /// contains the marker, this test proves the file is wired into git
    /// and runs on commit.
    #[test]
    fn installed_pre_commit_hook_refuses_artifacts_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wt = setup_repo_with_worktree(tmp.path(), "wt", "behavior");
        install_pre_commit_hook(&wt).expect("install");

        let art_dir = wt.join(".artifacts");
        std::fs::create_dir_all(&art_dir).unwrap();
        std::fs::write(art_dir.join("oops.txt"), "scratch\n").unwrap();
        let add = Command::new("git")
            .args(["add", ".artifacts/oops.txt"])
            .current_dir(&wt)
            .output()
            .expect("git add failed");
        assert!(
            add.status.success(),
            "git add: {}",
            String::from_utf8_lossy(&add.stderr)
        );
        let commit = Command::new("git")
            .args(["commit", "-m", "should be refused"])
            .current_dir(&wt)
            .output()
            .expect("git commit failed");
        assert!(
            !commit.status.success(),
            "pre-commit hook should have refused the .artifacts/ commit, but it succeeded"
        );
        let stderr = String::from_utf8_lossy(&commit.stderr);
        assert!(
            stderr.contains(".artifacts") || stderr.contains("pre-commit"),
            "expected hook message in stderr, got: {stderr}"
        );
    }
}
