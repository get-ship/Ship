//! Job list, detail, cancel, and end-turn handlers.
//!
//! Pure read helpers (`list_processes_at`, `load_recent_events_by_type`,
//! `drift_active_for_slug`, `truncate`) have moved to
//! `ship_substrate::runtime_reads`. This module re-exports them for
//! backwards-compat with `runtime_rpc.rs` and `daemon.rs` callers, then
//! owns the pipeline-runtime-dependent `annotate_job` and the HTTP handlers.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types;
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::{ApiState, MeshResponse};

// Re-exports so existing callers resolve without source changes.
pub use ship_substrate::runtime_reads::{
    drift_active_for_slug, list_processes_at, load_recent_events_by_type, truncate,
};


/// GET /api/v1/runtime/jobs
///
/// Returns each `PipelineRun` with two operator-visibility annotations:
///
/// - `process_state`: `"alive"` when the latest process projection for the
///   slug is still Running, `"missing"` when the most recent supervisor
///   sweep flagged drift (no_process) without a coherence_restored after
///   it, `"terminal"` for jobs in a terminal status, `"none"` otherwise.
/// - `notes`: optional truncated error / blocker for the row's NOTES column.
pub async fn list_jobs(State(state): State<ApiState>) -> impl IntoResponse {
    let jobs = match ship_pipeline_runtime::projections::pipeline_run::load_pipeline_runs_at(state.ship_dir()) {
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
///
/// Kept in shipd (not ship-substrate) because it depends on
/// `ship_pipeline_runtime::projections::pipeline_run::PipelineRun`, which
/// ship-substrate cannot import.
pub(crate) fn annotate_job(
    job: &ship_pipeline_runtime::projections::pipeline_run::PipelineRun,
    processes: &[ship_substrate::projections::process::ProcessRecord],
    supervisor_events: &[ship_substrate::events::EventEnvelope],
) -> serde_json::Value {
    use ship_pipeline_runtime::projections::pipeline_run::PipelineRunStatus;
    use ship_substrate::projections::process::ProcessStatus;

    let terminal = matches!(
        job.status,
        PipelineRunStatus::Failed | PipelineRunStatus::Merged | PipelineRunStatus::Cancelled | PipelineRunStatus::Completed
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

// ---- Job detail response types ----

#[derive(serde::Serialize)]
pub struct PipelineRunDetailResponse {
    job: ship_pipeline_runtime::projections::pipeline_run::PipelineRun,
    step_observations: Vec<ship_substrate::events::pipeline::PipelineStepObservedPayload>,
    recent_events: Vec<EventSummary>,
}

#[derive(serde::Serialize)]
pub struct EventSummary {
    event_type: String,
    created_at: String,
    payload_preview: String,
}

/// GET /api/v1/runtime/jobs/{pipeline_run_id}
pub async fn get_job_detail(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(pipeline_run_id): Path<String>,
) -> axum::response::Response {
    let job = match ship_pipeline_runtime::projections::pipeline_run::load_pipeline_run_by_id_at(state.ship_dir(), &pipeline_run_id) {
        Ok(Some(j)) => j,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("job '{pipeline_run_id}' not found")),
                }),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            ).into_response();
        }
    };

    // pipeline.step.observed is Runtime-scoped → read runtime.db.
    let runtime_db = ship_substrate::db::runtime_db_path(state.ship_dir());
    let step_observations = ship_substrate::db::events::query_events_by_entity_and_type_at(
        &runtime_db, &pipeline_run_id, "pipeline.step.observed",
    )
    .unwrap_or_default()
    .into_iter()
    .filter_map(|env| {
        serde_json::from_str::<ship_substrate::events::pipeline::PipelineStepObservedPayload>(&env.payload_json).ok()
    })
    .collect::<Vec<_>>();

    let recent_events = ship_substrate::db::events::query_recent_events_for_entity_at(&runtime_db, &pipeline_run_id, 50)
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

    let detail = PipelineRunDetailResponse { job, step_observations, recent_events };
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!(detail),
        }),
    )
        .into_response()
}


// ---- Cancel ----

