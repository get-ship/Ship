//! Secondary event handlers — update forwarding, cleanup, finalization, merge.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::process::{event_types as process_event_types, ProcessKilledPayload};
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};
use crate::services::dispatch_ports::AgentPlatform;
use crate::services::orchestrator::emit::persist_and_route;
use crate::services::orchestrator::OrchestratorContext;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Forward `job.update` messages to the agent via mesh routing.
pub async fn handle_job_update(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
) {
    let entity_id = &envelope.entity_id;
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.update payload: {e}");
            return;
        }
    };

    let slug = payload
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or(entity_id);

    let mesh_envelope = match EventEnvelope::new(
        "mesh.send",
        "service.job-dispatch",
        &serde_json::json!({
            "from": "service.job-dispatch",
            "to": slug,
            "body": {
                "type": "pipeline.run.updated",
                "job_id": entity_id,
                "payload": payload,
            }
        }),
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(entity_id, "job-dispatch: failed to build mesh.send: {e}");
            return;
        }
    };

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        project_id: None,
        session_id: None,
    };
    match kernel.lock().await.emit(mesh_envelope, &ctx).await {
        Ok(_) => tracing::info!(entity_id, "job-dispatch: forwarded job.update to agent"),
        Err(e) => tracing::warn!(entity_id, "job-dispatch: mesh forward failed: {e}"),
    }
}

/// Clean up worktree, tmux session, and branch for a completed/merged job.
pub async fn handle_job_cleanup(
    envelope: &EventEnvelope,
    platform: &dyn AgentPlatform,
) {
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed cleanup payload: {e}");
            return;
        }
    };

    let job_id = payload["job_id"].as_str().unwrap_or(&envelope.entity_id);
    let payload_slug = payload["slug"].as_str().unwrap_or("");

    let slug = if !payload_slug.is_empty() {
        payload_slug.to_string()
    } else {
        crate::projections::pipeline_run::load_pipeline_run_by_id(job_id)
            .ok()
            .flatten()
            .or_else(|| {
                crate::projections::pipeline_run::load_pipeline_run_by_id(&envelope.entity_id)
                    .ok()
                    .flatten()
            })
            .map(|j| j.slug)
            .unwrap_or_else(|| job_id.to_string())
    };

    let branch = format!("job/{slug}");
    tracing::info!(slug, job_id, branch = %branch, "job-dispatch: cleaning up job resources");
    platform.cleanup_job(&slug, &branch).await;
    tracing::info!(slug, job_id, "job-dispatch: cleanup complete");
}

/// After `advance_pipeline` returns false on a `job.step_completed` event, emit
/// `job.completed` if all pipeline steps are now in `completed_steps`.
pub async fn finalize_pipeline_if_done(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
) {
    let payload: serde_json::Value =
        serde_json::from_str(&envelope.payload_json).unwrap_or_default();
    let job_id = match payload["job_id"].as_str()
        .or(Some(envelope.entity_id.as_str()))
        .filter(|s| !s.is_empty())
    {
        Some(id) => id.to_string(),
        None => return,
    };

    let job = match crate::projections::pipeline_run::load_pipeline_run_by_id(&job_id) {
        Ok(Some(j)) => j,
        _ => return,
    };

    let all_done = match job.steps {
        Some(ref steps) => steps.steps.iter().all(|s| job.completed_steps.contains(&s.id)),
        None => return,
    };
    if !all_done {
        return;
    }

    let slug = payload["slug"].as_str().unwrap_or(job.slug.as_str()).to_string();
    tracing::info!(job_id, slug, "job-dispatch: all pipeline steps done, emitting job.completed");
    let env = match EventEnvelope::new(
        event_types::PIPELINE_RUN_COMPLETED,
        &job_id,
        &serde_json::json!({ "job_id": &job_id, "slug": &slug }),
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(_) => return,
    };
    persist_and_route(kernel, env, &slug, None).await;
}

