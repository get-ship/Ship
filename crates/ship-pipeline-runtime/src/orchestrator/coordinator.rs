//! `PipelineRunCoordinator` (C7 / P2.5) — the journaled DAG walk for one
//! pipeline run.
//!
//! The program IS the authoritative advancement state machine: the step DAG
//! is journaled once at the start (`pipeline.steps` record — replay walks the
//! identical DAG even if the projection later changes shape), each round's
//! unblocked set is a journaled record, concurrent siblings run as ONE
//! `parallel` node (the C6 fanout pattern), and a `pipeline.finalize` op
//! closes the run exactly once under replay.
//!
//! Step ops (`step.<id>`) are where the dispatch impl plugs in. The LIVE
//! dispatch (C9 cutover) consults the step's own journal from C4-C6 as its
//! idempotence oracle — complete → return the journaled verdict; suspended →
//! re-attach a waiter, never re-spawn; absent → dispatch and await — per
//! `.artifacts/c7-coordinator-spec.md`. Until the cutover, the event-driven
//! `advance_pipeline` path remains the only live advancement mechanism; this
//! program is exercised against seeded dispatches by `tests_coordinator.rs`.
//!
//! Failure policy mirrors the router's verdict arms: `Advance` and
//! `Inconclusive` both advance (succeeded = true at the walk level — the gate
//! step catches Inconclusive problems); `Failed` halts the walk after the
//! current sibling node completes, and `pipeline.finalize` carries the
//! failed step id.
//!
//! Op sequence is DETERMINISTIC given the journaled DAG and the journaled
//! per-round unblocked sets — must NOT change without migrating journals.

use durable_execution::{EngineError, ExecCtx, Program};
use serde_json::{Value, json};
use ship_pipelines::types::Pipeline;
use async_trait::async_trait;

/// The journal key for a run's coordinator. Its presence is the bridge
/// signal: a run with a coordinator journal is coordinator-owned and the
/// event-driven `advance_pipeline` path must not advance it (C9 cutover).
pub(crate) fn coordinator_journal_key(pipeline_run_id: &str) -> String {
    format!("{pipeline_run_id}/coordinator")
}

pub(crate) struct PipelineRunCoordinator {
    /// Initial DAG snapshot — used only on the first (live-frontier) run;
    /// every replay reads the DAG back from the `pipeline.steps` record.
    pub pipeline: Pipeline,
}

#[async_trait]
impl Program for PipelineRunCoordinator {
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
        // Journal the DAG once — the walk's source of truth across replays.
        let dag_val = {
            let p = self.pipeline.clone();
            ctx.record("pipeline.steps", move || async move {
                serde_json::to_value(&p).unwrap_or(Value::Null)
            })
            .await?
        };
        let dag: Pipeline = serde_json::from_value(dag_val).map_err(|e| EngineError::Dispatch {
            op: "pipeline.steps".into(),
            reason: format!("journaled DAG deserialize: {e}"),
        })?;

        let mut completed: Vec<String> = Vec::new();
        let mut failed_step: Option<String> = None;
        let mut round = 0usize;
        loop {
            // Journal each round's unblocked set so replay walks the same
            // order even if Pipeline::unblocked_steps changes its tiebreaks.
            let unblocked_val = {
                let dag = dag.clone();
                let completed = completed.clone();
                let halted = failed_step.is_some();
                ctx.record(&format!("unblocked.{round}"), move || async move {
                    if halted {
                        return json!([]);
                    }
                    let ids: Vec<String> = dag
                        .unblocked_steps(&completed)
                        .iter()
                        .map(|s| s.id.clone())
                        .collect();
                    json!(ids)
                })
                .await?
            };
            let unblocked: Vec<String> =
                serde_json::from_value(unblocked_val).unwrap_or_default();
            if unblocked.is_empty() {
                break;
            }

            // Concurrent siblings = one parallel node, outcomes in DAG order.
            let calls = unblocked
                .iter()
                .map(|id| (format!("step.{id}"), json!({})))
                .collect::<Vec<_>>();
            let outcomes = ctx.parallel(calls).await?;

            for (id, outcome) in unblocked.iter().zip(outcomes.iter()) {
                completed.push(id.clone());
                let succeeded = outcome["succeeded"].as_bool().unwrap_or(false);
                if !succeeded && failed_step.is_none() {
                    failed_step = Some(id.clone());
                }
            }
            round += 1;
        }

        let summary = json!({ "completed": completed, "failed_step": failed_step });
        ctx.op("pipeline.finalize", summary.clone()).await?;
        Ok(summary)
    }
}
