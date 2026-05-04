//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use runtime::events::{ActorConfig, EventEnvelope};
use runtime::events::job::{JobCreatedPayload, event_types};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

// ---- Job request types ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "slug": "write-readme",
    "agent": "writer",
    "branch": "job/write-readme",
    "spec_path": ".ship/specs/write-readme.md",
    "source_spec_path": "/home/me/project/.ship/specs/write-readme.md",
    "spec_content": "# Write README\n\nSummarize the project.\n",
    "vars": { "topic": "ship" },
    "headless": false
}))]
pub struct CreateJobRequest {
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    /// Operator-facing path to the spec file (absolute when resolvable).
    /// Plumbed into the JobRecord so `ship jobs` can distinguish which
    /// spec each job was dispatched from. Distinct from `spec_path`
    /// (daemon-side worktree destination). Optional: `--spec-text`
    /// dispatches and legacy callers send `None`.
    #[serde(default)]
    pub source_spec_path: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub pipeline: Option<String>,
    pub provider: Option<String>,
    /// Inline spec content. Travels in the JobCreated event; the worktree
    /// adapter writes it to `spec_path` at dispatch time.
    pub spec_content: Option<String>,
    /// Pipeline variables for `{{ var }}` interpolation in step fields.
    /// Merged over pipeline-file defaults; CLI `--var` flags land here.
    #[serde(default)]
    pub vars: Option<std::collections::HashMap<String, String>>,
    /// When true, force the headless (ProcessExecutor) path. When false or
    /// absent, the default TmuxWindowExecutor (tmux-window) path is used.
    #[serde(default)]
    pub headless: bool,
}

/// GET /api/v1/runtime/jobs
///
/// Returns each `JobRecord` with two operator-visibility annotations:
///
/// - `process_state`: `"alive"` when the latest process projection for the
///   slug is still Running, `"missing"` when the most recent supervisor
///   sweep flagged drift (no_process) without a coherence_restored after
///   it, `"terminal"` for jobs in a terminal status, `"none"` otherwise.
/// - `notes`: optional truncated error / blocker for the row's NOTES column.
#[utoipa::path(
    operation_id = "listJobs",
    get,
    path = "/api/v1/jobs",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_jobs(State(state): State<ApiState>) -> impl IntoResponse {
    let jobs = match runtime::projections::job::load_jobs_at(state.ship_dir()) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(e.to_string()),
                }),
            );
        }
    };
    let processes = list_processes_at(state.ship_dir(), None).unwrap_or_default();
    let supervisor_events =
        load_recent_events_by_type(state.ship_dir(), &["supervisor.drift_detected", "supervisor.coherence_restored"], 500)
            .unwrap_or_default();

    let annotated: Vec<serde_json::Value> = jobs
        .values()
        .map(|j| annotate_job(j, &processes, &supervisor_events))
        .collect();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "jobs": annotated }),
        }),
    )
}

/// Annotate a job with `process_state` + `notes` for `ship jobs`.
pub(crate) fn annotate_job(
    job: &runtime::projections::job::JobRecord,
    processes: &[runtime::projections::process::ProcessRecord],
    supervisor_events: &[runtime::events::EventEnvelope],
) -> serde_json::Value {
    use runtime::projections::job::JobStatus;
    use runtime::projections::process::ProcessStatus;

    let terminal = matches!(
        job.status,
        JobStatus::Failed | JobStatus::Merged | JobStatus::Cancelled | JobStatus::Completed
    );

    // Most recent process for this slug, by spawned_at.
    let latest_proc = processes
        .iter()
        .filter(|p| p.slug == job.slug)
        .max_by_key(|p| p.spawned_at);

    let process_state = if terminal {
        "terminal"
    } else if matches!(latest_proc.map(|p| p.status), Some(ProcessStatus::Running)) {
        "alive"
    } else if drift_active_for_slug(&job.slug, supervisor_events) {
        "missing"
    } else {
        "none"
    };

    let notes = job
        .error
        .as_ref()
        .or(job.blocker.as_ref())
        .map(|s| truncate(s, 60));

    let mut obj = serde_json::to_value(job).unwrap_or(serde_json::json!({}));
    if let Some(map) = obj.as_object_mut() {
        map.insert("process_state".into(), serde_json::json!(process_state));
        map.insert("notes".into(), serde_json::json!(notes));
    }
    obj
}

