//! Secondary event handlers — update forwarding, cleanup, finalization, merge.

use crate::events::job::event_types;
use crate::events::{CallerKind, EmitContext, EventEnvelope};
use crate::services::dispatch_ports::AgentPlatform;
use crate::services::orchestrator::emit::persist_and_route;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Forward `job.update` messages to the agent via mesh routing.
pub async fn handle_job_update(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
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
                "type": "job.update",
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
        workspace_id: None,
        session_id: None,
    };
    match kernel.lock().await.route(mesh_envelope, &ctx).await {
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
    } else if let Ok(jobs) = crate::projections::job::load_jobs() {
        jobs.values()
            .find(|j| j.job_id == job_id || j.job_id == envelope.entity_id)
            .map(|j| j.slug.clone())
            .unwrap_or_else(|| job_id.to_string())
    } else {
        job_id.to_string()
    };

    let branch = format!("job/{slug}");
    tracing::info!(slug, job_id, branch = %branch, "job-dispatch: cleaning up job resources");
    platform.cleanup_job(&slug, &branch).await;
    tracing::info!(slug, job_id, "job-dispatch: cleanup complete");
}

/// After `advance_pipeline` returns false on a `job.step_completed` event, emit
/// `job.completed` if all pipeline steps are now in `completed_steps`.
pub async fn finalize_pipeline_if_done(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
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

    let jobs = match crate::projections::job::load_jobs() {
        Ok(j) => j,
        Err(_) => return,
    };
    let job = match jobs.values().find(|j| j.job_id == job_id) {
        Some(j) => j.clone(),
        None => return,
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
        event_types::JOB_COMPLETED,
        &job_id,
        &serde_json::json!({ "job_id": &job_id, "slug": &slug }),
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(_) => return,
    };
    persist_and_route(kernel, env, &slug, None).await;
}

/// Execute a merge step: git merge the job branch, emit `job.merged` on success.
/// Returns `true` if the merge succeeded.
pub async fn execute_merge_step(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    job_id: &str,
    slug: &str,
    branch: &str,
    platform: &dyn AgentPlatform,
) -> bool {
    match platform.git_merge(branch) {
        Ok(true) => {
            tracing::info!(slug, branch, "pipeline: merge successful");
            let merged_payload = crate::events::job::JobMergedPayload {
                job_id: job_id.to_string(),
                slug: slug.to_string(),
            };
            let merged_envelope = EventEnvelope::new(
                event_types::JOB_MERGED,
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
