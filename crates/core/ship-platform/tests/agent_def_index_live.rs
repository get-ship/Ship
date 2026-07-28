//! Live agent-def index proofs (S5 — agents arc).
//!
//! S1 built `AgentDefIndex` (read-only, DataStore-backed, derived FROM the def
//! files) but left it DEAD: nothing built it at boot and nothing fed it the
//! `agent.def.updated` change notification. S5 closes the consumer half of the
//! producer→index loop. The consumable logic lives in ship-platform
//! (`config::agent_index`), so it is testable WITHOUT the shipd boot wall:
//!   - boot build → every def file on disk has an index row;
//!   - feed an `agent.def.updated` envelope (present file) → the row reflects the
//!     re-read content (new hash);
//!   - feed a delete-marker envelope (file gone) → the row is dropped;
//!   - feed the SAME envelope twice → idempotent (one row, not a duplicate).
//!
//! Every def is authored through the SAME FsBroker door the reader uses (zero raw
//! fs of def content), and the index binds to a per-test SQLite `DataStore`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ship_config::agents_index::{AgentDefIndex, AgentDefQuery};
use ship_config::agents_write::{notify_agent_def_saved, save_agents_config};
use ship_config::types::AgentProfile;
use agent_schema::ProfileMeta;
use ship_platform::config::agent_index::{apply_def_updated, build_def_index, spawn_def_index_consumer};
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{
    DataStore, FsBroker, FsCap, FsMode, FsRoot, KernelLog, LocalFsBroker, SingleBackendDataStore,
    SpaceId as PortSpaceId,
};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::AgentDefUpdated;
use ship_types::SpaceId;

/// Canonical project `.ship` dir under a fresh tempdir (canonicalized for the
/// FsBroker symlink-escape guard, as in the S1 index proofs).
fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    base.join(".ship")
}

/// FsCap rooted at `<ship_dir>/agents` for authoring def files in the test.
fn agents_cap(ship_dir: &Path) -> (FsCap, PathBuf) {
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(PortSpaceId::new("config")),
        FsMode::ReadWrite,
    );
    (cap, dir)
}

/// An index bound to a fresh per-test db.
async fn make_index(ship_dir: &Path) -> AgentDefIndex {
    let db = ship_substrate::db::space_db_path(ship_dir, "idx-test");
    let svc = DataService::open_at(&db).unwrap();
    let store: Arc<dyn DataStore> = Arc::new(SingleBackendDataStore::new(svc, "idx-test"));
    let index = AgentDefIndex::new(store);
    index.ensure_table().await.unwrap();
    index
}

fn write_def(fs: &LocalFsBroker, cap: &FsCap, id: &str, doc: serde_json::Value) {
    fs.write_atomic(cap, &format!("{id}.jsonc"), doc.to_string().as_bytes())
        .unwrap();
}

/// Build the `agent.def.updated` envelope a producer (op write or watcher)
/// emits: `{path, content_hash}` with the rel path as the entity.
fn def_event(rel: &str, content_hash: &str) -> EventEnvelope {
    let payload = AgentDefUpdated { path: rel.into(), content_hash: content_hash.into() };
    EventEnvelope::new(event_types::AGENT_DEF_UPDATED, rel, &payload).unwrap()
}

// ── Test 1: boot build → every def on disk has an index row ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_def_index_populates_rows_for_every_def() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, _) = agents_cap(&dir);

    write_def(&fs, &cap, "alpha", serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }));
    write_def(&fs, &cap, "beta", serde_json::json!({ "agent": { "id": "beta", "name": "Beta" } }));

    let index = make_index(&dir).await;
    build_def_index(&index, &fs, &dir, "project").await.unwrap();

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 2, "boot build indexed both def files");
    assert!(rows.iter().any(|r| r.id == "alpha"));
    assert!(rows.iter().any(|r| r.id == "beta"));
}

// ── Test 2: feed an agent.def.updated → the row reflects the re-read content ───

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn apply_def_updated_upserts_row_on_change() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, _) = agents_cap(&dir);

    write_def(
        &fs,
        &cap,
        "alpha",
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha", "role": "worker" } }),
    );
    let index = make_index(&dir).await;
    apply_def_updated(&index, &fs, &dir, "project", &def_event("alpha.jsonc", "h0"))
        .await
        .unwrap();
    let before = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].name, "Alpha");
    let hash_before = before[0].content_hash.clone();

    // The file is edited on disk, then the change notification is fed: the
    // consumer re-reads the file and upserts the row (the consumer trusts the
    // file, not the payload — the index's content_hash comes from the re-read).
    write_def(
        &fs,
        &cap,
        "alpha",
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha Prime", "role": "lead" } }),
    );
    apply_def_updated(&index, &fs, &dir, "project", &def_event("alpha.jsonc", "h1"))
        .await
        .unwrap();

    let after = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(after.len(), 1, "upsert, not duplicate");
    assert_eq!(after[0].name, "Alpha Prime", "row reflects the re-read content");
    assert_eq!(after[0].role.as_deref(), Some("lead"));
    assert_ne!(after[0].content_hash, hash_before, "edited content → new hash in the row");
}

