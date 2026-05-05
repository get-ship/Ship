//! Projection system — derived state from the event log.
//!
//! A [`Projection`] declares which event types it handles and how to apply them
//! to a read model (SQL table). The [`EventBus`] dispatches events to registered
//! projections after every append.
//!
//! Projections are the ONLY mutable state in the system. They can be truncated
//! and rebuilt from the event log at any time via [`rebuild`].

pub mod async_projection;
pub mod compile_state;
pub mod job;
pub mod job_transitions;
pub mod process;
pub mod projects;
pub mod spawn;
pub mod actor;
mod registry;
mod session;

#[cfg(test)]
mod tests_actor;
#[cfg(test)]
mod tests_daemon_events;
#[cfg(test)]
mod tests_session;

pub use actor::ActorProjection;
pub use async_projection::{AsyncProjection, spawn_projection};
pub use compile_state::{CompileStateProjection, ProjectCompileState};
pub use projects::{ProjectRow, ProjectsProjection};
pub use job::{JobRecord, JobStatus, load_jobs, project as project_jobs};
pub use process::{ProcessRecord, ProcessStatus, load_processes};
pub use registry::{EventBus, Projection};
pub use session::SessionProjection;
pub use spawn::{ProjectionHandle, spawn_with_failure_counter};
