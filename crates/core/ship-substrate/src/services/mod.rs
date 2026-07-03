//! Facade re-export of the platform-service-actor primitive, owned by
//! [`ship_substrate_core::services`].
//!
//! The harness value types that used to live here (`dispatch_ports`,
//! `server_session`) moved to the L1 `harness-adapter` contract leaf.

pub use ship_substrate_core::services::{
    run_service, spawn_service, HealthStatus, ServiceHandle, ServiceHandler,
};
