//! Supervisor — workspace lifecycle management and job dispatch for shipd.

pub mod adapters;
pub(crate) mod agent_lifecycle;
mod dispatch_handlers;
pub(crate) mod dispatch_pipeline;
mod dispatch_replay;
pub(crate) mod orphan_cleanup;
pub(crate) mod executors;
pub mod helpers;
pub mod job_dispatch;
pub(crate) mod liveness;
pub(crate) mod pending_dispatch;
pub(crate) mod pipeline_loader;
pub(crate) mod registry;
pub mod terminal_launcher;
mod workspace_api;
mod workspace_sync;

pub(crate) use registry::graceful_shutdown;
pub use workspace_api::start_workspace;
pub use workspace_sync::subscribe_workspace_events;

#[cfg(test)]
mod tests_service_architecture;

#[cfg(test)]
#[path = "tests_executor.rs"]
mod tests_executor;

#[cfg(test)]
#[path = "tests_cleanup_lifecycle.rs"]
mod tests_cleanup_lifecycle;

#[cfg(test)]
#[path = "tests_concurrency.rs"]
mod tests_concurrency;

#[cfg(test)]
#[path = "tests_daemon_startup.rs"]
mod tests_daemon_startup;

#[cfg(test)]
#[path = "tests_orphan_cleanup.rs"]
mod tests_orphan_cleanup;

#[cfg(test)]
#[path = "tests_event_integrity.rs"]
mod tests_event_integrity;

#[cfg(test)]
#[path = "tests_e2e_reliability.rs"]
mod tests_e2e_reliability;

#[cfg(test)]
#[path = "tests_graceful_shutdown.rs"]
mod tests_graceful_shutdown;

#[cfg(test)]
#[path = "tests_process_exit.rs"]
mod tests_process_exit;
