//! Concrete job executors for agent dispatch.
//!
//! `TmuxWindowExecutor` — runs pipeline steps as tmux windows inside a
//!                        shared per-slug session (default for `ship run`;
//!                        users attach with `tmux attach`). Not a PTY.
//! `ProcessExecutor`    — spawns agents as direct child processes under a
//!                        real PTY (headless fallback used when tmux is
//!                        unavailable or the caller opts in via
//!                        `--headless`).
//! `ServerExecutor`     — delegates to a harness's long-lived server-mode
//!                        session (codex app-server / opencode serve).
//!                        Picked when the harness advertises `server_mode`
//!                        AND a session is bound in the registry.
//! `RoutingExecutor`    — dispatches each SessionContext to Server, TmuxWindow,
//!                        or Process based on harness capabilities +
//!                        `ctx.headless`.

pub(crate) mod process;
pub(crate) mod routing;
pub(crate) mod server;
pub(crate) mod synth_emit;
pub(crate) mod tmux_window;

pub(crate) use process::ProcessExecutor;
pub use routing::RoutingExecutor;
pub(crate) use server::ServerExecutor;
pub(crate) use tmux_window::TmuxWindowExecutor;

/// Returns the substrate `ProcessHandle` for a registered ExecutorHandle when
/// the underlying executor is PTY-backed (`ProcessExecutor`). Tmux- and
/// server-mode executors don't carry a substrate handle — they expose their
/// child via tmux or an external session — so this returns `None` for them,
/// which callers (e.g. the agent-session WebSocket attach route) treat as
/// "not PTY-attachable" and respond with `409 Conflict`.
pub fn pty_actor_handle(
    eh: &crate::dispatch_ports::ExecutorHandle,
) -> Option<ship_substrate::supervisor::ProcessHandle> {
    process::ProcessState::actor_handle_from(&eh.inner)
}

/// Serializes tests that fork real OS processes — tmux windows, PTY children,
/// `tmux -V` probes. They contend on fork/PTY resources, and a concurrent
/// burst triggers transient fork failures (see `tmux_window::tmux_available`,
/// which already retries once for the same reason). `cargo test` runs tests in
/// parallel by default, so any such test must hold this guard for its duration
/// to keep the suite deterministic. Poison-tolerant: a panicking test must not
/// wedge the rest.
#[cfg(test)]
pub(crate) fn real_spawn_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Build a PTY-backed `ExecutorHandle` directly from a substrate
/// `ProcessHandle`. Used by tests that want to register a PTY child into the
/// `ProcessRegistry` without going through the full ProcessExecutor spawn
/// path. The resulting handle round-trips through `pty_actor_handle`.
pub fn pty_executor_handle_for_test(
    slug: &str,
    actor: ship_substrate::supervisor::ProcessHandle,
) -> crate::dispatch_ports::ExecutorHandle {
    process::ProcessState::executor_handle_for_test(slug, actor)
}

#[cfg(test)]
#[path = "tests_server.rs"]
mod tests_server;

#[cfg(test)]
#[path = "mint_token_tests.rs"]
mod mint_token_tests;
