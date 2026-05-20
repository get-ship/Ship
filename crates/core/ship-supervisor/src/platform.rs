//! Platform trait implementations — connects the runtime orchestrator to
//! shipd's process management (tmux, git worktree, agent lifecycle).

use anyhow::Result;
use async_trait::async_trait;
use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::KernelRouter;
use ship_platform::services::dispatch_ports::{AgentPlatform, SessionExecutor};
use ship_platform::services::orchestrator::emit::emit_executor_session_destroyed;
use ship_platform::services::process_registry::ProcessRegistry;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers;

/// Shipd's concrete implementation of `AgentPlatform`.
///
/// Wraps the process-management helpers (tmux, git worktree, agent lifecycle)
/// behind the trait boundary so the runtime orchestrator can call back.
pub struct ShipdPlatform {
    pub registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn SessionExecutor>,
    pub kernel: Arc<Mutex<KernelRouter>>,
}

#[async_trait]
impl AgentPlatform for ShipdPlatform {
    fn worktrees_dir(&self) -> PathBuf {
        helpers::worktrees_dir()
    }

    fn compile_agent_config(&self, worktree_path: &Path, agent_id: &str) -> Result<()> {
        helpers::compile_agent_config(worktree_path, agent_id)
    }

    async fn graceful_agent_exit(&self, slug: &str, harness: Option<&str>) {
        super::agent_lifecycle::graceful_agent_exit(slug, harness).await;
    }

    async fn cleanup_job(&self, slug: &str, branch: &str) {
        let tmux_session = format!("job-{slug}");
        let worktree_path = helpers::worktrees_dir().join(slug);

        if let Err(e) = helpers::kill_tmux_session(&tmux_session) {
            tracing::warn!(slug, "platform: tmux cleanup failed: {e}");
        }
        // Destruction intent fired whether the kill succeeded or the session
        // was already gone — operators still care that cleanup touched it.
        emit_executor_session_destroyed(
            &self.kernel,
            slug,
            "tmux",
            &tmux_session,
            DestructionCause::External,
        )
        .await;
        if let Err(e) = helpers::remove_worktree(&worktree_path) {
            tracing::warn!(slug, "platform: worktree cleanup failed: {e}");
        }
        if let Err(e) = helpers::delete_branch(branch) {
            tracing::warn!(slug, "platform: branch cleanup failed: {e}");
        }
    }

    fn git_merge(&self, branch: &str) -> Result<bool> {
        let merge_result = ship_substrate::spawn_bounded::Cmd::new("git")
            .args(["merge", branch, "--no-edit"])
            .current_dir(helpers::repo_dir())
            .timeout_secs(60)
            .output()
            .map_err(|e| anyhow::anyhow!("git merge command failed: {e}"))?;

        if merge_result.status.success() {
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&merge_result.stderr);
            tracing::error!(branch, "platform: merge failed: {stderr}");
            Ok(false)
        }
    }

    async fn cleanup_orphaned_worktrees(&self) {
        super::orphan_cleanup::cleanup_orphaned_worktrees(&self.kernel).await;
    }
}

/// No-op platform for tests that don't need real tmux/git operations.
/// Lives in the public surface (not `#[cfg(test)]`) so test code in
/// downstream crates can build it; the runtime cost is one zero-sized
/// struct definition.
pub struct MockPlatform;

#[async_trait]
impl AgentPlatform for MockPlatform {
    fn worktrees_dir(&self) -> PathBuf { PathBuf::from("/tmp/test-worktrees") }
    fn compile_agent_config(&self, _: &Path, _: &str) -> Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> Result<bool> { Ok(true) }
    async fn cleanup_orphaned_worktrees(&self) {}
}
