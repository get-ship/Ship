/// Filter integration tests — dot-path filter matching.
///
/// Covers: absent filter passes, exact match, mismatch rejected, missing path
/// rejected, multi-key all-must-match, array index paths, and engine dispatch
/// with matching and non-matching filters.
use serde_json::json;
use ship_substrate::events::EventEnvelope;
use crate::filter;

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
        "event_type": "pipeline.run.started",
        "payload": { "status": "failed" }
    });
    let filter_val = json!({
        "event_type": "pipeline.run.started",
        "payload.status": "failed",
    });
    assert!(filter::matches(Some(&filter_val), &ctx));
}

#[test]
fn triggers_filter_multi_key_partial_match_rejected() {
    let ctx = json!({
        "event_type": "pipeline.run.started",
        "payload": { "status": "ok" }
    });
    let filter_val = json!({
        "event_type": "pipeline.run.started",
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
        "pipeline.run.started",
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
        Some(&json!({ "event_type": "pipeline.run.started" })),
        &ctx
    ));
    assert!(filter::matches(
        Some(&json!({ "entity_id": "job-1" })),
        &ctx
    ));
}

// `triggers_filter_engine_passes_with_matching_filter` and
// `triggers_filter_engine_blocks_with_nonmatching_filter` (the
// descriptor-driven filter-gated dispatch cases) are retired:
// `compiler::TriggerDef` has no TS shape for the dot-path payload filter
// yet (def.rs's `convert_trigger_match` always emits `filter: None`), so
// there is no descriptor input left to build them from. `filter::matches`'s
// dot-path matching logic itself stays fully covered by the unit tests
// above, which call it directly.
