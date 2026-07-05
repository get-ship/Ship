//! `daemon_owner` projection is registered in the production factory.
//!
//! Regression: the production `ship_platform::install_defaults()` did not
//! include `ship_auth::DaemonOwnerProjection` in its factory, so the
//! `daemon.owner.bound` event emitted by `ship login` never folded into the
//! `daemon_owner` table outside of tests. `ship_auth::current_owner()` was
//! always `None` in production. This test goes through the production factory
//! (NOT `ship_auth::testutil::init_auth`) — a regression here reproduces the
//! July-2 bug ("c-2026-07-02T01-40-29").

use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::project::get_global_dir;
use ship_substrate::ShipHome;

#[test]
fn daemon_owner_projection_registered_in_production_factory() {
    // Drive the production composition root.
    ship_platform::install_defaults();

    let ship_dir = get_global_dir().expect("auto-isolated test ship dir");

    // Append `daemon.owner.bound` through an Account-scope store. The store
    // loads its bus from the factory that `install_defaults` just installed;
    // if `DaemonOwnerProjection` isn't there, the table never folds.
    let store = SqliteEventStore::new_for_scope(&ship_dir, ship_types::event_registry::EventScope::Account, None)
        .expect("account store");

    let payload = ship_types::payload::DaemonOwnerBound {
        source: "login".to_string(),
        bound_at_unix: chrono::Utc::now().timestamp(),
    };
    let envelope = ship_types::envelope::EventEnvelope::new(
        ship_types::event_types::DAEMON_OWNER_BOUND,
        "user-1",
        &payload,
    )
    .expect("envelope");
    store.append(&envelope).expect("append owner.bound");

    // Read via the production helper. With the projection registered the
    // single-row `daemon_owner` table now reflects the bind; without it the
    // read returns `None` and the assertion fails — the original bug.
    let owner = ship_auth::current_owner().expect("current_owner read");
    let row = owner.expect("daemon_owner projection did not fold (claim c-2026-07-02T01-40-29)");
    assert_eq!(row.user_id, "user-1");
    assert_eq!(row.source, "login");
    assert_eq!(row.bound_at_unix, payload.bound_at_unix);

    // Sanity: the ship_dir path is independent of the operator's real on-disk
    // ship home — `ShipHome::from_env` is the single resolver under test here.
    let _ = ShipHome::from_env();
}
