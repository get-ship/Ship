//! Behavioral tests for the journaled fanout node (C6 / P2.4).
//!
//! The monitor/router lane plumbing is pinned by the existing fanout suites;
//! here we pin the PROGRAM contract: lane outcomes journal as one parallel
//! node in input order, the aggregate decision is a journaled value, and the
//! parent-outcome emission happens exactly once under replay.

use async_trait::async_trait;
use durable_execution::{Dispatch, DispatchError, Journal, RunStatus, execute};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::LaneOutcome;
use super::coordinator_programs::FanoutProgram;

/// Lane ops answer from a pre-seeded table; the emit op only counts.
struct SeededDispatch {
    lanes: BTreeMap<String, Value>,
    emits: Arc<AtomicUsize>,
}

#[async_trait]
impl Dispatch for SeededDispatch {
    async fn call(&self, op: &str, _args: Value) -> Result<Value, DispatchError> {
        if let Some(v) = self.lanes.get(op) {
            return Ok(v.clone());
        }
        match op {
            "fanout.emit_outcome" => {
                self.emits.fetch_add(1, Ordering::SeqCst);
                Ok(json!({}))
            }
            other => Err(DispatchError::Terminal(format!("unknown op `{other}`"))),
        }
    }
}

fn reported(lane_id: &str, success: bool) -> Value {
    json!({
        "reported": true,
        "outcome": serde_json::to_value(LaneOutcome {
            lane_id: lane_id.into(),
            success,
            outputs: BTreeMap::new(),
        })
        .unwrap(),
    })
}

/// Two successful lanes with min_ok=2 → Advance journaled, emission once,
/// and a replay re-drive returns the SAME decision without re-emitting.
#[tokio::test]
async fn fanout_aggregates_and_emits_exactly_once() {
    let emits = Arc::new(AtomicUsize::new(0));
    let lanes: BTreeMap<String, Value> = [
        ("fanout.lane.a".to_string(), reported("a", true)),
        ("fanout.lane.b".to_string(), reported("b", true)),
    ]
    .into();
    let dispatch: Arc<dyn Dispatch> = Arc::new(SeededDispatch { lanes, emits: emits.clone() });
    let program = FanoutProgram {
        lane_ops: vec!["fanout.lane.a".into(), "fanout.lane.b".into()],
        min_ok: 2,
        total: 2,
    };

    let (status, journal) = execute(Journal::new(), dispatch.clone(), &program).await;
    let RunStatus::Completed(aggregate) = status else {
        panic!("fanout program must complete, got {status:?}");
    };
    assert_eq!(aggregate["reported"], json!(2));
    assert_eq!(aggregate["decision"]["Advance"]["ok_count"], json!(2));
    assert_eq!(emits.load(Ordering::SeqCst), 1, "parent outcome emitted once");
    // 2 lane slots + 1 aggregate record + 1 emit op.
    assert_eq!(journal.entries().len(), 4);

    let (status2, _) = execute(journal, dispatch, &program).await;
    let RunStatus::Completed(aggregate2) = status2 else {
        panic!("replay must complete");
    };
    assert_eq!(aggregate2, aggregate, "replay returns the journaled decision");
    assert_eq!(emits.load(Ordering::SeqCst), 1, "replay must NOT re-emit");
}

/// An unreported lane (sender dropped — cancellation teardown) is excluded
/// from the outcomes but still counts against min_ok via `total`: one success
/// out of total=2 with min_ok=2 → Fail.
#[tokio::test]
async fn unreported_lane_fails_min_ok() {
    let emits = Arc::new(AtomicUsize::new(0));
    let lanes: BTreeMap<String, Value> = [
        ("fanout.lane.a".to_string(), reported("a", true)),
        (
            "fanout.lane.b".to_string(),
            json!({ "reported": false, "outcome": Value::Null }),
        ),
    ]
    .into();
    let dispatch: Arc<dyn Dispatch> = Arc::new(SeededDispatch { lanes, emits: emits.clone() });
    let program = FanoutProgram {
        lane_ops: vec!["fanout.lane.a".into(), "fanout.lane.b".into()],
        min_ok: 2,
        total: 2,
    };

    let (status, _) = execute(Journal::new(), dispatch, &program).await;
    let RunStatus::Completed(aggregate) = status else {
        panic!("fanout program must complete, got {status:?}");
    };
    assert_eq!(aggregate["reported"], json!(1));
    assert_eq!(aggregate["decision"]["Fail"]["ok_count"], json!(1));
    assert_eq!(
        emits.load(Ordering::SeqCst),
        1,
        "the emit op still runs — it owns the Fail emission (and terminal suppression)"
    );
}
