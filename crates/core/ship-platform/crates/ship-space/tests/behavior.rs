//! Behavioral proofs for ship-space.
//!
//! All tests use temp-dir ports — no network, no real daemon. The `ProjectsProjection`
//! is installed once per test-binary via the `install_projects_projection()` helper
//! below; this ensures the event-fold writes to the `projects` SQL table so the
//! async registry reads (which now read from SQL, not the dissolved `space_registry`
//! DataStore) see each newly registered space.
//!
//! Ported from the retired `services/space/tests.rs` (non-window) + the
//! `projects/mod.rs` + `projects/active.rs` proofs, re-expressed against the
//! ports. The window proofs are CUT — they belong to the ship-device topic.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use ship_space::config;
use ship_space::info;
use ship_space::registry;
use ship_space::service::SpaceService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{DataStore, KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

// ── Projection install ────────────────────────────────────────────────────────

/// Install the `ProjectsProjection` into the global extra-projections factory
/// once per test binary. Safe to call from each test: the `Once` guard makes
/// subsequent calls no-ops. Without this the event-fold path (which populates
/// the `projects` SQL table) does not run inside the isolated kernel, and all
/// registry reads return empty / not-found.
fn install_projects_projection() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![Box::new(ship_space::ProjectsProjection)]
        }));
    });
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    install_projects_projection();
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

/// A DataStore over a fresh per-scope db bound to `partition`. The registry
/// active pointer (`space_active`) is Runtime-scoped: bind the runtime
/// partition. Per-space settings bind the target space's partition.
fn store_for(tmp: &tempfile::TempDir, db_name: &str, partition: &str) -> Arc<dyn DataStore> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    let db_path = ship_dir.join(format!("{db_name}.db"));
    let svc = DataService::open_at(&db_path).unwrap();
    Arc::new(SingleBackendDataStore::new(svc, partition))
}

/// A `SpaceService` whose `ctx` binds the Runtime registry store (the active
/// pointer) and the kernel handle.
fn registry_service(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> SpaceService {
    let store = store_for(tmp, "runtime", "runtime");
    let ctx = Arc::new(SpaceContext::new(SpaceId::new("runtime"), store, kernel));
    SpaceService::new(ctx)
}

// ── config::set ─────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_rejects_non_json_value_and_emits_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let store = store_for(&tmp, "spaces/space-a/space", "space-a");
    let mut rx = kernel.subscribe();

    let err = config::set(store.as_ref(), &kernel, "space-a", "theme", "not json at all")
        .await
        .expect_err("invalid JSON is rejected");
    assert!(matches!(err, ship_space::error::SpaceError::BadArgs(_)));

    // Nothing emitted on bad input.
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(rx.try_recv().is_err(), "no event emitted on bad input");
    // Nothing persisted.
    assert!(config::get(store.as_ref(), "theme").await.unwrap().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_canonicalizes_echoes_and_emits_one_event() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let store = store_for(&tmp, "spaces/space-a/space", "space-a");
    let mut rx = kernel.subscribe();

    // Whitespace-laden but valid JSON re-serializes to canonical form.
    let out = config::set(store.as_ref(), &kernel, "space-a", "theme", "  \"dark\"  ")
        .await
        .expect("valid JSON accepted");
    assert_eq!(out.key, "theme");
    assert_eq!(out.value_json, "\"dark\"", "value canonicalized");

    // Exactly one space.setting.changed event, carrying the key as the payload's
    // entity_id and the canonical value in the payload (the space rides the
    // envelope at the kernel/routing layer, not the payload — proven below by the
    // override landing in the space's own partition).
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    assert_eq!(event.event_type, "space.setting.changed");
    let payload: serde_json::Value = serde_json::from_str(&event.payload_json).unwrap();
    assert_eq!(payload["key"], json!("theme"), "key in the payload");
    assert_eq!(payload["value_json"], json!("\"dark\""), "canonical value in the payload");
    // exactly one event of this type.
    let extra: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok())
        .filter(|e| e.event_type == "space.setting.changed")
        .collect();
    assert!(extra.is_empty(), "exactly one space.setting.changed emitted");

    // The override is persisted + canonical, in THIS space's partition.
    let stored = config::get(store.as_ref(), "theme").await.unwrap();
    assert_eq!(stored, Some(json!("dark")));
    // A different space's partition does not see this override (space rides the
    // partition, not the payload).
    let other = store_for(&tmp, "spaces/space-b/space", "space-b");
    assert!(config::get(other.as_ref(), "theme").await.unwrap().is_none());
}

