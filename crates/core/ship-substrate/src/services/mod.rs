//! Facade over the platform-service-actor primitive + the ship-code-bound
//! dispatch types.
//!
//! `spawn_service`/`ServiceHandler`/`ServiceHandle`/`HealthStatus` are a
//! kernel/actor-layer primitive owned by [`ship_substrate_core::services`];
//! re-exported here so `ship_substrate::services::*` keeps resolving.
//!
//! `dispatch_ports` (LaunchCommand/LaunchProfile) and `server_session` are
//! ship-code's harness value types, stranded in L0 only because
//! `ship-runner::harness_adapter` imports them and must not depend on the L1
//! ship-code crate. They move to ship-code once that import cycle is broken.

pub mod dispatch_ports;
pub mod server_session;

pub use ship_substrate_core::services::{
    run_service, spawn_service, HealthStatus, ServiceHandle, ServiceHandler,
};
