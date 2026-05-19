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

use crate::auth::{AuthContext};
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

    /// Long-running interactive session — NOT a pipeline step. The
    /// harness sits at its TUI prompt waiting for stdin pushes
    /// (`/api/v1/sessions/{slug}/message`) or WebSocket attach. Cancel
    /// is the only exit. Mutually exclusive with `auto`. Drives
    /// cmdb-modal chat, reliable PTY attach, and harness-fires-hook
    /// observability (SessionStart only exists in interactive mode).
    #[serde(default)]
    pub keepalive: bool,

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
    Extension(_auth): Extension<AuthContext>,
    Json(req): Json<DispatchRequest>,
) -> impl IntoResponse {
    // Pipeline-step + long-running-chat are mutually exclusive intents.
    // auto:true completes per turn; keepalive:true never auto-exits.
    if req.auto && req.keepalive {
        return (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(
                    "auto and keepalive are mutually exclusive — \
                     pipeline steps use auto, long-running chat uses keepalive"
                ),
            }),
        );
    }

    let workspace_label = req
        .workspace
        .clone()
        .unwrap_or_else(|| req.agent.clone());

    // Resume: resolve an existing (agent, workspace) session and reuse
    // its slug/branch. With no match, error clearly per the spec.
    let resume_outcome = if req.resume {
        let worktrees_root = crate::supervisor::helpers::worktrees_dir();
        match resolve_resume(
            &req.agent,
            &workspace_label,
            req.harness.as_deref(),
            &worktrees_root,
        ) {
            Ok(o) => Some(o),
            Err(ResumeResolveError::Status { status, message }) => {
                return (
                    status,
                    Json(MeshResponse {
                        ok: false,
                        data: serde_json::json!(message),
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
        .or_else(|| resume_outcome.as_ref().map(|s| s.slug.clone()))
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
        .or_else(|| resume_outcome.as_ref().map(|s| s.branch.clone()))
        .unwrap_or_else(|| format!("job/{slug}"));
    let resume_harness = resume_outcome.as_ref().map(|o| o.session.harness.clone());
    let resume_session_id = resume_outcome.as_ref().map(|o| o.session.session_id.clone());
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
        // On resume, pin the harness recorded in session.json so the
        // executor's `build_resume_args` resolves the same adapter the
        // session was opened with. Caller-provided override still wins;
        // the harness-mismatch check above rejected conflicts already.
        harness: req.harness.or(resume_harness),
        depends_on: req.depends_on,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: req.vars,
        spec_content: req.spec,
        step_specs: None,
        headless: req.auto,
        keepalive: req.keepalive,
        prompt: req.prompt,
        resume: req.resume,
        resume_session_id,
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
/// `resume: true`. Carries the worktree slug/branch plus the
/// `.ship/session.json` record needed to build the harness-specific
/// resume command downstream.
pub(crate) struct ResumeOutcome {
    pub slug: String,
    pub branch: String,
    pub session: ship_platform::services::harnesses::session_file::SessionFile,
}

#[derive(Debug)]
pub(crate) enum ResumeResolveError {
    Status {
        status: StatusCode,
        message: String,
    },
}

/// Locate the prior session for `(agent, workspace)`:
/// 1. Use the sessions DB to find the slug + branch (most-recent match).
/// 2. Read `<worktree>/.ship/session.json` for the harness-native id.
/// 3. Verify the requested harness (when provided) matches the session.
/// 4. Verify the resolved harness adapter actually supports `build_resume_args`.
///
/// Every failure path returns a typed error the HTTP handler renders as
/// a 400 (the spec forbids silent fresh launches).
fn resolve_resume(
    agent: &str,
    workspace: &str,
    requested_harness: Option<&str>,
    worktrees_root: &std::path::Path,
) -> Result<ResumeOutcome, ResumeResolveError> {
    let sessions = ship_substrate::db::session::list_sessions_db(Some(workspace), 100)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("resume lookup failed: {e}"),
        })?;
    let hit = sessions
        .into_iter()
        .find(|s| s.agent_id.as_deref() == Some(agent))
        .ok_or_else(|| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "resume: no existing session for agent `{agent}` in workspace `{workspace}`"
            ),
        })?;

    let work_dir = worktrees_root.join(&hit.id);
    let session_file = load_and_validate_session_file(&work_dir, &hit.id, requested_harness)?;

    Ok(ResumeOutcome {
        slug: hit.id,
        branch: hit.branch,
        session: session_file,
    })
}

/// Worktree-local part of the resume resolver: reads `.ship/session.json`,
/// checks the harness matches `requested_harness` (when supplied), and
/// confirms the adapter's `build_resume_args` produces a valid command for
/// the recorded session id. Returns the parsed session file on success.
///
/// Split out so unit tests can exercise the file-/adapter-validation
/// branches without seeding the sessions DB.
fn load_and_validate_session_file(
    work_dir: &std::path::Path,
    slug: &str,
    requested_harness: Option<&str>,
) -> Result<ship_platform::services::harnesses::session_file::SessionFile, ResumeResolveError> {
    let session_file = ship_platform::services::harnesses::session_file::read_session_file(work_dir)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!("resume: failed to read session file: {e}"),
        })?
        .ok_or_else(|| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "resume: no .ship/session.json in worktree `{}` — the prior run did not capture a harness session id",
                work_dir.display()
            ),
        })?;

    if let Some(req_h) = requested_harness
        && req_h != session_file.harness
    {
        return Err(ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "resume: harness mismatch — request asked for `{req_h}` but the session was opened with `{}`",
                session_file.harness
            ),
        });
    }

    // Verify the adapter actually supports resume before promising the
    // caller it'll work. Gemini/Cursor will fail here; the dispatch path
    // then never hands off to the executor.
    let adapter = ship_platform::services::harness_adapter::adapter_for(&session_file.harness)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!("resume: {e}"),
        })?;
    let profile = ship_platform::services::dispatch_ports::LaunchProfile {
        harness: Some(session_file.harness.clone()),
        persist_session: true,
        ..Default::default()
    };
    adapter
        .build_resume_args(slug, &session_file.session_id, &profile)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!("resume: {e}"),
        })?;

    Ok(session_file)
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
            scopes: vec![crate::auth::Scope::Full],
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
            keepalive: false,
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

    use ship_platform::services::harnesses::session_file::write_session_file;

    #[test]
    fn load_session_file_missing_returns_400() {
        let tmp = TempDir::new().unwrap();
        let err = load_and_validate_session_file(tmp.path(), "some-slug", None)
            .expect_err("absent session.json must 400");
        match err {
            ResumeResolveError::Status { status, message } => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert!(
                    message.contains("no .ship/session.json"),
                    "message was: {message}"
                );
            }
        }
    }

    #[test]
    fn load_session_file_harness_mismatch_returns_400() {
        let tmp = TempDir::new().unwrap();
        write_session_file(tmp.path(), "claude", "sess-1", "rust-lane").unwrap();
        let err = load_and_validate_session_file(tmp.path(), "slug-x", Some("codex"))
            .expect_err("harness mismatch must 400");
        match err {
            ResumeResolveError::Status { status, message } => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert!(
                    message.contains("harness mismatch"),
                    "message was: {message}"
                );
            }
        }
    }

    #[test]
    fn load_session_file_unsupported_harness_returns_400() {
        // Gemini/cursor: session.json on disk but adapter refuses resume.
        // Dispatch must surface a 400 — never silently fresh-launch.
        let tmp = TempDir::new().unwrap();
        write_session_file(tmp.path(), "gemini", "anything", "rust-lane").unwrap();
        let err = load_and_validate_session_file(tmp.path(), "slug-x", None)
            .expect_err("gemini resume must 400");
        match err {
            ResumeResolveError::Status { status, message } => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert!(
                    message.contains("does not support session resume"),
                    "message was: {message}"
                );
            }
        }
    }

    #[test]
    fn load_session_file_happy_path_returns_record() {
        let tmp = TempDir::new().unwrap();
        write_session_file(tmp.path(), "claude", "sess-9", "rust-lane").unwrap();
        let s = load_and_validate_session_file(tmp.path(), "slug-x", Some("claude"))
            .expect("claude resume must validate");
        assert_eq!(s.harness, "claude");
        assert_eq!(s.session_id, "sess-9");
        assert_eq!(s.agent, "rust-lane");
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
            keepalive: false,
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
            keepalive: false,
        };
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
