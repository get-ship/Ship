//! Transport-neutral agent-session dispatch core.
//!
//! `dispatch_agent_session_core` is the shared logic behind the Connect
//! `AgentsService.Dispatch` wire method (and the now-removed REST
//! `POST /api/v1/agent_sessions/dispatch` handler). It validates the
//! request, resolves resume state, builds the `agent.session.started` event,
//! persists it, and returns a structured result.
//!
//! Daemon-specific state (kernel, origin URL, worktrees path) is injected via
//! [`AgentDispatchPort`] so shipd provides a concrete adapter while this module
//! remains daemon-free.

use async_trait::async_trait;
use connectrpc::RequestContext;

use ship_substrate::events::EventEnvelope;
use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionStartedPayload,
};

use super::dispatch_request::{
    DispatchRequest, billing_warning_for_axes, validate_axes,
};
use super::dispatch_resume::{ResumeResolveError, resolve_resume};

// ── Port trait ────────────────────────────────────────────────────────────────

/// Daemon-specific capabilities the dispatch core needs. Implemented by shipd;
/// declared here so ship-agents stays daemon-free.
#[async_trait]
pub trait AgentDispatchPort: Send + Sync {
    /// Persist an event envelope and route it to the kernel.
    async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()>;

    /// Base URL the daemon is listening on (e.g. `http://localhost:7587`).
    fn daemon_origin(&self) -> String;

    /// Absolute path to the worktrees root directory.
    fn worktrees_dir(&self) -> std::path::PathBuf;

    /// Per-project workspace isolation guard. Returns a violation message when
    /// an MCP session bound to project A claims to act on project B; `None`
    /// when the caller is trusted or the workspace is consistent.
    ///
    /// This is an auth-context concern that the dispatch port exposes so the
    /// ConnectRPC dispatch handler can enforce it without importing shipd's
    /// `AuthContext` type directly. The REST handler enforces it before calling
    /// `dispatch_agent_session_core`.
    fn check_workspace_violation(
        &self,
        ctx: &RequestContext,
        claimed_workspace: Option<&str>,
    ) -> Option<String>;
}

// ── Result type ───────────────────────────────────────────────────────────────

/// Outcome of a successful single-agent dispatch.
pub struct DispatchOutcome {
    pub session_id: String,
    pub slug: String,
    pub branch: String,
    pub resume: bool,
    pub studio_url: String,
    pub billing_warning: Option<String>,
}

/// Structured error from dispatch core — callers map to HTTP/ConnectRPC status.
#[derive(Debug)]
pub enum DispatchCoreError {
    /// Invalid request (map to 400 / invalid_argument).
    BadRequest(String),
    /// Internal failure (map to 500 / internal).
    Internal(String),
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Transport-neutral core of single-agent dispatch. Shared by the REST
/// handler and the ConnectRPC `AgentsService.dispatch` wire method.
///
/// The `mcp_workspace_violation` guard is NOT performed here — it is an
/// auth-context concern that belongs at the transport boundary (REST handler
/// or rpc_wire). Callers must enforce it before invoking this function.
pub async fn dispatch_agent_session_core(
    port: &dyn AgentDispatchPort,
    req: DispatchRequest,
) -> Result<DispatchOutcome, DispatchCoreError> {
    // Validate the (open_tab × invocation × keepalive) matrix.
    if let Err(message) = validate_axes(&req) {
        return Err(DispatchCoreError::BadRequest(message));
    }

    let workspace_label = req
        .workspace
        .clone()
        .unwrap_or_else(|| req.agent.clone());

    // Resume: resolve an existing (agent, workspace) session and reuse
    // its slug/branch. With no match, error clearly per the spec.
    let resume_outcome = if req.resume {
        let worktrees_root = port.worktrees_dir();
        match resolve_resume(
            &req.agent,
            &workspace_label,
            req.harness.as_deref(),
            &worktrees_root,
        ) {
            Ok(o) => Some(o),
            Err(ResumeResolveError::Status { status, message }) => {
                return Err(if status == 400 {
                    DispatchCoreError::BadRequest(message)
                } else {
                    DispatchCoreError::Internal(message)
                });
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
        baseline_sha: resolve_project_head_sha(),
        parent_pipeline_run_id: None,
    };

    let envelope = EventEnvelope::new(
        agent_session_event_types::AGENT_SESSION_STARTED,
        &session_id,
        &payload,
    )
    .map(|e| e.with_actor_id("studio"))
    .map_err(|e| DispatchCoreError::Internal(e.to_string()))?;

    port.persist_and_route(&envelope)
        .await
        .map_err(|e| DispatchCoreError::Internal(e.to_string()))?;

    let studio_url = format!("{}/sessions/{}", port.daemon_origin(), slug);
    let billing_warning = billing_warning_for_axes(
        payload.invocation,
        payload.harness.as_deref(),
    );

    Ok(DispatchOutcome {
        session_id,
        slug,
        branch,
        resume: req.resume,
        studio_url,
        billing_warning,
    })
}

// ── Internal helper ───────────────────────────────────────────────────────────

/// Resolve HEAD SHA of the active project root. Used at dispatch time to
/// capture the session's baseline — the point every future worktree branches
/// from. `None` when the project dir can't be resolved or git fails.
fn resolve_project_head_sha() -> Option<String> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    ship_code::runner::observe::resolve_head_sha(project_root)
}