// ── Test 3: a delete-marker envelope → the row is dropped ──────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn apply_def_updated_drops_row_on_delete_marker() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, agents_dir) = agents_cap(&dir);

    write_def(&fs, &cap, "keep", serde_json::json!({ "agent": { "id": "keep", "name": "Keep" } }));
    write_def(&fs, &cap, "gone", serde_json::json!({ "agent": { "id": "gone", "name": "Gone" } }));
    let index = make_index(&dir).await;
    build_def_index(&index, &fs, &dir, "project").await.unwrap();
    assert_eq!(index.query(AgentDefQuery::default()).await.unwrap().len(), 2);

    // The file is removed, then the delete-marker (empty hash) is fed: the
    // consumer re-reads → file absent → drops the row.
    std::fs::remove_file(agents_dir.join("gone.jsonc")).unwrap();
    apply_def_updated(&index, &fs, &dir, "project", &def_event("gone.jsonc", ""))
        .await
        .unwrap();

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 1, "the deleted def's row is dropped");
    assert_eq!(rows[0].id, "keep");
}

// ── Test 4: double-emit is idempotent (op + watcher both fire) ─────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn apply_def_updated_is_idempotent_under_double_emit() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, _) = agents_cap(&dir);

    write_def(&fs, &cap, "alpha", serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }));
    let index = make_index(&dir).await;

    // The same change notification arrives twice (op-emit then watcher-emit).
    apply_def_updated(&index, &fs, &dir, "project", &def_event("alpha.jsonc", "h0"))
        .await
        .unwrap();
    apply_def_updated(&index, &fs, &dir, "project", &def_event("alpha.jsonc", "h0"))
        .await
        .unwrap();

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 1, "double-emit yields one row, not a duplicate");
    assert_eq!(rows[0].id, "alpha");
}

// ── Test 5: a non-matching event type is ignored ──────────────────────────────
//
// The consumer applies ONLY agent.def.updated; a different event must not touch
// the index (proves the event-type filter the spawned actor relies on).

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn apply_ignores_non_def_events() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, _) = agents_cap(&dir);

    write_def(&fs, &cap, "alpha", serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }));
    let index = make_index(&dir).await;
    build_def_index(&index, &fs, &dir, "project").await.unwrap();

    // A skill.changed event is NOT an agent.def.updated — the consumer ignores it.
    let other = EventEnvelope::new(
        event_types::SKILL_CHANGED,
        "skills",
        &serde_json::json!({ "path": "alpha.jsonc", "content_hash": "x" }),
    )
    .unwrap();
    apply_def_updated(&index, &fs, &dir, "project", &other).await.unwrap();

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 1, "a non-def event leaves the index unchanged");
    assert_eq!(rows[0].id, "alpha");
}

// ── Test 6: end-to-end — gated write → emit → spawned consumer → row appears ───
//
// The full producer→index loop the daemon mounts, exercised WITHOUT the shipd
// boot wall: ONE shared KernelLog carries the gated-write NOTIFY to the spawned
// consumer's broadcast subscription, which re-reads the def and upserts the row.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn write_then_emit_then_spawned_consumer_indexes_the_row() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    std::fs::create_dir_all(&dir).unwrap();
    let fs = LocalFsBroker::new();

    // The index over the space db + the shared KernelLog both producers ride.
    let index = Arc::new(make_index(&dir).await);
    let kernel = Arc::new(KernelLog::new(new_isolated_for_test(dir.clone()).unwrap()));
    // I-2: `PartitionedSink::route()` refuses a Space event whose key has no
    // `spaces` registry row. `notify_agent_def_saved` below emits under
    // "idx-test" and swallows the error (fire-and-forget NOTIFY), so a
    // missing registration here would silently drop the event, not panic.
    ship_substrate::testutil::register_space(&dir, "idx-test").unwrap();

    // Spawn the live consumer subscribed to the shared log.
    let _handle = spawn_def_index_consumer(
        index.clone(),
        kernel.subscribe(),
        dir.clone(),
        "project".to_string(),
    );

    // A gated FsBroker write, then the deterministic post-write NOTIFY through
    // the SAME log the consumer reads.
    save_agents_config(
        &fs,
        &dir,
        &[AgentProfile {
            agent: ProfileMeta { id: "live".into(), name: "Live".into(), ..Default::default() },
            ..Default::default()
        }],
    )
    .unwrap();
    notify_agent_def_saved(&kernel, &fs, &dir, "live", Some(SpaceId::new("idx-test"))).await;

    // The consumer applies asynchronously — poll briefly for the row to land.
    let mut found = false;
    for _ in 0..40 {
        let rows = index.query(AgentDefQuery::default()).await.unwrap();
        if rows.iter().any(|r| r.id == "live") {
            found = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    assert!(found, "the spawned consumer indexed the def after the gated write emit");
}
