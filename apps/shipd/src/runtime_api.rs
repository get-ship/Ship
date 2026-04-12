//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/runtime` in lib.rs.
//! All handlers are read-only and use the existing `ApiState`.

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

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

// ---- Query params ----

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SessionsQuery {
    pub workspace_id: Option<String>,
}

// ---- Handlers ----

/// GET /api/runtime/workspaces
///
/// Returns only active workspaces. Archived/dead workspaces are excluded
/// from the API surface — the reconciler handles cleanup internally.
#[utoipa::path(
    get,
    path = "/api/runtime/workspaces",
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn list_workspaces(State(state): State<ApiState>) -> impl IntoResponse {
    // Source of truth: git worktrees. DB rows overlay metadata when present.
    let mut workspaces: Vec<serde_json::Value> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 1. Git worktrees (one per branch checked out).
    if let Some(wt) = git_worktrees() {
        for (branch, path) in wt {
            if seen_names.contains(&branch) { continue; }
            seen_names.insert(branch.clone());
            workspaces.push(serde_json::json!({
                "id": branch.replace('/', "-"),
                "name": branch,
                "path": path,
                "active_agent": null,
                "created_at": chrono::Utc::now().to_rfc3339(),
            }));
        }
    }

    // 2. DB-tracked workspaces not already listed (defensive — any rows remaining
    //    from a prior run that still correspond to something meaningful).
    if let Ok(db_ws) = runtime::workspace::list_workspaces(state.ship_dir()) {
        for w in db_ws {
            if seen_names.contains(&w.name) { continue; }
            seen_names.insert(w.name.clone());
            workspaces.push(serde_json::to_value(&w).unwrap_or(serde_json::json!({})));
        }
    }

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "workspaces": workspaces }),
        }),
    )
}

/// Parse `git worktree list --porcelain` from the active project root.
/// Returns (branch, path) pairs, skipping detached and bare entries.
fn git_worktrees() -> Option<Vec<(String, String)>> {
    let project_ship = runtime::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() { return None; }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut out = Vec::new();
    for block in text.split("\n\n") {
        let mut path: Option<String> = None;
        let mut branch: Option<String> = None;
        let mut skip = false;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p.trim().to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                branch = Some(b.trim().to_string());
            } else if line == "bare" || line == "detached" {
                skip = true;
            }
        }
        if skip { continue; }
        if let (Some(p), Some(b)) = (path, branch) {
            out.push((b, p));
        }
    }
    Some(out)
}

/// GET /api/runtime/sessions[?workspace_id=<id>]
#[utoipa::path(
    get,
    path = "/api/runtime/sessions",
    params(SessionsQuery),
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn list_sessions(
    State(_state): State<ApiState>,
    Query(params): Query<SessionsQuery>,
) -> impl IntoResponse {
    let workspace_id = params.workspace_id.as_deref();
    match runtime::db::session::list_workspace_sessions_db(workspace_id, 100) {
        Ok(sessions) => {
            let data: Vec<serde_json::Value> = sessions
                .into_iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "workspace_id": s.workspace_id,
                        "workspace_branch": s.workspace_branch,
                        "status": s.status,
                        "started_at": s.started_at,
                        "ended_at": s.ended_at,
                        "agent_id": s.agent_id,
                        "primary_provider": s.primary_provider,
                        "goal": s.goal,
                        "summary": s.summary,
                        "tool_call_count": s.tool_call_count,
                        "created_at": s.created_at,
                        "updated_at": s.updated_at,
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "sessions": data }),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}

/// GET /api/runtime/agents
#[utoipa::path(
    get,
    path = "/api/runtime/agents",
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn list_agents(State(_state): State<ApiState>) -> impl IntoResponse {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(d) => d,
        Err(_) => {
            return (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!({ "agents": [] }) }));
        }
    };

    let agents_dir = project_dir.join("agents");
    let mut agents = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".jsonc") && !name.ends_with(".toml") { continue; }
            let id = name.trim_end_matches(".jsonc").trim_end_matches(".toml").to_string();
            let label = std::fs::read_to_string(entry.path()).ok()
                .and_then(|content| {
                    let cleaned: String = content.lines()
                        .map(|l| if l.trim_start().starts_with("//") { "" } else { l })
                        .collect::<Vec<_>>().join("\n");
                    serde_json::from_str::<serde_json::Value>(&cleaned).ok()
                })
                .and_then(|v| v["name"].as_str().map(str::to_string))
                .unwrap_or_else(|| id.clone());
            agents.push(serde_json::json!({ "agent_id": id, "label": label }));
        }
    }

    agents.sort_by(|a, b| a["agent_id"].as_str().cmp(&b["agent_id"].as_str()));
    (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!({ "agents": agents }) }))
}

