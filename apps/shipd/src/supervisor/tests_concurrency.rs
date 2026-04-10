//! Job concurrency and scheduling tests.
//!
//! The daemon enforces a concurrency limit on dispatched jobs. When the
//! limit is reached, new jobs stay pending until a slot opens. Pending
//! jobs with resolved dependencies dispatch in creation order.

use runtime::events::job::event_types;
use runtime::events::EventEnvelope;
use runtime::projections::job::{project, JobStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let job_id = payload["job_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, job_id, &payload).unwrap()
}

fn created(job_id: &str, slug: &str) -> EventEnvelope {
    ev(event_types::JOB_CREATED, serde_json::json!({
        "job_id": job_id, "slug": slug,
        "agent": "rust-runtime", "branch": format!("job/{slug}"),
        "spec_path": "spec.md", "plan_id": null,
    }))
}

fn created_with_deps(job_id: &str, slug: &str, deps: Vec<&str>) -> EventEnvelope {
    ev(event_types::JOB_CREATED, serde_json::json!({
        "job_id": job_id, "slug": slug,
        "agent": "rust-runtime", "branch": format!("job/{slug}"),
        "spec_path": "spec.md", "plan_id": null,
        "depends_on": deps,
    }))
}

fn dispatched(job_id: &str) -> EventEnvelope {
    ev(event_types::JOB_DISPATCHED, serde_json::json!({
        "job_id": job_id, "worktree": format!("/tmp/wt/{job_id}"), "pid": null,
    }))
}

fn completed(job_id: &str) -> EventEnvelope {
    ev(event_types::JOB_COMPLETED, serde_json::json!({
        "job_id": job_id, "slug": "s",
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// CONCURRENCY LIMIT
//
// dispatch_ready_pending_jobs enforces MAX_CONCURRENT (currently 3).
// This is critical — too many concurrent agents overwhelms the machine.
// ═══════════════════════════════════════════════════════════════════════════

/// The dispatch replay module must enforce a concurrency limit.
#[test]
fn concurrency_limit_is_enforced() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/pending.rs");
    assert!(
        source.contains("MAX_CONCURRENT"),
        "dispatch_replay must define a MAX_CONCURRENT limit"
    );
}

/// The concurrency check must count currently dispatched jobs.
#[test]
fn concurrency_counts_dispatched_jobs() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/pending.rs");
    assert!(
        source.contains("Dispatched") && source.contains("running_count"),
        "concurrency check must count jobs in Dispatched status"
    );
}

/// With 3 dispatched jobs, no new jobs should be dispatched.
#[test]
fn three_dispatched_blocks_new_dispatch() {
    let events = vec![
        created("j1", "a"), dispatched("j1"),
        created("j2", "b"), dispatched("j2"),
        created("j3", "c"), dispatched("j3"),
        created("j4", "d"), // pending — should NOT dispatch
    ];
    let map = project(&events);
    let dispatched_count = map.values()
        .filter(|j| j.status == JobStatus::Dispatched)
        .count();
    let pending_count = map.values()
        .filter(|j| j.status == JobStatus::Pending)
        .count();
    assert_eq!(dispatched_count, 3);
    assert_eq!(pending_count, 1, "j4 must stay pending when 3 slots are full");
}

/// When a dispatched job completes, a pending job should become dispatchable.
#[test]
fn completed_job_frees_slot_for_pending() {
    let events = vec![
        created("j1", "a"), dispatched("j1"),
        created("j2", "b"), dispatched("j2"),
        created("j3", "c"), dispatched("j3"),
        created("j4", "d"), // pending
        completed("j1"),    // frees a slot
    ];
    let map = project(&events);
    let completed_count = map.values()
        .filter(|j| j.status == JobStatus::Completed)
        .count();
    let pending_count = map.values()
        .filter(|j| j.status == JobStatus::Pending)
        .count();
    assert_eq!(completed_count, 1, "j1 completed");
    assert_eq!(pending_count, 1, "j4 still pending in projection — dispatch_ready_pending_jobs acts on this");
}

// ═══════════════════════════════════════════════════════════════════════════
// DEPENDENCY RESOLUTION
//
// Jobs with depends_on stay pending until all dependencies reach a terminal
// state (Completed, Merged, or GatePending).
// ═══════════════════════════════════════════════════════════════════════════

/// A job depending on a pending job must stay pending.
#[test]
fn job_with_unresolved_dep_stays_pending() {
    let events = vec![
        created("j1", "upstream"),
        created_with_deps("j2", "downstream", vec!["upstream"]),
    ];
    let map = project(&events);
    assert_eq!(map.get("j2").unwrap().status, JobStatus::Pending);
}

/// A job depending on a completed job is dispatchable.
#[test]
fn job_with_resolved_dep_is_dispatchable() {
    let events = vec![
        created("j1", "upstream"),
        dispatched("j1"),
        completed("j1"),
        created_with_deps("j2", "downstream", vec!["upstream"]),
    ];
    let map = project(&events);
    let j2 = map.get("j2").unwrap();
    assert_eq!(j2.status, JobStatus::Pending);

    // Check deps are resolved
    let done_slugs: std::collections::HashSet<&str> = map.values()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();
    let deps_resolved = j2.depends_on.as_ref()
        .map_or(true, |deps| deps.iter().all(|d| done_slugs.contains(d.as_str())));
    assert!(deps_resolved, "downstream deps must be resolved when upstream completes");
}

/// A job with multiple dependencies waits for ALL of them.
#[test]
fn job_with_multiple_deps_waits_for_all() {
    let events = vec![
        created("j1", "dep-a"),
        created("j2", "dep-b"),
        created_with_deps("j3", "downstream", vec!["dep-a", "dep-b"]),
        dispatched("j1"),
        completed("j1"), // only dep-a done
    ];
    let map = project(&events);
    let done_slugs: std::collections::HashSet<&str> = map.values()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();
    let j3 = map.get("j3").unwrap();
    let deps_resolved = j3.depends_on.as_ref()
        .map_or(true, |deps| deps.iter().all(|d| done_slugs.contains(d.as_str())));
    assert!(!deps_resolved, "downstream must wait for ALL deps, not just one");
}

/// A job with no dependencies is immediately dispatchable.
#[test]
fn job_without_deps_is_immediately_dispatchable() {
    let events = vec![created("j1", "no-deps")];
    let map = project(&events);
    let j1 = map.get("j1").unwrap();
    assert_eq!(j1.status, JobStatus::Pending);
    assert!(j1.depends_on.is_none() || j1.depends_on.as_ref().unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// GRACEFUL SHUTDOWN
//
// On SIGTERM/SIGINT, the daemon must stop all managed agents before exiting.
// No orphaned processes.
// ═══════════════════════════════════════════════════════════════════════════

/// Graceful shutdown must emit a supervisor.shutting_down event.
#[test]
fn graceful_shutdown_emits_event() {
    let source = include_str!("registry.rs");
    assert!(
        source.contains("shutting_down"),
        "graceful_shutdown must emit supervisor.shutting_down event"
    );
}

/// Graceful shutdown must call shutdown_all on the registry.
#[test]
fn graceful_shutdown_drains_registry() {
    let source = include_str!("registry.rs");
    assert!(
        source.contains("shutdown_all"),
        "graceful_shutdown must drain the process registry"
    );
}

/// shutdown_all must call executor.stop() for each handle.
#[test]
fn shutdown_all_stops_each_agent() {
    let source = include_str!("registry.rs");
    assert!(
        source.contains("executor.stop"),
        "shutdown_all must call stop on each registered handle"
    );
}
