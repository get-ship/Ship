//! Ship Cloud sync transport and service actor.
//!
//! Push/pull cursors live in `ship_substrate::db::cursors` (the `cursors`
//! table in platform.db, alongside per-projection rebuild cursors).
//!
//! `client` + `types` are the thin HTTP transport layer. `sync_service`
//! (behind the `unstable` feature) is the `ServiceHandler` actor that drives
//! the push cycle and emits `sync.*` events into the kernel.
//!
//! Originally nested under `ship-platform/src/sync/`; carved into its own
//! crate so the cloud integration can evolve independently and so that
//! `ship-cloud` itself can be reused by other consumers (e.g. the gateway
//! mirror, third-party daemon integrations) without pulling in the full
//! `ship-platform` L1 surface.

pub mod client;
pub mod event_registrations;
pub mod types;
#[cfg(feature = "unstable")]
pub mod sync_service;

pub use client::SyncClient;
pub use types::{PullResponse, PushRequest, PushResponse};
#[cfg(feature = "unstable")]
pub use sync_service::{SyncConfig, SyncServiceHandler};
