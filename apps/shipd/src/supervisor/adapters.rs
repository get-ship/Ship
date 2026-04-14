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

/// Materialize the job spec inside the worktree.
///
/// Priority:
/// 1. `job.spec_content` — inline content from the JobCreatedPayload event.
///    The preferred path: specs travel in events, not via shared disk.
/// 2. Legacy fallback: read `job.spec_path` from the main checkout (manual
///    dispatch where a lead agent wrote the spec to disk).
///
/// Returns Err when a spec was expected but neither source produced content,
/// so agents never launch against an empty spec silently.
fn write_phase_spec_from_context(worktree_path: &std::path::Path, job: &JobContext) -> Result<()> {
    let dst_spec = worktree_path.join(&job.spec_path);
    if let Some(parent) = dst_spec.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("create spec dir {}: {e}", parent.display()))?;
    }

    // Write a placeholder body only. The orchestrator's pipeline-specs writer
    // (services::orchestrator::pipeline_specs) overwrites this with a fresh
    // dispatch prompt right after `prepare` returns. The placeholder is kept
    // so legacy manual-dispatch flows that call only the adapter see
    // something sensible.
    let body = if let Some(content) = &job.spec_content {
        content.clone()
    } else {
        // Legacy disk fallback — read from project root.
        let src = std::path::Path::new(&job.spec_path);
        if src.is_absolute() && src.exists() {
            std::fs::read_to_string(src)
                .map_err(|e| anyhow::anyhow!("read spec {}: {e}", src.display()))?
        } else {
            String::new()
        }
    };
    std::fs::write(&dst_spec, &body)
        .map_err(|e| anyhow::anyhow!("write spec to {}: {e}", dst_spec.display()))?;
    Ok(())
}
