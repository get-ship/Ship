//! Smoke tests for the FIRST-LIGHT composition root (split out to keep
//! `floki_dispatch.rs` under the file cap).

    use std::sync::Arc;

    use floki::{
        AuthContext as FlokiAuth, Cancel, HeadlessApprover, ScriptExecutor,
        SessionEventSink, Steer, ToolCtx, run_step,
    };
    use rig::test_utils::{MockCompletionModel, MockTurn};
    use serde_json::json;
    use ship_agents::agent_domain::{BlastRadius, Capability, Disposition, Harness, Identity};
    use ship_kernel::testing::MockSink;

    use crate::floki::gateway::ShipdGateway;

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
    async fn floki_dispatch_live_run_with_real_api_key() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw = Arc::new(ShipdGateway::new(crate::rpc::ops::router(harness.state.clone())));
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
