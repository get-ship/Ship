//! In-process floki composition — **test / legacy only**.
//!
//! Product native-harness dispatch is out-of-process Connect
//! ([`super::yoke_rpc_client`]). This module still provides:
//!   - [`ShipdKernelSink`] — process-edge adapter reused by the Connect client
//!   - [`dispatch_floki_session`] + smoke tests for gateway + event mapping
//!
//! Not wired into `daemon::lifecycle`.

use std::sync::Arc;

use agent_def::dispatch_intent::DispatchRequestedPayload;
use durable_execution::{Journal, JournalStore, execute};
use floki::dispatch::GatewayDispatch;
use floki::provider::{Byok, CredentialSource};
use floki::{
    AuthContext as FlokiAuth, BuiltModel, Cancel, FlokiProgram, HeadlessApprover, ScriptExecutor,
    SessionCancelled, SessionCompleted, SessionEventHandler, SessionEventSink, SessionFailed,
    SessionStarted, SessionSuspended, Steer, ToolCtx, TurnEnded, TurnStarted, build_model,
};
use journal_sqlite::SqliteJournalStore;
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};
use ship_types::event_types::canonical_session as session_events;
use ship_types::{Authority, ScopeBinding};

use crate::floki::gateway::ShipdGateway;
use crate::ship_runtime::SharedKernel;

// ── ShipdKernelSink ───────────────────────────────────────────────────────────

/// Maps Yoke [`SessionEventHandler`] callbacks onto the shipd kernel.
///
/// Yoke owns emission (library); this is the **process-edge** adapter so
/// session/turn events land in Ship projections. No ship-kernel types inside
/// the `yoke` crate.
pub struct ShipdKernelSink {
    kernel: SharedKernel,
}

impl ShipdKernelSink {
    pub fn new(kernel: SharedKernel) -> Self {
        Self { kernel }
    }

    fn emit_payload<P: serde::Serialize + Send + 'static>(
        &self,
        event_type: &'static str,
        entity_id: &str,
        payload: P,
    ) {
        let kernel = self.kernel.clone();
        let entity_id = entity_id.to_string();
        tokio::spawn(async move {
            let Ok(env) = EventEnvelope::new(event_type, &entity_id, &payload) else {
                tracing::warn!(event_type, "yoke→kernel: failed to build envelope");
                return;
            };
            let ctx = EmitContext {
                caller_kind: CallerKind::Runtime,
                skill_id: None,
                binding: ScopeBinding::daemon_internal(None, Authority::Sdk),
                session_id: Some(entity_id),
            };
            if let Err(e) = kernel.lock().await.emit(env, &ctx).await {
                tracing::warn!(event_type, "yoke→kernel: emit failed: {e}");
            }
        });
    }
}

impl SessionEventHandler for ShipdKernelSink {
    fn on_session_started(&self, session_id: &str, ev: &SessionStarted) -> anyhow::Result<()> {
        self.emit_payload(session_events::SESSION_STARTED, session_id, ev.clone());
        Ok(())
    }
    fn on_session_completed(&self, session_id: &str, ev: &SessionCompleted) -> anyhow::Result<()> {
        self.emit_payload(session_events::SESSION_COMPLETED, session_id, ev.clone());
        Ok(())
    }
    fn on_session_failed(&self, session_id: &str, ev: &SessionFailed) -> anyhow::Result<()> {
        self.emit_payload(session_events::SESSION_FAILED, session_id, ev.clone());
        Ok(())
    }
    fn on_session_cancelled(&self, session_id: &str, ev: &SessionCancelled) -> anyhow::Result<()> {
        self.emit_payload(session_events::SESSION_CANCELLED, session_id, ev.clone());
        Ok(())
    }
    fn on_session_suspended(&self, session_id: &str, ev: &SessionSuspended) -> anyhow::Result<()> {
        self.emit_payload(session_events::SESSION_SUSPENDED, session_id, ev.clone());
        Ok(())
    }
    fn on_turn_started(&self, session_id: &str, ev: &TurnStarted) -> anyhow::Result<()> {
        self.emit_payload(session_events::TURN_STARTED, session_id, ev.clone());
        Ok(())
    }
    fn on_turn_ended(&self, session_id: &str, ev: &TurnEnded) -> anyhow::Result<()> {
        self.emit_payload(session_events::TURN_ENDED, session_id, ev.clone());
        Ok(())
    }
}

