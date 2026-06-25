//! Behavioral proofs for ship-space.
//!
//! All tests use temp-dir ports — no network, no real daemon. The `ProjectsProjection`
//! is installed once per test-binary via the `install_projects_projection()` helper
//! below; this ensures the event-fold writes to the `projects` SQL table so the
//! async registry reads (which now read from SQL, not the dissolved `space_registry`
//! DataStore) see each newly registered space.
//!
//! S4 cut: the per-space `config::set` + `info::compose` overlay tests are gone
//! — the legacy `space_overlay` table is retired and the unified
//! `settings_overlay` is owned by `SettingsService` (ship-config). The
//! registry/identity tests below cover the rest of `ship-space`.

use std::sync::Arc;
use std::time::Duration;

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

/// Initialize a minimal project layout at `path` that `registry::add` and
/// `resolve_active_space` require: a `.ship/` directory and a `ship.jsonc`
/// with an `{}` body (no pre-set id).  The first call to `ensure_space_id` will
/// mint and persist a nanoid into that file, making the path registerable.
fn init_project(path: &std::path::Path) {
    let ship_dir = path.join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    std::fs::write(ship_dir.join("ship.jsonc"), "{}").unwrap();
}

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    install_projects_projection();
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

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

// ── info::compose (identity-only, S4) ─────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_no_space_returns_empty_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    let composed = ship_space::info::compose(&kernel, None).await.unwrap();
    assert_eq!(composed.space_id, None);
    assert_eq!(composed.root, None);
    assert_eq!(composed.name, None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_unknown_space_raises_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    let err = ship_space::info::compose(&kernel, Some("space-unknown"))
        .await
        .expect_err("unknown space must raise NotFound");
    assert!(matches!(err, ship_space::error::SpaceError::NotFound(_)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_live_space_resolves_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel.clone());

    let space_path = tmp.path().join("acme");
    std::fs::create_dir_all(&space_path).unwrap();
    init_project(&space_path);
    let record = svc.add(&space_path, Some("Acme")).await.unwrap();
    let space = record.id.clone();

    let composed = ship_space::info::compose(&kernel, Some(&space))
        .await
        .unwrap();
    assert_eq!(composed.space_id.as_deref(), Some(space.as_str()));
    assert_eq!(composed.name.as_deref(), Some("Acme"));
    assert_eq!(composed.root.as_deref(), Some(record.path.as_str()));
}

// ── registry: add / list / activate / active ─────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn add_is_idempotent_same_id_name_preserved() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    let path = tmp.path().join("proj");
    std::fs::create_dir_all(&path).unwrap();
    init_project(&path);
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
    init_project(&a);
    init_project(&b);
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
    init_project(&path);
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
async fn activate_unknown_id_is_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    let err = svc.activate("deadbeef").await.expect_err("unknown id rejected");
    assert!(matches!(err, ship_space::error::SpaceError::NotFound(_)));
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn active_is_none_on_cold_boot_then_resolves_after_activate() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = registry_service(&tmp, kernel);

    assert!(svc.active().await.unwrap().is_none(), "None before any activate");

    let path = tmp.path().join("p");
    std::fs::create_dir_all(&path).unwrap();
    init_project(&path);
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
    init_project(&path);
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
    init_project(&path);
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
    init_project(&path);

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
