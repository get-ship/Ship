/// Filter integration tests — dot-path filter matching.
///
/// Covers: absent filter passes, exact match, mismatch rejected, missing path
/// rejected, multi-key all-must-match, array index paths, and engine dispatch
/// with matching and non-matching filters.
use serde_json::json;
use ship_substrate::events::EventEnvelope;
use crate::triggers::filter;
use crate::triggers::engine::HookEngine;

use super::integration_engine::{
    diff, recent_hook_events, test_kernel, tmp_ship_dir, write_hook,
};

fn make_envelope(event_type: &str, entity_id: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, entity_id, &payload).unwrap()
}

// ── filter::matches unit coverage ────────────────────────────────────────────

#[test]
fn triggers_filter_absent_filter_matches_any_context() {
    assert!(filter::matches(None, &json!({})));
    assert!(filter::matches(None, &json!({ "anything": "here" })));
}

#[test]
fn triggers_filter_exact_match_on_simple_path() {
    let ctx = json!({ "payload": { "status": "failed" } });
    assert!(filter::matches(
        Some(&json!({ "payload.status": "failed" })),
        &ctx
    ));
}

#[test]
fn triggers_filter_mismatch_on_simple_path_rejected() {
    let ctx = json!({ "payload": { "status": "ok" } });
    assert!(!filter::matches(
        Some(&json!({ "payload.status": "failed" })),
        &ctx
    ));
}

#[test]
fn triggers_filter_missing_path_rejected() {
    let ctx = json!({ "payload": {} });
    assert!(!filter::matches(
        Some(&json!({ "payload.status": "failed" })),
        &ctx
    ));
}

#[test]
fn triggers_filter_multi_key_all_must_match() {
    let ctx = json!({
        "event_type": "pipeline.run.completed",
        "payload": { "status": "failed" }
    });
    let filter_val = json!({
        "event_type": "pipeline.run.completed",
        "payload.status": "failed",
    });
    assert!(filter::matches(Some(&filter_val), &ctx));
}

#[test]
fn triggers_filter_multi_key_partial_match_rejected() {
    let ctx = json!({
        "event_type": "pipeline.run.completed",
        "payload": { "status": "ok" }
    });
    let filter_val = json!({
        "event_type": "pipeline.run.completed",
        "payload.status": "failed",
    });
    // Both keys must match; payload.status doesn't, so whole filter fails.
    assert!(!filter::matches(Some(&filter_val), &ctx));
}

#[test]
fn triggers_filter_array_index_path() {
    let ctx = json!({ "payload": { "items": ["a", "b", "c"] } });
    assert!(filter::matches(
        Some(&json!({ "payload.items.0": "a" })),
        &ctx
    ));
    assert!(!filter::matches(
        Some(&json!({ "payload.items.0": "b" })),
        &ctx
    ));
}

#[test]
fn triggers_filter_build_event_context_exposes_envelope_fields() {
    let env = EventEnvelope::new(
        "pipeline.run.failed",
        "job-1",
        &json!({ "pipeline_run_id": "job-1", "error": "boom", "status": "failed" }),
    )
    .unwrap();
    let ctx = filter::build_event_context(&env);
    assert!(filter::matches(
        Some(&json!({ "payload.status": "failed" })),
        &ctx
    ));
    assert!(!filter::matches(
        Some(&json!({ "payload.status": "ok" })),
        &ctx
    ));
    // Envelope-level fields are accessible via dot-path.
    assert!(filter::matches(
        Some(&json!({ "event_type": "pipeline.run.failed" })),
        &ctx
    ));
    assert!(filter::matches(
        Some(&json!({ "entity_id": "job-1" })),
        &ctx
    ));
}

// ── Engine dispatch with filter ───────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_filter_engine_passes_with_matching_filter() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "filtered.jsonc",
        r#"{
            "name": "filtered",
            "trigger": {
                "event": "pipeline.run.completed",
                "filter": { "payload.status": "ok" }
            },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped",
                  "payload": { "matched": true } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let env = make_envelope(
        "pipeline.run.completed",
        "run-1",
        json!({ "pipeline_run_id": "run-1", "slug": "run-1", "status": "ok" }),
    );

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&env).await;
    let after = recent_hook_events(&ship_dir);
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 1, "trigger with matching filter must fire");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_filter_engine_blocks_with_nonmatching_filter() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "strict-filter.jsonc",
        r#"{
            "name": "strict-filter",
            "trigger": {
                "event": "pipeline.run.completed",
                "filter": { "payload.status": "failed" }
            },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    // Status is "ok", filter expects "failed" — must not fire.
    let env = make_envelope(
        "pipeline.run.completed",
        "run-1",
        json!({ "pipeline_run_id": "run-1", "slug": "run-1", "status": "ok" }),
    );

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&env).await;
    let after = recent_hook_events(&ship_dir);
    assert!(
        diff(&before, &after).is_empty(),
        "trigger with non-matching filter must not fire"
    );
}
