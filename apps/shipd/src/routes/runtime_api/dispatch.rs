//! Single-agent dispatch — `POST /api/v1/agent_sessions/dispatch`.
//!
//! Agent-centric: requires `agent`. Emits `agent.session.started` (the
//! dispatch primitive). The pipeline-run endpoint (see `pipeline_run.rs`)
//! emits `pipeline.run.started` for multi-step composition; the pipeline
//! runner downstream emits one `agent.session.started` per step with
//! `parent_pipeline_run_id` set.
//!
//! Three orthogonal axes carry the dispatch intent. `auto: bool` is
//! gone — it conflated three independent concerns (UX/attach, harness
//! invocation, lifecycle) into one boolean. See the agent-primitive
//! inversion plan in `.artifacts/agents-arc/discussions/` for the
//! rationale.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionStartedPayload, InvocationMode,
};
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

    // ── The three orthogonal axes (replace the old single `auto: bool`) ──
    //
    // Each defaults to its no-surprise value so simple POSTs still work:
    //   { "agent": "rust-lane" }
    // = interactive PTY tab, keepalive. The same shape `ship use` opens.

    /// Does this dispatch open an interactive tab / attachable PTY
    /// surface? Default true. Set false for background / dispatched
    /// jobs that the operator will observe later (CLI returns the
    /// session_id + studio_url; nothing attaches to the operator's
    /// terminal).
    #[serde(default = "default_open_tab")]
    pub open_tab: bool,

    /// How the harness is invoked:
    ///   - pty (default): PTY-attached harness invocation (claude TUI,
    ///     codex TUI, ...). Subscription-billed for claude.
    ///   - print: one-shot non-interactive mode (claude -p,
    ///     codex exec). API-billed for claude post-2026-06-19.
    ///   - stream: server-mode (codex app-server, opencode REST).
    ///     Only codex/opencode today.
    #[serde(default = "default_invocation")]
    #[schema(value_type = String, example = "pty")]
    pub invocation: InvocationMode,

    /// Does the harness stay alive after the current work, awaiting
    /// the next operator push? Default true (matches the chat/worker
    /// expectation). Required for `POST /api/v1/agent_sessions/{slug}/push`.
    /// Mutually exclusive with `invocation = print` (print exits on
    /// completion).
    #[serde(default = "default_keepalive")]
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

fn default_open_tab() -> bool { true }
fn default_keepalive() -> bool { true }
fn default_invocation() -> InvocationMode { InvocationMode::Pty }

/// Validate the (open_tab × invocation × keepalive) triple. Returns
/// an operator-readable error when the combination is invalid.
///
/// The matrix is sparse because print mode is structurally one-shot
/// and has no PTY to attach to. Stream mode has no implemented
/// attach surface today.
fn validate_axes(req: &DispatchRequest) -> Result<(), String> {
    match req.invocation {
        InvocationMode::Print => {
            if req.open_tab {
                return Err(
                    "invocation=print is incompatible with open_tab=true \
                     (print mode has no attachable PTY). \
                     Set open_tab=false or change invocation to pty."
                        .to_string(),
                );
            }
            if req.keepalive {
                return Err(
                    "invocation=print is incompatible with keepalive=true \
                     (print mode exits on completion; cannot push more turns). \
                     Set keepalive=false or change invocation to pty."
                        .to_string(),
                );
            }
        }
        InvocationMode::Stream => {
            if req.open_tab {
                return Err(
                    "invocation=stream is incompatible with open_tab=true \
                     (server-mode attach not yet implemented). \
                     Set open_tab=false."
                        .to_string(),
                );
            }
        }
        InvocationMode::Pty => {}
    }
    Ok(())
}

