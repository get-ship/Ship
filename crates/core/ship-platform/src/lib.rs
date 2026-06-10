// ── L1 modules (agent/collection domain layer) ────────────────────────────────

/// Platform config facade — thin re-export shim over `ship-config` (SSOT).
pub mod config {
    // Core trait
    pub use ship_config::DiskBackedConfig;

    // Types (live in ship_config::project, not ::types)
    pub use ship_config::project::{
        AgentLayerConfig, HookConfig, HookTrigger, PermissionConfig,
    };
    pub use ship_substrate::project::PRIMARY_CONFIG_FILE;

    // Project config
    pub use ship_config::project::{
        AgentProfile, HarnessProjectOverrides, McpConfig, McpSection, McpServerConfig,
        McpServerType, ProjectConfig, ProjectDiscovery,
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
        pub use ship_config::agents_index::{
            apply_def_updated, build_def_index, spawn_def_index_consumer,
        };
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

    // Settings view-state helpers (display structs for the Settings page).
    // Relocated from shipd::services::settings::view_state (L2 → L1).
    pub mod settings_view;

    // Relocated from platform-internal crud/discover/watcher
    pub use ship_config::crud::{add_mcp_server, ensure_standard_dirs, list_mcp_servers, remove_mcp_server};
    pub use ship_agents::active_agent::get_active_agent;
    pub use ship_config::discover::discover_projects;
    pub use ship_config::watcher::{spawn_config_watcher, ConfigWatcher};
}

/// Isolation decision layer — moved to ship-isolation crate (SSOT).
pub mod isolation {
    pub use ship_isolation::decision::*;
}

/// Isolation ops (wizard/UI) — moved to ship-isolation crate (SSOT).
pub mod isolation_ops {
    pub use ship_isolation::decision_ops::*;
}

/// Service infrastructure — in-process registry + git re-export + fixtures.
pub mod services {
    /// Agent-session orchestration re-export (C8b fold).
    pub mod agent_sessions {
        pub use ship_agents::{
            handle_agent_session_started, handle_agents_event,
            start_agents, AgentPorts,
        };
        // cancel is gated on harness-drivers (ship-code dep required).
        #[cfg(feature = "harness-drivers")]
        pub use ship_agents::handle_agent_cancelled;
    }

}

pub mod testutil;

/// Install the platform-level projection set and substrate hooks. Call
/// once at application startup, before constructing any `SqliteEventStore`.
/// Idempotent — second and later calls are silently ignored.
pub fn install_defaults() {
    static INSTALLED: std::sync::Once = std::sync::Once::new();
    INSTALLED.call_once(install_defaults_inner);
}

fn install_defaults_inner() {
    ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
        vec![
            // Each projection now lives with its domain; the assembly point
            // pulls from those homes. The pipeline-run projection registers
            // separately from `ship-pipeline-runtime` (it depends on this
            // crate, so it can't be referenced from here).
            Box::new(ship_agents::services::compile::CompileStateProjection),
            Box::new(ship_substrate::db::projects::ProjectsProjection),
            // Per-space settings overlay (event-sourced fold of
            // `space.setting.changed`). The live overlay read is owned by the
            // ship-space crate's own DataStore-backed table (`SpaceService.Info`);
            // this projection materializes the same events for the event history.
            Box::new(config::SpaceSettingsProjection),
            Box::new(ship_auth::AuthSessionsProjection),
            Box::new(ship_auth::DaemonOwnerProjection),
            // Per-agent approved grant baseline the mint path reads.
            Box::new(ship_agents::services::permissions::ProvisionedGrantsProjection),
            // Canonical session projection — folds session.* events into
            // canonical_sessions (runtime.db). Replaces the legacy
            // AgentSessionProjection (retired by canonical-emitter-migration).
            Box::new(ship_agents::services::sessions::CanonicalSessionProjection),
            // CollectionItemProjection — event-sourced collection rows (ADR
            // D6/D7). Moved here from the L0 default bus (keystone A,
            // 2026-06-10): the COLLECTION domain owns its projection at L1.
            // Same-TX guarantee is preserved because this factory runs inside
            // the same SQLite transaction as the event append.
            Box::new(ship_collection::item_projection::CollectionItemProjection),
            // ship-analytics — folds hook.* into rollup tables. Lives
            // in its own crate (queries, cost, mining) so downstream
            // dashboard / billing / harness-mining work has a home
            // outside the platform crate.
            Box::new(ship_analytics::AnalyticsProjection::new()),
        ]
    }));
    ship_substrate::supervisor::install_space_id_resolver(Box::new(|path| {
        let path_str = path.to_string_lossy();
        ship_substrate::db::projects::find_by_path(&path_str).ok().flatten().map(|p| p.id)
    }));
}
