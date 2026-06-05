//! Side helpers used by the step monitor's main loop: kill-on-timeout and
//! `step.observed` emission. Separated from the loop itself to keep `mod.rs`
//! under the 300-line file cap.
//!
//! The git/log/string scanning utilities that used to live here are pure L0
//! mechanism and now live in [`ship_runner::observe`]; the kill-on-timeout
//! teardown is the L0 [`ship_runner::stop_by_slug`] primitive. This module
//! keeps only the pipeline-domain `step.observed` emission.

use std::sync::Arc;

use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{PipelineStepObservedPayload, StepTrigger, StepVerdict, event_types};
use ship_substrate::events::{EventEnvelope, KernelRouter};
use ship_substrate::services::dispatch_ports::{SessionExecutor, ProcessRegistryPort};

use ship_runner::observe::scan_agent_log_tail;

use super::StepMonitor;

/// Stop the dispatched process for `slug` on a step timeout, via the L0
/// teardown primitive. `emit_and_finish` calls `registry.unregister` after us,
/// and `stop_by_slug` `take`s the handle, so there's no double-remove.
pub(super) async fn kill_spawned_process(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<dyn ProcessRegistryPort>,
    executor: &Arc<dyn SessionExecutor>,
    slug: &str,
) {
    ship_runner::stop_by_slug(kernel, registry, executor, slug, "step_timeout", "step_monitor").await;
}

/// Build the payload, emit `step.observed`, and unregister from the registry.
#[allow(clippy::too_many_arguments)]
pub(super) async fn emit_and_finish(
    monitor: &StepMonitor,
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<dyn ProcessRegistryPort>,
    trigger: StepTrigger,
    exit_code: Option<i32>,
    done_declared: bool,
    reason: Option<String>,
    commits: u32,
    uncommitted: u32,
    turns: u32,
) {
    registry.unregister(&monitor.slug).await;

    let done_result = if done_declared { Some(true) } else { None };
    let verdict = PipelineStepObservedPayload::compute_verdict(
        &monitor.step_type, done_declared, done_result,
        exit_code, commits, uncommitted, &trigger,
    );
    let duration_secs = monitor.started_at.elapsed().as_secs();
    let succeeded = verdict == StepVerdict::Advance;
    // Captured before `verdict` moves into the payload below.
    let step_is_failure = verdict == StepVerdict::Failed;

    tracing::info!(
        slug = monitor.slug, step = monitor.step_name,
        ?trigger, ?verdict, done_declared, ?exit_code,
        commits, uncommitted, turns, ?reason,
        "step-monitor: emitting observation"
    );

    // Capture the tail of the agent's stdout/stderr log when the step
    // didn't Advance, so operators see error messages without shelling
    // into the worktree. Successful steps skip this to keep the event
    // stream lean.
    let log_tail = if succeeded {
        None
    } else {
        scan_agent_log_tail(&monitor.work_dir, &monitor.slug)
    };

    let payload = PipelineStepObservedPayload {
        pipeline_run_id: monitor.pipeline_run_id.clone(),
        slug: monitor.slug.clone(),
        step_index: monitor.step_index,
        step_name: monitor.step_name.clone(),
        trigger,
        done_declared,
        done_result,
        exit_code,
        commits_since_start: commits,
        uncommitted_files: uncommitted,
        duration_secs,
        reason,
        verdict,
        succeeded,
        start_sha: monitor.start_sha.clone(),
        log_tail,
    };

    ship_runner::emit::emit_process_exited(
        kernel, &monitor.slug, None, exit_code, None,
        duration_secs * 1000, "unknown",
    ).await;

    emit_step_observed(kernel, &monitor.pipeline_run_id, &monitor.slug, &payload).await;

    // Terminalize the composed agent_session so it doesn't sit at
    // Dispatched forever. Failed verdict → session failed; Advance /
    // Inconclusive (both let the pipeline proceed) → completed.
    crate::orchestrator::dispatch::emit_composed_session_terminal(
        kernel,
        &monitor.slug,
        &monitor.step_name,
        step_is_failure,
        exit_code,
    )
    .await;
}

async fn emit_step_observed(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    payload: &PipelineStepObservedPayload,
) {
    let envelope = match EventEnvelope::new(event_types::PIPELINE_STEP_OBSERVED, pipeline_run_id, payload) {
        Ok(e) => e.with_actor_id("service.orchestrator"),
        Err(e) => {
            tracing::warn!(slug, "step-monitor: failed to build envelope: {e}");
            return;
        }
    };

    ship_runner::emit::persist_and_route(kernel, envelope, slug, None).await;
}
