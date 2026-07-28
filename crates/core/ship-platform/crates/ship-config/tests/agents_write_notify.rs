//! Write → notify proofs (S5 — agents arc).
//!
//! The gated def WRITE door is [`ship_config::agents_write::save_agents_config`]
//! (FsBroker). S5 closes the producer→index loop: after a successful gated
//! write, the write path emits ONE `agent.def.updated {path, content_hash}`
//! (Space-scoped) — deterministically, NOT via the inotify watcher. These proofs
//! assert that post-write NOTIFY:
//!   - a gated write → the file is written AND one agent.def.updated carries the
//!     NEW content hash (so the index can re-read);
//!   - a delete (file gone) → the notification carries an empty hash (the
//!     deleted-marker the index drops the row on);
//!   - the notify NEVER carries file content (Bucket-2 invariant).
//!
//! The emit is the SAME `agent.def.updated` the watcher authors (one event
//! shape), so a double-emit (op + watcher) is harmless at the idempotent index.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ship_config::agents_write::{notify_agent_def_saved, save_agents_config};
use ship_config::types::AgentProfile;
use agent_schema::ProfileMeta;

/// A minimal profile with just an id + name.
fn profile(id: &str, name: &str) -> AgentProfile {
    AgentProfile {
        agent: ProfileMeta { id: id.into(), name: name.into(), ..Default::default() },
        ..Default::default()
    }
}
use ship_substrate::events::EventFilter;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::ports::{KernelLog, LocalFsBroker};
use ship_types::SpaceId;
use ship_types::event_types;

// Force a reference to an `agent-def` symbol so the linker keeps the
// `agent-def::event_registrations` inventory submissions alive in this
// integration test binary. Without this, `agent.def.updated` is stripped
// from the registry this test sees and the kernel's
// retention-driven persist/deliver branch takes the unregistered default
// (Durable) — masking the actual production routing. See the matching
// block in tests/watcher.rs.
#[allow(dead_code)]
const _AGENT_DEF_LINK: fn() -> Option<std::any::TypeId> = || {
    Some(std::any::TypeId::of::<agent_schema::Agent>())
};
/// Canonical project `.ship` dir under a fresh tempdir. `/tmp` is symlinked on
/// WSL/macOS; the FsBroker's symlink-escape guard canonicalizes the deepest
/// existing ancestor, so the cap root must be canonical — production dirs are.
fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    base.join(".ship")
}

async fn setup() -> (tempfile::TempDir, PathBuf, Arc<KernelLog>) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    std::fs::create_dir_all(&dir).unwrap();
    let kernel = Arc::new(KernelLog::new(new_isolated_for_test(dir.clone()).unwrap()));
    // I-2: register the fixed "sp-1" space id this suite shares, so
    // `PartitionedSink::route()` doesn't refuse the Space-scoped
    // `agent.def.updated` these tests emit under it.
    ship_substrate::testutil::register_space(&dir, "sp-1").unwrap();
    (tmp, dir, kernel)
}

/// All `agent.def.updated` events landed in the space db (the event is
/// Space-scoped → routed to the bound space, not runtime.db).
fn def_events(ship_dir: &Path, space: &str) -> Vec<ship_types::payload::AgentDefUpdated> {
    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::space_db_path(ship_dir, space))
            .unwrap();
    store
        .query(&EventFilter {
            event_type: Some(event_types::AGENT_DEF_UPDATED.to_string()),
            limit: Some(50),
            ..Default::default()
        })
        .unwrap()
        .iter()
        .map(|e| serde_json::from_str(&e.payload_json).unwrap())
        .collect()
}

// ── Test 1: a gated write → file written + ONE agent.def.updated w/ new hash ──

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gated_write_then_notify_emits_one_def_updated_with_new_hash() {
    let (_tmp, dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let space = SpaceId::new("sp-1");

    // The gated FsBroker write door — writes `.ship/agents/alpha.jsonc`.
    save_agents_config(&fs, &dir, &[profile("alpha", "Alpha")]).unwrap();

    // The file exists on disk (the write landed).
    let path = ship_substrate::project::agents_ns(&dir).join("alpha.jsonc");
    assert!(path.is_file(), "the gated write produced the def file");
    let on_disk = std::fs::read(&path).unwrap();
    let expected_hash = ship_types::sha256_hex(&on_disk);

    // The post-write NOTIFY emits exactly one agent.def.updated carrying the
    // NEW content hash — never file content (Bucket-2).
    notify_agent_def_saved(&kernel, &fs, &dir, "alpha", Some(space.clone())).await;

    let events = def_events(&dir, "sp-1");
    assert_eq!(events.len(), 1, "exactly one agent.def.updated per gated write");
    assert_eq!(events[0].path, "alpha.jsonc", "rel path under .ship/agents/");
    assert_eq!(events[0].content_hash, expected_hash, "carries the file's content hash");
    assert!(!events[0].content_hash.is_empty(), "an existing file carries a hash");
}

// ── Test 2: an edit re-notifies with a CHANGED hash ───────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn edit_then_notify_carries_a_changed_hash() {
    let (_tmp, dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let space = SpaceId::new("sp-1");

    save_agents_config(&fs, &dir, &[profile("alpha", "Alpha")]).unwrap();
    notify_agent_def_saved(&kernel, &fs, &dir, "alpha", Some(space.clone())).await;
    let hash_before = def_events(&dir, "sp-1")[0].content_hash.clone();

    // Re-save with a changed name → new file content → the notify hash changes.
    save_agents_config(&fs, &dir, &[profile("alpha", "Alpha Prime")]).unwrap();
    notify_agent_def_saved(&kernel, &fs, &dir, "alpha", Some(space)).await;

    let events = def_events(&dir, "sp-1");
    assert_eq!(events.len(), 2, "the edit emits a second notification");
    let hashes: Vec<String> = events.into_iter().map(|p| p.content_hash).collect();
    assert!(hashes.contains(&hash_before), "the first hash is present");
    assert!(hashes.iter().any(|h| *h != hash_before), "edited content → a NEW hash appears");
}

// ── Test 3: a deleted def → notify carries an empty hash (deleted marker) ──────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_then_notify_carries_empty_hash_marker() {
    let (_tmp, dir, kernel) = setup().await;
    let fs = LocalFsBroker::new();
    let space = SpaceId::new("sp-1");

    // Write then prune via re-save with an empty set (the gated delete path).
    save_agents_config(&fs, &dir, &[profile("gone", "Gone")]).unwrap();
    save_agents_config(&fs, &dir, &[]).unwrap();
    let path = ship_substrate::project::agents_ns(&dir).join("gone.jsonc");
    assert!(!path.exists(), "the prune removed the def file");

    // The notify for the now-absent file carries an empty hash — the marker the
    // read-only index uses to drop the row.
    notify_agent_def_saved(&kernel, &fs, &dir, "gone", Some(space)).await;

    let events = def_events(&dir, "sp-1");
    assert_eq!(events.len(), 1, "one agent.def.updated for the deleted def");
    assert_eq!(events[0].path, "gone.jsonc");
    assert!(events[0].content_hash.is_empty(), "missing file → empty hash = deleted marker");
}
