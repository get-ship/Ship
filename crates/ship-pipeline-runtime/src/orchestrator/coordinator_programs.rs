//! Durable-execution programs and dispatch impls for pipeline-run coordination.
//!
//! **C2 additions:**
//! - [`DispatchProgram`]: the three-op agent-spawn program (journaled; deterministic).
//! - [`ShadowDispatch`]: post-hoc oracle dispatch — returns pre-seeded results without
//!   calling any real ports. Used in C2 to construct the journal after the primary
//!   dispatch path has already executed, with no double-execution risk.
//! - [`PipelineCoordinatorDispatch`]: skeleton for the live dispatch (wired in C7/P2.5).
//!   All ops return `Err(Terminal)` in C1-C6; it is never called from the live path
//!   during that interval.

use durable_execution::{EngineError, ExecCtx, Program};
use serde_json::{json, Value};
use ship_substrate::events::pipeline::{
    PipelineStepObservedPayload, StepTrigger, StepType, StepVerdict,
};
use std::collections::HashMap;
use async_trait::async_trait;
use durable_execution::{Dispatch, DispatchError};

/// Journaled first-step agent-spawn program: the three ops that constitute
/// `dispatch_pipeline_run_internal`'s agent-spawn path.
///
/// Op sequence is DETERMINISTIC — the contract enforced by the engine's
/// replay-divergence guard. Must NOT be changed without migrating existing journals.
///
/// Does NOT cover the run-step or fanout early-exit branches of
/// `dispatch_pipeline_run_internal`.
pub(crate) struct DispatchProgram;

#[async_trait]
impl Program for DispatchProgram {
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
        let worktree = ctx.op("pipeline.prepare_isolation", json!({})).await?;
        let spawn    = ctx.op("pipeline.spawn_agent", json!({ "worktree": worktree })).await?;
        let _emit    = ctx.op("pipeline.emit_dispatched", json!({ "worktree": worktree, "spawn": spawn })).await?;
        Ok(spawn)
    }
}

/// Journaled shell-step program (C4 / P2.2): the three ops that constitute a
/// `type: run` pipeline step.
///
/// Unlike [`DispatchProgram`] (shadow-journaled in C2), this program runs LIVE —
/// `RunStepDispatch` (orchestrator/run_step.rs) executes the real `StepExecutor`
/// and emits the real `pipeline.step.*` events from inside the ops. On replay
/// (daemon restart mid-step, or a re-dispatched step that already completed),
/// completed ops return their logged results: the command is NOT re-executed
/// and the step events are NOT re-emitted.
///
/// Op sequence is DETERMINISTIC — the contract enforced by the engine's
/// replay-divergence guard. Must NOT be changed without migrating existing journals.
///
/// Executor failures (spawn error / timeout / memory cap) are ENCODED in the
/// `run_step.execute` result value rather than surfaced as `DispatchError` — the
/// terminal emission op must still run (and be journaled) on the failure path.
pub(crate) struct RunStepProgram;

#[async_trait]
impl Program for RunStepProgram {
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
        let _started = ctx.op("run_step.emit_started", json!({})).await?;
        let result = ctx.op("run_step.execute", json!({})).await?;
        let _term = ctx.op("run_step.emit_terminal", json!({ "result": result })).await?;
        Ok(result)
    }
}

/// The evidence the step monitor (C5 bridge) / coordinator (C7+) resolves the
/// `agent.running` suspend with. Serialized as the suspend decision, so the
/// journal carries the durable record of WHY the step finished.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct StepEvidence {
    pub trigger: StepTrigger,
    pub exit_code: Option<i32>,
    pub done_declared: bool,
    pub reason: Option<String>,
    pub commits: u32,
    pub uncommitted: u32,
    pub turns: u32,
}

/// Journaled agent-step program (C5 / P2.3): spawn → park while the agent
/// runs → verdict → observed emission.
///
/// The suspend is the architectural heart: "the agent is running" is a parked
/// journal state, not an in-memory select-loop. In the C5 bridge the
/// `StepMonitor` still owns the timing machinery (idle/hard-cap/settle) and
/// acts as the EVIDENCE FEEDER — it resolves the suspend with a
/// [`StepEvidence`] instead of emitting `pipeline.step.observed` itself. At
/// C7/P2.5 the coordinator takes over the timers and the monitor dissolves.
///
/// `agent.spawn` is an echo op in the bridge (the imperative dispatch path
/// already spawned the agent); it becomes the real spawn at C7.
///
/// The verdict is a `record` (pure computation on the journaled evidence —
/// `compute_verdict` is the existing ladder, unchanged); the emission is an
/// `op` (effectful, exactly-once under replay).
///
/// Op sequence is DETERMINISTIC — must NOT change without migrating journals.
pub(crate) struct AgentStepProgram {
    pub step_type: StepType,
}

#[async_trait]
impl Program for AgentStepProgram {
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
        let _spawn = ctx.op("agent.spawn", json!({})).await?;
        let evidence = ctx.suspend("agent.running").await?;
        let step_type = self.step_type.clone();
        let ev = evidence.clone();
        let verdict = ctx
            .record("agent.verdict", move || async move {
                match serde_json::from_value::<StepEvidence>(ev) {
                    Ok(e) => {
                        let done_result = if e.done_declared { Some(true) } else { None };
                        let v = PipelineStepObservedPayload::compute_verdict(
                            &step_type,
                            e.done_declared,
                            done_result,
                            e.exit_code,
                            e.commits,
                            e.uncommitted,
                            &e.trigger,
                        );
                        serde_json::to_value(v).unwrap_or(Value::Null)
                    }
                    Err(err) => {
                        // Malformed evidence is a programming bug (we serialize
                        // it ourselves) — degrade to Inconclusive so the
                        // pipeline surfaces it to a human instead of stalling.
                        tracing::error!("agent-step: malformed evidence, verdict inconclusive: {err}");
                        serde_json::to_value(StepVerdict::Inconclusive).unwrap_or(Value::Null)
                    }
                }
            })
            .await?;
        ctx.op(
            "agent.emit_observed",
            json!({ "evidence": evidence, "verdict": verdict }),
        )
        .await?;
        Ok(verdict)
    }
}

