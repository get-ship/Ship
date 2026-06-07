//! PartitionedSink routing + isolation tests (Phase 4a).
//!
//! Each test proves an event lands in exactly the per-scope db file its
//! `(scope, scope_key)` maps to — and is absent from every other scope's db.
//! Read-back opens the specific per-scope db directly via
//! `SqliteEventStore::new_at_db_path` so the assertion is independent of any
//! router or projection read path.

use std::sync::Arc;

use ship_kernel::EventSink;
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use tempfile::tempdir;

use super::event_store::{EventStore, SqliteEventStore};
use super::partitioned_sink::PartitionedSink;
use crate::db;

fn event(scope: EventScope, scope_key: Option<&str>) -> EventEnvelope {
    EventEnvelope::new("test.partition", "entity-1", &serde_json::json!({}))
        .unwrap()
        .with_scope(scope, scope_key)
}

/// Open the db at `path` and assert whether event `id` is present.
fn has_event(path: &std::path::Path, id: &str) -> bool {
    // The per-scope db may not even exist if no event was routed there — treat
    // a missing file as "absent" rather than provisioning it.
    if !path.exists() {
        return false;
    }
    let store = SqliteEventStore::new_at_db_path(path).unwrap();
    store.get(id).unwrap().is_some()
}

#[test]
fn space_event_routes_to_its_own_space_db() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink = PartitionedSink::new(root.to_path_buf());

    let ev = event(EventScope::Space, Some("A"));
    sink.persist(&ev).unwrap();

    assert!(has_event(&db::space_db_path(root, "A"), &ev.id));
}

#[test]
fn space_event_isolated_from_other_scopes() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink = PartitionedSink::new(root.to_path_buf());

    let ev = event(EventScope::Space, Some("A"));
    sink.persist(&ev).unwrap();

    // Not in another space, nor in account/runtime.
    assert!(!has_event(&db::space_db_path(root, "B"), &ev.id));
    assert!(!has_event(&db::account_db_path(root), &ev.id));
    assert!(!has_event(&db::runtime_db_path(root), &ev.id));
}

#[test]
fn two_spaces_are_independently_addressable() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink = PartitionedSink::new(root.to_path_buf());

    let a = event(EventScope::Space, Some("A"));
    let b = event(EventScope::Space, Some("B"));
    sink.persist(&a).unwrap();
    sink.persist(&b).unwrap();

    assert!(has_event(&db::space_db_path(root, "A"), &a.id));
    assert!(has_event(&db::space_db_path(root, "B"), &b.id));
    // Cross-check isolation: A's event is not in B's db and vice versa.
    assert!(!has_event(&db::space_db_path(root, "B"), &a.id));
    assert!(!has_event(&db::space_db_path(root, "A"), &b.id));
}

#[test]
fn account_event_routes_to_account_db() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink = PartitionedSink::new(root.to_path_buf());

    let ev = event(EventScope::Account, Some("acct-1"));
    sink.persist(&ev).unwrap();

    assert!(has_event(&db::account_db_path(root), &ev.id));
    assert!(!has_event(&db::runtime_db_path(root), &ev.id));
}

#[test]
fn runtime_event_routes_to_runtime_db() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink = PartitionedSink::new(root.to_path_buf());

    let ev = event(EventScope::Runtime, Some("machine-1"));
    sink.persist(&ev).unwrap();

    assert!(has_event(&db::runtime_db_path(root), &ev.id));
    assert!(!has_event(&db::account_db_path(root), &ev.id));
}

#[test]
fn space_event_without_scope_key_errs() {
    let tmp = tempdir().unwrap();
    let sink = PartitionedSink::new(tmp.path().to_path_buf());

    let ev = event(EventScope::Space, None);
    let err = sink.persist(&ev).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("scope_key"),
        "error should name the missing scope_key, got: {err}"
    );
}

/// Phase 4d §3 proof: persisting through the sink dispatches the FULL
/// projection bus into the scope's OWN db, not just the raw events row. Here
/// the substrate `ActorProjection` materializes the `actors` row in runtime.db
/// when an `actor.created` (Runtime) event is routed — and that row is absent
/// from account.db.
#[test]
fn per_scope_projection_fires_into_scope_db() {
    use crate::db::block_on;

    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink = PartitionedSink::new(root.to_path_buf());

    let payload = serde_json::json!({
        "kind": "test-worker",
        "environment_type": "local",
        "restart_count": 0
    });
    let ev = EventEnvelope::new("actor.created", "actor-xyz", &payload)
        .unwrap()
        .with_actor_id("actor-xyz")
        .with_scope(EventScope::Runtime, Some("machine-1"));
    sink.persist(&ev).unwrap();

    // The actors projection row landed in runtime.db…
    let count_in = |path: &std::path::Path| -> i64 {
        if !path.exists() {
            return 0;
        }
        let mut conn = db::open_db_at(path).unwrap();
        block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM actors WHERE id = ?")
                .bind("actor-xyz")
                .fetch_one(&mut conn)
                .await
        })
        .unwrap_or(0)
    };
    assert_eq!(count_in(&db::runtime_db_path(root)), 1, "actors row must be in runtime.db");
    assert_eq!(count_in(&db::account_db_path(root)), 0, "actors row must NOT be in account.db");
}

/// The kernel emit path never provisions a `platform.db` — the sink routes by
/// scope into the per-scope files only.
#[test]
fn kernel_emit_path_never_creates_platform_db() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink = PartitionedSink::new(root.to_path_buf());
    sink.persist(&event(EventScope::Runtime, Some("m"))).unwrap();
    assert!(
        !db::platform_db_path(root).exists(),
        "PartitionedSink must not create platform.db"
    );
}

#[test]
fn implements_event_sink_trait_as_arc() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink: Arc<dyn EventSink> = Arc::new(PartitionedSink::new(root.to_path_buf()));
    let ev = event(EventScope::Runtime, Some("machine-1"));
    sink.persist(&ev).unwrap();
    assert!(has_event(&db::runtime_db_path(root), &ev.id));
}
