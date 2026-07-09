//! Transport-neutral agent-session dispatch core.
//!
//! `dispatch_agent_session_core` is the shared logic behind the Connect
//! `AgentsService.Dispatch` wire method (and the now-removed REST
//! `POST /api/v1/agent_sessions/dispatch` handler). It validates the
//! request, resolves resume state, builds the `dispatch.requested` event,
//! persists it, and returns a structured result.
//!
//! Daemon-specific state (kernel, origin URL, worktrees path) is injected via
//! [`AgentDispatchPort`] so shipd provides a concrete adapter while this module
//! remains daemon-free.

use async_trait::async_trait;
use connectrpc::RequestContext;

use agent_def::canonical_session::{SessionStarted, event_types as canonical_session_event_types};
use agent_def::dispatch_intent::{DispatchRequestedPayload, event_types as dispatch_event_types};
use ship_substrate::events::EventEnvelope;

use super::dispatch_request::{
    DispatchRequest, billing_warning_for_axes, validate_axes, validate_dispatch_request,
};
#[cfg(feature = "harness-drivers")]
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
    ///
    /// `None` when the daemon has no harness-driver backend wired (the
    /// floki path of ship-standard builds; the harness-only `resume: true`
    /// flow refuses in that case at the wire boundary). The dispatch
    /// core's resume branch is `#[cfg(feature = "harness-drivers")]`-gated,
    /// so the only producer of `None` here is shipd's NoopDispatch port in
    /// `--no-default-features` builds. Production shipd/helmd always
    /// supply `Some`.
    #[cfg(feature = "harness-drivers")]
    fn worktrees_dir(&self) -> Option<std::path::PathBuf>;

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
    // Shape validation FIRST — before any side effect (resume resolve,
    // persist_and_route, event emits). The "agent is in the index" check is
    // enforced at the wire boundary (rpc_wire::dispatch) since it needs the
    // ship_dir to consult the index.
    if let Err(message) = validate_dispatch_request(&req) {
        return Err(DispatchCoreError::BadRequest(message));
    }
    if let Err(message) = validate_axes(&req) {
        return Err(DispatchCoreError::BadRequest(message));
    }

    // The slug-fallback label — the dispatch core derives
    // `<agent>-<ulid>` when the caller omits an explicit slug. The
    // field-10 `workspace` slug label is dead and was removed from
    // the wire; the resume resolver's `workspace` argument is a
    // legacy string key, not a real space id (the comment on
    // resolve_resume says so), so it gets the empty default.
    let slug_label = req.agent.clone();

    // Resume is harness-only (the resume target is always a prior coding-agent
    // session — a floki session has nothing to resume). The sligntly-
    // downcasted `ResumeFields` projection lives here so the function body
    // can stay readable under both `harness-drivers` branches without
    // referring to the gated `ResumeOutcome` type from the off-branch.
    struct ResumeFields {
        slug: String,
        branch: String,
        resume_harness: Option<String>,
        resume_session_id: Option<String>,
    }
    #[cfg(feature = "harness-drivers")]
    let resume: Option<ResumeFields> = if req.resume {
        let worktrees_root = port.worktrees_dir().ok_or_else(|| {
            DispatchCoreError::Internal(
                "resume: no worktree root available — dispatch backend has no \
                 harness-driver support (build shipd with --features ship-code)"
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
    #[cfg(not(feature = "harness-drivers"))]
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
        // Per-call fork source: the wire's `repo_path` field 16 (empty
        // = daemon default) and `base_ref` field 17 (empty = that
        // repo's HEAD). Consumed in-call by `GitWorktreeIsolation::prepare`
        // — never persisted, never exported to env.
        repo_path: opt_nonempty(req.repo_path.as_deref()),
        base_ref: opt_nonempty(req.base_ref.as_deref()),
        source_spec_path: req.source_spec_path,
        // Ship-standard is git-free by ruling (decisions-2026-07-09 §"Git
        // surface"). The floki dispatch closure ignores `baseline_sha`;
        // a coding-agent dispatch captures the SHIP-git HEAD via
        // `resolve_project_head_sha` only when the harness-driver stack is
        // wired. With `harness-drivers` off, `baseline_sha = None` is the
        // intended design — NOT a regression.
        #[cfg(feature = "harness-drivers")]
        baseline_sha: resolve_project_head_sha(),
        #[cfg(not(feature = "harness-drivers"))]
        baseline_sha: None,
        open_tab: req.open_tab,
        invocation: req.invocation,
        keepalive: req.keepalive,
        prompt: req.prompt,
        spec: req.spec,
        resume: req.resume,
        resume_session_id,
        depends_on: req.depends_on,
        vars: req.vars,
        parent_pipeline_run_id: None,
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
#[cfg(feature = "harness-drivers")]
fn resolve_project_head_sha() -> Option<String> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    ship_git::read::resolve_head_sha(project_root)
}

/// Normalize a per-call string option: `Some("")` collapses to `None` so
/// the wire's "empty = default" convention survives the JSON round-trip
/// without a sentinel value. Used for `repo_path` / `base_ref` on the
/// dispatch payload.
fn opt_nonempty(s: Option<&str>) -> Option<String> {
    s.filter(|s| !s.is_empty()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    //! Validation-gate tests for `dispatch_agent_session_core`. The point of
    //! these is the wire-contract claim c-2026-07-05T12-33-00-000Z-a1b2c3d4:
    //! a malformed request must be rejected with `BadRequest` BEFORE any
    //! event is emitted — the test doubles below count `persist_and_route`
    //! calls and assert they stay at zero for every rejected request.

    use super::*;
    use crate::services::agents::dispatch_request::{DispatchRequest, default_invocation};
    use agent_def::InvocationMode;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// `AgentDispatchPort` test double that counts `persist_and_route` calls
    /// and rejects nothing — exercises the real validation gate in
    /// `dispatch_agent_session_core`. The other accessors are not exercised
    /// on the rejected paths (validation short-circuits first); we provide
    /// trivial impls so the type is constructible.
    struct CountingPort {
        persist_calls: AtomicUsize,
    }
    #[async_trait]
    impl AgentDispatchPort for CountingPort {
        async fn persist_and_route(&self, _envelope: &EventEnvelope) -> anyhow::Result<()> {
            self.persist_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn daemon_origin(&self) -> String {
            "http://127.0.0.1:0".to_string()
        }
        #[cfg(feature = "harness-drivers")]
        fn worktrees_dir(&self) -> Option<std::path::PathBuf> {
            Some(std::path::PathBuf::from("/tmp"))
        }
    }
    fn req_with(
        agent: &str,
        spec: Option<&str>,
        prompt: Option<&str>,
        slug: Option<&str>,
    ) -> DispatchRequest {
        DispatchRequest {
            agent: agent.to_string(),
            spec: spec.map(str::to_string),
            prompt: prompt.map(str::to_string),
            resume: false,
            harness: None,
            model: None,
            open_tab: true,
            invocation: default_invocation(),
            keepalive: true,
            slug: slug.map(str::to_string),
            branch: None,
            source_spec_path: None,
            depends_on: None,
            vars: None,
            repo_path: None,
            base_ref: None,
        }
    }

    /// The headline test (c-2026-07-05T12-33-00-000Z-a1b2c3d4): a
    /// literal `{}` shape — empty agent, no spec, no prompt, no slug —
    /// must be rejected as `BadRequest` and must NOT emit a single event.
    #[tokio::test]
    async fn empty_request_rejected_with_no_side_effects() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        let req = req_with("", None, None, None);
        // Avoid `expect_err` (which needs `Debug` on the Ok arm) — match the
        // outcome and assert the Err variant instead.
        let outcome = dispatch_agent_session_core(&port, req).await;
        let err = match outcome {
            Ok(_) => panic!("empty request must be rejected, got Ok"),
            Err(e) => e,
        };
        match err {
            DispatchCoreError::BadRequest(m) => {
                assert!(m.contains("`agent` is required"), "got: {m}");
            }
            other => panic!("expected BadRequest, got {other:?}"),
        }
        assert_eq!(
            port.persist_calls.load(Ordering::SeqCst),
            0,
            "empty request must not emit any event"
        );
    }

    #[tokio::test]
    async fn missing_agent_rejected_no_side_effects() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        let req = req_with("", Some("spec body"), None, None);
        let outcome = dispatch_agent_session_core(&port, req).await;
        let err = match outcome {
            Ok(_) => panic!("empty agent must be rejected, got Ok"),
            Err(e) => e,
        };
        assert!(matches!(err, DispatchCoreError::BadRequest(_)));
        assert_eq!(port.persist_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn both_inputs_blank_rejected_no_side_effects() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        // Agent is set, slug is None — passes the first two checks; both
        // spec and prompt blank trips the third check.
        let req = req_with("rust-lane", None, None, None);
        let outcome = dispatch_agent_session_core(&port, req).await;
        let err = match outcome {
            Ok(_) => panic!("blank spec+prompt must be rejected, got Ok"),
            Err(e) => e,
        };
        match err {
            DispatchCoreError::BadRequest(m) => assert!(m.contains("at least one of"), "got: {m}"),
            other => panic!("expected BadRequest, got {other:?}"),
        }
        assert_eq!(port.persist_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn bad_slug_rejected_no_side_effects() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        let req = req_with("rust-lane", Some("spec"), None, Some("-bad-slug"));
        let outcome = dispatch_agent_session_core(&port, req).await;
        let err = match outcome {
            Ok(_) => panic!("bad slug must be rejected, got Ok"),
            Err(e) => e,
        };
        match err {
            DispatchCoreError::BadRequest(m) => assert!(m.contains("does not match"), "got: {m}"),
            other => panic!("expected BadRequest, got {other:?}"),
        }
        assert_eq!(port.persist_calls.load(Ordering::SeqCst), 0);
    }

    /// Sanity: a well-formed request DOES proceed past the validation
    /// gate and the actor-side helper at least runs through (the test
    /// double counts the `persist_and_route` call so the gate is
    /// proven non-blocking on the happy path).
    #[tokio::test]
    async fn well_formed_request_proceeds_past_validation() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        let req = req_with(
            "rust-lane",
            Some("spec body"),
            None,
            Some("dispatch-consolidation"),
        );
        // We don't care if the full path succeeds (it depends on a real
        // session) — only that the validation gate is permissive and the
        // dispatch.requested event lands.
        let _ = dispatch_agent_session_core(&port, req).await;
        assert!(
            port.persist_calls.load(Ordering::SeqCst) >= 1,
            "well-formed request must emit at least the dispatch.requested event"
        );
    }
}
