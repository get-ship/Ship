//! Single-agent dispatch — `POST /api/v1/agent_sessions/dispatch`.
//!
//! Agent-centric: requires `agent`. The pipeline-run endpoint (see
//! `pipeline_run.rs`) is the separate concern for multi-step pipelines.
//! Both share `dispatch_one`, which builds a `PipelineRunStartedPayload`
//! and emits a `pipeline.run.started` event.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, event_types};
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::{ApiState, MeshResponse};

/// Wire shape for `POST /api/v1/agent_sessions/dispatch`. Agent-centric —
/// `agent` is required and there is no `pipeline` field; pipeline runs
/// live at `POST /api/v1/pipelines/run`.
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DispatchRequest {
    /// Ship-agent profile id (required).
    pub agent: String,

    /// Inline spec body. Becomes `.artifacts/job-spec.md` for the agent at launch.
    #[serde(default)]
    pub spec: Option<String>,

    /// Inline one-shot prompt. Either piped via the harness's one-shot
    /// flag (`auto` mode) or injected as the first user turn (interactive).
    #[serde(default)]
    pub prompt: Option<String>,

    /// Continue the last session for this (agent, workspace) pair. Errors
    /// with 400 when no matching session exists.
    #[serde(default)]
    pub resume: bool,

    /// Harness override for this dispatch. None → project/user default chain.
    #[serde(default)]
    pub harness: Option<String>,

    /// Model override. None → harness default.
    #[serde(default)]
    pub model: Option<String>,

    /// Autonomous (headless) vs interactive. Default false (interactive).
    #[serde(default)]
    pub auto: bool,

    /// Workspace label (used for slug generation). Power-user override.
    #[serde(default)]
    pub workspace: Option<String>,

    /// Explicit slug. When `None`, derived from `workspace` + a ULID suffix.
    #[serde(default)]
    pub slug: Option<String>,

    /// Explicit branch. When `None`, defaults to `job/<slug>`.
    #[serde(default)]
    pub branch: Option<String>,

    /// Operator-facing path to the spec file (for `ship jobs` display).
    #[serde(default)]
    pub source_spec_path: Option<String>,

    /// Parent job slugs this run depends on.
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,

    /// Per-launch vars. Pipelines pass their own vars through to each
    /// dispatch via the pipeline-run endpoint.
    #[serde(default)]
    pub vars: Option<std::collections::HashMap<String, String>>,
}

/// POST /api/v1/agent_sessions/dispatch
///
/// Agent-centric single-agent dispatch. Rejects payloads carrying a
/// `pipeline` field (those belong at `POST /api/v1/pipelines/run`).
#[utoipa::path(
    post,
    path = "/api/v1/agent_sessions/dispatch",
    request_body = DispatchRequest,
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn dispatch_agent_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<DispatchRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::CreateJobs) {
        return e;
    }

    let workspace_label = req
        .workspace
        .clone()
        .unwrap_or_else(|| req.agent.clone());

    // Resume: resolve an existing (agent, workspace) session and reuse
    // its slug/branch. With no match, error clearly per the spec.
    let resume_session = if req.resume {
        match resolve_resume_session(&req.agent, &workspace_label) {
            Ok(Some(s)) => Some(s),
            Ok(None) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(MeshResponse {
                        ok: false,
                        data: serde_json::json!(format!(
                            "resume: no existing session for agent `{}` in workspace `{}`",
                            req.agent, workspace_label
                        )),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(MeshResponse {
                        ok: false,
                        data: serde_json::json!(format!("resume lookup failed: {e}")),
                    }),
                );
            }
        }
    } else {
        None
    };

    let slug = req
        .slug
        .clone()
        .or_else(|| resume_session.as_ref().map(|s| s.slug.clone()))
        .unwrap_or_else(|| {
            format!(
                "{}-{}",
                workspace_label,
                &ship_substrate::gen_ulid()[..8].to_lowercase()
            )
        });
    let branch = req
        .branch
        .clone()
        .or_else(|| resume_session.as_ref().map(|s| s.branch.clone()))
        .unwrap_or_else(|| format!("job/{slug}"));
    let job_id = ship_substrate::gen_ulid();

    let payload = PipelineRunStartedPayload {
        job_id: job_id.clone(),
        slug,
        agent: req.agent.clone(),
        branch,
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: req.source_spec_path,
        plan_id: None,
        model: req.model,
        harness: req.harness,
        depends_on: req.depends_on,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: req.vars,
        spec_content: req.spec,
        step_specs: None,
        headless: req.auto,
        prompt: req.prompt,
        resume: req.resume,
        baseline_sha: super::resolve_project_head_sha(),
    };

    dispatch_one(&state, payload, &job_id).await
}

