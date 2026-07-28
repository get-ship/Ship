//! Transport-neutral coding-class dispatch core.
//!
//! `dispatch_code_session_core` is the shared logic behind the Connect
//! `CodeService.Dispatch` wire method (and the historical
//! `ship.ops.v1.AgentsService.Dispatch` op before the mount split). It
//! validates the request, resolves resume state, builds the
//! `dispatch.requested` event, persists it, and returns a structured
//! result.
//!
//! Daemon-specific state (kernel, origin URL, worktrees path) is injected
//! via [`AgentDispatchPort`] so shipd provides a concrete adapter while
//! this module remains daemon-free.
//!
//! The in-process (floki-class) counterpart lives in
//! `dispatch_core::dispatch_agent_session_core` and accepts the slim
//! `DispatchRequest { agent, prompt, model, slug }`. The two cores share
//! differ only in the request shape each accepts.
//!
//! The trait is daemon-free; [`async_trait`] was the historical
//! annotation but the `dispatch_code_session_core` is a plain `async fn`
//! (Rust 2024 — no attribute needed). The `AgentDispatchPort` trait
//! itself uses `#[async_trait]`, declared in `dispatch_core::AgentDispatchPort`.
use agent_schema::canonical_session::{SessionStarted, event_types as canonical_session_event_types};
use agent_schema::dispatch_intent::{DispatchRequestedPayload, event_types as dispatch_event_types};
use ship_substrate::events::EventEnvelope;

use super::code_dispatch_request::{
    CodeDispatchRequest, billing_warning_for_axes, validate_code_dispatch_request,
};
#[cfg(feature = "bridge")]
use super::dispatch_resume::{ResumeResolveError, resolve_resume};

// ── Port trait (re-export of the agent port) ─────────────────────────────────

/// Coding-class dispatch reuses the agent-domain [`AgentDispatchPort`]
/// trait — the two surfaces (in-process floki + coding harness) share
/// the same daemon-side state (kernel persist-and-route, daemon origin
/// for the `studio_url`, worktrees root for resume). The
/// `worktrees_dir` accessor is gated on `ship-code` because the
/// floki path of a ship-standard build doesn't expose it.
pub use super::dispatch_core::AgentDispatchPort;

// ── Result type (re-export) ──────────────────────────────────────────────────

/// Outcome of a successful coding-class dispatch. Same shape as
/// [`super::dispatch_core::DispatchOutcome`].
pub use super::dispatch_core::DispatchOutcome;

/// Structured error from dispatch core — callers map to HTTP/ConnectRPC status.
pub use super::dispatch_core::DispatchCoreError;

// ── Core logic ────────────────────────────────────────────────────────────────

