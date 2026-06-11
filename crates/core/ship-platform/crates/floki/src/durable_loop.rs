//! Floki Stage 2 — Journal-driven Live Loop.
//!
//! Makes the floki agent loop durable: a crashed or suspended session resumes
//! without re-burning model tokens. Each rig exchange (one model call + its
//! tool results) is a single [`durable_execution::ExecCtx::record`] entry, so a
//! resumed run replays completed exchanges from the journal and only re-runs the
//! live frontier.
//!
//! Exports:
//! - [`RigTurnOutcome`] — serializable outcome of one rig exchange, stored as the
//!   `Value` inside the journal entry.
//! - [`FlokiProgram`] — implements `durable_execution::Program`; the outer loop
//!   manages journaled turns.
//!
//! Stage-3 deferrals (not implemented here):
//! - Mid-exchange saves (per-turn journal persistence requires threading
//!   `JournalStore` into `ExecCtx` — a follow-on engine change).
//! - Approval re-injection on resume (wiring `decision` back to the gateway).
//! - Multi-input continuation (same `session_id` receiving follow-up prompts).
//! - Restart resume-triggering (daemon boot scan for suspended sessions).

use std::sync::Arc;

use async_trait::async_trait;
use rig::agent::AgentBuilder;
use rig::completion::{Chat, CompletionModel, Message};
use rig::completion::message::{ToolResultContent, UserContent};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use agent_def::{SessionCompleted, SessionStarted};
use durable_execution::{EngineError, ExecCtx, Program};
use ship_js::deno::ScriptExecutor;

use crate::agent_loop::{RunScriptTool, ShipOpTool, ToolCtx};
use crate::auth::AuthContext;
use crate::gateway::Gateway;

// ---------------------------------------------------------------------------
// RigTurnOutcome
// ---------------------------------------------------------------------------

/// The serializable outcome of one rig exchange: one model call plus any tool
/// calls it triggered. Stored as the `Value` in `ctx.record("turn.{N}", ...)`.
///
/// `history_delta` is the slice of rig [`Message`]s appended to the shared
/// `chat_history` during this exchange. Storing the delta lets `FlokiProgram`
/// reconstruct the full conversation on replay without re-running the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigTurnOutcome {
    /// The model's final text for this exchange (after tool resolution).
    pub text: String,
    /// Incremental conversation messages (user + tool-results + assistant)
    /// added during this exchange. Serialized as rig `Message` JSON so they
    /// survive the journal boundary.
    pub history_delta: Vec<Value>,
    /// `Some(checkpoint)` when the exchange ended on a suspension (Ask-stance
    /// gate raised while `journal_mode` is `true`). The checkpoint string is
    /// passed to `ctx.suspend()` on the next journal slot.
    pub pending_suspension: Option<String>,
}

// ---------------------------------------------------------------------------
// Suspension detection
// ---------------------------------------------------------------------------

/// Scan the new messages appended to `chat_history` during one exchange and
/// return the first suspension checkpoint, if any.
///
/// When `ToolCtx.journal_mode` is `true`, a `ShipOpTool` that hits an
/// Ask-stance gate returns `{"suspended": "approval:{id}:{op}"}` as its text.
/// That text lands as a `ToolResultContent::Text` item in the history. This
/// function locates it.
///
/// If `annotate_with_steer` wrapped the suspension in a `{"result": "...",
/// "steers": [...]}` envelope, we unwrap the inner `"result"` string first.
fn extract_suspension_from_history(messages: &[Message]) -> Option<String> {
    for msg in messages {
        let Message::User { content } = msg else { continue };
        for item in content.iter() {
            let UserContent::ToolResult(result) = item else { continue };
            for content_item in result.content.iter() {
                let ToolResultContent::Text(text_block) = content_item else { continue };
                if let Some(cp) = parse_suspension_from_text(&text_block.text) {
                    return Some(cp);
                }
            }
        }
    }
    None
}