/// True if the most recent supervisor.drift_detected for `slug` is newer
/// than any subsequent supervisor.coherence_restored — i.e. the drift has
/// not yet been resolved by a sweep action.
pub(crate) fn drift_active_for_slug(slug: &str, events: &[runtime::events::EventEnvelope]) -> bool {
    let mut latest_drift: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut latest_restored: Option<chrono::DateTime<chrono::Utc>> = None;
    for e in events {
        if e.entity_id != slug {
            continue;
        }
        match e.event_type.as_str() {
            "supervisor.drift_detected" => {
                latest_drift = Some(latest_drift.map_or(e.created_at, |d| d.max(e.created_at)));
            }
            "supervisor.coherence_restored" => {
                latest_restored =
                    Some(latest_restored.map_or(e.created_at, |d| d.max(e.created_at)));
            }
            _ => {}
        }
    }
    match (latest_drift, latest_restored) {
        (Some(d), Some(r)) => d > r,
        (Some(_), None) => true,
        _ => false,
    }
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        chars[..max.saturating_sub(1)].iter().collect::<String>() + "…"
    }
}

/// Project process events from an explicit ship_dir without going through
/// the global db_path. Mirrors `load_jobs_at` for jobs.
pub(crate) fn list_processes_at(
    ship_dir: &std::path::Path,
    slug: Option<&str>,
) -> anyhow::Result<Vec<runtime::projections::process::ProcessRecord>> {
    runtime::db::ensure_db_at(ship_dir)?;
    let events = runtime::db::events::list_recent_events_at(ship_dir, 50_000)?;
    let mut records = runtime::projections::process::project(&events);
    if let Some(s) = slug {
        records.retain(|r| r.slug == s);
    }
    Ok(records)
}

/// Filter recent events by event_type set, against an explicit ship_dir.
pub(crate) fn load_recent_events_by_type(
    ship_dir: &std::path::Path,
    types: &[&str],
    limit: usize,
) -> anyhow::Result<Vec<runtime::events::EventEnvelope>> {
    runtime::db::ensure_db_at(ship_dir)?;
    let events = runtime::db::events::list_recent_events_at(ship_dir, limit)?;
    Ok(events
        .into_iter()
        .filter(|e| types.contains(&e.event_type.as_str()))
        .collect())
}

// ---- Job detail response types ----

#[derive(serde::Serialize)]
pub struct JobDetailResponse {
    job: runtime::projections::job::JobRecord,
    step_observations: Vec<runtime::events::job::StepObservedPayload>,
    recent_events: Vec<EventSummary>,
}

#[derive(serde::Serialize)]
pub struct EventSummary {
    event_type: String,
    created_at: String,
    payload_preview: String,
}

/// GET /api/v1/runtime/jobs/{job_id}
#[utoipa::path(
    operation_id = "getJobDetail",
    get,
    path = "/api/v1/jobs/{job_id}",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "job not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn get_job_detail(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(job_id): Path<String>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::ReadJobs) {
        return e.into_response();
    }
    let jobs = match runtime::projections::job::load_jobs_at(state.ship_dir()) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            ).into_response();
        }
    };

    let job = match jobs.get(&job_id) {
        Some(j) => j.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("job '{job_id}' not found")),
                }),
            ).into_response();
        }
    };

    let platform_db = runtime::db::platform_db_path(state.ship_dir());
    let step_observations = runtime::db::events::query_events_by_entity_and_type_at(
        &platform_db, &job_id, "step.observed",
    )
    .unwrap_or_default()
    .into_iter()
    .filter_map(|env| {
        serde_json::from_str::<runtime::events::job::StepObservedPayload>(&env.payload_json).ok()
    })
    .collect::<Vec<_>>();

    let recent_events = runtime::db::events::query_recent_events_for_entity_at(&platform_db, &job_id, 50)
        .unwrap_or_default()
        .into_iter()
        .map(|env| {
            let preview = if env.payload_json.len() > 200 {
                let boundary = env.payload_json
                    .char_indices()
                    .take_while(|(i, _)| *i < 200)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &env.payload_json[..boundary])
            } else {
                env.payload_json.clone()
            };
            EventSummary {
                event_type: env.event_type.clone(),
                created_at: env.created_at.to_rfc3339(),
                payload_preview: preview,
            }
        })
        .collect::<Vec<_>>();

    let detail = JobDetailResponse { job, step_observations, recent_events };
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!(detail),
        }),
    )
        .into_response()
}

/// POST /api/v1/runtime/jobs — emit job.created event through the kernel.
///
/// The kernel persists the event and routes it to subscribers.
/// The job_dispatch service handles all heavy lifting (worktree creation,
/// dependency resolution, agent spawning).
#[utoipa::path(
    operation_id = "createJob",
    post,
    path = "/api/v1/jobs",
    request_body = CreateJobRequest,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 409, description = "job slug already exists", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn create_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::CreateJobs) {
        return e;
    }
    let job_id = runtime::gen_ulid();
    let baseline_sha = super::resolve_project_head_sha();
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug: req.slug,
        agent: req.agent,
        branch: req.branch,
        spec_path: req.spec_path,
        source_spec_path: req.source_spec_path,
        plan_id: None,
        model: None,
        provider: req.provider,
        depends_on: req.depends_on,
        pipeline: None,
        steps: None,
        pipeline_name: req.pipeline,
        vars: req.vars,
        spec_content: req.spec_content,
        step_specs: None,
        headless: req.headless,
        baseline_sha,
    };

    // Enrich with pipeline BEFORE persisting — the persisted event must include steps
    // so projections can track pipeline state.
    let payload = match runtime::services::orchestrator::pipeline_loader::ensure_pipeline(payload) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse { ok: false, data: serde_json::json!(e) }),
            );
        }
    };

    let envelope = match EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
        .map(|e| e.with_actor_id("studio"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(e.to_string()),
                }),
            );
        }
    };
    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "job_id": &job_id }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}

