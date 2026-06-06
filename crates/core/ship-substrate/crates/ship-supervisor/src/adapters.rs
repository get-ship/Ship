//! Concrete adapters for job isolation.
//!
//! `GitWorktreeIsolation` — creates git worktrees, writes phase specs, compiles agent config.
//! Executor implementations live in the `executors` submodule.

use anyhow::Result;
use async_trait::async_trait;
use ship_substrate::services::dispatch_ports::{IsolationStrategy, SessionContext};
use ship_pipeline_runtime::orchestrator::isolation::write_phase_spec_from_context;
use std::path::PathBuf;

use super::helpers::{compile_agent_config, create_worktree, worktrees_dir};

/// Serializes `git worktree add` across concurrent dispatches. git takes a
/// lock on the main repo's index/worktrees during `worktree add`, so two
/// concurrent adds race (`index.lock` contention). Now that the orchestrator
/// dispatches off its mailbox (concurrently), this guard keeps the git step
/// one-at-a-time without blocking the mailbox or unrelated work.
static WORKTREE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Isolation strategy that creates git worktrees and compiles agent configs.
pub(crate) struct GitWorktreeIsolation;

#[async_trait]
impl IsolationStrategy for GitWorktreeIsolation {
    async fn prepare(&self, job: &SessionContext) -> Result<PathBuf> {
        let worktree_path = worktrees_dir().join(&job.slug);
        {
            let _guard = WORKTREE_LOCK.lock().await;
            create_worktree(&worktree_path, &job.branch, None)?;
        }
        let session_dir = worktree_path.join(".artifacts");
        std::fs::create_dir_all(&session_dir)?;
        // Dispatch marker — MCP's tool gate uses this to detect it's
        // serving a dispatched agent when SHIP_ACTOR_ID env gets stripped
        // by the harness's MCP subprocess spawn. Worktree-scoped; the
        // project root never has this marker.
        let _ = std::fs::write(session_dir.join(".dispatched"), &job.slug);
        write_phase_spec_from_context(&worktree_path, job)?;
        // Run steps (pure shell) carry no agent — there's nothing to
        // compile, and an empty/placeholder agent would fail profile
        // resolution. Only compile when a real agent is named (every
        // agent step and bare dispatch sets one).
        if !job.agent.is_empty() {
            compile_agent_config(&worktree_path, &job.agent)?;
        }
        Ok(worktree_path)
    }

    async fn cleanup(&self, job: &SessionContext) -> Result<()> {
        let worktree_path = worktrees_dir().join(&job.slug);
        if worktree_path.exists() {
            super::helpers::remove_worktree(&worktree_path)?;
        }
        super::helpers::delete_branch(&job.branch)?;
        Ok(())
    }
}

