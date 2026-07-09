//! `AgentStepDispatch` — the live `Dispatch` for [`AgentStepProgram`]'s ops —
//! plus the kill-on-timeout helper used by the monitor loop.
//!
//! The former `emit_and_finish` (build payload → emit `step.observed` →
//! terminalize the composed session) dissolved into the `agent.emit_observed`
//! op here (C5 / P2.3): the emission now happens exactly once under journal
//! replay, because a completed op is never re-dispatched.
//!
//! The git/log/string scanning utilities that used to live here are L0
//! pipeline-runtime mechanism and live in
//! [`crate::loader::git_helpers`]; the kill-on-timeout teardown is the L0
//! [`ship_code::runner::stop_by_slug`] primitive.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use durable_execution::{Dispatch, DispatchError};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{
    PipelineStepObservedPayload, StepVerdict, event_types,
};
use ship_substrate::events::{EventEnvelope, KernelRouter};
use ship_code::dispatch_ports::{SessionExecutor, ProcessRegistryPort};

use crate::loader::git_helpers::scan_agent_log_tail;

use crate::orchestrator::coordinator_programs::StepEvidence;

/// Stop the dispatched process for `slug` on a step timeout, via the L0
/// teardown primitive. The `agent.emit_observed` op calls
/// `registry.unregister` after us, and `stop_by_slug` `take`s the handle, so
/// there's no double-remove.
pub(super) async fn kill_spawned_process(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<dyn ProcessRegistryPort>,
    executor: &Arc<dyn SessionExecutor>,
    slug: &str,
) {
    ship_code::runner::stop_by_slug(kernel, registry, executor, slug, "step_timeout", "step_monitor").await;
}

/// Live dispatch for [`AgentStepProgram`]: holds the step identity + the ports
/// the observed-emission needs. Ops run on the live frontier only — replay
/// returns logged results without calling back in here.
pub(super) struct AgentStepDispatch {
    pub kernel: Arc<Mutex<KernelRouter>>,
    pub registry: Arc<dyn ProcessRegistryPort>,
    pub pipeline_run_id: String,
    pub slug: String,
    pub step_index: usize,
    pub step_name: String,
    pub work_dir: PathBuf,
    pub started_at: Instant,
    pub start_sha: Option<String>,
}

#[async_trait]
impl Dispatch for AgentStepDispatch {
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
        match op {
            // C5 bridge: the imperative dispatch path already spawned the
            // agent before the monitor starts — this op only marks the spawn
            // in the journal. It becomes the real spawn call at C7 (P2.5).
            "agent.spawn" => Ok(json!({ "bridged": true })),
            "agent.emit_observed" => {
                let evidence: StepEvidence = serde_json::from_value(args["evidence"].clone())
                    .map_err(|e| DispatchError::Terminal(format!("agent.emit_observed: bad evidence: {e}")))?;
                let verdict: StepVerdict = serde_json::from_value(args["verdict"].clone())
                    .map_err(|e| DispatchError::Terminal(format!("agent.emit_observed: bad verdict: {e}")))?;
                self.emit_observed(evidence, verdict).await;
                Ok(json!({}))
            }
            other => Err(DispatchError::Terminal(format!(
                "AgentStepDispatch: unknown op `{other}`"
            ))),
        }
    }
}

impl AgentStepDispatch {
    /// Build the payload, emit `step.observed`, and unregister from the
    /// registry — the body of the former `emit_and_finish`, with the verdict
    /// computed upstream by the program's `agent.verdict` record.
    async fn emit_observed(&self, evidence: StepEvidence, verdict: StepVerdict) {
        self.registry.unregister(&self.slug).await;

        let StepEvidence {
            trigger, exit_code, done_declared, reason, commits, uncommitted, turns,
        } = evidence;
        let done_result = if done_declared { Some(true) } else { None };
        let duration_secs = self.started_at.elapsed().as_secs();
        let succeeded = verdict == StepVerdict::Advance;
        // Captured before `verdict` moves into the payload below.
        let step_is_failure = verdict == StepVerdict::Failed;

        tracing::info!(
            slug = self.slug, step = self.step_name,
            ?trigger, ?verdict, done_declared, ?exit_code,
            commits, uncommitted, turns, ?reason,
            "step-monitor: emitting observation"
        );

        // Capture the tail of the agent's stdout/stderr log when the step
        // didn't Advance, so operators see error messages without shelling
        // into the worktree. Successful steps skip this to keep the event
        // stream lean.
        let log_tail = if succeeded {
            None
        } else {
            scan_agent_log_tail(&self.work_dir, &self.slug)
        };

        let payload = PipelineStepObservedPayload {
            pipeline_run_id: self.pipeline_run_id.clone(),
            slug: self.slug.clone(),
            step_index: self.step_index,
            step_name: self.step_name.clone(),
            trigger,
            done_declared,
            done_result,
            exit_code,
            commits_since_start: commits,
            uncommitted_files: uncommitted,
            duration_secs,
            reason,
            verdict,
            succeeded,
            start_sha: self.start_sha.clone(),
            log_tail,
        };

        ship_runner::emit::emit_process_exited(
            &self.kernel, &self.slug, None, exit_code, None,
            duration_secs * 1000, "unknown",
        ).await;

        let envelope = match EventEnvelope::new(
            event_types::PIPELINE_STEP_OBSERVED, &self.pipeline_run_id, &payload,
        ) {
            Ok(e) => e.with_actor_id("service.orchestrator"),
            Err(e) => {
                tracing::warn!(slug = self.slug, "step-monitor: failed to build envelope: {e}");
                return;
            }
        };
        ship_runner::emit::persist_and_route(&self.kernel, envelope, &self.slug).await;

        // Terminalize the composed agent_session so it doesn't sit at
        // Dispatched forever. Failed verdict → session failed; Advance /
        // Inconclusive (both let the pipeline proceed) → completed.
        crate::orchestrator::dispatch::emit_composed_session_terminal(
            &self.kernel,
            &self.slug,
            &self.step_name,
            step_is_failure,
            exit_code,
        )
        .await;
    }
}
