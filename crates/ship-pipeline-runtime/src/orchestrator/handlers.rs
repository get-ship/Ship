//! Secondary event handlers — update forwarding, cleanup, finalization, merge.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::process::{event_types as process_event_types, ProcessKilledPayload};
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};
use ship_substrate::services::dispatch_ports::AgentPlatform;
use ship_runner::emit::persist_and_route;
use crate::orchestrator::OrchestratorContext;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Forward `job.update` messages to the agent via mesh routing.
pub async fn handle_pipeline_run_updated(
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
        "service.orchestrator",
        &serde_json::json!({
            "from": "service.orchestrator",
            "to": slug,
            "body": {
                "type": "pipeline.run.updated",
                "pipeline_run_id": entity_id,
                "payload": payload,
            }
        }),
    ) {
        Ok(e) => e.with_actor_id("service.orchestrator"),
        Err(e) => {
            tracing::warn!(entity_id, "job-dispatch: failed to build mesh.send: {e}");
            return;
        }
    };

    // Directed mesh.send — space absent by design. Daemon-internal binding for
    // shape consistency; caller_kind preserved (Mcp) to keep routing behavior.
    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    match kernel.lock().await.emit(mesh_envelope, &ctx).await {
        Ok(_) => tracing::info!(entity_id, "job-dispatch: forwarded job.update to agent"),
        Err(e) => tracing::warn!(entity_id, "job-dispatch: mesh forward failed: {e}"),
    }
}

/// Clean up worktree, tmux session, and branch for a completed/merged job.
pub async fn handle_cleanup(
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

    let pipeline_run_id = payload["pipeline_run_id"].as_str().unwrap_or(&envelope.entity_id);
    let payload_slug = payload["slug"].as_str().unwrap_or("");

    let slug = if !payload_slug.is_empty() {
        payload_slug.to_string()
    } else {
        crate::projections::pipeline_run::load_pipeline_run_by_id(pipeline_run_id)
            .ok()
            .flatten()
            .or_else(|| {
                crate::projections::pipeline_run::load_pipeline_run_by_id(&envelope.entity_id)
                    .ok()
                    .flatten()
            })
            .map(|j| j.slug)
            .unwrap_or_else(|| pipeline_run_id.to_string())
    };

    let branch = format!("job/{slug}");
    tracing::info!(slug, pipeline_run_id, branch = %branch, "job-dispatch: cleaning up job resources");
    platform.cleanup_job(&slug, &branch).await;
    tracing::info!(slug, pipeline_run_id, "job-dispatch: cleanup complete");
}

/// After `advance_pipeline` returns false on a `job.step_completed` event, emit
/// `job.completed` if all pipeline steps are now in `completed_steps`.
pub async fn finalize_pipeline_if_done(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
) {
    let payload: serde_json::Value =
        serde_json::from_str(&envelope.payload_json).unwrap_or_default();
    let pipeline_run_id = match payload["pipeline_run_id"].as_str()
        .or(Some(envelope.entity_id.as_str()))
        .filter(|s| !s.is_empty())
    {
        Some(id) => id.to_string(),
        None => return,
    };

    let job = match crate::projections::pipeline_run::load_pipeline_run_by_id(&pipeline_run_id) {
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
    tracing::info!(pipeline_run_id, slug, "job-dispatch: all pipeline steps done, emitting job.completed");
    let env = match EventEnvelope::new(
        event_types::PIPELINE_RUN_COMPLETED,
        &pipeline_run_id,
        &serde_json::json!({ "pipeline_run_id": &pipeline_run_id, "slug": &slug }),
    ) {
        Ok(e) => e.with_actor_id("service.orchestrator"),
        Err(_) => return,
    };
    persist_and_route(kernel, env, &slug, None).await;
}

/// Handle `job.cancelled` — terminate the agent process, tear down the
/// tmux session, and emit `process.killed` for audit. The projection
/// has already transitioned the job to Cancelled via the transition
/// table; this handler's job is to stop the running work.
pub async fn handle_cancelled(
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

    // Registry first (fast path), via the L0 slug-keyed teardown primitive.
    // On a registry miss, try the executor's resume path — best effort — and
    // stop the recovered handle through the same L0 teardown core.
    let pid = if let Some(pid) =
        ship_runner::stop_by_slug(kernel, &octx.registry, &octx.executor, &slug, "cancellation", "operator").await
    {
        pid
    } else if let Some(handle) = octx.executor.resume_handle(&slug, None, None).await {
        ship_runner::stop_handle(kernel, &octx.executor, &slug, handle, "cancellation", "operator").await
    } else {
        // No handle to stop — but a dispatch for this slug may be spawning
        // right now (cancel raced the spawn). Drop a tombstone so the
        // in-flight dispatch tears the child down after it registers instead
        // of leaking it. Also covers a cancel processed entirely before the
        // dispatch handler runs.
        octx.registry.mark_cancelled(&slug).await;
        tracing::info!(slug, "job-dispatch: cancel — no live handle found; tombstoned for in-flight spawn, proceeding to tmux cleanup");
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
            ship_agents::services::sessions::projection::load_session_by_slug(&slug)
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
            let env = env.with_actor_id("service.orchestrator");
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
    octx.run_locks.lock().await.remove(&envelope.entity_id);
}


/// Execute a merge step: git merge the job branch, emit `job.merged` on success.
/// Returns `true` if the merge succeeded.
pub async fn execute_merge_step(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    branch: &str,
    platform: &dyn AgentPlatform,
) -> bool {
    match platform.git_merge(branch) {
        Ok(true) => {
            tracing::info!(slug, branch, "pipeline: merge successful");
            let merged_payload = ship_substrate::events::pipeline::PipelineRunMergedPayload {
                pipeline_run_id: pipeline_run_id.to_string(),
                slug: slug.to_string(),
            };
            let merged_envelope = EventEnvelope::new(
                event_types::PIPELINE_RUN_MERGED,
                pipeline_run_id,
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
            let fail_payload = ship_substrate::events::pipeline::PipelineRunFailedPayload {
                pipeline_run_id: pipeline_run_id.to_string(),
                error: format!("merge declined: branch `{branch}` could not be merged"),
            };
            let fail_envelope = EventEnvelope::new(
                event_types::PIPELINE_RUN_FAILED,
                pipeline_run_id,
                &fail_payload,
            );
            match fail_envelope {
                Ok(envelope) => {
                    persist_and_route(kernel, envelope, slug, None).await;
                }
                Err(e) => {
                    tracing::error!(slug, "pipeline: failed to build pipeline.run.failed envelope: {e}");
                }
            }
            false
        }
        Err(e) => {
            tracing::error!(slug, "pipeline: git merge command failed: {e}");
            false
        }
    }
}
