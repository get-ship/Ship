// ── L1 modules (agent/collection domain layer) ────────────────────────────────
pub mod agents;
pub mod auth;
pub mod commands;
pub mod config;
pub mod hooks;
pub mod isolation;
pub mod isolation_ops;
pub mod sandbox;
pub mod triggers;
pub mod kv;
pub mod projections;
pub mod projects;
pub mod registry;
pub mod services;
pub mod sessions;
pub mod toolchain;
pub mod view_authoring;

// Module aliases — sub-paths of agents that callers reference at crate root.
pub use agents::permissions;
pub use agents::rule;
pub use agents::skill;
pub use agents::skill_paths;
pub use agents::skill_vars;

#[cfg(feature = "unstable")]
pub mod sync;

#[cfg(test)]
pub mod testutil;

/// Install the platform-level projection set and substrate hooks. Call
/// once at application startup, before constructing any `SqliteEventStore`.
/// Idempotent — second and later calls are silently ignored.
pub fn install_defaults() {
    ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
        vec![
            Box::new(projections::CompileStateProjection),
            Box::new(projections::ProjectsProjection),
            // Per-space settings overlay (event-sourced), layered over the
            // global daemon settings by `SpaceService.Info`.
            Box::new(projections::SpaceSettingsProjection),
            Box::new(projections::AuthSessionsProjection),
            Box::new(projections::DaemonOwnerProjection),
            // Per-agent approved grant baseline the mint path reads.
            Box::new(projections::ProvisionedGrantsProjection),
            Box::new(projections::PipelineRunProjection),
            // Agent-session primitive — sibling of jobs. Tracks
            // single-agent dispatches (bare ship use, MCP dispatch,
            // pipeline-step sessions). jobs now scopes to pipeline
            // composition runs only.
            Box::new(projections::AgentSessionProjection),
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
