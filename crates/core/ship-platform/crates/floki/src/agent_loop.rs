//! S1.6 — the deliberately-boring loop.
//!
//! rig drives the model turn (commodity; we do not reinvent it). Our only job is
//! to make the model's tool calls land on the ONE door, gated and verified. The
//! differentiator is tool quality, not loop cleverness — so this file is small.
//!
//! `ShipOpTool` is the formal-tool path: the model emits a `ship_op` tool call,
//! it routes through `gateway.call` (where the gate runs) wrapped by the spine
//! (nothing accepted blindly). A gate denial comes back as a tool RESULT the
//! model can read and adapt to — not an exception. Real failures error.
//!
//! Code-mode (`run_script` as a loop tool, S3): deno_core's JsRuntime is
//! `!Send`, so scripts execute on ship-js's dedicated VM thread (ScriptExecutor)
//! bridged through GatewayDispatch — see RunScriptTool below.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rig::agent::AgentBuilder;
use rig::completion::{CompletionModel, Prompt, ToolDefinition};
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use agent_def::{
    Agent, SessionCancelled, SessionCompleted, SessionEventSink, SessionFailed,
    SessionStarted, SessionSuspended, TurnEnded, TurnStarted,
};
use ship_js::Dispatch;
use ship_js::deno::ScriptExecutor;

use crate::approver::UserInput;
use crate::auth::AuthContext;
use crate::dispatch::GatewayDispatch;
use crate::gateway::{Gateway, GatewayError};
use crate::sdkgen;
use crate::spine::SpineError;

/// Structured result of a headless run — what a pipeline step or an orchestrator
/// consumes.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub text: String,
    pub json: Option<Value>,
}
// ---------------------------------------------------------------------------
// Human-in-the-loop machinery: steer, cancel, approver.
// ---------------------------------------------------------------------------
//
// The loop is otherwise headless (rig drives the model turn). The three
// handles below are how a supervisor observes it (`session_sink`), injects a
// message (`steer`), or stops it (`cancel`). The `Approver` is how a
// human-in-the-loop `Ask`-stance effect is resolved.
//
// All three go through the canonical `SessionEventSink`; no side channel.
// ---------------------------------------------------------------------------
/// A steer message: a supervisor-injected user message that the model sees
/// on the next turn. Producers push (`push`); the loop drains (`drain_all`)
/// when it can surface the text to the model. Cheap clone, lock-protected
/// so any number of supervisors can share the handle.
#[derive(Default, Clone)]
pub struct Steer {
    pending: Arc<Mutex<VecDeque<String>>>,
}
impl Steer {
    pub fn new() -> Self { Self::default() }
    /// Push a steered user message. The model sees it on the next turn.
    pub fn push(&self, text: impl Into<String>) {
        self.pending.lock().unwrap().push_back(text.into());
    }
    /// Drain all pending steers in FIFO order. Called by the tool layer
    /// when it has a safe place to surface them to the model.
    pub fn drain_all(&self) -> Vec<String> {
        let mut q = self.pending.lock().unwrap();
        q.drain(..).collect()
    }
    /// Is there anything pending? Cheap peek.
    pub fn is_empty(&self) -> bool {
        self.pending.lock().unwrap().is_empty()
    }
}
/// A cooperative cancel flag. Set by a supervisor (or the run_step pre-
/// check); observed at every safe point — before each rig prompt, at the
/// start of every tool call, and at the start of every script-job dispatch.
/// The flag is NEVER used to drop a future mid-op; safe points only.
#[derive(Default, Clone)]
pub struct Cancel {
    flag: Arc<AtomicBool>,
}
impl Cancel {
    pub fn new() -> Self { Self::default() }
    pub fn cancel(&self) { self.flag.store(true, Ordering::SeqCst); }
    pub fn is_cancelled(&self) -> bool { self.flag.load(Ordering::SeqCst) }
}
/// The decision authority for an `Ask`-stance effect. Two impls: a
/// `HeadlessApprover` (denies anything — no human ⇒ deny), and a
/// `ScriptedApprover` (awaits decisions on an mpsc channel — tests push
/// scripted answers). Approvers are `Send + Sync` so they live inside
/// rig's `Send` tool futures.
#[async_trait]
pub trait Approver: Send + Sync {
    async fn request(&self, op: &str, args: &Value, request_id: &str) -> UserInput;
}
/// Headless approver: an unattended run must never block forever awaiting
/// a human. `Deny` is the deterministic resolution; the call still passes
/// through the gate (and is denied as data) — it does NOT manufacture
/// operator clearance.
pub struct HeadlessApprover;
#[async_trait]
impl Approver for HeadlessApprover {
    async fn request(&self, _op: &str, _args: &Value, request_id: &str) -> UserInput {
        UserInput::Deny { request_id: request_id.to_string(), reason: "no human in loop".into() }
    }
}
/// Scripted approver: awaits a `UserInput` on the receiver. Tests push
/// scripted decisions; the real interactive impl would push from a UI or
/// the shell. If the channel closes before a decision arrives, deny
/// (fail-closed). Uses `tokio::sync::Mutex` because the `async fn` body
/// holds the guard across the `recv().await` — a `std::sync::Mutex`
/// guard is not `Send` across an await, which would break rig's `Send`
/// tool-future bound.
pub struct ScriptedApprover {
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<UserInput>>>,
}

