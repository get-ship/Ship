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

pub mod ast;
pub mod builtin;
pub mod error;
pub mod helpers;
pub mod patch;
pub mod projection;
pub mod schema;
pub mod service;
pub mod types;
pub mod wire;

pub use wire::CollectionWire;
