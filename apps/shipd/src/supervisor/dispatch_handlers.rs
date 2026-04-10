//! Secondary event handlers for job dispatch — update forwarding, cleanup, DAG unblocking.

use runtime::events::job::event_types;
use runtime::events::{CallerKind, EmitContext, EventEnvelope, EventStore, SqliteEventStore};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{delete_branch, kill_tmux_session, remove_worktree, worktrees_dir};

/// Forward `job.update` messages to the agent via mesh routing.
pub(crate) async fn handle_job_update(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
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
pub(crate) async fn handle_job_cleanup(envelope: &EventEnvelope) {
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed cleanup payload: {e}");
            return;
        }
    };

    let job_id = payload["job_id"].as_str().unwrap_or(&envelope.entity_id);
    let payload_slug = payload["slug"].as_str().unwrap_or("");

    // Resolve slug from job projection if not in payload
    let slug = if !payload_slug.is_empty() {
        payload_slug.to_string()
    } else if let Ok(jobs) = runtime::projections::job::load_jobs() {
        jobs.values()
            .find(|j| j.job_id == job_id || j.job_id == envelope.entity_id)
            .map(|j| j.slug.clone())
            .unwrap_or_else(|| job_id.to_string())
    } else {
        job_id.to_string()
    };

    let tmux_session = format!("job-{slug}");
    let worktree_path = worktrees_dir().join(&slug);
    let branch = format!("job/{slug}");

    tracing::info!(
        slug,
        job_id,
        tmux_session = %tmux_session,
        worktree = %worktree_path.display(),
        branch = %branch,
        "job-dispatch: cleaning up job resources"
    );

    if let Err(e) = kill_tmux_session(&tmux_session) {
        tracing::warn!(slug, tmux_session = %tmux_session, "job-dispatch: tmux cleanup failed: {e}");
    }
    if let Err(e) = remove_worktree(&worktree_path) {
        tracing::warn!(slug, worktree = %worktree_path.display(), "job-dispatch: worktree cleanup failed: {e}");
    }
    if let Err(e) = delete_branch(&branch) {
        tracing::warn!(slug, branch = %branch, "job-dispatch: branch cleanup failed: {e}");
    }

    tracing::info!(slug, job_id, "job-dispatch: cleanup complete");
}

/// After `advance_pipeline` returns false on a `job.step_completed` event, emit
/// `job.completed` if all pipeline steps are now in `completed_steps`.
pub(crate) async fn finalize_pipeline_if_done(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
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

    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(_) => return,
    };
    let job = match jobs.values().find(|j| j.job_id == job_id) {
        Some(j) => j.clone(),
        None => return,
    };

    // Only emit job.completed when all pipeline steps are recorded as done.
    let all_done = match job.steps {
        Some(ref steps) => steps.steps.iter().all(|s| job.completed_steps.contains(&s.id)),
        None => return, // No pipeline — liveness handler owns job.completed
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
    if let Ok(store) = SqliteEventStore::new() {
        let _ = EventStore::append(&store, &env);
    }
    let ctx = EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, workspace_id: None, session_id: None };
    let _ = kernel.lock().await.route(env, &ctx).await;
}

/// Execute a merge step: git merge the job branch, emit `job.merged` on success.
/// Returns `true` if the merge succeeded.
pub(crate) async fn execute_merge_step(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    job_id: &str,
    slug: &str,
    branch: &str,
) -> bool {
    let merge_result = std::process::Command::new("git")
        .args(["merge", branch, "--no-edit"])
        .current_dir(super::helpers::repo_dir())
        .output();

    match merge_result {
        Ok(output) if output.status.success() => {
            tracing::info!(slug, branch, "pipeline: merge successful");
            let merged_payload = runtime::events::job::JobMergedPayload {
                job_id: job_id.to_string(),
                slug: slug.to_string(),
            };
            let merged_envelope = EventEnvelope::new(
                event_types::JOB_MERGED,
                job_id,
                &merged_payload,
            );
            if let Ok(envelope) = merged_envelope {
                if let Ok(store) = SqliteEventStore::new() {
                    let _ = EventStore::append(&store, &envelope);
                }
                let ctx = EmitContext {
                    caller_kind: CallerKind::Runtime,
                    skill_id: None,
                    workspace_id: None,
                    session_id: None,
                };
                let _ = kernel.lock().await.route(envelope, &ctx).await;
            }
            true
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!(slug, branch, "pipeline: merge failed: {stderr}");
            false
        }
        Err(e) => {
            tracing::error!(slug, "pipeline: git merge command failed: {e}");
            false
        }
    }
}

