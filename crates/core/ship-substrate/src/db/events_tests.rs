//! Tests for db::events — typed event queries, time filtering.

use super::events::{
    list_all_events, list_events_since_time, list_recent_events, query_events_since,
};
use crate::events::types::SpaceSeen;
use crate::events::types::event_types;
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::testutil::init_project_minimal as init_project;
use chrono::Utc;
use ship_types::envelope::EventEnvelope;
use ship_types::Stamped;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();
    (tmp, ship_dir)
}

/// Seed events into the Runtime event log (runtime.db) — the db the typed
/// queries (`list_all_events` etc.) read after the Phase-4d partition.
fn runtime_store() -> SqliteEventStore {
    SqliteEventStore::new_for_scope(
        &crate::project::get_global_dir().unwrap(),
        ship_types::event_registry::EventScope::Runtime,
        None,
    )
    .unwrap()
}

fn make_seen_event(seen_at: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::SPACE_SEEN,
        "space",
        &SpaceSeen {
            seen_at: seen_at.to_string(),
        },
    )
    .unwrap()
}

#[test]
fn list_all_events_starts_empty() {
    let (_tmp, _ship_dir) = setup();
    let all = list_all_events().unwrap();
    assert!(all.is_empty());
}

#[test]
fn append_and_list_all() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    store.append(&Stamped::unchecked(make_seen_event("first"))).unwrap();
    store.append(&Stamped::unchecked(make_seen_event("second"))).unwrap();

    let all = list_all_events().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].event_type, event_types::SPACE_SEEN);
    assert_eq!(all[1].event_type, event_types::SPACE_SEEN);
}

#[test]
fn list_since_filters_by_time() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    store.append(&Stamped::unchecked(make_seen_event("before"))).unwrap();

    let future = Utc::now() + chrono::Duration::hours(1);
    let filtered = list_events_since_time(&future, None).unwrap();
    assert!(filtered.is_empty());

    let past = Utc::now() - chrono::Duration::hours(1);
    let all = list_events_since_time(&past, None).unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn list_since_with_limit() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    let past = Utc::now() - chrono::Duration::hours(1);
    for i in 0..5 {
        store
            .append(&Stamped::unchecked(make_seen_event(&format!("event-{}", i))))
            .unwrap();
    }
    let limited = list_events_since_time(&past, Some(3)).unwrap();
    assert_eq!(limited.len(), 3);
}

#[test]
fn list_recent_returns_in_asc_order() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    let mut inserted_ids = Vec::new();
    for i in 0..5 {
        let ev = make_seen_event(&format!("event-{}", i));
        inserted_ids.push(ev.id.clone());
        store.append(&Stamped::unchecked(ev.clone())).unwrap();
    }
    // Ask for last 3, returned in ascending (oldest-first) order
    let recent = list_recent_events(3).unwrap();
    assert_eq!(recent.len(), 3);
    // The returned slice must be sorted ascending by id
    let ids: Vec<&str> = recent.iter().map(|e| e.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(
        ids, sorted,
        "list_recent_events must return events in ASC order"
    );
}

#[test]
fn query_events_since_no_cursor_returns_all() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    store.append(&Stamped::unchecked(make_seen_event("a"))).unwrap();
    store.append(&Stamped::unchecked(make_seen_event("b"))).unwrap();
    let all = query_events_since(None, false).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn query_events_since_cursor_returns_later_events() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    let ev1 = make_seen_event("first");
    let cursor = ev1.id.clone();
    store.append(&Stamped::unchecked(ev1.clone())).unwrap();
    store.append(&Stamped::unchecked(make_seen_event("second"))).unwrap();
    store.append(&Stamped::unchecked(make_seen_event("third"))).unwrap();
    let after = query_events_since(Some(&cursor), false).unwrap();
    assert_eq!(after.len(), 2);
}

#[test]
fn query_events_since_latest_cursor_returns_empty() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    let ev = make_seen_event("only");
    let cursor = ev.id.clone();
    store.append(&Stamped::unchecked(ev.clone())).unwrap();
    let after = query_events_since(Some(&cursor), false).unwrap();
    assert!(after.is_empty());
}

#[test]
fn query_events_since_elevated_only() {
    use crate::events::types::{SpaceOpened, event_types as et};
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();
    // Non-elevated event
    store.append(&Stamped::unchecked(make_seen_event("noise"))).unwrap();
    // Elevated event
    let elevated = EventEnvelope::new(
        et::SPACE_OPENED,
        "ws-1",
        &SpaceOpened {
            path: "/test/path".into(),
            name: "test".into(),
            opened_at: "2026-01-01T00:00:00Z".into(),
        },
    )
    .unwrap()
    .elevate();
    store.append(&Stamped::unchecked(elevated.clone())).unwrap();
    let all = query_events_since(None, false).unwrap();
    assert_eq!(all.len(), 2);
    let elevated_only = query_events_since(None, true).unwrap();
    assert_eq!(elevated_only.len(), 1);
    assert_eq!(elevated_only[0].event_type, et::SPACE_OPENED);
}
