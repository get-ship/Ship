//! Replay and reconciliation for pipeline-run dispatch.
//!
//! On daemon startup:
//! 1. Re-drives Completed runs with pending work by synthesizing fresh
//!    `pipeline.run.completed` envelopes (stale completions that arrived
//!    while the daemon was down)
//! 2. Reconciles runs stuck in `Dispatched` state after a daemon restart

use ship_substrate::events::EventEnvelope;
use crate::projections::pipeline_run::{load_pipeline_runs, PipelineRun, PipelineRunStatus};
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
pub(super) fn needs_replay(job: &PipelineRun) -> bool {
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
pub(super) fn select_replay_targets(jobs: &HashMap<String, PipelineRun>) -> Vec<PipelineRun> {
    let mut stale: Vec<PipelineRun> = jobs
        .values()
        .filter(|j| j.status == PipelineRunStatus::Completed)
        .filter(|j| needs_replay(j))
        .cloned()
        .collect();

    stale.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    if stale.len() > MAX_REPLAYS_PER_STARTUP {
        tracing::warn!(
            total = stale.len(),
            cap = MAX_REPLAYS_PER_STARTUP,
            "job-dispatch: replay set exceeds cap; replaying most recent only. \
             Investigate stale completions: ship procs --kind pipeline-step"
        );
        stale.truncate(MAX_REPLAYS_PER_STARTUP);
    }

    stale
}

/// Re-drive Completed runs with pending work by synthesizing fresh `pipeline.run.completed` envelopes.
pub async fn replay_unprocessed_completions(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
) {
    let jobs = match load_pipeline_runs() {
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

    tracing::info!(count = stale.len(), "job-dispatch: replaying stale pipeline.run.completed events");

    for job in &stale {
        let payload = serde_json::json!({
            "pipeline_run_id": job.pipeline_run_id,
            "slug": job.slug,
        });
        let envelope = match EventEnvelope::new(
            ship_substrate::events::pipeline::event_types::PIPELINE_RUN_COMPLETED,
            &job.pipeline_run_id,
            &payload,
        ) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(slug = job.slug, "job-dispatch: replay failed to build envelope: {e}");
                continue;
            }
        };

        tracing::info!(slug = job.slug, pipeline_run_id = job.pipeline_run_id, "job-dispatch: replaying job.completed");

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
    let jobs = match load_pipeline_runs() {
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
    let jobs = match crate::projections::pipeline_run::load_pipeline_runs_at(ship_dir) {
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
    jobs: std::collections::HashMap<String, crate::projections::pipeline_run::PipelineRun>,
    _store_dir: Option<&std::path::Path>,
) {
    let stale: Vec<_> = jobs
        .values()
        .filter(|j| j.status == PipelineRunStatus::Dispatched)
        .collect();

    if stale.is_empty() {
        tracing::debug!("job-dispatch: no stale dispatched jobs to reconcile");
        return;
    }

    tracing::info!(count = stale.len(), "job-dispatch: reconciling stale dispatched jobs");

    for job in stale {
        // C3: journal-first telemetry — check whether a C2+ shadow journal
        // exists for this run. Presence of a complete journal confirms the
        // spawn happened (the pre-restart daemon succeeded in `executor.spawn`
        // AND saved the journal). Absence means the run was dispatched before
        // C2 journaling landed (pre-C2 run) — either path falls through to the
        // same `resume_handle` check below, preserving pre-C3 behavior.
        // The behavioral skip of `resume_handle` (using the journal as the sole
        // signal) is deferred to C9 (P2.5) when the coordinator journal becomes
        // the authoritative restart source.
        let journal = octx.journal_store.load(&job.pipeline_run_id).await.ok().flatten();
        let spawn_was_journaled = journal
            .as_ref()
            .is_some_and(|j| j.entries().len() == 3 && !j.is_suspended());

        if spawn_was_journaled {
            tracing::debug!(
                slug = job.slug,
                "job-dispatch: reconcile — journal confirms spawn, checking process liveness"
            );
        } else if journal.is_none() {
            tracing::debug!(
                slug = job.slug,
                "job-dispatch: reconcile — no journal (pre-C2 run), using legacy resume_handle path"
            );
        }

        // Both journal-present and journal-absent paths call resume_handle —
        // the journal tells us spawn happened but not whether the process
        // survived the restart. `Some` means the agent survived — re-attach.
        // `None` means no live process found — record an observation event and
        // let the liveness sweep decide. A single missed probe is not
        // sufficient evidence to terminate the job.
        match octx
            .executor
            .resume_handle(
                &job.slug,
                job.harness.as_deref(),
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
            }
            None => {
                tracing::info!(
                    slug = job.slug, pipeline_run_id = job.pipeline_run_id,
                    "job-dispatch: reconcile — no live process, deferring to liveness sweep"
                );
                ship_runner::emit::emit_executor_session_destroyed(
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
