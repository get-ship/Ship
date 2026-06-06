/// Fan-out integration tests — N triggers matching one event.
///
/// Verifies that when multiple triggers subscribe to the same event, all of
/// them fire; that triggers subscribing to different events only fire for their
/// own event; and that an empty trigger set produces no actions.
use serde_json::{Value, json};
use ship_substrate::events::EventEnvelope;
use crate::engine::HookEngine;

use super::integration_engine::{
    diff, recent_hook_events, test_kernel, tmp_ship_dir, write_hook,
};

fn run_failed_envelope() -> EventEnvelope {
    EventEnvelope::new(
        "pipeline.run.failed",
        "run-1",
        &json!({ "pipeline_run_id": "run-1", "error": "boom", "status": "failed" }),
    )
    .unwrap()
}

fn run_completed_envelope() -> EventEnvelope {
    EventEnvelope::new(
        "pipeline.run.completed",
        "run-1",
        &json!({ "pipeline_run_id": "run-1", "slug": "run-1", "status": "ok" }),
    )
    .unwrap()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_fan_out_two_triggers_same_event_both_fire() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "hook-a.jsonc",
        r#"{
            "name": "hook-a",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "a" } }
            ]
        }"#,
    );
    write_hook(
        &ship_dir,
        "hook-b.jsonc",
        r#"{
            "name": "hook-b",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "b" } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 2);

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let emitted = diff(&before, &after);

    assert_eq!(emitted.len(), 2, "both triggers must fire for the same event");
    let froms: Vec<String> = emitted
        .iter()
        .map(|e| {
            let p: Value = serde_json::from_str(&e.payload_json).unwrap();
            p["from"].as_str().unwrap_or("").to_string()
        })
        .collect();
    assert!(froms.contains(&"a".to_string()), "hook-a must fire");
    assert!(froms.contains(&"b".to_string()), "hook-b must fire");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_fan_out_three_triggers_same_event_all_fire() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    for name in ["hook-1", "hook-2", "hook-3"] {
        write_hook(
            &ship_dir,
            &format!("{name}.jsonc"),
            &format!(
                r#"{{
                    "name": "{name}",
                    "trigger": {{ "event": "pipeline.run.failed" }},
                    "actions": [
                        {{ "type": "event.emit", "event_type": "hook.tripped",
                           "payload": {{ "from": "{name}" }} }}
                    ]
                }}"#
            ),
        );
    }

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 3);

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let emitted = diff(&before, &after);

    assert_eq!(emitted.len(), 3, "all three triggers must fire for one event");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_fan_out_disjoint_event_types_only_matching_fires() {
    // Two triggers on different events. Dispatching one event fires exactly one.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "on-fail.jsonc",
        r#"{
            "name": "on-fail",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "fail" } }
            ]
        }"#,
    );
    write_hook(
        &ship_dir,
        "on-done.jsonc",
        r#"{
            "name": "on-done",
            "trigger": { "event": "pipeline.run.completed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "done" } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_hook_events(&ship_dir);
    // Dispatch completed — only on-done must fire.
    engine.dispatch_event(&run_completed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let emitted = diff(&before, &after);

    assert_eq!(emitted.len(), 1, "only the on-done trigger fires for completed");
    let payload: Value = serde_json::from_str(&emitted[0].payload_json).unwrap();
    assert_eq!(payload["from"], "done");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_fan_out_no_triggers_produces_no_actions() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    // No trigger files written — engine starts empty.
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;
    assert!(engine.snapshot().await.is_empty());

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    assert!(
        diff(&before, &after).is_empty(),
        "empty trigger set must produce zero actions"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_fan_out_mixed_subscriptions_independent() {
    // Three triggers: two on failed, one on completed.
    // Dispatching failed fires 2; dispatching completed fires 1.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "fail-1.jsonc",
        r#"{ "name": "fail-1", "trigger": { "event": "pipeline.run.failed" },
            "actions": [{ "type": "event.emit", "event_type": "hook.tripped",
                          "payload": {"t":"f1"} }] }"#,
    );
    write_hook(
        &ship_dir,
        "fail-2.jsonc",
        r#"{ "name": "fail-2", "trigger": { "event": "pipeline.run.failed" },
            "actions": [{ "type": "event.emit", "event_type": "hook.tripped",
                          "payload": {"t":"f2"} }] }"#,
    );
    write_hook(
        &ship_dir,
        "done-1.jsonc",
        r#"{ "name": "done-1", "trigger": { "event": "pipeline.run.completed" },
            "actions": [{ "type": "event.emit", "event_type": "hook.tripped",
                          "payload": {"t":"d1"} }] }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 3);

    let before_fail = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after_fail = recent_hook_events(&ship_dir);
    let fail_emitted = diff(&before_fail, &after_fail);
    assert_eq!(fail_emitted.len(), 2, "2 fail triggers must fire on failed event");

    let before_done = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_completed_envelope()).await;
    let after_done = recent_hook_events(&ship_dir);
    let done_emitted = diff(&before_done, &after_done);
    assert_eq!(done_emitted.len(), 1, "1 done trigger must fire on completed event");
}
