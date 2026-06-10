use crate::db::{block_on, open_db_at, runtime_db_path};
use ship_types::envelope::EventEnvelope;
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
    let ev = EventEnvelope::new(
        event_types::canonical_session::SESSION_STARTED,
        "entity-a",
        &session_started("entity-a"),
    )?
    .with_context(Some("ws-1"), Some("sess-1"));

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
    store.append(&EventEnvelope::new(event_types::WORKSPACE_ACTIVATED, entity, &WorkspaceActivated { agent_id: None, harnesses: vec![] })?)?;
    store.append(&EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, entity, &session_started(entity))?)?;
    store.append(&EventEnvelope::new(event_types::WORKSPACE_ARCHIVED, entity, &WorkspaceArchived {})?)?;

    let ws = store.query(&EventFilter { entity_id: Some(entity.into()), event_type: Some(event_types::WORKSPACE_ACTIVATED.into()), ..Default::default() })?;
    assert_eq!(ws.len(), 1);
    assert_eq!(ws[0].event_type, event_types::WORKSPACE_ACTIVATED);

    let sess = store.query(&EventFilter { entity_id: Some(entity.into()), event_type: Some(event_types::canonical_session::SESSION_STARTED.into()), ..Default::default() })?;
    assert_eq!(sess.len(), 1);
    assert_eq!(sess[0].event_type, event_types::canonical_session::SESSION_STARTED);
    Ok(())
}

#[test]
fn test_filter_by_workspace_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "e1", &session_started("e1"))?
        .with_context(Some("ws-scoped"), None);
    store.append(&ev)?;
    store.append(&EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "e2", &session_started("e2"))?)?;

    let results = store.query(&EventFilter { scope_key: Some("ws-scoped".into()), ..Default::default() })?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].space_id(), Some("ws-scoped"));
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

    rt!(event_types::WORKSPACE_ACTIVATED,   WorkspaceActivated { agent_id: Some("a1".into()), harnesses: vec!["claude".into()] });
    rt!(event_types::WORKSPACE_ARCHIVED,    WorkspaceArchived {});
    rt!(event_types::canonical_session::SESSION_STARTED, session_started("rt-entity"));
    rt!(event_types::canonical_session::SESSION_COMPLETED, TestCompleted { session_id: "rt-entity".into() });
    rt!(event_types::canonical_session::SESSION_FAILED, TestFailed { session_id: "rt-entity".into(), reason: "err".into() });
    rt!(event_types::ACTOR_CREATED,         ActorCreated { kind: "worker".into(), environment_type: "cli".into(), workspace_id: None, parent_actor_id: None, restart_count: 0, ..Default::default() });
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
    let types = event_types::ALL;
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
fn registry_actor_created_is_runtime_ephemeral() {
    let def = ship_types::event_registry::lookup("actor.created")
        .expect("actor.created must be in registry");
    assert_eq!(
        def.allowed_scopes,
        &[ship_types::event_registry::EventScope::Runtime]
    );
    assert_eq!(def.primary_scope(), ship_types::event_registry::EventScope::Runtime);
    assert_eq!(def.retention, ship_types::event_registry::Retention::Ephemeral);
}

#[test]
fn registry_agent_def_updated_is_space_ephemeral() {
    let def = ship_types::event_registry::lookup("agent.def.updated")
        .expect("agent.def.updated must be in registry");
    assert_eq!(
        def.allowed_scopes,
        &[ship_types::event_registry::EventScope::Space],
        "def change notification is Space-scoped (it lives in a space's .ship)"
    );
    assert_eq!(def.primary_scope(), ship_types::event_registry::EventScope::Space);
    assert_eq!(
        def.retention,
        ship_types::event_registry::Retention::Ephemeral,
        "Bucket-2 notification is ephemeral — the file is truth"
    );
}

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
    assert!(count >= 100, "registry must have at least 100 events; got {count}");
}

