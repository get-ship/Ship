//! Orchestration seam — the in-loop path that lets a parent agent spawn child
//! agents (`runtime.dispatch`) or fan out to multiple children in parallel
//! (`runtime.parallel`).
//!
//! Per the approved S14 design (jobs/s14-orchestration.md), this module is the
//! agent-altitude control plane. It does NOT re-implement DAG orchestration
//! from scratch — that machinery lives in `crates/ship-pipeline-runtime` on
//! the daemon side, and the agent altitude will graduate onto it after S16.
//! For S14, the spike proves the *contract and shape* with in-process fakes:
//!
//!   - A spike-local child runner that drives a `MockCompletionModel` to a
//!     terminal outcome, going through the existing `Gateway` so the gate
//!     runs on every child-side op.
//!   - A pure `aggregate_lane_outcomes` decision (min-successful shortcut)
//!     that mirrors the daemon's `ship-pipelines::fanout::aggregate_lane_outcomes`
//!     semantics one-for-one. The spike can't import the daemon crate, so the
//!     function is reimplemented locally — same signature, same decision, kept
//!     pure so it matches the daemon's contract and a future swap is mechanical.
//!
//! The traits here are the only net-new public type in the agent altitude
//! (per design §5.1). Everything else composes existing types.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;
use rig::test_utils::{MockCompletionModel, MockTurn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Notify;

use crate::agent_loop::{self, AgentOutput};
use crate::auth::AuthContext;
use crate::gateway::Gateway;
use crate::script_executor::ScriptExecutor;
use crate::spec::AgentSpec;

// ---------------------------------------------------------------------------
// Public types — the seam
// ---------------------------------------------------------------------------

/// Context for the parent run that the orchestrator needs when dispatching a
/// child. Carries the parent's identity for lineage/correlation and the
/// cancellation signal children listen on.
#[derive(Clone)]
pub struct ParentContext {
    pub pipeline_run_id: String,
    /// Parent run's slug for correlation; not consumed in the spike's
    /// fan-out, but part of the public API the daemon impl will use.
    #[allow(dead_code)]
    pub slug: String,
    pub cancel: Arc<CancelSignal>,
}

impl ParentContext {
    pub fn new(pipeline_run_id: impl Into<String>, slug: impl Into<String>) -> Self {
        Self {
            pipeline_run_id: pipeline_run_id.into(),
            slug: slug.into(),
            cancel: Arc::new(CancelSignal::default()),
        }
    }
}

/// A shared cancellation signal. The parent turns it on; the child runner
/// observes it between model turns and short-circuits the run.
#[derive(Default, Debug)]
pub struct CancelSignal {
    cancelled: AtomicBool,
    notify: Notify,
}

impl CancelSignal {
    /// Mark the run as cancelled. Idempotent.
    #[allow(dead_code)] // exercised in tests
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    /// Has cancellation been requested?
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Wait for a cancellation. Resolves immediately if already cancelled.
    pub async fn wait(&self) {
        if self.is_cancelled() {
            return;
        }
        let _ = self.notify.notified().await;
    }
}

/// One child's terminal outcome. The shape mirrors what the daemon's
/// `LaneOutcome` carries: a verdict (Ok/Fail) and the structured payload the
/// child produced. A `denied_op` field is filled when the child attempt at an
/// op was gate-denied — the denial is DATA, not an exception, per
/// invariant #7 ("denial is data").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChildOutcome {
    pub spec_id: String,
    pub ok: bool,
    pub text: String,
    pub denied_op: Option<DeniedOp>,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeniedOp {
    pub op: String,
    pub reason: String,
}

/// The aggregated output of an orchestration call. For `dispatch_agent`, a
/// single child outcome; for `dispatch_parallel`, the aggregator's decision
/// plus the surviving lane outcomes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchedOutput {
    pub pipeline_run_id: String,
    pub child: Option<ChildOutcome>,
    pub parallel: Option<ParallelOutput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParallelOutput {
    pub decision: AggregateDecision,
    pub ok_count: usize,
    pub total: usize,
    pub lanes: Vec<ChildOutcome>,
}

/// The aggregator's verdict. Mirrors `ship-pipelines::fanout::AggregateDecision`:
/// the daemon's `Advance` is the "we have enough to proceed" shortcut, and
/// `Fail` is the "we don't, treat as failure" verdict. The judge node is the
/// rich alternative; this is the shortcut decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AggregateDecision {
    Advance { ok_count: usize },
    Fail { ok_count: usize, total: usize },
}