impl ScriptedApprover {
    /// Channel end for the test driver to push scripted decisions on.
    pub fn channel() -> (mpsc::Sender<UserInput>, Self) {
        let (tx, rx) = mpsc::channel(16);
        (tx, Self { rx: Arc::new(tokio::sync::Mutex::new(rx)) })
    }
}

#[async_trait]
impl Approver for ScriptedApprover {
    async fn request(&self, _op: &str, _args: &Value, request_id: &str) -> UserInput {
        // Stamp the decision with the request_id we were asked about — an approver
        // decides about THIS request, so its response must carry THIS id (the tool
        // validates the correlation). The scripted test pushes the decision *kind*;
        // the id is always the one passed in, never a stale value from the channel.
        match self.rx.lock().await.recv().await {
            Some(UserInput::Approve { .. }) => UserInput::Approve {
                request_id: request_id.to_string(),
            },
            Some(UserInput::Deny { reason, .. }) => UserInput::Deny {
                request_id: request_id.to_string(),
                reason,
            },
            // Fail-closed: the channel closing means no human is listening.
            None => UserInput::Deny {
                request_id: request_id.to_string(),
                reason: "approver channel closed".into(),
            },
        }
    }
}
/// Context threaded into every tool call. Carries the canonical
/// [`SessionEventSink`] (the kernel-emit port) and the session identity
/// fields that every emit call requires.
///
/// NOTE: `space_id` is threaded in at construction time by the embedder.
/// This is the minimal sensible construction — a future design step will
/// route space_id via the control-plane session context.
#[derive(Clone)]
pub struct ToolCtx {
    /// Canonical kernel-emit port. All session lifecycle events route here.
    pub session_sink: SessionEventSink,
    /// The ULID / stable id of the current session.
    pub session_id: String,
    /// The Space this session belongs to.
    pub space_id: String,
    pub approver: Arc<dyn Approver>,
    pub steer: Steer,
    pub cancel: Cancel,
    /// When `true`, `Ask`-stance ops return `{"suspended": checkpoint}` instead of
    /// blocking on `approver.request()`. The outer [`crate::durable_loop::FlokiProgram`]
    /// detects this marker and calls `ctx.suspend()`, writing a `Suspended` entry to
    /// the durable-execution journal. This is the Stage-2 journal-driven approval path.
    ///
    /// Default: `false` (interactive/headless path unchanged).
    pub journal_mode: bool,
}
impl ToolCtx {
    /// Emit `session.suspended` when an `Ask`-stance gate is awaiting a human
    /// decision. The `checkpoint` encodes the approval request so a resume
    /// can re-raise the gate at the right point.
    pub fn emit_approval_requested(&self, op: &str, _args: &Value, request_id: &str) {
        let checkpoint = format!("approval:{request_id}:{op}");
        self.session_sink
            .emit_session_suspended(
                &self.session_id,
                &SessionSuspended {
                    session_id: self.session_id.clone(),
                    checkpoint,
                },
            )
            .ok();
    }
    /// Emit `session.cancelled` from a tool that observes the cooperative
    /// cancel flag.
    pub fn emit_cancelled(&self, reason: &str) {
        self.session_sink
            .emit_session_cancelled(
                &self.session_id,
                &SessionCancelled {
                    session_id: self.session_id.clone(),
                    reason: Some(reason.to_string()),
                },
            )
            .ok();
    }
}
// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------
/// The formal op tool: one tool, every granted op. Routes through the one
/// door, with the approval handshake (`Ask` stance) and steer/cancel
/// observability layered on top. The gate is the only enforcement point.
pub struct ShipOpTool {
    pub gateway: Arc<dyn Gateway>,
    pub auth: AuthContext,
    pub ctx: ToolCtx,
}

#[derive(Deserialize)]
pub struct OpArgs {
    op: String,
    #[serde(default)]
    args: Value,
}