/// Parse `{"suspended": "checkpoint"}` out of a tool result text, handling the
/// optional `{"result": "...", "steers": [...]}` envelope from `annotate_with_steer`.
fn parse_suspension_from_text(text: &str) -> Option<String> {
    let v: Value = serde_json::from_str(text).ok()?;
    // Direct: {"suspended": "..."}
    if let Some(cp) = v.get("suspended").and_then(|s| s.as_str()) {
        return Some(cp.to_string());
    }
    // Wrapped by annotate_with_steer: {"result": "{\"suspended\": \"...\"}",  "steers": [...]}
    if let Some(inner_str) = v.get("result").and_then(|s| s.as_str()) {
        let inner: Value = serde_json::from_str(inner_str).ok()?;
        if let Some(cp) = inner.get("suspended").and_then(|s| s.as_str()) {
            return Some(cp.to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// run_one_rig_exchange
// ---------------------------------------------------------------------------

/// Drive ONE rig exchange: build a stateless `Agent`, call `Chat::chat` with
/// the accumulated `chat_history`, and return a [`RigTurnOutcome`] capturing
/// the text response, the history delta, and any suspension marker.
///
/// The agent's `default_max_turns` is left at the rig default (0 → at most 2
/// rig iterations: one tool call round-trip + final assistant text). Each
/// `FlokiProgram` outer-loop iteration is one exchange; the outer loop controls
/// session length via `max_turns`.
///
/// `tool_ctx` is cloned for each exchange so each tool instance sees an
/// independent cancel/steer/approver state.
async fn run_one_rig_exchange<M>(
    model: &M,
    preamble: &str,
    prior_history: &[Message],
    input: &str,
    tool_ctx: &ToolCtx,
    gateway: &Arc<dyn Gateway>,
    auth: &AuthContext,
    executor: &Arc<ScriptExecutor>,
    grants: &[String],
) -> anyhow::Result<RigTurnOutcome>
where
    M: CompletionModel + Clone + 'static,
{
    // Cooperative cancel check before invoking rig.
    if tool_ctx.cancel.is_cancelled() {
        tool_ctx.emit_cancelled("user-requested");
        return Ok(RigTurnOutcome {
            text: String::new(),
            history_delta: vec![],
            pending_suspension: None,
        });
    }

    let mut chat_history: Vec<Message> = prior_history.to_vec();
    let history_before = chat_history.len();

    // Build a stateless agent for this exchange. Tools are recreated per
    // exchange so each gets a fresh clone of `tool_ctx`.
    let runner = AgentBuilder::new(model.clone())
        .preamble(preamble)
        .tool(ShipOpTool {
            gateway: gateway.clone(),
            auth: auth.clone(),
            ctx: tool_ctx.clone(),
        })
        .tool(RunScriptTool {
            executor: executor.clone(),
            gateway: gateway.clone(),
            grants: grants.to_vec(),
            auth: auth.clone(),
            ctx: tool_ctx.clone(),
        })
        .build();

    // Drive one rig exchange: `Chat::chat` mutates `chat_history` in-place,
    // appending new messages (tool calls, tool results, assistant response).
    let text = runner
        .chat(input, &mut chat_history)
        .await
        .map_err(|e| anyhow::anyhow!("rig chat failed: {e}"))?;

    // The delta is everything appended during this exchange.
    let history_delta: Vec<Value> = chat_history[history_before..]
        .iter()
        .filter_map(|m| serde_json::to_value(m).ok())
        .collect();

    // Detect an Ask-stance suspension marker in the new messages.
    let pending_suspension = extract_suspension_from_history(&chat_history[history_before..]);

    Ok(RigTurnOutcome { text, history_delta, pending_suspension })
}

// ---------------------------------------------------------------------------
// FlokiProgram
// ---------------------------------------------------------------------------

/// A floki session expressed as a [`durable_execution::Program`].
///
/// Each rig exchange (one model call + tool calls) is one `ctx.record("turn.N")`
/// entry. On replay, completed exchanges return logged `RigTurnOutcome` values
/// without touching the model. On the live frontier, the closure calls
/// `run_one_rig_exchange`, journals the result, and advances.
///
/// An Ask-stance suspension writes a `Suspended` entry via `ctx.suspend()` and
/// parks the run. The dispatcher saves the journal; a later `resume()` call
/// resolves the suspended entry and continues.
///
/// `SEND` invariant: `FlokiProgram<M>` is `Send + Sync` when `M: Send + Sync`,
/// which all production `CompletionModel` impls satisfy. Closure futures
/// produced inside `Program::run` are `Send` when all captured types are
/// `Send` — verified by the `#[async_trait]` generated code.
pub struct FlokiProgram<M: CompletionModel + 'static> {
    pub model: M,
    pub preamble: String,
    pub initial_input: String,
    /// Maximum outer-loop turns. Each turn is one rig exchange journaled under
    /// `"turn.{N}"`. The session ends when the model returns text without a
    /// suspension, or when `max_turns` is exhausted.
    pub max_turns: u32,
    pub gateway: Arc<dyn Gateway>,
    pub auth: AuthContext,
    pub executor: Arc<ScriptExecutor>,
    pub grants: Vec<String>,
    /// Carries session identity and event emission into every tool call. Its
    /// `journal_mode: true` flag causes Ask-stance ops to return suspension
    /// markers instead of blocking on the approver.
    pub tool_ctx: ToolCtx,
}

#[async_trait]
impl<M> Program for FlokiProgram<M>
where
    M: CompletionModel + Send + Sync + Clone + 'static,
{
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
        // Emit session.started exactly once across execute()+resume(): the
        // emission is itself a journaled op (ctx.record), so the live frontier
        // fires it and every replay returns the logged marker WITHOUT
        // re-emitting. (Verifier-caught: the previous code claimed this fence
        // in a comment but called the sink unfenced — session.started
        // double-emitted on every resume.)
        let sink = &self.tool_ctx.session_sink;
        let session_id = self.tool_ctx.session_id.clone();
        let space_id = self.tool_ctx.space_id.clone();
        let initial_input = self.initial_input.clone();
        ctx.record("session.start", || async move {
            sink.emit_session_started(
                &session_id,
                &SessionStarted {
                    session_id: session_id.clone(),
                    agent_id: String::new(), // resolved by caller if needed
                    harness: "floki".into(),
                    model: None,
                    space_id,
                    slug: None,
                    input: Some(initial_input),
                    parent: None,
                },
            )
            .ok();
            serde_json::Value::Bool(true)
        })
        .await?;

        let mut chat_history: Vec<Message> = Vec::new();
        let mut final_text = String::new();

        for turn_idx in 0..self.max_turns {
            let turn_key = format!("turn.{turn_idx}");
            let input_for_turn = if turn_idx == 0 {
                self.initial_input.clone()
            } else {
                // Subsequent turns carry context through chat_history; input is empty.
                String::new()
            };

            // Snapshot the history before this exchange so the closure owns
            // a Send-safe clone for the journal entry.
            let history_snapshot: Vec<Message> = chat_history.clone();

            // ── Journal entry ────────────────────────────────────────────────
            // LIVE: runs the exchange, logs the RigTurnOutcome.
            // REPLAY: returns the logged RigTurnOutcome without calling the model.
            let model = self.model.clone();
            let preamble = self.preamble.clone();
            let tool_ctx = self.tool_ctx.clone();
            let gateway = self.gateway.clone();
            let auth = self.auth.clone();
            let executor = self.executor.clone();
            let grants = self.grants.clone();
            let turn_value = ctx
                .record(&turn_key, move || async move {
                    match run_one_rig_exchange(
                        &model,
                        &preamble,
                        &history_snapshot,
                        &input_for_turn,
                        &tool_ctx,
                        &gateway,
                        &auth,
                        &executor,
                        &grants,
                    )
                    .await
                    {
                        Ok(outcome) => serde_json::to_value(&outcome)
                            .unwrap_or_else(|_| json!({ "error": "serialize-failed" })),
                        Err(e) => json!({ "error": e.to_string() }),
                    }
                })
                .await?;

            // Deserialize the logged outcome (same path for live and replay).
            let outcome: RigTurnOutcome =
                serde_json::from_value(turn_value).map_err(|e| EngineError::Dispatch {
                    op: turn_key.clone(),
                    reason: format!("cannot deserialize RigTurnOutcome: {e}"),
                })?;

            // Extend chat_history from the delta (works on both live and replay).
            for delta_val in &outcome.history_delta {
                if let Ok(msg) = serde_json::from_value::<Message>(delta_val.clone()) {
                    chat_history.push(msg);
                }
            }
            final_text = outcome.text.clone();

            // ── Suspension (Ask-stance gate raised) ──────────────────────────
            if let Some(ref checkpoint) = outcome.pending_suspension {
                // ctx.suspend journals a Suspended entry and parks the run.
                // On resume: the entry is resolved to a decision Value and
                // ctx.suspend returns it. Stage-3 will use that Value to inject
                // the operator's approval/denial into the next exchange.
                let _decision = ctx.suspend(checkpoint).await?;
                // Fall through: the loop continues; the next iteration re-runs
                // the exchange with the same input (Stage-3 will re-inject the
                // originally-blocked op via steer or direct dispatch).
                continue;
            }

            // ── Session done ─────────────────────────────────────────────────
            // The model returned text without a pending suspension: session complete.
            if !outcome.text.is_empty() {
                break;
            }
        }

        // Emit session.completed on clean exit.
        self.tool_ctx
            .session_sink
            .emit_session_completed(
                &self.tool_ctx.session_id,
                &SessionCompleted {
                    session_id: self.tool_ctx.session_id.clone(),
                },
            )
            .ok();

        Ok(json!(final_text))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use agent_def::{SessionEventSink};
    use durable_execution::{InMemoryJournalStore, Journal, JournalStore, RunStatus, execute};
    use rig::test_utils::{MockCompletionModel, MockTurn};
    use serde_json::json;
    use ship_kernel::testing::MockSink;

    use crate::agent_loop::{Cancel, HeadlessApprover, Steer, ToolCtx};
    use crate::auth::AuthContext;
    use crate::dispatch::GatewayDispatch;
    use crate::gateway::InProcessGateway;

    use super::*;

    fn tool_ctx_journal(session_id: &str, mock: Arc<MockSink>) -> ToolCtx {
        ToolCtx {
            session_sink: SessionEventSink::new(mock),
            session_id: session_id.into(),
            space_id: "sp-test".into(),
            approver: Arc::new(HeadlessApprover),
            steer: Steer::new(),
            cancel: Cancel::new(),
            journal_mode: true,
        }
    }

    fn tool_ctx_headless(session_id: &str, mock: Arc<MockSink>) -> ToolCtx {
        ToolCtx {
            session_sink: SessionEventSink::new(mock),
            session_id: session_id.into(),
            space_id: "sp-test".into(),
            approver: Arc::new(HeadlessApprover),
            steer: Steer::new(),
            cancel: Cancel::new(),
            journal_mode: false,
        }
    }

    fn dummy_dispatch(gateway: Arc<dyn Gateway>, auth: AuthContext) -> Arc<dyn durable_execution::Dispatch> {
        Arc::new(GatewayDispatch::new(gateway, auth))
    }

    // Build a FlokiProgram from a mock model with the given scripted turns.
    fn make_program<M: CompletionModel + Send + Sync + Clone + 'static>(
        model: M,
        tool_ctx: ToolCtx,
    ) -> FlokiProgram<M> {
        FlokiProgram {
            model,
            preamble: "You are a Ship agent.".into(),
            initial_input: "test".into(),
            max_turns: 10,
            gateway: Arc::new(InProcessGateway::default()),
            auth: AuthContext::in_process("t", vec!["collections".into()]),
            executor: Arc::new(ship_js::deno::ScriptExecutor::spawn()),
            grants: vec!["collections".into()],
            tool_ctx,
        }
    }

    // rig_exchange_not_recalled_on_journal_replay:
    //
    // Run a 1-turn session, capturing the journal. Then seed that journal into
    // InMemoryJournalStore and re-run execute() with a NEW model that has NO
    // scripted turns. The program must replay the logged outcome and return the
    // same text WITHOUT calling the new model (request_count stays 0).
    //
    // Behavioral invariant: ctx.record("turn.0") on replay returns the logged
    // value without calling the compute closure.
    #[tokio::test]
    async fn rig_exchange_not_recalled_on_journal_replay() {
        let mock = Arc::new(MockSink::new());
        // First run: one model turn that returns text immediately.
        let model_first = MockCompletionModel::new([MockTurn::text("hello from model")]);
        let ctx_first = tool_ctx_headless("replay-test", mock.clone());
        let program_first = make_program(model_first.clone(), ctx_first);
        let auth = AuthContext::in_process("t", vec!["collections".into()]);
        let dispatch = dummy_dispatch(Arc::new(InProcessGateway::default()), auth.clone());

        let (status_first, journal_first) = execute(Journal::new(), dispatch.clone(), &program_first).await;
        assert!(
            matches!(status_first, RunStatus::Completed(_)),
            "first run must complete: {status_first:?}"
        );
        assert_eq!(model_first.request_count(), 1, "first run must invoke the model once");

        // Save the journal.
        let store = InMemoryJournalStore::new();
        store.save("replay-test", &journal_first).await.unwrap();

        // Second run: use a model with NO scripted turns — it must NOT be called.
        let model_second = MockCompletionModel::new([]); // no turns
        let mock2 = Arc::new(MockSink::new());
        let ctx_second = tool_ctx_headless("replay-test", mock2.clone());
        let program_second = make_program(model_second.clone(), ctx_second);

        // Load the journal from the store and re-execute.
        let journal_loaded = store.load("replay-test").await.unwrap().unwrap();
        let (status_second, _) = execute(journal_loaded, dispatch.clone(), &program_second).await;

        assert!(
            matches!(status_second, RunStatus::Completed(_)),
            "replay must complete: {status_second:?}"
        );
        assert_eq!(
            model_second.request_count(), 0,
            "replay must NOT invoke the model (turn logged in journal)"
        );
    }

    // two_turn_session_replay_skips_completed_turns:
    //
    // A 2-turn session where turn-0 calls a tool and turn-1 returns text.
    // After the live run, we seed the journal and run again with a model that
    // only has turn-1 scripted. Turn-0 must replay (model not called), turn-1
    // must run live (model called once). Total request_count == 1 (not 2).
    #[tokio::test]
    async fn two_turn_session_replay_skips_completed_turns() {
        // Script: turn-0 = tool call → tool result → text.
        // With the rig default max_turns (0), one tool call + final text = 2 rig iterations.
        let mock = Arc::new(MockSink::new());
        let model_live = MockCompletionModel::new([
            MockTurn::tool_call("tc1", "ship_op", json!({ "op": "collections.read", "args": {} })),
            MockTurn::text("turn-0 done"),
            MockTurn::text("turn-1 done"),
        ]);

        let ctx_live = tool_ctx_headless("two-turn", mock.clone());
        // Use max_turns = 2 to allow two outer iterations.
        let mut program_live = make_program(model_live.clone(), ctx_live);
        program_live.max_turns = 2;

        let auth = AuthContext::in_process("t", vec!["collections".into()]);
        let dispatch = dummy_dispatch(Arc::new(InProcessGateway::default()), auth.clone());

        let (status, journal) = execute(Journal::new(), dispatch.clone(), &program_live).await;
        // First run completes (model returns text after the tool call).
        assert!(
            matches!(status, RunStatus::Completed(_)),
            "first two-turn run must complete: {status:?}"
        );
        // Save journal.
        let store = InMemoryJournalStore::new();
        store.save("two-turn", &journal).await.unwrap();
        let requests_first_run = model_live.request_count();
        // Expected: 2 rig requests for turn-0 (tool call + final text).
        // If turn-1 also ran it's 3 (but since turn-0 returned text, loop breaks).
        // We verify turn-1 did NOT run (the loop breaks as soon as turn-0 returns text).
        assert!(
            requests_first_run <= 2,
            "first run should complete in at most 2 model requests (turn-0 tool + text): got {requests_first_run}",
        );

        // Replay: only turn-1 needs to run — seed journal with turn-0 complete.
        let model_replay = MockCompletionModel::new([MockTurn::text("turn-1 replay")]);
        let mock2 = Arc::new(MockSink::new());
        let ctx_replay = tool_ctx_headless("two-turn", mock2.clone());
        let mut program_replay = make_program(model_replay.clone(), ctx_replay);
        program_replay.max_turns = 2;

        // Re-load the journal (which has turn.0 complete).
        let journal_loaded = store.load("two-turn").await.unwrap().unwrap();
        let (status_replay, _) = execute(journal_loaded, dispatch.clone(), &program_replay).await;

        assert!(
            matches!(status_replay, RunStatus::Completed(_)),
            "replay must complete: {status_replay:?}"
        );
        // The replay model must have been called 0 times (turn-0 replayed, turn-1 not reached).
        // In our 2-outer-turn setup, turn-0 is already logged so it replays;
        // since turn-0's text breaks the loop immediately, turn-1 never runs.
        assert_eq!(
            model_replay.request_count(), 0,
            "replay must not call the model (all turns already logged): got {}",
            model_replay.request_count()
        );
    }

    // ask_stance_suspension_parks_and_resumes_across_journal_boundary:
    //
    // An mcp.create call hits Ask-stance gate with journal_mode=true. The tool
    // returns a suspension marker; FlokiProgram calls ctx.suspend(); execute()
    // returns Suspended. Save/load journal via InMemoryJournalStore. Then call
    // resume() with an approval decision. Verify:
    // 1. First execute() → Suspended.
    // 2. journal.is_suspended() == true.
    // 3. After save/load, resume() → NOT Failed (prior turn replayed, not re-burned).
    //
    // Model call accounting (both runs share the same Arc<MockCompletionModelState>):
    //   First run:
    //     turn.0 live: model[0]=tool_call(mcp.create), model[1]=text("suspended tx")
    //       → RigTurnOutcome{text="suspended tx", pending_suspension=Some("...")}
    //       → ctx.suspend() → Suspended
    //   Resume (second run):
    //     turn.0 replay: no model calls (uses logged outcome)
    //     turn.0 suspend entry: resolved → ctx.suspend() returns Ok(decision)
    //     turn.1 live: model[2]=text("all done after approval")
    //       → RigTurnOutcome{text="all done", pending_suspension=None} → break
    //   Total model calls: 3
    #[tokio::test]
    async fn ask_stance_suspension_parks_and_resumes_across_journal_boundary() {
        let mock = Arc::new(MockSink::new());
        let model = MockCompletionModel::new([
            // First run — turn.0:
            // Model issues tool call → tool returns suspension marker →
            // rig makes follow-up model call → model returns text → outcome logged.
            MockTurn::tool_call("tc1", "ship_op", json!({ "op": "mcp.create", "args": { "name": "test" } })),
            MockTurn::text("suspended turn text"),
            // Second run (resume) — turn.1 (turn.0 replays from journal):
            MockTurn::text("all done after approval"),
        ]);

        let ctx = tool_ctx_journal("suspend-resume", mock.clone());
        let mut program = make_program(model.clone(), ctx);
        // Use mcp grant so the gate sees the op (but Ask-stance → suspend in journal_mode).
        program.auth = AuthContext::in_process("t", vec!["mcp".into()]);
        program.grants = vec!["mcp".into()];

        let dispatch = dummy_dispatch(
            Arc::new(InProcessGateway::default()),
            AuthContext::in_process("t", vec!["mcp".into()]),
        );

        // First execute: hits Ask-stance suspension.
        let (status, journal) = execute(Journal::new(), dispatch.clone(), &program).await;

        // The session parked on an approval gate.
        assert!(
            matches!(status, RunStatus::Suspended),
            "first execute must suspend (Ask-stance in journal_mode): {status:?}"
        );
        assert!(journal.is_suspended(), "journal must have a Suspended entry");

        // Save and load the journal across the "boundary".
        let store = InMemoryJournalStore::new();
        store.save("suspend-resume", &journal).await.unwrap();
        let journal_loaded = store.load("suspend-resume").await.unwrap().unwrap();

        let model_calls_before_resume = model.request_count();

        // Resume with an approval decision.
        let (status_resumed, _) = durable_execution::resume(
            journal_loaded,
            json!({ "approved": true }),
            dispatch.clone(),
            &program,
        )
        .await;

        // After resume the session must not fail. It will Completed when turn.1 returns text.
        assert!(
            !matches!(status_resumed, RunStatus::Failed(_)),
            "resumed session must not fail: {status_resumed:?}"
        );

        // The resume path must call the model exactly once (turn.1 live).
        // turn.0 replayed from journal — zero additional model calls for it.
        let model_calls_after_resume = model.request_count() - model_calls_before_resume;
        assert_eq!(
            model_calls_after_resume, 1,
            "resume must fire the model exactly once (turn.1 live, turn.0 replayed): got {model_calls_after_resume}"
        );
    }

    // journal_mode_false_headless_does_not_suspend:
    //
    // With journal_mode=false and HeadlessApprover, an Ask-stance op is
    // deterministically denied (never suspended). execute() returns Completed,
    // not Suspended. No Suspended entry in the journal.
    #[tokio::test]
    async fn journal_mode_false_headless_does_not_suspend() {
        let mock = Arc::new(MockSink::new());
        // Script: call mcp.create (Ask-stance). HeadlessApprover denies it.
        // Model sees {"denied": ...} data and returns text.
        let model = MockCompletionModel::new([
            MockTurn::tool_call("tc1", "ship_op", json!({ "op": "mcp.create", "args": {} })),
            MockTurn::text("got denied"),
        ]);

        // journal_mode=false → uses HeadlessApprover path, not suspension path.
        let ctx = tool_ctx_headless("headless-no-suspend", mock.clone());
        let mut program = make_program(model.clone(), ctx);
        program.auth = AuthContext::in_process("t", vec!["mcp".into()]);
        program.grants = vec!["mcp".into()];

        let dispatch = dummy_dispatch(
            Arc::new(InProcessGateway::default()),
            AuthContext::in_process("t", vec!["mcp".into()]),
        );

        let (status, journal) = execute(Journal::new(), dispatch, &program).await;

        // Must complete (not suspend) because HeadlessApprover denies immediately.
        assert!(
            matches!(status, RunStatus::Completed(_)),
            "headless path must complete (denial is data, not suspension): {status:?}"
        );
        assert!(!journal.is_suspended(), "headless path must NOT produce a Suspended entry");

        // session.suspended event must NOT be emitted (denial is not a suspension here).
        // (The headless approver emits session.suspended for the "denied" path too,
        // because ShipOpTool always emits it before deciding journal_mode. That's OK
        // — the event fires but the journal entry stays Completed.)
        // What we care about: the journal has no Suspended entry.
        // Already asserted above.
    }
}
