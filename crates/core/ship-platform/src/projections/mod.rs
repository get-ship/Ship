//! Domain projections — read models derived from the event log.
//!
//! Each module here owns a slice of platform-level state (auth, projects,
//! jobs, agent compile state). The projection *framework* (the
//! `Projection` trait, `EventBus`, `AsyncProjection`) lives in
//! `ship_substrate::projections`; this module hosts the implementations
//! that encode platform domain facts.

pub mod agent_session;
pub mod auth;
pub mod compile_state;
pub mod pipeline_run;
pub mod job_transitions;
pub mod projects;

pub use agent_session::{
    AgentSession, AgentSessionProjection, AgentSessionStatus,
    load_session_by_id, load_session_by_id_at, load_session_by_slug,
    load_session_by_slug_at, load_sessions, load_sessions_at,
    load_sessions_by_parent, project as project_agent_sessions,
};
pub use auth::{
    AuthSessionRow, AuthSessionsProjection, DaemonOwnerProjection, DaemonOwnerRow,
};
pub use compile_state::{CompileStateProjection, ProjectCompileState};
pub use pipeline_run::{
    PipelineRunProjection, PipelineRun, PipelineRunStatus, load_pipeline_run_by_id, load_pipeline_run_by_id_at, load_pipeline_run_by_slug,
    load_pipeline_run_by_slug_at, load_pipeline_runs, project as project_pipeline_runs,
};
pub use projects::{ProjectRow, ProjectsProjection};
