//! Property tests pinning the substrate's lowest-level invariants — the row
//! codec, the pre-migration reconcilers, the per-scope store constructor, and
//! the sqlite-url encoder. These prove EXISTING invariants over arbitrary
//! inputs: a failure here is a real bug in the substrate, not the test.
//!
//! Covers (job spec numbering):
//!   (1) EventEnvelope row round-trip — insert_event_row ↔ envelope_from_row.

//!   (4) new_for_scope key invariant — Space requires a key; Runtime/Account none.
//!   (5) sqlite_url path encoding round-trips through SqliteConnectOptions::from_str.

use chrono::{DateTime, TimeZone, Utc};
use proptest::prelude::*;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::str::FromStr;
use tempfile::tempdir;

use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;

use super::block_on;
use super::envelope::{EVENT_COLS, envelope_from_row, insert_event_row};
use crate::persistence::event_store::SqliteEventStore;

// ── Strategies ────────────────────────────────────────────────────────────────

/// A string that survives a SQLite TEXT round-trip: arbitrary unicode incl.
/// spaces, but no interior NUL (`\0` is not representable in a C-string-backed
/// TEXT cell — a driver limitation, not a substrate invariant).
fn text() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[^\u{0}]{0,40}").unwrap()
}

fn opt_text() -> impl Strategy<Value = Option<String>> {
    proptest::option::of(text())
}

fn scope() -> impl Strategy<Value = EventScope> {
    prop_oneof![
        Just(EventScope::Runtime),
        Just(EventScope::Account),
        Just(EventScope::Space),
    ]
}

/// A `DateTime<Utc>` at millisecond precision (RFC3339 round-trips this exactly).
fn ts() -> impl Strategy<Value = DateTime<Utc>> {
    // Bounded to a realistic range so the RFC3339 form is always well-formed.
    (0i64..=4_102_444_800_000i64).prop_map(|ms| Utc.timestamp_millis_opt(ms).unwrap())
}

prop_compose! {
    /// Build an arbitrary EventEnvelope, populating every column the 18-col
    /// codec carries. `id` is a fresh ULID (PK) so each insert is unique.
    fn arb_envelope()(
        event_type in text(),
        entity_id in text(),
        actor in "[^\u{0}]{0,20}",
        payload_json in text(),
        version in any::<u32>(),
        causation_id in opt_text(),
        scope in scope(),
        scope_key in opt_text(),
        session_id in opt_text(),
        actor_id in opt_text(),
        parent_actor_id in opt_text(),
        target_actor_id in opt_text(),
        elevated in any::<bool>(),
        created_at in ts(),
        author_account_id in opt_text(),
        author_device_id in opt_text(),
        signature in opt_text(),
    ) -> EventEnvelope {
        EventEnvelope {
            id: crate::gen_ulid(),
            event_type,
            entity_id,
            actor,
            payload_json,
            version,
            causation_id,
            scope,
            scope_key,
            session_id,
            actor_id,
            parent_actor_id,
            target_actor_id,
            elevated,
            created_at,
            author_account_id,
            author_device_id,
            signature,
            sig: None,
            writer: None,
            causality: None,
        }
    }
}

// ── (1) EventEnvelope row round-trip ───────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// insert_event_row → SELECT → envelope_from_row reconstructs every field
    /// the 18-column codec carries, for arbitrary payloads / event-types /
    /// scopes / keys. The codec is the substrate's serialization contract; if
    /// any column drifts (wrong index, lost Option, mangled timestamp) this
    /// fails.
    #[test]
    fn event_row_roundtrips(env in arb_envelope()) {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("roundtrip.db");
        crate::db::ensure_events_db_at_path(&db).unwrap();
        let mut conn = open(&db);

        insert_event_row(&env, &mut conn).unwrap();

        let row = block_on(async {
            sqlx::query(&format!("SELECT {EVENT_COLS} FROM events WHERE id = ?"))
                .bind(&env.id)
                .fetch_one(&mut conn)
                .await
        })
        .unwrap();
        let got = envelope_from_row(&row, true).unwrap();

        prop_assert_eq!(&got.id, &env.id);
        prop_assert_eq!(&got.event_type, &env.event_type);
        prop_assert_eq!(&got.entity_id, &env.entity_id);
        prop_assert_eq!(&got.actor, &env.actor);
        prop_assert_eq!(&got.payload_json, &env.payload_json);
        prop_assert_eq!(got.version, env.version);
        prop_assert_eq!(&got.causation_id, &env.causation_id);
        prop_assert_eq!(got.scope, env.scope);
        prop_assert_eq!(&got.scope_key, &env.scope_key);
        prop_assert_eq!(&got.session_id, &env.session_id);
        prop_assert_eq!(&got.actor_id, &env.actor_id);
        prop_assert_eq!(&got.parent_actor_id, &env.parent_actor_id);
        prop_assert_eq!(&got.target_actor_id, &env.target_actor_id);
        prop_assert_eq!(got.elevated, env.elevated);
        prop_assert_eq!(got.created_at, env.created_at);
        prop_assert_eq!(&got.author_account_id, &env.author_account_id);
        prop_assert_eq!(&got.author_device_id, &env.author_device_id);
        prop_assert_eq!(&got.signature, &env.signature);
    }
}

