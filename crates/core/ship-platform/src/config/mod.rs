//! Platform config facade — re-exports from `ship-config` crate (SSOT) plus
//! the thin platform-specific items that depend on platform internals
//! (`crud` → services::compile, `watcher` → spawn loop, `discover` → fs walk).
//!
//! ALL type/settings/agent definitions live in the `ship-config` crate. This
//! module is a thin shim for the items that cannot move there due to
//! platform-internal dependencies (services::compile, KernelRouter spawn loop).

mod crud;
mod discover;
pub mod watcher;

#[cfg(test)]
mod tests_boundaries;
#[cfg(test)]
mod tests_crud;
#[cfg(test)]
mod tests_io;

// ─── ship-config re-exports ────────────────────────────────────────────────────

// Core trait
pub use ship_config::DiskBackedConfig;

// Types (live in ship_config::project, not ::types)
pub use ship_config::project::{
    AgentLayerConfig, HookConfig, HookTrigger, PermissionConfig,
};
pub use ship_substrate::project::PRIMARY_CONFIG_FILE;

// Project config
pub use ship_config::project::{
    AgentProfile, HarnessProjectOverrides, McpConfig, McpSection, McpServerConfig, McpServerType,
    ProjectConfig, ProjectDiscovery,
};

// MCP
pub use ship_config::project::get_mcp_config;

// IO
pub use ship_config::project::{get_config, get_effective_config, save_config};

// Agent def readers
pub use ship_config::agents::{
    read_agent_doc_in, read_agent_docs_in, read_agent_profile, read_agent_profile_at,
    read_agent_profiles, read_agent_profiles_in,
};

// Agent def index + consumer
pub use ship_config::agents_index::{
    AgentDefIndex, AgentDefQuery, AgentDefRecord,
    apply_def_updated, build_def_index, spawn_def_index_consumer,
};
pub use ship_config::agents_write::notify_agent_def_saved;
pub mod agent_index {
    pub use ship_config::agents_index::{apply_def_updated, build_def_index, spawn_def_index_consumer};
}

// Settings
pub use ship_config::settings::{
    get_all_settings, get_setting, get_setting_u64_or, resolve_bool, resolve_git_log_limit,
    resolve_hooks_max_depth, resolve_str, resolve_watcher_debounce, set_setting,
    set_setting_and_emit, setting_default,
};

// File-backed config types
pub use ship_config::library::LibrarySettings;
pub use ship_config::local::{worktrees_dir, LocalSettings};
pub use ship_config::library;
pub use ship_config::local;

// Space settings projection
pub use ship_config::projection::SpaceSettingsProjection;

// Platform-specific
pub use crud::{
    add_mcp_server, ensure_standard_dirs,
    get_active_agent, list_mcp_servers,
    remove_mcp_server,
};
pub use discover::discover_projects;
pub use watcher::{spawn_config_watcher, ConfigWatcher};
