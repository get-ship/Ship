//! Job event router — dispatches kernel events to the appropriate handler.
//!
//! This is the entry point for all `job.*` and `step.*` events flowing
//! through the orchestrator.

use crate::events::job::StepObservedPayload;
use crate::events::EventEnvelope;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch::handle_job_created;
use super::review_retry::{emit_job_failed_for_review, try_review_retry};
use super::handlers::{
    finalize_pipeline_if_done, handle_job_cancelled, handle_job_cleanup, handle_job_update,
};
use super::pipeline::advance_pipeline;
use super::OrchestratorContext;

/// Bootstrap the orchestrator: start liveness monitoring, reconcile state,
/// and begin processing events from the mailbox.
///
/// Called once during daemon startup. Returns when the mailbox closes.
pub async fn start_orchestrator(
    kernel: Arc<Mutex<crate::events::KernelRouter>>,
    octx: Arc<OrchestratorContext>,
) {
    let actor_id = "service.job-dispatch".to_string();
    let config = crate::events::ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec!["job.".to_string(), "step.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
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

    // Start the periodic liveness sweep.
    octx.platform.start_liveness_sweep(kernel.clone());

    // On startup: reconcile stale jobs, replay completions, dispatch pending.
    let replay_kernel = kernel.clone();
    let replay_octx = octx.clone();
    tokio::spawn(async move {
        super::replay::reconcile_dispatched_jobs(&replay_kernel, &replay_octx).await;
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
        handle_job_event(&kr, &envelope, &octx).await;
    }
    tracing::info!("job-dispatch: mailbox closed");
}

/// Handle a single job/step event.
///
/// Called from the mailbox receive loop for each `job.*` or `step.*` event.
pub async fn handle_job_event(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) {
    // Dedup by event id. Prevents a replayed envelope — mailbox
    // redelivery, startup replay racing live delivery, test harness
    // replays — from advancing the pipeline twice.
    if octx.event_dedup.record(&envelope.id) == super::dedup::Seen::Duplicate {
        tracing::debug!(
            event_id = %envelope.id,
            event_type = %envelope.event_type,
            "job-dispatch: duplicate event — skipping"
        );
        return;
    }

    // Terminal-state guard. Once a job is Failed/Merged/Cancelled the
    // router ignores subsequent pipeline events for it — late-arriving
    // step.observed signals (kill-session firing after teardown,
    // replayed deliveries) cannot resurrect a dead job. Operator events
    // (job.created/update/cancelled) are exempt: those are signals FROM
    // the operator, not pipeline progression.
    if !is_operator_event(&envelope.event_type)
        && let Some(record) = load_job_record(&envelope.entity_id)
        && matches!(
            record.status,
            crate::projections::job::JobStatus::Failed
                | crate::projections::job::JobStatus::Merged
                | crate::projections::job::JobStatus::Cancelled
        )
    {
        tracing::warn!(
            entity_id = %envelope.entity_id,
            event_type = %envelope.event_type,
            status = ?record.status,
            "job-dispatch: ignoring event for terminal job"
        );
        return;
    }

    match envelope.event_type.as_str() {
        "job.created" => handle_job_created(kernel, envelope, octx).await,
        "job.update" => handle_job_update(kernel, envelope).await,
        "job.cancelled" => handle_job_cancelled(kernel, envelope, octx).await,
        "job.completed" => {
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
                    octx.job_locks.lock().await.remove(&envelope.entity_id);
                    super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
                }
            }
        }
        "job.step_completed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.step_completed");
            if !advance_pipeline(kernel, envelope, octx).await {
                finalize_pipeline_if_done(kernel, envelope).await;
                super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
            }
        }
        "job.step_failed" => {
            tracing::warn!(entity_id = %envelope.entity_id, "job-dispatch: pipeline step failed — advancement halted");
            let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).unwrap_or_default();
            if let Some(slug) = payload["slug"].as_str() {
                octx.registry.unregister(slug).await;
            }
        }
        "job.merged" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.merged");
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
            }
            octx.job_locks.lock().await.remove(&envelope.entity_id);
            handle_job_cleanup(envelope, &*octx.platform).await;
            super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
        }
        "job.failed" => {
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
                octx.done_flags.lock().await.remove(slug.as_str());
                octx.activity_txs.lock().await.remove(slug.as_str());
            }
            octx.job_locks.lock().await.remove(&envelope.entity_id);
        }
        "step.done_declared" => {
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                if let Some(flag) = octx.done_flags.lock().await.get(slug.as_str()) {
                    flag.store(true, Ordering::Release);
                    tracing::info!(slug, "job-dispatch: step_done declared");
                } else {
                    tracing::warn!(slug, "job-dispatch: step.done_declared but no active monitor");
                }
            }
        }
        "step.observed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling step.observed");
            let payload: StepObservedPayload = match serde_json::from_str(&envelope.payload_json) {
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

            use crate::events::job::StepVerdict;

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
                octx.job_locks.lock().await.remove(&payload.job_id);
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
                        emit_job_failed_for_review(kernel, &payload).await;
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
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) -> bool {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).unwrap_or_default();
    let job_id = payload["job_id"].as_str().unwrap_or(envelope.entity_id.as_str()).to_string();
    let jobs = match crate::projections::job::load_jobs() {
        Ok(j) => j,
        Err(_) => return false,
    };
    let Some(job) = jobs.values().find(|j| j.job_id == job_id).cloned() else {
        return false;
    };
    let Some(pipeline) = job.steps.as_ref() else {
        return false;
    };
    if !pipeline.auto_merge {
        return false;
    }
    tracing::info!(
        job_id = %job.job_id, slug = %job.slug, branch = %job.branch,
        "job-dispatch: auto_merge — running merge after final step"
    );
    octx.platform.graceful_agent_exit(&job.slug, None).await;
    super::handlers::execute_merge_step(
        kernel,
        &job.job_id,
        &job.slug,
        &job.branch,
        &*octx.platform,
    )
    .await
}

