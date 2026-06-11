//! Pipeline event router — dispatches kernel events to the appropriate handler.
//!
//! This is the entry point for the `pipeline.*` events (run + step lifecycle)
//! and the composed-session `agent.session.completed/failed` terminals flowing
//! through the orchestrator.

use ship_code::runner::router::{extract_slug, terminal_guard, Admit};
use ship_substrate::events::pipeline::{PipelineStepDoneDeclaredPayload, PipelineStepObservedPayload};
use ship_substrate::events::EventEnvelope;
use ship_types::event_types as et;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch::handle_pipeline_run_started;
use super::emit::emit_step_failed;
use super::review_retry::{emit_pipeline_run_failed_for_review, try_review_retry};
use super::handlers::{
    finalize_pipeline_if_done, handle_cancelled, handle_cleanup, handle_pipeline_run_updated,
};
use super::pipeline::advance_pipeline;
use super::step_io::{check_required_outputs, run_step_promotes};
use super::OrchestratorContext;

/// Bootstrap the orchestrator: start liveness monitoring, reconcile state,
/// and begin processing events from the mailbox.
///
/// Called once during daemon startup. Returns when the mailbox closes.
pub async fn start_orchestrator(
    kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: Arc<OrchestratorContext>,
) {
    let actor_id = "service.orchestrator".to_string();
    // Subscribes to two namespaces:
    //   - `pipeline.*` for composition lifecycle (multi-step pipeline runs)
    //   - `agent.session.*` to advance COMPOSED pipeline sessions only — the
    //     orchestrator reacts to `agent.session.completed/failed` keyed on the
    //     `parent_pipeline_run_id` path. It NO LONGER spawns or cancels: the
    //     bare `agent.session.started` / `agent.session.cancelled` handling
    //     carved into the `service.agents` actor (agents-arc S3b), which is the
    //     SOLE actor that acts on those two events (no-double-spawn invariant).
    // The router below switches on the event type to pick the handler.
    let config = ship_substrate::events::ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec!["pipeline.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["pipeline.".to_string(), "agent.session.".to_string()],
        see_directed_events: false,
    };

    let mailbox = {
        let mut k = kernel.lock().await;
        match k.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => mb,
            Err(e) => {
                tracing::warn!("job-dispatch: failed to spawn actor: {e}");
                return;
            }
        }
    };

    // On startup: reconcile stale jobs, replay completions, dispatch pending.
    let replay_kernel = kernel.clone();
    let replay_octx = octx.clone();
    tokio::spawn(async move {
        super::replay::reconcile_dispatched_jobs(&replay_kernel, &replay_octx).await;
        super::fanout_recover::reconcile_fanout_lanes(&replay_kernel, &replay_octx).await;
        super::replay::replay_unprocessed_completions(&replay_kernel, &replay_octx).await;
        super::pending::dispatch_ready_pending_jobs(&replay_kernel, &replay_octx).await;
        replay_octx.platform.cleanup_orphaned_worktrees().await;
    });

    // Start the periodic coherence sweep (30s interval).
    let sweep_kernel = kernel.clone();
    let sweep_octx = octx.clone();
    tokio::spawn(async move {
        super::sweep::start_coherence_sweep(sweep_kernel, sweep_octx).await;
    });

    let kr = kernel.clone();
    let mut mb = mailbox;
    while let Some(envelope) = mb.recv().await {
        // All events the orchestrator still acts on are fast and
        // order-sensitive (advancement, completion, merge, the composed
        // step.observed/done_declared bookkeeping). The slow off-mailbox
        // `agent.session.started` prep moved to `service.agents` (S3b), which
        // owns its own DISPATCH_GATE detachment — nothing left here needs it.
        handle_orchestrator_event(&kr, &envelope, &octx).await;
    }
    tracing::info!("job-dispatch: mailbox closed");
}

