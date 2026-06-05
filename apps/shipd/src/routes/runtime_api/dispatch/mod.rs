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

mod request;
mod resume;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_typed_result;

pub use request::DispatchRequest;
use resume::{ResumeResolveError, resolve_resume};
use request::{validate_axes, billing_warning_for_axes};

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionStartedPayload,
};
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, event_types};

use crate::auth::AuthContext;
use crate::routes::rest_api::{ApiState, MeshResponse};

/// POST /api/v1/agent_sessions/dispatch
///
/// Agent-centric single-agent dispatch. Rejects payloads carrying a
/// `pipeline` field (those belong at `POST /api/v1/pipelines/run`).
pub async fn dispatch_agent_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<DispatchRequest>,
) -> impl IntoResponse {
    // Per-project enforcement teeth: an MCP session bound to project A
    // cannot dispatch work into project B. Trusted (CLI/SDK) callers and
    // unbound sessions pass through. See auth::identity. This guard lives in
    // the REST handler because it needs the request's AuthContext; the
    // transport-neutral core below is shared with the ConnectRPC path.
    if let Some(message) =
        crate::auth::identity::mcp_workspace_violation(&auth, req.workspace.as_deref())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(message),
            }),
        );
    }

    dispatch_agent_session_core(&state, req).await
}

/// Transport-neutral core of single-agent dispatch — shared by the REST
/// handler above and the ConnectRPC `CommandOpsService`. Behavior is verbatim;
/// only the entry shape differs (owned extractors vs a typed request).
pub(crate) async fn dispatch_agent_session_core(
    state: &ApiState,
    req: DispatchRequest,
) -> (StatusCode, Json<MeshResponse>) {
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
    pipeline_run_id: &str,
) -> (StatusCode, Json<MeshResponse>) {
    let payload =
        match ship_pipeline_runtime::orchestrator::pipeline_loader::ensure_pipeline(payload) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(MeshResponse { ok: false, data: serde_json::json!(e) }),
                );
            }
        };

    let envelope = match EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, pipeline_run_id, &payload)
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
                    "pipeline_run_id": pipeline_run_id,
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
