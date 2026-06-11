//! Program-level behavioral tests for `AgentStepProgram` (C5 / P2.3).
//!
//! The monitor-level behavior (timers feeding evidence, dedup on a completed
//! journal) is pinned in `tests_heartbeat`; here we pin the program contract
//! itself: park at `agent.running`, resolve with evidence, verdict via the
//! ladder, emission exactly once under replay.

use durable_execution::{Dispatch, DispatchError, Journal, RunStatus, execute, resume};
use async_trait::async_trait;
use serde_json::{Value, json};
use ship_substrate::events::pipeline::{StepTrigger, StepType, StepVerdict};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::coordinator_programs::{AgentStepProgram, StepEvidence};

/// Counts `agent.emit_observed` calls — the effectful op whose exactly-once
/// property is the point of the journal.
struct CountingDispatch {
    emits: Arc<AtomicUsize>,
}

#[async_trait]
impl Dispatch for CountingDispatch {
    async fn call(&self, op: &str, _args: Value) -> Result<Value, DispatchError> {
        match op {
            "agent.spawn" => Ok(json!({ "bridged": true })),
            "agent.emit_observed" => {
                self.emits.fetch_add(1, Ordering::SeqCst);
                Ok(json!({}))
            }
            other => Err(DispatchError::Terminal(format!("unknown op `{other}`"))),
        }
    }
}

fn evidence(trigger: StepTrigger, done_declared: bool, commits: u32) -> Value {
    serde_json::to_value(StepEvidence {
        trigger,
        exit_code: None,
        done_declared,
        reason: None,
        commits,
        uncommitted: 0,
        turns: 3,
    })
    .unwrap()
}

/// The full lifecycle: park → resolve(DoneDeclared) → Advance verdict →
/// emission exactly once, including across a replay re-drive.
#[tokio::test]
async fn agent_step_parks_resolves_and_emits_exactly_once() {
    let emits = Arc::new(AtomicUsize::new(0));
    let dispatch: Arc<dyn Dispatch> = Arc::new(CountingDispatch { emits: emits.clone() });
    let program = AgentStepProgram { step_type: StepType::Agent };

    // Fresh journal: spawn (echo) then park at agent.running.
    let (status, journal) = execute(Journal::new(), dispatch.clone(), &program).await;
    assert_eq!(status, RunStatus::Suspended, "must park while the agent runs");
    assert!(journal.is_suspended());
    assert_eq!(emits.load(Ordering::SeqCst), 0, "no emission before evidence");

    // Resolve the suspend with done-declared evidence → verdict Advance.
    let (status, journal) = resume(
        journal,
        evidence(StepTrigger::DoneDeclared, true, 2),
        dispatch.clone(),
        &program,
    )
    .await;
    let RunStatus::Completed(verdict) = status else {
        panic!("resolved step must complete, got {status:?}");
    };
    assert_eq!(
        verdict,
        serde_json::to_value(StepVerdict::Advance).unwrap(),
        "done_declared(true) must resolve to Advance via the ladder"
    );
    assert_eq!(emits.load(Ordering::SeqCst), 1, "observation emitted once");

    // Re-drive the completed journal (restart / re-dispatch): everything
    // replays, nothing re-emits.
    let (status, _) = execute(journal, dispatch, &program).await;
    assert!(matches!(status, RunStatus::Completed(_)));
    assert_eq!(
        emits.load(Ordering::SeqCst),
        1,
        "replay must NOT re-emit the observation"
    );
}

/// Timeout evidence with no work product resolves to Failed — the ladder is
/// journaled via the `agent.verdict` record, so replay returns the same
/// verdict without recomputing.
#[tokio::test]
async fn timeout_without_work_resolves_failed_and_verdict_is_journaled() {
    let emits = Arc::new(AtomicUsize::new(0));
    let dispatch: Arc<dyn Dispatch> = Arc::new(CountingDispatch { emits: emits.clone() });
    let program = AgentStepProgram { step_type: StepType::Agent };

    let (_, journal) = execute(Journal::new(), dispatch.clone(), &program).await;
    let (status, journal) = resume(
        journal,
        evidence(StepTrigger::Timeout, false, 0),
        dispatch.clone(),
        &program,
    )
    .await;
    let RunStatus::Completed(verdict) = status else {
        panic!("resolved step must complete, got {status:?}");
    };
    assert_eq!(verdict, serde_json::to_value(StepVerdict::Failed).unwrap());

    // The verdict lives in the journal: the record entry holds it, so a
    // replay yields the identical verdict value.
    let (status2, _) = execute(journal, dispatch, &program).await;
    let RunStatus::Completed(verdict2) = status2 else {
        panic!("replay must complete");
    };
    assert_eq!(verdict2, serde_json::to_value(StepVerdict::Failed).unwrap());
    assert_eq!(emits.load(Ordering::SeqCst), 1);
}
