//! Transport-neutral in-process agent-session dispatch core.
//!
//! `dispatch_agent_session_core` is the shared logic behind the Connect
//! `AgentsService.Dispatch` wire method (and the now-removed REST
//! `POST /api/v1/agent_sessions/dispatch` handler). It validates the
//! in-process request, derives slug/branch, builds the
//! `dispatch.requested` event, persists it, and returns a structured
//! result.
//!
//! In-process dispatch is floki-class only (mount split, 2026-07-10):
//! the wire carries only `{agent, prompt, model, slug}` — the
//! coding-class fields (spec / resume / harness / invocation /
//! keepalive / branch / source_spec_path / depends_on / repo_path /
//! base_ref) live on `CodeDispatchRequest` in `code_dispatch_request.rs`
//! and are served by `CodeService.Dispatch`. The floki runtime has
//! nothing to resume, no worktree to provision, no harness to launch
//! under supervision — those concepts are coding-class only.
//!
//! Daemon-specific state (kernel, origin URL) is injected via
//! [`AgentDispatchPort`] so shipd provides a concrete adapter while this
//! module remains daemon-free. The `worktrees_dir` accessor is gated on
//! `ship-code` because the coding path of a ship-standard build
//! doesn't expose it; the in-process core doesn't touch it.

use async_trait::async_trait;

use agent_schema::canonical_session::{SessionStarted, event_types as canonical_session_event_types};
use agent_schema::dispatch_intent::{DispatchRequestedPayload, event_types as dispatch_event_types};
use ship_substrate::events::EventEnvelope;

use super::dispatch_request::{DispatchRequest, validate_dispatch_request};

// ── Port trait ────────────────────────────────────────────────────────────────

/// Daemon-specific capabilities the dispatch core needs. Implemented by shipd;
/// declared here so ship-agents stays daemon-free.
///
/// The coding-class counterpart (`CodeService.Dispatch`) reuses this
/// same trait — the two surfaces share kernel persist-and-route, the
/// daemon origin (for the `studio_url`), and (on the coding path only,
/// gated) the worktrees root.
#[async_trait]
pub trait AgentDispatchPort: Send + Sync {
    /// Persist an event envelope and route it to the kernel.
    async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()>;

    /// Base URL the daemon is listening on (e.g. `http://localhost:7587`).
    fn daemon_origin(&self) -> String;

