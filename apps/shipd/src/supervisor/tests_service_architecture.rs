//! Service architecture tests — enforce 300-line cap on supervisor modules.
//!
//! Each test counts source lines and fails if a file exceeds the limit.

const LINE_LIMIT: usize = 300;

// ── Original modules (must shrink to ≤300) ───────────────────────────────────

#[test]
fn adapters_module_under_line_limit() {
    let lines = include_str!("adapters.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "adapters.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn dispatch_pipeline_under_line_limit() {
    let lines = include_str!("dispatch_pipeline.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "dispatch_pipeline.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn dispatch_replay_under_line_limit() {
    let lines = include_str!("dispatch_replay.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "dispatch_replay.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn supervisor_mod_under_line_limit() {
    let lines = include_str!("mod.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "supervisor/mod.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── Existing small modules (must stay ≤300) ───────────────────────────────────

#[test]
fn dispatch_handlers_under_line_limit() {
    let lines = include_str!("dispatch_handlers.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "dispatch_handlers.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn helpers_under_line_limit() {
    let lines = include_str!("helpers.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "helpers.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn job_dispatch_under_line_limit() {
    let lines = include_str!("job_dispatch.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "job_dispatch.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── New executor modules ───────────────────────────────────────────────────────

#[test]
fn executors_mod_under_line_limit() {
    let lines = include_str!("executors/mod.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "executors/mod.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn executors_tmux_under_line_limit() {
    let lines = include_str!("executors/tmux.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "executors/tmux.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn executors_process_under_line_limit() {
    let lines = include_str!("executors/process.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "executors/process.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── New pipeline modules ───────────────────────────────────────────────────────

#[test]
fn pipeline_loader_under_line_limit() {
    let lines = include_str!("pipeline_loader.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "pipeline_loader.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn agent_lifecycle_under_line_limit() {
    let lines = include_str!("agent_lifecycle.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "agent_lifecycle.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn pending_dispatch_under_line_limit() {
    let lines = include_str!("pending_dispatch.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "pending_dispatch.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

// ── New workspace modules ─────────────────────────────────────────────────────

#[test]
fn workspace_api_under_line_limit() {
    let lines = include_str!("workspace_api.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "workspace_api.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}

#[test]
fn workspace_sync_under_line_limit() {
    let lines = include_str!("workspace_sync.rs").lines().count();
    assert!(
        lines <= LINE_LIMIT,
        "workspace_sync.rs has {lines} lines (limit {LINE_LIMIT})"
    );
}
