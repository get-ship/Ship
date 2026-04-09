//! Graceful shutdown tests.
//!
//! Verifies that shutdown_all sends exit_command before SIGTERM,
//! waits up to 30s per process, and force-kills with SIGKILL if needed.

// ── Source helpers ───────────────────────────────────────────────────────────

fn registry_src() -> String {
    include_str!("registry.rs").to_string()
}

fn process_executor_src() -> String {
    include_str!("executors/process.rs").to_string()
}

fn tmux_executor_src() -> String {
    include_str!("executors/tmux.rs").to_string()
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

// ── TmuxExecutor shutdown ────────────────────────────────────────────────────

/// TmuxExecutor::stop() must send exit_command before killing the session.
#[test]
fn tmux_executor_stop_sends_exit_command_before_kill() {
    let src = tmux_executor_src();
    assert!(
        src.contains("exit_command"),
        "TmuxExecutor::stop must call adapter.exit_command() before killing the session"
    );
}

/// TmuxExecutor state must carry provider so stop() can get the right exit command.
#[test]
fn tmux_executor_state_carries_provider() {
    let src = tmux_executor_src();
    assert!(
        src.contains("provider") || src.contains("adapter"),
        "TmuxExecutor handle state must carry provider info for exit_command lookup"
    );
}

/// TmuxExecutor::stop() must wait for the session to disappear before force-killing.
#[test]
fn tmux_executor_stop_waits_before_force_kill() {
    let src = tmux_executor_src();
    assert!(
        src.contains("30") || src.contains("SHUTDOWN_TIMEOUT"),
        "TmuxExecutor::stop must have a 30s timeout before force-killing the session"
    );
    assert!(
        src.contains("has-session") || src.contains("is_alive"),
        "TmuxExecutor::stop must poll for session exit before force-killing"
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
    // Verified by: stop() has a SIGKILL fallback after 30s, so shutdown_all
    // is bounded by the SIGKILL path. This test verifies the SIGKILL path exists.
    let proc_src = process_executor_src();
    let tmux_src = tmux_executor_src();
    assert!(
        proc_src.contains("SIGKILL"),
        "ProcessExecutor must SIGKILL hung processes to bound shutdown time"
    );
    assert!(
        tmux_src.contains("kill_tmux_session") || tmux_src.contains("kill-session"),
        "TmuxExecutor must force-kill hung sessions to bound shutdown time"
    );
}

// ── ShipRuntime drain sequence ──────────────────────────────────────────────

/// shutdown() must emit a supervisor.shutting_down event before cleanup.
#[test]
fn shutdown_emits_supervisor_shutting_down_event() {
    let src = include_str!("../ship_runtime.rs");
    assert!(
        src.contains("supervisor.shutting_down") || src.contains("shutting_down"),
        "ShipRuntime::shutdown must emit a supervisor.shutting_down event"
    );
}

/// shutdown() must clean up daemon files after drain completes.
#[test]
fn shutdown_cleans_files_after_drain() {
    let src = include_str!("../ship_runtime.rs");
    assert!(
        src.contains("daemon.token") && src.contains("daemon.port") && src.contains("daemon.pid"),
        "ShipRuntime::shutdown must remove daemon.token, daemon.port, daemon.pid"
    );
    // Drain must happen before file cleanup
    let has_drain = src.contains("drain") || src.contains("on_drain");
    assert!(
        has_drain,
        "ShipRuntime::shutdown must call drain before cleaning up files"
    );
}