impl Tool for ShipOpTool {
    const NAME: &'static str = "ship_op";
    type Error = SpineError;
    type Args = OpArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Call a gated Ship op (e.g. collections.read). The daemon gate decides; \
                          a denial returns as { denied } in the result — adjust, don't pretend."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "op":   { "type": "string", "description": "dotted op name, e.g. collections.read" },
                    "args": { "type": "object" }
                },
                "required": ["op"]
            }),
        }
    }

    async fn call(&self, a: OpArgs) -> Result<String, SpineError> {
        // Cooperative cancel: safe point at the top of the call. If the
        // flag is set, we do NOT call the gateway — the effect is not
        // half-applied. Surface the cancellation as data the model can
        // read (and as a `session.cancelled` event for supervisors), then return.
        if self.ctx.cancel.is_cancelled() {
            self.ctx.emit_cancelled("user-requested");
            return Ok(json!({ "cancelled": "user-requested" }).to_string());
        }

        match self.gateway.call(&a.op, a.args.clone(), &self.auth, crate::gateway::GatewayCallKind::First).await {
            // VERIFY: nothing accepted blindly — the result must confirm itself.
            Ok(v) => {
                let body = if v.get("ok").and_then(Value::as_bool) == Some(true) {
                    v.to_string()
                } else {
                    return Err(SpineError::Unverified(format!(
                        "op `{}` did not confirm success",
                        a.op
                    )));
                };
                Ok(annotate_with_steer(&self.ctx, body))
            }
            // A gate denial is a RESULT the model reads, not an exception.
            Err(GatewayError::Denied(msg)) => {
                Ok(annotate_with_steer(&self.ctx, json!({ "denied": msg }).to_string()))
            }
            // Ask stance: the gate would allow, but a human must approve.
            // Emit session.suspended, await the decision, then re-dispatch on
            // Approve (the gate still runs) or surface as a denial
            // result on Deny. The op is NEVER dispatched before approval.
            Err(GatewayError::Ask { op, args, request_id }) => {
                self.ctx.emit_approval_requested(&op, &args, &request_id);
                // Journal-driven path (Stage 2): instead of blocking on the approver,
                // return a structured suspension marker so the outer FlokiProgram can
                // call ctx.suspend() and write a Suspended entry to the journal. The
                // marker is visible to the model as a tool result it can read.
                if self.ctx.journal_mode {
                    let checkpoint = format!("approval:{request_id}:{op}");
                    return Ok(annotate_with_steer(
                        &self.ctx,
                        json!({ "suspended": checkpoint }).to_string(),
                    ));
                }
                let decision = self.ctx.approver.request(&op, &args, &request_id).await;
                // Correlation guard: the decision MUST be about the request we
                // raised. A response carrying a different id (a buggy/hostile
                // approver) is fail-closed as a denial — never re-dispatched.
                let decision_id = match &decision {
                    UserInput::Approve { request_id } => request_id,
                    UserInput::Deny { request_id, .. } => request_id,
                };
                if decision_id != &request_id {
                    return Ok(annotate_with_steer(
                        &self.ctx,
                        json!({ "denied": format!(
                            "approval response id `{decision_id}` did not match request `{request_id}`"
                        ) })
                        .to_string(),
                    ));
                }
                match decision {
                    UserInput::Approve { .. } => match self.gateway.call(&op, args, &self.auth, crate::gateway::GatewayCallKind::Approved).await {
                        Ok(v) => {
                            if v.get("ok").and_then(Value::as_bool) == Some(true) {
                                Ok(annotate_with_steer(&self.ctx, v.to_string()))
                            } else {
                                Err(SpineError::Unverified(format!(
                                    "op `{op}` did not confirm success"
                                )))
                            }
                        }
                        Err(GatewayError::Denied(msg)) => {
                            Ok(annotate_with_steer(&self.ctx, json!({ "denied": msg }).to_string()))
                        }
                        Err(e) => Err(SpineError::Execute(e.to_string())),
                    },
                    UserInput::Deny { reason, .. } => Ok(annotate_with_steer(
                        &self.ctx,
                        json!({ "denied": format!("approval denied: {reason}") }).to_string(),
                    )),
                }
            }
            // A real failure (unknown op, dispatch error) surfaces as an error.
            Err(e) => Err(SpineError::Execute(e.to_string())),
        }
    }
}

/// If the `Steer` queue has pending messages, drain them and prefix the
/// tool's output so the model observes them on the next turn. This is the
/// only way a steer reaches the model: via a tool result the model reads
/// before its next decision. Pure helper: no effect, no allocation if
/// nothing is pending.
fn annotate_with_steer(ctx: &ToolCtx, body: String) -> String {
    let pending = ctx.steer.drain_all();
    if pending.is_empty() {
        body
    } else {
        // JSON-encode the steered text as a structured side-channel value
        // so the model can distinguish it from the op's own result.
        let steered: Vec<Value> = pending
            .into_iter()
            .map(|t| json!({ "steer": t }))
            .collect();
        json!({ "result": Value::String(body), "steers": steered }).to_string()
    }
}

/// Code-mode tool: the model writes JS that imports the scoped `ship` SDK. Runs
/// on the dedicated VM thread (the `!Send` bridge) so it composes inside rig's
/// `Send` loop. Best for composition/computation; ship_op is for single atoms.
pub struct RunScriptTool {
    pub executor: Arc<ScriptExecutor>,
    /// The one door. Captured here (with `auth`) into a `Dispatch` per run, so
    /// ship-js's auth-agnostic executor still routes every op through the gate.
    pub gateway: Arc<dyn Gateway>,
    pub grants: Vec<String>,
    pub auth: AuthContext,
    pub ctx: ToolCtx,
}

#[derive(Deserialize)]
pub struct ScriptArgs {
    src: String,
}

