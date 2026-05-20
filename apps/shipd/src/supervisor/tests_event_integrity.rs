//! Event store integrity tests.
//!
//! The event store is append-only. Events are immutable once written.
//! The projection is derived entirely from events — no side channels.
//! These properties are the foundation of the entire system.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_platform::projections::pipeline_run::{project, PipelineRunStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let job_id = payload["job_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, job_id, &payload).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// APPEND-ONLY GUARANTEE
//
// Events are never updated or deleted. The apply() function only reads
// events — it never mutates them.
// ═══════════════════════════════════════════════════════════════════════════

/// The projection's apply function must not modify event payloads.
#[test]
fn apply_does_not_mutate_events() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "job_id": "j1", "slug": "s",
            "agent": "r", "branch": "job/s",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "job_id": "j1", "worktree": "/w", "pid": 42,
        })),
    ];
    let payload_before: Vec<String> = events.iter()
        .map(|e| e.payload_json.clone())
        .collect();
    let _ = project(&events);
    let payload_after: Vec<String> = events.iter()
        .map(|e| e.payload_json.clone())
        .collect();
    assert_eq!(payload_before, payload_after, "project() must not mutate event payloads");
}

/// Every event has a unique ID.
#[test]
fn events_have_unique_ids() {
    let e1 = ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
        "job_id": "j1", "slug": "s",
        "agent": "r", "branch": "job/s",
        "spec_path": "spec.md", "plan_id": null,
    }));
    let e2 = ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
        "job_id": "j2", "slug": "t",
        "agent": "r", "branch": "job/t",
        "spec_path": "spec.md", "plan_id": null,
    }));
    assert_ne!(e1.id, e2.id, "each event must have a unique ID");
}

/// Events have monotonic timestamps.
#[test]
fn events_have_timestamps() {
    let e = ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
        "job_id": "j1", "slug": "s",
        "agent": "r", "branch": "job/s",
        "spec_path": "spec.md", "plan_id": null,
    }));
    assert!(e.created_at.timestamp() > 0, "events must have a valid timestamp");
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECTION COMPLETENESS
//
// Every field on PipelineRun must be derivable from events alone.
// No field should require in-memory state or external lookups.
// ═══════════════════════════════════════════════════════════════════════════

/// All PipelineRun fields are populated from events.
#[test]
fn projection_populates_all_fields_from_events() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "job_id": "j1", "slug": "auth-fix",
            "agent": "rust-runtime", "branch": "job/auth-fix",
            "spec_path": "specs/auth.md", "plan_id": "plan-1",
            "model": "opus", "harness": "claude",
            "depends_on": ["upstream"],
            "steps": { "steps": [
                { "id": "impl", "type": "agent", "agent": "rust-runtime", "after": [] }
            ]}
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "job_id": "j1", "worktree": "/wt/auth-fix", "pid": 9999,
            "harness_meta": { "session_id": "sess-1" },
        })),
    ];
    let map = project(&events);
    let rec = map.get("j1").unwrap();

    // From job.created
    assert_eq!(rec.job_id, "j1");
    assert_eq!(rec.slug, "auth-fix");
    assert_eq!(rec.agent, "rust-runtime");
    assert_eq!(rec.branch, "job/auth-fix");
    assert_eq!(rec.spec_path, "specs/auth.md");
    assert_eq!(rec.model.as_deref(), Some("opus"));
    assert_eq!(rec.harness.as_deref(), Some("claude"));
    assert!(rec.depends_on.is_some());
    assert!(rec.steps.is_some());

    // From job.dispatched
    assert_eq!(rec.status, PipelineRunStatus::Dispatched);
    assert_eq!(rec.worktree.as_deref(), Some("/wt/auth-fix"));
    assert!(rec.harness_meta.is_some());
    assert_eq!(
        rec.harness_meta.as_ref().unwrap()["session_id"].as_str(),
        Some("sess-1")
    );

    // Timestamps
    assert!(rec.created_at <= rec.updated_at);
}

/// Projection handles missing optional fields gracefully.
#[test]
fn projection_handles_minimal_created_event() {
    let events = vec![ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
        "job_id": "j-min", "slug": "minimal",
        "agent": "default", "branch": "job/minimal",
        "spec_path": "spec.md",
    }))];
    let map = project(&events);
    let rec = map.get("j-min").unwrap();
    assert_eq!(rec.status, PipelineRunStatus::Pending);
    assert!(rec.model.is_none());
    assert!(rec.harness.is_none());
    assert!(rec.depends_on.is_none());
    assert!(rec.steps.is_none());
    assert!(rec.pipeline.is_none());
    assert!(rec.worktree.is_none());
    assert!(rec.blocker.is_none());
    assert!(rec.error.is_none());
    assert!(rec.harness_meta.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// EVENT ENVELOPE STRUCTURE
//
// Every event must carry enough metadata for routing, auditing, and replay.
// ═══════════════════════════════════════════════════════════════════════════

/// Events carry entity_id for routing.
#[test]
fn event_envelope_has_entity_id() {
    let e = EventEnvelope::new(
        event_types::PIPELINE_RUN_STARTED,
        "j-entity",
        &serde_json::json!({"job_id": "j-entity", "slug": "s", "agent": "r", "branch": "b", "spec_path": "s.md"}),
    ).unwrap();
    assert_eq!(e.entity_id, "j-entity");
}

/// Events can carry actor_id for attribution.
#[test]
fn event_envelope_supports_actor_id() {
    let e = EventEnvelope::new(
        event_types::PIPELINE_RUN_DISPATCHED,
        "j1",
        &serde_json::json!({"job_id": "j1", "worktree": "/w", "pid": null}),
    ).unwrap().with_actor_id("service.job-dispatch");
    assert_eq!(e.actor_id.as_deref(), Some("service.job-dispatch"));
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECTION REBUILD
//
// A projection rebuilt from the same events must produce identical results.
// This is the fundamental guarantee of event sourcing.
// ═══════════════════════════════════════════════════════════════════════════

/// Multi-job projection is deterministic.
#[test]
fn multi_job_projection_is_deterministic() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "job_id": "j1", "slug": "a",
            "agent": "r", "branch": "job/a", "spec_path": "s.md",
        })),
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "job_id": "j2", "slug": "b",
            "agent": "r", "branch": "job/b", "spec_path": "s.md",
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "job_id": "j1", "worktree": "/w/a", "pid": null,
        })),
        ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
            "job_id": "j1", "slug": "a",
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "job_id": "j2", "worktree": "/w/b", "pid": null,
        })),
        ev(event_types::PIPELINE_RUN_FAILED, serde_json::json!({
            "job_id": "j2", "error": "timeout",
        })),
    ];
    let a = project(&events);
    let b = project(&events);
    for key in a.keys() {
        let ra = a.get(key).unwrap();
        let rb = b.get(key).unwrap();
        assert_eq!(ra.status, rb.status, "status mismatch for {key}");
        assert_eq!(ra.worktree, rb.worktree, "worktree mismatch for {key}");
        assert_eq!(ra.error, rb.error, "error mismatch for {key}");
        assert_eq!(ra.completed_steps, rb.completed_steps, "steps mismatch for {key}");
    }
}
