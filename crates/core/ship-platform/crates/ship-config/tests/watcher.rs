//! Watcher authorship proofs — ported from the pre-carve `config/watcher.rs`
//! keystone test. Drives the authoring path DIRECTLY with a synthetic signal
//! (no inotify, not flaky), and proves scope routing + the FsBroker eviction of
//! the content-hash read.

use std::path::Path;
use std::sync::Arc;

use ship_config::watcher::{author_agent_def_updated, author_change, author_file_changed};
use ship_substrate::db::ensure_db_at;
use ship_substrate::events::EventFilter;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::ports::{
    FsBroker, FsCap, FsMode, FsRoot, KernelLog, LocalFsBroker, SpaceId as PortSpaceId,
};
use ship_types::SpaceId;
use ship_types::event_types;

async fn setup() -> (tempfile::TempDir, std::path::PathBuf, Arc<KernelLog>) {
    let tmp = tempfile::tempdir().unwrap();
    // Canonicalize: `/tmp` is symlinked on WSL/macOS; the FsBroker's symlink-
    // escape guard canonicalizes the deepest existing ancestor and would reject
    // a non-canonical cap root. Production paths are already canonical.
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    let ship_dir = ShipHome::project_ship_dir(&base);
    std::fs::create_dir_all(&ship_dir).unwrap();
    ensure_db_at(&ship_dir).unwrap();
    let kernel = Arc::new(KernelLog::new(new_isolated_for_test(ship_dir.clone()).unwrap()));
    (tmp, ship_dir, kernel)
}

fn count_events(db_path: &Path, event_type: &str) -> usize {
    let store = SqliteEventStore::new_at_db_path(db_path).unwrap();
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            limit: Some(50),
            ..Default::default()
        })
        .unwrap()
        .len()
}