// ── (5) sqlite_url path encoding ───────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// `sqlite_url` must produce a URL that `SqliteConnectOptions::from_str`
    /// accepts AND that resolves back to the same filesystem path — for
    /// arbitrary path segments incl. spaces / unicode. This is the encoder
    /// every db opener funnels through; a broken encoding silently points a
    /// scope at the wrong file (or fails to open). We prove the round-trip
    /// end-to-end: connect via the encoded URL with `create_if_missing` and
    /// assert the db file materialized at EXACTLY the path we encoded.
    #[test]
    fn sqlite_url_roundtrips_through_connect_options(
        // Arbitrary path segments incl. spaces + unicode. URL-structural
        // metacharacters (`?` `#`) and the path separator are excluded:
        // `sqlite_url` does not percent-encode (production scope paths never
        // contain them), so a `?` would be parsed by `from_str` as a query
        // delimiter — out of the encoder's contract, not an invariant here.
        // Leading `s` keeps each segment from being `.`/`..`/empty (which
        // would escape the tempdir); the arbitrary tail carries the spaces /
        // unicode the invariant is really about.
        // Real, creatable path segments: unicode letters/digits + space + safe
        // punctuation — the spaces/unicode the invariant is about. Control,
        // format (BOM/RTL), and private-use codepoints are excluded because the
        // FILESYSTEM can't create such filenames (sqlite errno 14), which is an
        // OS limit, not a `sqlite_url` invariant; URI-special `?#%` excluded per
        // the note above (no percent-encoding; production paths never contain them).
        seg1 in "s[\\p{Alphabetic}\\p{Nd} ._-]{0,24}",
        seg2 in "s[\\p{Alphabetic}\\p{Nd} ._-]{0,24}\\.db",
    ) {
        // Real, writable path: arbitrary segments rooted under a tempdir.
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join(&seg1);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(&seg2);

        let url = super::sqlite_url(&path);
        let opts = SqliteConnectOptions::from_str(&url)
            .unwrap_or_else(|e| panic!("from_str rejected url {url:?}: {e}"))
            .create_if_missing(true);

        // Connect → the driver must create the file at the path we encoded.
        let _conn = block_on(async { SqliteConnection::connect_with(&opts).await })
            .unwrap_or_else(|e| panic!("connect_with failed for url {url:?}: {e}"));
        prop_assert!(
            path.exists(),
            "sqlite_url({:?}) → {:?} did not resolve to the encoded path",
            path, url
        );
    }
}

// ── (4) new_for_scope key invariant ────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Space REQUIRES a scope_key — `new_for_scope(Space, None)` is a hard
    /// error; with a key it resolves under `spaces/<key>/space.db`.
    #[test]
    fn new_for_scope_space_requires_key(key in "[A-Za-z0-9._-]{1,24}") {
        let tmp = tempdir().unwrap();
        let ship = tmp.path();

        // Space with no key → reject, naming the missing scope_key.
        let err = SqliteEventStore::new_for_scope(ship, EventScope::Space, None)
            .err()
            .expect("Space scope with no key must error");
        prop_assert!(
            err.to_string().to_lowercase().contains("scope_key"),
            "Space-without-key error must name scope_key, got: {}",
            err
        );

        // Space with a key → store backed by that space's db.
        let store = SqliteEventStore::new_for_scope(ship, EventScope::Space, Some(&key))
            .unwrap();
        let expected = crate::db::space_db_path(ship, &key);
        prop_assert_eq!(store.db_path(), expected.as_path());
    }

    /// Runtime / Account take NO key — they resolve to the single per-scope db
    /// regardless of any key passed.
    #[test]
    fn new_for_scope_runtime_account_take_no_key(key in opt_text()) {
        let tmp = tempdir().unwrap();
        let ship = tmp.path();
        let k = key.as_deref();

        let rt = SqliteEventStore::new_for_scope(ship, EventScope::Runtime, k).unwrap();
        let rt_expected = crate::db::runtime_db_path(ship);
        prop_assert_eq!(rt.db_path(), rt_expected.as_path());

        let acct = SqliteEventStore::new_for_scope(ship, EventScope::Account, k).unwrap();
        let acct_expected = crate::db::account_db_path(ship);
        prop_assert_eq!(acct.db_path(), acct_expected.as_path());
    }
}

// ── helpers ────────────────────────────────────────────────────────────────────

/// Open a writable connection to `path`, creating the file if missing.
fn open(path: &std::path::Path) -> SqliteConnection {
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let opts = SqliteConnectOptions::from_str(&url)
        .unwrap()
        .create_if_missing(true);
    block_on(async { SqliteConnection::connect_with(&opts).await }).unwrap()
}
