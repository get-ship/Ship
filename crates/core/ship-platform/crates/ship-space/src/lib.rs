//! `ship-space` — SpaceService over substrate ports (L1).
//!
//! A space = the unit of isolation/coordination/config (one `.ship/` directory).
//! This crate owns three capabilities and nothing else (window management lives
//! in the ship-device topic — the carve split it out):
//!
//! - **registry** ([`registry`]) — list / active / add / activate. The
//!   `projects` table is Runtime-scoped (read daemon-wide, no space in hand), so
//!   the registry reads through a Runtime-bound `DataStore` and emits
//!   `project.registered` / `project.seen` / `project.switched` through
//!   `KernelLog` (the kernel routes them to runtime.db). The crate's
//!   [`projection::SpaceProjection`] materializes the `projects` table + the
//!   `active_space` pointer in the same append transaction.
//! - **config** ([`config`]) — the per-space settings overlay. Reads the
//!   `space_settings` table (Space-scoped, per-space `space.db`); writes emit
//!   `space.setting.changed` (the space rides the envelope, not the payload).
//! - **info** ([`info`]) — identity (id/root/name from `projects`) + the
//!   effective-settings overlay (global ⟵ space). The daemon supplies its global
//!   settings snapshot; the overlay is here.
//!
//! No sqlite/sqlx/rusqlite types and no `std::fs`/`tokio::fs` anywhere in this
//! crate. All storage/events go through the port types from ship-substrate.

pub mod config;
pub mod error;
pub mod helpers;
pub mod info;
pub mod projection;
pub mod registry;
pub mod service;
pub mod types;
pub mod wire;

pub use wire::SpaceWire;