/// Build the operator-facing billing warning when one applies. Today
/// the only triggering combination is `invocation=print` with claude
/// (post-2026-06-19, `claude -p` is billed at API rates regardless of
/// subscription).
fn billing_warning_for_axes(
    invocation: InvocationMode,
    harness: Option<&str>,
) -> Option<String> {
    if invocation != InvocationMode::Print {
        return None;
    }
    if harness.unwrap_or("claude") != "claude" {
        return None;
    }
    Some(
        "claude -p is API-billed (subscription plan does not apply). \
         Use invocation=pty for subscription billing."
            .to_string(),
    )
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
    // Validate the (open_tab × invocation × keepalive) matrix.
    if let Err(message) = validate_axes(&req) {
        return (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(message),
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
            // Collision-free suffix from the ULID's random tail — a
            // `[..8]` prefix is pure timestamp and collides for rapid
            // same-workspace dispatches. See gen_slug_suffix.
            format!("{}-{}", workspace_label, ship_substrate::gen_slug_suffix())
        });
    let branch = req
        .branch
        .clone()
        .or_else(|| resume_outcome.as_ref().map(|s| s.branch.clone()))
        .unwrap_or_else(|| format!("job/{slug}"));
    let resume_harness = resume_outcome.as_ref().map(|o| o.session.harness.clone());
    let resume_session_id = resume_outcome.as_ref().map(|o| o.session.session_id.clone());
    let session_id = ship_substrate::gen_ulid();

    // Build the agent-session primitive payload. Pipeline composition
    // (multi-step) uses pipeline_run.rs which still emits
    // PIPELINE_RUN_STARTED + sets parent_pipeline_run_id on each step's
    // agent.session.started. This endpoint is for bare single-agent
    // dispatch; parent_pipeline_run_id is always None here.
    let payload = AgentSessionStartedPayload {
        session_id: session_id.clone(),
        slug: slug.clone(),
        agent: req.agent.clone(),
        harness: req.harness.or(resume_harness),
        model: req.model,
        workspace: workspace_label.clone(),
        branch: branch.clone(),
        source_spec_path: req.source_spec_path,
        open_tab: req.open_tab,
        invocation: req.invocation,
        keepalive: req.keepalive,
        prompt: req.prompt,
        spec: req.spec,
        resume: req.resume,
        resume_session_id,
        depends_on: req.depends_on,
        vars: req.vars,
        baseline_sha: super::resolve_project_head_sha(),
        parent_pipeline_run_id: None,
    };

    let envelope = match EventEnvelope::new(
        agent_session_event_types::AGENT_SESSION_STARTED,
        &session_id,
        &payload,
    )
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

    if let Err(e) = state.persist_and_route(&envelope).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        );
    }

    let studio_url = format!("{}/sessions/{}", state.daemon_origin(), slug);
    // billing_warning_for inspects (invocation, harness) — both Copy/clonable
    // or already cloned into the payload above. Reconstruct a minimal view
    // rather than borrow `req` which was partially moved into the payload.
    let warning = billing_warning_for_axes(
        payload.invocation,
        payload.harness.as_deref(),
    );

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({
                "session_id": session_id,
                "slug": slug,
                "branch": branch,
                "resume": req.resume,
                "studio_url": studio_url,
                "billing_warning": warning,
            }),
        }),
    )
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
    use crate::auth::Scope;
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

    fn bare_req(agent: &str, workspace: &str) -> DispatchRequest {
        DispatchRequest {
            agent: agent.into(),
            spec: None,
            prompt: None,
            resume: false,
            harness: None,
            model: None,
            open_tab: true,
            invocation: InvocationMode::Pty,
            keepalive: true,
            workspace: Some(workspace.into()),
            slug: None,
            branch: None,
            source_spec_path: None,
            depends_on: None,
            vars: None,
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn single_agent_dispatch_returns_session_id_slug_studio_url() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let req = bare_req("rust-lane", "solo");
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let val = body_json(resp).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        let data = &val["data"];
        assert!(data["slug"].as_str().unwrap().starts_with("solo-"));
        assert!(data["session_id"].as_str().is_some(), "session_id must be present");
        assert!(
            data["studio_url"]
                .as_str()
                .unwrap_or("")
                .contains("/sessions/"),
            "studio_url must point at the session route"
        );
        assert!(
            data["billing_warning"].is_null(),
            "PTY dispatch must not return a billing_warning"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_with_invocation_print_returns_billing_warning_for_claude() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let mut req = bare_req("rust-lane", "billing-test");
        req.invocation = InvocationMode::Print;
        req.open_tab = false;
        req.keepalive = false;
        req.harness = Some("claude".to_string());
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let val = body_json(resp).await;
        let warning = val["data"]["billing_warning"].as_str().unwrap_or("");
        assert!(
            warning.contains("API-billed"),
            "billing_warning must surface API-billed for claude -p, got: {warning}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_with_invocation_print_no_warning_for_non_claude() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let mut req = bare_req("rust-lane", "billing-test");
        req.invocation = InvocationMode::Print;
        req.open_tab = false;
        req.keepalive = false;
        req.harness = Some("codex".to_string());
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let val = body_json(resp).await;
        assert!(
            val["data"]["billing_warning"].is_null(),
            "non-claude print mode must not surface a billing warning"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_rejects_print_with_open_tab() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let mut req = bare_req("rust-lane", "v");
        req.invocation = InvocationMode::Print;
        req.open_tab = true;
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let val = body_json(resp).await;
        let msg = val["data"].as_str().unwrap_or("");
        assert!(
            msg.contains("print") && msg.contains("attachable PTY"),
            "error must explain the conflict, got: {msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_rejects_print_with_keepalive() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let mut req = bare_req("rust-lane", "v");
        req.invocation = InvocationMode::Print;
        req.open_tab = false;
        req.keepalive = true;
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let val = body_json(resp).await;
        let msg = val["data"].as_str().unwrap_or("");
        assert!(
            msg.contains("print") && msg.contains("keepalive"),
            "error must explain print exits on completion, got: {msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_rejects_stream_with_open_tab() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let mut req = bare_req("codex", "v");
        req.invocation = InvocationMode::Stream;
        req.open_tab = true;
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let val = body_json(resp).await;
        let msg = val["data"].as_str().unwrap_or("");
        assert!(
            msg.contains("stream") && msg.contains("attach"),
            "error must explain server-mode attach unsupported, got: {msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatch_background_pty_keepalive_combination_valid() {
        // open_tab=false + invocation=pty + keepalive=true is the
        // canonical background subscription worker case. The old
        // `auto && keepalive → 400` mutex used to reject this; the
        // new model accepts it as a first-class combination.
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let mut req = bare_req("rust-lane", "bg");
        req.open_tab = false;
        req.invocation = InvocationMode::Pty;
        req.keepalive = true;
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "background PTY keepalive worker must dispatch"
        );
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
        let mut req = bare_req("rust-lane", "empty-ws");
        req.resume = true;
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
    async fn dispatch_accepts_prompt_and_returns_session() {
        let dir = TempDir::new().unwrap();
        let state = make_state(&dir);
        let mut req = bare_req("rust-lane", "p");
        req.prompt = Some("hello".into());
        let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── DispatchRequest deserialization defaults ──────────────────

    #[test]
    fn minimal_request_deserializes_with_defaults() {
        // POST with just {"agent": "rust-lane"} must work — defaults
        // cover the three orthogonal axes. This is the shape every
        // shell, CLI, and MCP call relies on for the common case.
        let body = serde_json::json!({ "agent": "rust-lane" });
        let req: DispatchRequest = serde_json::from_value(body).expect("must accept minimal body");
        assert!(req.open_tab, "default open_tab=true (interactive)");
        assert!(req.keepalive, "default keepalive=true (chat shape)");
        assert_eq!(req.invocation, InvocationMode::Pty);
    }

    #[test]
    fn invocation_field_parses_print_and_stream() {
        for (raw, expected) in [
            ("\"pty\"", InvocationMode::Pty),
            ("\"print\"", InvocationMode::Print),
            ("\"stream\"", InvocationMode::Stream),
        ] {
            let body = format!(
                "{{\"agent\":\"rust-lane\",\"invocation\":{raw},\"open_tab\":false,\"keepalive\":false}}"
            );
            let req: DispatchRequest = serde_json::from_str(&body).expect("must parse");
            assert_eq!(req.invocation, expected);
        }
    }

    #[test]
    fn rejects_old_auto_field() {
        // `auto` is gone. A request that includes it should fail
        // deserialization (deny_unknown_fields), surfacing the rename
        // to operators rather than silently dropping the field.
        let body = serde_json::json!({
            "agent": "rust-lane",
            "auto": true,
        });
        let err = serde_json::from_value::<DispatchRequest>(body)
            .err()
            .expect("must reject the removed auto field");
        let msg = err.to_string();
        assert!(
            msg.contains("auto") || msg.contains("unknown field"),
            "expected auto-field rejection (the new model uses open_tab + invocation + keepalive), got: {msg}"
        );
    }
}
