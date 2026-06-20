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

use ship_substrate::events::pipeline::{FanoutSpec, PipelineStep, StepType};
use ship_substrate::events::KernelRouter;
use crate::projections::pipeline_run::PipelineRun;
use ship_code::dispatch_ports::{SessionContext, LaunchProfile, StepInfo};
use ship_runner::emit::{
    emit_process_spawned, emit_step_started_event, emit_step_failed,
};
use crate::orchestrator::step_monitor::{StepMonitor, start_step_monitor};
use crate::orchestrator::{FanoutLaneRef, LaneOutcome, OrchestratorContext};

// Re-exports so callers that `use super::fanout::*` keep working.
pub use crate::orchestrator::fanout_aggregate::{
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
        match super::step_io::resolve_consumes(kernel, step, pipeline, &job.pipeline_run_id, &parent_wt).await {
            Ok(inputs) => inputs,
            Err(e) => {
                tracing::error!(slug, step_id = step.id, "fanout: consumes resolution failed: {e}");
                let _ = emit_step_failed(kernel, &job.pipeline_run_id, slug, &step.id, &e).await;
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

        let lane_ctx = SessionContext {
            pipeline_run_id: job.pipeline_run_id.clone(),
            slug: lane_slug.clone(),
            agent: lane.agent.clone(),
            branch: lane_branch,
            spec_path: job.spec_path.clone(),
            spec_content: None,
            work_dir: lane_wt.clone(),
            env: ship_code::runner::build_spawn_env(&lane_slug, &lane.agent, octx.spawn_policy.as_ref()).await,
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
                &lane_wt, &lane_slug, &job.pipeline_run_id,
                &job.spec_path, pipeline, step, &shared_inputs,
            );
        }

        let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
        let lane_ctx_with_exit = SessionContext { exit_tx: Some(exit_tx), work_dir: lane_wt.clone(), ..lane_ctx };

        let handle = match octx.executor.spawn(&lane_ctx_with_exit).await {
            Ok(h) => h,
            Err(e) => {
                tracing::error!(slug, lane_id = lane.id, "fanout: spawn failed: {e}");
                continue;
            }
        };

        let exec_kind = if job.headless { "process" } else { "pty" };
        emit_process_spawned(
            kernel, &lane_slug, &job.pipeline_run_id, handle.pid,
            "claude", &[], &[], &lane_wt.to_string_lossy(),
            exec_kind, Some(&lane_step_id),
        ).await;
        octx.registry.register(&lane_slug, handle).await;

        let monitor = StepMonitor::new(
            job.pipeline_run_id.clone(),
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
            pipeline_run_id: job.pipeline_run_id.clone(),
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
            octx.journal_store.clone(),
        );

        // Persist a lane-scoped step.dispatched with `fanout_parent_step`
        // + `lane_slug` so the restart reconciler can enumerate lanes
        // after a daemon crash — `fanout_lane_refs` lives in-memory only.
        super::fanout_recover::emit_fanout_lane_dispatched(
            kernel, &job.pipeline_run_id, slug, &step.id,
            &lane.id, &lane_slug, &lane_step_id,
        ).await;

        lane_rxs.push((lane.id.clone(), rx));
        spawned += 1;
    }

    if spawned == 0 {
        let _ = emit_step_failed(kernel, &job.pipeline_run_id, slug, &step.id, "no lanes spawned").await;
        return 0;
    }

    // Journal-driven aggregation (C6/P2.4): the wait → aggregate → emit
    // sequence runs as FanoutProgram through the durable-execution engine.
    // Lane outcomes still resolve through the per-lane oneshots the router
    // fills; the aggregate decision and the parent emission are journaled
    // (exactly-once under replay). The hand-rolled aggregator task is gone.
    let lane_ops: Vec<String> = lane_rxs
        .iter()
        .map(|(lane_id, _)| format!("fanout.lane.{lane_id}"))
        .collect();
    let dispatch = std::sync::Arc::new(super::fanout_driver::FanoutDispatch {
        kernel: kernel.clone(),
        pipeline_run_id: job.pipeline_run_id.clone(),
        parent_slug: slug.clone(),
        step_id: step.id.clone(),
        step_idx,
        total: fanout.lanes.len(),
        started_at: Instant::now(),
        lane_rxs: Mutex::new(
            lane_rxs
                .into_iter()
                .map(|(lane_id, rx)| (format!("fanout.lane.{lane_id}"), rx))
                .collect(),
        ),
    });
    super::fanout_driver::start_fanout_driver(
        octx.journal_store.clone(),
        dispatch,
        lane_ops,
        fanout.min_successful_lanes,
    );

    // Emit step_started for the fanout step so the projection sets current_step.
    emit_step_started_event(kernel, &job.pipeline_run_id, &step.id, "fanout", slug).await;

    spawned
}
