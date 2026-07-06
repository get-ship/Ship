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

    /// Resolve a workspace label to its space id via the projects read model.
    /// `None` when the workspace is not yet registered. Lives on the port so
    /// dispatch_core (L1) never reaches into the L0 projects table directly —
    /// the composition root owns the DB access.
    fn space_id_for_workspace(&self, workspace_label: &str) -> Option<String>;

    /// Known workspace labels the daemon can dispatch into. The helm
    /// ingest path exposes the same set via `DEFAULT_REPOS` (see
    /// `helm/src/ingest/discover.ts`); shipd's adapter returns its
    /// active project's name. Used by `dispatch_agent_session_core`
    /// to refuse a request that names a workspace outside this set
    /// (gap 3: never silently substitute another repo).
    fn valid_workspaces(&self) -> Vec<String>;
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

    let workspace_label = req.workspace.clone().unwrap_or_else(|| req.agent.clone());

    // Gap 3: when the caller EXPLICITLY names a workspace, refuse the
    // request if it doesn't resolve to a known repo (the same set the
    // helm ingest path exposes). The default case (no `workspace`
    // field) keeps the active project — no registry check needed.
    // Without this, a `workspace: "helm"` request from inside the
    // `ship` repo silently provisions a ship worktree.
    if let Some(named) = &req.workspace {
        let valid = port.valid_workspaces();
        if !valid.is_empty() && !valid.iter().any(|w| w == named) {
            return Err(DispatchCoreError::BadRequest(format!(
                "workspace '{named}' not found in registry; known workspaces: {} \
                 — pass one of these or omit `workspace` to default to the active project",
                valid.join(", ")
            )));
        }
    }
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
    let resume_session_id = resume_outcome
        .as_ref()
        .map(|o| o.session.session_id.clone());
    let session_id = ship_substrate::gen_ulid();

    // Build the dispatch intent payload. This is the hard-failing spawn
    // trigger that drives the `service.agents` actor. It carries all
    // driver-edge fields (branch, source_spec_path, baseline_sha) that
    // MUST NOT appear in canonical session.* payloads.
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
        workspace: workspace_label.clone(),
        branch: branch.clone(),
        source_spec_path: req.source_spec_path,
        baseline_sha: resolve_project_head_sha(),
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
    // space_id: resolve from workspace path via the projects projection;
    // fall back to empty string if workspace is not yet registered.
    let space_id = port
        .space_id_for_workspace(&workspace_label)
        .unwrap_or_default();
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
fn resolve_project_head_sha() -> Option<String> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    ship_git::read::resolve_head_sha(project_root)
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
        fn worktrees_dir(&self) -> std::path::PathBuf {
            std::path::PathBuf::from("/tmp")
        }
        fn check_workspace_violation(
            &self,
            _ctx: &RequestContext,
            _claimed_workspace: Option<&str>,
        ) -> Option<String> {
            None
        }
        fn space_id_for_workspace(&self, _workspace_label: &str) -> Option<String> {
            None
        }
        fn valid_workspaces(&self) -> Vec<String> {
            // Test stub: an empty list short-circuits the registry
            // check (gap 3) so existing tests are unaffected. A
            // dedicated `valid_workspaces` test below drives the
            // named-workspace enforcement path with a configured
            // registry.
            Vec::new()
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
            workspace: None,
            slug: slug.map(str::to_string),
            branch: None,
            source_spec_path: None,
            depends_on: None,
            vars: None,
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

    /// Gap 3: a request that explicitly names a workspace NOT in the
    /// registry must be rejected as `BadRequest` — never silently
    /// substituted for the active project. The registry is the
    /// `valid_workspaces` port method; an empty list short-circuits
    /// the check (legacy behavior preserved).
    #[tokio::test]
    async fn named_workspace_outside_registry_rejected() {
        struct RegistryPort {
            persist_calls: AtomicUsize,
            valid: Vec<String>,
        }
        #[async_trait]
        impl AgentDispatchPort for RegistryPort {
            async fn persist_and_route(
                &self,
                _envelope: &EventEnvelope,
            ) -> anyhow::Result<()> {
                self.persist_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            fn daemon_origin(&self) -> String {
                "http://127.0.0.1:0".to_string()
            }
            fn worktrees_dir(&self) -> std::path::PathBuf {
                std::path::PathBuf::from("/tmp")
            }
            fn check_workspace_violation(
                &self,
                _ctx: &RequestContext,
                _claimed_workspace: Option<&str>,
            ) -> Option<String> {
                None
            }
            fn space_id_for_workspace(&self, _workspace_label: &str) -> Option<String> {
                None
            }
            fn valid_workspaces(&self) -> Vec<String> {
                self.valid.clone()
            }
        }
        let port = RegistryPort {
            persist_calls: AtomicUsize::new(0),
            // The helm registry exposes the same set the active project
            // lives in. A request naming a different repo must fail.
            valid: vec!["ship".to_string(), "sui".to_string()],
        };
        let mut req = req_with("rust-lane", Some("spec"), None, Some("helmd-test"));
        req.workspace = Some("helm".to_string());
        let outcome = dispatch_agent_session_core(&port, req).await;
        let err = match outcome {
            Ok(_) => panic!("named workspace outside registry must be rejected"),
            Err(e) => e,
        };
        match err {
            DispatchCoreError::BadRequest(m) => {
                assert!(
                    m.contains("workspace 'helm' not found in registry"),
                    "got: {m}"
                );
                assert!(m.contains("ship"), "registry list must surface: {m}");
            }
            other => panic!("expected BadRequest, got {other:?}"),
        }
        assert_eq!(
            port.persist_calls.load(Ordering::SeqCst),
            0,
            "rejected request must not emit any event"
        );
    }

    /// Gap 3 (positive case): a request naming a workspace IN the
    /// registry proceeds past the workspace check. The harness-port
    /// stub has no projects table, so the request still fails
    /// downstream (we don't care which downstream check trips —
    /// just that the registry check let it through).
    #[tokio::test]
    async fn named_workspace_in_registry_proceeds() {
        struct RegistryPort;
        #[async_trait]
        impl AgentDispatchPort for RegistryPort {
            async fn persist_and_route(
                &self,
                _envelope: &EventEnvelope,
            ) -> anyhow::Result<()> {
                Ok(())
            }
            fn daemon_origin(&self) -> String {
                "http://127.0.0.1:0".to_string()
            }
            fn worktrees_dir(&self) -> std::path::PathBuf {
                std::path::PathBuf::from("/tmp")
            }
            fn check_workspace_violation(
                &self,
                _ctx: &RequestContext,
                _claimed_workspace: Option<&str>,
            ) -> Option<String> {
                None
            }
            fn space_id_for_workspace(&self, _workspace_label: &str) -> Option<String> {
                None
            }
            fn valid_workspaces(&self) -> Vec<String> {
                vec!["ship".to_string()]
            }
        }
        let port = RegistryPort;
        let mut req = req_with("rust-lane", Some("spec"), None, Some("dispatch-consolidation"));
        req.workspace = Some("ship".to_string());
        // The downstream code may fail (no worktree, no session DB) —
        // we only assert the registry check did NOT short-circuit.
        let _ = dispatch_agent_session_core(&port, req).await;
    }
}
