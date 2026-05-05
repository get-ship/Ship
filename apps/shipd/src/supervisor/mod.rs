//! Supervisor — process management for shipd.
//!
//! After the orchestrator extraction, this module contains ONLY platform-specific
//! process management: executors, liveness monitoring, orphan cleanup, process
//! registry, terminal launching, and the platform trait implementations.
//!
//! Job orchestration (pipeline advancement, step dispatch, event routing) lives
//! in `ship_platform::services::orchestrator`.

pub mod adapters;
pub(crate) mod agent_lifecycle;
pub(crate) mod executors;
pub mod helpers;
pub mod job_dispatch;
pub(crate) mod liveness;
pub(crate) mod process_events;
pub(crate) mod orphan_cleanup;
pub(crate) mod platform;
pub(crate) mod registry;
pub mod terminal_launcher;

pub(crate) use registry::graceful_shutdown;

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

#[cfg(test)]
#[path = "tests_executor_events.rs"]
mod tests_executor_events;

#[cfg(test)]
#[path = "tests_pipeline_e2e.rs"]
mod tests_pipeline_e2e;

#[cfg(test)]
#[path = "tests_pipeline_vars.rs"]
mod tests_pipeline_vars;

#[cfg(test)]
#[path = "tests_actor_registration.rs"]
mod tests_actor_registration;

#[cfg(test)]
#[path = "tests_step_tracking.rs"]
mod tests_step_tracking;

#[cfg(test)]
#[path = "tests_pipeline_reliability.rs"]
mod tests_pipeline_reliability;