// ── info::compose (overlay) ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_no_space_passes_settings_through_unoverlaid() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let settings = store_for(&tmp, "spaces/none/space", "none");

    let base = json!({ "theme": "light", "x": 1 });
    let composed = info::compose(&kernel, settings.as_ref(), None, base.clone())
        .await
        .unwrap();
    assert_eq!(composed.space_id, None);
    assert_eq!(composed.root, None);
    assert_eq!(composed.name, None);
    assert_eq!(composed.settings, base, "no space ⟹ unoverlaid base");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_unknown_space_leaves_base_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let settings = store_for(&tmp, "spaces/space-unknown/space", "space-unknown");

    let base = json!({ "theme": "light" });
    let composed =
        info::compose(&kernel, settings.as_ref(), Some("space-unknown"), base.clone())
            .await
            .unwrap();
    assert_eq!(composed.space_id.as_deref(), Some("space-unknown"));
    // Unknown space ⟹ no projects row ⟹ no identity, no override ⟹ base unchanged.
    assert_eq!(composed.root, None);
    assert_eq!(composed.name, None);
    assert_eq!(composed.settings, base);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_live_space_resolves_identity_and_overlays_override() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel.clone());

    // Register a space so the registry has identity.
    let space_path = tmp.path().join("acme");
    std::fs::create_dir_all(&space_path).unwrap();
    let record = svc.add(&space_path, Some("Acme")).await.unwrap();
    let space = record.id.clone();

    // Write a per-space override against the space's partition store.
    let settings = store_for(&tmp, &format!("spaces/{space}/space"), &space);
    config::set(settings.as_ref(), &kernel, &space, "theme", "\"dark\"")
        .await
        .unwrap();

    let base = json!({ "theme": "light", "untouched": true });
    let composed = svc
        .info(settings.as_ref(), Some(&space), base)
        .await
        .unwrap();
    assert_eq!(composed.space_id.as_deref(), Some(space.as_str()));
    assert_eq!(composed.name.as_deref(), Some("Acme"));
    assert_eq!(composed.root.as_deref(), Some(record.path.as_str()));
    // Space override wins per-key; non-overridden keys survive.
    assert_eq!(composed.settings["theme"], json!("dark"));
    assert_eq!(composed.settings["untouched"], json!(true));
}

