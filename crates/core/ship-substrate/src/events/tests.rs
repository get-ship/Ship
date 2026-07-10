use crate::db::{block_on, open_db_at, runtime_db_path};
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use crate::events::filter::EventFilter;
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::events::types::*;
use crate::events::types::event_types;
use crate::testutil::init_project_minimal as init_project;
use tempfile::tempdir;
use serde::{Serialize, Deserialize};

// Local test stand-ins for canonical session types (moved to agent-def)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSession {
    session_id: String,
    agent_id: String,
    harness: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    space_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestTurn {
    session_id: String,
    idx: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestCompleted {
    session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestFailed {
    session_id: String,
    reason: String,
}

fn setup() -> (tempfile::TempDir, SqliteEventStore) {
    let tmp = tempdir().unwrap();
    let ship = init_project(tmp.path()).unwrap();
    let store = SqliteEventStore::new_at_db_path(&runtime_db_path(&ship)).unwrap();
    (tmp, store)
}

fn session_started(session_id: &str) -> TestSession {
    TestSession {
        session_id: session_id.into(),
        agent_id: "agent-a".into(),
        harness: "claude".into(),
        model: None,
        space_id: "ws-1".into(),
        input: None,
        parent: None,
    }
}

#[test]
fn test_round_trip() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let mut ev = EventEnvelope::new(
        event_types::canonical_session::SESSION_STARTED,
        "entity-a",
        &session_started("entity-a"),
    )?
    .with_scope(EventScope::Space, Some("ws-1"));
    ev.session_id = Some("sess-1".to_string());

    store.append(&ev)?;
    let got = store.get(&ev.id)?.expect("event not found");

    assert_eq!(got.id, ev.id);
    assert_eq!(got.event_type, ev.event_type);
    assert_eq!(got.entity_id, ev.entity_id);
    assert_eq!(got.actor, ev.actor);
    assert_eq!(got.payload_json, ev.payload_json);
    assert_eq!(got.version, ev.version);
    assert_eq!(got.scope, ev.scope);
    assert_eq!(got.scope_key, ev.scope_key);
    assert_eq!(got.session_id, ev.session_id);
    Ok(())
}

#[test]
fn test_ordering() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "ordering-entity";
    for i in 0..100 {
        let ev = EventEnvelope::new(
            event_types::canonical_session::TURN_STARTED,
            entity,
            &TestTurn { session_id: entity.into(), idx: i },
        )?;
        store.append(&ev)?;
    }
    let results = store.query_aggregate(entity)?;
    assert_eq!(results.len(), 100);
    let ids: Vec<&str> = results.iter().map(|e| e.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "events must be in ULID (time) order");
    Ok(())
}

#[test]
fn test_filter_by_entity_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev_a = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "entity-a", &session_started("entity-a"))?;
    let ev_b = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "entity-b", &session_started("entity-b"))?;
    store.append(&ev_a)?;
    store.append(&ev_b)?;

    let results = store.query_aggregate("entity-a")?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id, "entity-a");
    Ok(())
}

#[test]
fn test_filter_by_event_type() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "filter-type-entity";
    store.append(&EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, entity, &session_started(entity))?)?;
    store.append(&EventEnvelope::new(event_types::canonical_session::SESSION_COMPLETED, entity, &TestCompleted { session_id: entity.into() })?)?;

    let sess = store.query(&EventFilter { entity_id: Some(entity.into()), event_type: Some(event_types::canonical_session::SESSION_STARTED.into()), ..Default::default() })?;
    assert_eq!(sess.len(), 1);
    assert_eq!(sess[0].event_type, event_types::canonical_session::SESSION_STARTED);
    Ok(())
}


#[test]
fn test_causation_chain() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev_a = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "cause-entity", &session_started("cause-entity"))?;
    store.append(&ev_a)?;
    let ev_b = EventEnvelope::new(event_types::canonical_session::SESSION_COMPLETED, "cause-entity", &TestCompleted { session_id: "cause-entity".into() })?
        .with_causation(&ev_a.id);
    store.append(&ev_b)?;

    let got_b = store.get(&ev_b.id)?.unwrap();
    assert_eq!(got_b.causation_id.as_deref(), Some(ev_a.id.as_str()));
    Ok(())
}

#[test]
fn test_immutability() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "immut-entity", &session_started("immut-entity"))?;
    store.append(&ev)?;

    let db_p = store.db_path().to_path_buf();
    let mut conn = open_db_at(&db_p)?;
    let result = block_on(async {
        sqlx::query("UPDATE events SET actor = 'hacked' WHERE id = ?")
            .bind(&ev.id)
            .execute(&mut conn)
            .await
    });
    assert!(result.is_err(), "immutability trigger must block UPDATE");
    Ok(())
}

