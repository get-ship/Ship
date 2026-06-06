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
pub mod library;
pub mod local;
pub mod overlay;
pub mod patch;
pub mod projection;
pub mod project;
pub mod service;
pub mod settings;
pub mod types;
pub mod watcher;
pub mod wire;

pub use service::{ResolvedSetting, SettingScope, SettingsCtx, SettingsService};
pub use settings::resolve_hooks_max_depth;
pub use wire::SettingsWire;

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;

use ship_substrate::ShipHome;
use ship_substrate::config_doc;

/// Settings document persisted as JSONC under a Ship home directory.
///
/// Each impl supplies `path_at(&ShipHome)`; the trait wires the standard
/// load / save / parse behavior over the substrate config-document accessor —
/// L1 never touches `std::fs` for config bytes. Override `sensitive()` to
/// `true` for documents holding machine-local secrets (writes them owner-only).
pub trait DiskBackedConfig: Sized + Default + Serialize + DeserializeOwned {
    /// Path under an explicit ship home.
    fn path_at(home: &ShipHome) -> PathBuf;

    /// Path under the active SHIP_HOME.
    fn path() -> PathBuf {
        Self::path_at(&ShipHome::from_env())
    }

    /// Whether this document holds machine-local secrets. When `true`, saves
    /// restrict the file to owner-only (`chmod 600` on Unix). Default `false`.
    fn sensitive() -> bool {
        false
    }

    /// Strip JSONC comments and parse.
    fn parse(raw: &str) -> Result<Self> {
        config_doc::parse_doc(raw)
    }

    /// Load from `path()`. Defaults on missing or unparseable input.
    fn load() -> Self {
        Self::load_at(&Self::path())
    }

    /// Load from an explicit path. Defaults on missing or unparseable input.
    fn load_at(path: &Path) -> Self {
        config_doc::read_doc_or_default(path)
    }

    /// Save to `path()`, creating parent dirs as needed.
    fn save(&self) -> Result<()> {
        self.save_at(&Self::path())
    }

    /// Save to an explicit path, creating parent dirs as needed.
    fn save_at(&self, path: &Path) -> Result<()> {
        if Self::sensitive() {
            config_doc::write_secret_doc(path, self)
        } else {
            config_doc::write_doc(path, self)
        }
    }
}
