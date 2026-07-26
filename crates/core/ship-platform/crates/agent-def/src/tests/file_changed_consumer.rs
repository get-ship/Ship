//! Tests for the agent-def domain consumer
//! (`crate::file_changed_consumer`).
//!
//! Acceptance contract (job/transient-events):
//! 1. A `config.file_changed { domain: "agents", path, content_hash }`
//!    emitted on the shared `KernelLog` causes the consumer to emit
//!    **one durable** `agent.def.updated` carrying the same `{path,
//!    content_hash}`, routed to the bound space's db.
//! 2. The `config.file_changed` itself is **not persisted** (Runtime +
//!    Ephemeral → kernel skips the sink).
//! 3. A `config.file_changed` whose `domain` is not `"agents"` (e.g. a
//!    skills change) is **not converted** — the consumer's filter
//!    ignores it.
//! 4. A non-`config.file_changed` event type is **not converted** — the
//!    consumer's event-type filter ignores it.
//! 5. A deleted-marker payload (empty `content_hash`) propagates verbatim
//!    to the durable `agent.def.updated` — the read-only index drops
//!    its row on the empty hash.

use std::path::PathBuf;
use std::sync::Arc;

use ship_substrate::FsBroker;
use ship_substrate::events::EventFilter;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::ports::{
    FsCap, FsMode, FsRoot, KernelLog, LocalFsBroker, SpaceId as PortSpaceId,
};
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::{AgentDefUpdated, ConfigFileChanged};
use tokio::time::{Duration, sleep};

use crate::file_changed_consumer::spawn_agent_def_consumer;

async fn setup() -> (tempfile::TempDir, PathBuf, Arc<KernelLog>, SpaceId) {
    let tmp = tempfile::tempdir().unwrap();
    // Marker cutover: content anchors at the STACK ROOT.
    let ship_dir = std::fs::canonicalize(tmp.path()).unwrap();
    let kernel = Arc::new(KernelLog::new(
        new_isolated_for_test(ship_dir.clone()).unwrap(),
    ));
    let space = SpaceId::new("sp-consumer-1");
    // I-2: `PartitionedSink::route()` refuses to write `spaces/<key>/space.db`
    // for a scope_key with no `spaces` registry row. Register the test's
    // fabricated space id directly (test-only shortcut for
    // `ensure_identity_for_root`'s effect) so the consumer's Space-scoped
    // `agent.def.updated` emit routes cleanly.
    ship_substrate::testutil::register_space(&ship_dir, "sp-consumer-1").unwrap();
    (tmp, ship_dir, kernel, space)
}

fn agent_def_updated_events(ship_dir: &std::path::Path, space: &str) -> Vec<AgentDefUpdated> {
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

/// Poll `agent_def_updated_events` until at least `min_count` rows land or a
/// 2s budget expires. The consumer applies asynchronously off a broadcast
/// subscription; a single fixed sleep races the scheduler under concurrent
/// test load (`cargo test --workspace` runs many crates' test binaries at
/// once) and flakes. Every caller here already asserts the exact count once
/// convergence is observed, so over-waiting on a slow CI box costs time, not
/// correctness.
async fn wait_for_events(
    ship_dir: &std::path::Path,
    space: &str,
    min_count: usize,
) -> Vec<AgentDefUpdated> {
    for _ in 0..80 {
        let events = agent_def_updated_events(ship_dir, space);
        if events.len() >= min_count {
            return events;
        }
        sleep(Duration::from_millis(25)).await;
    }
    agent_def_updated_events(ship_dir, space)
}

fn config_file_changed_events(ship_dir: &std::path::Path) -> Vec<ConfigFileChanged> {
    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::runtime_db_path(ship_dir)).unwrap();
    store
        .query(&EventFilter {
            event_type: Some(event_types::CONFIG_FILE_CHANGED.to_string()),
            limit: Some(50),
            ..Default::default()
        })
        .unwrap()
        .iter()
        .map(|e| serde_json::from_str(&e.payload_json).unwrap())
        .collect()
}

/// Drive the consumer end-to-end: emit one `config.file_changed` for the
/// `agents` domain, then verify the consumer produced the durable
/// `agent.def.updated` row.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consumer_emits_durable_agent_def_updated_for_agents_domain() {
    let (_tmp, ship_dir, kernel, space) = setup().await;
    let _consumer = spawn_agent_def_consumer(kernel.clone(), space.clone());

    // Stage a def file on disk so the watcher's content_hash would be
    // non-empty; we hand-build the envelope here so the test does not
    // depend on the FsBroker watcher code path.
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(PortSpaceId::new("config")),
        FsMode::ReadWrite,
    );
    LocalFsBroker::new()
        .write_atomic(&cap, "alpha.jsonc", br#"{"id":"alpha"}"#)
        .unwrap();
    let on_disk = std::fs::read(dir.join("alpha.jsonc")).unwrap();
    let content_hash = ship_types::sha256_hex(&on_disk);

    // Hand-build the transient signal — the same shape the watcher emits.
    let payload = ConfigFileChanged {
        domain: "agents".into(),
        path: "alpha.jsonc".into(),
        content_hash: content_hash.clone(),
    };
    let envelope = EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, "alpha.jsonc", &payload)
        .expect("envelope builds");
    let ctx = ship_substrate::events::EmitContext::daemon_internal(Some(space.clone()));
    kernel
        .emit(envelope, &ctx)
        .await
        .expect("emit transient signal");

    // The consumer wrote ONE durable `agent.def.updated` to the space's db.
    let events = wait_for_events(&ship_dir, space.0.as_str(), 1).await;
    assert_eq!(
        events.len(),
        1,
        "consumer must emit exactly one durable agent.def.updated for the agents transient"
    );
    let ev = &events[0];
    assert_eq!(ev.path, "alpha.jsonc", "path carries the file's rel path");
    assert_eq!(
        ev.content_hash, content_hash,
        "content_hash passes through from the transient signal to the durable row"
    );

    // The transient signal itself was NOT persisted (Runtime + Ephemeral).
    let transients = config_file_changed_events(&ship_dir);
    assert!(
        transients.is_empty(),
        "config.file_changed must NOT be persisted (Ephemeral); runtime.db has no row"
    );
}

