//! Runner mechanism — moved from L0 ship-runner to L1 ship-code.
//!
//! Contains the orchestration helpers that depend on dispatch-port traits:
//! event dedup, router gate, exit-signal handler, spawn-env builder,
//! invocation-mode resolver, observe utilities, teardown helpers.
//!
//! The pure kernel-emit primitives (`persist_and_route`, `EventSink`,
//! `KernelEventSink`, process/pipeline/executor emitters) stay in L0
//! `ship_runner::emit` because `ship-code` depends on `ship-runner`.
//! The `HarnessAdapter` trait and its implementations also stay in
//! `ship-runner` / `ship-harnesses` for the same layer reason.

pub mod dedup;
pub mod exit_handler;
pub mod invocation;
pub mod observe;
pub mod router;
pub mod spawn_env;
pub mod teardown;

pub use dedup::{EventDedup, Seen};
pub use exit_handler::ExitVocabulary;
pub use invocation::resolve_step_invocation;
pub use observe::{resolve_head_sha, scan_agent_log_tail, scan_git_evidence, strip_ansi};
pub use router::{extract_slug, is_operator_event, terminal_guard, DISPATCH_GATE};
pub use teardown::{stop_by_slug, stop_handle};
pub use spawn_env::build_spawn_env;
