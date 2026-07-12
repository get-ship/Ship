/// Reload semantics integration tests.
///
/// Tests `HookEngine::reload()` behavior: triggers added on reload become live,
/// triggers removed on reload stop firing, triggers replaced on reload fire only
/// the new definition, and an empty reload clears all triggers.
use serde_json::json;
use ship_substrate::events::EventEnvelope;
use crate::engine::HookEngine;

use super::integration_engine::{
    diff, recent_hook_events, test_kernel, tmp_ship_dir, trigger_def,
};

fn run_failed_envelope() -> EventEnvelope {
    EventEnvelope::new(
        "pipeline.run.started",
        "run-1",
        &json!({ "pipeline_run_id": "run-1", "error": "boom", "status": "failed" }),
    )
    .unwrap()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_reload_added_trigger_becomes_live() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    // Initial reload — no triggers.
    engine.reload(&[]).await;
    assert!(engine.snapshot().await.is_empty());

    // Add a trigger and reload.
    let defs = vec![trigger_def(
        r#"{
            "id": "new",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "added": true } }
            ]
        }"#,
    )];
    engine.reload(&defs).await;
    assert_eq!(engine.snapshot().await.len(), 1, "trigger must be live after reload");

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    assert_eq!(
        diff(&before, &after).len(),
        1,
        "newly added trigger must fire after reload"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_reload_removed_trigger_stops_firing() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    // Start with one trigger.
    let defs = vec![trigger_def(
        r#"{
            "id": "removable",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
            ]
        }"#,
    )];
    engine.reload(&defs).await;
    assert_eq!(engine.snapshot().await.len(), 1);

    // Reload with an empty set — trigger must stop.
    engine.reload(&[]).await;
    assert!(
        engine.snapshot().await.is_empty(),
        "removed trigger must be absent after reload"
    );

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    assert!(
        diff(&before, &after).is_empty(),
        "removed trigger must not fire after reload"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_reload_empty_dir_clears_all_triggers() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    // Load three triggers.
    let defs: Vec<stack_model::TriggerDef> = ["t1", "t2", "t3"]
        .iter()
        .map(|name| {
            trigger_def(&format!(
                r#"{{ "id": "{name}", "trigger": {{ "event": "pipeline.run.started" }},
                       "actions": [] }}"#
            ))
        })
        .collect();
    engine.reload(&defs).await;
    assert_eq!(engine.snapshot().await.len(), 3);

    // Reload with an empty set — all cleared.
    engine.reload(&[]).await;
    assert!(
        engine.snapshot().await.is_empty(),
        "all triggers must be cleared after empty reload"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_reload_replaced_trigger_fires_new_definition() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    // First definition emits payload version "v1".
    let v1 = vec![trigger_def(
        r#"{
            "id": "mutable",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "version": "v1" } }
            ]
        }"#,
    )];
    engine.reload(&v1).await;

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let first_emitted = diff(&before, &after);
    assert_eq!(first_emitted.len(), 1);
    let p: serde_json::Value =
        serde_json::from_str(&first_emitted[0].payload_json).unwrap();
    assert_eq!(p["version"], "v1");

    // Reload with version "v2".
    let v2 = vec![trigger_def(
        r#"{
            "id": "mutable",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "version": "v2" } }
            ]
        }"#,
    )];
    engine.reload(&v2).await;

    let before2 = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after2 = recent_hook_events(&ship_dir);
    let second_emitted = diff(&before2, &after2);
    assert_eq!(second_emitted.len(), 1);
    let p2: serde_json::Value =
        serde_json::from_str(&second_emitted[0].payload_json).unwrap();
    assert_eq!(p2["version"], "v2", "reloaded trigger must use new definition");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_reload_invalid_hook_does_not_discard_valid_hooks() {
    // A reload that encounters one bad def must still load the good ones.
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    let defs = vec![
        trigger_def(
            r#"{ "id": "good", "trigger": { "event": "pipeline.run.started" }, "actions": [] }"#,
        ),
        // Unknown event type — invalid, must be skipped, not fatal.
        trigger_def(
            r#"{ "id": "bad", "trigger": { "event": "not.a.real.event" }, "actions": [] }"#,
        ),
    ];
    engine.reload(&defs).await;

    assert_eq!(
        engine.snapshot().await.len(),
        1,
        "valid trigger must survive alongside a bad def"
    );
    assert_eq!(engine.snapshot().await[0].name, "good");
}
