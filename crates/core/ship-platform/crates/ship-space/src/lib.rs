//! `ship-space` — SpaceService over substrate ports (L1).
//!
//! A space = THE unit of isolation/coordination/config. Identity is owned by
//! the space, never derived from a path. The runtime tracks location as
//! device-local state on the `spaces` Runtime-scoped projection.
//!
//! This crate owns three capabilities:
//!
//! - **registry** ([`spaces`]) — list / get / find_by_path. The `spaces`
//!   table is Runtime-scoped (read daemon-wide). [`identity`] mints identity
//!   and emits `space.opened` / `space.seen`; [`spaces_projection`] folds
//!   them into the `spaces` SQL table.
//! - **identity** ([`identity`]) — the SINGLE mint-once accessor
//!   (`ensure_identity_for_root`) every caller reaches. Identity's exact
//!   on-disk home hides here so a future fenced space-config session can
//!   relocate it.
//! - **info** — lives on the per-request `Service::info` (returns the
//!   served-space identity from the live `SpaceContext`).
//!
//! What died with the 2026-07-09 cutover: `Active`, the `active_id` field
//! on `SpaceList`, the single-row `space_active` pointer, and the path-mint
//! semantics (identity is owned by the space, never derived from the path).

pub mod error;
pub mod event_registrations;
pub mod helpers;
pub mod identity;
pub mod service;
pub mod spaces;
pub mod spaces_projection;
pub mod types;
pub mod wire;

pub use spaces_projection::SpacesProjection;
pub use wire::SpaceWire;
