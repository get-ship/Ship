//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/runtime` in lib.rs.
//! All handlers are read-only and use the existing `ApiState`.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use runtime::events::{ActorConfig, EventEnvelope};
use runtime::events::job::{JobCreatedPayload, event_types};
use serde::Deserialize;

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
    match runtime::workspace::list_workspaces(state.ship_dir()) {
        Ok(ws) => {
            let active: Vec<_> = ws
                .into_iter()
                .filter(|w| w.status == runtime::workspace::WorkspaceStatus::Active)
                .collect();
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "workspaces": active }),
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
pub async fn list_agents(State(state): State<ApiState>) -> impl IntoResponse {
    let registry = state.mesh_registry().read().await;
    let agents: Vec<serde_json::Value> = registry
        .values()
        .map(|e| {
            serde_json::json!({
                "agent_id": e.agent_id,
                "label": e.label,
                "capabilities": e.capabilities,
                "status": e.status,
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "agents": agents }),
        }),
    )
}

// ---- Views ----

/// GET /api/runtime/views — list available views from .ship/views/.
pub async fn list_views() -> impl IntoResponse {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(MeshResponse { ok: true, data: serde_json::json!({ "views": [] }) }),
            );
        }
    };

    let views_dir = project_dir.join("views");
    let mut views = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&views_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                let index = entry.path().join("index.html");
                if index.exists() {
                    views.push(serde_json::json!({ "name": name }));
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(MeshResponse { ok: true, data: serde_json::json!({ "views": views }) }),
    )
}

/// GET /api/runtime/view/{name} — serve a view's index.html from .ship/views/{name}/.
///
/// Reads the view HTML and inlines the ship-sdk script so the view can
/// communicate with Studio via postMessage.
pub async fn serve_view(
    Path(name): Path<String>,
) -> impl IntoResponse {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!("no project root found") }),
            );
        }
    };

    // Reject path traversal attempts
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse { ok: false, data: serde_json::json!("invalid view name") }),
        );
    }

    // get_project_dir returns the .ship/ directory
    let view_path = project_dir.join("views").join(&name).join("index.html");
    let sdk_path = project_dir.join("views/ship-sdk.js");

    let html = match std::fs::read_to_string(&view_path) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("view '{name}' not found")) }),
            );
        }
    };

    // Inline the SDK script so the view doesn't need to resolve relative paths
    let html = if let Ok(sdk) = std::fs::read_to_string(&sdk_path) {
        html.replace(
            r#"<script src="../ship-sdk.js"></script>"#,
            &format!("<script>\n{sdk}\n</script>"),
        )
    } else {
        html
    };

    (
        StatusCode::OK,
        Json(MeshResponse { ok: true, data: serde_json::json!({ "html": html }) }),
    )
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
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    let job_id = runtime::gen_ulid();
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug: req.slug,
        agent: req.agent,
        branch: req.branch,
        spec_path: req.spec_path,
        plan_id: None,
        model: None,
        provider: None,
        depends_on: req.depends_on,
        pipeline: None,
        steps: None,
        pipeline_name: req.pipeline,
        vars: None,
    };

    // Enrich with pipeline BEFORE persisting — the persisted event must include steps
    // so projections can track pipeline state.
    let payload = crate::supervisor::pipeline_loader::ensure_pipeline(payload);

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

/// POST /api/runtime/jobs/end — mark a job as completed by slug.
///
/// Called by the session Stop hook when an agent's session ends.
/// Resolves the job by slug, emits job.completed through the kernel.
/// The job_dispatch subscriber handles cleanup and unblocking dependents.
#[utoipa::path(
    post,
    path = "/api/runtime/jobs/end",
    request_body = JobEndRequest,
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn end_job(
    State(state): State<ApiState>,
    Json(req): Json<JobEndRequest>,
) -> impl IntoResponse {
    // Find the job by slug
    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    // Find the dispatched job with this slug — prefer dispatched over other statuses
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

    // Guard: should always be dispatched given the filter above, but keep for clarity
    if job.status != runtime::projections::job::JobStatus::Dispatched {
        return (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("job '{}' is {:?}, not dispatched — skipping", req.slug, job.status)) }),
        );
    }

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

/// POST /api/runtime/rebuild-projections — rebuild all projections from events.
#[utoipa::path(
    post,
    path = "/api/runtime/rebuild-projections",
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn rebuild_projections(
    State(state): State<ApiState>,
) -> impl IntoResponse {
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
struct ActorGuard {
    actor_id: String,
    kernel: crate::rest_api::SharedKernel,
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
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
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
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Mutex;

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
            workspaces.iter().any(|w| w["branch"] == "feature/rt-ws-test"),
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
            workspace_branch: ws.branch.clone(),
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
            workspace_branch: ws_a.branch.clone(),
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
    async fn list_workspaces_excludes_archived() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/active-ws");
        create_test_workspace(&state, "feature/archived-ws");

        // Archive one workspace
        runtime::workspace::transition_workspace_status(
            state.ship_dir(),
            "feature/archived-ws",
            runtime::workspace::WorkspaceStatus::Archived,
        )
        .unwrap();

        let response = list_workspaces(State(state)).await.into_response();
        let val = collect_body(response).await;
        let workspaces = val["data"]["workspaces"].as_array().unwrap();

        assert!(
            workspaces.iter().any(|w| w["branch"] == "feature/active-ws"),
            "active workspace should appear"
        );
        assert!(
            !workspaces.iter().any(|w| w["branch"] == "feature/archived-ws"),
            "archived workspace should be excluded"
        );
    }
}
