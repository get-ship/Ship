//! `ship-artifacts` — ArtifactsService over substrate ports (L1).
//!
//! Content-addressed blob store + metadata projection. Two planes:
//! - metadata → `DataStore` via `SpaceContext.data()` + kernel events
//! - blob bytes → private `BlobBackend` (FS today; swap to substrate BlobStore later)
//!
//! No sqlite/sqlx/rusqlite types anywhere in this crate.

pub mod error;
pub mod gc;
pub(crate) mod backend;
pub(crate) mod helpers;
pub mod patch;
pub mod projection;
pub mod service;
pub mod types;
pub mod wire;

pub use wire::ArtifactsWire;
