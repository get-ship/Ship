//! Subprocess helpers for the supervisor — worktree, tmux, and agent-config utilities.

use anyhow::Result;
use std::path::PathBuf;

pub(crate) fn worktrees_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SHIP_WORKTREE_DIR") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join("dev").join("ship-worktrees")
}

pub(crate) fn repo_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SHIP_REPO_DIR") {
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
    let output = std::process::Command::new(&ship_bin)
        .args(["use", agent_id])
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

pub(crate) fn ensure_tmux_session(
    session_name: &str,
    worktree_path: &std::path::Path,
) -> Result<()> {
    // Check if session/window already exists.
    let check = std::process::Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .status();

    if let Ok(s) = check
        && s.success()
    {
        return Ok(());
    }

    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?;

    // Try creating a window in the "ship" session first (works with tmux -CC / iTerm2).
    // Falls back to a new detached session if "ship" session doesn't exist.
    let parent_session = "ship";
    let has_parent = std::process::Command::new("tmux")
        .args(["has-session", "-t", parent_session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if has_parent {
        let status = std::process::Command::new("tmux")
            .args(["new-window", "-t", parent_session, "-n", session_name, "-c", path_str])
            .status()
            .map_err(|e| anyhow::anyhow!("failed to run tmux: {e}"))?;

        if status.success() {
            return Ok(());
        }
        // Fall through to new-session if new-window failed
    }

    let status = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", session_name, "-c", path_str])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run tmux: {e}"))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "tmux new-session failed for session '{session_name}'"
        ));
    }
    Ok(())
}

pub(crate) fn send_agent_command(session_name: &str, providers: &[String]) {
    let Some(cmd) = provider_cli(providers) else {
        tracing::info!(session = session_name, "no known provider CLI; skipping agent spawn");
        return;
    };

    // Try window in "ship" session first, fall back to standalone session
    let window_target = format!("ship:{session_name}");
    let target = if std::process::Command::new("tmux")
        .args(["has-session", "-t", "ship"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        &window_target
    } else {
        session_name
    };

    let result = std::process::Command::new("tmux")
        .args(["send-keys", "-t", target, &cmd, "Enter"])
        .status();

    if let Err(e) = result {
        tracing::warn!(session = session_name, target, "tmux send-keys failed: {e}");
    }
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

pub(crate) fn provider_cli(providers: &[String]) -> Option<String> {
    use runtime::services::dispatch_ports::LaunchProfile;
    use runtime::services::provider_adapter::adapter_for;

    for p in providers {
        match p.as_str() {
            "claude" | "claude-code" | "codex" => {
                let profile = LaunchProfile {
                    provider: Some(p.clone()),
                    persist_session: true,
                    ..Default::default()
                };
                // Providers in the match arm are known; Err here is unreachable.
                let adapter = adapter_for(p).ok()?;
                return Some(adapter.build_interactive("workspace", &profile).to_shell_string());
            }
            _ => {}
        }
    }
    None
}
