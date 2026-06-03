//! `ship-config` — ConfigService over substrate ports (L1).
//!
//! The `config` domain has three tiers:
//!
//! - **settings** (KV): the `daemon_settings` table in `runtime.db`, accessed
//!   through the substrate-owned `ship_substrate::db::daemon_settings` port.
//!   Event-sourced via `config.changed`. This is the proto contract's surface
//!   (List/Get/Set/Watch). The well-known defaults + precedence resolvers live
//!   in [`settings`].
//! - **file-tier** (flags-as-files): the declarative `.ship/agents/` projection
//!   and the per-domain `.ship/<domain>/` change watcher. ALL disk goes through
//!   the [`ship_substrate::ports::FsBroker`] gated door — ZERO `std::fs`/
//!   `tokio::fs` in this crate. See [`agents`] and [`watcher`].
//! - **events**: emitted via `SpaceContext.kernel().emit(...)` ([`KernelLog`]).
//!
//! No sqlite/sqlx/rusqlite types and no raw filesystem calls anywhere in this
//! crate. Storage + events go through the port types from `ship-substrate`.
//!
//! [`KernelLog`]: ship_substrate::ports::KernelLog

pub mod agents;
pub mod error;
pub mod patch;
pub mod service;
pub mod settings;
pub mod types;
pub mod watcher;
pub mod wire;

pub use wire::ConfigWire;
