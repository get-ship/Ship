//! Subprocess helpers for the supervisor — worktree, tmux, and agent-config utilities.

use anyhow::Result;
use std::path::PathBuf;

pub fn worktrees_dir() -> PathBuf {
    let project = repo_dir()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ship".to_string());
    ship_config::worktrees::worktrees_dir(&project)
}

pub fn repo_dir() -> PathBuf {
    if let Ok(dir) = ship_substrate::env::read_raw(ship_substrate::env::Var::RepoDir) {
        return PathBuf::from(dir);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn create_worktree(
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

pub fn compile_agent_config(
    worktree_path: &std::path::Path,
    agent_id: &str,
) -> Result<()> {
    // Roll up the worktree's full composed world — cache tier + the
    // operator-enabled library tier + the space's own source, precedence-
    // law composed — the same `ship_resolver::roll_up` shipd's boot index
    // ingests, so a dispatch-compiled agent sees what the index sees
    // instead of a single-source `.ship/` read. `roll_up` is async
    // (V8 TS eval); `roll_up_blocking` bridges the sync trait boundary.
    let composed = ship_resolver::roll_up_blocking(worktree_path, None, None)
        .map_err(|e| anyhow::anyhow!("roll-up failed for agent '{agent_id}': {e}"))?;

    // In-process compile via the agent-def engine — the same pipeline
    // `compile_project` (the CLI's `ship use` wrapper) runs, minus the
    // single-source load + `#122` flat merge the roll-up already
    // superseded. `scoped_launch`: this is a daemon-spawned launch, so the
    // dispatcher injects $SHIP_TOKEN + $SHIP_DAEMON_URL into the agent's
    // env; the compile emits the daemon-hosted authed-HTTP `ship` MCP entry
    // (scope-enforced) rather than the trusted-local stdio shim.
    let summary = agent_def::compile::compile_project_from_library(
        &agent_def::compile::CompileOptions {
            space_root: worktree_path.to_path_buf(),
            output_root: None,
            harness: None,
            active_agent: Some(agent_id.to_string()),
            extra_skills: vec![],
            plugins_installed: vec![],
            scoped_launch: true,
            dry_run: false,
        },
        composed.library,
    )
    .map_err(|e| anyhow::anyhow!("compile failed for agent '{agent_id}': {e}"))?;
    if summary.harnesses.is_empty() {
        return Err(anyhow::anyhow!(
            "compile for agent '{agent_id}' produced no harness outputs — \
             no harnesses configured in the worktree's .ship/"
        ));
    }
    Ok(())
}


/// Kill a tmux session by name. No-op if session does not exist.
pub fn kill_tmux_session(session_name: &str) -> Result<()> {
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
pub fn remove_worktree(worktree_path: &std::path::Path) -> Result<()> {
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
pub fn delete_branch(branch: &str) -> Result<()> {
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


#[cfg(test)]
mod tests {
    use super::*;

    /// The dispatch-compile behavior: an in-process compile of a worktree
    /// with a `.ship/` agent produces the harness-native outputs the old
    /// `ship use <id> --scoped` shell-out produced (CLAUDE.md at the root).
    #[test]
    fn compile_agent_config_writes_harness_outputs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let write = |rel: &str, content: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        };
        write(".ship/rules/style.md", "Use explicit types.\n");
        write(
            ".ship/agents/demo.jsonc",
            r#"{ "agent": { "id": "demo", "name": "Demo", "harnesses": ["claude"] } }"#,
        );

        compile_agent_config(root, "demo").unwrap();
        assert!(
            root.join("CLAUDE.md").exists(),
            "scoped compile must write the harness-native output"
        );
    }

}
