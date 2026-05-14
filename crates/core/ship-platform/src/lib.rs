// ── L1 modules (agent/collection domain layer) ────────────────────────────────
pub mod agents;
pub mod artifacts;
pub mod auth;
pub mod collections;
pub mod config;
pub mod hooks;
pub mod kv;
pub mod projections;
pub mod projects;
pub mod registry;
pub mod services;
pub mod sessions;

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
            Box::new(projections::AuthSessionsProjection),
            Box::new(projections::DaemonOwnerProjection),
            Box::new(projections::JobProjection),
        ]
    }));
    ship_substrate::supervisor::install_project_id_resolver(Box::new(|path| {
        projects::find_by_path(path).ok().flatten().map(|p| p.id)
    }));
}
