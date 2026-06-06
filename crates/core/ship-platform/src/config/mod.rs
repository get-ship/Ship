//! Platform config facade — thin re-export shim over `ship-config` (SSOT).
//!
//! All logic previously in the `crud`, `discover`, and `watcher` sub-modules
//! has been relocated to the `ship-config` crate. This shim re-exports the
//! full public surface so existing callers continue to compile unchanged.

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

// Relocated from platform-internal crud/discover/watcher
pub use ship_config::crud::{add_mcp_server, ensure_standard_dirs, list_mcp_servers, remove_mcp_server};
pub use ship_agents::active_agent::get_active_agent;
pub use ship_config::discover::discover_projects;
pub use ship_config::watcher::{spawn_config_watcher, ConfigWatcher};