impl Tool for RunScriptTool {
    const NAME: &'static str = "run_script";
    type Error = SpineError;
    type Args = ScriptArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Run JavaScript that imports the scoped `ship` SDK (only granted ops are \
                          importable; the gate still enforces). Returns values the script emits. \
                          Prefer this for multi-step work; use ship_op for a single op."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": { "src": { "type": "string", "description": "ES module source; import from \"ship\"" } },
                "required": ["src"]
            }),
        }
    }

    async fn call(&self, a: ScriptArgs) -> Result<String, SpineError> {
        // Cooperative cancel: same safe-point contract as `ShipOpTool`.
        // The script is dispatched on the VM thread; we check here BEFORE
        // queueing the job so no V8 work is started after a cancel. A
        // job already in flight when cancel fires is observed between
        // spawns (the executor's recv loop checks the flag).
        if self.ctx.cancel.is_cancelled() {
            self.ctx.emit_cancelled("user-requested");
            return Ok(json!({ "cancelled": "user-requested" }).to_string());
        }
        // Soft layer: the scoped SDK, exposing only granted services. Hard
        // layer: the gate, reached through the `Dispatch` adapter over
        // (gateway, auth). ship-js runs the module on its !Send VM thread and
        // returns whatever the script `op_emit`ted.
        let dispatch: Arc<dyn Dispatch> =
            Arc::new(GatewayDispatch::new(self.gateway.clone(), self.auth.clone()));
        let sdk = sdkgen::generate_sdk(&self.grants);
        let emitted = self
            .executor
            .run(dispatch, sdk, a.src)
            .await
            .map_err(|e| SpineError::Execute(e.to_string()))?;
        let body = serde_json::to_string(&emitted).unwrap_or_else(|_| "[]".into());
        Ok(annotate_with_steer(&self.ctx, body))
    }
}