/// Handle a single pipeline or composed-session event.
///
/// Called from the mailbox receive loop for each `pipeline.*` or composed
/// `agent.session.completed/failed` event.
pub async fn handle_orchestrator_event(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) {
    // Runner-mechanism entry gate (L0 `ship_runner::router`): dedup by event
    // id (prevents a replayed envelope — mailbox redelivery, startup replay
    // racing live delivery, test replays — from advancing twice) AND the
    // terminal-state guard (once a job is Failed/Merged/Cancelled, late
    // pipeline events can't resurrect it; operator events are exempt). The
    // terminal check goes through the ProjectionReader port (primitive `bool`)
    // so the runner never names the projection status enum.
    match terminal_guard(&octx.event_dedup, &*octx.projections, envelope).await {
        Admit::Proceed => {}
        Admit::Duplicate | Admit::Terminal => return,
    }

    match envelope.event_type.as_str() {
        // `agent.session.started` / `agent.session.cancelled` are owned solely
        // by the `service.agents` actor (agents-arc S3b). The orchestrator no
        // longer spawns or cancels — it keeps `agent.session.completed/failed`
        // (below) only to advance COMPOSED pipeline sessions.
        "pipeline.run.started" => handle_pipeline_run_started(kernel, envelope, octx).await,
        "pipeline.run.updated" => handle_pipeline_run_updated(kernel, envelope).await,
        et::PIPELINE_RUN_CANCELLED => handle_cancelled(kernel, envelope, octx).await,
        "pipeline.run.completed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.completed");
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
            }
            if !advance_pipeline(kernel, envelope, octx).await {
                // No more steps to dispatch. If the pipeline declared
                // `auto_merge: true`, run the merge now and emit
                // `job.merged`. Otherwise the job sits in Completed.
                if maybe_auto_merge(kernel, envelope, octx).await {
                    // Merge dispatched — lock stays held until job.merged arrives.
                } else {
                    octx.run_locks.lock().await.remove(&envelope.entity_id);
                    super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
                }
            }
        }
        et::PIPELINE_STEP_COMPLETED => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.step_completed");
            if !advance_pipeline(kernel, envelope, octx).await {
                finalize_pipeline_if_done(kernel, envelope).await;
                super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
            }
        }
        et::PIPELINE_STEP_FAILED => {
            tracing::warn!(entity_id = %envelope.entity_id, "job-dispatch: pipeline step failed — advancement halted");
            let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).unwrap_or_default();
            if let Some(slug) = payload["slug"].as_str() {
                octx.registry.unregister(slug).await;
            }
        }
        et::PIPELINE_RUN_MERGED => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.merged");
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
            }
            octx.run_locks.lock().await.remove(&envelope.entity_id);
            handle_cleanup(envelope, &*octx.platform).await;
            super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
        }
        "pipeline.run.failed" => {
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
                octx.done_flags.lock().await.remove(slug.as_str());
                octx.activity_txs.lock().await.remove(slug.as_str());
            }
            octx.run_locks.lock().await.remove(&envelope.entity_id);
        }
        "pipeline.step.done_declared" => {
            let slug = extract_slug(envelope);
            // Set done flag so the step monitor knows step_done was called.
            if let Some(slug) = &slug {
                if let Some(flag) = octx.done_flags.lock().await.get(slug.as_str()) {
                    flag.store(true, Ordering::Release);
                    tracing::info!(slug, "job-dispatch: step_done declared");
                } else {
                    tracing::warn!(slug, "job-dispatch: step.done_declared but no active monitor");
                }
            }
            // Check required outputs — fail the step immediately when any are missing.
            if let Ok(declared) = serde_json::from_str::<PipelineStepDoneDeclaredPayload>(&envelope.payload_json) {
                check_and_fail_missing_outputs(kernel, &declared).await;
            }
        }
        "pipeline.step.observed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling step.observed");
            let payload: PipelineStepObservedPayload = match serde_json::from_str(&envelope.payload_json) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("job-dispatch: malformed step.observed payload: {e}");
                    return;
                }
            };
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.done_flags.lock().await.remove(slug.as_str());
                octx.activity_txs.lock().await.remove(slug.as_str());
            }

            // Fanout lane completion: route to the aggregator instead of advance_pipeline.
            if let Some(slug) = &slug {
                let lane_ref = octx.fanout_lane_refs.lock().await.get(slug.as_str()).cloned();
                if let Some(lane_ref) = lane_ref {
                    super::fanout::complete_fanout_lane(kernel, &payload, &lane_ref, octx).await;
                    return;
                }
            }

            use ship_substrate::events::pipeline::StepVerdict;

            // Evaluate any declared post-step diff gate. This short-circuits
            // positive verdicts into a structured pipeline halt when the
            // step's diff violates its declared contract.
            if !matches!(payload.verdict, StepVerdict::Failed)
                && let Some(reason) = evaluate_step_gate(&payload, octx).await
            {
                tracing::warn!(
                    entity_id = %envelope.entity_id,
                    step = payload.step_name,
                    "job-dispatch: step diff gate failed"
                );
                emit_step_gate_failure(kernel, &payload, &reason).await;
                if let Some(slug) = &slug {
                    octx.registry.unregister(slug).await;
                }
                octx.run_locks.lock().await.remove(&payload.pipeline_run_id);
                return;
            }

            match &payload.verdict {
                StepVerdict::Advance => {
                    tracing::info!(
                        entity_id = %envelope.entity_id,
                        step = payload.step_name,
                        trigger = ?payload.trigger,
                        commits = payload.commits_since_start,
                        done_declared = payload.done_declared,
                        "job-dispatch: step evidence → advance"
                    );
                    // Run promotes before advancing — failures are non-fatal.
                    run_promotes_for_step(kernel, &payload).await;
                    if !advance_pipeline(kernel, envelope, octx).await {
                        finalize_pipeline_if_done(kernel, envelope).await;
                        super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
                    }
                }
                StepVerdict::Inconclusive => {
                    // No strong signal either way. Advance rather than hang —
                    // the gate step (if any) will catch problems.
                    tracing::warn!(
                        entity_id = %envelope.entity_id,
                        step = payload.step_name,
                        trigger = ?payload.trigger,
                        commits = payload.commits_since_start,
                        done_declared = payload.done_declared,
                        "job-dispatch: step evidence inconclusive — advancing (gate will review)"
                    );
                    if !advance_pipeline(kernel, envelope, octx).await {
                        finalize_pipeline_if_done(kernel, envelope).await;
                        super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
                    }
                }
                StepVerdict::Failed => {
                    tracing::warn!(
                        entity_id = %envelope.entity_id,
                        step = payload.step_name,
                        trigger = ?payload.trigger,
                        reason = ?payload.reason,
                        ?payload.exit_code,
                        "job-dispatch: step failed"
                    );
                    if let Some(slug) = &slug {
                        octx.registry.unregister(slug).await;
                    }
                    if !try_review_retry(kernel, octx, &payload).await {
                        emit_pipeline_run_failed_for_review(kernel, &payload).await;
                    }
                }
            }
        }
        _ => {}
    }
}

