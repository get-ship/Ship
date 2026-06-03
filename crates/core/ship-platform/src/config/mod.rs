pub mod library;
pub mod local;
pub mod security;

mod crud;

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
mod discover;
mod io;
mod mcp;
mod merge;
pub mod agents;
mod project;
mod types;
pub mod watcher;

#[cfg(test)]
mod tests_boundaries;
#[cfg(test)]
mod tests_crud;
#[cfg(test)]
mod tests_io;

// ─── Public API — mirrors the original config.rs public surface ───────────────

pub use types::{
    AgentLayerConfig, HookConfig, HookTrigger,
    PRIMARY_CONFIG_FILE, PermissionConfig,
};

pub use project::{
    AgentProfile, HarnessProjectOverrides, McpConfig, McpSection, McpServerConfig, McpServerType,
    ProjectConfig, ProjectDiscovery,
};

pub use mcp::get_mcp_config;

pub use io::{get_config, get_effective_config, save_config};

pub use crud::{
    add_mcp_server, ensure_standard_dirs,
    get_active_agent, list_mcp_servers,
    remove_mcp_server,
};

pub use discover::discover_projects;
pub use watcher::{spawn_config_watcher, ConfigWatcher};

// RUNTIME-scope settings API — the single source of truth now lives in the
// `ship-config` crate (`ship_config::settings`); the legacy in-tree settings
// module was unified into it (S4). Re-exported here so existing
// `ship_platform::config::{get_setting, …}` consumers keep working.
pub use ship_config::settings::{
    get_all_settings, get_setting, get_setting_u64_or, resolve_bool, resolve_git_log_limit,
    resolve_hooks_max_depth, resolve_str, resolve_watcher_debounce, set_setting,
    set_setting_and_emit, setting_default,
};

pub use library::LibrarySettings;
pub use local::{worktrees_dir, LocalSettings};
