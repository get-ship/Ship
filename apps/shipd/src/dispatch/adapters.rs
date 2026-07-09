//! Concrete adapters for job isolation.
//!
//! `GitWorktreeIsolation` — creates git worktrees, writes phase specs, compiles agent config.
//! The git/tmux/worktree subprocess helpers it calls live in
//! `ship_agents::runtime::helpers`; the executors in `ship_agents::runtime::executors`.
//!
//! `prepare` delegates the worktree-create step to `ship_git::write::create_worktree`
//! (the libgit2-direct leaf also backing the `ship.internal.v1.GitService/CreateWorktree`
//! RPC). ship-code's older `create_worktree` helper was removed — its reimplementation
//! of `git worktree add` shell logic is subsumed by the leaf.

use anyhow::Result;
use async_trait::async_trait;
use ship_code::dispatch_ports::{IsolationStrategy, SessionContext};
use ship_git::write::{self, WorktreeOutcome};
use ship_pipeline_runtime::orchestrator::isolation::write_phase_spec_from_context;
use std::path::PathBuf;

use ship_agents::runtime::helpers::{compile_agent_config, repo_dir, worktrees_dir};

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
            // Dispatch retries should be idempotent: a worktree at the same
            // path on the same branch re-uses cleanly (Reused); a path occupied
            // by a different branch refuses loudly (Failed → bubble up). The
            // leaf's `WorktreeOutcome` matches the `Created / Reused / Failed`
            // Per-call fork source: the dispatch's `repo_path` is the
            // absolute path the worktree forks from; `base_ref` is the
            // ref the new branch starts from. `None` / empty falls back
            // to the daemon default + that repo's HEAD, matching the
            // proto's empty-string = default convention. Consumed
            // in-call; never persisted.
            let repo_root = job.repo_path.clone().unwrap_or_else(repo_dir);
            let base_ref = job.base_ref.as_deref().unwrap_or("");
            let outcome = write::create_worktree(
                &repo_root,
                &worktree_path,
                &job.branch,
                base_ref,
                true,
            );
            if let WorktreeOutcome::Failed { error, .. } = outcome {
                return Err(anyhow::anyhow!(
                    "git worktree add failed for '{}': {error}",
                    worktree_path.display()
                ));
            }
        }
        let session_dir = worktree_path.join(".artifacts");
        std::fs::create_dir_all(&session_dir)?;
        // Dispatch marker — MCP's tool gate uses this to detect it's
        // serving a dispatched agent when SHIP_ACTOR_ID env gets stripped
        // by the harness's MCP subprocess spawn. Worktree-scoped; the
        // project root never has this marker.
        std::fs::write(session_dir.join(".dispatched"), &job.slug).map_err(|e| {
            // hardening-r2 §5.4: the .dispatched marker is what the MCP tool
            // gate reads to detect a dispatched context (vs interactive).
            // A silent miss leaves the agent gated as interactive with no
            // record why — propagate so the dispatch aborts loudly.
            anyhow::anyhow!(
                "write .dispatched marker (slug={}) failed: {}",
                job.slug,
                e
            )
        })?;
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
            ship_agents::runtime::helpers::remove_worktree(&worktree_path)?;
        }
        ship_agents::runtime::helpers::delete_branch(&job.branch)?;
        Ok(())
    }
}

