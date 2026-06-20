//! Test helpers for ship-auth integration tests.
//!
//! Registers the auth + grants projections onto the substrate event bus so that
//! `SqliteEventStore::new_for_scope` sees the full auth read-model. Called at
//! the start of any test that appends events and then reads projections.

use std::sync::Once;

static AUTH_INSTALLED: Once = Once::new();

/// Register auth projections (AuthSessions, DaemonOwner, ProvisionedGrants) on
/// the substrate projection factory. Idempotent — safe to call from multiple
/// tests. Returns the `.ship/` directory inside `base_dir`.
pub fn init_auth(base_dir: impl AsRef<std::path::Path>) -> anyhow::Result<std::path::PathBuf> {
    AUTH_INSTALLED.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![
                Box::new(crate::projection::AuthSessionsProjection)
                    as Box<dyn ship_substrate::projections::Projection>,
                Box::new(crate::projection::DaemonOwnerProjection),
                Box::new(crate::grants_projection::ProvisionedGrantsProjection),
            ]
        }));
    });

    ship_substrate::testutil::init_project_minimal(base_dir)
}

/// Query all events of a given type from the auto-isolated Account-scope
/// store. Account-scoped events (`auth.session.*`, `daemon.owner.*`,
/// `agent.grants.*`) land in `account.db`; this helper opens that store
/// with the same auto-isolated path the test environment uses.
pub fn query_events(
    event_type: &str,
) -> Vec<ship_types::envelope::EventEnvelope> {
    use ship_substrate::events::filter::EventFilter;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    let store = SqliteEventStore::new_for_scope(
        &ship_substrate::project::get_global_dir().expect("global dir"),
        ship_types::event_registry::EventScope::Account,
        None,
    )
    .expect("open auto-isolated test store");
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .expect("query events")
}