// ── registry: add / list / activate / active ─────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn add_is_idempotent_same_id_name_preserved() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    let path = tmp.path().join("proj");
    std::fs::create_dir_all(&path).unwrap();
    let r1 = svc.add(&path, Some("test")).await.unwrap();
    let r2 = svc.add(&path, Some("test-renamed")).await.unwrap();
    assert_eq!(r1.id, r2.id, "id stable across re-add");
    assert_eq!(r2.name, "test", "name preserved on second add");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_returns_spaces_newest_first() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    let a = tmp.path().join("re-a");
    let b = tmp.path().join("re-b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    svc.add(&a, Some("re-a")).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    svc.add(&b, Some("re-b")).await.unwrap();

    let listed = svc.list().await.unwrap();
    let b_pos = listed.iter().position(|p| p.name == "re-b").unwrap();
    let a_pos = listed.iter().position(|p| p.name == "re-a").unwrap();
    assert!(b_pos < a_pos, "newer space sorts first");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn activate_switches_pointer_and_emits_switched() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel.clone());
    let mut rx = kernel.subscribe();

    let path = tmp.path().join("a");
    std::fs::create_dir_all(&path).unwrap();
    let r = svc.add(&path, Some("a")).await.unwrap();

    // Drain the add event(s).
    tokio::time::sleep(Duration::from_millis(20)).await;
    while rx.try_recv().is_ok() {}

    let active = svc.activate(&r.id).await.unwrap();
    assert_eq!(active.id, r.id);
    assert_eq!(svc.active_id().await.unwrap().as_deref(), Some(r.id.as_str()));

    // The dissolution made the former silent `touch` an EVENT: activate()
    // emits project.seen (bumps last_seen_at via ProjectsProjection) and THEN
    // project.switched. Pin the sequence.
    let first = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    assert_eq!(first.event_type, "project.seen");
    let second = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    assert_eq!(second.event_type, "project.switched");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn activate_unknown_id_is_bad_args() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    let err = svc.activate("deadbeef").await.expect_err("unknown id rejected");
    assert!(matches!(err, ship_space::error::SpaceError::BadArgs(_)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn active_is_none_on_cold_boot_then_resolves_after_activate() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    assert!(svc.active().await.unwrap().is_none(), "None before any activate");

    let path = tmp.path().join("p");
    std::fs::create_dir_all(&path).unwrap();
    let r = svc.add(&path, Some("p")).await.unwrap();
    svc.activate(&r.id).await.unwrap();

    let active = svc.active().await.unwrap().expect("active resolves the activated record");
    assert_eq!(active.id, r.id);
    assert_eq!(active.name, "p");
}

// ── list surfaces the active pointer ──────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_surfaces_active_id_after_activate() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    let path = tmp.path().join("p1");
    std::fs::create_dir_all(&path).unwrap();
    let r = svc.add(&path, Some("p1")).await.unwrap();
    assert_eq!(svc.active_id().await.unwrap(), None, "no active pointer pre-activate");
    svc.activate(&r.id).await.unwrap();
    assert_eq!(svc.active_id().await.unwrap().as_deref(), Some(r.id.as_str()));
}

// ── registry::add direct (path → stable id) ──────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn add_derives_name_from_path_when_unnamed() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let path = tmp.path().join("my-space");
    std::fs::create_dir_all(&path).unwrap();
    let r = registry::add(&kernel, &path, None).await.unwrap();
    assert_eq!(r.name, "my-space", "name derived from final path segment");
}

// ── Behavioral proof: single table (dual-read-model dissolution) ─────────────
//
// Proves that a `project.registered` event folds into the `projects` SQL table
// (via `ProjectsProjection`) and is visible to BOTH:
//   1. The async registry read (ship_space::registry::get) — the L1 read path
//   2. The sync SQL read (ship_substrate::db::projects::get_at) — the L0 read
//      path used by the space-id resolver and `SpaceRegistry` impl.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn project_registered_event_is_visible_in_both_async_registry_and_sync_sql() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel.clone());
    let ship_dir = ShipHome::project_ship_dir(tmp.path());

    let path = tmp.path().join("my-project");
    std::fs::create_dir_all(&path).unwrap();

    // Register the space — emits `project.registered`; ProjectsProjection folds
    // it into the `projects` SQL table inside the kernel transaction.
    let record = svc.add(&path, Some("MyProject")).await.unwrap();

    // 1. Async registry read (L1 — reads from `projects` SQL via kernel.ship_dir).
    let async_list = svc.list().await.unwrap();
    assert!(
        !async_list.is_empty(),
        "async registry::list must see the registered space"
    );
    let async_record = async_list.iter().find(|r| r.id == record.id);
    assert!(async_record.is_some(), "async registry::get must find the registered space by id");
    assert_eq!(async_record.unwrap().name, "MyProject");

    // 2. Sync SQL read (L0 — the path the space-id resolver and SpaceRegistry use).
    let sync_by_id = ship_substrate::db::projects::get_at(&ship_dir, &record.id)
        .expect("sync get_at should not fail");
    assert!(
        sync_by_id.is_some(),
        "sync SQL read (get_at) must see the registered space"
    );
    assert_eq!(sync_by_id.unwrap().name, "MyProject");

    let path_str = path.to_string_lossy().to_string();
    let sync_by_path = ship_substrate::db::projects::find_by_path_at(&ship_dir, &path_str)
        .expect("sync find_by_path_at should not fail");
    assert!(
        sync_by_path.is_some(),
        "sync SQL read (find_by_path_at) must see the registered space"
    );
    assert_eq!(sync_by_path.unwrap().id, record.id);
}
