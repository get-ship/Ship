//! FIRST-LIGHT — the shipd path that assembles floki's components and runs
//! a real agent turn in-process.
//!
//! Provides:
//!   - [`ShipdKernelSink`] — sync `EventSink` adapter for the async kernel.
//!     Bridges `SessionEventSink`'s synchronous `persist` contract to the
//!     async `KernelRouter::emit` via a background `tokio::spawn`, matching
//!     the fire-and-forget pattern used by all other shipd emit paths.
//!   - [`run_floki_step`] — the composition root for a floki-harness agent run.
//!     Resolves credentials from env, wires `ToolCtx` with a kernel-backed
//!     `SessionEventSink`, and calls `floki::run_step` — the canonical loop.
//!
//! Routing: when `agent.dispatch` identifies a floki-harness agent, callers
//! invoke `run_floki_step`. The existing old-harness path is unmodified;
//! this is the parallel entry point for first-light (slice: FIRST-LIGHT).
//!
//! Layering: L2 (shipd). Depends on floki (L1) + `ShipdGateway` (also L2).
//! The floki crate gains NO shipd dependency — arrow is shipd → floki only.

use std::sync::Arc;

use floki::{
    AgentOutput, AuthContext as FlokiAuth, BuiltModel, Cancel, HeadlessApprover,
    ScriptExecutor, SessionEventSink, Steer, ToolCtx, build_model, run_step,
};
use floki::provider::{Byok, CredentialSource};
use ship_substrate::events::agent_session::AgentSessionStartedPayload;
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope, EventSink};
use ship_types::{Authority, ScopeBinding};

use super::gateway::ShipdGateway;
use crate::ship_runtime::SharedKernel;

// ── ShipdKernelSink ───────────────────────────────────────────────────────────

/// Sync `EventSink` adapter for the shipd kernel.
///
/// `SessionEventSink` calls `EventSink::persist` synchronously from within an
/// async context — rig's tool futures must be `Send`, and agent-def's sink is
/// designed for sync callers. `ShipdKernelSink` bridges the gap by spawning a
/// background task for each `KernelRouter::emit` call, matching shipd's other
/// fire-and-forget emit paths (e.g. `ports::pipeline::persist_and_route`).
///
/// Failures are logged at WARN and never returned (mirrors the rest of shipd's
/// emit paths which also use fire-and-forget semantics).
pub struct ShipdKernelSink {
    kernel: SharedKernel,
}

impl ShipdKernelSink {
    pub fn new(kernel: SharedKernel) -> Self {
        Self { kernel }
    }
}

impl EventSink for ShipdKernelSink {
    fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()> {
        let kernel = self.kernel.clone();
        let event = event.clone();
        // Fire-and-forget: spawn a background task so `persist` returns
        // synchronously. Valid because `persist` is always called from within
        // a tokio async context (run_step runs as a tokio task).
        tokio::spawn(async move {
            let ctx = EmitContext {
                caller_kind: CallerKind::Runtime,
                skill_id: None,
                binding: ScopeBinding::daemon_internal(
                    event.space_id().map(ship_types::SpaceId::new),
                    Authority::Sdk,
                ),
                session_id: event.session_id.clone(),
            };
            if let Err(e) = kernel.lock().await.emit(event, &ctx).await {
                tracing::warn!("floki::first_light: kernel emit failed: {e}");
            }
        });
        Ok(())
    }
}

// ── run_floki_step ────────────────────────────────────────────────────────────

