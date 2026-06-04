//! `ship-config` — the SettingsService home (L1). The unified, scope-
//! discriminated settings keystone over substrate ports. (The crate-NAME rename
//! ship-config → ship-settings is deferred to S4; the legacy `config` file-tier
//! below stays live until then.)
//!
//! The settings surface has three scopes (`settings.proto`'s SettingScope):
//!
//! - **RUNTIME** — machine-local launch defaults: the `daemon_settings` table in
//!   `runtime.db`, through `ship_substrate::db::daemon_settings`. Event-sourced
//!   via `settings.changed` riding `EventScope::Runtime`. The well-known infra
//!   defaults + precedence resolvers live in [`settings`].
//! - **ACCOUNT** — the user's account: the `settings_overlay` table in
//!   `account.db` (a per-account `DataStore`). `settings.changed` rides
//!   `EventScope::Account`.
//! - **SPACE** — the active space: the `settings_overlay` table in that space's
//!   `space.db`. `settings.changed` rides `EventScope::Space`, keyed by the
//!   space id. Errors cleanly when no space is active.
//!
//! The overlay read/write for the DataStore-backed scopes lives in [`overlay`];
//! the scope-discriminated business logic in [`service`]; the ConnectRPC wire in
//! [`wire`].
//!
//! The **legacy declarative file-tier** also lives here (unchanged): the
//! `.ship/agents/` projection ([`agents`]) and the per-domain `.ship/<domain>/`
//! change watcher ([`watcher`]). ALL disk goes through the
//! [`ship_substrate::ports::FsBroker`] gated door — ZERO `std::fs`/`tokio::fs` in
//! this crate. Storage + events go through the port types from `ship-substrate`.
//!
//! [`KernelLog`]: ship_substrate::ports::KernelLog

pub mod agents;
pub mod agents_index;
pub mod agents_write;
pub mod error;
pub mod overlay;
pub mod patch;
pub mod service;
pub mod settings;
pub mod types;
pub mod watcher;
pub mod wire;

pub use service::{ResolvedSetting, SettingScope, SettingsCtx, SettingsService};
pub use wire::SettingsWire;
