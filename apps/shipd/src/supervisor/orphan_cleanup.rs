//! Startup orphan cleanup — removes worktrees for merged and failed jobs.
//!
//! After a daemon restart, jobs that previously reached Merged or Failed status
//! may still have worktrees, branches, and tmux sessions on disk. This module
//! scans the job projection on startup and cleans them up.

use runtime::events::EventEnvelope;
use runtime::projections::job::{load_jobs, JobStatus};

use super::dispatch_handlers::handle_job_cleanup;
use super::helpers::worktrees_dir;

/// Scan for orphaned worktrees from previous daemon runs.
///
/// Loads the job projection and finds jobs in a terminal state (Merged or Failed)
/// that still have a worktree directory on disk. Cleans up each one using the
/// standard cleanup handler so cleanup is consistent and idempotent.
pub(crate) async fn cleanup_orphaned_worktrees() {
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
        let payload = serde_json::json!({
            "job_id": job.job_id,
            "slug": job.slug,
        });
        let envelope = match EventEnvelope::new(
            runtime::events::job::event_types::JOB_MERGED,
            &job.job_id,
            &payload,
        ) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(
                    slug = job.slug,
                    "job-dispatch: orphan — failed to build envelope: {e}"
                );
                continue;
            }
        };

        tracing::info!(
            slug = job.slug,
            job_id = job.job_id,
            "job-dispatch: removing orphaned worktree"
        );
        handle_job_cleanup(&envelope).await;
    }

    tracing::info!("job-dispatch: orphan cleanup complete");
}
