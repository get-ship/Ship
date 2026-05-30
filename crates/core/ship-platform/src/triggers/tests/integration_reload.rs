/// Reload semantics integration tests.
///
/// Tests `HookEngine::reload()` behavior: triggers added on reload become live,
/// triggers removed on reload stop firing, triggers replaced on reload fire only
/// the new definition, and an empty reload clears all triggers.
use serde_json::json;
use ship_substrate::events::EventEnvelope;
use crate::triggers::engine::HookEngine;

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

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_reload_added_trigger_becomes_live() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    // Initial reload — no triggers.
    engine.reload().await;
    assert!(engine.snapshot().await.is_empty());

    // Write a trigger file and reload.
    write_hook(
        &ship_dir,
        "new.jsonc",
        r#"{
            "name": "new",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "added": true } }
            ]
        }"#,
    );
    engine.reload().await;
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
    let hook_path = ship_dir.join("triggers").join("removable.jsonc");
    std::fs::write(
        &hook_path,
        r#"{
            "name": "removable",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
            ]
        }"#,
    )
    .unwrap();
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 1);

    // Remove the file and reload — trigger must stop.
    std::fs::remove_file(&hook_path).unwrap();
    engine.reload().await;
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
    for name in ["t1", "t2", "t3"] {
        write_hook(
            &ship_dir,
            &format!("{name}.jsonc"),
            &format!(
                r#"{{ "name": "{name}", "trigger": {{ "event": "pipeline.run.failed" }},
                       "actions": [] }}"#
            ),
        );
    }
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 3);

    // Remove all trigger files.
    for name in ["t1", "t2", "t3"] {
        std::fs::remove_file(ship_dir.join("triggers").join(format!("{name}.jsonc"))).unwrap();
    }
    engine.reload().await;
    assert!(
        engine.snapshot().await.is_empty(),
        "all triggers must be cleared after empty-dir reload"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_reload_replaced_trigger_fires_new_definition() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    let hook_path = ship_dir.join("triggers").join("mutable.jsonc");

    // First definition emits payload version "v1".
    std::fs::write(
        &hook_path,
        r#"{
            "name": "mutable",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "version": "v1" } }
            ]
        }"#,
    )
    .unwrap();
    engine.reload().await;

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let first_emitted = diff(&before, &after);
    assert_eq!(first_emitted.len(), 1);
    let p: serde_json::Value =
        serde_json::from_str(&first_emitted[0].payload_json).unwrap();
    assert_eq!(p["version"], "v1");

    // Overwrite with version "v2" and reload.
    std::fs::write(
        &hook_path,
        r#"{
            "name": "mutable",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "version": "v2" } }
            ]
        }"#,
    )
    .unwrap();
    engine.reload().await;

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
    // A reload that encounters one bad file must still load the good ones.
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());

    write_hook(
        &ship_dir,
        "good.jsonc",
        r#"{ "name": "good", "trigger": { "event": "pipeline.run.failed" }, "actions": [] }"#,
    );
    write_hook(&ship_dir, "bad.jsonc", "not valid json {");
    engine.reload().await;

    assert_eq!(
        engine.snapshot().await.len(),
        1,
        "valid trigger must survive alongside a bad file"
    );
    assert_eq!(engine.snapshot().await[0].name, "good");
}