// Keystone: the config service (not the OS watcher) authors config events with
// the IDENTICAL event_type / subject / entity_id the watcher used to emit.
#[tokio::test(flavor = "multi_thread")]
async fn config_service_authors_domain_event_with_identical_shape() {
    let (_tmp, ship_dir, kernel) = setup().await;

    // Coarse domain event — same event_type (AGENT_CHANGED) + entity_id
    // (subject "agents"). agent.changed is Space-scoped → routes to the bound
    // space's db (sp-1).
    author_change(&kernel, event_types::AGENT_CHANGED, "agents", Some(SpaceId::new("sp-1"))).await;

    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::space_db_path(&ship_dir, "sp-1"))
            .unwrap();
    let evs = store
        .query(&EventFilter {
            event_type: Some(event_types::AGENT_CHANGED.to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(evs.len(), 1, "config service must author exactly one agent.changed");
    assert_eq!(evs[0].entity_id, "agents", "entity_id (subject) preserved");
}

// config.file_changed: per-file event, rel path entity, content hash via FsBroker.
// config.* is Runtime-scoped → lands in runtime.db (scope-routing invariant).
#[tokio::test(flavor = "multi_thread")]
async fn config_service_authors_file_changed_runtime_scoped_via_fs_broker() {
    let (_tmp, ship_dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();

    // The watched domain dir exists in production (the OS watcher only fires on
    // existing dirs); create it as a test fixture so the broker's cap root is a
    // real, canonical directory.
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(PortSpaceId::new("config")),
        FsMode::ReadWrite,
    );
    fs.write_atomic(&cap, "x.jsonc", b"{}").unwrap();
    let file = dir.join("x.jsonc");

    author_file_changed(&kernel, &fs, "agents", &dir, &file, Some(SpaceId::new("sp-1"))).await;

    assert_eq!(
        count_events(&ship_substrate::db::runtime_db_path(&ship_dir), event_types::CONFIG_FILE_CHANGED),
        1,
        "config service must author config.file_changed into runtime.db"
    );
}

// Empty content hash on read failure = deleted marker (behavior preserved from
// the raw-std::fs::read predecessor, now via FsBroker).
#[tokio::test(flavor = "multi_thread")]
async fn file_changed_deleted_marker_has_empty_hash() {
    let (_tmp, ship_dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    // Point at a file that does not exist — the broker read fails → empty hash.
    let missing = dir.join("gone.jsonc");

    author_file_changed(&kernel, &fs, "agents", &dir, &missing, Some(SpaceId::new("sp-1"))).await;

    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::runtime_db_path(&ship_dir)).unwrap();
    let evs = store
        .query(&EventFilter {
            event_type: Some(event_types::CONFIG_FILE_CHANGED.to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(evs.len(), 1, "one config.file_changed for the deleted file");
    let payload: ship_types::payload::ConfigFileChanged =
        serde_json::from_str(&evs[0].payload_json).unwrap();
    assert_eq!(payload.path, "gone.jsonc", "rel path preserved");
    assert!(payload.content_hash.is_empty(), "missing file → empty hash = deleted marker");
}

// S1: the evolved agent def notification. `agent.def.updated` is the Format-2,
// space-stamped, content-hashed event for the agents domain. It is Space-scoped
// → it routes to the bound space's db, carries {path, content_hash} (NEVER file
// content), and the hash changes when the file content changes.
#[tokio::test(flavor = "multi_thread")]
async fn agent_def_updated_is_space_scoped_with_content_hash() {
    let (_tmp, ship_dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(PortSpaceId::new("config")),
        FsMode::ReadWrite,
    );
    fs.write_atomic(&cap, "alpha.jsonc", br#"{"agent":{"id":"alpha","name":"Alpha"}}"#)
        .unwrap();
    let file = dir.join("alpha.jsonc");

    author_agent_def_updated(&kernel, &fs, &dir, &file, Some(SpaceId::new("sp-1"))).await;

    // agent.def.updated is Space-scoped → it lands in the bound space's db, not
    // runtime.db.
    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::space_db_path(&ship_dir, "sp-1"))
            .unwrap();
    let evs = store
        .query(&EventFilter {
            event_type: Some(event_types::AGENT_DEF_UPDATED.to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(evs.len(), 1, "one agent.def.updated emitted");
    assert_eq!(evs[0].entity_id, "alpha.jsonc", "entity is the rel path");
    let payload: ship_types::payload::AgentDefUpdated =
        serde_json::from_str(&evs[0].payload_json).unwrap();
    assert_eq!(payload.path, "alpha.jsonc");
    assert!(!payload.content_hash.is_empty(), "an existing file carries its content hash");
    let hash_before = payload.content_hash.clone();

    // Edit the file → the hash carried by the next notification must change.
    fs.write_atomic(
        &cap,
        "alpha.jsonc",
        br#"{"agent":{"id":"alpha","name":"Alpha Prime"}}"#,
    )
    .unwrap();
    author_agent_def_updated(&kernel, &fs, &dir, &file, Some(SpaceId::new("sp-1"))).await;

    let evs2 = store
        .query(&EventFilter {
            event_type: Some(event_types::AGENT_DEF_UPDATED.to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(evs2.len(), 2, "second edit emits a second notification");
    // Order-independent: across the two notifications, one carries the original
    // hash and the other carries a NEW hash — the edited content changed it.
    let hashes: Vec<String> = evs2
        .iter()
        .map(|e| {
            serde_json::from_str::<ship_types::payload::AgentDefUpdated>(&e.payload_json)
                .unwrap()
                .content_hash
        })
        .collect();
    assert!(hashes.contains(&hash_before), "the first notification's hash is present");
    assert!(
        hashes.iter().any(|h| *h != hash_before),
        "edited content → a NEW content_hash appears"
    );
}

// A removed def file → the notification carries an empty hash (deleted marker),
// the signal the index uses to drop the row.
#[tokio::test(flavor = "multi_thread")]
async fn agent_def_updated_deleted_marker_has_empty_hash() {
    let (_tmp, ship_dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let missing = dir.join("gone.jsonc");

    author_agent_def_updated(&kernel, &fs, &dir, &missing, Some(SpaceId::new("sp-1"))).await;

    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::space_db_path(&ship_dir, "sp-1"))
            .unwrap();
    let evs = store
        .query(&EventFilter {
            event_type: Some(event_types::AGENT_DEF_UPDATED.to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(evs.len(), 1, "one agent.def.updated for the deleted file");
    let payload: ship_types::payload::AgentDefUpdated =
        serde_json::from_str(&evs[0].payload_json).unwrap();
    assert_eq!(payload.path, "gone.jsonc");
    assert!(payload.content_hash.is_empty(), "missing file → empty hash = deleted marker");
}