// ---- Cancel ----

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CancelJobRequest {
    #[serde(default = "default_cancel_reason")]
    pub reason: String,
}

pub(crate) fn default_cancel_reason() -> String {
    "operator-cancelled".to_string()
}

/// POST /api/v1/runtime/jobs/{slug}/cancel — emit `job.cancelled` for the
/// job with this slug. Idempotent: returns 409 with the current status
/// when the job is already in a terminal state, and emits no event.
#[utoipa::path(
    operation_id = "cancelJob",
    post,
    path = "/api/v1/jobs/{slug}/cancel",
    request_body = CancelJobRequest,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "job not found", body = crate::openapi_types::ErrorResponse),
        (status = 409, description = "job already finished", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn cancel_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(slug): Path<String>,
    Json(req): Json<CancelJobRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }

    let jobs = match runtime::projections::job::load_jobs_at(state.ship_dir()) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    let job = match jobs.values().find(|j| j.slug == slug) {
        Some(j) => j.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("no job with slug '{slug}'")),
                }),
            );
        }
    };

    use runtime::projections::job::JobStatus;
    if matches!(
        job.status,
        JobStatus::Failed | JobStatus::Merged | JobStatus::Cancelled | JobStatus::Completed
    ) {
        return (
            StatusCode::CONFLICT,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!({
                    "slug": &slug,
                    "status": format!("{:?}", job.status).to_lowercase(),
                }),
            }),
        );
    }

    let payload = runtime::events::job::JobCancelledPayload {
        job_id: job.job_id.clone(),
        slug: slug.clone(),
        reason: req.reason,
        requested_by: "operator".to_string(),
    };

    let envelope = match EventEnvelope::new(
        event_types::JOB_CANCELLED,
        &job.job_id,
        &payload,
    )
    .map(|e| e.with_actor_id("operator"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "slug": &slug, "status": "cancelling" }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        ),
    }
}

// ---- Job end (hook-triggered) ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "slug": "write-readme" }))]
pub struct JobEndRequest {
    pub slug: String,
}

/// POST /api/v1/runtime/jobs/end — signal that a provider turn ended for a job.
///
/// Called by the session Stop hook when an agent finishes a response turn.
/// For multi-step pipeline jobs, this sends a TurnEnded signal to the step
/// monitor (which accumulates evidence before deciding to advance).
/// For single-step jobs, emits job.completed directly.
#[utoipa::path(
    operation_id = "endJob",
    post,
    path = "/api/v1/jobs/end",
    request_body = JobEndRequest,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn end_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<JobEndRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }
    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    let job = match jobs.values()
        .filter(|j| j.slug == req.slug)
        .find(|j| j.status == runtime::projections::job::JobStatus::Dispatched)
    {
        Some(j) => j.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("no dispatched job with slug '{}'", req.slug)) }),
            );
        }
    };

    let is_pipeline = job.steps.as_ref().is_some_and(|s| s.steps.len() > 1);

    // Multi-step pipeline: send TurnEnded to the step monitor.
    // The monitor accumulates evidence and decides when to advance.
    if is_pipeline {
        if let Some(dh) = state.dispatch_handle() {
            let txs = dh.activity_txs.lock().await;
            if let Some(tx) = txs.get(&req.slug) {
                let _ = tx.send(runtime::services::orchestrator::StepActivity::TurnEnded);
                return (
                    StatusCode::OK,
                    Json(MeshResponse { ok: true, data: serde_json::json!(format!("turn ended for pipeline step '{}'", req.slug)) }),
                );
            }
        }
        // No active step monitor — fall through to job.completed for safety.
        tracing::warn!(slug = req.slug, "end_job: pipeline step has no active step monitor, falling through to job.completed");
    }

    // Single-step job or fallback: emit job.completed directly.
    let payload = runtime::events::job::JobCompletedPayload {
        job_id: job.job_id.clone(),
        slug: job.slug.clone(),
    };

    let envelope = match EventEnvelope::new(event_types::JOB_COMPLETED, &job.job_id, &payload)
        .map(|e| e.with_actor_id("hook"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("job '{}' completed", req.slug)) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        ),
    }
}