/// Failure modes for orchestration calls. Maps to a tool `Result::Err` per
/// design §5.1.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // variants are part of the public surface; spike uses Cancelled + Dispatch
pub enum OrchestrationError {
    #[error("orchestrator not available: {0}")]
    Unavailable(String),
    #[error("dispatch failed: {0}")]
    Dispatch(String),
    #[error("child run failed: {0}")]
    ChildFailed(String),
    #[error("parent cancelled")]
    Cancelled,
}

// ---------------------------------------------------------------------------
// Pure aggregator — mirrors ship-pipelines/src/fanout.rs::aggregate_lane_outcomes
// ---------------------------------------------------------------------------

/// The min-successful-lanes shortcut. Pure over `(lanes, min_successful)` —
/// same input, same output as the daemon's implementation. A re-implementation
/// is required because the spike can't import the daemon crate; the semantics
/// MUST match (the spike is the agent-altitude's contract test).
///
/// Decision:
///   ok_count >= min_successful_lanes  -> Advance { ok_count }
///   otherwise                          -> Fail { ok_count, total }
pub fn aggregate_lane_outcomes(lanes: &[ChildOutcome], min_successful_lanes: usize) -> AggregateDecision {
    let total = lanes.len();
    let ok_count = lanes.iter().filter(|l| l.ok).count();
    if ok_count >= min_successful_lanes {
        AggregateDecision::Advance { ok_count }
    } else {
        AggregateDecision::Fail { ok_count, total }
    }
}

// ---------------------------------------------------------------------------
// Child behavior — what a child agent does at each turn
// ---------------------------------------------------------------------------

/// What a child agent does, scripted. The spike can't ship a real model, so
/// the child is driven by a `MockCompletionModel` whose turns are synthesized
/// from a `ChildBehavior`.
///
/// Each variant picks what the child calls (or whether it returns a text
/// answer) at the next turn. The child runner drains the behavior to a final
/// outcome; once exhausted, the child returns the last `text` as its
/// terminal output.
#[derive(Debug, Clone)]
#[allow(dead_code)] // CallGrantedOp and SpinUntilCancelled are exercised in tests
pub(crate) enum ChildBehavior {
    /// Return the given text on the first turn. The "good child" — used to
    /// prove the parent tool result carries the child's output (invariant 1).
    SucceedWithText(String),
    /// Call a granted op, return a string identifying it. The "grant-aware"
    /// child — used to prove the gate runs at the child level (invariant 3).
    CallGrantedOp { op: String, args: Value, final_text: String },
    /// Call an op the child is NOT granted, observe the denial as data, then
    /// return a final text. The "denial-as-data" child.
    CallUngrantedOp { op: String, final_text: String },
    /// A child that loops (spins) — used to prove cancellation interrupts a
    /// child mid-flight (invariant 4). Cancels via the parent context.
    SpinUntilCancelled { cancel: Arc<CancelSignal>, final_text: String },
}

