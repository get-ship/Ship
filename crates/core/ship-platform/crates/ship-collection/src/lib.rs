//! `ship-collection` — CollectionService over substrate ports (L1).
//!
//! Schema registry + thin RPC facade. The substrate does the heavy lifting;
//! this crate validates against schema and delegates:
//! - reads/writes → `DataStore` via `SpaceContext.data()`
//! - events → `KernelLog` via `SpaceContext.kernel().emit(...)`
//! - live → `ReactiveRegistry` (snapshot-diff resolver)
//!
//! No sqlite/sqlx/rusqlite types anywhere in this crate. All storage/events
//! go through the port types from ship-substrate.
//!
//! Event payloads and registrations live here because the COLLECTION domain
//! owns its events at L1 (keystone-A de-bucket, 2026-06-10).

pub mod builtin;
pub mod error;
pub mod event_registrations;
pub mod events;
pub mod helpers;
pub mod item_projection;
pub mod patch;
pub mod projection;
pub mod schema;
pub mod schema_def;
pub mod service;
pub mod types;
pub mod wire;

pub use schema_def::{CollectionDef, CollectionFieldDef, CollectionFieldType};
pub use wire::CollectionWire;