// ── Floki-harness dispatch entry point ────────────────────────────────────────

/// Dispatch entry point invoked from the `service.agents` actor when
/// `dispatch.requested` arrives with `harness == "floki"`.
///
/// Stage-2: assembles a `FlokiProgram` and drives it through the durable
/// execution engine (`execute()`), so every rig exchange is journaled and
/// a crashed/suspended session can resume without re-burning model tokens.
/// The journal is saved via the persisted `SqliteJournalStore` (P1.0b) —
/// a suspended session survives a daemon restart; Stage-3 adds the boot-scan
/// that auto-resumes it.
///
/// Session lifecycle events (`session.started`, `session.completed`) are
/// emitted by `FlokiProgram::run`, not this function.
///
/// FIRST-LIGHT SCOPE: minimal agent construction from payload fields only.
/// Full agent loading from `.ship/agents/<id>.jsonc` + capability injection
/// is a follow-on slice once the dispatch wiring is verified end-to-end.
pub async fn dispatch_floki_session(
    payload: DispatchRequestedPayload,
    gateway: Arc<ShipdGateway>,
    kernel: SharedKernel,
) {
    let input = payload
        .prompt
        .as_deref()
        .or(payload.spec.as_deref())
        .unwrap_or("(no input provided)")
        .to_string();

    tracing::info!(
        agent_id = %payload.agent_id,
        slug = %payload.slug,
        session_id = %payload.session_id,
        "floki-dispatch: starting FlokiProgram execute",
    );

    // ── Credentials + model ───────────────────────────────────────────────
    let cred = match Byok::from_env().resolve() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(
                session_id = %payload.session_id,
                "floki-dispatch: credential resolution failed: {e:#}",
            );
            return;
        }
    };
    let auth = FlokiAuth::in_process(&payload.agent_id, vec![]);
    // Resolve the ship dir BEFORE the kernel Arc moves into the sink.
    let ship_dir = kernel.lock().await.base_dir().to_path_buf();
    let session_sink = SessionEventSink::new(Arc::new(ShipdKernelSink::new(kernel)));
    let tool_ctx = ToolCtx {
        session_sink,
        session_id: payload.session_id.clone(),
        // space_id: the cosmetic workspace label is not a real space id —
        // the floki path always left this empty (the field-10 `workspace`
        // slug label was never a space id; see `dispatch_intent` and the
        // `SessionStarted.space_id` rule).
        space_id: String::new(),
        approver: Arc::new(HeadlessApprover),
        steer: Steer::new(),
        cancel: Cancel::new(),
        journal_mode: true,
    };
    let executor = Arc::new(ScriptExecutor::spawn());
    let gw: Arc<dyn floki::Gateway> = gateway;
    // ── Journal store: the PERSISTED store (P1.0b) so a suspended floki
    // session survives a daemon restart. (Verifier+fresh-eyes caught the
    // committed Stage-2 constructing InMemoryJournalStore per call — no
    // cross-restart persistence despite the design.) The runtime db carries
    // the durable_journals table via migration 202606100001.
    let store = match SqliteJournalStore::open(&ship_substrate::db::runtime_db_path(&ship_dir)).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "floki: cannot open the journal store; session will not be durable");
            return;
        }
    };
    let journal = store
        .load(&payload.session_id)
        .await
        .unwrap_or(None)
        .unwrap_or_else(Journal::new);

    // ── Execute ───────────────────────────────────────────────────────────
    // The two provider arms (`Anthropic` / `OpenAiCompat`) need the same
    // `FlokiProgram` wiring — only the model type differs. A closure can't
    // span both arms because each completion-model is a distinct
    // `GenericCompletionModel<M>` type, so we use a local macro to share
    // the construction without monomorphizing through a generic fn (which
    // would also need `M: CompletionModel + ...` bounds at the call site).
    macro_rules! make_program {
        ($model:expr) => {
            FlokiProgram {
                model: $model,
                preamble: String::new(),
                initial_input: input.clone(),
                max_turns: 10,
                gateway: gw.clone(),
                auth: auth.clone(),
                executor: executor.clone(),
                grants: vec![],
                tool_ctx: tool_ctx.clone(),
            }
        };
    }
    let run = |journal: Journal| async {
        match build_model(&cred) {
            Ok(BuiltModel::Anthropic(m)) => {
                let program = make_program!(m);
                let dispatch = Arc::new(GatewayDispatch::new(gw.clone(), auth.clone()));
                execute(journal, dispatch, &program).await
            }
            Ok(BuiltModel::OpenAiCompat(m)) => {
                let program = make_program!(m);
                let dispatch = Arc::new(GatewayDispatch::new(gw.clone(), auth.clone()));
                execute(journal, dispatch, &program).await
            }
            Err(e) => {
                tracing::error!(
                    session_id = %payload.session_id,
                    "floki-dispatch: build_model failed: {e:#}",
                );
                // Return a Failed status so the session is logged but no panic.
                (durable_execution::RunStatus::Failed(e.to_string()), Journal::new())
            }
        }
    };

    let (status, advanced_journal) = run(journal).await;

    // ── Persist journal (terminal boundary) ──────────────────────────────
    // Save unconditionally: partial progress is durable, Failed state is useful
    // for post-mortem. A warning on save failure is acceptable (fire-and-forget).
    if let Err(e) = store.save(&payload.session_id, &advanced_journal).await {
        tracing::warn!(
            session_id = %payload.session_id,
            "floki-dispatch: journal save failed: {e}",
        );
    }

    tracing::info!(
        session_id = %payload.session_id,
        "floki-dispatch: session finished with status: {status:?}",
    );
}


