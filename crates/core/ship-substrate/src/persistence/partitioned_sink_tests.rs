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

// ── Transient-signal mechanism (job/transient-events) ────────────────────────
//
// The kernel's `KernelRouter::route` consults the event registry's
// `Retention` and SKIPS the sink for `Retention::Ephemeral` events —
// delivering them to mailboxes only. `Retention::Durable` keeps the
// existing persist+deliver behavior. See
// `.artifacts/transient-events-design.md` for the rationale.
//
// `actor.slept` is a real registered event (Runtime + Ephemeral);
// `actor.crashed` is Runtime + Durable. The pair proves the skip is
// driven by the registry lookup, not by the event shape, scope, or
// whether the sink is wired. (`actor.created` would have been the
// obvious pick but is now Durable — it's the projection's seed event.)

fn retention_test_event(event_type: &str, entity: &str, payload: serde_json::Value) -> EventEnvelope {
    use ship_types::event_registry::EventScope;
    EventEnvelope::new(event_type, entity, &payload)
        .unwrap()
        .with_actor_id(entity)
        .with_scope(EventScope::Runtime, Some("machine-1"))
}

fn actor_slept_payload() -> serde_json::Value {
    serde_json::json!({ "idle_secs": 1u64 })
}

/// Route an Ephemeral event through the kernel → the events table stays
/// empty for that id, but the mailbox receives the envelope. This is the
/// contract the `config.file_changed` watcher ping (Runtime/Ephemeral)
/// relies on.
#[tokio::test(flavor = "multi_thread")]
async fn kernel_router_skips_persist_for_ephemeral_event() {
    use ship_kernel::KernelRouter;
    use ship_types::event_registry::lookup;

    // Sanity: the event is registered as Ephemeral in this build.
    let def = lookup("actor.slept").expect("actor.slept must be registered");
    assert_eq!(
        def.retention,
        ship_types::event_registry::Retention::Ephemeral,
        "test fixture invariant: actor.slept is registered as Ephemeral"
    );

    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink: Arc<dyn EventSink> = Arc::new(PartitionedSink::new(root.to_path_buf()));
    let mut router = KernelRouter::new(sink);

    // Subscribe an actor on the "actor." namespace so the mailbox fires.
    let cfg = ship_types::ActorConfig {
        namespace: "actor".into(),
        write_namespaces: vec!["actor.".into()],
        read_namespaces: vec!["actor.".into()],
        subscribe_namespaces: vec!["actor.".into()],
        see_directed_events: false,
    };
    let mut mailbox = router.spawn_actor("listener", cfg).expect("spawn");

    let ev = retention_test_event("actor.slept", "actor-skip", actor_slept_payload());
    let id = ev.id.clone();
    router.route(ev).await.expect("route");

    // Ephemeral: the events table does NOT carry the id.
    assert!(
        !has_event(&db::runtime_db_path(root), &id),
        "Ephemeral event must NOT be persisted; runtime.db has no row for {id}"
    );

    // Mailbox DID receive the envelope — transient signals are delivered,
    // just not persisted.
    let delivered = mailbox.recv().await.expect("mailbox receives the Ephemeral event");
    assert_eq!(delivered.event_type, "actor.slept");
    assert_eq!(delivered.id, id);
 }

/// Route a Durable event through the kernel → the events table HAS the
/// row, and the mailbox also receives. Pair to the Ephemeral case proves
/// the skip is retention-driven, not a blanket no-op.
#[tokio::test(flavor = "multi_thread")]
async fn kernel_router_persists_durable_event_and_delivers() {
    use ship_kernel::KernelRouter;
    use ship_types::event_registry::lookup;

    let def = lookup("actor.crashed").expect("actor.crashed must be registered");
    assert_eq!(
        def.retention,
        ship_types::event_registry::Retention::Durable,
        "test fixture invariant: actor.crashed is registered as Durable"
    );

    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let sink: Arc<dyn EventSink> = Arc::new(PartitionedSink::new(root.to_path_buf()));
    let mut router = KernelRouter::new(sink);

    let cfg = ship_types::ActorConfig {
        namespace: "actor".into(),
        write_namespaces: vec!["actor.".into()],
        read_namespaces: vec!["actor.".into()],
        subscribe_namespaces: vec!["actor.".into()],
        see_directed_events: false,
    };
    let mut mailbox = router.spawn_actor("listener", cfg).expect("spawn");

    let ev = retention_test_event(
        "actor.crashed",
        "actor-persist",
        serde_json::json!({ "error": "boom", "restart_count": 0u32 }),
    );
    let id = ev.id.clone();
    router.route(ev).await.expect("route");

    assert!(
        has_event(&db::runtime_db_path(root), &id),
        "Durable event must be persisted to runtime.db"
    );
    let delivered = mailbox.recv().await.expect("mailbox receives the Durable event");
    assert_eq!(delivered.event_type, "actor.crashed");
 }
