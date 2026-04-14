//! Concrete job executors for agent dispatch.
//!
//! `PtyExecutor`     — runs pipeline steps in tmux PTY windows (default
//!                     for `ship run`; users attach with `tmux attach`).
//! `ProcessExecutor` — spawns agents as direct child processes (headless
//!                     fallback used when tmux is unavailable or the
//!                     caller opts in via `--headless`).
//! `TmuxExecutor`    — legacy single-session interactive executor for
//!                     workspace agents. Retained for back-compat.
//! `RoutingExecutor` — dispatches each JobContext to Pty or Process
//!                     based on `ctx.headless`.

pub(crate) mod process;
pub(crate) mod pty;
pub(crate) mod routing;
pub(crate) mod tmux;

pub(crate) use process::ProcessExecutor;
pub(crate) use pty::PtyExecutor;
pub(crate) use routing::RoutingExecutor;
pub(crate) use tmux::TmuxExecutor;
