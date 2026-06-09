//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/v1/runtime` in lib.rs.
//! All handlers are read-only and use the existing `ApiState`.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
#[cfg(test)]
use axum::extract::Path;
#[cfg(test)]
use ship_substrate::events::EventEnvelope;
#[cfg(test)]
use ship_substrate::events::pipeline::event_types;

use crate::auth::AuthContext;
use crate::ship_runtime::ApiState;
use crate::wire::MeshResponse;

/// GET /api/v1/sessions/active-across-projects
///
/// Thin transport shell — delegates to
/// `ship_substrate::runtime_reads::list_active_sessions_with_projects`.
pub async fn list_sessions_across_projects(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
) -> impl IntoResponse {
    match ship_substrate::runtime_reads::list_active_sessions_with_projects() {
        Ok(data) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "sessions": data }),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e),
            }),
        )
            .into_response(),
    }
}


// ── mr-04 follow-up: per-handler submodules ──────────────────────────
//
// Each submodule owns one logical chunk of the runtime read API. The
// parent re-exports the public handler symbols + helpers (used by
// the test mod below) so existing call sites continue resolving.

pub mod daemon;
pub mod events;
pub mod jobs;
pub mod jobs_status;
pub mod pipeline_run;

pub use daemon::{daemon_status, rebuild_projections};
pub use events::{EventsPollQuery, event_stream};
pub use jobs::{
    CancelJobRequest, EventSummary, PipelineRunDetailResponse, TurnEndRequest,
    cancel_job, end_job, get_job_detail, list_jobs,
};
pub use jobs_status::{step_done, update_job_status};
pub use pipeline_run::run_pipeline;
pub use ship_pipeline_runtime::run_core::PipelineRunRequest;

// Helpers re-exported for the in-file test mod below.
#[cfg(test)]
pub(crate) use jobs::drift_active_for_slug;


#[cfg(test)]
mod tests {
    use super::*;
    use axum::Extension;
    use axum::extract::State;
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn make_api_state(dir: &TempDir) -> ApiState {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
        // Isolated per-test kernel — init_kernel_router uses a OnceLock that
        // pins the first test's ship_dir globally, breaking all subsequent
        // tests with "unable to open database file" when they try to write.
        let kernel = ship_substrate::events::new_isolated_for_test(ship_dir.clone()).unwrap();
        Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
    }