/// Run an agent to completion: rig drives the turn loop; our tools make every
/// effect pass the gate + spine. `ship_op` is the atomic path; `run_script` is
/// code-mode via the VM-thread bridge. `ctx` threads the supervisor's
/// `session_sink`/`approver`/`steer`/`cancel` into every tool call. Headless;
/// returns a structured result. Honors the cancel flag at the safe point before
/// invoking rig.
///
/// Emits the canonical session lifecycle events:
/// - `session.started`  at entry (unless already cancelled)
/// - `session.turn.started` / `session.turn.ended` bracketing the rig prompt
/// - `session.completed` on clean exit
/// - `session.failed` on rig error
/// - `session.cancelled` on cooperative cancel (via `ToolCtx::emit_cancelled`)
pub async fn run_step<M>(
    model: M,
    agent: &Agent,
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
    executor: Arc<ScriptExecutor>,
    ctx: ToolCtx,
    input: &str,
) -> anyhow::Result<AgentOutput>
where
    M: CompletionModel + 'static,
{
    // Cooperative cancel: safe point before the rig prompt. If the flag
    // is already set, do NOT call rig; emit `session.cancelled` and return a
    // benign result. The rig future itself cannot be cancelled mid-
    // flight (that would be a dropped future mid-op); if a cancel fires
    // while rig is running, the next tool call inside rig observes the
    // flag and short-circuits, then the post-prompt check below sees
    // it and emits `session.cancelled`.
    if ctx.cancel.is_cancelled() {
        ctx.emit_cancelled("user-requested");
        return Ok(AgentOutput { text: String::new(), json: None });
    }

    // Emit session.started — the canonical session-open event.
    // Floki sessions have no external slug identity (they are not addressable
    // by the cancel or sweep paths the way harness-driver sessions are).
    ctx.session_sink
        .emit_session_started(
            &ctx.session_id,
            &SessionStarted {
                session_id: ctx.session_id.clone(),
                agent_id: agent.identity.id.clone(),
                harness: "floki".into(),
                model: agent.disposition.model.clone(),
                space_id: ctx.space_id.clone(),
                slug: None,
                input: Some(input.to_string()),
                parent: None,
            },
        )
        .ok();

    // Emit turn.started — one turn per run_step invocation (rig drives
    // internal model turns; we bracket the whole prompt call as turn 0).
    ctx.session_sink
        .emit_turn_started(
            &ctx.session_id,
            &TurnStarted { session_id: ctx.session_id.clone(), idx: 0 },
        )
        .ok();

    let preamble = if agent.disposition.rules.is_empty() {
        "You are a Ship agent. Use ship_op for a single gated op, or run_script for \
         multi-step work via the ship SDK. If an op returns { denied }, adjust — \
         do not pretend it succeeded."
    } else {
        &agent.disposition.rules
    };

    // The gate's grant list = the capability domains the agent declares. Blast
    // is intrinsic to each op (looked up by the gateway), never claimed here.
    let grants: Vec<String> = agent
        .capabilities
        .iter()
        .map(|c| c.domain.clone())
        .collect();

    let runner = AgentBuilder::new(model)
        .preamble(preamble)
        .tool(ShipOpTool {
            gateway: gateway.clone(),
            auth: auth.clone(),
            ctx: ctx.clone(),
        })
        .tool(RunScriptTool {
            executor,
            gateway,
            grants,
            auth,
            ctx: ctx.clone(),
        })
        .build();

    let result = runner
        .prompt(input)
        .max_turns(agent.disposition.max_turns.unwrap_or(10))
        .await;

    // Post-prompt cancel check: if cancel fired during the rig run, a
    // tool call already emitted `session.cancelled`; we surface the result and
    // let the caller decide. No effect is left half-applied because
    // every tool call checked the flag at its safe point.
    if ctx.cancel.is_cancelled() {
        ctx.emit_cancelled("user-requested");
    }

    match result {
        Ok(text) => {
            // Emit turn.ended then session.completed.
            ctx.session_sink
                .emit_turn_ended(
                    &ctx.session_id,
                    &TurnEnded {
                        session_id: ctx.session_id.clone(),
                        idx: 0,
                        stop_reason: Some("end_turn".into()),
                        ..Default::default()
                    },
                )
                .ok();
            ctx.session_sink
                .emit_session_completed(
                    &ctx.session_id,
                    &SessionCompleted { session_id: ctx.session_id.clone() },
                )
                .ok();
            Ok(AgentOutput { text, json: None })
        }
        Err(e) => {
            // Emit session.failed for any rig-level error.
            ctx.session_sink
                .emit_session_failed(
                    &ctx.session_id,
                    &SessionFailed {
                        session_id: ctx.session_id.clone(),
                        reason: e.to_string(),
                    },
                )
                .ok();
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use agent_def::{BlastRadius, Capability, Disposition, Harness, Identity, SessionEventSink};
    use async_trait::async_trait;
    use rig::test_utils::{MockCompletionModel, MockTurn};
    use ship_kernel::testing::MockSink;

    use crate::approver::UserInput;
    use crate::gateway::{GatewayCallKind, InProcessGateway};
    use super::*;

    fn auth(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    /// A minimal Floki `Agent` for the loop tests: `floki` harness, the given
    /// model, and one capability per granted domain (blast is irrelevant to the
    /// gate — it reads each op's intrinsic blast — so C2 is an arbitrary stand-in).
    fn demo_agent(model: &str, grants: &[&str]) -> Agent {
        Agent {
            harness: Harness {
                id: "floki".into(),
                settings: Default::default(),
            },
            identity: Identity {
                id: "t".into(),
                name: "t".into(),
                version: "0.1.0".into(),
                description: String::new(),
                tags: vec![],
            },
            disposition: Disposition {
                rules: String::new(),
                model: Some(model.into()),
                temperature: None,
                max_turns: Some(20),
            },
            capabilities: grants
                .iter()
                .map(|g| Capability::new(*g, BlastRadius::C2Elevated))
                .collect(),
            skills: vec![],
            mcp: vec![],
        }
    }

    /// Build a `ToolCtx` wired to a `HeadlessApprover` and a fresh `MockSink`.
    /// Tests that need a scripted approver / a pre-set cancel / pending
    /// steers build their own.
    fn test_ctx() -> (ToolCtx, Arc<MockSink>) {
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let ctx = ToolCtx {
            session_sink,
            session_id: "test-session".into(),
            space_id: "test-space".into(),
            approver: Arc::new(HeadlessApprover),
            steer: Steer::new(),
            cancel: Cancel::new(),
            journal_mode: false,
        };
        (ctx, mock)
    }

    // ── tool-level behavior (deterministic, no model) ──────────────────────
    #[tokio::test]
    async fn tool_returns_result_for_granted_op() {
        let (ctx, _mock) = test_ctx();
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway::default()), auth: auth(&["collections"]), ctx };
        let out = tool
            .call(OpArgs { op: "collections.read".into(), args: json!({}) })
            .await
            .expect("granted op should succeed");
        assert!(!out.contains("denied"), "granted op must not be denied");
    }

    #[tokio::test]
    async fn tool_surfaces_denial_as_result_not_error() {
        let (ctx, _mock) = test_ctx();
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway::default()), auth: auth(&["collections"]), ctx };
        // runtime is ungranted; the gate denies the C2 op. The tool returns it
        // as DATA (`{ denied }`), not a thrown error.
        let out = tool
            .call(OpArgs { op: "runtime.dispatch".into(), args: json!({}) })
            .await
            .expect("denial is a result, not a tool error");
        assert!(out.contains("denied"), "ungranted op must come back denied");
    }

    // ── loop behavior: a model tool call routes through the gateway ─────────
    /// Records the ops the loop actually dispatched, to prove the model's tool
    /// call reached the one door.
    struct RecordingGateway {
        inner: InProcessGateway,
        seen: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl Gateway for RecordingGateway {
        async fn call(&self, op: &str, args: Value, a: &AuthContext, _kind: GatewayCallKind) -> Result<Value, GatewayError> {
            self.seen.lock().unwrap().push(op.to_string());
            self.inner.call(op, args, a, _kind).await
        }
    }

    #[tokio::test]
    async fn model_tool_call_routes_through_the_gate() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let gw = Arc::new(RecordingGateway { inner: InProcessGateway::default(), seen: seen.clone() });

        // Scripted model: call ship_op once, then answer.
        let model = MockCompletionModel::new([
            MockTurn::tool_call("1", "ship_op", json!({ "op": "collections.read", "args": {} })),
            MockTurn::text("done"),
        ]);
        let agent = demo_agent("mock", &["collections"]);
        let exec = Arc::new(ScriptExecutor::spawn());
        let (ctx, _mock) = test_ctx();

        let out = run_step(model, &agent, gw, auth(&["collections"]), exec, ctx, "read the leads")
            .await
            .expect("loop should complete");

        // Behavioral: the loop finished AND the op the model requested actually
        // crossed the gateway — the model→tool→one-door path is wired.
        assert!(out.text.contains("done"));
        assert!(
            seen.lock().unwrap().iter().any(|op| op == "collections.read"),
            "the model's tool call must route through the gateway"
        );
    }

    // ── the !Send bridge: code-mode runs on the VM thread, gated, and the
    //    tool future is Send (it compiles as a rig tool at all) ──────────────
    #[tokio::test]
    async fn run_script_tool_executes_js_through_gate_off_thread() {
        let exec = Arc::new(ScriptExecutor::spawn());
        let (ctx, _mock) = test_ctx();
        let tool = RunScriptTool {
            executor: exec,
            gateway: Arc::new(InProcessGateway::default()),
            grants: vec!["collections".into()],
            auth: auth(&["collections"]),
            ctx,
        };
        // JS imports the scoped SDK (granted) AND raw-calls an ungranted op.
        let src = r#"
            import { collections } from "ship";
            const granted = await collections.read({});
            const raw_ungranted = await Deno.core.ops.op_ship_call("runtime.dispatch", {});
            Deno.core.ops.op_emit({ granted, raw_ungranted });
        "#
        .to_string();
        let out = tool
            .call(ScriptArgs { src })
            .await
            .expect("script should run on the VM thread")
            .to_lowercase();
        // The gate ran on the dedicated thread: the granted op is clean, the
        // ungranted raw op is denied (GatewayError Display is "DENIED: ...").
        assert!(out.contains("denied"), "ungranted op must be gate-denied in code-mode");
    }

    // The model drives code-mode end to end: a run_script tool call executes JS
    // through the bridge and the loop completes.
    #[tokio::test]
    async fn loop_drives_code_mode_via_run_script() {
        let exec = Arc::new(ScriptExecutor::spawn());
        let model = MockCompletionModel::new([
            MockTurn::tool_call(
                "1",
                "run_script",
                json!({ "src": "import { collections } from \"ship\"; const r = await collections.read({}); Deno.core.ops.op_emit(r);" }),
            ),
            MockTurn::text("counted the leads"),
        ]);
        let agent = demo_agent("mock", &["collections"]);
        let (ctx, _mock) = test_ctx();
        let out = run_step(
            model,
            &agent,
            Arc::new(InProcessGateway::default()),
            auth(&["collections"]),
            exec,
            ctx,
            "how many leads?",
        )
        .await
        .expect("code-mode loop should complete");
        assert!(out.text.contains("counted"));
    }

    // ── approval handshake: Ask stance ops route through the approver ─────
    #[tokio::test]
    async fn ask_with_approval_proceeds_through_the_gate() {
        let (tx, scripted) = ScriptedApprover::channel();
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let ctx = ToolCtx {
            session_sink,
            session_id: "test-session".into(),
            space_id: "test-space".into(),
            approver: Arc::new(scripted),
            steer: Steer::new(),
            cancel: Cancel::new(),
            journal_mode: false,
        };
        // mcp.create is ask-stance; the agent must have the mcp grant.
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway::default()), auth: auth(&["mcp"]), ctx };

        // Script: approve the request when it arrives.
        tx.send(UserInput::Approve { request_id: "x".into() }).await.unwrap();

        let out = tool
            .call(OpArgs { op: "mcp.create".into(), args: json!({}) })
            .await
            .expect("approval + dispatch should produce a result");
        // Behavioral: the request was approved AND the op actually ran (no
        // `denied` in the output).
        assert!(!out.contains("denied"), "approved op must not surface as denied: {out}");
        // The supervisor saw a session.suspended event (approval gate raised).
        let evs = mock.captured();
        assert!(
            evs.iter().any(|e| e.event_type == "session.suspended"),
            "approval gate must emit session.suspended; got: {evs:?}"
        );
    }

    // Ask + Deny: the op is NEVER dispatched. The denial is surfaced as
    // data (`{"denied": "approval denied: ..."}`), not an error.
    #[tokio::test]
    async fn ask_with_denial_surfaces_as_data_no_effect_dispatched() {
        // Wrap the gateway so we can count its calls — the Ask probe is
        // one call; the re-dispatch on approval would be a second. A
        // deny must result in exactly ONE call (the probe) and no echo.
        struct CountingGateway { inner: InProcessGateway, calls: Arc<Mutex<Vec<String>>> }
        #[async_trait]
        impl Gateway for CountingGateway {
            async fn call(&self, op: &str, args: Value, a: &AuthContext, _kind: GatewayCallKind) -> Result<Value, GatewayError> {
                self.calls.lock().unwrap().push(op.to_string());
                self.inner.call(op, args, a, _kind).await
            }
        }

        let (tx, scripted) = ScriptedApprover::channel();
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let calls = Arc::new(Mutex::new(Vec::new()));
        let gw = Arc::new(CountingGateway { inner: InProcessGateway::default(), calls: calls.clone() });
        let ctx = ToolCtx {
            session_sink,
            session_id: "test-session".into(),
            space_id: "test-space".into(),
            approver: Arc::new(scripted),
            steer: Steer::new(),
            cancel: Cancel::new(),
            journal_mode: false,
        };
        let tool = ShipOpTool { gateway: gw, auth: auth(&["mcp"]), ctx };

        tx.send(UserInput::Deny { request_id: "x".into(), reason: "not now".into() }).await.unwrap();

        let out = tool
            .call(OpArgs { op: "mcp.create".into(), args: json!({ "name": "blocked" }) })
            .await
            .expect("denial is data, not a tool error");
        assert!(out.contains("denied"), "denied approval must surface as denied data: {out}");
        assert!(out.contains("not now"), "the human's reason should be visible: {out}");
        // With deny, only the probe runs (no re-dispatch).
        let seen = calls.lock().unwrap().clone();
        assert_eq!(seen.len(), 1, "denied ask must not re-dispatch: calls={seen:?}");
        // The session.suspended event WAS emitted before the denial.
        assert!(
            mock.captured().iter().any(|e| e.event_type == "session.suspended"),
            "approval gate must emit session.suspended even when denied"
        );
    }

    // Headless approver: an `Ask` for an unattended run deterministically
    // denies, surfaces as data, and does not hang waiting for a human.
    #[tokio::test]
    async fn headless_ask_resolves_to_deny_deterministically() {
        let (ctx, mock) = test_ctx();
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway::default()), auth: auth(&["mcp"]), ctx };
        // No scripted approver; the test_ctx uses HeadlessApprover. A
        // headless run must NEVER block waiting for a human.
        let out = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            tool.call(OpArgs { op: "mcp.create".into(), args: json!({}) }),
        )
        .await
        .expect("headless ask must not hang")
        .expect("denial is data, not an error");
        assert!(out.contains("denied"));
        // The reason should be the headless sentinel so callers can tell.
        assert!(out.contains("no human in loop"));
        // The session.suspended event was emitted.
        assert!(
            mock.captured().iter().any(|e| e.event_type == "session.suspended"),
            "headless ask must still emit session.suspended"
        );
    }

    // Journal-mode Ask: when `journal_mode = true`, an Ask-stance op returns a
    // `{"suspended": "approval:<request_id>:<op>"}` marker immediately without
    // blocking on the approver. The outer FlokiProgram detects this and parks
    // the journal run via `ctx.suspend()`. The approver is never called.
    #[tokio::test]
    async fn ask_in_journal_mode_returns_suspension_marker() {
        struct PanicApprover; // must never be invoked in journal mode.
        #[async_trait]
        impl Approver for PanicApprover {
            async fn request(&self, op: &str, _args: &Value, _id: &str) -> UserInput {
                panic!("approver must NOT be called in journal_mode; op={op}");
            }
        }
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let ctx = ToolCtx {
            session_sink,
            session_id: "test-session".into(),
            space_id: "test-space".into(),
            approver: Arc::new(PanicApprover),
            steer: Steer::new(),
            cancel: Cancel::new(),
            journal_mode: true,
        };
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway::default()), auth: auth(&["mcp"]), ctx };

        let out = tool
            .call(OpArgs { op: "mcp.create".into(), args: json!({}) })
            .await
            .expect("journal-mode ask must return data, not an error");

        // Must contain the suspension marker, not a denial or error.
        assert!(out.contains("\"suspended\""), "journal-mode ask must return suspension marker: {out}");
        assert!(out.contains("approval:"), "suspension marker must include approval: prefix: {out}");
        assert!(out.contains("mcp.create"), "suspension marker must name the op: {out}");
        // session.suspended event must still be emitted.
        assert!(
            mock.captured().iter().any(|e| e.event_type == "session.suspended"),
            "journal-mode ask must still emit session.suspended; got: {:?}", mock.captured()
        );
    }

    // Steer: text pushed to the Steer handle is observed by the model on
    // the next tool result (and thus on the next rig turn). The acceptance
    // test is structural: the tool result contains the steered text.
    #[tokio::test]
    async fn steered_text_is_visible_to_the_next_tool_result() {
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let steer = Steer::new();
        let ctx = ToolCtx {
            session_sink,
            session_id: "test-session".into(),
            space_id: "test-space".into(),
            approver: Arc::new(HeadlessApprover),
            steer: steer.clone(),
            cancel: Cancel::new(),
            journal_mode: false,
        };
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway::default()), auth: auth(&["collections"]), ctx };

        // Run a baseline op (no steer queued) — result is plain.
        let out1 = tool
            .call(OpArgs { op: "collections.read".into(), args: json!({}) })
            .await
            .expect("baseline");
        assert!(!out1.contains("steers"));

        // Push a steer; the NEXT tool call must surface it.
        steer.push("please focus on Q4");
        let out2 = tool
            .call(OpArgs { op: "collections.read".into(), args: json!({}) })
            .await
            .expect("steer-annotated");
        assert!(out2.contains("please focus on Q4"), "steer must be visible: {out2}");
        assert!(out2.contains("\"steers\""), "steer must be in the side-channel: {out2}");

        // After draining, no more steers.
        assert!(steer.is_empty(), "steer must be drained on read");
    }

    // Cancel: a pre-set cancel flag short-circuits the tool at the safe
    // point — the gateway is NEVER called, the `session.cancelled` event is
    // emitted, and the tool returns data the model can read.
    #[tokio::test]
    async fn cancel_short_circuits_the_tool_at_the_safe_point() {
        struct PanicOnCall; // must never be invoked.
        #[async_trait]
        impl Gateway for PanicOnCall {
            async fn call(&self, op: &str, _args: Value, _a: &AuthContext, _kind: GatewayCallKind) -> Result<Value, GatewayError> {
                panic!("gateway must NOT be called after a cancel: op={op}");
            }
        }
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let cancel = Cancel::new();
        cancel.cancel(); // pre-set
        let ctx = ToolCtx {
            session_sink,
            session_id: "test-session".into(),
            space_id: "test-space".into(),
            approver: Arc::new(HeadlessApprover),
            steer: Steer::new(),
            cancel,
            journal_mode: false,
        };
        let tool = ShipOpTool { gateway: Arc::new(PanicOnCall), auth: auth(&["collections"]), ctx };

        let out = tool
            .call(OpArgs { op: "collections.read".into(), args: json!({}) })
            .await
            .expect("cancel surfaces as data, not a tool error");
        assert!(out.contains("cancelled"), "tool must surface cancellation as data: {out}");
        // The session.cancelled event was emitted.
        assert!(
            mock.captured().iter().any(|e| e.event_type == "session.cancelled"),
            "cancel must emit session.cancelled; got: {:?}", mock.captured()
        );
    }
    // run_step with a pre-cancelled run: emits session.cancelled and returns
    // without invoking the rig prompt (the model is never touched). The
    // mock is given a scripted turn; if rig were invoked, the turn
    // would be consumed and the model's request count would be > 0.
    #[tokio::test]
    async fn run_step_emits_cancelled_when_already_cancelled() {
        let model = MockCompletionModel::new([MockTurn::text("unseen")]);
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        let cancel = Cancel::new();
        cancel.cancel();
        let ctx = ToolCtx {
            session_sink,
            session_id: "test-session".into(),
            space_id: "test-space".into(),
            approver: Arc::new(HeadlessApprover),
            steer: Steer::new(),
            cancel,
            journal_mode: false,
        };
        let agent = demo_agent("mock", &["collections"]);
        let exec = Arc::new(ScriptExecutor::spawn());
        let out = run_step(
            model.clone(),
            &agent,
            Arc::new(InProcessGateway::default()),
            auth(&["collections"]),
            exec,
            ctx,
            "anything",
        )
        .await
        .expect("cancelled run is a clean result, not an error");
        // No model output; session.cancelled event was emitted; rig was never called.
        assert!(out.text.is_empty());
        assert_eq!(model.request_count(), 0, "model must not be invoked when cancel is pre-set");
        assert!(
            mock.captured().iter().any(|e| e.event_type == "session.cancelled"),
            "pre-cancelled run must emit session.cancelled; got: {:?}", mock.captured()
        );
    }

    // Code-mode cannot run the interactive approval handshake, so an ask-stance
    // op (granted service, but human-approval-required) comes back to the script
    // as a TERMINAL denial it reads as data — not an approval prompt, not success.
    #[tokio::test]
    async fn run_script_tool_ask_stance_surfaces_as_terminal_denial() {
        let exec = Arc::new(ScriptExecutor::spawn());
        let (ctx, _mock) = test_ctx();
        let tool = RunScriptTool {
            executor: exec,
            gateway: Arc::new(InProcessGateway::default()),
            grants: vec!["mcp".into()],
            auth: auth(&["mcp"]),
            ctx,
        };
        // mcp.create is granted (mcp in grants) but ask-stance. In code-mode the
        // gate's Ask flattens to a terminal denial mentioning approval.
        let src = r#"
            import { mcp } from "ship";
            const r = await mcp.create({ name: "x" });
            Deno.core.ops.op_emit(r);
        "#
        .to_string();
        let out = tool
            .call(ScriptArgs { src })
            .await
            .expect("script runs")
            .to_lowercase();
        assert!(
            out.contains("approval"),
            "an ask-stance op in code-mode must surface as a terminal denial mentioning approval: {out}",
        );
    }

    // The whole-loop denial path: a model tool call for an UNGRANTED op is denied
    // as DATA (not an exception), so the loop does not error — it completes and
    // the model takes its next turn. (An exception would propagate as Err here.)
    #[tokio::test]
    async fn model_tool_call_for_ungranted_op_does_not_break_the_loop() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let gw = Arc::new(RecordingGateway {
            inner: InProcessGateway::default(),
            seen: seen.clone(),
        });
        let model = MockCompletionModel::new([
            // runtime.dispatch is C2, ungranted for a collections-only agent.
            MockTurn::tool_call("1", "ship_op", json!({ "op": "runtime.dispatch", "args": {} })),
            MockTurn::text("that was denied — adjusting"),
        ]);
        let agent = demo_agent("mock", &["collections"]);
        let exec = Arc::new(ScriptExecutor::spawn());
        let (ctx, _mock) = test_ctx();
        let out = run_step(
            model,
            &agent,
            gw,
            auth(&["collections"]),
            exec,
            ctx,
            "spawn a child",
        )
        .await
        .expect("the loop completes even though the requested op was denied");
        assert!(out.text.contains("adjusting"), "the model continues after the denial");
        assert!(
            seen.lock().unwrap().iter().any(|op| op == "runtime.dispatch"),
            "the ungranted op still crossed the gateway (and was denied as data)",
        );
    }
}
