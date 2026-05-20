//! Fanout step dispatch — runs K lanes in parallel, then aggregates results.
//!
//! Each lane gets its own worktree, slug, and step monitor. When all/min
//! lanes complete the aggregator emits a synthetic `step.observed` for the
//! parent fanout step, which advances the pipeline to the judge step.
//!
//! Lane completion, the failed-lane marker, and the pure
//! `Advance`/`Fail` decision live in [`super::fanout_aggregate`].

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{
    FanoutSpec, PipelineStep, PipelineStepObservedPayload, StepTrigger, StepType, StepVerdict, event_types,
};
use ship_substrate::events::{EventEnvelope, KernelRouter};
use crate::projections::pipeline_run::PipelineRun;
use crate::services::dispatch_ports::{JobContext, LaunchProfile, StepInfo};
use crate::services::orchestrator::emit::{
    emit_process_spawned, emit_step_dispatched_event, emit_step_failed, persist_and_route,
};
use crate::services::orchestrator::fanout_aggregate::{AggregateDecision, aggregate_lane_outcomes};
use crate::services::orchestrator::step_monitor::{StepMonitor, start_step_monitor};
use crate::services::orchestrator::{FanoutLaneRef, LaneOutcome, OrchestratorContext};

// Re-exports so callers that `use super::fanout::*` keep working.
pub use crate::services::orchestrator::fanout_aggregate::{
    FAILED_MARKER_NAME, complete_fanout_lane,
};