/// Handle `job.cancelled` — terminate the agent process, tear down the
/// tmux session, and emit `process.killed` for audit. The projection
/// has already transitioned the job to Cancelled via the transition
/// table; this handler's job is to stop the running work.
pub async fn handle_job_cancelled(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) {
    let payload: serde_json::Value =
        serde_json::from_str(&envelope.payload_json).unwrap_or_default();
    let slug = match payload["slug"].as_str() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            tracing::warn!(
                entity_id = %envelope.entity_id,
                "job-dispatch: job.cancelled missing slug — skipping process kill"
            );
            return;
        }
    };

    // Registry first (fast path). On miss, try the executor's resume path
    // — best effort. `handle` is consumed by executor.stop.
    let pid = if let Some(handle) = octx.registry.take(&slug).await {
        let pid = handle.pid;
        super::emit::emit_process_signal_sent(
            kernel, &slug, pid, "SIGTERM", "cancellation", "operator",
        ).await;
        if let Err(e) = octx.executor.stop(&handle).await {
            tracing::warn!(slug, "job-dispatch: cancel stop returned err: {e}");
        }
        pid
    } else if let Some(handle) = octx.executor.resume_handle(&slug, None, None).await {
        let pid = handle.pid;
        super::emit::emit_process_signal_sent(
            kernel, &slug, pid, "SIGTERM", "cancellation", "operator",
        ).await;
        if let Err(e) = octx.executor.stop(&handle).await {
            tracing::warn!(slug, "job-dispatch: cancel stop (via resume) returned err: {e}");
        }
        pid
    } else {
        tracing::info!(slug, "job-dispatch: cancel — no live handle found; proceeding to tmux cleanup");
        None
    };

    // Ensure tmux session + worktree are gone. Look up branch from the
    // projection so delete_branch can run.
    // Branch for cleanup: pipeline runs carry it on the jobs row; bare
    // agent sessions carry it on the agent_sessions row. Try jobs first,
    // fall back to the session projection (agent.session.cancelled).
    let branch = crate::projections::pipeline_run::load_pipeline_run_by_slug(&slug)
        .ok()
        .flatten()
        .map(|j| j.branch)
        .or_else(|| {
            crate::projections::agent_session::load_session_by_slug(&slug)
                .ok()
                .flatten()
                .map(|s| s.branch)
        })
        .unwrap_or_default();
    octx.platform.cleanup_job(&slug, &branch).await;

    // Emit process.killed for audit.
    let killed = ProcessKilledPayload {
        slug: slug.clone(),
        pid,
        signal: "SIGTERM+SIGKILL".into(),
        reason: "cancellation".into(),
        actor: "operator".into(),
    };
    match EventEnvelope::new(process_event_types::PROCESS_KILLED, &slug, &killed) {
        Ok(env) => {
            let env = env.with_actor_id("service.job-dispatch");
            persist_and_route(kernel, env, &slug, None).await;
        }
        Err(e) => {
            tracing::warn!(slug, "job-dispatch: failed to build process.killed envelope: {e}");
        }
    }

    // Fanout reliability: walk every lane registered under this job and
    // tear it down. The parent-slug kill above only hits the parent
    // fanout step (if any); lane tmux windows + worktrees would survive
    // as orphans otherwise, and the aggregator's `rx.await` would block
    // forever. Drop the lane oneshot senders so the aggregator wakes
    // promptly — it checks the parent job's terminal status before
    // emitting a misleading step.failed.
    super::fanout_cancel::cancel_job_fanout_lanes(kernel, &envelope.entity_id, octx).await;

    // Drop per-slug bookkeeping so a subsequent retry with the same slug
    // isn't confused by stale state.
    octx.done_flags.lock().await.remove(slug.as_str());
    octx.activity_txs.lock().await.remove(slug.as_str());
    octx.job_locks.lock().await.remove(&envelope.entity_id);
}


/// Execute a merge step: git merge the job branch, emit `job.merged` on success.
/// Returns `true` if the merge succeeded.
pub async fn execute_merge_step(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    job_id: &str,
    slug: &str,
    branch: &str,
    platform: &dyn AgentPlatform,
) -> bool {
    match platform.git_merge(branch) {
        Ok(true) => {
            tracing::info!(slug, branch, "pipeline: merge successful");
            let merged_payload = ship_substrate::events::pipeline::PipelineRunMergedPayload {
                job_id: job_id.to_string(),
                slug: slug.to_string(),
            };
            let merged_envelope = EventEnvelope::new(
                event_types::PIPELINE_RUN_MERGED,
                job_id,
                &merged_payload,
            );
            match merged_envelope {
                Ok(envelope) => {
                    persist_and_route(kernel, envelope, slug, None).await;
                }
                Err(e) => {
                    tracing::error!(slug, "pipeline: failed to build job.merged envelope: {e}");
                }
            }
            true
        }
        Ok(false) => {
            tracing::error!(slug, branch, "pipeline: merge failed");
            false
        }
        Err(e) => {
            tracing::error!(slug, "pipeline: git merge command failed: {e}");
            false
        }
    }
}
