//! Contract tests for the typed AGENT CONTROL results (`AgentsService`):
//! Dispatch / Stop / Steer / Grant (S4). Each was a `JsonResponse` blob; the
//! fixture now decodes into a real message. The TS Connect client asserts the
//! inverse against the same `agents_service.json` file.

use serde_json::Value;
use ship_ops_rpc::{DispatchResult, GrantResult, SteerResult, StopResult};

const AGENT: &str = include_str!("../../../../proto/fixtures/agents_service.json");

fn response_ok(fixtures_json: &str, method: &str) -> Value {
    let v: Value = serde_json::from_str(fixtures_json).expect("fixtures parse");
    v[method]["response_ok"].clone()
}

#[test]
fn dispatch_result_carries_session_and_slug() {
    let r: DispatchResult =
        serde_json::from_value(response_ok(AGENT, "Dispatch")).expect("parse DispatchResult");
    assert!(!r.session_id.is_empty());
    assert!(!r.slug.is_empty());
    assert!(!r.studio_url.is_empty());
}

#[test]
fn stop_result_is_structured() {
    // No more `"stopped: <slug>"` JSON string — a real {slug, stopped} object.
    let r: StopResult =
        serde_json::from_value(response_ok(AGENT, "Stop")).expect("parse StopResult");
    assert!(!r.slug.is_empty());
    assert!(r.stopped);
}

#[test]
fn steer_result_is_structured() {
    let r: SteerResult =
        serde_json::from_value(response_ok(AGENT, "Steer")).expect("parse SteerResult");
    assert!(!r.slug.is_empty());
    assert!(r.steered);
}

#[test]
fn grant_result_carries_provisioned_baseline() {
    let r: GrantResult =
        serde_json::from_value(response_ok(AGENT, "Grant")).expect("parse GrantResult");
    assert!(!r.agent.is_empty());
    assert!(!r.grants.is_empty());
    assert!(!r.provisioned.is_empty());
}