#[test]
fn test_payload_round_trip() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    macro_rules! rt {
        ($etype:expr, $payload:expr) => {{
            let ev = EventEnvelope::new($etype, "rt-entity", &$payload)?;
            store.append(&ev)?;
            let got = store.get(&ev.id)?.expect("not found");
            assert_eq!(got.payload_json, ev.payload_json, "payload mismatch: {}", $etype);
        }};
    }

    rt!(event_types::canonical_session::SESSION_STARTED, session_started("rt-entity"));
    rt!(event_types::canonical_session::SESSION_COMPLETED, TestCompleted { session_id: "rt-entity".into() });
    rt!(event_types::canonical_session::SESSION_FAILED, TestFailed { session_id: "rt-entity".into(), reason: "err".into() });
    rt!(event_types::ACTOR_CREATED,         ActorCreated { kind: "worker".into(), environment_type: "cli".into(), parent_actor_id: None, restart_count: 0, ..Default::default() });
    rt!(event_types::ACTOR_WOKE,            ActorWoke {});
    rt!(event_types::ACTOR_SLEPT,           ActorSlept { idle_secs: 45 });
    rt!(event_types::ACTOR_STOPPED,         ActorStopped { reason: "shutdown".into(), ..Default::default() });
    rt!(event_types::ACTOR_CRASHED,         ActorCrashed { error: "OOM".into(), restart_count: 2 });

    Ok(())
}

#[test]
fn test_concurrent_appends() -> anyhow::Result<()> {
    use std::sync::Arc;

    let (_tmp, store) = setup();
    let store = Arc::new(store);
    let entity = "concurrent-entity";
    let mut handles = Vec::new();

    for i in 0..10 {
        let store = Arc::clone(&store);
        let entity = entity.to_string();
        handles.push(std::thread::spawn(move || {
            let ev = EventEnvelope::new(
                event_types::canonical_session::TURN_STARTED,
                &entity,
                &TestTurn { session_id: entity.clone(), idx: i },
            )
            .expect("create envelope");
            store.append(&ev).expect("append");
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    let results = store.query_aggregate(entity)?;
    assert_eq!(results.len(), 10, "all 10 appends must succeed");

    let ids: std::collections::HashSet<_> = results.iter().map(|e| &e.id).collect();
    assert_eq!(ids.len(), 10, "no duplicate event IDs");
    Ok(())
}

#[test]
fn test_empty_queries() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let results = store.query_aggregate("no-such-entity")?;
    assert!(results.is_empty());
    let results = store.query(&EventFilter { event_type: Some("nonexistent.type".into()), ..Default::default() })?;
    assert!(results.is_empty());
    Ok(())
}

#[test]
fn test_version_field() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    let ev = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "ver-entity", &session_started("ver-entity"))?;
    assert_eq!(ev.version, 1);
    store.append(&ev)?;
    let got = store.get(&ev.id)?.unwrap();
    assert_eq!(got.version, 1);

    let mut ev2 = EventEnvelope::new(event_types::canonical_session::SESSION_COMPLETED, "ver-entity", &TestCompleted { session_id: "ver-entity".into() })?;
    ev2.version = 2;
    store.append(&ev2)?;
    let got2 = store.get(&ev2.id)?.unwrap();
    assert_eq!(got2.version, 2);
    Ok(())
}

#[test]
fn test_property_event_ordering() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "prop-entity";
    let names = ship_types::event_registry::all_names();
    let types: &[&str] = &names;
    let n = 75;
    let mut expected_ids = Vec::with_capacity(n);

    for i in 0..n {
        let etype = types[i % types.len()];
        let ev = EventEnvelope::new(etype, entity, &TestTurn { session_id: entity.into(), idx: i as u32 })?;
        expected_ids.push(ev.id.clone());
        store.append(&ev)?;
    }

    let results = store.query_aggregate(entity)?;

    assert_eq!(results.len(), n, "all events must be stored");

    let ids: Vec<&str> = results.iter().map(|e| e.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "results must be in ULID order");

    let result_set: std::collections::HashSet<_> = results.iter().map(|e| &e.id).collect();
    assert_eq!(result_set.len(), n, "no duplicate events");
    for id in &expected_ids {
        assert!(result_set.contains(id), "event {id} is missing");
    }
    Ok(())
}

