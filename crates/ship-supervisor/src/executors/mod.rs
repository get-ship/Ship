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
//! `RoutingExecutor`    — dispatches each JobContext to Server, TmuxWindow,
//!                        or Process based on harness capabilities +
//!                        `ctx.headless`.

pub(crate) mod process;
pub(crate) mod routing;
pub(crate) mod server;
pub(crate) mod tmux_window;

pub(crate) use process::ProcessExecutor;
pub(crate) use routing::RoutingExecutor;
pub(crate) use server::ServerExecutor;
pub(crate) use tmux_window::TmuxWindowExecutor;

#[cfg(test)]
#[path = "tests_server.rs"]
mod tests_server;
