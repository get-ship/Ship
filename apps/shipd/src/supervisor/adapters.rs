//! Concrete adapters for job isolation.
//!
//! `GitWorktreeIsolation` — creates git worktrees, writes phase specs, compiles agent config.
//! Executor implementations live in the `executors` submodule.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{IsolationStrategy, JobContext};
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
        write_phase_spec_from_context(&worktree_path, job);
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

fn write_phase_spec_from_context(worktree_path: &std::path::Path, job: &JobContext) {
    let dst_spec = worktree_path.join(".artifacts").join("job-spec.md");
    let src = std::path::Path::new(&job.spec_path);
    let content = if src.exists() {
        std::fs::read_to_string(src).unwrap_or_default()
    } else {
        let parent = worktree_path.parent().and_then(|p| p.parent());
        if let Some(root) = parent {
            let alt = root.join(&job.spec_path);
            std::fs::read_to_string(&alt).unwrap_or_default()
        } else {
            String::new()
        }
    };
    let _ = std::fs::write(&dst_spec, &content);
}