/// Transport-neutral core of coding-class dispatch. Shared by the
/// `CodeService.dispatch` wire method (and the historical
/// `AgentsService.dispatch` op before the mount split).
///
/// The `mcp_workspace_violation` guard is NOT performed here — it is an
/// auth-context concern that belongs at the transport boundary. Callers
/// must enforce it before invoking this function.
pub async fn dispatch_code_session_core(
    port: &dyn AgentDispatchPort,
    req: CodeDispatchRequest,
) -> Result<DispatchOutcome, DispatchCoreError> {
    // Shape validation FIRST — before any side effect (resume resolve,
    // persist_and_route, event emits). The "agent is in the index" check is
    // enforced at the wire boundary (code_wire::dispatch) since it needs the
    // ship_dir to consult the index.
    if let Err(message) = validate_code_dispatch_request(&req) {
        return Err(DispatchCoreError::BadRequest(message));
    }

    // The slug-fallback label — the dispatch core derives
    // `<agent>-<ulid>` when the caller omits an explicit slug.
    let slug_label = req.agent.clone();

    // Resume is coding-class only (a floki session has nothing to
    // resume — the resume target is always a prior coding-agent session
    // with `.ship/session.json` on disk). The `ResumeFields` projection
    // lives here so the function body can stay readable under both
    // `ship-code` branches without referring to the gated
    // `ResumeOutcome` type from the off-branch.
    struct ResumeFields {
        slug: String,
        branch: String,
        resume_harness: Option<String>,
        resume_session_id: Option<String>,
    }
    #[cfg(feature = "bridge")]
    let resume: Option<ResumeFields> = if req.resume {
        let worktrees_root = port.worktrees_dir().ok_or_else(|| {
            DispatchCoreError::Internal(
                "resume: no worktree root available — dispatch backend has no \
                 harness-driver support (build shipd with --features bridge)"
                    .to_string(),
            )
        })?;
        match resolve_resume(
            &req.agent,
            // Empty `workspace` key — the legacy field-10 slug label is
            // dead; resume matches by `agent` only. The string stays a
            // label so the projection can still key on it for back-compat
            // with any pre-cutover session rows.
            "",
            req.harness.as_deref(),
            &worktrees_root,
        ) {
            Ok(o) => Some(ResumeFields {
                slug: o.slug,
                branch: o.branch,
                resume_harness: Some(o.session.harness),
                resume_session_id: Some(o.session.session_id),
            }),
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
    #[cfg(not(feature = "bridge"))]
    let resume: Option<ResumeFields> = None;

    let slug = req
        .slug
        .clone()
        .or_else(|| resume.as_ref().map(|r| r.slug.clone()))
        .unwrap_or_else(|| {
            // Collision-free suffix from the ULID's random tail — a
            // `[..8]` prefix is pure timestamp and collides for rapid
            // same-agent dispatches. See gen_slug_suffix.
            format!("{}-{}", slug_label, ship_substrate::gen_slug_suffix())
        });
    let branch = req
        .branch
        .clone()
        .or_else(|| resume.as_ref().map(|r| r.branch.clone()))
        .unwrap_or_else(|| format!("job/{slug}"));
    let resume_harness = resume.as_ref().and_then(|r| r.resume_harness.clone());
    let resume_session_id = resume
        .as_ref()
        .and_then(|r| r.resume_session_id.clone());
    let session_id = ship_substrate::gen_ulid();

    // Build the dispatch intent payload. This is the hard-failing spawn
    // trigger that drives the `service.agents` actor. It carries all
    // driver-edge fields (branch, repo_path, base_ref, source_spec_path,
    // baseline_sha) that MUST NOT appear in canonical session.* payloads.
    //
    // Pipeline composition (multi-step) uses pipeline_run.rs which emits
    // PIPELINE_RUN_STARTED + sets parent_pipeline_run_id on each step's
    // dispatch.requested. This endpoint is for bare single-agent dispatch;
    // parent_pipeline_run_id is always None here.
    let dispatch_payload = DispatchRequestedPayload {
        session_id: session_id.clone(),
        slug: slug.clone(),
        agent_id: req.agent.clone(),
        harness: req.harness.or(resume_harness),
        model: req.model,
        branch: branch.clone(),
        // Per-call fork source: the wire's `repo_path` field 13 (empty
        // = daemon default) and `base_ref` field 14 (empty = that
        // repo's HEAD). Consumed in-call by `GitWorktreeIsolation::prepare`
        // — never persisted, never exported to env.
        repo_path: opt_nonempty(req.repo_path.as_deref()),
        base_ref: opt_nonempty(req.base_ref.as_deref()),
        source_spec_path: req.source_spec_path,
        // Ship-standard is git-free by ruling (decisions-2026-07-09 §"Git
        // surface"). The floki dispatch closure ignores `baseline_sha`;
        // a coding-agent dispatch captures the SHIP-git HEAD via
        // `resolve_project_head_sha` only when the harness-driver stack is
        // wired. With `ship-code` off, `baseline_sha = None` is the
        // intended design — NOT a regression.
        #[cfg(feature = "bridge")]
        baseline_sha: resolve_project_head_sha(),
        #[cfg(not(feature = "bridge"))]
        baseline_sha: None,
        // Coding-class dispatch never opens a studio tab via the wire
        // (the `CodeDispatchRequest` proto has no `open_tab` field —
        // that's a floki-only concept). Keep the `false` default so the
        // payload projection stays consistent with the no-open-tab
        // pre-cutover behavior on the harness path.
        open_tab: false,
        invocation: req.invocation,
        keepalive: req.keepalive,
        prompt: req.prompt,
        spec: req.spec,
        resume: req.resume,
        resume_session_id,
        depends_on: req.depends_on,
        vars: None,
    };

    // dispatch.requested: the hard-failing spawn trigger. The service.agents
    // actor subscribes to this and owns the harness spawn atom.
    let dispatch_env = EventEnvelope::new(
        dispatch_event_types::DISPATCH_REQUESTED,
        &session_id,
        &dispatch_payload,
    )
    .map(|e| e.with_actor_id("studio"))
    .map_err(|e| DispatchCoreError::Internal(e.to_string()))?;

    port.persist_and_route(&dispatch_env)
        .await
        .map_err(|e| DispatchCoreError::Internal(e.to_string()))?;

    // Canonical session.started — feeds the canonical_sessions projection.
    // Best-effort: a canonical emit failure must NOT block the dispatch.
    // The dispatch.requested above is already in-flight; the spawn trigger
    // has been issued regardless of whether this audit row lands.
    // space_id: the cosmetic workspace label is not a real space id — leave
    // empty (consumers tolerate this via the default).
    let space_id = String::new();
    if let Ok(canonical_env) = EventEnvelope::new(
        canonical_session_event_types::SESSION_STARTED,
        &session_id,
        &SessionStarted {
            session_id: session_id.clone(),
            agent_id: dispatch_payload.agent_id.clone(),
            harness: dispatch_payload
                .harness
                .clone()
                .unwrap_or_else(|| "claude".into()),
            model: dispatch_payload.model.clone(),
            space_id,
            slug: Some(slug.clone()),
            input: dispatch_payload.prompt.clone(),
            parent: None,
        },
    ) {
        let canonical_env = canonical_env.with_actor_id("studio");
        // Best-effort: a canonical emit failure must not block the dispatch.
        let _ = port.persist_and_route(&canonical_env).await;
    }

    let studio_url = format!("{}/sessions/{}", port.daemon_origin(), slug);
    let billing_warning = billing_warning_for_axes(
        dispatch_payload.invocation,
        dispatch_payload.harness.as_deref(),
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
///
/// Gated: ship-standard is git-free by ruling (decisions-2026-07-09 §"Git
/// surface"), so the floki path carries `baseline_sha = None`. The
/// harness-driver stack reaches the SHIP-git leaf here; absent the
/// feature, the call site has `baseline_sha = None` and this function
/// is not compiled.
#[cfg(feature = "bridge")]
fn resolve_project_head_sha() -> Option<String> {
    let stack_root = ship_substrate::project::get_stack_root(None).ok()?;
    ship_git::read::resolve_head_sha(&stack_root)
}

/// Normalize a per-call string option: `Some("")` collapses to `None` so
/// the wire's "empty = default" convention survives the JSON round-trip
/// without a sentinel value. Used for `repo_path` / `base_ref` on the
/// dispatch payload.
fn opt_nonempty(s: Option<&str>) -> Option<String> {
    s.filter(|s| !s.is_empty()).map(str::to_string)
}
