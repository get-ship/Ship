//! Replay and reconciliation for job dispatch.
//!
//! On daemon startup:
//! 1. Replays stale `job.completed` events that arrived while daemon was down
//! 2. Reconciles jobs stuck in `Dispatched` state after a daemon restart

use ship_substrate::events::EventEnvelope;
use crate::projections::job::{load_jobs, JobRecord, JobStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::pipeline::advance_pipeline;
use super::OrchestratorContext;

/// Max stale completions to replay on a single daemon startup. Older jobs
/// beyond this are dropped with a warn log — a signal that the projection
/// is polluted and needs operator attention, not that the daemon should
/// spend seconds replaying them on every restart.
pub(super) const MAX_REPLAYS_PER_STARTUP: usize = 10;

/// Returns true when a Completed job has work the replay would actually
/// do — currently: pipeline progression is still pending, OR an
/// `auto_merge` pipeline finished but never merged (worktree still on disk).
///
/// Returns false when the job is fully terminal: no DAG pipeline, or
/// every step completed with no pending auto_merge.
pub(super) fn needs_replay(job: &JobRecord) -> bool {
    let Some(pipeline) = job.steps.as_ref() else {
        return false;
    };

    let total_steps = pipeline.steps.len();
    let completed = job.completed_step_count();
    if completed < total_steps {
        return true;
    }

    if pipeline.auto_merge {
        let worktree = job.worktree.as_deref().map(std::path::Path::new);
        return matches!(worktree, Some(p) if p.exists());
    }

    false
}

/// Select the jobs to replay: Completed + needs_replay, sorted most-recent
/// first, capped at `MAX_REPLAYS_PER_STARTUP`. Logs a warn when capped.
pub(super) fn select_replay_targets(jobs: &HashMap<String, JobRecord>) -> Vec<JobRecord> {
    let mut stale: Vec<JobRecord> = jobs
        .values()
        .filter(|j| j.status == JobStatus::Completed)
        .filter(|j| needs_replay(j))
        .cloned()
        .collect();

    stale.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    if stale.len() > MAX_REPLAYS_PER_STARTUP {
        tracing::warn!(
            total = stale.len(),
            cap = MAX_REPLAYS_PER_STARTUP,
            "job-dispatch: replay set exceeds cap; replaying most recent only. \
             Investigate stale completions: ship jobs --status completed"
        );
        stale.truncate(MAX_REPLAYS_PER_STARTUP);
    }

    stale
}

/// Replay unprocessed `job.completed` events that arrived while the daemon was down.
pub async fn replay_unprocessed_completions(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
) {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: replay failed to load jobs: {e}");
            return;
        }
    };

    let stale = select_replay_targets(&jobs);

    if stale.is_empty() {
        tracing::debug!("job-dispatch: no stale completed jobs to replay");
        return;
    }

    tracing::info!(count = stale.len(), "job-dispatch: replaying stale job.completed events");

    for job in &stale {
        let payload = serde_json::json!({
            "job_id": job.job_id,
            "slug": job.slug,
        });
        let envelope = match EventEnvelope::new(
            ship_substrate::events::job::event_types::JOB_COMPLETED,
            &job.job_id,
            &payload,
        ) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(slug = job.slug, "job-dispatch: replay failed to build envelope: {e}");
                continue;
            }
        };

        tracing::info!(slug = job.slug, job_id = job.job_id, "job-dispatch: replaying job.completed");

        if !advance_pipeline(kernel, &envelope, octx).await {
            tracing::info!(slug = job.slug, "job-dispatch: replay — no pipeline to advance, awaiting gate/merge");
        }
    }

    tracing::info!("job-dispatch: replay complete");
}

/// Reconcile jobs stuck in `Dispatched` state after daemon restart.
pub async fn reconcile_dispatched_jobs(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
) {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: reconcile failed to load jobs: {e}");
            return;
        }
    };

    reconcile_dispatched_inner(kernel, octx, jobs, None).await;
}

/// Path-aware variant for tests.
pub async fn reconcile_dispatched_jobs_at(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
    ship_dir: &std::path::Path,
) {
    let jobs = match crate::projections::job::load_jobs_at(ship_dir) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: reconcile failed to load jobs: {e}");
            return;
        }
    };

    reconcile_dispatched_inner(kernel, octx, jobs, Some(ship_dir)).await;
}

async fn reconcile_dispatched_inner(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
    jobs: std::collections::HashMap<String, crate::projections::job::JobRecord>,
    _store_dir: Option<&std::path::Path>,
) {
    let stale: Vec<_> = jobs
        .values()
        .filter(|j| j.status == JobStatus::Dispatched)
        .collect();

    if stale.is_empty() {
        tracing::debug!("job-dispatch: no stale dispatched jobs to reconcile");
        return;
    }

    tracing::info!(count = stale.len(), "job-dispatch: reconciling stale dispatched jobs");

    for job in stale {
        // Ask the executor to rehydrate a real handle from out-of-band
        // state (tmux session names, window names, etc). `Some` means
        // the agent survived the restart — re-attach. `None` means no
        // live process was found on this pass — record an observation
        // event and let the liveness sweep decide after the stale
        // threshold. A single missed probe is not sufficient evidence
        // to terminate the job.
        match octx
            .executor
            .resume_handle(
                &job.slug,
                job.provider.as_deref(),
                job.current_step.as_deref(),
            )
            .await
        {
            Some(handle) => {
                tracing::info!(
                    slug = job.slug,
                    "job-dispatch: reconcile — process still alive, re-attaching"
                );
                octx.registry.register(&job.slug, handle).await;
                continue;
            }
            None => {
                tracing::info!(
                    slug = job.slug, job_id = job.job_id,
                    "job-dispatch: reconcile — no live process, deferring to liveness sweep"
                );
                crate::services::orchestrator::emit::emit_executor_session_destroyed(
                    kernel,
                    &job.slug,
                    "tmux_window",
                    &job.slug,
                    ship_substrate::events::executor::DestructionCause::External,
                )
                .await;
            }
        }
    }

    tracing::info!("job-dispatch: reconciliation complete");
}

// (Dead `emit_step_failed` removed — `super::emit::emit_step_failed` is the
// canonical helper. The local variant also double-emitted via both
// SqliteEventStore::append AND KernelRouter::emit, which would duplicate the
// row had it ever been called.)
