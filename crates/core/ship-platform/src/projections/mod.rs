//! Domain projections — read models derived from the event log.
//!
//! Each module here owns a slice of platform-level state (auth, projects,
//! jobs, agent compile state). The projection *framework* (the
//! `Projection` trait, `EventBus`, `AsyncProjection`) lives in
//! `ship_substrate::projections`; this module hosts the implementations
//! that encode platform domain facts.

pub mod auth;
pub mod compile_state;
pub mod job;
pub mod job_transitions;
pub mod projects;

pub use auth::{
    AuthSessionRow, AuthSessionsProjection, DaemonOwnerProjection, DaemonOwnerRow,
};
pub use compile_state::{CompileStateProjection, ProjectCompileState};
pub use job::{
    JobProjection, JobRecord, JobStatus, load_job_by_id, load_job_by_id_at, load_job_by_slug,
    load_job_by_slug_at, load_jobs, project as project_jobs,
};
pub use projects::{ProjectRow, ProjectsProjection};