    /// Absolute path to the worktrees root directory.
    ///
    /// `None` when the daemon has no harness-driver backend wired.
    /// The in-process core never reads it; the coding core reads it
    /// under the `ship-code` cfg to resolve `resume: true`.
    #[cfg(feature = "bridge")]
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

/// Transport-neutral core of in-process (floki-class) single-agent
/// dispatch. Shared by the ConnectRPC `AgentsService.dispatch` wire
/// method.
///
/// The `mcp_workspace_violation` guard is NOT performed here — it is an
/// auth-context concern that belongs at the transport boundary. Callers
/// must enforce it before invoking this function.
pub async fn dispatch_agent_session_core(
    port: &dyn AgentDispatchPort,
    req: DispatchRequest,
) -> Result<DispatchOutcome, DispatchCoreError> {
    // Shape validation FIRST — before any side effect. The "agent is in
    // the index" check is enforced at the wire boundary
    // (rpc_wire::dispatch) since it needs the ship_dir to consult the
    // index.
    if let Err(message) = validate_dispatch_request(&req) {
        return Err(DispatchCoreError::BadRequest(message));
    }

    // The slug-fallback label — the dispatch core derives
    // `<agent>-<ulid>` when the caller omits an explicit slug. The
    // in-process path has no resume (floki has nothing to resume), so
    // we never look at `worktrees_dir`.
    let slug_label = req.agent.clone();

    // In-process: no resume, no branch override, no worktree. Slug
    // is auto-derived from the agent id when not provided. Branch is
    // always `job/<slug>` for consistency with the coding surface
    // (the floki actor treats branch as advisory metadata; no git ops
    // run).
    let slug = req.slug.clone().unwrap_or_else(|| {
        // Collision-free suffix from the ULID's random tail — a
        // `[..8]` prefix is pure timestamp and collides for rapid
        // same-agent dispatches. See gen_slug_suffix.
        format!("{}-{}", slug_label, ship_substrate::gen_slug_suffix())
    });
    let branch = format!("job/{slug}");
    let session_id = ship_substrate::gen_ulid();

    // Build the dispatch intent payload. The in-process shape only
    // carries the four agent / prompt / model / slug fields; the
    // driver-edge fields (repo_path / base_ref / source_spec_path /
    // harness / branch override / resume / spec / depends_on / vars)
    // are all `None` / `false` because the wire didn't carry them.
    // The `service.agents` actor routes floki-harness envelopes
    // (`harness == Some("floki")`) to the in-process floki closure —
    // it ignores the driver-edge fields the coding path uses.
    let dispatch_payload = DispatchRequestedPayload {
        session_id: session_id.clone(),
        slug: slug.clone(),
        agent_id: req.agent.clone(),
        // The floki path always runs floki; the coding path's harness
        // override lives on `CodeDispatchRequest`. Wire a "floki"
        // marker so the actor routes this envelope to the floki
        // closure (which ignores it otherwise).
        harness: Some("floki".to_string()),
        model: req.model,
        branch: branch.clone(),
        repo_path: None,
        base_ref: None,
        source_spec_path: None,
        // Ship-standard is git-free by ruling (decisions-2026-07-09 §"Git
        // surface"); the in-process path never captures a baseline.
        baseline_sha: None,
        // In-process: no attach surface, no keepalive concept. The
        // floki runtime drives itself in-process.
        open_tab: false,
        // Default invocation mode for the dispatch payload projection;
        // the floki runtime doesn't read this field. `Pty` matches
        // the historical pre-cutover default.
        invocation: agent_schema::InvocationMode::Pty,
        keepalive: false,
        prompt: req.prompt.clone(),
        // In-process never carries a spec body — the prompt is the
        // first user turn and there's no job-spec markdown to land
        // in a worktree (no worktree).
        spec: None,
        resume: false,
        resume_session_id: None,
        depends_on: None,
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

    Ok(DispatchOutcome {
        session_id,
        slug,
        branch,
        resume: false,
        studio_url,
        billing_warning: None,
    })
}

#[cfg(test)]
mod tests {
    //! Validation-gate tests for `dispatch_agent_session_core`. The point of
    //! these is the wire-contract claim: a malformed request must be
    //! rejected with `BadRequest` BEFORE any event is emitted — the test
    //! doubles below count `persist_and_route` calls and assert they stay
    //! at zero for every rejected request.

    use super::*;
    use crate::services::agents::dispatch_request::DispatchRequest;
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
        #[cfg(feature = "bridge")]
        fn worktrees_dir(&self) -> Option<std::path::PathBuf> {
            Some(std::path::PathBuf::from("/tmp"))
        }
    }
    fn req_with(agent: &str, prompt: Option<&str>, slug: Option<&str>) -> DispatchRequest {
        DispatchRequest {
            agent: agent.to_string(),
            prompt: prompt.map(str::to_string),
            model: None,
            slug: slug.map(str::to_string),
        }
    }

    /// The headline test: a literal `{}` shape — empty agent, no prompt,
    /// no slug — must be rejected as `BadRequest` and must NOT emit a
    /// single event.
    #[tokio::test]
    async fn empty_request_rejected_with_no_side_effects() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        let req = req_with("", None, None);
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
        let req = req_with("", Some("do the thing"), None);
        let outcome = dispatch_agent_session_core(&port, req).await;
        let err = match outcome {
            Ok(_) => panic!("empty agent must be rejected, got Ok"),
            Err(e) => e,
        };
        assert!(matches!(err, DispatchCoreError::BadRequest(_)));
        assert_eq!(port.persist_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn blank_prompt_rejected_no_side_effects() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        // Agent is set, slug is None — passes the first two checks; a
        // blank prompt trips the third check.
        let req = req_with("rust-lane", None, None);
        let outcome = dispatch_agent_session_core(&port, req).await;
        let err = match outcome {
            Ok(_) => panic!("blank prompt must be rejected, got Ok"),
            Err(e) => e,
        };
        match err {
            DispatchCoreError::BadRequest(m) => assert!(m.contains("`prompt` is required"), "got: {m}"),
            other => panic!("expected BadRequest, got {other:?}"),
        }
        assert_eq!(port.persist_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn bad_slug_rejected_no_side_effects() {
        let port = CountingPort {
            persist_calls: AtomicUsize::new(0),
        };
        let req = req_with("rust-lane", Some("do the thing"), Some("-bad-slug"));
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
            Some("do the thing"),
            Some("dispatch-consolidation"),
        );
        let _ = dispatch_agent_session_core(&port, req).await;
        assert!(
            port.persist_calls.load(Ordering::SeqCst) >= 1,
            "well-formed request must emit at least the dispatch.requested event"
        );
    }
}