/// Run `git merge` on the job's branch if the pipeline was declared with
/// `auto_merge: true`. Called from the job.completed handler when no more
/// steps are available to dispatch. Returns `true` when a merge was
/// dispatched (whether it succeeded or not).
async fn maybe_auto_merge(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) -> bool {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).unwrap_or_default();
    let pipeline_run_id = payload["pipeline_run_id"].as_str().unwrap_or(envelope.entity_id.as_str()).to_string();
    let Some(job) = crate::projections::pipeline_run::load_pipeline_run_by_id(&pipeline_run_id)
        .ok()
        .flatten()
    else {
        return false;
    };
    let Some(pipeline) = job.steps.as_ref() else {
        return false;
    };
    if !pipeline.auto_merge {
        return false;
    }
    tracing::info!(
        pipeline_run_id = %job.pipeline_run_id, slug = %job.slug, branch = %job.branch,
        "job-dispatch: auto_merge — running merge after final step"
    );
    octx.platform.graceful_agent_exit(&job.slug, None).await;
    super::handlers::execute_merge_step(
        kernel,
        &job.pipeline_run_id,
        &job.slug,
        &job.branch,
        &*octx.platform,
    )
    .await
}

/// Check that all `required: true` outputs were reported in `step.done_declared`.
/// When any are missing, emits `step.failed` + `job.failed` immediately.
async fn check_and_fail_missing_outputs(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    declared: &PipelineStepDoneDeclaredPayload,
) {
    let Some(job) = crate::projections::pipeline_run::load_pipeline_run_by_id(&declared.pipeline_run_id)
        .ok()
        .flatten()
    else {
        return;
    };
    let Some(pipeline) = job.steps.as_ref() else { return };
    let Some(step) = pipeline.steps.iter().find(|s| s.id == declared.step_id) else {
        return;
    };
    if let Some(reason) = check_required_outputs(&step.produces, &declared.outputs) {
        tracing::warn!(
            pipeline_run_id = declared.pipeline_run_id, step_id = declared.step_id,
            "job-dispatch: required output missing: {reason}"
        );
        emit_step_failed(
            kernel, &declared.pipeline_run_id, &declared.slug, &declared.step_id, &reason,
        ).await;
    }
}

