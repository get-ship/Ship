//! Concrete job executors for agent dispatch.
//!
//! `TmuxWindowExecutor` — runs pipeline steps as tmux windows inside a
//!                        shared per-slug session (default for `ship run`;
//!                        users attach with `tmux attach`). Not a PTY.
//! `ProcessExecutor`    — spawns agents as direct child processes under a
//!                        real PTY (headless fallback used when tmux is
//!                        unavailable or the caller opts in via
//!                        `--headless`).
//! `ServerExecutor`     — delegates to a provider's long-lived server-mode
//!                        session (codex app-server / opencode serve).
//!                        Picked when the provider advertises `server_mode`
//!                        AND a session is bound in the registry.
//! `TmuxExecutor`       — legacy single-session interactive executor for
//!                        workspace agents. Retained for back-compat.
//! `RoutingExecutor`    — dispatches each JobContext to Server, TmuxWindow,
//!                        or Process based on provider capabilities +
//!                        `ctx.headless`.

pub(crate) mod process;
pub(crate) mod routing;
pub(crate) mod server;
pub(crate) mod tmux;
pub(crate) mod tmux_window;

pub(crate) use process::ProcessExecutor;
pub(crate) use routing::RoutingExecutor;
pub(crate) use server::ServerExecutor;
pub(crate) use tmux::TmuxExecutor;
pub(crate) use tmux_window::TmuxWindowExecutor;
