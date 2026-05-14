//! Concrete adapters for job isolation.
//!
//! `GitWorktreeIsolation` — creates git worktrees, writes phase specs, compiles agent config.
//! Executor implementations live in the `executors` submodule.

use anyhow::Result;
use async_trait::async_trait;
use ship_platform::services::dispatch_ports::{IsolationStrategy, JobContext};
use ship_platform::services::orchestrator::isolation::write_phase_spec_from_context;
use std::path::PathBuf;

use super::helpers::{compile_agent_config, create_worktree, worktrees_dir};

/// Isolation strategy that creates git worktrees and compiles agent configs.
pub(crate) struct GitWorktreeIsolation;

#[async_trait]
impl IsolationStrategy for GitWorktreeIsolation {
    async fn prepare(&self, job: &JobContext) -> Result<PathBuf> {
        let worktree_path = worktrees_dir().join(&job.slug);
        create_worktree(&worktree_path, &job.branch, None)?;
        let session_dir = worktree_path.join(".artifacts");
        std::fs::create_dir_all(&session_dir)?;
        // Dispatch marker — MCP's tool gate uses this to detect it's
        // serving a dispatched agent when SHIP_ACTOR_ID env gets stripped
        // by the provider's MCP subprocess spawn. Worktree-scoped; the
        // project root never has this marker.
        let _ = std::fs::write(session_dir.join(".dispatched"), &job.slug);
        write_phase_spec_from_context(&worktree_path, job)?;
        compile_agent_config(&worktree_path, &job.agent)?;
        Ok(worktree_path)
    }

    async fn cleanup(&self, job: &JobContext) -> Result<()> {
        let worktree_path = worktrees_dir().join(&job.slug);
        if worktree_path.exists() {
            super::helpers::remove_worktree(&worktree_path)?;
        }
        super::helpers::delete_branch(&job.branch)?;
        Ok(())
    }
}

