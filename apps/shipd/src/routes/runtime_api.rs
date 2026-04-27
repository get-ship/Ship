//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/v1/runtime` in lib.rs.
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
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

// ---- Query params ----

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SessionsQuery {
    pub workspace_id: Option<String>,
}

// ---- Handlers ----

/// GET /api/v1/runtime/workspaces
///
/// Returns only active workspaces. Archived/dead workspaces are excluded
/// from the API surface — the reconciler handles cleanup internally.
#[utoipa::path(
    operation_id = "listWorkspaces",
    get,
    path = "/api/v1/workspaces",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_workspaces(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
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
        .into_response()
}

/// Resolve HEAD SHA of the active project root. Used at `job.created` time
/// to capture the pipeline's baseline — the point every future worktree
/// branches from, and the `from` end of gate diff evaluation. `None` when
/// the project dir can't be resolved or git fails; gates fall back to the
/// step-local start SHA in that case.
pub(super) fn resolve_project_head_sha() -> Option<String> {
    let project_ship = runtime::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    runtime::services::orchestrator::git_helpers::resolve_head_sha(project_root)
}

/// Parse `git worktree list --porcelain` from the active project root.
/// Returns (branch, path) pairs, skipping detached and bare entries.
pub(super) fn git_worktrees() -> Option<Vec<(String, String)>> {
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

/// GET /api/v1/runtime/sessions[?workspace_id=<id>]
#[utoipa::path(
    operation_id = "listSessions",
    get,
    path = "/api/v1/sessions",
    params(SessionsQuery),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_sessions(
    State(_state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<SessionsQuery>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
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
        .into_response()
}


// ── mr-04 follow-up: per-handler submodules ──────────────────────────
//
// Each submodule owns one logical chunk of the runtime read API. The
// parent re-exports the public handler symbols + helpers (used by
// the test mod below) so existing call sites continue resolving.

pub mod agents;
pub mod daemon;
pub mod dispatch;
pub mod events;
pub mod jobs;
pub mod procs;

pub use agents::{SteerAgentRequest, list_agents, steer_agent, stop_agent};
pub use daemon::{daemon_status, rebuild_projections};
pub use dispatch::{DispatchRequest, dispatch_pipeline};
pub use events::{EventsPollQuery, event_stream, poll_events};
pub(crate) use events::ActorGuard;
pub use jobs::{
    CancelJobRequest, CreateJobRequest, EventSummary, JobDetailResponse, JobEndRequest,
    cancel_job, create_job, end_job, get_job_detail, list_jobs,
};
pub use procs::{ProcsQuery, list_procs};

// Helpers re-exported for the in-file test mod below.
#[cfg(test)]
pub(crate) use jobs::{
    annotate_job, drift_active_for_slug, list_processes_at, load_recent_events_by_type,
    truncate,
};


#[cfg(test)]
mod tests {
    use super::*;
    use axum::Extension;
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn make_api_state(dir: &TempDir) -> ApiState {
        let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        runtime::db::ensure_db_at(&ship_dir).unwrap();
        // Isolated per-test kernel — init_kernel_router uses a OnceLock that
        // pins the first test's ship_dir globally, breaking all subsequent
        // tests with "unable to open database file" when they try to write.
        let kernel = runtime::events::new_isolated_for_test(ship_dir.clone()).unwrap();
        Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
    }

    fn test_auth() -> Extension<AuthContext> {
        Extension(AuthContext {
            caller_kind: runtime::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        })
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
        let response = list_workspaces(State(state), test_auth()).await.into_response();
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
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
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
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
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
        let response = list_agents(State(state), test_auth()).await.into_response();
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

        let response = list_workspaces(State(state), test_auth()).await.into_response();
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

        let response = list_agents(State(state), test_auth()).await.into_response();
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
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
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
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
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

        let response = list_workspaces(State(state), test_auth()).await.into_response();
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

        let response = list_workspaces(State(state), test_auth()).await.into_response();
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
        let response = get_job_detail(State(state), test_auth(), Path("nonexistent-job-id".to_string()))
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
            source_spec_path: None,
            plan_id: None,
            model: None,
            provider: None,
            depends_on: None,
            pipeline: None,
            steps: None,
            pipeline_name: None,
            vars: None,
            spec_content: None,
            step_specs: None,
            headless: false,
            baseline_sha: None,
        };
        let envelope = runtime::events::EventEnvelope::new(
            runtime::events::job::event_types::JOB_CREATED,
            &job_id,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&envelope).await.unwrap();

        let response = get_job_detail(State(state), test_auth(), Path(job_id.clone()))
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

    async fn emit_job_created(state: &ApiState, slug: &str) -> String {
        let job_id = runtime::gen_ulid();
        let payload = runtime::events::job::JobCreatedPayload {
            job_id: job_id.clone(),
            slug: slug.into(),
            agent: "a".into(),
            branch: format!("job/{slug}"),
            spec_path: ".artifacts/job-spec.md".into(),
            source_spec_path: None,
            plan_id: None, model: None, provider: None, depends_on: None,
            pipeline: None, steps: None, pipeline_name: None, vars: None,
            spec_content: None, step_specs: None, headless: false,
            baseline_sha: None,
        };
        let env = EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
            .unwrap()
            .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
        job_id
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_emits_job_cancelled_for_pending_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let job_id = emit_job_created(&state, "cancel-ok").await;

        let req = CancelJobRequest { reason: "smoke".into() };
        let response = cancel_job(
            State(state.clone()),
            Extension(full_auth()),
            Path("cancel-ok".to_string()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let jobs = runtime::projections::job::load_jobs_at(state.ship_dir()).unwrap();
        let job = jobs.get(&job_id).unwrap();
        assert_eq!(job.status, runtime::projections::job::JobStatus::Cancelled);

        // Confirm the cancelled event actually landed.
        let events = runtime::db::events::query_recent_events_for_entity_at(
            &runtime::db::platform_db_path(state.ship_dir()),
            &job_id, 20,
        )
        .unwrap();
        assert!(
            events.iter().any(|e| e.event_type == event_types::JOB_CANCELLED),
            "expected job.cancelled in event log"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_returns_409_for_already_terminal_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let job_id = emit_job_created(&state, "cancel-term").await;

        // Seed a terminal state first.
        let fail_payload = runtime::events::job::JobFailedPayload {
            job_id: job_id.clone(),
            error: "boom".into(),
        };
        let env = EventEnvelope::new(event_types::JOB_FAILED, &job_id, &fail_payload)
            .unwrap()
            .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();

        let req = CancelJobRequest { reason: "after-fail".into() };
        let response = cancel_job(
            State(state.clone()),
            Extension(full_auth()),
            Path("cancel-term".to_string()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        // Make sure no job.cancelled was emitted (idempotent no-op).
        let events = runtime::db::events::query_recent_events_for_entity_at(
            &runtime::db::platform_db_path(state.ship_dir()),
            &job_id, 20,
        )
        .unwrap();
        assert!(
            events.iter().all(|e| e.event_type != event_types::JOB_CANCELLED),
            "409 path must emit no job.cancelled event"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_returns_404_for_unknown_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let req = CancelJobRequest { reason: "x".into() };
        let response = cancel_job(
            State(state),
            Extension(full_auth()),
            Path("nope".to_string()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
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

    // ---- Slice 6 endpoints ----

    async fn emit_process_spawned(state: &ApiState, slug: &str, pid: u32) {
        let payload = runtime::events::process::ProcessSpawnedPayload {
            slug: slug.to_string(),
            job_id: format!("job-{slug}"),
            pid: Some(pid),
            cmd: "claude".to_string(),
            argv: vec![],
            env_keys: vec![],
            cwd: "/tmp".to_string(),
            executor_kind: "process".to_string(),
            step_id: None,
        };
        let env = EventEnvelope::new(
            runtime::events::process::event_types::PROCESS_SPAWNED,
            slug,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    async fn emit_drift_detected(state: &ApiState, slug: &str) {
        let payload = runtime::events::supervisor::DriftDetectedPayload {
            slug: slug.into(),
            job_id: format!("job-{slug}"),
            expected_status: "Dispatched".into(),
            actual_state: "no_process".into(),
            detail: "test".into(),
        };
        let env = EventEnvelope::new(
            runtime::events::supervisor::event_types::SUPERVISOR_DRIFT_DETECTED,
            slug,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    async fn emit_sweep_completed(state: &ApiState, drift_count: usize) {
        let payload = runtime::events::supervisor::SweepCompletedPayload {
            duration_ms: 12,
            drift_count,
            actions_taken: drift_count,
        };
        let env = EventEnvelope::new(
            runtime::events::supervisor::event_types::SUPERVISOR_SWEEP_COMPLETED,
            "supervisor",
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_jobs_annotates_terminal_jobs_as_terminal() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let job_id = emit_job_created(&state, "term-anno").await;

        // Drive into Failed.
        let env = EventEnvelope::new(
            event_types::JOB_FAILED,
            &job_id,
            &runtime::events::job::JobFailedPayload {
                job_id: job_id.clone(),
                error: "boom".into(),
            },
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();

        let response = list_jobs(State(state)).await.into_response();
        let val = collect_body(response).await;
        let jobs = val["data"]["jobs"].as_array().unwrap();
        let row = jobs.iter().find(|j| j["slug"] == "term-anno").unwrap();
        assert_eq!(row["process_state"], serde_json::json!("terminal"));
        assert_eq!(row["notes"], serde_json::json!("boom"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_jobs_marks_drift_as_missing_when_unresolved() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _job_id = emit_job_created(&state, "drift-job").await;
        emit_drift_detected(&state, "drift-job").await;

        let response = list_jobs(State(state)).await.into_response();
        let val = collect_body(response).await;
        let jobs = val["data"]["jobs"].as_array().unwrap();
        let row = jobs.iter().find(|j| j["slug"] == "drift-job").unwrap();
        assert_eq!(row["process_state"], serde_json::json!("missing"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_jobs_marks_alive_for_running_process() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _job_id = emit_job_created(&state, "alive-job").await;
        emit_process_spawned(&state, "alive-job", 1234).await;

        let response = list_jobs(State(state)).await.into_response();
        let val = collect_body(response).await;
        let jobs = val["data"]["jobs"].as_array().unwrap();
        let row = jobs.iter().find(|j| j["slug"] == "alive-job").unwrap();
        assert_eq!(row["process_state"], serde_json::json!("alive"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_procs_filters_by_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        emit_process_spawned(&state, "procs-a", 1).await;
        emit_process_spawned(&state, "procs-b", 2).await;

        let q = Query(ProcsQuery { slug: Some("procs-a".into()), limit: None });
        let response = list_procs(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let procs = val["data"]["procs"].as_array().unwrap();
        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0]["slug"], "procs-a");
        assert_eq!(procs[0]["pid"], 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_procs_returns_all_when_no_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        emit_process_spawned(&state, "all-a", 10).await;
        emit_process_spawned(&state, "all-b", 11).await;

        let q = Query(ProcsQuery { slug: None, limit: None });
        let response = list_procs(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let procs = val["data"]["procs"].as_array().unwrap();
        assert!(procs.len() >= 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn poll_events_filters_by_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "tail-x").await;
        let _ = emit_job_created(&state, "tail-y").await;

        let q = Query(EventsPollQuery {
            since_id: None,
            slug: Some("tail-x".into()),
            namespace: None,
            limit: None,
        });
        let response = poll_events(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let events = val["data"]["events"].as_array().unwrap();
        assert!(!events.is_empty(), "should have at least one event for slug");
        for e in events {
            let payload: serde_json::Value =
                serde_json::from_str(e["payload_json"].as_str().unwrap_or("{}")).unwrap();
            assert_eq!(payload["slug"], "tail-x");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn poll_events_namespace_filter() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "ns-test").await;
        emit_process_spawned(&state, "ns-test", 99).await;

        let q = Query(EventsPollQuery {
            since_id: None,
            slug: Some("ns-test".into()),
            namespace: Some("process".into()),
            limit: None,
        });
        let response = poll_events(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let events = val["data"]["events"].as_array().unwrap();
        assert!(!events.is_empty());
        for e in events {
            assert!(
                e["event_type"].as_str().unwrap().starts_with("process."),
                "namespace filter must restrict to process.*"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn poll_events_respects_since_id_cursor() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "cursor-test").await;

        let first = Query(EventsPollQuery {
            since_id: None,
            slug: Some("cursor-test".into()),
            namespace: None,
            limit: None,
        });
        let response = poll_events(State(state.clone()), first).await.into_response();
        let val = collect_body(response).await;
        let events = val["data"]["events"].as_array().unwrap();
        let last_id = events.last().unwrap()["id"].as_str().unwrap().to_string();

        let second = Query(EventsPollQuery {
            since_id: Some(last_id),
            slug: Some("cursor-test".into()),
            namespace: None,
            limit: None,
        });
        let response = poll_events(State(state), second).await.into_response();
        let val = collect_body(response).await;
        assert_eq!(val["data"]["events"].as_array().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn daemon_status_returns_pid_and_uptime() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "status-job").await;
        emit_sweep_completed(&state, 0).await;

        let response = daemon_status(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let val = collect_body(response).await;
        let d = &val["data"];
        assert!(d["pid"].as_u64().unwrap() > 0);
        assert!(d["jobs_total"].as_u64().unwrap() >= 1);
        assert_eq!(d["jobs_by_status"]["pending"], serde_json::json!(1));
        assert!(d["last_sweep_at"].is_string());
        assert_eq!(d["last_sweep_drift"], serde_json::json!(0));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn drift_active_for_slug_resolved_returns_false() {
        let drift_ts = chrono::Utc::now() - chrono::Duration::seconds(10);
        let restored_ts = chrono::Utc::now();
        let mk = |t: &str, ts: chrono::DateTime<chrono::Utc>| EventEnvelope {
            id: "x".into(),
            event_type: t.into(),
            entity_id: "slug".into(),
            actor: "test".into(),
            payload_json: "{}".into(),
            version: 1,
            causation_id: None,
            workspace_id: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: ts,
        };
        let events = vec![
            mk("supervisor.drift_detected", drift_ts),
            mk("supervisor.coherence_restored", restored_ts),
        ];
        assert!(!drift_active_for_slug("slug", &events));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn drift_active_for_slug_unresolved_returns_true() {
        let mk = |t: &str| EventEnvelope {
            id: "x".into(),
            event_type: t.into(),
            entity_id: "slug".into(),
            actor: "test".into(),
            payload_json: "{}".into(),
            version: 1,
            causation_id: None,
            workspace_id: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: chrono::Utc::now(),
        };
        let events = vec![mk("supervisor.drift_detected")];
        assert!(drift_active_for_slug("slug", &events));
    }
}
