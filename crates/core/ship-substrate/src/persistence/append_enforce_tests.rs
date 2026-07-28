//! Append-chokepoint enforcement proofs.
//!
//! These exercise the OFF-KERNEL write path directly — `SqliteEventStore::
//! append_enforced(...)` — not `KernelRouter::emit`. They are the append-path
//! mirror of `events/kernel_router_enforce_tests.rs`: they prove that the
//! direct-append sites (`db::*_events`, `log`, platform CRUD) run the same
//! stamp-or-reject rule the emit chokepoint runs, so no live write reaches
//! persist with an unenforced scope_key.

use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use ship_types::identity::{AccountId, DeviceId, SpaceId};
use agent_schema::Authority;
use ship_types::{ScopeBinding};

use super::event_store::{EventStore, SqliteEventStore};

/// A session binding bound to `space`, carrying device + account identity.
fn space_binding(space: &str) -> ScopeBinding {
    ScopeBinding::from_session(
        DeviceId::new("dev-1"),
        Some(AccountId::new("acct-1")),
        Some(SpaceId::new(space)),
        Authority::Space {
            space_id: space.to_string(),
        },
    )
}

/// A space-unbound session binding (device + account, no space).
fn spaceless_binding() -> ScopeBinding {
    ScopeBinding::from_session(
        DeviceId::new("dev-1"),
        Some(AccountId::new("acct-1")),
        None,
        Authority::Sdk,
    )
}

/// A Space-scoped store for `space` rooted under a temp ship-home. The same
/// db `append_enforced` lands in is the one the reader queries back.
fn space_store(tmp: &tempfile::TempDir, space: &str) -> SqliteEventStore {
    let root = tmp.path().join(".ship");
    SqliteEventStore::new_for_scope(&root, EventScope::Space, Some(space)).unwrap()
}

#[test]
fn append_enforced_stamps_space_key_and_author_from_binding() {
    let tmp = tempfile::tempdir().unwrap();
    let store = space_store(&tmp, "space-A");
    // Space event with NO scope_key — append_enforced must stamp it from the
    // binding, plus the author identity, before persisting.
    let event = EventEnvelope::new("session.started", "entity-1", &serde_json::json!({}))
        .unwrap()
        .with_scope(EventScope::Space, None);
    let id = event.id.clone();

    let stamped = store
        .append_enforced(&event, &space_binding("space-A"))
        .expect("happy path persists");
    // Returned envelope is the stamped one (so callers deliver the enforced row).
    assert_eq!(stamped.scope_key.as_deref(), Some("space-A"));
    assert_eq!(stamped.author_device_id.as_deref(), Some("dev-1"));
    assert_eq!(stamped.author_account_id.as_deref(), Some("acct-1"));

    let persisted = store.get(&id).unwrap().expect("event must be persisted");
    assert_eq!(persisted.scope, EventScope::Space);
    assert_eq!(persisted.scope_key.as_deref(), Some("space-A"));
    assert_eq!(persisted.author_device_id.as_deref(), Some("dev-1"));
    assert_eq!(persisted.author_account_id.as_deref(), Some("acct-1"));
}

#[test]
fn append_enforced_rejects_cross_space_and_does_not_persist() {
    let tmp = tempfile::tempdir().unwrap();
    // Store is space-B; binding is space-A → CrossSpace reject. We target the
    // space-B store directly (its db is where a key=space-B event would land).
    let store = space_store(&tmp, "space-B");
    let event = EventEnvelope::new("session.started", "entity-1", &serde_json::json!({}))
        .unwrap()
        .with_scope(EventScope::Space, Some("space-B"));
    let id = event.id.clone();

    let err = store
        .append_enforced(&event, &space_binding("space-A"))
        .unwrap_err();
    assert!(
        err.to_string().contains("cross-space"),
        "unexpected error: {err}"
    );

    assert!(
        store.get(&id).unwrap().is_none(),
        "rejected cross-space event must not persist"
    );
}

#[test]
fn append_enforced_rejects_space_event_from_space_unbound_binding() {
    let tmp = tempfile::tempdir().unwrap();
    let store = space_store(&tmp, "space-A");
    let event = EventEnvelope::new("session.started", "entity-1", &serde_json::json!({}))
        .unwrap()
        .with_scope(EventScope::Space, None);
    let id = event.id.clone();

    let err = store
        .append_enforced(&event, &spaceless_binding())
        .unwrap_err();
    assert!(
        err.to_string().contains("space-unbound"),
        "unexpected error: {err}"
    );

    assert!(
        store.get(&id).unwrap().is_none(),
        "space event from a space-unbound binding must not persist"
    );
}
