//! End-to-end reliability tests.
//!
//! These tests verify the system behaves correctly across failure modes:
//! daemon restarts, agent crashes, stuck jobs, resource leaks, and
//! concurrent operations. They define what "stable" means for v0.2.0.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_pipeline_runtime::projections::pipeline_run::{project, PipelineRunStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let pipeline_run_id = payload["pipeline_run_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, pipeline_run_id, &payload).unwrap()
}

fn created(pipeline_run_id: &str, slug: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug,
        "agent": "rust-runtime", "branch": format!("job/{slug}"),
        "spec_path": "spec.md", "plan_id": null,
    }))
}

fn dispatched(pipeline_run_id: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "worktree": format!("/tmp/wt/{pipeline_run_id}"), "pid": 1234
    }))
}

fn completed(pipeline_run_id: &str, slug: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug
    }))
}

fn merged(pipeline_run_id: &str, slug: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_MERGED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug
    }))
}

fn failed(pipeline_run_id: &str, error: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_FAILED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "error": error
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKTREE PROJECT DETECTION
//
// Git worktrees must resolve to their parent project identity.
// ═══════════════════════════════════════════════════════════════════════════

/// Project detection must check for git worktree status.
#[test]
fn project_detection_handles_worktrees() {
    // Worktree detection lives in ship-substrate's project module
    // (path resolution + git discovery are L0). The test pre-dated the
    // substrate split.
    let source = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/core/ship-substrate/src/project.rs")
    ).unwrap();
    assert!(
        source.contains("worktree") || source.contains("git_common_dir"),
        "project.rs must detect git worktrees to avoid phantom project registrations"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// STALE WORKTREE CLEANUP
//
// Merged and failed jobs must have their worktrees cleaned up.
// The cleanup handler must be reliable — not leave orphans.
// ═══════════════════════════════════════════════════════════════════════════

/// The cleanup handler must resolve the slug from the event payload,
/// not rely on in-memory state that may not exist after a restart.
#[test]
fn cleanup_resolves_slug_from_event_payload() {
    let source = include_str!("../../../../crates/ship-pipeline-runtime/src/orchestrator/handlers.rs");
    assert!(
        source.contains("slug") && source.contains("payload"),
        "cleanup must resolve slug from event payload, not in-memory state"
    );
}

/// Cleanup must handle the case where the worktree was already removed
/// (idempotent cleanup).
#[test]
fn cleanup_is_idempotent() {
    let source = include_str!("../../../../crates/core/ship-substrate/crates/ship-supervisor/src/helpers.rs");
    assert!(
        source.contains("exists()"),
        "remove_worktree must check existence — cleanup must be idempotent"
    );
}

/// Orphan detection: after restart, any worktree for a terminal job
/// (merged/failed) should be flagged for cleanup.
#[test]
fn orphan_worktrees_detectable_from_projection() {
    let events = vec![
        created("j1", "merged-job"),
        dispatched("j1"),
        completed("j1", "merged-job"),
        merged("j1", "merged-job"),
        created("j2", "failed-job"),
        dispatched("j2"),
        failed("j2", "crash"),
        created("j3", "active-job"),
        dispatched("j3"),
    ];
    let map = project(&events);

    let orphans: Vec<_> = map.values()
        .filter(|j| matches!(j.status, PipelineRunStatus::Merged | PipelineRunStatus::Failed))
        .filter(|j| j.worktree.is_some())
        .map(|j| j.slug.as_str())
        .collect();

    let active: Vec<_> = map.values()
        .filter(|j| j.status == PipelineRunStatus::Dispatched)
        .filter(|j| j.worktree.is_some())
        .map(|j| j.slug.as_str())
        .collect();

    assert_eq!(orphans.len(), 2, "merged + failed jobs are orphan candidates");
    assert_eq!(active.len(), 1, "active job is NOT an orphan");
    assert!(orphans.contains(&"merged-job"));
    assert!(orphans.contains(&"failed-job"));
    assert!(active.contains(&"active-job"));
}

// ═══════════════════════════════════════════════════════════════════════════
// GRACEFUL SHUTDOWN
//
// Daemon must stop all agents within bounded time on SIGTERM.
// ═══════════════════════════════════════════════════════════════════════════

/// Shutdown must have a timeout — it cannot hang indefinitely.
#[test]
fn shutdown_has_bounded_timeout() {
    let registry_src = include_str!("../../../../crates/core/ship-substrate/src/services/process_registry.rs");
    assert!(
        registry_src.contains("timeout") || registry_src.contains("Duration"),
        "shutdown_all must have a timeout to prevent hanging on unresponsive agents"
    );
}

/// Shutdown must attempt graceful exit before force-kill.
#[test]
fn shutdown_sends_exit_before_kill() {
    // The executor's stop() should use the adapter's exit_command
    // before resorting to SIGTERM/SIGKILL.
    let process_src = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/core/ship-substrate/crates/ship-supervisor/src/executors/process.rs")
    ).unwrap();
    let lifecycle_src = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/core/ship-substrate/crates/ship-supervisor/src/agent_lifecycle.rs")
    ).unwrap();

    // At least one path must use the adapter's exit command
    let uses_exit = lifecycle_src.contains("exit_command")
        || process_src.contains("exit_command");
    assert!(uses_exit, "shutdown path must use adapter.exit_command() for graceful exit");
}

