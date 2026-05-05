//! Platform trait implementations — connects the runtime orchestrator to
//! shipd's process management (tmux, git worktree, agent lifecycle).

use anyhow::Result;
use async_trait::async_trait;
use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::KernelRouter;
use ship_primitives::services::dispatch_ports::{
    AgentPlatform, ExitSignal, ExecutorHandle, JobExecutor, ProcessRegistryPort,
};
use ship_primitives::services::orchestrator::emit::emit_executor_session_destroyed;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers;
use super::registry::ProcessRegistry;

/// Shipd's concrete implementation of `AgentPlatform`.
///
/// Wraps the process-management helpers (tmux, git worktree, agent lifecycle)
/// behind the trait boundary so the runtime orchestrator can call back.
pub struct ShipdPlatform {
    pub registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn JobExecutor>,
    pub health_registry: crate::supervised::HealthRegistry,
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

    async fn graceful_agent_exit(&self, slug: &str, provider: Option<&str>) {
        super::agent_lifecycle::graceful_agent_exit(slug, provider).await;
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
        let merge_result = std::process::Command::new("git")
            .args(["merge", branch, "--no-edit"])
            .current_dir(helpers::repo_dir())
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

    fn start_exit_handler(
        &self,
        kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
        exit_rx: tokio::sync::mpsc::UnboundedReceiver<ExitSignal>,
    ) {
        super::liveness::start_exit_handler(kernel, self.registry.clone(), exit_rx);
    }

    fn start_liveness_sweep(
        &self,
        kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
    ) {
        super::liveness::start_liveness_sweep(kernel, self.registry.clone(), self.executor.clone(), &self.health_registry);
    }

    async fn cleanup_orphaned_worktrees(&self) {
        super::orphan_cleanup::cleanup_orphaned_worktrees(&self.kernel).await;
    }
}

/// No-op platform for tests that don't need real tmux/git operations.
#[cfg(test)]
pub struct MockPlatform;

#[cfg(test)]
#[async_trait]
impl AgentPlatform for MockPlatform {
    fn worktrees_dir(&self) -> PathBuf { PathBuf::from("/tmp/test-worktrees") }
    fn compile_agent_config(&self, _: &Path, _: &str) -> Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> Result<bool> { Ok(true) }
    fn start_exit_handler(&self, _: Arc<Mutex<ship_substrate::events::KernelRouter>>, _: tokio::sync::mpsc::UnboundedReceiver<ExitSignal>) {}
    fn start_liveness_sweep(&self, _: Arc<Mutex<ship_substrate::events::KernelRouter>>) {}
    async fn cleanup_orphaned_worktrees(&self) {}
}

/// Adapter that wraps `Arc<Mutex<ProcessRegistry>>` behind `ProcessRegistryPort`.
pub struct RegistryAdapter(pub Arc<Mutex<ProcessRegistry>>);

#[async_trait]
impl ProcessRegistryPort for RegistryAdapter {
    async fn register(&self, slug: &str, handle: ExecutorHandle) {
        self.0.lock().await.register(slug, handle);
    }

    async fn unregister(&self, slug: &str) {
        self.0.lock().await.unregister(slug);
    }

    async fn take(&self, slug: &str) -> Option<ExecutorHandle> {
        self.0.lock().await.take(slug)
    }

    async fn live_slugs(&self) -> Vec<String> {
        self.0.lock().await.live_slugs()
    }

    async fn get_pid(&self, slug: &str) -> Option<u32> {
        self.0.lock().await.get_pid(slug)
    }
}