// ---- Job request types ----

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateJobRequest {
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
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
}

/// GET /api/runtime/jobs
#[utoipa::path(
    get,
    path = "/api/runtime/jobs",
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn list_jobs(State(_state): State<ApiState>) -> impl IntoResponse {
    match runtime::projections::job::load_jobs() {
        Ok(jobs) => {
            let data: Vec<_> = jobs.into_values().collect();
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "jobs": data }),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
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

/// GET /api/runtime/jobs/{job_id}
#[utoipa::path(
    get,
    path = "/api/runtime/jobs/{job_id}",
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn get_job_detail(
    State(_state): State<ApiState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
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
            );
        }
    };

    let step_observations = runtime::db::events::query_events_by_entity_and_type(
        &job_id, "step.observed",
    )
    .unwrap_or_default()
    .into_iter()
    .filter_map(|env| {
        serde_json::from_str::<runtime::events::job::StepObservedPayload>(&env.payload_json).ok()
    })
    .collect::<Vec<_>>();

    let recent_events = runtime::db::events::query_recent_events_for_entity(&job_id, 50)
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
}

/// POST /api/runtime/jobs — emit job.created event through the kernel.
///
/// The kernel persists the event and routes it to subscribers.
/// The job_dispatch service handles all heavy lifting (worktree creation,
/// dependency resolution, agent spawning).
#[utoipa::path(
    post,
    path = "/api/runtime/jobs",
    request_body = CreateJobRequest,
    responses((status = 200, body = MeshResponse)),
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
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug: req.slug,
        agent: req.agent,
        branch: req.branch,
        spec_path: req.spec_path,
        plan_id: None,
        model: None,
        provider: req.provider,
        depends_on: req.depends_on,
        pipeline: None,
        steps: None,
        pipeline_name: req.pipeline,
        vars: req.vars,
        spec_content: req.spec_content,
    };

    // Enrich with pipeline BEFORE persisting — the persisted event must include steps
    // so projections can track pipeline state.
    let payload = runtime::services::orchestrator::pipeline_loader::ensure_pipeline(payload);

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

// ---- Job end (hook-triggered) ----

#[derive(Deserialize, utoipa::ToSchema)]
pub struct JobEndRequest {
    pub slug: String,
}

/// POST /api/runtime/jobs/end — signal that a provider turn ended for a job.
///
/// Called by the session Stop hook when an agent finishes a response turn.
/// For multi-step pipeline jobs, this sends a TurnEnded signal to the step
/// monitor (which accumulates evidence before deciding to advance).
/// For single-step jobs, emits job.completed directly.
#[utoipa::path(
    post,
    path = "/api/runtime/jobs/end",
    request_body = JobEndRequest,
    responses((status = 200, body = MeshResponse)),
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

// ---- Agent control ----

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SteerAgentRequest {
    pub message: String,
}

/// POST /api/runtime/agents/{slug}/stop — stop a running agent by slug.
///
/// Uses the process registry and executor to terminate the agent process.
#[utoipa::path(
    post,
    path = "/api/runtime/agents/{slug}/stop",
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn stop_agent(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let handle = {
        let mut reg = dh.registry.lock().await;
        match reg.take(&slug) {
            Some(h) => h,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
                );
            }
        }
    };

    match dh.executor.stop(&handle).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("stopped: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to stop agent: {e}")) }),
        ),
    }
}

/// POST /api/runtime/agents/{slug}/steer — inject a message into a running agent.
#[utoipa::path(
    post,
    path = "/api/runtime/agents/{slug}/steer",
    request_body = SteerAgentRequest,
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn steer_agent(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(slug): Path<String>,
    Json(req): Json<SteerAgentRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let reg = dh.registry.lock().await;
    let handle = match reg.get(&slug) {
        Some(h) => h,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
            );
        }
    };

    match dh.executor.send(handle, &req.message).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("steered: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to steer agent: {e}")) }),
        ),
    }
}

