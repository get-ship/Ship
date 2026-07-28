//! `ship-platform` (L1) — the domain-layer facade and projection assembly point.
//!
//! This crate re-exports the L1 domain sub-crates (`ship-config`, `ship-agents`,
//! `ship-auth`, `ship-isolation`, etc.) at their legacy `ship_platform::*` paths
//! and installs the domain projection set via [`install_defaults`].
//!
//! Most logic lives in the sub-crates; this file is the composition root and
//! back-compat re-export surface.

// ── L1 modules (agent/collection domain layer) ────────────────────────────────

/// Platform config facade — thin re-export shim over `ship-config` (SSOT).
pub mod config {
    pub use ship_config::error::ConfigError;
    // `ship_config::project` (SpaceManifest / get_config / save_config /
    // PRIMARY_CONFIG_FILE) died with the A2 close-out (2026-07-11). The
    // resolved descriptor (`agent_schema::ResolvedConfig`) is the one source
    // of truth post-cutover; the on-disk config surface is dead.
    // Agent def readers (still live — A2 delete only removed the
    // deprecated `ship_config::project::SpaceManifest`/`get_config`
    // chain, not these typed agent-def read functions).
    pub use ship_config::agents::{
        read_agent_doc_in, read_agent_docs_in, read_agent_profile, read_agent_profile_at,
        read_agent_profiles, read_agent_profiles_in,
    };
    pub use ship_config::agents_ts::read_agent_ts_sources;

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

    // MCP CRUD (slice 2c). The `McpServerConfig` type now lives in
    // `ship_config::crud` (the deleted `project::` module is gone).
    pub use ship_config::crud::{
        McpServerConfig, McpServerType, add_mcp_server, ensure_standard_dirs,
        list_mcp_servers, remove_mcp_server,
    };
    // Settings
    pub use ship_config::settings::{
        get_all_settings, get_setting, get_setting_u64_or, resolve_bool, resolve_git_log_limit,
        resolve_hooks_max_depth, resolve_str, resolve_watcher_debounce, set_setting,
        set_setting_and_emit, setting_default,
    };

    // Settings view-state helpers (display structs for the Settings page).
    // Relocated from shipd::services::settings::view_state (L2 → L1).
    pub mod settings_view;

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

#[cfg(any(test, feature = "testutil"))]
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
        #[allow(unused_mut)]
        let mut projections: Vec<Box<dyn ship_substrate::projections::Projection>> = vec![
            // Each projection now lives with its domain; the assembly point
            // pulls from those homes. The pipeline-run projection registers
            // separately from `ship-pipeline-runtime` (it depends on this
            // crate, so it can't be referenced from here).
            // SpacesProjection (keystone-A's ProjectsProjection rewritten for
            // the 2026-07-09 space cutover) folds `space.opened` and
            // `space.seen` events into the runtime-scoped `spaces` projection
            // table. Identity is owned by the space, never derived from the
            // path — see `ship_space::identity::ensure_identity_for_root`.
            Box::new(ship_space::SpacesProjection),
            Box::new(ship_space::SpaceStacksProjection),
            Box::new(ship_auth::AuthSessionsProjection),
            // Single-row daemon-owner projection — folds `daemon.owner.*`
            // events into the `daemon_owner` table `current_owner()` reads.
            // Was only registered by `ship-auth::testutil`; production
            // (shipd via `install_defaults`) silently skipped it, so
            // `current_owner()` was always None outside tests (claim
            // c-2026-07-02T01-40-29, 2026-07-02 schema assessment).
            Box::new(ship_auth::DaemonOwnerProjection),
            // Per-agent approved grant baseline the mint path reads.
            // ProvisionedGrantsProjection moved to ship-auth (keystone-A
            // Phase A bundle 2, 2026-06-10): the AUTH domain owns it at L1.
            Box::new(ship_auth::ProvisionedGrantsProjection),
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
            // ArtifactProjection — event-sourced artifact METADATA (ADR
            // D6/D7), moved from the L0 default bus (keystone-A de-bucket,
            // 2026-06-10). Blobs stay on the content-addressed blob plane.
            Box::new(ship_artifacts::metadata_projection::ArtifactProjection),
            // ship-analytics — folds hook.* into rollup tables. Lives
            // in its own crate (queries, cost, mining) so downstream
            // dashboard / billing / harness-mining work has a home
            // outside the platform crate.
            Box::new(ship_analytics::AnalyticsProjection::new()),
        ];
        // Compile-state projection rides the ship-code feature: the compile
        // driver (agent-def/compile) is third-party-harness territory and the
        // standard build never carries it.
        #[cfg(feature = "ship-code")]
        projections.push(Box::new(agent_def::compile::CompileStateProjection));
        projections
    }));
}