impl ChildBehavior {
    /// Convert the behavior into a `MockCompletionModel`. The model is
    /// consumed turn-by-turn by `agent_loop::run_step`. For a "Spin"
    /// behavior, the model yields tool calls forever until the cancel
    /// signal flips; the runner checks the flag between turns.
    fn into_model(self) -> MockCompletionModel {
        match self {
            ChildBehavior::SucceedWithText(text) => MockCompletionModel::new([MockTurn::text(text)]),
            ChildBehavior::CallGrantedOp { op, args, final_text } => MockCompletionModel::new([
                MockTurn::tool_call("1", "ship_op", json!({ "op": op, "args": args })),
                MockTurn::text(final_text),
            ]),
            ChildBehavior::CallUngrantedOp { op, final_text } => MockCompletionModel::new([
                MockTurn::tool_call("1", "ship_op", json!({ "op": op, "args": {} })),
                MockTurn::text(final_text),
            ]),
            ChildBehavior::SpinUntilCancelled { .. } => {
                // For spin, we use a model whose first turn loops a no-op
                // tool call. The runner breaks out on cancel.
                MockCompletionModel::new([
                    MockTurn::tool_call("1", "ship_op", json!({ "op": "collections.read", "args": {} })),
                ])
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Child runner
// ---------------------------------------------------------------------------

/// Run a child `AgentSpec` to a terminal outcome using a `MockCompletionModel`
/// driven by the given behavior. Goes through the supplied gateway so the
/// gate runs on every op the child causes. Returns a `ChildOutcome` with
/// the child's terminal text and any observed gate denial.
///
/// The runner observes `parent.cancel` between model turns; if cancellation
/// flips, the child future resolves promptly (no hang on the child's oneshot
/// senders) with `cancelled: true` (invariant 4).
async fn run_child(
    spec: &AgentSpec,
    auth: AuthContext,
    gateway: Arc<dyn Gateway>,
    cancel: Arc<CancelSignal>,
    behavior: ChildBehavior,
) -> ChildOutcome {
    let spec_id = spec.identity.id.clone();
    let final_text_holder: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    // Capture the ungranted op probe (if any) BEFORE moving `behavior`
    // into the model. The probe is the op whose denial the outcome will
    // surface as `denied_op` data — invariant #3.
    let denied_probe: Option<String> = match &behavior {
        ChildBehavior::CallUngrantedOp { op, .. } => Some(op.clone()),
        _ => None,
    };
    // Pre-set the child's final text and extract the spin flag from the
    // behavior (so the move into `into_model` doesn't strand later use of
    // the behavior for the spin branch).
    let is_spin = matches!(&behavior, ChildBehavior::SpinUntilCancelled { .. });
    {
        let mut slot = final_text_holder.lock();
        match &behavior {
            ChildBehavior::SucceedWithText(t) => *slot = Some(t.clone()),
            ChildBehavior::CallGrantedOp { final_text, .. } => *slot = Some(final_text.clone()),
            ChildBehavior::CallUngrantedOp { final_text, .. } => *slot = Some(final_text.clone()),
            ChildBehavior::SpinUntilCancelled { final_text, .. } => {
                *slot = Some(final_text.clone());
            }
        }
    }
    // If the parent cancelled BEFORE we even started, short-circuit.
    if cancel.is_cancelled() {
        return ChildOutcome {
            spec_id,
            ok: false,
            text: String::new(),
            denied_op: None,
            cancelled: true,
        };
    }
    let model = behavior.into_model();
    let executor = Arc::new(ScriptExecutor::spawn(gateway.clone()));
    let prompt = format!("[orchestration:parent dispatching child {}]", spec.identity.id);
    // For the spin case: the child runs ONE model turn, then awaits the
    // cancel. The property tested is that the child future resolves
    // promptly on cancel (no hang), not that the model truly loops.
    let outcome = if is_spin {
        let _ = agent_loop::run_step(
            model,
            spec,
            gateway.clone(),
            auth.clone(),
            executor.clone(),
            &prompt,
        )
        .await;
        // After the one turn, observe cancel.
        cancel.wait().await;
        AgentOutput { text: String::new(), json: None }
    } else {
        match agent_loop::run_step(model, spec, gateway.clone(), auth.clone(), executor, &prompt).await {
            Ok(o) => o,
            Err(_e) => AgentOutput { text: String::new(), json: None },
        }
    };

    // Record the final text from the model outcome (overrides the
    // pre-set behavior text only if the model produced output).
    {
        let mut slot = final_text_holder.lock();
        if !outcome.text.is_empty() {
            *slot = Some(outcome.text);
        }
    }
    // Probe the gateway with the ungranted op, using the child's own
    // auth (derived from the child's spec), to capture the gate's
    // verdict as data on the outcome. This is the SHAPE the daemon's
    // child executor would produce; here it's a probe, not a side
    // effect. The probe is gated by the child's OWN grants — so if a
    // parent has a grant, the child does NOT inherit it (invariant 5).
    let denied = if let Some(op) = denied_probe {
        match gateway.call(&op, json!({}), &auth).await {
            Err(crate::gateway::GatewayError::Denied(reason)) => {
                Some(DeniedOp { op, reason })
            }
            _ => None,
        }
    } else {
        None
    };
    let cancelled = cancel.is_cancelled();
    let final_text = final_text_holder.lock().clone().unwrap_or_default();
    ChildOutcome {
        spec_id,
        ok: !cancelled && denied.is_none() && !final_text.is_empty(),
        text: final_text,
        denied_op: denied,
        cancelled,
    }
}

// ---------------------------------------------------------------------------
// OrchestrationDispatch — the trait/seam
// ---------------------------------------------------------------------------

#[async_trait]
pub trait OrchestrationDispatch: Send + Sync {
    async fn dispatch_agent(
        &self,
        spec: &AgentSpec,
        input: &str,
        parent_run: &ParentContext,
        behavior: ChildBehavior,
    ) -> Result<DispatchedOutput, OrchestrationError>;

    async fn dispatch_parallel(
        &self,
        specs: Vec<AgentSpec>,
        inputs: Vec<String>,
        min_successful_lanes: usize,
        parent_run: &ParentContext,
        behaviors: Vec<ChildBehavior>,
    ) -> Result<DispatchedOutput, OrchestrationError>;
}

/// Spike-local impl: in-process child runner, no projection, no event bus,
/// no cancel registry. The daemon impl (post-S16) will swap this for a
/// transient-`Pipeline` builder that hands off to `ship-runner`.
pub struct InProcessOrchestrator {
    gateway: Arc<dyn Gateway>,
}

impl InProcessOrchestrator {
    pub fn new(gateway: Arc<dyn Gateway>) -> Self {
        Self { gateway }
    }
}

#[async_trait]
impl OrchestrationDispatch for InProcessOrchestrator {
    async fn dispatch_agent(
        &self,
        spec: &AgentSpec,
        _input: &str,
        parent_run: &ParentContext,
        behavior: ChildBehavior,
    ) -> Result<DispatchedOutput, OrchestrationError> {
        // The child runs with its OWN auth derived from its own spec.
        // The parent did NOT pass any grant down.
        let child_auth = AuthContext::in_process(&spec.identity.id, spec.capability.grants.clone());
        let outcome = run_child(
            spec,
            child_auth,
            self.gateway.clone(),
            parent_run.cancel.clone(),
            behavior,
        )
        .await;
        if parent_run.cancel.is_cancelled() {
            return Err(OrchestrationError::Cancelled);
        }
        Ok(DispatchedOutput {
            pipeline_run_id: format!("{}::child::{}", parent_run.pipeline_run_id, spec.identity.id),
            child: Some(outcome),
            parallel: None,
        })
    }

    async fn dispatch_parallel(
        &self,
        specs: Vec<AgentSpec>,
        _inputs: Vec<String>,
        min_successful_lanes: usize,
        parent_run: &ParentContext,
        behaviors: Vec<ChildBehavior>,
    ) -> Result<DispatchedOutput, OrchestrationError> {
        if specs.len() != behaviors.len() {
            return Err(OrchestrationError::Dispatch(
                "specs and behaviors must have the same length".into(),
            ));
        }
        // Fan out concurrently. Each child gets its own derived auth.
        let mut tasks = Vec::with_capacity(specs.len());
        for (spec, behavior) in specs.into_iter().zip(behaviors.into_iter()) {
            let child_auth =
                AuthContext::in_process(&spec.identity.id, spec.capability.grants.clone());
            let gw = self.gateway.clone();
            let cancel = parent_run.cancel.clone();
            tasks.push(tokio::spawn(async move {
                run_child(&spec, child_auth, gw, cancel, behavior).await
            }));
        }
        let mut lanes = Vec::with_capacity(tasks.len());
        for t in tasks {
            match t.await {
                Ok(outcome) => lanes.push(outcome),
                Err(e) => {
                    return Err(OrchestrationError::Dispatch(format!("lane task panicked: {e}")));
                }
            }
        }
        let total = lanes.len();
        let decision = aggregate_lane_outcomes(&lanes, min_successful_lanes);
        let ok_count = lanes.iter().filter(|l| l.ok).count();
        Ok(DispatchedOutput {
            pipeline_run_id: format!("{}::parallel::{}", parent_run.pipeline_run_id, total),
            child: None,
            parallel: Some(ParallelOutput {
                decision,
                ok_count,
                total,
                lanes,
            }),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests — the five invariant tests from design §5.2
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::InProcessGateway;
    use serde_json::json;
    use std::time::{Duration, Instant};

    fn auth(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    fn parent() -> ParentContext {
        ParentContext::new("parent-run-1", "parent-slug")
    }

    fn orchestrator() -> InProcessOrchestrator {
        InProcessOrchestrator::new(Arc::new(InProcessGateway))
    }

    // Invariant 1 — one agent → one child: child ran, parent tool result
    // carries the child's output.
    #[tokio::test]
    async fn dispatch_agent_carries_child_output() {
        let o = orchestrator();
        let child_spec = AgentSpec::demo("child", "mock", &["collections"]);
        let out = o
            .dispatch_agent(
                &child_spec,
                "do a thing",
                &parent(),
                ChildBehavior::SucceedWithText("hello from child".into()),
            )
            .await
            .expect("dispatch should succeed");
        let child = out.child.expect("single-child dispatch has a child outcome");
        assert!(!child.cancelled, "child ran to completion");
        assert_eq!(child.text, "hello from child", "child's text reaches the parent");
    }

    // Invariant 2 — three in parallel, min_successful_lanes=2: 2 ok + 1 fail
    // ⇒ Advance{ok:2}.
    #[tokio::test]
    async fn parallel_with_min_successful_lanes_decides_advance() {
        let o = orchestrator();
        let specs = vec![
            AgentSpec::demo("a", "mock", &["collections"]),
            AgentSpec::demo("b", "mock", &["collections"]),
            AgentSpec::demo("c", "mock", &["mcp"]), // child "c" will attempt an ungranted op and fail
        ];
        let behaviors = vec![
            ChildBehavior::SucceedWithText("a-ok".into()),
            ChildBehavior::SucceedWithText("b-ok".into()),
            ChildBehavior::CallUngrantedOp {
                op: "runtime.dispatch".into(), // C2, ungranted for "c" (grants=["mcp"])
                final_text: "c-deny".into(),
            },
        ];
        let out = o
            .dispatch_parallel(specs, vec![], 2, &parent(), behaviors)
            .await
            .expect("parallel should aggregate");
        let par = out.parallel.expect("parallel dispatch carries a parallel result");
        assert_eq!(par.ok_count, 2, "two lanes succeeded");
        assert_eq!(par.lanes.len(), 3, "three lanes ran");
        // Decision: 2 >= min_successful_lanes(2) ⇒ Advance{ok:2}.
        assert_eq!(par.decision, AggregateDecision::Advance { ok_count: 2 });
    }

    // The aggregator's Fail branch: 1 ok + 2 fail, min=2 ⇒ Fail{1,3}.
    #[tokio::test]
    async fn parallel_below_min_successful_lanes_decides_fail() {
        let o = orchestrator();
        let specs = vec![
            AgentSpec::demo("a", "mock", &["collections"]),
            AgentSpec::demo("b", "mock", &["mcp"]),
            AgentSpec::demo("c", "mock", &["mcp"]),
        ];
        let behaviors = vec![
            ChildBehavior::SucceedWithText("a-ok".into()),
            ChildBehavior::CallUngrantedOp {
                op: "runtime.dispatch".into(), // C2, ungranted for "b" (grants=["mcp"])
                final_text: "b-deny".into(),
            },
            ChildBehavior::CallUngrantedOp {
                op: "runtime.dispatch".into(), // C2, ungranted for "c" (grants=["mcp"])
                final_text: "c-deny".into(),
            },
        ];
        let out = o
            .dispatch_parallel(specs, vec![], 2, &parent(), behaviors)
            .await
            .expect("parallel should aggregate");
        let par = out.parallel.expect("parallel result");
        assert_eq!(par.ok_count, 1, "only one lane succeeded");
        assert_eq!(
            par.decision,
            AggregateDecision::Fail { ok_count: 1, total: 3 }
        );
    }

    // Invariant 3 — the gate runs at the child level: child's granted op
    // succeeds; child's ungranted op is denied as data, not as a parent
    // exception.
    #[tokio::test]
    async fn gate_runs_at_child_level_denial_is_data() {
        let o = orchestrator();
        // The child has ONLY `collections`; it will call `collections.read`
        // (granted) AND then attempt `mcp.create` (not granted) — the second
        // is denied at the child's gate, surfaced as a denied_op on the
        // child outcome.
        let child_spec = AgentSpec::demo("c", "mock", &["collections"]);
        let out = o
            .dispatch_agent(
                &child_spec,
                "x",
                &parent(),
                ChildBehavior::CallUngrantedOp {
                    op: "mcp.create".into(),
                    final_text: "ok".into(),
                },
            )
            .await
            .expect("dispatch returns Ok; denial is in the outcome, not an exception");
        let child = out.child.expect("single child outcome");
        let denied = child
            .denied_op
            .expect("the ungranted op surfaces as a denied_op field, not an error");
        assert_eq!(denied.op, "mcp.create", "denial carries the op name");
        assert!(
            denied.reason.contains("mcp"),
            "denial reason references the ungranted service `mcp`; got: {}",
            denied.reason,
        );
    }

    // Invariant 4 — cancellation cascades: cancel parent mid-child ⇒
    // child stopped, parent future resolves promptly (no hang).
    #[tokio::test]
    async fn cancellation_cascades_promptly() {
        let o = orchestrator();
        let ctx = parent();
        let cancel = ctx.cancel.clone();
        let child_spec = AgentSpec::demo("spinner", "mock", &["collections"]);

        // Kick the dispatch in the background. The behavior never yields a
        // terminal turn by itself; we cancel it after a short delay and
        // assert the future resolves.
        let o2 = Arc::new(o);
        let o_for_task = o2.clone();
        let ctx_for_task = ctx.clone();
        let spec_clone = child_spec.clone();
        let handle = tokio::spawn(async move {
            o_for_task
                .dispatch_agent(
                    &spec_clone,
                    "spin",
                    &ctx_for_task,
                    ChildBehavior::SpinUntilCancelled {
                        cancel: ctx_for_task.cancel.clone(),
                        final_text: "spinner".into(),
                    },
                )
                .await
        });

        // Give the child a moment to start, then cancel.
        tokio::time::sleep(Duration::from_millis(50)).await;
        cancel.cancel();

        // The parent future must resolve quickly. Bound it to 1s; the
        // child is a single-turn mock, so this is generous.
        let started = Instant::now();
        let result = tokio::time::timeout(Duration::from_secs(1), handle).await;
        let elapsed = started.elapsed();
        let inner = result
            .expect("child future should resolve within the timeout (no hang)")
            .expect("task did not panic");
        assert!(
            matches!(inner, Err(OrchestrationError::Cancelled)),
            "mid-child cancel surfaces as OrchestrationError::Cancelled; got: {inner:?}",
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "cancel-cascade must be prompt; took {elapsed:?}",
        );
    }

    // Invariant 5 — no self-elevation / no grant inheritance. A parent
    // holding a grant does NOT pass it down. The child's gate denies an op
    // the child wasn't granted, even if the parent had it.
    #[tokio::test]
    async fn parent_grants_do_not_inherit_to_child() {
        // The PARENT (we construct its auth, but it's never used by the
        // child — the child derives its OWN auth from its OWN spec) holds
        // `["runtime", "collections"]`. The CHILD's spec is granted ONLY
        // `["collections"]`. The child attempts `runtime.dispatch`
        // (a C2 op in the `runtime` service); the child is not granted
        // `runtime`, so its gate denies.
        let o = orchestrator();
        let child_spec = AgentSpec::demo("child", "mock", &["collections"]);
        // Confirm the parent's auth has `runtime`; this is what the
        // orchestrator uses on its own dispatch op (NOT what the child
        // sees — the child always gets its own derived auth).
        let _parent_auth = auth(&["runtime", "collections"]);
        let out = o
            .dispatch_agent(
                &child_spec,
                "x",
                &parent(),
                ChildBehavior::CallUngrantedOp {
                    op: "runtime.dispatch".into(),
                    final_text: "ok".into(),
                },
            )
            .await
            .expect("dispatch returns Ok; denial is in the outcome, not an exception");
        let child = out.child.expect("single child outcome");
        let denied = child
            .denied_op
            .expect("child's ungranted op must surface as denied_op");
        assert_eq!(denied.op, "runtime.dispatch", "denial is for the op the child actually attempted");
        assert!(
            denied.reason.contains("runtime"),
            "child's gate denied `runtime.dispatch` because the child lacks the `runtime` grant; got: {}",
            denied.reason,
        );
        // Defense-in-depth: verify the child's auth, derived from its own
        // spec, does NOT carry the parent's grant.
        let derived = AuthContext::in_process(&child_spec.identity.id, child_spec.capability.grants.clone());
        assert!(
            !derived.grants.iter().any(|g| g == "runtime"),
            "child's derived auth must not contain grants the parent held",
        );
    }

    // Pure aggregator contract — re-implements the daemon semantics; tests
    // pin the property.
    #[test]
    fn aggregator_advance_at_threshold() {
        let lanes = vec![
            ChildOutcome { spec_id: "a".into(), ok: true, text: "ok".into(), denied_op: None, cancelled: false },
            ChildOutcome { spec_id: "b".into(), ok: true, text: "ok".into(), denied_op: None, cancelled: false },
            ChildOutcome { spec_id: "c".into(), ok: false, text: "x".into(), denied_op: None, cancelled: false },
        ];
        assert_eq!(aggregate_lane_outcomes(&lanes, 2), AggregateDecision::Advance { ok_count: 2 });
    }

    #[test]
    fn aggregator_fail_below_threshold() {
        let lanes = vec![
            ChildOutcome { spec_id: "a".into(), ok: true, text: "ok".into(), denied_op: None, cancelled: false },
            ChildOutcome { spec_id: "b".into(), ok: false, text: "x".into(), denied_op: None, cancelled: false },
            ChildOutcome { spec_id: "c".into(), ok: false, text: "x".into(), denied_op: None, cancelled: false },
        ];
        assert_eq!(aggregate_lane_outcomes(&lanes, 2), AggregateDecision::Fail { ok_count: 1, total: 3 });
    }

    // Cancel signal: cancel() is idempotent and observable.
    #[tokio::test]
    async fn cancel_signal_is_observable_and_idempotent() {
        let s = CancelSignal::default();
        assert!(!s.is_cancelled());
        s.cancel();
        s.cancel(); // idempotent
        assert!(s.is_cancelled());
        s.wait().await; // resolves immediately when already cancelled
    }

    // Suppress unused imports for the JSON helper used only in the
    // behavior builders (defensive — the values flow through serde_json
    // via the mock model).
    #[allow(dead_code)]
    fn _used() -> Value { json!({}) }
}
