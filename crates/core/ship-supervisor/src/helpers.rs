//! Subprocess helpers for the supervisor — worktree, tmux, and agent-config utilities.

use anyhow::Result;
use std::path::PathBuf;

pub fn worktrees_dir() -> PathBuf {
    let project = repo_dir()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ship".to_string());
    ship_platform::config::worktrees_dir(&project)
}

pub(crate) fn repo_dir() -> PathBuf {
    if let Ok(dir) = ship_substrate::env::read_raw(ship_substrate::env::Var::RepoDir) {
        return PathBuf::from(dir);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub(crate) fn create_worktree(
    worktree_path: &std::path::Path,
    branch: &str,
    base_branch: Option<&str>,
) -> Result<()> {
    if worktree_path.exists() {
        return Ok(());
    }
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?;

    let status = std::process::Command::new("git")
        .args(["worktree", "add", path_str, branch])
        .current_dir(repo_dir())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;

    if !status.success() {
        // Branch may not exist yet — create it from base_branch.
        let base = base_branch.unwrap_or("HEAD");
        let args = vec!["worktree", "add", path_str, "-b", branch, base];
        let status2 = std::process::Command::new("git")
            .args(&args)
            .current_dir(repo_dir())
            .status()
            .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
        if !status2.success() {
            return Err(anyhow::anyhow!(
                "git worktree add failed for branch '{}' at '{}'",
                branch,
                worktree_path.display()
            ));
        }
    }
    Ok(())
}

pub(crate) fn compile_agent_config(
    worktree_path: &std::path::Path,
    agent_id: &str,
) -> Result<()> {
    // Invoke the `ship` CLI sitting next to `shipd`. Using current_exe()
    // directly spawns another daemon (shipd has no `use` subcommand and
    // ignores argv) — that produced a fork bomb in the supervisor.
    let self_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("current_exe failed: {e}"))?;
    let ship_bin = self_exe
        .parent()
        .map(|dir| dir.join("ship"))
        .unwrap_or_else(|| std::path::PathBuf::from("ship"));
    // `--scoped`: this is a daemon-spawned launch, so the dispatcher injects
    // $SHIP_TOKEN + $SHIP_DAEMON_URL into the agent's env. Compile the
    // daemon-hosted authed-HTTP `ship` MCP entry (scope-enforced) rather than
    // the trusted-local stdio shim.
    let output = std::process::Command::new(&ship_bin)
        .args(["use", agent_id, "--scoped"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run ship: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "exit {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }
    Ok(())
}


/// Kill a tmux session by name. No-op if session does not exist.
pub(crate) fn kill_tmux_session(session_name: &str) -> Result<()> {
    let status = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session_name])
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Ok(()), // session may not exist — that's fine
        Err(e) => Err(anyhow::anyhow!("failed to run tmux: {e}")),
    }
}

/// Remove a git worktree. No-op if path does not exist.
pub(crate) fn remove_worktree(worktree_path: &std::path::Path) -> Result<()> {
    if !worktree_path.exists() {
        return Ok(());
    }
    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?;
    let status = std::process::Command::new("git")
        .args(["worktree", "remove", path_str, "--force"])
        .current_dir(repo_dir())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "git worktree remove failed for '{}'",
            worktree_path.display()
        ));
    }
    Ok(())
}

/// Delete a local git branch. No-op if branch does not exist.
pub(crate) fn delete_branch(branch: &str) -> Result<()> {
    let status = std::process::Command::new("git")
        .args(["branch", "-D", branch])
        .current_dir(repo_dir())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
    if !status.success() {
        // Branch may not exist — not an error for cleanup.
        tracing::debug!(branch, "branch deletion returned non-zero (may not exist)");
    }
    Ok(())
}

