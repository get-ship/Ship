//! Concrete job executors for agent dispatch.
//!
//! `TmuxWindowExecutor` — runs pipeline steps as tmux windows inside a
//!                        shared per-slug session (default for `ship run`;
//!                        users attach with `tmux attach`). Not a PTY.
//! `ProcessExecutor`    — spawns agents as direct child processes under a
//!                        real PTY (headless fallback used when tmux is
//!                        unavailable or the caller opts in via
//!                        `--headless`).
//! `TmuxExecutor`       — legacy single-session interactive executor for
//!                        workspace agents. Retained for back-compat.
//! `RoutingExecutor`    — dispatches each JobContext to TmuxWindow or
//!                        Process based on `ctx.headless`.

pub(crate) mod process;
pub(crate) mod routing;
pub(crate) mod tmux;
pub(crate) mod tmux_window;

pub(crate) use process::ProcessExecutor;
pub(crate) use routing::RoutingExecutor;
pub(crate) use tmux::TmuxExecutor;
pub(crate) use tmux_window::TmuxWindowExecutor;
