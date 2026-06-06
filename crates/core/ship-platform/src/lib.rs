// ── Device-domain re-exports (back-compat) ───────────────────────────────────
// The sandbox module moved from this crate to `ship-device` as part of the
// supervisor→platform cycle break (c2-sandbox-device). Re-export its public
// surface here so downstream callers (`apps/shipd`, the supervisor) can keep
// importing `ship_platform::sandbox::*` until they migrate in lockstep.
pub use ship_device::sandbox::{
    dispatcher, RuntimeEndpoint, Sandbox, SandboxDispatcher, SandboxError,
    SandboxId, SandboxLifetime, SandboxName, SandboxRegistry, SandboxState,
};
// ── L1 modules (agent/collection domain layer) ────────────────────────────────
pub mod agents;
pub mod auth;
pub mod commands;
pub mod config;
pub mod hooks;
pub mod isolation;
pub mod isolation_ops;
pub mod triggers;
pub use ship_kv as kv;
pub mod projects;
pub mod registry;
pub mod services;
pub mod sessions;
// View-authoring + compile services. Carved out to the `ship-views` crate
// in a later step (c5-views); kept here for now and re-exported from
// ship-views when that lands.
pub mod view_authoring;

// Toolchain is an L0 concern (binary management infra — ensure/install/update
// of the bundled `sui-compiler`). It lives in `ship_substrate::toolchain` and
// is re-exported here for back-compat with downstream callers that import
// `ship_platform::toolchain::*`.
pub use ship_substrate::toolchain;
// Module aliases — sub-paths of agents that callers reference at crate root.
pub use agents::permissions;
pub use agents::rule;
pub use agents::skill;
pub use agents::skill_paths;
pub use agents::skill_vars;

#[cfg(test)]
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
            Box::new(services::compile::CompileStateProjection),
            Box::new(projects::ProjectsProjection),
            // Per-space settings overlay (event-sourced fold of
            // `space.setting.changed`). The live overlay read is owned by the
            // ship-space crate's own DataStore-backed table (`SpaceService.Info`);
            // this projection materializes the same events for the event history.
            Box::new(config::SpaceSettingsProjection),
            Box::new(auth::AuthSessionsProjection),
            Box::new(auth::DaemonOwnerProjection),
            // Per-agent approved grant baseline the mint path reads.
            Box::new(services::permissions::ProvisionedGrantsProjection),
            // Agent-session primitive — sibling of pipeline runs. Tracks
            // single-agent dispatches (bare ship use, MCP dispatch,
            // pipeline-step sessions).
            Box::new(services::sessions::AgentSessionProjection),
            // ship-analytics — folds hook.* into rollup tables. Lives
            // in its own crate (queries, cost, mining) so downstream
            // dashboard / billing / harness-mining work has a home
            // outside the platform crate.
            Box::new(ship_analytics::AnalyticsProjection::new()),
        ]
    }));
    ship_substrate::supervisor::install_space_id_resolver(Box::new(|path| {
        projects::find_by_path(path).ok().flatten().map(|p| p.id)
    }));
    commands::seed_builtins();
}
