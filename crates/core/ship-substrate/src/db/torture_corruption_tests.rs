//! T1 storage-torture, scenario 4: corruption -> heal (exploratory scaffold).
//!
//! Flips real bytes at a table's own SQLite b-tree ROOT PAGE (located by
//! reading `sqlite_master.rootpage` + `PRAGMA page_size` while the db is
//! still open, then closing it) to produce genuine, targeted, deterministic
//! corruption — not a blind offset guess. `PRAGMA wal_checkpoint(TRUNCATE)`
//! runs first so the main db file (not the `-wal` side file) actually holds
//! the byte we're about to flip.

use crate::db::{self, block_on, integrity, rebuild};
use crate::events::EventScope;
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::testutil::init_project_minimal as init_project;
use ship_types::envelope::EventEnvelope;
use ship_types::Stamped;
use ship_types::event_types as ev;
use tempfile::tempdir;

fn actor_created_event(id: &str) -> EventEnvelope {
    EventEnvelope::new(
        ev::ACTOR_CREATED,
        id,
        &serde_json::json!({ "kind": "local", "environment_type": "local" }),
    )
    .unwrap()
    .with_actor_id(id)
    .with_scope(EventScope::Runtime, None)
}

fn rootpage_and_page_size(db_path: &std::path::Path, table: &str) -> (i64, i64) {
    let mut conn = db::open_db_at(db_path).unwrap();
    // Flush WAL frames into the main file so the byte offset we compute
    // against `rootpage` matches what's actually on disk once we close and
    // read the raw file.
    block_on(async {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&mut conn)
            .await
    })
    .unwrap();
    let rootpage: i64 = block_on(async {
        sqlx::query_scalar("SELECT rootpage FROM sqlite_master WHERE name = ?")
            .bind(table)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    let page_size: i64 = block_on(async {
        sqlx::query_scalar("PRAGMA page_size")
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    (rootpage, page_size)
}

/// Flip the leading b-tree page-header bytes of `table`'s root page —
/// structural corruption real enough for `PRAGMA integrity_check` to catch,
/// confined (as far as SQLite's on-disk format allows aiming) to that one
/// table's own page rather than shared schema (`sqlite_master`, page 1) or
/// another table's pages.
fn corrupt_table_root_page(db_path: &std::path::Path, table: &str) {
    let (rootpage, page_size) = rootpage_and_page_size(db_path, table);
    assert!(rootpage >= 1, "table {table} must have a root page");
    let offset = ((rootpage - 1) * page_size) as usize;

    let mut data = std::fs::read(db_path).unwrap();
    assert!(
        data.len() >= offset + 16,
        "file too short to reach {table}'s root page at offset {offset}"
    );
    data[offset] ^= 0xFF;
    std::fs::write(db_path, &data).unwrap();
}

fn setup_with_events(
    n: usize,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    Vec<String>,
) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();
    let db_path = db::runtime_db_path(&ship_dir);
    let store = SqliteEventStore::new_for_scope(&ship_dir, EventScope::Runtime, None).unwrap();
    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        let id = format!("torture-corrupt-{i}");
        store.append(&Stamped::unchecked(actor_created_event(&id))).unwrap();
        ids.push(id);
    }
    (tmp, ship_dir, db_path, ids)
}

/// Corrupting the `actors` PROJECTION table's own root page: `check_integrity`
/// must detect it (never a false-healthy `Ok`), and classify it as
/// recoverable (`ProjectionsCorrupt`) since the log (`events`) is untouched.
#[test]
fn corrupted_projection_root_page_is_detected_not_false_healthy() {
    let (_tmp, _ship_dir, db_path, _ids) = setup_with_events(5);

    corrupt_table_root_page(&db_path, "actors");

    match integrity::check_integrity(&db_path) {
        Ok(integrity::IntegrityStatus::Ok) => {
            panic!("corrupted actors root page must NOT report Ok (false-healthy)");
        }
        Ok(integrity::IntegrityStatus::ProjectionsCorrupt { details })
        | Ok(integrity::IntegrityStatus::EventsCorrupt { details }) => {
            assert!(!details.is_empty());
        }
        Err(_) => {
            // A hard Err (PRAGMA integrity_check itself failing to execute
            // against a structurally-invalid page, rather than returning
            // corruption-description rows) is also "detected, not silently
            // healed" — it never claims Ok. See the module doc: this is the
            // observed real-world severity band for a raw page-header flip.
        }
    }
}

/// Corrupting the `events` LOG's own root page must be reported as fatal
/// (`EventsCorrupt`), never silently classified as the recoverable
/// `ProjectionsCorrupt` — the log itself cannot be replayed if it's the
/// thing that's broken.
#[test]
fn corrupted_log_root_page_is_reported_fatal_not_recoverable() {
    let (_tmp, _ship_dir, db_path, _ids) = setup_with_events(5);

    corrupt_table_root_page(&db_path, "events");

    match integrity::check_integrity(&db_path) {
        Ok(integrity::IntegrityStatus::Ok) => {
            panic!("corrupted events root page must NOT report Ok (false-healthy)");
        }
        Ok(integrity::IntegrityStatus::ProjectionsCorrupt { details }) => {
            panic!("log corruption misclassified as recoverable ProjectionsCorrupt: {details}");
        }
        Ok(integrity::IntegrityStatus::EventsCorrupt { details }) => {
            assert!(!details.is_empty());
        }
        Err(_) => {
            // Fatal, just surfaced as Err rather than the EventsCorrupt
            // variant — never falsely healthy either way.
        }
    }
}

/// The heal path: when corruption hits ONLY the projection table (the log
/// survives), `rebuild_scope_db` must restore the master invariant —
/// `actors` reconstructed to exactly fold(events log) — and a follow-up
/// integrity check must come back clean. `ActorProjection::truncate` clears
/// the table via `db::heal::truncate_or_heal`, which never needs to read the
/// table's own (possibly-corrupted) pages: on a plain `DELETE FROM actors`
/// failure it captures the table's schema from `sqlite_master`, drops the
/// schema entry + `VACUUM`s to reclaim the damaged pages, then recreates the
/// empty table from the captured DDL for replay to refill.
#[test]
fn heal_restores_master_invariant_when_only_projection_corrupted() {
    let (_tmp, ship_dir, db_path, ids) = setup_with_events(5);

    corrupt_table_root_page(&db_path, "actors");

    // Precondition: confirm the check at least detected SOMETHING (not a
    // false-healthy Ok) before judging the heal path.
    match integrity::check_integrity(&db_path) {
        Ok(integrity::IntegrityStatus::Ok) => {
            panic!("precondition failed: corruption not detected at all")
        }
        _ => {}
    }

    let rebuild_result = rebuild::rebuild_projections(&ship_dir);

    match rebuild_result {
        Ok(report) => {
            assert_eq!(report.events_replayed, ids.len());
            // Master invariant: every id we appended must be present, and
            // ONLY those ids.
            let mut conn = db::open_db_at(&db_path).unwrap();
            let count: i64 = block_on(async {
                sqlx::query_scalar("SELECT COUNT(*) FROM actors")
                    .fetch_one(&mut conn)
                    .await
            })
            .unwrap();
            assert_eq!(
                count as usize,
                ids.len(),
                "rebuild must restore exactly fold(log), no more no less"
            );

            match integrity::check_integrity(&db_path).unwrap() {
                integrity::IntegrityStatus::Ok => {}
                other => panic!("post-heal integrity check must be Ok, got {other:?}"),
            }
        }
        Err(e) => {
            // FINDING (see job report): `ActorProjection::truncate` runs
            // `DELETE FROM actors`, which must walk the same structurally-
            // damaged root page the corruption hit. The heal path cannot
            // execute — the db is left permanently corrupted rather than
            // restored, for exactly the corruption shape (a live table's own
            // root page, not a wholesale-missing table) a byte-level
            // corruption realistically produces.
            panic!(
                "FINDING: rebuild_projections cannot execute the heal when the projection table's \
                 OWN root page (not just its absence) is corrupted — {e}"
            );
        }
    }
}

/// The unrecoverable case: corruption hit the LOG itself. `rebuild` reading
/// through a corrupted `events` table must not silently produce a healed-
/// looking (but wrong) state — it must fail loud.
#[test]
fn unrecoverable_log_corruption_is_not_silently_healed() {
    let (_tmp, ship_dir, db_path, _ids) = setup_with_events(5);

    corrupt_table_root_page(&db_path, "events");

    match integrity::check_integrity(&db_path) {
        Ok(integrity::IntegrityStatus::EventsCorrupt { .. }) => {}
        Err(_) => {
            // Fatal, surfaced as Err rather than the EventsCorrupt variant —
            // still "detected", never falsely healthy.
        }
        other => panic!(
            "expected the log corruption to be classified EventsCorrupt (or a hard Err), got {other:?}"
        ),
    }

    // Attempting a rebuild against a corrupted log must error, not produce
    // a plausible-looking-but-wrong `actors` table.
    let result = rebuild::rebuild_projections(&ship_dir);
    assert!(
        result.is_err(),
        "rebuild must fail loud when the log itself (not just a projection) is corrupted — \
         silently 'succeeding' here would mean healing to a WRONG state"
    );
}