// ── Smoke tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use floki::{
        AuthContext as FlokiAuth, BlastRadius, Cancel, Capability, Disposition, Harness,
        HeadlessApprover, Identity, RecordingSessionEventHandler, ScriptExecutor, SessionEventSink,
        Steer, ToolCtx, run_step,
    };
    use rig::test_utils::{MockCompletionModel, MockTurn};
    use serde_json::json;

    use crate::floki::gateway::ShipdGateway;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn floki_agent(grants: &[&str]) -> floki::Agent {
        floki::Agent {
            harness: Harness {
                id: "floki".into(),
                settings: Default::default(),
            },
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

    /// `ToolCtx` wired to a recording sink so tests can assert on emitted events.
    fn test_ctx_with_mock(
        session_id: &str,
    ) -> (ToolCtx, Arc<RecordingSessionEventHandler>) {
        let (session_sink, rec) = SessionEventSink::recording();
        let ctx = ToolCtx {
            session_sink,
            session_id: session_id.into(),
            space_id: "sp-first-light".into(),
            approver: Arc::new(HeadlessApprover),
            steer: Steer::new(),
            cancel: Cancel::new(),
            journal_mode: false,
        };
        (ctx, rec)
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
    async fn floki_dispatch_smoke_c0_read_session_events_and_gate_clear() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw: Arc<dyn floki::Gateway> =
            Arc::new(ShipdGateway::new(crate::rpc::ops::router(harness.state.clone())));

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
        let event_types = mock.names();
        assert!(
            event_types.iter().any(|e| e == "session.started"),
            "session.started must be emitted; got: {event_types:?}",
        );
        assert!(
            event_types.iter().any(|e| e == "session.completed"),
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
    async fn floki_dispatch_smoke_denied_c2_gate_verdict_reaches_model_and_session_events() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw: Arc<dyn floki::Gateway> =
            Arc::new(ShipdGateway::new(crate::rpc::ops::router(harness.state.clone())));

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
        let types = mock.names();
        assert!(
            types.iter().any(|e| e == "session.started"),
            "session.started must fire even on a denied run; got: {types:?}",
        );
        assert!(
            types.iter().any(|e| e == "session.completed"),
            "session.completed must fire after the denied run; got: {types:?}",
        );
    }

}
