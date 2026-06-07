//! Pipeline runs — `POST /api/v1/pipelines/run`.
//!
//! Thin HTTP transport adapter. Domain logic lives in
//! `ship_pipeline_runtime::run_core::run_pipeline_core`.
//! This handler enforces the `CreateRuns` scope gate (transport concern)
//! then delegates to the crate core.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use ship_pipeline_runtime::run_core::{PipelineRunRequest, RunPipelineCoreError, run_pipeline_core};

use crate::auth::{AuthContext, Scope, require_scope};
use crate::pipeline_ports::PipelineDispatchAdapter;
use crate::routes::rest_api::{ApiState, MeshResponse};

/// POST /api/v1/pipelines/run
pub async fn run_pipeline(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<PipelineRunRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::CreateRuns) {
        return e;
    }

    let port = PipelineDispatchAdapter::new(state);
    match run_pipeline_core(&port, req).await {
        Ok(outcome) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({
                    "pipeline_run_id": outcome.pipeline_run_id,
                    "slug": outcome.slug,
                    "branch": outcome.branch,
                    "resume": outcome.resume,
                }),
            }),
        ),
        Err(RunPipelineCoreError::NotFound(m)) => (
            StatusCode::NOT_FOUND,
            Json(MeshResponse { ok: false, data: serde_json::json!(m) }),
        ),
        Err(RunPipelineCoreError::BadRequest(m)) => (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse { ok: false, data: serde_json::json!(m) }),
        ),
        Err(RunPipelineCoreError::Internal(m)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(m) }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, OnceLock};
    use tempfile::TempDir;

    /// CWD is process-global; the handler resolves pipelines by walking
    /// from `std::env::current_dir()`. Serialize tests that chdir so they
    /// don't tear out each other's working directory mid-run.
    fn cwd_guard() -> std::sync::MutexGuard<'static, ()> {
        static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn make_state(dir: &TempDir) -> ApiState {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
        let kernel = ship_substrate::events::new_isolated_for_test(ship_dir.clone()).unwrap();
        Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
    }

    fn full_auth() -> Extension<AuthContext> {
        Extension(AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
            grants: vec![],
            operator_cred: true,
        })
    }

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(
        clippy::await_holding_lock,
        reason = "test serializes cwd mutation across the await — the whole point of this lock"
    )]
    async fn pipeline_run_missing_pipeline_returns_404() {
        let _g = cwd_guard();
        let dir = TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(ship_dir.join("pipelines")).unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let state = make_state(&dir);
        let req = PipelineRunRequest {
            pipeline: "does-not-exist".into(),
            spec: None,
            vars: None,
            harness: None,
            model: None,
            workspace: None,
            slug: None,
            auto: false,
            source_spec_path: None,
        };
        let resp = run_pipeline(State(state), full_auth(), Json(req))
            .await
            .into_response();
        std::env::set_current_dir(prev).unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let val = body_json(resp).await;
        assert_eq!(val["ok"], serde_json::json!(false));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(
        clippy::await_holding_lock,
        reason = "test serializes cwd mutation across the await — the whole point of this lock"
    )]
    async fn pipeline_run_multi_step_dispatches_via_shared_primitive() {
        let _g = cwd_guard();
        let dir = TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        let pipelines_dir = ship_dir.join("pipelines");
        std::fs::create_dir_all(&pipelines_dir).unwrap();
        std::fs::write(
            pipelines_dir.join("two-step.jsonc"),
            r#"{
                "name": "two-step",
                "steps": [
                    { "id": "implement", "agent": "rust-lane" },
                    { "id": "review",    "agent": "gatekeeper", "after": ["implement"] }
                ]
            }"#,
        )
        .unwrap();

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let state = make_state(&dir);
        let req = PipelineRunRequest {
            pipeline: "two-step".into(),
            spec: Some("walk the DAG".into()),
            vars: None,
            harness: None,
            model: None,
            workspace: Some("dag-ws".into()),
            slug: None,
            auto: true,
            source_spec_path: None,
        };
        let resp = run_pipeline(State(state), full_auth(), Json(req))
            .await
            .into_response();
        std::env::set_current_dir(prev).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let val = body_json(resp).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        let slug = val["data"]["slug"].as_str().unwrap();
        assert!(slug.starts_with("dag-ws-"), "slug should be workspace-prefixed: {slug}");
    }

    #[test]
    fn pipeline_run_request_rejects_unknown_fields() {
        let body = serde_json::json!({
            "pipeline": "tdd",
            "agent": "rust-lane",
        });
        let err = serde_json::from_value::<PipelineRunRequest>(body)
            .err()
            .expect("must reject unknown fields");
        let msg = err.to_string();
        assert!(
            msg.contains("agent") || msg.contains("unknown field"),
            "expected agent-field rejection on pipeline-run, got: {msg}"
        );
    }
}