/// Assemble and run one floki agent turn in shipd.
///
/// Resolves credentials from env (`ANTHROPIC_API_KEY` / `OPENAI_API_KEY` +
/// `SHIP_MODEL`), constructs `FlokiAuth` from the agent's declared capability
/// grants, wires a `SessionEventSink` backed by the real kernel, and calls
/// `floki::run_step` — the canonical per-turn loop.
///
/// Canonical `session.*` events are emitted to the kernel regardless of the
/// turn outcome. Returns `Err` if credentials are absent from env or if the
/// turn itself fails.
///
/// # Floki-harness routing
///
/// Call this from the `agent.dispatch` handler when `agent.harness.id == "floki"`.
/// The old-harness dispatch path (`dispatch_agent_session_core`) is unmodified;
/// this is the parallel first-light entry point.
pub async fn run_floki_step(
    agent: &floki::Agent,
    gateway: Arc<ShipdGateway>,
    session_id: String,
    space_id: String,
    kernel: SharedKernel,
    input: &str,
) -> anyhow::Result<AgentOutput> {
    // ── Credentials + model ───────────────────────────────────────────────────
    let cred = Byok::from_env().resolve()?;
    let auth = FlokiAuth::in_process(
        &agent.identity.id,
        agent.capabilities.iter().map(|c| c.domain.clone()).collect(),
    );

    // ── Session event sink ────────────────────────────────────────────────────
    let session_sink = SessionEventSink::new(Arc::new(ShipdKernelSink::new(kernel)));

    // ── ToolCtx ───────────────────────────────────────────────────────────────
    let ctx = ToolCtx {
        session_sink,
        session_id,
        space_id,
        approver: Arc::new(HeadlessApprover),
        steer: Steer::new(),
        cancel: Cancel::new(),
    };

    // ── ScriptExecutor ────────────────────────────────────────────────────────
    // One per session for first-light. A production path shares one executor
    // across sessions (the VM thread is FIFO-queued) — that's a follow-on
    // concern once lifecycle management is defined end to end.
    let executor = Arc::new(ScriptExecutor::spawn());

    // ── Build model and run ───────────────────────────────────────────────────
    // `Arc<ShipdGateway>` coerces to `Arc<dyn floki::Gateway>`.
    match build_model(&cred)? {
        BuiltModel::Anthropic(m) => {
            run_step(m, agent, gateway, auth, executor, ctx, input).await
        }
        BuiltModel::OpenAiCompat(m) => {
            run_step(m, agent, gateway, auth, executor, ctx, input).await
        }
    }
}

// ── Floki-harness dispatch entry point ────────────────────────────────────────

/// Dispatch entry point invoked from the `service.agents` actor when
/// `agent.session.started` arrives with `harness == "floki"`.
///
/// Assembles a minimal `floki::Agent` from the payload (first-light: the
/// identity and model are resolved from the payload; capabilities are left
/// empty — a follow-on slice loads the full profile from disk). Calls
/// `run_floki_step` and logs any error (the `agent.session.started` event
/// already recorded the intent; full session lifecycle events
/// `session.started`/`session.completed` are emitted by `run_step` itself).
///
/// FIRST-LIGHT SCOPE: minimal agent construction from payload fields only.
/// Full agent loading from `.ship/agents/<id>.jsonc` + capability injection
/// is a follow-on slice once the dispatch wiring is verified end-to-end.
pub async fn dispatch_floki_session(
    payload: AgentSessionStartedPayload,
    gateway: Arc<ShipdGateway>,
    kernel: SharedKernel,
) {
    let agent = build_agent_from_payload(&payload);
    let input = payload
        .prompt
        .as_deref()
        .or(payload.spec.as_deref())
        .unwrap_or("(no input provided)");

    tracing::info!(
        agent = %payload.agent,
        slug = %payload.slug,
        session_id = %payload.session_id,
        "floki-dispatch: starting run_floki_step",
    );

    if let Err(e) = run_floki_step(
        &agent,
        gateway,
        payload.session_id.clone(),
        payload.workspace.clone(),
        kernel,
        input,
    )
    .await
    {
        tracing::error!(
            agent = %payload.agent,
            slug = %payload.slug,
            session_id = %payload.session_id,
            "floki-dispatch: run_floki_step failed: {e:#}",
        );
    }
}

/// Build a minimal `floki::Agent` from the dispatch payload.
///
/// Pulls `identity.id` and `disposition.model` from the payload; other fields
/// use safe defaults. A follow-on slice will load the full profile from disk
/// and inject the real capability set.
fn build_agent_from_payload(payload: &AgentSessionStartedPayload) -> floki::Agent {
    // agent_def types are re-exported by ship_agents under `agent_domain`.
    use ship_agents::agent_domain::{Capability, Disposition, Harness, Identity};
    floki::Agent {
        harness: Harness { id: "floki".into(), settings: Default::default() },
        identity: Identity {
            id: payload.agent.clone(),
            name: payload.agent.clone(),
            version: "0.1.0".into(),
            description: String::new(),
            tags: vec![],
        },
        disposition: Disposition {
            rules: String::new(),
            model: payload.model.clone(),
            temperature: None,
            max_turns: None,
        },
        capabilities: vec![] as Vec<Capability>,
        skills: vec![],
        mcp: vec![],
    }
}

// ── Smoke tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "first_light_tests.rs"]
mod tests;
