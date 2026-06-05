//! Ship Cloud sync transport — HTTP push/pull of elevated events to the cloud gateway.
//!
//! Push/pull cursors live in `ship_substrate::db::cursors` (the `cursors`
//! table in platform.db, alongside per-projection rebuild cursors).
//!
//! This module is a thin HTTP transport layer — `SyncClient` + 4 request/
//! response types. It emits 0 events directly (events are emitted by the
//! wrapping service actor that lives in `ship-platform`'s
//! `services::sync::SyncServiceHandler`).
//!
//! Originally nested under `ship-platform/src/sync/`; carved into its own
//! crate so the cloud integration can evolve independently and so that
//! `ship-cloud` itself can be reused by other consumers (e.g. the gateway
//! mirror, third-party daemon integrations) without pulling in the full
//! `ship-platform` L1 surface.

pub mod client;
pub mod types;

pub use client::SyncClient;
pub use types::{PullResponse, PushRequest, PushResponse};