/// Dispatch all lanes of a fanout step concurrently. Spawns a background
/// aggregator that waits for all/min lanes and then emits `step.observed`
/// for the parent fanout step.
///
/// Returns the number of lanes successfully spawned (for the caller to
/// determine whether `dispatched_any` should be true).
pub async fn dispatch_fanout_step(
    kernel: &Arc<Mutex<KernelRouter>>,
    job: &PipelineRun,
    step: &PipelineStep,
    fanout: &FanoutSpec,
    step_idx: usize,
    ship_dir_override: Option<&Path>,
    octx: &OrchestratorContext,
) -> usize {
    let wt_root = ship_dir_override
        .and_then(|p| p.parent())
        .map(|p| p.join("worktrees"))
        .unwrap_or_else(|| octx.platform.worktrees_dir());
    let slug = &job.slug;

    // Resolve shared inputs once — all lanes read the same upstream artifacts.
    let shared_inputs = if let Some(pipeline) = &job.steps {
        let parent_wt = wt_root.join(slug);
        match super::step_io::resolve_consumes(step, pipeline, &job.job_id, &parent_wt) {
            Ok(inputs) => inputs,
            Err(e) => {
                tracing::error!(slug, step_id = step.id, "fanout: consumes resolution failed: {e}");
                let _ = emit_step_failed(kernel, &job.job_id, slug, &step.id, &e).await;
                return 0;
            }
        }
    } else {
        vec![]
    };

    let mut spawned = 0;
    let mut lane_rxs: Vec<(String, tokio::sync::oneshot::Receiver<LaneOutcome>)> = Vec::new();

    for lane in &fanout.lanes {
        let lane_slug = format!("{}-{}-{}", slug, step.id, lane.id);
        let lane_step_id = format!("{}.{}", step.id, lane.id);
        let lane_branch = format!("{}-{}-{}", job.branch, step.id, lane.id);
        let lane_wt = wt_root.join(&lane_slug);

        let harness = lane.harness.clone()
            .or_else(|| step.harness.clone())
            .or_else(|| job.harness.clone());
        let model = lane.model.clone()
            .or_else(|| step.model.clone())
            .or_else(|| job.model.clone());

        let lane_ctx = JobContext {
            job_id: job.job_id.clone(),
            slug: lane_slug.clone(),
            agent: lane.agent.clone(),
            branch: lane_branch,
            spec_path: job.spec_path.clone(),
            spec_content: None,
            work_dir: lane_wt.clone(),
            env: [("SHIP_ACTOR_ID".to_string(), lane_slug.clone())].into(),
            exit_tx: None,
            launch: LaunchProfile {
                harness,
                model,
                persist_session: true,
                ..Default::default()
            },
            step: Some(StepInfo { index: step_idx, id: lane_step_id.clone() }),
            invocation: super::dispatch::resolve_step_invocation(
                step.invocation.as_deref(),
                job.headless,
            ),
            // Fanout lanes are pipeline steps, never long-running.
            keepalive: false,
        };

        let lane_wt = match octx.isolation.prepare(&lane_ctx).await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(slug, lane_id = lane.id, "fanout: isolation prepare failed: {e}");
                continue;
            }
        };

        // Write pipeline specs + lane dispatch prompt to the lane's worktree.
        if let Some(pipeline) = &job.steps {
            super::pipeline_specs::prepare_step_for_dispatch(
                &lane_wt, &lane_slug, &job.job_id,
                &job.spec_path, pipeline, step, &shared_inputs,
            );
        }

        let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
        let lane_ctx_with_exit = JobContext { exit_tx: Some(exit_tx), work_dir: lane_wt.clone(), ..lane_ctx };

        let handle = match octx.executor.spawn(&lane_ctx_with_exit).await {
            Ok(h) => h,
            Err(e) => {
                tracing::error!(slug, lane_id = lane.id, "fanout: spawn failed: {e}");
                continue;
            }
        };

        let exec_kind = if job.headless { "process" } else { "pty" };
        emit_process_spawned(
            kernel, &lane_slug, &job.job_id, handle.pid,
            "claude", &[], &[], &lane_wt.to_string_lossy(),
            exec_kind, Some(&lane_step_id),
        ).await;
        octx.registry.register(&lane_slug, handle).await;

        let monitor = StepMonitor::new(
            job.job_id.clone(),
            step_idx,
            lane_slug.clone(),
            lane_step_id.clone(),
            StepType::Agent,
            lane_wt.clone(),
        );
        let (activity_tx, activity_rx) = tokio::sync::mpsc::unbounded_channel();
        octx.done_flags.lock().await.insert(lane_slug.clone(), monitor.done_flag());
        octx.activity_txs.lock().await.insert(lane_slug.clone(), activity_tx);

        let (tx, rx) = tokio::sync::oneshot::channel::<LaneOutcome>();
        octx.fanout_lane_txs.lock().await.insert(lane_slug.clone(), tx);
        octx.fanout_lane_refs.lock().await.insert(lane_slug.clone(), FanoutLaneRef {
            job_id: job.job_id.clone(),
            parent_slug: slug.clone(),
            fanout_step_id: step.id.clone(),
            fanout_step_index: step_idx,
            lane_id: lane.id.clone(),
        });

        start_step_monitor(
            monitor,
            kernel.clone(),
            octx.registry.clone(),
            octx.executor.clone(),
            octx.probe.clone(),
            exit_rx,
            activity_rx,
        );

        // Persist a lane-scoped step.dispatched with `fanout_parent_step`
        // + `lane_slug` so the restart reconciler can enumerate lanes
        // after a daemon crash — `fanout_lane_refs` lives in-memory only.
        super::fanout_recover::emit_fanout_lane_dispatched(
            kernel, &job.job_id, slug, &step.id,
            &lane.id, &lane_slug, &lane_step_id,
        ).await;

        lane_rxs.push((lane.id.clone(), rx));
        spawned += 1;
    }

    if spawned == 0 {
        let _ = emit_step_failed(kernel, &job.job_id, slug, &step.id, "no lanes spawned").await;
        return 0;
    }

    // Background aggregator: wait for all lane completions, then emit parent step outcome.
    let kernel_clone = kernel.clone();
    let job_id = job.job_id.clone();
    let parent_slug = slug.clone();
    let step_id = step.id.clone();
    let min_ok = fanout.min_successful_lanes;
    let total = fanout.lanes.len();
    let start = Instant::now();

    tokio::spawn(async move {
        let mut outcomes: Vec<LaneOutcome> = Vec::with_capacity(lane_rxs.len());
        for (_lane_id, rx) in lane_rxs {
            match rx.await {
                Ok(outcome) => outcomes.push(outcome),
                Err(_) => {
                    // Sender dropped (lane dispatch failed silently after
                    // registering rx). Treat as a failed lane with no outputs.
                }
            }
        }

        // If every lane's sender was dropped without a value AND the
        // parent job is already terminal (typically Cancelled), exit
        // quietly. The terminal-state guard would drop any step.failed
        // we emitted anyway, and emitting one would pollute the event
        // log with a misleading "insufficient successful lanes" reason
        // for an operator-initiated cancellation.
        if outcomes.is_empty() && parent_job_is_terminal(&job_id) {
            tracing::info!(
                job_id, step_id,
                "fanout: aggregator exiting — parent job is terminal (cancelled or failed)"
            );
            return;
        }

        let decision = aggregate_lane_outcomes(&outcomes, min_ok, total);
        let duration_secs = start.elapsed().as_secs();

        let ok_count = match decision {
            AggregateDecision::Fail { ok_count, reason } => {
                tracing::warn!(
                    job_id, step_id, ok_count, min_ok, total,
                    "fanout: insufficient successful lanes — failing fanout step"
                );
                emit_step_failed(&kernel_clone, &job_id, &parent_slug, &step_id, &reason).await;
                return;
            }
            AggregateDecision::Advance { ok_count } => ok_count,
        };

        // Emit synthetic step.observed(Advance) for the parent fanout step.
        let payload = PipelineStepObservedPayload {
            job_id: job_id.clone(),
            slug: parent_slug.clone(),
            step_index: step_idx,
            step_name: step_id.clone(),
            trigger: StepTrigger::DoneDeclared,
            done_declared: true,
            done_result: Some(true),
            exit_code: None,
            commits_since_start: 0,
            uncommitted_files: 0,
            duration_secs,
            reason: Some(format!("fanout: {ok_count}/{total} lanes succeeded")),
            verdict: StepVerdict::Advance,
            succeeded: true,
            start_sha: None,
            log_tail: None,
        };
        if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_OBSERVED, &job_id, &payload) {
            let env = env.with_actor_id("service.job-dispatch");
            persist_and_route(&kernel_clone, env, &parent_slug, None).await;
        }
    });

    // Emit step_dispatched for the fanout step so the projection sets current_step.
    emit_step_dispatched_event(kernel, &job.job_id, &step.id, "fanout", slug, ship_dir_override).await;

    spawned
}

/// Peek at the projection to see whether `job_id` is in a terminal state.
/// Used by the fanout aggregator to suppress misleading `step.failed`
/// emissions after operator cancellation (or any other path that drops
/// every lane sender without reporting an outcome).
///
/// Returns `false` when the projection can't be loaded so we never
/// swallow a genuine fanout failure.
fn parent_job_is_terminal(job_id: &str) -> bool {
    use crate::projections::pipeline_run::{PipelineRunStatus, load_pipeline_run_by_id};
    match load_pipeline_run_by_id(job_id) {
        Ok(Some(job)) => matches!(
            job.status,
            PipelineRunStatus::Cancelled | PipelineRunStatus::Failed | PipelineRunStatus::Merged
        ),
        _ => false,
    }
}
