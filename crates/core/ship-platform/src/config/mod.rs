pub mod catalog;
pub mod credentials;
pub mod library;
pub mod local;
pub mod security;

mod crud;

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;

use ship_substrate::ShipHome;

/// Settings document persisted as JSONC under a Ship home directory.
///
/// Each impl supplies `path_at(&ShipHome)`; the trait wires the standard
/// load / save / parse behavior. Override `after_save` for per-doc concerns
/// (e.g. `chmod 600` on credentials).
pub trait DiskBackedConfig: Sized + Default + Serialize + DeserializeOwned {
    /// Path under an explicit ship home.
    fn path_at(home: &ShipHome) -> PathBuf;

    /// Path under the active SHIP_HOME.
    fn path() -> PathBuf {
        Self::path_at(&ShipHome::from_env())
    }

    /// Strip JSONC comments and parse.
    fn parse(raw: &str) -> Result<Self> {
        let stripped = ship_substrate::fs_util::strip_jsonc_comments(raw);
        Ok(serde_json::from_str(&stripped)?)
    }

    /// Load from `path()`. Defaults on missing or unparseable input.
    fn load() -> Self {
        Self::load_at(&Self::path())
    }

    /// Load from an explicit path. Defaults on missing or unparseable input.
    fn load_at(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => Self::parse(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save to `path()`, creating parent dirs as needed.
    fn save(&self) -> Result<()> {
        self.save_at(&Self::path())
    }

    /// Save to an explicit path, creating parent dirs as needed.
    fn save_at(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(self)?;
        std::fs::write(path, format!("{body}\n"))?;
        Self::after_save(path)?;
        Ok(())
    }

    /// Hook for per-impl post-save behavior. Default: no-op.
    fn after_save(_path: &Path) -> Result<()> {
        Ok(())
    }
}
mod discover;
mod io;
mod mcp;
mod merge;
pub mod agents;
mod project;
mod runtime_settings;
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
    LEGACY_CONFIG_FILE, PRIMARY_CONFIG_FILE, PermissionConfig,
};

pub use project::{
    AgentProfile, McpConfig, McpSection, McpServerConfig, McpServerType, ProjectConfig,
    ProjectDiscovery,
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

pub use runtime_settings::{
    drain_legacy_project_config_kv, drain_legacy_settings_kv, get_all_settings, get_setting,
    set_setting, set_setting_and_emit, setting_default,
};

pub use credentials::Credentials;
pub use library::LibrarySettings;
pub use local::{worktrees_dir, LocalSettings};