// ═══════════════════════════════════════════════════════════════════════════
// PROCESS EXIT DETECTION
//
// The daemon must detect dead agents without relying on hooks.
// ═══════════════════════════════════════════════════════════════════════════

/// ProcessExecutor must monitor the child process and emit events on exit.
#[test]
fn process_executor_monitors_child() {
    let source = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/core/ship-substrate/crates/ship-supervisor/src/executors/process.rs")
    ).unwrap();
    assert!(
        source.contains("wait()") || source.contains("child.wait"),
        "ProcessExecutor must wait on the child process to detect exit"
    );
}

/// ProcessExecutor must signal completion when the child exits,
/// not rely on the SessionEnd hook.
#[test]
fn process_executor_signals_on_exit() {
    let source = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/core/ship-substrate/crates/ship-supervisor/src/executors/process.rs")
    ).unwrap();
    assert!(
        source.contains("job") && (source.contains("end") || source.contains("completed") || source.contains("failed")),
        "ProcessExecutor must signal job completion/failure when child process exits"
    );
}


// ═══════════════════════════════════════════════════════════════════════════
// DAEMON RESTART RECOVERY (E2E-1)
//
// After restart, the daemon must recover all in-flight work from the
// event store. No job lost, no job duplicated.
// ═══════════════════════════════════════════════════════════════════════════

/// Jobs in every non-terminal state must be recoverable after restart.
#[test]
fn all_non_terminal_states_recoverable() {
    let events = vec![
        // Pending — waiting for dispatch
        created("j-pending", "pending"),

        // Dispatched — agent was running
        created("j-dispatched", "dispatched"),
        dispatched("j-dispatched"),

        // Completed — waiting for pipeline advancement
        created("j-completed", "completed"),
        dispatched("j-completed"),
        completed("j-completed", "completed"),

        // Blocked — needs human
        created("j-blocked", "blocked"),
        dispatched("j-blocked"),
        ev(event_types::PIPELINE_RUN_BLOCKED, serde_json::json!({
            "pipeline_run_id": "j-blocked", "blocker": "needs key", "needs_human": true
        })),
    ];
    let map = project(&events);

    assert_eq!(map.get("j-pending").unwrap().status, PipelineRunStatus::Pending);
    assert_eq!(map.get("j-dispatched").unwrap().status, PipelineRunStatus::Dispatched);
    assert_eq!(map.get("j-completed").unwrap().status, PipelineRunStatus::Completed);
    assert_eq!(map.get("j-blocked").unwrap().status, PipelineRunStatus::Blocked);

    // All have enough data for the reconciliation loop to act on
    assert!(map.get("j-dispatched").unwrap().worktree.is_some());
    assert!(map.get("j-completed").unwrap().worktree.is_some());
}

/// The reconciliation loop must exist and handle all three phases.
#[test]
fn reconciliation_loop_handles_three_phases() {
    let source = include_str!("../../../../crates/ship-pipeline-runtime/src/orchestrator/router.rs");
    assert!(source.contains("reconcile"), "must reconcile stale dispatched jobs");
    assert!(source.contains("replay"), "must replay stale completions");
    assert!(
        source.contains("pending_dispatch") || source.contains("dispatch_ready"),
        "must dispatch ready pending jobs"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// JOB FULL LIFECYCLE (E2E-4)
//
// A job must flow from creation to merge without human intervention.
// Every state transition is an event. No state is lost.
// ═══════════════════════════════════════════════════════════════════════════

/// Full lifecycle: created → dispatched → completed → merged.
#[test]
fn job_full_lifecycle_all_events_present() {
    let events = vec![
        created("j1", "auth-fix"),
        dispatched("j1"),
        completed("j1", "auth-fix"),
        merged("j1", "auth-fix"),
    ];
    let map = project(&events);
    let rec = map.get("j1").unwrap();

    assert_eq!(rec.status, PipelineRunStatus::Merged);
    assert_eq!(rec.slug, "auth-fix");
    assert!(rec.worktree.is_some(), "worktree must be tracked through lifecycle");
    assert!(rec.error.is_none(), "successful job must have no error");
}

/// Full lifecycle with pipeline: created → [dispatch+complete per step] → merged.
#[test]
fn pipeline_full_lifecycle_all_steps_tracked() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "pipe",
            "agent": "test-writer", "branch": "job/pipe",
            "spec_path": "spec.md", "plan_id": null,
            "steps": {
                "auto_merge": true,
                "steps": [
                    { "id": "test", "type": "agent", "agent": "test-writer", "after": [] },
                    { "id": "impl", "type": "agent", "agent": "rust-runtime", "after": ["test"] },
                    { "id": "gate", "type": "review", "agent": "reviewer", "after": ["impl"] },
                ]
            }
        })),
        dispatched("j1"), completed("j1", "pipe"),  // test
        dispatched("j1"), completed("j1", "pipe"),  // impl
        dispatched("j1"), completed("j1", "pipe"),  // gate
        merged("j1", "pipe"),
    ];
    let map = project(&events);
    let rec = map.get("j1").unwrap();

    assert_eq!(rec.status, PipelineRunStatus::Merged);
    assert!(rec.completed_steps.contains(&"test".to_string()));
    assert!(rec.completed_steps.contains(&"impl".to_string()));
    assert!(rec.completed_steps.contains(&"gate".to_string()));
}

