//! Pipeline runs — `POST /api/v1/pipelines/run`.
//!
//! Owns the multi-step concern. Loads the named pipeline JSONC from
//! `.ship/pipelines/<name>.jsonc`, builds a `PipelineRunStartedPayload`
//! seeded with the parsed steps, and hands it to the shared `dispatch_one`
//! primitive. The orchestrator walks the DAG from there.
//!
//! Distinct from `POST /api/v1/agent_sessions/dispatch`, which dispatches
//! a SINGLE agent and does no pipeline loading — see `dispatch.rs`.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use ship_substrate::events::pipeline::PipelineRunStartedPayload;
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::{ApiState, MeshResponse};

use super::dispatch::dispatch_one;

/// Wire shape for `POST /api/v1/pipelines/run`. `pipeline` names a file
/// under `.ship/pipelines/<name>.jsonc`.
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineRunRequest {
    /// Pipeline name (file in `.ship/pipelines/<name>.jsonc`).
    pub pipeline: String,

    /// Inline spec body. Written to `.artifacts/job-spec.md` for the first
    /// agent; per-step bodies come from the pipeline file's `step_specs:`.
    #[serde(default)]
    pub spec: Option<String>,

    /// Pipeline-level `{{ var }}` overrides. Merged over the file's defaults.
    #[serde(default)]
    pub vars: Option<std::collections::HashMap<String, String>>,

    /// Harness override applied to every step that does not declare its own.
    #[serde(default)]
    pub harness: Option<String>,

    /// Model override applied to every step that does not declare its own.
    #[serde(default)]
    pub model: Option<String>,

    /// Workspace label (used for slug generation). Defaults to the
    /// pipeline name.
    #[serde(default)]
    pub workspace: Option<String>,

    /// Explicit slug override.
    #[serde(default)]
    pub slug: Option<String>,

    /// Autonomous (headless) vs interactive. Default false (interactive).
    #[serde(default)]
    pub auto: bool,

    /// Operator-facing path to the spec file (for `ship jobs` display).
    #[serde(default)]
    pub source_spec_path: Option<String>,
}

/// POST /api/v1/pipelines/run
#[utoipa::path(
    post,
    path = "/api/v1/pipelines/run",
    request_body = PipelineRunRequest,
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn run_pipeline(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<PipelineRunRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::CreateRuns) {
        return e;
    }

    // Locate the pipeline JSONC. Matches the resolution strategy used by
    // `pipeline_loader::load_pipeline_file` so `ensure_pipeline` (called
    // inside `dispatch_one`) will find the same file.
    let pipeline_path = {
        let mut dir = std::env::current_dir().ok();
        let mut found = None;
        while let Some(d) = dir.as_ref() {
            let pipelines_dir = ship_substrate::ShipHome::project_ship_dir(d).join("pipelines");
            if pipelines_dir.is_dir() {
                let candidate = pipelines_dir.join(format!("{}.jsonc", req.pipeline));
                if candidate.exists() {
                    found = Some(candidate);
                }
                break;
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
        found
    };
    let Some(pipeline_path) = pipeline_path else {
        return (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(format!("pipeline '{}' not found", req.pipeline)),
            }),
        );
    };
    let raw = match std::fs::read_to_string(&pipeline_path) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("read pipeline: {e}")),
                }),
            );
        }
    };
    let (normalized, meta) = match ship_pipelines::normalize_pipeline(&raw) {
        Ok(n) => n,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("pipeline parse: {e}")),
                }),
            );
        }
    };

    let steps = match normalized.get("steps").and_then(|v| v.as_array()) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!("pipeline has no steps"),
                }),
            );
        }
    };

    // Run-level default agent = the first step's agent. A pure-shell
    // (run) first step has none; leave it empty rather than falling back
    // to a bogus "default" profile — worktree prep skips agent
    // compilation for empty agents (a real agent step always names one).
    let first_agent = steps
        .first()
        .and_then(|s| s.get("agent"))
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();

    let pipeline_name = meta.name.clone().unwrap_or_else(|| req.pipeline.clone());
    let workspace_label = req.workspace.unwrap_or_else(|| pipeline_name.clone());
    let slug = req.slug.unwrap_or_else(|| {
        // Random-tail suffix; a ULID prefix is pure timestamp and
        // collides for rapid same-workspace dispatches (gen_slug_suffix).
        format!("{}-{}", workspace_label, ship_substrate::gen_slug_suffix())
    });
    let branch = format!("job/{slug}");
    let pipeline_run_id = ship_substrate::gen_ulid();

    let payload = PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.clone(),
        slug,
        agent: first_agent,
        branch,
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: req.source_spec_path,
        plan_id: None,
        model: req.model,
        harness: req.harness,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: Some(pipeline_name),
        vars: req.vars,
        spec_content: req.spec,
        step_specs: None,
        headless: req.auto,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: super::resolve_project_head_sha(),
    };

    dispatch_one(&state, payload, &pipeline_run_id).await
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
            session_id: None,
            scopes: vec![Scope::Full],
            grants: vec![],
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
        // Create the `.ship/` dir so the handler's upward walk stops here
        // instead of climbing into the repo's own `.ship/`.
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
