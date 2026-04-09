//! Concrete job executors for agent dispatch.
//!
//! `TmuxExecutor` — spawns agents in tmux sessions, polls for readiness.
//! `ProcessExecutor` — spawns agents as direct child processes (headless).

pub(crate) mod process;
pub(crate) mod run_step;
pub(crate) mod tmux;

pub(crate) use process::ProcessExecutor;
pub(crate) use tmux::TmuxExecutor;