/// POST /api/runtime/rebuild-projections — rebuild all projections from events.
#[utoipa::path(
    post,
    path = "/api/runtime/rebuild-projections",
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn rebuild_projections(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::AdminDaemon) {
        return e;
    }
    match runtime::db::rebuild::rebuild_projections(state.ship_dir()) {
        Ok(report) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({
                    "events_replayed": report.events_replayed,
                    "projections_rebuilt": report.projections_rebuilt,
                    "duration_ms": report.duration_ms,
                }),
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

/// Stops an actor when dropped — used to clean up the SSE stream actor.
pub(crate) struct ActorGuard {
    pub(crate) actor_id: String,
    pub(crate) kernel: crate::rest_api::SharedKernel,
}

impl Drop for ActorGuard {
    fn drop(&mut self) {
        let actor_id = self.actor_id.clone();
        let kernel = self.kernel.clone();
        tokio::spawn(async move {
            let _ = kernel.lock().await.stop_actor(&actor_id);
        });
    }
}

/// GET /api/runtime/events — SSE stream of all kernel events.
#[utoipa::path(
    get,
    path = "/api/runtime/events",
    responses((status = 200, description = "SSE stream of kernel events")),
    tag = "runtime"
)]
pub async fn event_stream(
    State(state): State<ApiState>,
) -> Result<Sse<impl futures::stream::Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
    let actor_id = format!(
        "studio-events-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["".to_string()],
    };

    let mailbox = {
        let mut kernel = state.kernel().lock().await;
        kernel.spawn_actor(&actor_id, config).map(|(_store, mb)| mb).map_err(|e| {
            tracing::warn!("event_stream: actor spawn failed: {e}");
            let status = match &e {
                runtime::KernelError::ActorAlreadyExists { .. } => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string())
        })?
    };

    let guard = ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    let stream = async_stream::stream! {
        let _guard = guard;
        let mut mb = mailbox;
        while let Some(event) = mb.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            yield Ok::<_, std::convert::Infallible>(
                Event::default().event("ship.event").data(json)
            );
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Extension;
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn make_api_state(dir: &TempDir) -> ApiState {
        let ship_dir = dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        runtime::db::ensure_db_at(&ship_dir).unwrap();
        let kernel = runtime::events::init_kernel_router(ship_dir.clone()).unwrap();
        Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
    }

    async fn collect_body(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_ok_with_array() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = list_workspaces(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["workspaces"].is_array(), "expected workspaces array");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_ok_no_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let query = Query(SessionsQuery { workspace_id: None });
        let response = list_sessions(State(state), query).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["sessions"].is_array(), "expected sessions array");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_ok_with_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let query = Query(SessionsQuery {
            workspace_id: Some("nonexistent-workspace".to_string()),
        });
        let response = list_sessions(State(state), query).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["sessions"].is_array());
        assert_eq!(val["data"]["sessions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_agents_returns_ok_empty_registry() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = list_agents(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["agents"].is_array());
        assert_eq!(val["data"]["agents"].as_array().unwrap().len(), 0);
    }

    fn create_test_workspace(state: &ApiState, branch: &str) {
        runtime::workspace::create_workspace(
            state.ship_dir(),
            runtime::workspace::CreateWorkspaceRequest {
                branch: branch.into(),
                ..Default::default()
            },
        )
        .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_created_workspace() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/rt-ws-test");

        let response = list_workspaces(State(state)).await.into_response();
        let val = collect_body(response).await;
        let workspaces = val["data"]["workspaces"].as_array().unwrap();
        assert!(
            workspaces.iter().any(|w| w["name"] == "feature/rt-ws-test"),
            "created workspace should appear in list"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_agents_returns_populated_registry() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);

        {
            let mut reg = state.mesh_registry().write().await;
            reg.insert(
                "agent.test".into(),
                runtime::services::mesh::MeshEntry {
                    agent_id: "agent.test".into(),
                    label: "Test Agent".into(),
                    capabilities: vec!["rust".into()],
                    registered_at: chrono::Utc::now(),
                    status: runtime::services::mesh::AgentStatus::Active,
                },
            );
        }

        let response = list_agents(State(state)).await.into_response();
        let val = collect_body(response).await;
        let agents = val["data"]["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0]["agent_id"], "agent.test");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_created_session() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/rt-sess-test");

        let ws = runtime::workspace::get_workspace(state.ship_dir(), "feature/rt-sess-test")
            .unwrap()
            .unwrap();
        let payload = runtime::events::types::SessionStarted {
            goal: Some("test".into()),
            workspace_id: ws.id.clone(),
            workspace_branch: ws.name.clone(),
            ..Default::default()
        };
        runtime::db::session_events::insert_session_with_started_event(
            "rt-test-session",
            &ws.id,
            &payload,
        )
        .unwrap();

        let query = Query(SessionsQuery { workspace_id: None });
        let response = list_sessions(State(state), query).await.into_response();
        let val = collect_body(response).await;
        let sessions = val["data"]["sessions"].as_array().unwrap();
        assert!(sessions.len() >= 1, "created session should appear");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_filters_by_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/ws-filter-a");
        create_test_workspace(&state, "feature/ws-filter-b");

        let ws_a = runtime::workspace::get_workspace(state.ship_dir(), "feature/ws-filter-a")
            .unwrap()
            .unwrap();
        let payload = runtime::events::types::SessionStarted {
            goal: Some("a".into()),
            workspace_id: ws_a.id.clone(),
            workspace_branch: ws_a.name.clone(),
            ..Default::default()
        };
        runtime::db::session_events::insert_session_with_started_event(
            "rt-filter-sess",
            &ws_a.id,
            &payload,
        )
        .unwrap();

        // Filter by workspace B — should return empty.
        let ws_b = runtime::workspace::get_workspace(state.ship_dir(), "feature/ws-filter-b")
            .unwrap()
            .unwrap();
        let query = Query(SessionsQuery {
            workspace_id: Some(ws_b.id),
        });
        let response = list_sessions(State(state), query).await.into_response();
        let val = collect_body(response).await;
        let sessions = val["data"]["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 0, "no sessions for workspace B");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_multiple() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/multi-a");
        create_test_workspace(&state, "feature/multi-b");

        let response = list_workspaces(State(state)).await.into_response();
        let val = collect_body(response).await;
        let workspaces = val["data"]["workspaces"].as_array().unwrap();
        assert!(workspaces.len() >= 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_all() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/ws-a");
        create_test_workspace(&state, "feature/ws-b");

        let response = list_workspaces(State(state)).await.into_response();
        let val = collect_body(response).await;
        let workspaces = val["data"]["workspaces"].as_array().unwrap();

        assert!(
            workspaces.iter().any(|w| w["name"] == "feature/ws-a"),
            "workspace a should appear"
        );
        assert!(
            workspaces.iter().any(|w| w["name"] == "feature/ws-b"),
            "workspace b should appear"
        );
    }

    fn full_auth() -> AuthContext {
        AuthContext {
            caller_kind: runtime::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn stop_agent_no_dispatch_handle_returns_503() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = stop_agent(State(state), Extension(full_auth()), Path("test-slug".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_job_detail_not_found() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = get_job_detail(State(state), Path("nonexistent-job-id".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(false));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_job_detail_returns_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);

        // Emit a job.created event so load_jobs() finds it.
        let job_id = runtime::gen_ulid();
        let payload = runtime::events::job::JobCreatedPayload {
            job_id: job_id.clone(),
            slug: "test-detail".into(),
            agent: "agent-1".into(),
            branch: "feature/detail".into(),
            spec_path: ".artifacts/job-spec.md".into(),
            plan_id: None,
            model: None,
            provider: None,
            depends_on: None,
            pipeline: None,
            steps: None,
            pipeline_name: None,
            vars: None,
            spec_content: None,
        };
        let envelope = runtime::events::EventEnvelope::new(
            runtime::events::job::event_types::JOB_CREATED,
            &job_id,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&envelope).await.unwrap();

        let response = get_job_detail(State(state), Path(job_id.clone()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert_eq!(val["data"]["job"]["job_id"], job_id);
        assert_eq!(val["data"]["job"]["slug"], "test-detail");
        assert!(val["data"]["step_observations"].is_array());
        assert!(val["data"]["recent_events"].is_array());
        // The job.created event should appear in recent_events
        assert!(val["data"]["recent_events"].as_array().unwrap().len() >= 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn steer_agent_no_dispatch_handle_returns_503() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let req = SteerAgentRequest { message: "hello".to_string() };
        let response = steer_agent(State(state), Extension(full_auth()), Path("test-slug".to_string()), Json(req))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
