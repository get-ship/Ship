//! `ship-space` — SpaceService over substrate ports (L1).
//!
//! A space = the unit of isolation/coordination/config (one `.ship/` directory).
//! This crate owns three capabilities and nothing else (window management lives
//! in the ship-device topic — the carve split it out):
//!
//! - **registry** ([`registry`]) — list / active / add / activate. The
//!   `projects` table is Runtime-scoped (read daemon-wide, no space in hand).
//!   The registry emits `project.registered` / `project.seen` /
//!   `project.switched` through `KernelLog`; `ProjectsProjection` folds them
//!   into the `projects` SQL table (the SOLE record-read source post-dissolution
//!   — the `space_registry` DataStore table was retired in 2026-06-10 as part of
//!   the keystone-A dual-read-model dissolution). The [`projection`] module now
//!   only manages the `space_active` single-row pointer.
//! - **info** ([`info`]) — identity (id/root/name from `projects`). The per-space
//!   settings overlay moved to the unified `settings_overlay` (S4), read via
//!   `ship_config::overlay`; the legacy `space_overlay` writer/reader is gone.
//!
pub mod error;
pub mod event_registrations;
pub mod helpers;
pub mod info;
pub mod projection;
pub mod projects_projection;
pub mod registry;
pub mod service;
pub mod types;
pub mod wire;

pub use projects_projection::ProjectsProjection;
pub use wire::SpaceWire;
