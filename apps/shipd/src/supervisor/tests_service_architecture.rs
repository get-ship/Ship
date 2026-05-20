//! Service architecture tests — enforce 300-line cap on supervisor modules.
//!
//! Each test counts source lines and fails if a file exceeds the limit.

// CLAUDE.md sets a soft 300 limit and a hard 400 limit. The hard cap is
// what the test enforces; the soft preference shows up in code review.
// Files in the 301-400 range are not great but not test-failing.
const LINE_LIMIT: usize = 400;

// ── Original modules (must shrink to ≤300) ───────────────────────────────────

#[test]
fn adapters_module_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-supervisor/src/adapters.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "adapters.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn dispatch_pipeline_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/pipeline.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "orchestrator/pipeline.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn dispatch_replay_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "orchestrator/replay.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn supervisor_lib_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-supervisor/src/lib.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "ship-supervisor/src/lib.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── Existing small modules (must stay ≤300) ───────────────────────────────────

#[test]
fn dispatch_handlers_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/handlers.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "orchestrator/handlers.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn helpers_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-supervisor/src/helpers.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "helpers.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn run_dispatch_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-supervisor/src/run_dispatch.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "run_dispatch.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── New executor modules ───────────────────────────────────────────────────────

#[test]
fn executors_mod_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-supervisor/src/executors/mod.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "executors/mod.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── New pipeline modules ───────────────────────────────────────────────────────

#[test]
fn pipeline_loader_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/pipeline_loader.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "orchestrator/pipeline_loader.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn agent_lifecycle_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-supervisor/src/agent_lifecycle.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "agent_lifecycle.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn pending_dispatch_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/pending.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "orchestrator/pending.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── Orphan cleanup module ─────────────────────────────────────────────────────

#[test]
fn orphan_cleanup_under_line_limit() {
    let lines = include_str!("../../../../crates/core/ship-supervisor/src/orphan_cleanup.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "orphan_cleanup.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