#[test]
fn registry_actor_created_is_runtime_durable() {
    // Reclassified Durable: `actor.created` is the BIRTH of the actors
    // projection row (see ActorProjection) — losing it on daemon restart
    // would leave the actors table empty. Job/transient-events spike
    // (`.artifacts/transient-events-design.md`) is the call site; the
    // activity ticks (`actor.slept`/`woke`/`stopped`) keep Ephemeral.
    let def = ship_types::event_registry::lookup("actor.created")
        .expect("actor.created must be in registry");
    assert_eq!(
        def.allowed_scopes,
        &[ship_types::event_registry::EventScope::Runtime]
    );
    assert_eq!(def.primary_scope(), ship_types::event_registry::EventScope::Runtime);
    assert_eq!(def.retention, ship_types::event_registry::Retention::Durable);
}

// `registry_agent_def_updated_is_space_ephemeral` moved to agent-def
// tests/event_scope_discipline (keystone-A Phase A): the registration it
// asserts on now lives there and is not linked into this L0 binary.

#[test]
fn registry_names_are_unique() {
    let names = ship_types::event_registry::all_names();
    let mut set = std::collections::HashSet::new();
    for name in &names {
        assert!(set.insert(*name), "duplicate event name in registry: {name}");
    }
}

#[test]
fn registry_has_entries() {
    let count = ship_types::event_registry::all_names().len();
    // Floor lowered as keystone-A de-buckets domains to L1 crates. The exact
    // L0 count is pinned by `l0_registry_shrinks_under_keystone_a`; this
    // floor is just a smoke check that the binary isn't empty.
    // Keep WELL below the pin so domain de-buckets never trip the smoke
    // check — only an inventory/linker catastrophe should.
    assert!(count >= 40, "registry must have at least 40 events; got {count}");
}

/// L0 registration ratchet — the successor of the retired `ALL`-mirror test.
///
/// The vocabulary's source of truth is the inventory registry; in THIS (L0)
/// test binary `all_names()` sees exactly ship-substrate's own
/// `payload_registry.rs` registrations. Under keystone A (domain-owned events
/// at L1, ratified 2026-06-10) this set may only SHRINK: de-bucketing a domain
/// moves its `register_event!` lines to the owning L1 crate (agents and
/// collection are done). Bump the pin DOWN with the move that shrinks it;
/// growing it means new domain events were registered at L0 — stop and put
/// them in the owning domain crate instead (genuinely substrate-generic
/// events are the rare sanctioned exception).
///
/// Full-linkage vocabulary guards (every domain linked, NO exception lists)
/// live in shipd `tests/event_vocabulary.rs`.
#[test]
fn l0_registry_shrinks_under_keystone_a() {
    let n = ship_types::event_registry::all_names().len();
    assert_eq!(
        n, 41,
        "L0-registered event count changed. This pin is a keystone-A RATCHET: \
         it should only move DOWN (a domain de-bucket) — update it with the \
         move that shrinks it; growing it means new domain events were \
         registered at L0 — stop and put them in the owning domain crate \
         instead (genuinely substrate-generic events are the rare \
         sanctioned exception). \
         (Facade/-core fold, 2026-07: 44 -> 41. This crate's OWN registrations \
         are unchanged; the isolated lib-test links 3 fewer dep registrations \
         after ship-substrate-core merged into ship-substrate. shipd \
         tests/event_vocabulary.rs — the full-linkage guard — stays green, so \
         nothing was lost globally.)"
    );
}

/// Legacy families pinned until their sanctioned severs land. Lists may only
/// SHRINK (to empty), at which point delete the corresponding block.
#[test]
fn legacy_families_pinned_until_sever() {
    use std::collections::BTreeSet;
    let registry: BTreeSet<&str> =
        ship_types::event_registry::all_names().into_iter().collect();

    // hook.* — SEVERED (#25b, 2026-06-10): all 6 hook.* events deregistered.
    // Emitters converted to op.* (tool-use) and session.turn.ended (turns).
    // This block is intentionally empty; kept as a tombstone comment.

    // agent.session.* — SEVERED (#26, 2026-06-10): all 5 agent.session.*
    // events deregistered. Dispatch primitive is now `dispatch.requested`
    // (Runtime/Ephemeral, registered at L1 in agent-def). The payload structs
    // and the `events::agent_session` module are deleted (zero deserializer
    // call sites); `InvocationMode` lives in agent-def (L1).
    // This block is intentionally empty; kept as a tombstone comment.
    let _ = &registry; // suppress unused-variable warning when all blocks are tombstones
}
