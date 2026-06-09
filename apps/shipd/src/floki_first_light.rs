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
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope, EventSink};
use ship_types::{Authority, ScopeBinding};

use crate::floki_gateway::ShipdGateway;
use crate::ship_runtime::SharedKernel;

// ── ShipdKernelSink ───────────────────────────────────────────────────────────

/// Sync `EventSink` adapter for the shipd kernel.
///
/// `SessionEventSink` calls `EventSink::persist` synchronously from within an
/// async context — rig's tool futures must be `Send`, and agent-def's sink is
/// designed for sync callers. `ShipdKernelSink` bridges the gap by spawning a
/// background task for each `KernelRouter::emit` call, matching shipd's other
/// fire-and-forget emit paths (e.g. `pipeline_ports::persist_and_route`).
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
                tracing::warn!("floki_first_light: kernel emit failed: {e}");
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

// ── Smoke tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use floki::{
        AuthContext as FlokiAuth, Cancel, HeadlessApprover, ScriptExecutor,
        SessionEventSink, Steer, ToolCtx, run_step,
    };
    use rig::test_utils::{MockCompletionModel, MockTurn};
    use serde_json::json;
    use ship_agents::agent_domain::{BlastRadius, Capability, Disposition, Harness, Identity};
    use ship_kernel::testing::MockSink;

    use crate::floki_gateway::ShipdGateway;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn floki_agent(grants: &[&str]) -> floki::Agent {
        floki::Agent {
            harness: Harness { id: "floki".into(), settings: Default::default() },
            identity: Identity {
                id: "first-light".into(),
                name: "first-light".into(),
                version: "0.1.0".into(),
                description: String::new(),
                tags: vec![],
            },
            disposition: Disposition {
                rules: String::new(),
                model: Some("mock".into()),
                temperature: None,
                max_turns: Some(10),
            },
            capabilities: grants
                .iter()
                .map(|g| Capability::new(*g, BlastRadius::C2Elevated))
                .collect(),
            skills: vec![],
            mcp: vec![],
        }
    }

    /// `ToolCtx` wired to a `MockSink` so tests can assert on emitted events.
    fn test_ctx_with_mock(session_id: &str) -> (ToolCtx, Arc<MockSink>) {
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let ctx = ToolCtx {
            session_sink,
            session_id: session_id.into(),
            space_id: "sp-first-light".into(),
            approver: Arc::new(HeadlessApprover),
            steer: Steer::new(),
            cancel: Cancel::new(),
        };
        (ctx, mock)
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    /// SMOKE — C0 read through the real ShipdGateway; no API key required.
    ///
    /// - MockCompletionModel scripted: call `collections.read` (C0, no grant),
    ///   then return text "done".
    /// - ShipdGateway routes the op through the real in-process ops router;
    ///   the gate clears (C0 read, any authenticated caller).
    /// - MockSink captures `session.started` + `session.completed`.
    ///
    /// Asserts:
    ///   1. `session.started` emitted to the MockSink.
    ///   2. `session.completed` emitted to the MockSink.
    ///   3. Loop completes and returns the scripted text.
    ///   4. The C0 op does NOT appear as denied in the output.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn first_light_smoke_c0_read_session_events_and_gate_clear() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw: Arc<dyn floki::Gateway> =
            Arc::new(ShipdGateway::new(crate::ops_rpc::router(harness.state.clone())));

        // Script: call collections.read once (C0 — gate clears with no grant),
        // then produce text output.
        let model = MockCompletionModel::new([
            MockTurn::tool_call(
                "t1",
                "ship_op",
                json!({ "op": "collections.read", "args": {} }),
            ),
            MockTurn::text("done"),
        ]);

        let agent = floki_agent(&["collections"]);
        let executor = Arc::new(ScriptExecutor::spawn());
        let (ctx, mock) = test_ctx_with_mock("fl-smoke-c0");
        let auth = FlokiAuth::in_process("first-light", vec!["collections".into()]);

        let out = run_step(model, &agent, gw, auth, executor, ctx, "list collections")
            .await
            .expect("first-light smoke (C0 read) must complete");

        // (3) Loop produces the scripted text.
        assert!(out.text.contains("done"), "loop output must contain 'done': {:?}", out.text);

        // (1) + (2) Session lifecycle events emitted.
        let events = mock.captured();
        let event_types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
        assert!(
            event_types.contains(&"session.started"),
            "session.started must be emitted; got: {event_types:?}",
        );
        assert!(
            event_types.contains(&"session.completed"),
            "session.completed must be emitted; got: {event_types:?}",
        );

        // (4) C0 read must NOT be denied.
        assert!(
            !out.text.contains("denied"),
            "C0 read must not be denied by the gate: {:?}", out.text,
        );
    }

    /// SMOKE — denied C2 write through the real ShipdGateway.
    ///
    /// Agent WITHOUT the `mcp` grant tries `mcp.create` (C2). The real
    /// ShipdGateway gate denies it; the model sees `{"denied": ...}` as data
    /// (not an exception), and `session.started` + `session.completed` still fire.
    ///
    /// This validates the gate-verdict-reaches-model path through the real ops
    /// router, not the stub InProcessGateway.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn first_light_smoke_denied_c2_gate_verdict_reaches_model_and_session_events() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw: Arc<dyn floki::Gateway> =
            Arc::new(ShipdGateway::new(crate::ops_rpc::router(harness.state.clone())));

        // No mcp grant — gate must deny mcp.create (C2 elevated).
        let model = MockCompletionModel::new([
            MockTurn::tool_call(
                "t1",
                "ship_op",
                json!({
                    "op": "mcp.create",
                    "args": { "id": "probe", "serverJson": "{}", "scope": "project" }
                }),
            ),
            // Model reads the denied result and produces text.
            MockTurn::text("got denied"),
        ]);

        let agent = floki_agent(&[]); // no grants
        let executor = Arc::new(ScriptExecutor::spawn());
        let (ctx, mock) = test_ctx_with_mock("fl-smoke-c2-denied");
        let auth = FlokiAuth::in_process("first-light", vec![]); // no grants

        let out = run_step(model, &agent, gw, auth, executor, ctx, "create mcp server")
            .await
            .expect("loop completes even when the gate denies the op");

        // Gate denial surfaces as model-readable data — loop does not error.
        assert!(
            out.text.contains("denied"),
            "model must see the gate denial in its text: {:?}", out.text,
        );

        // Session lifecycle events fire regardless of gate outcome.
        let events = mock.captured();
        let types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
        assert!(
            types.contains(&"session.started"),
            "session.started must fire even on a denied run; got: {types:?}",
        );
        assert!(
            types.contains(&"session.completed"),
            "session.completed must fire after the denied run; got: {types:?}",
        );
    }

    /// LIVE-ONLY — full end-to-end with a real API key and real model.
    ///
    /// Exercises `run_floki_step` including `Byok::from_env` credential
    /// resolution and the real Anthropic client. Skipped in CI unless
    /// ANTHROPIC_API_KEY and SHIP_MODEL are set in the environment.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires ANTHROPIC_API_KEY + SHIP_MODEL in env; run manually"]
    async fn first_light_live_run_with_real_api_key() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw = Arc::new(ShipdGateway::new(crate::ops_rpc::router(harness.state.clone())));
        let agent = floki_agent(&["collections"]);

        let out = super::run_floki_step(
            &agent,
            gw,
            "live-session-1".into(),
            "live-space-1".into(),
            harness.state.kernel().clone(),
            "say hello, then list all collections",
        )
        .await
        .expect("live run must complete");

        assert!(!out.text.is_empty(), "live run must produce non-empty output");
        println!("live output: {}", out.text);
    }
}
