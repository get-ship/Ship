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

    if let Some(content) = &job.spec_content {
        std::fs::write(&dst_spec, content)
            .map_err(|e| anyhow::anyhow!("write inline spec to {}: {e}", dst_spec.display()))?;
        return Ok(());
    }

    // Legacy: read from the project root on disk.
    let src = std::path::Path::new(&job.spec_path);
    let content = if src.is_absolute() && src.exists() {
        std::fs::read_to_string(src)
            .map_err(|e| anyhow::anyhow!("read spec {}: {e}", src.display()))?
    } else {
        let root = worktree_path
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| anyhow::anyhow!("cannot locate project root from {}", worktree_path.display()))?;
        let alt = root.join(&job.spec_path);
        if !alt.exists() {
            tracing::warn!(
                slug = job.slug,
                spec_path = job.spec_path,
                "no inline spec_content and no spec on disk — worktree will have empty spec"
            );
            return Ok(());
        }
        std::fs::read_to_string(&alt)
            .map_err(|e| anyhow::anyhow!("read spec {}: {e}", alt.display()))?
    };

    std::fs::write(&dst_spec, &content)
        .map_err(|e| anyhow::anyhow!("write spec to {}: {e}", dst_spec.display()))?;
    Ok(())
}