/// The registry-vs-`ALL` consistency RATCHET — the guard `event_types/mod.rs`
/// points at. Two directions, both pinned to explicit checked-in lists:
///
/// (a) `ALL` minus registry: catches ORPHAN CONSTS — a name added to
///     `event_types::ALL` with no `register_event!` entry fails here. The only
///     justified absences are pinned in `EXPECTED_ALL_ONLY`.
/// (b) registry minus `ALL`: catches REGISTRY-ONLY DRIFT — an event registered
///     in `payload_registry.rs` without an `ALL` entry (or a justified-absence
///     row below) fails here.
///
/// BOTH lists are RATCHETS — they may only SHRINK. Never add a row without a
/// one-line justification; never loosen this into a subset check.
///
/// NOTE on linkage: `all_names()` is inventory-collected at LINK time, so this
/// test sees exactly the L0 registrations (`ship-substrate/payload_registry.rs`).
/// L1 registrations (agent-def `event_registrations.rs`: canonical `session.*`,
/// `op.*`) are NOT linked into this test binary — which is why the canonical
/// session family appears in list (a) even though it IS registered at L1.
#[test]
fn registry_vs_all_ratchet() {
    use std::collections::BTreeSet;

    let registry: BTreeSet<&str> =
        ship_types::event_registry::all_names().into_iter().collect();
    let all: BTreeSet<&str> = ship_types::event_types::ALL.iter().copied().collect();

    // (a) names in `ALL` with no registration linked into this (L0) binary.
    // RATCHET — may only SHRINK.
    const EXPECTED_ALL_ONLY: &[&str] = &[
        // L1 canonical session.* family — registered in agent-def
        // `event_registrations.rs` (not linked at L0); schema ownership pending
        // the keystone schema-ownership decision (Phase 4a, task #32).
        "session.cancelled",
        "session.completed",
        "session.failed",
        "session.started",
        "session.suspended",
        "session.turn.ended",
        "session.turn.started",
    ];
    let all_only: Vec<&str> = all.difference(&registry).copied().collect();
    assert_eq!(
        all_only, EXPECTED_ALL_ONLY,
        "event_types::ALL vs registry drift: every const in ALL must have a \
         register_event! entry (or a justified row in EXPECTED_ALL_ONLY — \
         a RATCHET that may only shrink)"
    );

    // (b) names registered at L0 but absent from `event_types::ALL`.
    // RATCHET — may only SHRINK: registering a new event without adding it to
    // ALL (or here, with a reason) must fail this assert.
    const EXPECTED_REGISTRY_ONLY: &[&str] = &[
        // legacy agent.session.* dispatch family — dual-emit sever + family
        // deletion pending the #26 spawn-intent design; intentionally kept out
        // of ALL so no new subscriber surface grows around it.
        "agent.session.cancelled",
        "agent.session.completed",
        "agent.session.dispatched",
        "agent.session.failed",
        "agent.session.started",
        // collection.* CRUD family moved to L1 (ship-collection keystone A,
        // 2026-06-10): these 5 events are now registered by
        // ship_collection::event_registrations — removed from this ratchet.
        // hook.* family — being converted to canonical op.* (#25b); consts were
        // never added to ALL and the family deregisters when the conversion lands.
        "hook.permission_denied",
        "hook.session_ended",
        "hook.session_started",
        "hook.tool_use_completed",
        "hook.tool_use_started",
        "hook.turn_observed",
        // pipeline gate/step extensions — registered with typed payloads in the
        // substrate `events::pipeline` submodule; no `event_types` consts were
        // ever minted for them (pipelines reshape R13/#27 owns this surface).
        "pipeline.gate.failed",
        "pipeline.gate.passed",
        "pipeline.gate.requested",
        "pipeline.run.blocked",
        "pipeline.step.done_declared",
        "pipeline.step.observed",
        "pipeline.step.output",
        "pipeline.step.started",
        // sync.push.completed — Ephemeral push-success ping; only the failure
        // sibling was threaded into ALL.
        "sync.push.completed",
    ];
    let registry_only: Vec<&str> = registry.difference(&all).copied().collect();
    assert_eq!(
        registry_only, EXPECTED_REGISTRY_ONLY,
        "registry vs event_types::ALL drift: every register_event! entry must \
         be in ALL (or carry a justified row in EXPECTED_REGISTRY_ONLY — a \
         RATCHET that may only shrink)"
    );
}