/// Shared dispatch primitive — builds the `pipeline.run.started` envelope
/// from `payload`, runs `ensure_pipeline` (so multi-step callers get their
/// DAG validated/interpolated), persists + routes it, and returns the
/// standard `MeshResponse` body.
///
/// Callable from both the agent-session dispatch endpoint and the
/// pipeline-run endpoint.
pub(crate) async fn dispatch_one(
    state: &ApiState,
    payload: PipelineRunStartedPayload,
    job_id: &str,
) -> (StatusCode, Json<MeshResponse>) {
    let payload =
        match ship_platform::services::orchestrator::pipeline_loader::ensure_pipeline(payload) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(MeshResponse { ok: false, data: serde_json::json!(e) }),
                );
            }
        };

    let envelope = match EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, job_id, &payload)
        .map(|e| e.with_actor_id("studio"))
    {
        Ok(e) => e,
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

    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({
                    "job_id": job_id,
                    "slug": payload.slug,
                    "branch": payload.branch,
                    "resume": payload.resume,
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

/// Snapshot of a prior session that the dispatch handler reuses when
/// `resume: true`. Only the bits dispatch needs.
pub(crate) struct ResumeSession {
    pub slug: String,
    pub branch: String,
}

/// Locate the most-recent session for `(agent, workspace)` if one exists.
/// Returns `Ok(None)` when nothing matches — the caller decides whether
/// that is a hard error (resume) or expected (cold start).
fn resolve_resume_session(
    agent: &str,
    workspace: &str,
) -> Result<Option<ResumeSession>, anyhow::Error> {
    let sessions = ship_substrate::db::session::list_sessions_db(Some(workspace), 100)?;
    let hit = sessions
        .into_iter()
        .find(|s| s.agent_id.as_deref() == Some(agent));
    Ok(hit.map(|s| ResumeSession {
        slug: s.id,
        branch: s.branch,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

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
        })
    }

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn single_agent_dispatch_returns_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let req = DispatchRequest {
            agent: "rust-lane".into(),
            spec: None,
            prompt: None,
            resume: false,
            harness: None,
            model: None,
            auto: true,
            workspace: Some("solo".into()),
            slug: None,
            branch: None,
            source_spec_path: None,
            depends_on: None,
            vars: None,
        };
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let val = body_json(resp).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["slug"].as_str().unwrap().starts_with("solo-"));
        assert!(val["data"]["job_id"].as_str().is_some());
    }

    #[test]
    fn dispatch_request_rejects_pipeline_field() {
        // `deny_unknown_fields` makes the legacy `pipeline:` key a 400.
        let body = serde_json::json!({
            "agent": "rust-lane",
            "pipeline": "tdd",
        });
        let err = serde_json::from_value::<DispatchRequest>(body)
            .err()
            .expect("must reject pipeline field");
        let msg = err.to_string();
        assert!(
            msg.contains("pipeline") || msg.contains("unknown field"),
            "expected pipeline-field rejection, got: {msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resume_without_existing_session_returns_400() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let req = DispatchRequest {
            agent: "rust-lane".into(),
            spec: None,
            prompt: None,
            resume: true,
            harness: None,
            model: None,
            auto: false,
            workspace: Some("empty-ws".into()),
            slug: None,
            branch: None,
            source_spec_path: None,
            depends_on: None,
            vars: None,
        };
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let val = body_json(resp).await;
        assert_eq!(val["ok"], serde_json::json!(false));
        let data = val["data"].as_str().unwrap_or("");
        assert!(
            data.contains("resume") && data.contains("no existing session"),
            "expected clear resume error, got: {data}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_includes_prompt_in_response() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let req = DispatchRequest {
            agent: "rust-lane".into(),
            spec: None,
            prompt: Some("hello".into()),
            resume: false,
            harness: None,
            model: None,
            auto: false,
            workspace: Some("p".into()),
            slug: None,
            branch: None,
            source_spec_path: None,
            depends_on: None,
            vars: None,
        };
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
