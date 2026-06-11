//! Behavioral tests for `PipelineRunCoordinator` (C7 / P2.5).
//!
//! The program contract through seeded dispatches: the DAG walk is journaled
//! and deterministic, siblings run as one parallel node, a failed step halts
//! the walk, and finalize fires exactly once across replays — nothing
//! re-dispatches when a completed journal is re-driven.

use async_trait::async_trait;
use durable_execution::{Dispatch, DispatchError, Journal, RunStatus, execute};
use serde_json::{Value, json};
use ship_pipelines::types::{Pipeline, PipelineStep, StepType};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex as StdMutex;

use super::coordinator::PipelineRunCoordinator;

fn step(id: &str, after: &[&str]) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: StepType::Agent,
        agent: Some("dev".into()),
        goal: None,
        after: after.iter().map(|s| s.to_string()).collect(),
        model: None,
        harness: None,
        command: None,
        max_retries: None,
        gate: None,
        produces: vec![],
        consumes: vec![],
        promotes: vec![],
        fanout: None,
        invocation: None,
    }
}

fn pipeline(steps: Vec<PipelineStep>) -> Pipeline {
    Pipeline { steps, auto_merge: false }
}

/// Seeded step outcomes; records dispatch order and counts finalize calls.
struct SeededSteps {
    outcomes: BTreeMap<String, bool>,
    dispatched: StdMutex<Vec<String>>,
    finalizes: Arc<AtomicUsize>,
}

#[async_trait]
impl Dispatch for SeededSteps {
    async fn call(&self, op: &str, _args: Value) -> Result<Value, DispatchError> {
        if let Some(id) = op.strip_prefix("step.") {
            self.dispatched.lock().unwrap().push(id.to_string());
            let succeeded = *self.outcomes.get(id).unwrap_or(&true);
            return Ok(json!({ "succeeded": succeeded }));
        }
        match op {
            "pipeline.finalize" => {
                self.finalizes.fetch_add(1, Ordering::SeqCst);
                Ok(json!({}))
            }
            other => Err(DispatchError::Terminal(format!("unknown op `{other}`"))),
        }
    }
}

fn seeded(outcomes: &[(&str, bool)]) -> (Arc<SeededSteps>, Arc<AtomicUsize>) {
    let finalizes = Arc::new(AtomicUsize::new(0));
    let d = Arc::new(SeededSteps {
        outcomes: outcomes.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        dispatched: StdMutex::new(vec![]),
        finalizes: finalizes.clone(),
    });
    (d, finalizes)
}

/// Linear 2-step DAG: both steps walk in order, finalize once; a replay
/// re-drive dispatches NOTHING and returns the identical summary.
#[tokio::test]
async fn linear_walk_completes_and_replays_without_redispatch() {
    let (dispatch, finalizes) = seeded(&[("build", true), ("verify", true)]);
    let program = PipelineRunCoordinator {
        pipeline: pipeline(vec![step("build", &[]), step("verify", &["build"])]),
    };

    let (status, journal) =
        execute(Journal::new(), dispatch.clone() as Arc<dyn Dispatch>, &program).await;
    let RunStatus::Completed(summary) = status else {
        panic!("coordinator must complete, got {status:?}");
    };
    assert_eq!(summary["completed"], json!(["build", "verify"]));
    assert_eq!(summary["failed_step"], Value::Null);
    assert_eq!(
        *dispatch.dispatched.lock().unwrap(),
        vec!["build".to_string(), "verify".to_string()],
        "linear DAG dispatches in dependency order"
    );
    assert_eq!(finalizes.load(Ordering::SeqCst), 1);

    let (status2, _) = execute(journal, dispatch.clone() as Arc<dyn Dispatch>, &program).await;
    let RunStatus::Completed(summary2) = status2 else {
        panic!("replay must complete");
    };
    assert_eq!(summary2, summary, "replay returns the journaled walk");
    assert_eq!(
        dispatch.dispatched.lock().unwrap().len(),
        2,
        "replay must NOT re-dispatch any step"
    );
    assert_eq!(finalizes.load(Ordering::SeqCst), 1, "finalize exactly once");
}

/// Diamond DAG (root → {left, right} → join): the siblings journal as one
/// parallel node and the join runs only after both.
#[tokio::test]
async fn diamond_runs_siblings_as_one_parallel_node() {
    let (dispatch, _) = seeded(&[]);
    let program = PipelineRunCoordinator {
        pipeline: pipeline(vec![
            step("root", &[]),
            step("left", &["root"]),
            step("right", &["root"]),
            step("join", &["left", "right"]),
        ]),
    };

    let (status, journal) =
        execute(Journal::new(), dispatch.clone() as Arc<dyn Dispatch>, &program).await;
    let RunStatus::Completed(summary) = status else {
        panic!("coordinator must complete, got {status:?}");
    };
    assert_eq!(summary["completed"], json!(["root", "left", "right", "join"]));

    // Journal shape: dag record + (unblocked record + step slots) per round +
    // a final empty unblocked record + finalize.
    // round0: root (1+1), round1: left,right (1+2), round2: join (1+1),
    // round3: empty (1) → 1 + 2 + 3 + 2 + 1 + 1 = 10 entries.
    assert_eq!(
        journal.entries().len(),
        10,
        "siblings occupy one parallel node: {:?}",
        journal.entries().iter().map(|e| &e.op).collect::<Vec<_>>()
    );
}

/// A failed step halts the walk: downstream steps are never dispatched and
/// finalize carries the failed step id. Inconclusive-style success (the
/// router's advance-on-inconclusive) is the dispatch's job to encode in
/// `succeeded` — the walk only honors the boolean.
#[tokio::test]
async fn failed_step_halts_the_walk() {
    let (dispatch, finalizes) = seeded(&[("build", false)]);
    let program = PipelineRunCoordinator {
        pipeline: pipeline(vec![
            step("build", &[]),
            step("verify", &["build"]),
            step("ship", &["verify"]),
        ]),
    };

    let (status, _) = execute(Journal::new(), dispatch.clone() as Arc<dyn Dispatch>, &program).await;
    let RunStatus::Completed(summary) = status else {
        panic!("coordinator must complete (a failed step is a journaled outcome), got {status:?}");
    };
    assert_eq!(summary["failed_step"], json!("build"));
    assert_eq!(summary["completed"], json!(["build"]));
    assert_eq!(
        *dispatch.dispatched.lock().unwrap(),
        vec!["build".to_string()],
        "downstream steps must never dispatch after a failure"
    );
    assert_eq!(finalizes.load(Ordering::SeqCst), 1, "finalize still closes the run");
}

/// A failing SIBLING completes its parallel node (all-lanes-complete, the C6
/// semantics) but halts the next round.
#[tokio::test]
async fn failed_sibling_completes_node_then_halts() {
    let (dispatch, _) = seeded(&[("left", false)]);
    let program = PipelineRunCoordinator {
        pipeline: pipeline(vec![
            step("root", &[]),
            step("left", &["root"]),
            step("right", &["root"]),
            step("join", &["left", "right"]),
        ]),
    };

    let (status, _) = execute(Journal::new(), dispatch.clone() as Arc<dyn Dispatch>, &program).await;
    let RunStatus::Completed(summary) = status else {
        panic!("coordinator must complete, got {status:?}");
    };
    assert_eq!(summary["failed_step"], json!("left"));
    // Both siblings ran (one node); join never dispatched.
    assert_eq!(summary["completed"], json!(["root", "left", "right"]));
    assert!(
        !dispatch.dispatched.lock().unwrap().contains(&"join".to_string()),
        "join must not dispatch after a failed sibling"
    );
}
