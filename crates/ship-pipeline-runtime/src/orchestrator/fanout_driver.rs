//! Fanout journal driver — the live `Dispatch` for [`FanoutProgram`] and the
//! task that executes it (C6 / P2.4).
//!
//! Replaces the former hand-rolled background aggregator in `fanout.rs`:
//! lane outcomes resolve through the same per-lane oneshot channels the
//! router fills (`complete_fanout_lane`), but the wait → aggregate → emit
//! sequence now runs as a journaled program. The aggregate decision is a
//! durable journal value and the parent `step.observed`/`step.failed`
//! emission is exactly-once under replay.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use durable_execution::{Dispatch, DispatchError, Journal, JournalStore, RunStatus};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tokio::sync::oneshot;

use ship_substrate::events::pipeline::{
    PipelineStepObservedPayload, StepTrigger, StepVerdict, event_types,
};
use ship_substrate::events::{EventEnvelope, KernelRouter};
use ship_substrate::events::emit::{emit_step_failed, persist_and_route};

use super::LaneOutcome;
use super::coordinator_programs::FanoutProgram;

/// The journal key for one fanout step of one run — its own namespace,
/// disjoint from the dispatch (`<run_id>`), run-step (`<run_id>/step/*`) and
/// agent-step (`<run_id>/agent-step/*`) journals.
fn fanout_journal_key(pipeline_run_id: &str, step_id: &str) -> String {
    format!("{pipeline_run_id}/fanout/{step_id}")
}

/// Live dispatch for [`FanoutProgram`]. Lane ops await the oneshot the
/// router resolves when the lane's `step.observed` arrives; the emit op
/// performs the parent-outcome emission the old aggregator did inline.
pub(super) struct FanoutDispatch {
    pub kernel: Arc<Mutex<KernelRouter>>,
    pub pipeline_run_id: String,
    pub parent_slug: String,
    pub step_id: String,
    pub step_idx: usize,
    pub total: usize,
    pub started_at: Instant,
    /// op label (`fanout.lane.<id>`) → the lane's outcome receiver. Each is
    /// taken exactly once (the engine calls each lane op exactly once on the
    /// live frontier; replays never reach the dispatch).
    pub lane_rxs: Mutex<HashMap<String, oneshot::Receiver<LaneOutcome>>>,
}

#[async_trait]
impl Dispatch for FanoutDispatch {
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
        if op.starts_with("fanout.lane.") {
            let rx = self.lane_rxs.lock().await.remove(op).ok_or_else(|| {
                DispatchError::Terminal(format!("FanoutDispatch: no receiver for `{op}`"))
            })?;
            return Ok(match rx.await {
                Ok(outcome) => json!({
                    "reported": true,
                    "outcome": serde_json::to_value(&outcome)
                        .map_err(|e| DispatchError::Terminal(format!("lane outcome serialize: {e}")))?,
                }),
                // Sender dropped without a value (cancellation teardown or a
                // dispatch that died after registering) — mirrors the old
                // aggregator's silent skip.
                Err(_) => json!({ "reported": false, "outcome": Value::Null }),
            });
        }
        match op {
            "fanout.emit_outcome" => {
                self.emit_outcome(&args["aggregate"]).await;
                Ok(json!({}))
            }
            other => Err(DispatchError::Terminal(format!(
                "FanoutDispatch: unknown op `{other}`"
            ))),
        }
    }
}

