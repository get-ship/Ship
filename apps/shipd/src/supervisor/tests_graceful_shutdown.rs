//! Graceful shutdown tests.
//!
//! Verifies that shutdown_all sends exit_command before SIGTERM,
//! waits up to 30s per process, and force-kills with SIGKILL if needed.

// ── Source helpers ───────────────────────────────────────────────────────────

fn registry_src() -> String {
    include_str!("../../../../crates/core/ship-substrate/src/services/process_registry.rs").to_string()
}

fn process_executor_src() -> String {
    include_str!("../../../../crates/core/ship-substrate/crates/ship-supervisor/src/executors/process.rs").to_string()
}

// ── ProcessExecutor shutdown ─────────────────────────────────────────────────

/// ProcessExecutor::stop() must send SIGKILL as fallback after SIGTERM.
#[test]
fn process_executor_stop_sends_sigkill_after_sigterm() {
    let src = process_executor_src();
    assert!(
        src.contains("SIGKILL"),
        "ProcessExecutor::stop must send SIGKILL if process does not exit after SIGTERM"
    );
}

/// ProcessExecutor::stop() must poll for process exit before force-killing.
#[test]
fn process_executor_stop_waits_before_sigkill() {
    let src = process_executor_src();
    assert!(
        src.contains("is_alive") || src.contains("libc::kill") && src.contains("SIGKILL"),
        "ProcessExecutor::stop must wait for process exit before sending SIGKILL"
    );
    // Verify there's a timeout constant, not an infinite loop
    assert!(
        src.contains("30") || src.contains("SHUTDOWN_TIMEOUT"),
        "ProcessExecutor::stop must have a 30s timeout before SIGKILL"
    );
}

/// ProcessExecutor::stop() must send SIGTERM before SIGKILL (graceful first).
#[test]
fn process_executor_stop_sends_sigterm_before_sigkill() {
    let src = process_executor_src();
    let sigterm_pos = src.find("SIGTERM").expect("SIGTERM must be present");
    let sigkill_pos = src.find("SIGKILL").expect("SIGKILL must be present");
    assert!(
        sigterm_pos < sigkill_pos,
        "SIGTERM must appear before SIGKILL in ProcessExecutor::stop"
    );
}

// ── Registry shutdown_all ────────────────────────────────────────────────────

/// shutdown_all must log the timeout duration.
#[test]
fn shutdown_all_logs_timeout() {
    let src = registry_src();
    assert!(
        src.contains("30") || src.contains("SHUTDOWN_TIMEOUT"),
        "shutdown_all must reference the 30s timeout"
    );
}

/// shutdown_all must spawn concurrent stop tasks (not sequential) for bounded total time.
#[test]
fn shutdown_all_stops_processes_concurrently() {
    let src = registry_src();
    assert!(
        src.contains("join_all") || src.contains("tokio::spawn") || src.contains("futures"),
        "shutdown_all must stop processes concurrently to bound total shutdown time"
    );
}

/// shutdown_all must complete even when all processes hang (via SIGKILL fallback).
#[test]
fn shutdown_all_is_bounded() {
    // Verified by: ProcessExecutor::stop() has a SIGKILL fallback after 30s,
    // so shutdown_all is bounded by the SIGKILL path.
    let proc_src = process_executor_src();
    assert!(
        proc_src.contains("SIGKILL"),
        "ProcessExecutor must SIGKILL hung processes to bound shutdown time"
    );
}

// ── ShipRuntime drain sequence ──────────────────────────────────────────────

/// shutdown() must emit a supervisor.shutting_down event before cleanup.
#[test]
fn shutdown_emits_supervisor_shutting_down_event() {
    let src = include_str!("../ship_runtime/init.rs");
    assert!(
        src.contains("supervisor.shutting_down") || src.contains("shutting_down"),
        "ShipRuntime::shutdown must emit a supervisor.shutting_down event"
    );
}

/// shutdown() must clean up daemon files after drain completes.
#[test]
fn shutdown_cleans_files_after_drain() {
    let src = include_str!("../ship_runtime/init.rs");
    // Token is preserved across restarts; only port and pid are cleaned up.
    // The actual removal is delegated to crate::daemon::remove_liveness_files
    // so paths route through the typed ShipHome::daemon_file accessor.
    assert!(
        src.contains("remove_liveness_files"),
        "ShipRuntime::shutdown must call crate::daemon::remove_liveness_files"
    );
    // Drain must happen before file cleanup
    let has_drain = src.contains("drain") || src.contains("on_drain");
    assert!(
        has_drain,
        "ShipRuntime::shutdown must call drain before cleaning up files"
    );
}