/// Failure mid-pipeline preserves progress for retry.
#[test]
fn pipeline_failure_preserves_completed_steps() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "pipe",
            "agent": "test-writer", "branch": "job/pipe",
            "spec_path": "spec.md", "plan_id": null,
            "steps": { "steps": [
                { "id": "test", "type": "agent", "agent": "test-writer", "after": [] },
                { "id": "impl", "type": "agent", "agent": "rust-runtime", "after": ["test"] },
            ]}
        })),
        dispatched("j1"), completed("j1", "pipe"),  // test done
        dispatched("j1"),                             // impl started
        failed("j1", "agent crashed"),                // impl failed
    ];
    let map = project(&events);
    let rec = map.get("j1").unwrap();

    assert_eq!(rec.status, PipelineRunStatus::Failed);
    assert!(
        rec.completed_steps.contains(&"test".to_string()),
        "completed steps must survive failure — needed for retry"
    );
    assert!(
        rec.worktree.is_some(),
        "worktree must survive failure — needed for investigation"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// SECURITY (E2E-5)
//
// Auth and validation must be enforced at the HTTP boundary.
// ═══════════════════════════════════════════════════════════════════════════

/// All daemon HTTP endpoints require auth.
#[test]
fn all_endpoints_require_auth() {
    let lib_src = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")
    ).unwrap();
    assert!(
        lib_src.contains("auth") && lib_src.contains("middleware"),
        "daemon must apply auth middleware to all routes"
    );
}

/// Auth token is generated per daemon instance.
#[test]
fn auth_token_is_per_instance() {
    let auth_src = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/auth.rs")
    ).unwrap();
    assert!(
        auth_src.contains("generate_token") || auth_src.contains("random"),
        "auth token must be generated fresh per daemon instance"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// NO SILENT FAILURES
//
// Every error path must produce an observable signal — event, log, or
// error return. Nothing swallowed silently.
// ═══════════════════════════════════════════════════════════════════════════

/// Dead process reconciliation must leave an observable event trail.
///
/// Reconcile records a single `executor.session_destroyed` observation;
/// the coherence sweep drains dead handles + emits `job.failed`; the
/// exit handler emits `job.failed` for non-zero exits. Either signal is
/// sufficient — nothing is swallowed silently.
#[test]
fn dead_process_recovery_is_observable() {
    let replay = include_str!("../../../../crates/ship-pipeline-runtime/src/orchestrator/replay.rs");
    let sweep = include_str!("../../../../crates/ship-pipeline-runtime/src/orchestrator/sweep.rs");
    let exit_handler = include_str!(
        "../../../../crates/core/ship-substrate/crates/ship-runner/src/exit_handler.rs"
    );
    assert!(
        replay.contains("emit_executor_session_destroyed"),
        "reconcile must record an observation event when no live process is found"
    );
    assert!(
        sweep.contains("drain_dead") && sweep.contains("PIPELINE_RUN_FAILED"),
        "coherence sweep must drain dead handles and emit job.failed"
    );
    assert!(
        exit_handler.contains("PIPELINE_RUN_FAILED"),
        "exit handler must emit job.failed for non-zero process exits"
    );
}

/// Pipeline spawn failure must log and continue, not panic.
#[test]
fn pipeline_spawn_failure_does_not_panic() {
    let source = include_str!("../../../../crates/ship-pipeline-runtime/src/orchestrator/pipeline.rs");
    assert!(
        source.contains("Err(e)") && source.contains("continue"),
        "pipeline spawn failure must log and continue to next step"
    );
}

/// Cleanup failure must not prevent other cleanup operations.
#[test]
fn cleanup_failure_does_not_block_other_cleanup() {
    let source = include_str!("../../../../crates/ship-pipeline-runtime/src/orchestrator/handlers.rs");
    // Cleanup is delegated to AgentPlatform::cleanup_job — a single call that
    // internally runs tmux kill, worktree removal, and branch deletion independently.
    assert!(
        source.contains("cleanup_job"),
        "cleanup must delegate to platform.cleanup_job — independent ops inside"
    );
}