/// Journaled fanout program (C6 / P2.4): K lanes as ONE `parallel` node →
/// aggregate decision as a `record` → parent-outcome emission as an `op`.
///
/// What the journal buys at this stage: lane outcomes land deterministically
/// (input order), the Advance/Fail decision is a durable journaled value, and
/// the parent `step.observed`/`step.failed` emission is exactly-once under
/// replay — a fanout whose journal is complete can be re-driven without
/// double-emitting. Lane RESOLUTION stays in-process (the dispatch awaits the
/// per-lane oneshot the router resolves); per-lane journaling and the
/// channels' deletion arrive with the coordinator at C7-C9.
///
/// Each lane op returns `{"reported": bool, "outcome": LaneOutcome|null}` —
/// `reported: false` means the lane's sender was dropped without a value
/// (cancellation teardown), mirroring the old aggregator's "skip" behavior.
///
/// Op sequence is DETERMINISTIC per construction (lane_ops in spawn order) —
/// must NOT change without migrating journals.
pub(crate) struct FanoutProgram {
    /// One op label per SPAWNED lane (`fanout.lane.<lane_id>`), spawn order.
    pub lane_ops: Vec<String>,
    pub min_ok: usize,
    /// Total lanes in the SPEC (not just spawned) — failed spawns count
    /// against `min_ok` exactly as the old aggregator counted them.
    pub total: usize,
}

#[async_trait]
impl Program for FanoutProgram {
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
        let calls = self
            .lane_ops
            .iter()
            .map(|op| (op.clone(), json!({})))
            .collect::<Vec<_>>();
        let results = ctx.parallel(calls).await?;

        let (min_ok, total) = (self.min_ok, self.total);
        let res = results.clone();
        let decision = ctx
            .record("fanout.aggregate", move || async move {
                let outcomes: Vec<ship_pipelines::fanout::LaneOutcome> = res
                    .iter()
                    .filter(|v| v["reported"].as_bool().unwrap_or(false))
                    .filter_map(|v| serde_json::from_value(v["outcome"].clone()).ok())
                    .collect();
                let reported = outcomes.len();
                let decision = ship_pipelines::fanout::aggregate_lane_outcomes(
                    &outcomes, min_ok, total,
                );
                json!({
                    "reported": reported,
                    "decision": serde_json::to_value(decision).unwrap_or(Value::Null),
                })
            })
            .await?;
        ctx.op("fanout.emit_outcome", json!({ "aggregate": decision })).await?;
        Ok(decision)
    }
}

/// C2 SHADOW ONLY — oracle dispatch that returns pre-seeded results without
/// calling any real ports.
///
/// Used in C2 to construct the journal post-hoc after the primary dispatch path
/// has already executed, without re-running `isolation.prepare()` or
/// `executor.spawn()`. There is NO double-execution risk because `ShadowDispatch`
/// never calls any real port — it only echoes pre-seeded values.
///
/// This becomes dead code at C7 (P2.5), when `PipelineCoordinatorDispatch`
/// becomes the live primary.
pub(crate) struct ShadowDispatch {
    results: HashMap<&'static str, Value>,
}

impl ShadowDispatch {
    pub fn for_agent_spawn(worktree_path: &str, pid: Option<u32>) -> Self {
        let mut results = HashMap::new();
        results.insert("pipeline.prepare_isolation", json!(worktree_path));
        results.insert("pipeline.spawn_agent", json!({ "pid": pid }));
        results.insert("pipeline.emit_dispatched", json!({}));
        Self { results }
    }
}

#[async_trait]
impl Dispatch for ShadowDispatch {
    async fn call(&self, op: &str, _args: Value) -> Result<Value, DispatchError> {
        self.results
            .get(op)
            .cloned()
            .ok_or_else(|| DispatchError::Terminal(format!("ShadowDispatch: unknown op `{op}`")))
    }
}

/// C1 SKELETON: PipelineCoordinatorDispatch — all ops `Err(Terminal)` until C7 (P2.5).
///
/// Becomes the real live dispatch at C7, at which point it replaces the imperative
/// agent-spawn path in `dispatch_pipeline_run_internal`. In C1-C6, it is never
/// called from the live path — it exists only as a compile-time placeholder for the
/// `Dispatch` impl the engine will use at C7.
pub(crate) struct PipelineCoordinatorDispatch {
    // Fields added incrementally as ops are wired (C7+).
    // In C1-C6 this struct is entirely empty / zero-size.
}

#[async_trait]
impl Dispatch for PipelineCoordinatorDispatch {
    async fn call(&self, op: &str, _args: Value) -> Result<Value, DispatchError> {
        // C1-C6: skeleton — all ops intentionally unimplemented.
        // C7 (P2.5): this becomes the primary dispatch with real port calls and
        //     P0.0 compat event side-effects on the live frontier only.
        Err(DispatchError::Terminal(format!("PipelineCoordinatorDispatch: op `{op}` not yet wired (C7)")))
    }
}