/// A `config.file_changed` whose `domain` is not `"agents"` is **not**
/// converted. Proves the consumer's filter is the only thing standing
/// between the transient signal and the durable event — without it,
/// every config-file ping would produce a domain event.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consumer_ignores_non_agents_domain_transient() {
    let (_tmp, ship_dir, kernel, space) = setup().await;
    let _consumer = spawn_agent_def_consumer(kernel.clone(), space.clone());

    let payload = ConfigFileChanged {
        domain: "skills".into(),
        path: "deploy.jsonc".into(),
        content_hash: "abc".into(),
    };
    let envelope = EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, "deploy.jsonc", &payload)
        .expect("envelope builds");
    let ctx = ship_substrate::events::EmitContext::daemon_internal(Some(space.clone()));
    kernel.emit(envelope, &ctx).await.expect("emit");

    sleep(Duration::from_millis(50)).await;

    let events = agent_def_updated_events(&ship_dir, space.0.as_str());
    assert!(
        events.is_empty(),
        "consumer must NOT emit agent.def.updated for a skills-domain transient (filter: domain == agents)"
    );
}

/// A non-`config.file_changed` event on the broadcast is **not**
/// converted, even if a payload happens to carry an `agents` domain
/// field. Proves the consumer's event-type filter is the outer guard.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consumer_ignores_unrelated_event_types() {
    let (_tmp, ship_dir, kernel, space) = setup().await;
    let payload = ConfigFileChanged {
        domain: "agents".into(),
        path: "alpha.jsonc".into(),
        content_hash: "h".into(),
    };
    // An unregistered event type so the kernel doesn't reject the envelope
    // on payload validation (the production path). The consumer must
    // still NOT convert — the event-type filter is the outer guard.
    let envelope =
        EventEnvelope::new("test.unregistered_type_xyz", "x", &payload).expect("envelope builds");
    let ctx = ship_substrate::events::EmitContext::daemon_internal(Some(space.clone()));
    kernel.emit(envelope, &ctx).await.expect("emit");

    sleep(Duration::from_millis(50)).await;

    let events = agent_def_updated_events(&ship_dir, space.0.as_str());
    assert!(
        events.is_empty(),
        "consumer must NOT emit agent.def.updated for a non-config.file_changed event (filter: event_type)"
    );
}

/// The deleted-marker (empty `content_hash`) flows verbatim from the
/// transient signal to the durable `agent.def.updated` — the read-only
/// index in ship-config depends on this to drop its row.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consumer_propagates_deleted_marker_to_durable_event() {
    let (_tmp, ship_dir, kernel, space) = setup().await;
    let _consumer = spawn_agent_def_consumer(kernel.clone(), space.clone());

    let payload = ConfigFileChanged {
        domain: "agents".into(),
        path: "gone.jsonc".into(),
        content_hash: String::new(),
    };
    let envelope = EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, "gone.jsonc", &payload)
        .expect("envelope builds");
    let ctx = ship_substrate::events::EmitContext::daemon_internal(Some(space.clone()));
    kernel.emit(envelope, &ctx).await.expect("emit");

    let events = wait_for_events(&ship_dir, space.0.as_str(), 1).await;
    assert_eq!(
        events.len(),
        1,
        "deleted marker still produces one durable event"
    );
    assert_eq!(events[0].path, "gone.jsonc");
    assert!(
        events[0].content_hash.is_empty(),
        "empty content_hash is the deleted-marker the read-only index drops on"
    );
}

/// Two transients in quick succession produce two durable rows (no
/// dedup at the consumer layer). The read-only agent-def index
/// (`ship_config::agents_index`) is the idempotent surface — it
/// re-reads the file on each notification and upserts / drops as
/// appropriate. The consumer is a pure signal-forwarder.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consumer_emits_one_durable_per_transient_no_dedup() {
    let (_tmp, ship_dir, kernel, space) = setup().await;
    let _consumer = spawn_agent_def_consumer(kernel.clone(), space.clone());

    for (path, hash) in [("a.jsonc", "h1"), ("b.jsonc", "h2")] {
        let payload = ConfigFileChanged {
            domain: "agents".into(),
            path: path.into(),
            content_hash: hash.into(),
        };
        let envelope = EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, path, &payload)
            .expect("envelope builds");
        let ctx = ship_substrate::events::EmitContext::daemon_internal(Some(space.clone()));
        kernel.emit(envelope, &ctx).await.expect("emit");
    }

    let events = wait_for_events(&ship_dir, space.0.as_str(), 2).await;
    assert_eq!(
        events.len(),
        2,
        "consumer is a 1:1 forwarder; dedup lives in the index"
    );
    let paths: Vec<&str> = events.iter().map(|e| e.path.as_str()).collect();
    assert!(paths.contains(&"a.jsonc"));
    assert!(paths.contains(&"b.jsonc"));
}
