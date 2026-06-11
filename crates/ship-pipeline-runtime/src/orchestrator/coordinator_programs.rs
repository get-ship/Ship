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