/// Run promotes for the step that just advanced. Non-fatal — failures are
/// logged but do not block pipeline advancement.
async fn run_promotes_for_step(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineStepObservedPayload,
) {
    let Some(job) = crate::projections::pipeline_run::load_pipeline_run_by_id(&payload.pipeline_run_id)
        .ok()
        .flatten()
    else {
        return;
    };
    let Some(pipeline) = job.steps.as_ref() else { return };
    let Some(step) = pipeline.steps.iter().find(|s| s.id == payload.step_name) else {
        return;
    };
    if !step.promotes.is_empty() {
        run_step_promotes(kernel, &payload.pipeline_run_id, step).await;
    }
}

/// Evaluate the step's declared `gate:` block against its diff. Returns a
/// failure reason if the gate halts the pipeline, `None` to continue.
async fn evaluate_step_gate(
    payload: &PipelineStepObservedPayload,
    octx: &OrchestratorContext,
) -> Option<String> {
    use super::gate_rules::{diff_paths, evaluate, GateOutcome};

    let job = crate::projections::pipeline_run::load_pipeline_run_by_id(&payload.pipeline_run_id)
        .ok()
        .flatten()?;
    let pipeline = job.steps.as_ref()?;
    let step = pipeline.steps.iter().find(|s| s.id == payload.step_name)?;
    let gate = step.gate.as_ref()?;

    // Gates evaluate `require_changed` / `forbid_changed` across the whole
    // pipeline run. Prefer the job-level baseline SHA captured at dispatch so
    // commits from earlier steps are visible; fall back to the gating step's
    // own start SHA for legacy events (pre-baseline_sha) that don't carry it.
    let start_sha = super::git_helpers::gate_baseline_sha(
        job.baseline_sha.as_deref(),
        payload.start_sha.as_deref(),
    );
    let commit_range = if start_sha.is_empty() {
        "<no baseline>..HEAD".to_string()
    } else {
        format!("{start_sha}..HEAD")
    };

    let work_dir = octx.platform.worktrees_dir().join(&payload.slug);
    let changed = if start_sha.is_empty() {
        Vec::new()
    } else {
        diff_paths(&work_dir, &start_sha)
    };

    match evaluate(Some(gate), &changed, &commit_range) {
        GateOutcome::Pass => None,
        GateOutcome::Fail(reason) => Some(reason),
    }
}

/// Emit `job.step_failed` + `job.failed` for a gate violation. Mirrors the
/// existing pipeline-halt shape — no new error taxonomy.
async fn emit_step_gate_failure(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineStepObservedPayload,
    reason: &str,
) {
    use ship_substrate::events::pipeline::{event_types, PipelineRunFailedPayload, PipelineStepFailedPayload};

    let step_fail = PipelineStepFailedPayload {
        pipeline_run_id: payload.pipeline_run_id.clone(),
        slug: payload.slug.clone(),
        step_id: payload.step_name.clone(),
        error: reason.to_string(),
    };
    if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_FAILED, &payload.pipeline_run_id, &step_fail) {
        let env = env.with_actor_id("service.orchestrator");
        super::emit::persist_and_route(kernel, env, &payload.slug, None).await;
    }

    let base_error = format!("step `{}` gate failed: {reason}", payload.step_name);
    let error = match &payload.log_tail {
        Some(tail) => format!("{base_error}\n\nagent log tail:\n{tail}"),
        None => base_error,
    };
    let run_fail = PipelineRunFailedPayload {
        pipeline_run_id: payload.pipeline_run_id.clone(),
        error,
    };
    if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_RUN_FAILED, &payload.pipeline_run_id, &run_fail) {
        let env = env.with_actor_id("service.orchestrator");
        super::emit::persist_and_route(kernel, env, &payload.slug, None).await;
    }
}
