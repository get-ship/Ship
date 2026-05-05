//! Startup orphan cleanup — removes worktrees for merged and failed jobs.
//!
//! After a daemon restart, jobs that previously reached Merged or Failed status
//! may still have worktrees, branches, and tmux sessions on disk. This module
//! scans the job projection on startup and cleans them up.

use std::sync::Arc;

use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::KernelRouter;
use ship_platform::projections::job::{load_jobs, JobStatus};
use ship_platform::services::orchestrator::emit::emit_executor_session_destroyed;
use tokio::sync::Mutex;

use super::helpers::worktrees_dir;

/// Scan for orphaned worktrees from previous daemon runs.
///
/// Loads the job projection and finds jobs in a terminal state (Merged or Failed)
/// that still have a worktree directory on disk. Cleans up each one using the
/// standard cleanup handler so cleanup is consistent and idempotent.
pub(crate) async fn cleanup_orphaned_worktrees(kernel: &Arc<Mutex<KernelRouter>>) {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: orphan scan failed to load jobs: {e}");
            return;
        }
    };

    let orphans: Vec<_> = jobs
        .values()
        .filter(|j| matches!(j.status, JobStatus::Merged | JobStatus::Failed))
        .filter(|j| {
            let path = j
                .worktree
                .as_deref()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| worktrees_dir().join(&j.slug));
            path.exists()
        })
        .collect();

    if orphans.is_empty() {
        tracing::debug!("job-dispatch: no orphaned worktrees found");
        return;
    }

    tracing::info!(
        count = orphans.len(),
        "job-dispatch: cleaning up orphaned worktrees"
    );

    for job in orphans {
        tracing::info!(
            slug = job.slug,
            job_id = job.job_id,
            "job-dispatch: removing orphaned worktree"
        );
        cleanup_orphan(&job.slug, kernel).await;
    }

    tracing::info!("job-dispatch: orphan cleanup complete");
}

/// Clean up a single orphaned job's resources.
async fn cleanup_orphan(slug: &str, kernel: &Arc<Mutex<KernelRouter>>) {
    use super::helpers::{delete_branch, kill_tmux_session, remove_worktree};

    let tmux_session = format!("job-{slug}");
    let worktree_path = worktrees_dir().join(slug);
    let branch = format!("job/{slug}");

    if let Err(e) = kill_tmux_session(&tmux_session) {
        tracing::warn!(slug, "orphan: tmux cleanup failed: {e}");
    }
    // Destruction intent fires regardless of whether the session was alive;
    // the operator-facing fact is that we've torn it down.
    emit_executor_session_destroyed(
        kernel,
        slug,
        "tmux",
        &tmux_session,
        DestructionCause::External,
    )
    .await;
    if let Err(e) = remove_worktree(&worktree_path) {
        tracing::warn!(slug, "orphan: worktree cleanup failed: {e}");
    }
    if let Err(e) = delete_branch(&branch) {
        tracing::warn!(slug, "orphan: branch cleanup failed: {e}");
    }
}

#[cfg(test)]
pub(crate) mod tests_only {
    use super::*;

    pub(crate) async fn cleanup_orphan_for_test(slug: &str, kernel: &Arc<Mutex<KernelRouter>>) {
        super::cleanup_orphan(slug, kernel).await;
    }
}
