//! OS-level agent supervision — the daemon-side companion to
//! `ship-platform`'s orchestrator. Owns the concrete things processes need
//! that platform crates can't (PTY allocation, tmux subprocess, git
//! worktree creation, terminal launching, harness-specific signal
//! handling, graceful shutdown).
//!
//! The orchestrator in the `ship-pipeline-runtime` crate (carved out of
//! `ship-platform`'s `services::orchestrator` per the P3 pipeline cut)
//! makes all dispatch / advancement decisions through a small set of
//! injected traits — `SessionExecutor`, `IsolationStrategy`, `AgentPlatform`,
//! `ProcessRegistryPort`. This crate provides the concrete
//! implementations the daemon uses to fulfill those traits.
//!
//! `apps/shipd` consumes this crate. Tests that exercise full-daemon
//! integration (with `ApiState`, `ShipRuntime`, etc.) live in shipd; the
//! tests inside this crate cover only the supervisor's own surface.

pub mod adapters;
pub mod agent_lifecycle;
pub mod executors;
pub mod helpers;
pub mod run_dispatch;
pub mod orphan_cleanup;
pub mod platform;
pub mod process_events;
pub mod terminal_launcher;

/// Re-export the platform's graceful_shutdown so callers can reach it
/// through the supervisor namespace — that's where the responsibility
/// for stopping spawned processes conceptually lives, even though the
/// implementation is in platform.
pub use ship_substrate::services::process_registry::graceful_shutdown;

/// Returns the substrate `ActorHandle` for an `ExecutorHandle` registered
/// by `ProcessExecutor` (PTY-backed). Tmux- and server-mode executors
/// return `None`. Used by the agent-session WebSocket attach route.
pub use executors::pty_actor_handle;

/// Test-only helper: wrap a substrate `ActorHandle` in an `ExecutorHandle`
/// shaped like `ProcessExecutor::spawn` would. Lets integration tests in
/// downstream crates seed `ProcessRegistry` with a PTY child without
/// driving the orchestrator end-to-end.
pub use executors::pty_executor_handle_for_test;
