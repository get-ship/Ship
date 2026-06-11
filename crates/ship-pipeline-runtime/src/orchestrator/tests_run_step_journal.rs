//! Behavioral tests for the journaled run step (C4 / P2.2).
//!
//! The headline property: a run step whose journal is already complete
//! replays without re-executing its shell command — restart-safe shell steps.
//! Pinned via a marker file the command appends to: one line per real
//! execution, regardless of how many times `execute_run_step` is driven.

use durable_execution::{InMemoryJournalStore, JournalStore};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use super::run_step::execute_run_step;

fn orchestrator_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.orchestrator".to_string(),
        write_namespaces: vec!["pipeline.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec![],
        see_directed_events: false,
    }
}

fn test_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router
        .spawn_actor("service.orchestrator", orchestrator_config())
        .unwrap();
    Arc::new(Mutex::new(router))
}

fn marker_lines(dir: &TempDir) -> usize {
    std::fs::read_to_string(dir.path().join("marker.txt"))
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// A completed run step must NOT re-execute its command when driven again
/// against the same journal store (restart / re-dispatch replay), and the
/// journal must hold exactly the program's three completed ops.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn completed_run_step_replays_without_reexecuting() {
    let work_dir = TempDir::new().unwrap();
    let store: Arc<dyn JournalStore> = Arc::new(InMemoryJournalStore::new());
    let kernel = test_kernel();
    let run_id = "j-runstep-replay";

    execute_run_step(
        run_id, "runstep-replay", "build", "echo ran >> marker.txt",
        work_dir.path(), &kernel, None, &store,
    )
    .await;
    assert_eq!(marker_lines(&work_dir), 1, "first drive executes the command");

    let journal = store
        .load("j-runstep-replay/step/build")
        .await
        .expect("store load ok")
        .expect("journal must be saved under <run_id>/step/<step_id>");
    let entries = journal.entries();
    assert_eq!(
        entries.len(),
        3,
        "RunStepProgram produces exactly three ops; got {:?}",
        entries.iter().map(|e| &e.op).collect::<Vec<_>>(),
    );
    assert_eq!(entries[0].op, "run_step.emit_started");
    assert_eq!(entries[1].op, "run_step.execute");
    assert_eq!(entries[2].op, "run_step.emit_terminal");
    assert!(!journal.is_suspended());

    // Second drive — the journal is complete, so every op replays its logged
    // result. The command must not run again (and, by the same mechanism, the
    // step events are not re-emitted: emit ops replay too).
    execute_run_step(
        run_id, "runstep-replay", "build", "echo ran >> marker.txt",
        work_dir.path(), &kernel, None, &store,
    )
    .await;
    assert_eq!(
        marker_lines(&work_dir),
        1,
        "replay must NOT re-execute the shell command"
    );
}

/// A failed command is a journaled terminal outcome, not a retry candidate:
/// driving the step again replays the failure without re-running the command.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_run_step_is_journaled_and_not_rerun() {
    let work_dir = TempDir::new().unwrap();
    let store: Arc<dyn JournalStore> = Arc::new(InMemoryJournalStore::new());
    let kernel = test_kernel();
    let run_id = "j-runstep-fail";

    execute_run_step(
        run_id, "runstep-fail", "lint", "echo boom >> marker.txt; exit 3",
        work_dir.path(), &kernel, None, &store,
    )
    .await;
    assert_eq!(marker_lines(&work_dir), 1);

    let journal = store
        .load("j-runstep-fail/step/lint")
        .await
        .unwrap()
        .expect("failure path must still save a complete journal");
    assert_eq!(journal.entries().len(), 3, "terminal emission journaled on failure too");
    assert!(!journal.is_suspended());

    execute_run_step(
        run_id, "runstep-fail", "lint", "echo boom >> marker.txt; exit 3",
        work_dir.path(), &kernel, None, &store,
    )
    .await;
    assert_eq!(
        marker_lines(&work_dir),
        1,
        "a journaled failure must not re-run the command"
    );
}

/// Per-step journal keys are disjoint: two steps of the same run journal
/// independently and neither collides with the run's DispatchProgram journal
/// (which lives under the bare pipeline_run_id).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step_journals_are_keyed_per_step() {
    let work_dir = TempDir::new().unwrap();
    let store: Arc<dyn JournalStore> = Arc::new(InMemoryJournalStore::new());
    let kernel = test_kernel();
    let run_id = "j-runstep-keys";

    execute_run_step(
        run_id, "runstep-keys", "one", "true",
        work_dir.path(), &kernel, None, &store,
    )
    .await;
    execute_run_step(
        run_id, "runstep-keys", "two", "true",
        work_dir.path(), &kernel, None, &store,
    )
    .await;

    assert!(store.load("j-runstep-keys/step/one").await.unwrap().is_some());
    assert!(store.load("j-runstep-keys/step/two").await.unwrap().is_some());
    assert!(
        store.load("j-runstep-keys").await.unwrap().is_none(),
        "the bare run-id key belongs to DispatchProgram and must stay untouched"
    );
}
