//! Daemon lifecycle: command dispatch, supervisor, status, repair, cleanup.

pub mod pidfile;
pub mod service_registration;
pub mod stop;
pub mod supervision;
pub mod supervision_pidfile;
pub mod supervision_systemd;

mod cleanup;
mod commands;
mod ensure;
mod legacy_pidfile;
mod paths;
mod pid_io;
mod repair;
mod status;
mod supervise;

#[cfg(test)]
#[path = "supervision_tests.rs"]
mod supervision_tests;

#[cfg(test)]
mod tests;

pub use commands::{run_daemon, DaemonCommand};
pub use ensure::{daemon_info_if_healthy, ensure_daemon};
pub use pid_io::{read_pid, write_pid};
pub use status::{format_relative_time, format_uptime};
pub(crate) use legacy_pidfile::{daemon_start_inner, daemon_stop_inner};