#[derive(Deserialize)]
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
///
/// Gate: cancel is a write — preserved here in the route; it must not be
/// collapsed into the pure read surface in ship-substrate.
pub async fn cancel_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(slug): Path<String>,
    Json(req): Json<CancelJobRequest>,
) -> impl IntoResponse {
    // ship.agents.write — cancel is a destructive control-plane write.
    if let Err(e) = require_scope(&auth, Scope::ManageRuns) {
        return e;
    }

    // Pipeline runs live in the jobs projection; bare agent sessions
    // live in the agent_sessions projection. The agent-primitive
    // inversion made single-agent dispatch emit only an
    // `agent.session.*` lifecycle (no jobs row), so cancel must resolve
    // against both projections and emit the matching cancelled event.
    // Jobs win the lookup (a composed pipeline step has a `::`-suffixed
    // session slug that operators don't cancel directly).
    let job = match ship_pipeline_runtime::projections::pipeline_run::load_pipeline_run_by_slug_at(state.ship_dir(), &slug) {
        Ok(Some(j)) => j,
        Ok(None) => {
            return cancel_agent_session(&state, &slug, req).await;
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    use ship_pipeline_runtime::projections::pipeline_run::PipelineRunStatus;
    if matches!(
        job.status,
        PipelineRunStatus::Failed | PipelineRunStatus::Merged | PipelineRunStatus::Cancelled | PipelineRunStatus::Completed
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

    let payload = ship_substrate::events::pipeline::PipelineRunCancelledPayload {
        pipeline_run_id: job.pipeline_run_id.clone(),
        slug: slug.clone(),
        reason: req.reason,
        requested_by: "operator".to_string(),
    };

    let envelope = match EventEnvelope::new(
        event_types::PIPELINE_RUN_CANCELLED,
        &job.pipeline_run_id,
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

/// Cancel a bare agent session (no jobs row). Emits
/// `agent.session.cancelled`, which the orchestrator router turns into a
/// process kill + `process.killed`. Idempotent: 409 when already
/// terminal, 404 when no session exists for the slug.
async fn cancel_agent_session(
    state: &ApiState,
    slug: &str,
    req: CancelJobRequest,
) -> (StatusCode, Json<MeshResponse>) {
    use agent_def::canonical_session::{SessionCancelled, event_types as canonical_event_types};
    use ship_agents::services::sessions::load_canonical_session_by_slug_at;

    let session = match load_canonical_session_by_slug_at(state.ship_dir(), slug) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!(
                        "no job or agent session with slug '{slug}'"
                    )),
                }),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    if session.status.is_terminal() {
        return (
            StatusCode::CONFLICT,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!({
                    "slug": slug,
                    "status": format!("{:?}", session.status).to_lowercase(),
                }),
            }),
        );
    }

    let payload = SessionCancelled {
        session_id: session.session_id.clone(),
        reason: Some(req.reason),
    };
    let envelope = match EventEnvelope::new(
        canonical_event_types::SESSION_CANCELLED,
        &session.session_id,
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
                data: serde_json::json!({ "slug": slug, "status": "cancelling" }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        ),
    }
}

// ---- Job end (hook-triggered) ----

#[derive(Deserialize)]
pub struct TurnEndRequest {
    pub slug: String,
}

/// POST /api/v1/runtime/jobs/end — signal that a harness turn ended for a job.
///
/// Thin HTTP transport adapter — delegates to
/// `ship_pipeline_runtime::run_core::end_turn_core`.
pub async fn end_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<TurnEndRequest>,
) -> impl IntoResponse {
    // ship.runtime.write (PipelinesService.EndTurn) — turn signals advance pipeline state.
    if let Err(e) = require_scope(&auth, Scope::ManageRuns) {
        return e;
    }
    use ship_pipeline_runtime::run_core::{EndTurnCoreError, EndTurnRequest, end_turn_core};
    use crate::pipeline_ports::{PipelineDispatchAdapter, TurnSignalAdapter};

    let dispatch_port = PipelineDispatchAdapter::new(state.clone());
    let signal_port = TurnSignalAdapter::new(state);
    match end_turn_core(&dispatch_port, &signal_port, EndTurnRequest { slug: req.slug }).await {
        Ok(msg) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(msg) }),
        ),
        Err(EndTurnCoreError::NotFound(m)) => (
            StatusCode::NOT_FOUND,
            Json(MeshResponse { ok: false, data: serde_json::json!(m) }),
        ),
        Err(EndTurnCoreError::BadRequest(m)) => (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse { ok: false, data: serde_json::json!(m) }),
        ),
        Err(EndTurnCoreError::Internal(m)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(m) }),
        ),
    }
}
