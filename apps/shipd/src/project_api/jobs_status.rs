//! POST /api/v1/jobs/{job_id}/status — agent-driven lifecycle updates.
//!
//! MCP's `update_job` tool POSTs here instead of writing to the event store
//! directly. The daemon builds the appropriate `job.*` envelope, persists it
//! through the kernel, and lets the projection rebuild advance.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{Scope, require_scope};
use crate::project_api::{err_response, ok_response};
use crate::routes::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "status": "dispatched",
    "worktree": "/tmp/ws-write-readme"
}))]
pub struct JobStatusReq {
    /// One of: dispatched, gate_requested, gate_passed, gate_failed, blocked,
    /// completed, merged, failed.
    pub status: String,
    pub worktree: Option<String>,
    pub pid: Option<u32>,
    pub gate_agent: Option<String>,
    pub blocker: Option<String>,
    pub error: Option<String>,
}

#[utoipa::path(
    operation_id = "updateJobStatus",
    post,
    path = "/api/v1/jobs/{job_id}/status",
    request_body = JobStatusReq,
    params(("job_id" = String, Path, description = "Job ULID")),
    responses(
        (status = 200, description = "Job status event emitted", body = MeshResponse),
        (status = 400, description = "Invalid status or missing field", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn update_job_status(
    State(state): State<ApiState>,
    auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Path(job_id): Path<String>,
    Json(body): Json<JobStatusReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    if let Some(axum::Extension(ref ac)) = auth {
        require_scope(ac, Scope::EmitEvents)?;
    }

    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::job::{
        JobBlockedPayload, JobCompletedPayload, JobDispatchedPayload, JobFailedPayload,
        JobGateFailedPayload, JobGatePassedPayload, JobGateRequestedPayload, JobMergedPayload,
        event_types,
    };

    let envelope_result = match body.status.as_str() {
        "dispatched" => {
            let worktree = body.worktree.ok_or_else(|| {
                err_response(StatusCode::BAD_REQUEST, "worktree is required for status 'dispatched'")
            })?;
            EventEnvelope::new(
                event_types::JOB_DISPATCHED,
                &job_id,
                &JobDispatchedPayload {
                    job_id: job_id.clone(),
                    worktree,
                    pid: body.pid,
                    provider_meta: None,
                },
            )
        }
        "gate_requested" => {
            let gate_agent = body.gate_agent.ok_or_else(|| {
                err_response(StatusCode::BAD_REQUEST, "gate_agent is required for status 'gate_requested'")
            })?;
            EventEnvelope::new(
                event_types::JOB_GATE_REQUESTED,
                &job_id,
                &JobGateRequestedPayload {
                    job_id: job_id.clone(),
                    gate_agent,
                },
            )
        }
        "gate_passed" => EventEnvelope::new(
            event_types::JOB_GATE_PASSED,
            &job_id,
            &JobGatePassedPayload { job_id: job_id.clone() },
        ),
        "gate_failed" => {
            let reason = body.error.ok_or_else(|| {
                err_response(StatusCode::BAD_REQUEST, "error is required for status 'gate_failed'")
            })?;
            EventEnvelope::new(
                event_types::JOB_GATE_FAILED,
                &job_id,
                &JobGateFailedPayload { job_id: job_id.clone(), reason },
            )
        }
        "blocked" => {
            let blocker = body.blocker.ok_or_else(|| {
                err_response(StatusCode::BAD_REQUEST, "blocker is required for status 'blocked'")
            })?;
            EventEnvelope::new(
                event_types::JOB_BLOCKED,
                &job_id,
                &JobBlockedPayload {
                    job_id: job_id.clone(),
                    blocker,
                    needs_human: true,
                },
            )
        }
        "completed" => {
            let slug = resolve_slug(state.ship_dir(), &job_id);
            EventEnvelope::new(
                event_types::JOB_COMPLETED,
                &job_id,
                &JobCompletedPayload { job_id: job_id.clone(), slug },
            )
        }
        "merged" => {
            let slug = resolve_slug(state.ship_dir(), &job_id);
            EventEnvelope::new(
                event_types::JOB_MERGED,
                &job_id,
                &JobMergedPayload { job_id: job_id.clone(), slug },
            )
        }
        "failed" => {
            let error = body.error.ok_or_else(|| {
                err_response(StatusCode::BAD_REQUEST, "error is required for status 'failed'")
            })?;
            EventEnvelope::new(
                event_types::JOB_FAILED,
                &job_id,
                &JobFailedPayload { job_id: job_id.clone(), error },
            )
        }
        other => {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                &format!(
                    "unknown status '{other}'. Valid: dispatched, gate_requested, gate_passed, \
                    gate_failed, blocked, completed, merged, failed"
                ),
            ));
        }
    };

    let envelope = envelope_result
        .map_err(|e| err_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    state
        .persist_and_route(&envelope)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(ok_response(json!({ "job_id": job_id, "status": body.status })))
}

fn resolve_slug(ship_dir: &std::path::Path, job_id: &str) -> String {
    ship_platform::projections::job::load_jobs_at(ship_dir)
        .ok()
        .and_then(|jobs| jobs.get(job_id).map(|r| r.slug.clone()))
        .unwrap_or_default()
}

// ---- Step-done declaration ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "summary": "Compiled and tests pass.",
    "outputs": { "artifact": "build.tar.gz" }
}))]
pub struct StepDoneReq {
    pub summary: Option<String>,
    /// Map of `produces.name → artifact_id` for step outputs.
    #[serde(default)]
    pub outputs: std::collections::BTreeMap<String, String>,
}

/// POST /api/v1/jobs/{slug}/step-done — agent declares the current pipeline step
/// complete. Daemon resolves the active step from the projection, builds a
/// `step.done_declared` envelope, and routes it through the kernel.
#[utoipa::path(
    operation_id = "jobStepDone",
    post,
    path = "/api/v1/jobs/{slug}/step-done",
    request_body = StepDoneReq,
    params(("slug" = String, Path, description = "Job slug")),
    responses(
        (status = 200, description = "Step-done event emitted", body = MeshResponse),
        (status = 400, description = "Invalid request", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "Job or active step not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn job_step_done(
    State(state): State<ApiState>,
    auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Path(slug): Path<String>,
    Json(body): Json<StepDoneReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    if let Some(axum::Extension(ref ac)) = auth {
        require_scope(ac, Scope::EmitEvents)?;
    }

    let jobs = ship_platform::projections::job::load_jobs_at(state.ship_dir())
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let job = jobs
        .values()
        .find(|j| j.slug == slug)
        .cloned()
        .ok_or_else(|| {
            err_response(StatusCode::NOT_FOUND, &format!("no job with slug '{slug}'"))
        })?;
    let step_id = job.current_step.clone().ok_or_else(|| {
        err_response(StatusCode::BAD_REQUEST, &format!("no active step for job '{slug}'"))
    })?;

    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::job::{StepDoneDeclaredPayload, event_types};

    let payload = StepDoneDeclaredPayload {
        job_id: job.job_id.clone(),
        slug: slug.clone(),
        step_id: step_id.clone(),
        summary: body.summary,
        outputs: body.outputs,
    };
    let envelope = EventEnvelope::new(event_types::STEP_DONE_DECLARED, &job.job_id, &payload)
        .map_err(|e| err_response(StatusCode::BAD_REQUEST, &e.to_string()))?;
    state
        .persist_and_route(&envelope)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(ok_response(json!({ "slug": slug, "step_id": step_id })))
}

#[cfg(test)]
#[path = "jobs_status_tests.rs"]
mod tests;
