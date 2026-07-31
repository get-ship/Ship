//! Test helpers for ship-auth integration tests.
//!
//! Production `account_store()` already registers AUTH projections on the bus
//! it opens. This helper only scaffolds a minimal `.ship/` tree for tests.

/// Scaffold a minimal `.ship/` tree for auth integration tests. Auth
/// projections ride the store opened by [`crate::scopes::account_store`] —
/// no process-global install step. Returns the `.ship/` directory inside
/// `base_dir`.
pub fn init_auth(base_dir: impl AsRef<std::path::Path>) -> anyhow::Result<std::path::PathBuf> {
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
    use ship_substrate::persistence::event_store::EventStore;
    // `account_store` already wires AUTH projections for same-TX folds.
    let store = crate::scopes::account_store().expect("open auto-isolated test store");
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .expect("query events")
}
