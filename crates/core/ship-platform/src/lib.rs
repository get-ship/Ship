// ── L1 modules (agent/collection domain layer) ────────────────────────────────
pub mod config;
pub mod isolation;
pub mod isolation_ops;
pub mod services;

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
            // Agent-session primitive — sibling of pipeline runs. Tracks
            // single-agent dispatches (bare ship use, MCP dispatch,
            // pipeline-step sessions).
            Box::new(ship_agents::services::sessions::AgentSessionProjection),
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