    fn test_auth() -> Extension<AuthContext> {
        Extension(AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
            // The operator stand-in: a verified operator cred clears the gate
            // (a bare CLI no longer does — S6).
            operator_cred: true,
        })
    }

    fn full_auth() -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
            operator_cred: true,
        }
    }

    async fn collect_body(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
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

        // Emit a job.created event so load_pipeline_runs() finds it.
        let pipeline_run_id = ship_substrate::gen_ulid();
        let payload = ship_substrate::events::pipeline::PipelineRunStartedPayload {
            pipeline_run_id: pipeline_run_id.clone(),
            slug: "test-detail".into(),
            agent: "agent-1".into(),
            branch: "feature/detail".into(),
            spec_path: ".artifacts/job-spec.md".into(),
            source_spec_path: None,
            plan_id: None,
            model: None,
            harness: None,
            depends_on: None,
            pipeline: None,
            steps: None,
            pipeline_name: None,
            vars: None,
            spec_content: None,
            step_specs: None,
            headless: false,
            prompt: None,
            resume: false,
            resume_session_id: None,
            baseline_sha: None,
            keepalive: false,
        };
        let envelope = ship_substrate::events::EventEnvelope::new(
            ship_substrate::events::pipeline::event_types::PIPELINE_RUN_STARTED,
            &pipeline_run_id,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&envelope).await.unwrap();

        let response = get_job_detail(State(state), test_auth(), Path(pipeline_run_id.clone()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert_eq!(val["data"]["job"]["pipeline_run_id"], pipeline_run_id);
        assert_eq!(val["data"]["job"]["slug"], "test-detail");
        assert!(val["data"]["step_observations"].is_array());
        assert!(val["data"]["recent_events"].is_array());
        // The job.created event should appear in recent_events
        assert!(!val["data"]["recent_events"].as_array().unwrap().is_empty());
    }

    async fn emit_pipeline_run_started(state: &ApiState, slug: &str) -> String {
        let pipeline_run_id = ship_substrate::gen_ulid();
        let payload = ship_substrate::events::pipeline::PipelineRunStartedPayload {
            pipeline_run_id: pipeline_run_id.clone(),
            slug: slug.into(),
            agent: "a".into(),
            branch: format!("job/{slug}"),
            spec_path: ".artifacts/job-spec.md".into(),
            source_spec_path: None,
            plan_id: None, model: None, harness: None, depends_on: None,
            pipeline: None, steps: None, pipeline_name: None, vars: None,
            spec_content: None, step_specs: None, headless: false,
            prompt: None,
            resume: false,
            resume_session_id: None,
            baseline_sha: None,
            keepalive: false,
        };
        let env = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, &pipeline_run_id, &payload)
            .unwrap()
            .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
        pipeline_run_id
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_emits_job_cancelled_for_pending_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let pipeline_run_id = emit_pipeline_run_started(&state, "cancel-ok").await;

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

        let jobs = ship_pipeline_runtime::projections::pipeline_run::load_pipeline_runs_at(state.ship_dir()).unwrap();
        let job = jobs.get(&pipeline_run_id).unwrap();
        assert_eq!(job.status, ship_pipeline_runtime::projections::pipeline_run::PipelineRunStatus::Cancelled);

        // Confirm the cancelled event actually landed.
        let events = ship_substrate::db::events::query_recent_events_for_entity_at(
            &ship_substrate::db::runtime_db_path(state.ship_dir()),
            &pipeline_run_id, 20,
        )
        .unwrap();
        assert!(
            events.iter().any(|e| e.event_type == event_types::PIPELINE_RUN_CANCELLED),
            "expected job.cancelled in event log"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_returns_409_for_already_terminal_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let pipeline_run_id = emit_pipeline_run_started(&state, "cancel-term").await;

        // Seed a terminal state first.
        let fail_payload = ship_substrate::events::pipeline::PipelineRunFailedPayload {
            pipeline_run_id: pipeline_run_id.clone(),
            error: "boom".into(),
        };
        let env = EventEnvelope::new(event_types::PIPELINE_RUN_FAILED, &pipeline_run_id, &fail_payload)
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
        let events = ship_substrate::db::events::query_recent_events_for_entity_at(
            &ship_substrate::db::runtime_db_path(state.ship_dir()),
            &pipeline_run_id, 20,
        )
        .unwrap();
        assert!(
            events.iter().all(|e| e.event_type != event_types::PIPELINE_RUN_CANCELLED),
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

    // ---- Slice 6 endpoints ----

    async fn emit_process_spawned(state: &ApiState, slug: &str, pid: u32) {
        let payload = ship_substrate::events::process::ProcessSpawnedPayload {
            slug: slug.to_string(),
            pipeline_run_id: format!("pipeline-run-{slug}"),
            pid: Some(pid),
            cmd: "claude".to_string(),
            argv: vec![],
            env_keys: vec![],
            cwd: "/tmp".to_string(),
            executor_kind: "process".to_string(),
            step_id: None,
        };
        let env = EventEnvelope::new(
            ship_substrate::events::process::event_types::PROCESS_SPAWNED,
            slug,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    async fn emit_drift_detected(state: &ApiState, slug: &str) {
        let payload = ship_substrate::events::supervisor::DriftDetectedPayload {
            slug: slug.into(),
            pipeline_run_id: format!("pipeline-run-{slug}"),
            expected_status: "Dispatched".into(),
            actual_state: "no_process".into(),
            detail: "test".into(),
        };
        let env = EventEnvelope::new(
            ship_substrate::events::supervisor::event_types::SUPERVISOR_DRIFT_DETECTED,
            slug,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    async fn emit_sweep_completed(state: &ApiState, drift_count: usize) {
        let payload = ship_substrate::events::supervisor::SweepCompletedPayload {
            duration_ms: 12,
            drift_count,
            actions_taken: drift_count,
        };
        let env = EventEnvelope::new(
            ship_substrate::events::supervisor::event_types::SUPERVISOR_SWEEP_COMPLETED,
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
        let pipeline_run_id = emit_pipeline_run_started(&state, "term-anno").await;

        // Drive into Failed.
        let env = EventEnvelope::new(
            event_types::PIPELINE_RUN_FAILED,
            &pipeline_run_id,
            &ship_substrate::events::pipeline::PipelineRunFailedPayload {
                pipeline_run_id: pipeline_run_id.clone(),
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
        let _job_id = emit_pipeline_run_started(&state, "drift-job").await;
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
        let _job_id = emit_pipeline_run_started(&state, "alive-job").await;
        emit_process_spawned(&state, "alive-job", 1234).await;

        let response = list_jobs(State(state)).await.into_response();
        let val = collect_body(response).await;
        let jobs = val["data"]["jobs"].as_array().unwrap();
        let row = jobs.iter().find(|j| j["slug"] == "alive-job").unwrap();
        assert_eq!(row["process_state"], serde_json::json!("alive"));
    }


    #[tokio::test(flavor = "multi_thread")]
    async fn daemon_status_returns_pid_and_uptime() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_pipeline_run_started(&state, "status-job").await;
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
            scope: ship_substrate::events::EventScope::Runtime,
            scope_key: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: ts,
            author_account_id: None,
            author_device_id: None,
            signature: None,
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
            scope: ship_substrate::events::EventScope::Runtime,
            scope_key: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: chrono::Utc::now(),
            author_account_id: None,
            author_device_id: None,
            signature: None,
        };
        let events = vec![mk("supervisor.drift_detected")];
        assert!(drift_active_for_slug("slug", &events));
    }
}