impl FanoutDispatch {
    /// The parent-outcome emission the old background aggregator performed:
    /// quiet exit when nothing reported and the parent is already terminal,
    /// `step.failed` on insufficient lanes, synthetic `step.observed(Advance)`
    /// otherwise.
    async fn emit_outcome(&self, aggregate: &Value) {
        let reported = aggregate["reported"].as_u64().unwrap_or(0);
        let decision = &aggregate["decision"];

        // If every lane's sender was dropped without a value AND the parent
        // job is already terminal (typically Cancelled), exit quietly. The
        // terminal-state guard would drop any step.failed we emitted anyway,
        // and emitting one would pollute the event log with a misleading
        // "insufficient successful lanes" reason for an operator cancel.
        if reported == 0 && parent_job_is_terminal(&self.pipeline_run_id) {
            tracing::info!(
                pipeline_run_id = self.pipeline_run_id, step_id = self.step_id,
                "fanout: outcome suppressed — parent job is terminal (cancelled or failed)"
            );
            return;
        }

        if let Some(fail) = decision.get("Fail") {
            let ok_count = fail["ok_count"].as_u64().unwrap_or(0);
            let reason = fail["reason"].as_str().unwrap_or("insufficient successful lanes");
            tracing::warn!(
                pipeline_run_id = self.pipeline_run_id, step_id = self.step_id,
                ok_count, total = self.total,
                "fanout: insufficient successful lanes — failing fanout step"
            );
            emit_step_failed(
                &self.kernel, &self.pipeline_run_id, &self.parent_slug,
                &self.step_id, reason,
            )
            .await;
            return;
        }

        let ok_count = decision
            .get("Advance")
            .and_then(|a| a["ok_count"].as_u64())
            .unwrap_or(0);
        let payload = PipelineStepObservedPayload {
            pipeline_run_id: self.pipeline_run_id.clone(),
            slug: self.parent_slug.clone(),
            step_index: self.step_idx,
            step_name: self.step_id.clone(),
            trigger: StepTrigger::DoneDeclared,
            done_declared: true,
            done_result: Some(true),
            exit_code: None,
            commits_since_start: 0,
            uncommitted_files: 0,
            duration_secs: self.started_at.elapsed().as_secs(),
            reason: Some(format!("fanout: {ok_count}/{} lanes succeeded", self.total)),
            verdict: StepVerdict::Advance,
            succeeded: true,
            start_sha: None,
            log_tail: None,
        };
        if let Ok(env) =
            EventEnvelope::new(event_types::PIPELINE_STEP_OBSERVED, &self.pipeline_run_id, &payload)
        {
            let env = env.with_actor_id("service.orchestrator");
            persist_and_route(&self.kernel, env, &self.parent_slug).await;
        }
    }
}

/// Spawn the journal-driving task for one fanout step: load-or-new the
/// fanout journal, execute [`FanoutProgram`], save the advanced journal.
#[allow(clippy::too_many_arguments)]
pub(super) fn start_fanout_driver(
    journal_store: Arc<dyn JournalStore>,
    dispatch: Arc<FanoutDispatch>,
    lane_ops: Vec<String>,
    min_ok: usize,
) {
    let program = FanoutProgram { lane_ops, min_ok, total: dispatch.total };
    let key = fanout_journal_key(&dispatch.pipeline_run_id, &dispatch.step_id);
    let slug = dispatch.parent_slug.clone();
    tokio::spawn(async move {
        let journal = match journal_store.load(&key).await {
            Ok(Some(j)) => j,
            Ok(None) => Journal::new(),
            Err(e) => {
                tracing::warn!(slug, "fanout: journal load failed, starting fresh: {e}");
                Journal::new()
            }
        };
        let (status, journal) =
            durable_execution::execute(journal, dispatch as Arc<dyn Dispatch>, &program).await;
        if let Err(e) = journal_store.save(&key, &journal).await {
            tracing::warn!(slug, "fanout: journal save failed (non-fatal): {e}");
        }
        if let RunStatus::Failed(e) = status {
            tracing::error!(slug, "fanout: engine failure driving fanout step: {e}");
        }
    });
}

/// Peek at the projection to see whether `pipeline_run_id` is in a terminal
/// state — used to suppress misleading `step.failed` emissions after operator
/// cancellation. Returns `false` when the projection can't be loaded so we
/// never swallow a genuine fanout failure.
fn parent_job_is_terminal(pipeline_run_id: &str) -> bool {
    use crate::projections::pipeline_run::{PipelineRunStatus, load_pipeline_run_by_id};
    match load_pipeline_run_by_id(pipeline_run_id) {
        Ok(Some(job)) => matches!(
            job.status,
            PipelineRunStatus::Cancelled | PipelineRunStatus::Failed | PipelineRunStatus::Merged
        ),
        _ => false,
    }
}
