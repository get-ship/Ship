//! Watcher authorship proofs — ported from the pre-carve `config/watcher.rs`
//! keystone test. Drives the authoring path DIRECTLY with a synthetic signal
//! (no inotify, not flaky), and proves scope routing + the FsBroker eviction of
//! the content-hash read.

use std::path::Path;
use std::sync::Arc;

use ship_config::watcher::{author_change, author_file_changed};
use ship_substrate::events::EventFilter;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::ports::{
    FsBroker, FsCap, FsMode, FsRoot, KernelLog, LocalFsBroker, SpaceId as PortSpaceId,
};
use ship_types::SpaceId;
use ship_types::event_types;

// Force a reference to an `agent-def` symbol so the linker keeps the
// `agent-def::event_registrations` inventory submissions alive in this
// integration test binary. `agent-def` is a `[dev-dependency]` of
// `ship-config` and would otherwise be dead-code-stripped, hiding
// `agent.def.updated` from the registry the runtime shipd binary sees
// (job/transient-events). The `TypeId` reference is a no-op at runtime
// — pure compile-time / link-time concern.
#[allow(dead_code)]
const _AGENT_DEF_LINK: fn() -> Option<std::any::TypeId> = || {
    Some(std::any::TypeId::of::<agent_def::Agent>())
};
async fn setup() -> (tempfile::TempDir, std::path::PathBuf, Arc<KernelLog>) {
    let tmp = tempfile::tempdir().unwrap();
    // Canonicalize: `/tmp` is symlinked on WSL/macOS; the FsBroker's symlink-
    // escape guard canonicalizes the deepest existing ancestor and would reject
    // a non-canonical cap root. Production paths are already canonical.
    // Marker cutover: the watch root IS the stack root.
    let ship_dir = std::fs::canonicalize(tmp.path()).unwrap();
    let kernel = Arc::new(KernelLog::new(new_isolated_for_test(ship_dir.clone()).unwrap()));
    // I-2: register the fixed "sp-1" space id this suite's tests share, so
    // `PartitionedSink::route()`'s registry-existence check doesn't refuse
    // the Space-scoped `agent.changed`/`agent.def.updated` events these
    // tests emit under it.
    ship_substrate::testutil::register_space(&ship_dir, "sp-1").unwrap();
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

// Keystone (job/transient-events): the config service (not the OS watcher)
// authors `agent.changed` as a TRANSIENT ping. The coarse `agent.changed` is
// a Bucket-2 watch-ping — the def file is truth, the row is a hint to wake
// domain consumers, and the durable `agent.def.updated` is what the read-only
// index re-reads. Both are Space-scoped; `agent.changed` is now
// `Retention::Ephemeral` and the kernel's `route` honors that — delivered
// to live mailboxes, NOT persisted to the space db. See
// `.artifacts/transient-events-design.md`.
#[tokio::test(flavor = "multi_thread")]
async fn config_service_authors_domain_event_with_identical_shape() {
    let (_tmp, ship_dir, kernel) = setup().await;

    let mut rx = kernel.subscribe();
    author_change(&kernel, event_types::AGENT_CHANGED, "agents", Some(SpaceId::new("sp-1"))).await;

    // 1. Live broadcast carried the event with the SAME shape the watcher
    //    used to emit — event_type, entity_id preserved.
    let delivered = rx.try_recv().expect("broadcast delivered the coarse event");
    assert_eq!(delivered.event_type, event_types::AGENT_CHANGED);
    assert_eq!(delivered.entity_id, "agents", "entity_id (subject) preserved");

    // 2. The event was NOT persisted — it is Ephemeral.
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
    assert!(
        evs.is_empty(),
        "Ephemeral agent.changed must NOT be persisted; space db has no row"
    );
}

// config.file_changed: per-file event, rel path entity, content hash via
// FsBroker. Runtime-scoped → routes to runtime.db's ephemeral path. Job/
// transient-events: this is the GENERIC TRANSIENT signal the watcher emits
// for every changed config file (no domain knowledge). Runtime + Ephemeral →
// delivered to live subscribers, NOT persisted to runtime.db. The domain
// (e.g. `agent-def`) is what owns the durable `agent.def.updated` row.
#[tokio::test(flavor = "multi_thread")]
async fn config_service_authors_file_changed_runtime_scoped_via_fs_broker() {
    let (_tmp, ship_dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();

    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(PortSpaceId::new("config")),
        FsMode::ReadWrite,
    );
    fs.write_atomic(&cap, "x.jsonc", b"{}").unwrap();
    let file = dir.join("x.jsonc");

    let mut rx = kernel.subscribe();
    author_file_changed(&kernel, &fs, "agents", &dir, &file, Some(SpaceId::new("sp-1"))).await;

    // The transient signal reached subscribers (the broadcast is the
    // channel the agent-def domain consumer uses).
    let delivered = rx.try_recv().expect("broadcast delivered config.file_changed");
    assert_eq!(delivered.event_type, event_types::CONFIG_FILE_CHANGED);
    let payload: ship_types::payload::ConfigFileChanged =
        serde_json::from_str(&delivered.payload_json).unwrap();
    assert_eq!(payload.domain, "agents", "domain field carries the watched dir name");
    assert_eq!(payload.path, "x.jsonc", "rel path under the watched dir");
    assert!(!payload.content_hash.is_empty(), "content hash is present for an existing file");

    // Runtime + Ephemeral → NOT persisted.
    assert_eq!(
        count_events(&ship_substrate::db::runtime_db_path(&ship_dir), event_types::CONFIG_FILE_CHANGED),
        0,
        "Ephemeral config.file_changed must NOT be persisted to runtime.db"
    );
}

// Empty content hash on read failure = deleted marker (behavior preserved
// from the raw-std::fs::read predecessor, now via FsBroker). Job/
// transient-events: the empty-hash marker rides the TRANSIENT
// `config.file_changed` — the domain (e.g. `agent-def`) sees the marker on
// the broadcast and decides whether to drop its index row; the watcher
// itself does not persist anything.
#[tokio::test(flavor = "multi_thread")]
async fn file_changed_deleted_marker_has_empty_hash() {
    let (_tmp, ship_dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let missing = dir.join("gone.jsonc");

    let mut rx = kernel.subscribe();
    author_file_changed(&kernel, &fs, "agents", &dir, &missing, Some(SpaceId::new("sp-1"))).await;

    // The deleted marker rides the broadcast (the channel the domain
    // consumer uses to drop its row).
    let delivered = rx.try_recv().expect("broadcast delivered the deleted marker");
    let payload: ship_types::payload::ConfigFileChanged =
        serde_json::from_str(&delivered.payload_json).unwrap();
    assert_eq!(payload.path, "gone.jsonc", "rel path preserved");
    assert!(payload.content_hash.is_empty(), "missing file → empty hash = deleted marker");

    // Runtime + Ephemeral → NOT persisted.
    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::runtime_db_path(&ship_dir)).unwrap();
    let evs = store
        .query(&EventFilter {
            event_type: Some(event_types::CONFIG_FILE_CHANGED.to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    assert!(evs.is_empty(), "Ephemeral config.file_changed must NOT be persisted");
}


// ── Watcher domain-freedom (job/transient-events) ──────────────────────────────
//
// The watcher owns the file-tier signal path: it authors only TRANSIENT pings
// (`agent.changed`, `config.file_changed`). It must NEVER author the DURABLE
// `agent.def.updated` event — the agent-def domain owns that. We prove this
// BEHAVIORALLY (R18: no source-grep): drive every authoring entry point and
// assert the durable type is neither broadcast nor persisted.

/// Behavioral proof: no watcher authoring path emits or persists the durable
/// `agent.def.updated`. Replaces the former source-grep guards, which broke on
/// rename and proved nothing about behavior.
#[tokio::test(flavor = "multi_thread")]
async fn watcher_never_authors_durable_agent_def_updated() {
    const AGENT_DEF_UPDATED: &str = "agent.def.updated";
    let (_tmp, ship_dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    fs.write_atomic(
        &FsCap::new_unchecked(
            dir.clone(),
            FsRoot::Space(PortSpaceId::new("config")),
            FsMode::ReadWrite,
        ),
        "a.jsonc",
        b"{}",
    )
    .unwrap();
    let file = dir.join("a.jsonc");

    let mut rx = kernel.subscribe();
    // Every authoring entry point the watcher exposes.
    author_change(&kernel, event_types::AGENT_CHANGED, "agents", Some(SpaceId::new("sp-1"))).await;
    author_file_changed(&kernel, &fs, "agents", &dir, &file, Some(SpaceId::new("sp-1"))).await;

    // Nothing the watcher broadcast was the durable agent-def event.
    while let Ok(ev) = rx.try_recv() {
        assert_ne!(
            ev.event_type, AGENT_DEF_UPDATED,
            "watcher must not author the durable `agent.def.updated` — the agent-def domain owns it"
        );
    }

    // And nothing durable was persisted under that type, in either scope.
    assert_eq!(
        count_events(&ship_substrate::db::space_db_path(&ship_dir, "sp-1"), AGENT_DEF_UPDATED),
        0,
        "watcher must not persist `agent.def.updated` to the space db"
    );
    assert_eq!(
        count_events(&ship_substrate::db::runtime_db_path(&ship_dir), AGENT_DEF_UPDATED),
        0,
        "watcher must not persist `agent.def.updated` to runtime.db"
    );
}

// ── Source-root coverage (Phase-3 J-event-channel) ──────────────────────────────
//
// `stack.config.ts` sits outside the three declarative `.ship/<domain>/`
// dirs (agents, skills, views) `spawn_config_watcher` watches by default
// (job-spec-p3-event-channel.md). `spawn_source_root_watch` adds a
// dedicated, filtered watch for it; a single edit must author exactly
// ONE `config.file_changed` — no double-fire from an overlapping watch.
#[tokio::test(flavor = "multi_thread")]
async fn ship_ts_edit_authors_exactly_one_config_file_changed() {
    let (_tmp, ship_dir, kernel) = setup().await;
    let workspace_dir = ship_dir.clone(); // the stack root IS the workspace
    std::fs::write(workspace_dir.join("stack.config.ts"), "export const x = 0;").unwrap();

    let mut rx = kernel.subscribe();
    let _watcher = ship_config::watcher::spawn_config_watcher(
        kernel.clone(),
        Some(workspace_dir.clone()),
    )
    .unwrap();

    // Let the watch open before editing.
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    std::fs::write(workspace_dir.join("stack.config.ts"), "export const x = 1;").unwrap();

    // Collect every config.file_changed touching stack.config.ts over a window
    // comfortably past the 200ms default debounce.
    let mut count = 0;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1200);
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Ok(ev)) if ev.event_type == event_types::CONFIG_FILE_CHANGED => {
                let payload: ship_types::payload::ConfigFileChanged =
                    serde_json::from_str(&ev.payload_json).unwrap();
                if payload.path == "stack.config.ts" {
                    count += 1;
                }
            }
            Ok(Ok(_)) => continue,
            _ => break,
        }
    }
    assert_eq!(
        count, 1,
        "a single stack.config.ts edit must author exactly one config.file_changed"
    );
}
