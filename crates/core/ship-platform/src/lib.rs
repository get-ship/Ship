//! `ship-platform` (L1) — composition root for domain projections.
//!
//! Domain types and APIs live in their own crates (`ship-settings`, `ship-agents`,
//! `ship-auth`, `ship-isolation`, etc.). Import those crates directly — this
//! crate no longer re-exports them. The one public surface here is the
//! projection assembly factory used by composition roots (shipd, shipctl, tests).

/// Settings page view-state helpers (display structs for the Settings page).
/// Relocated from shipd::services::settings::view_state (L2 → L1).
pub mod settings_view;

#[cfg(any(test, feature = "testutil"))]
pub mod testutil;

/// Build the platform-level projection factory for composition roots.
///
/// Pass the returned factory into `init_kernel_router_with`,
/// `PartitionedSink::with_extra_projections`, `rebuild_projections_with`,
/// or `SqliteEventStore::with_projection_factory`. There is no process-global
/// install step — projections travel as constructor arguments.
pub fn platform_projection_factory() -> ship_substrate::projections::ProjectionFactory {
    std::sync::Arc::new(|| {
        #[allow(unused_mut)]
        let mut projections: Vec<Box<dyn ship_substrate::projections::Projection>> = vec![
            // Each projection lives with its domain; the assembly point
            // pulls from those homes. The pipeline-run projection registers
            // separately from `pipeline-defs` (it depends on this
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
        #[cfg(feature = "bridge")]
        projections.push(Box::new(agent_def::compile::CompileStateProjection));
        projections
    })
}

/// Build the platform projection factory.
///
/// Historical name kept so call sites that previously "installed" into a
/// process-global now obtain the factory they must pass into store / kernel
/// constructors. Returns the same value as [`platform_projection_factory`].
pub fn install_defaults() -> ship_substrate::projections::ProjectionFactory {
    platform_projection_factory()
}