/// Events originating from the operator (or system policy), not from
/// pipeline progression. These are exempt from the terminal-state guard
/// so operators can still issue signals about a finalized job.
fn is_operator_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "job.created" | "job.update" | "job.cancelled"
    )
}

/// Load the projected record for a single job by entity_id.
///
/// Used by the terminal-state guard. Returns `None` if the projection
/// can't be loaded or the job doesn't exist — callers treat that as
/// "not terminal, proceed" so startup races don't drop events.
fn load_job_record(entity_id: &str) -> Option<crate::projections::job::JobRecord> {
    crate::projections::job::load_jobs()
        .ok()?
        .into_iter()
        .map(|(_, v)| v)
        .find(|r| r.job_id == entity_id)
}

/// Extract the slug from an event envelope payload.
fn extract_slug(envelope: &EventEnvelope) -> Option<String> {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).ok()?;
    payload["slug"].as_str().map(str::to_string)
}

/// Evaluate the step's declared `gate:` block against its diff. Returns a
/// failure reason if the gate halts the pipeline, `None` to continue.
async fn evaluate_step_gate(
    payload: &StepObservedPayload,
    octx: &OrchestratorContext,
) -> Option<String> {
    use super::gate_rules::{diff_paths, evaluate, GateOutcome};

    let jobs = crate::projections::job::load_jobs().ok()?;
    let job = jobs.values().find(|j| j.job_id == payload.job_id)?.clone();
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
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    payload: &StepObservedPayload,
    reason: &str,
) {
    use crate::events::job::{event_types, JobFailedPayload, StepFailedPayload};

    let step_fail = StepFailedPayload {
        job_id: payload.job_id.clone(),
        slug: payload.slug.clone(),
        step_id: payload.step_name.clone(),
        error: reason.to_string(),
    };
    if let Ok(env) = EventEnvelope::new(event_types::STEP_FAILED, &payload.job_id, &step_fail) {
        let env = env.with_actor_id("service.job-dispatch");
        super::emit::persist_and_route(kernel, env, &payload.slug, None).await;
    }

    let base_error = format!("step `{}` gate failed: {reason}", payload.step_name);
    let error = match &payload.log_tail {
        Some(tail) => format!("{base_error}\n\nagent log tail:\n{tail}"),
        None => base_error,
    };
    let job_fail = JobFailedPayload {
        job_id: payload.job_id.clone(),
        error,
    };
    if let Ok(env) = EventEnvelope::new(event_types::JOB_FAILED, &payload.job_id, &job_fail) {
        let env = env.with_actor_id("service.job-dispatch");
        super::emit::persist_and_route(kernel, env, &payload.slug, None).await;
    }
}
