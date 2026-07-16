//! Behavioral proofs for ship-space.
//!
//! All tests use temp-dir ports — no network, no real daemon. The
//! `SpacesProjection` is installed once per test-binary via the
//! `install_spaces_projection()` helper below; this ensures the event-fold
//! writes to the `spaces` SQL table so the async reads (which now go
//! through `spaces` + `db::spaces`) see each newly opened space.
//!
//! The 2026-07-09 cut: `info::compose`, `Active` / `active_id` / the
//! `space_active` pointer are GONE. The registry is now just `spaces`
//! (Runtime-scoped projection) and identity is mint-once via
//! `identity::ensure_identity_for_root`. These tests cover both halves:
//! the projection reads AND the mint-time canonicalization, plus the
//! served-space identity round-trip.

use std::sync::Arc;
use std::time::Duration;

use ship_space::identity;
use ship_space::service::SpaceService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{DataStore, KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

// ── Projection install ────────────────────────────────────────────────────────

/// Install the `SpacesProjection` into the global extra-projections factory
/// once per test binary. Safe to call from each test: the `Once` guard makes
/// subsequent calls no-ops. Without this the event-fold path (which populates
/// the `spaces` SQL table) does not run inside the isolated kernel, and all
/// registry reads return empty / not-found.
fn install_spaces_projection() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![
                Box::new(ship_space::SpacesProjection),
                Box::new(ship_space::SpaceStacksProjection),
            ]
        }));
    });
}

// ── Fixtures ──────────────────────────────────────────────────────────────────
//
// Opening a directory as a space needs NO project tree: `identity` mints
// the space id itself (a nanoid) and the projection fold records `path` +
// `opened_at` from the emitted `space.opened` event. Tests open bare
// directories.

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    install_spaces_projection();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

/// A `SpaceService` whose `ctx` carries an empty served-space `SpaceId`
/// (the runtime partition slot — typical of the `SpaceService::info` path
/// when there's no served space; also the slot used for the projection-
/// level reads, since they don't need an identity binding).
fn global_service(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> SpaceService {
    let _ = tmp; // currently unused; keep the fixture shape for future tests
    let store = store_for(kernel.clone(), "runtime", "runtime");
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(""), store, kernel));
    SpaceService::new(ctx)
}

fn store_for(kernel: Arc<KernelLog>, db_name: &str, partition: &str) -> Arc<dyn DataStore> {
    let ship_dir = kernel.try_ship_dir().expect("kernel ship_dir");
    let db_path = ship_dir.join(format!("{db_name}.db"));
    let svc = DataService::open_at(&db_path).unwrap();
    Arc::new(SingleBackendDataStore::new(svc, partition))
}

// ── info (served-space identity) ──────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_no_served_space_returns_empty_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = global_service(&tmp, kernel);

    let composed = svc.info().unwrap();
    assert_eq!(composed.space_id, None);
    assert_eq!(composed.root, None);
    assert_eq!(composed.name, None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_resolves_served_space_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = global_service(&tmp, kernel.clone());

    let path = tmp.path().join("acme");
    std::fs::create_dir_all(&path).unwrap();
    let record = svc.create(&path, Some("Acme")).await.unwrap();
    let space = record.space_id.clone();

    // Serve the newly-opened space and compose its identity from the
    // projection.
    let store = store_for(kernel.clone(), "runtime", "runtime");
    let served_ctx = Arc::new(SpaceContext::new(
        SpaceId::new(&space),
        store,
        kernel.clone(),
    ));
    let served = SpaceService::new(served_ctx);
    let composed = served.info().unwrap();
    assert_eq!(composed.space_id.as_deref(), Some(space.as_str()));
    assert_eq!(composed.name.as_deref(), Some("Acme"));
    assert_eq!(composed.root.as_deref(), Some(record.path.as_str()));
}

// ── list (Runtime-scoped spaces projection) ───────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_returns_spaces_newest_first() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = global_service(&tmp, kernel);

    let a = tmp.path().join("re-a");
    let b = tmp.path().join("re-b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    svc.create(&a, Some("re-a")).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    svc.create(&b, Some("re-b")).await.unwrap();

    let listed = svc.list().unwrap();
    let b_pos = listed.iter().position(|p| p.name == "re-b").unwrap();
    let a_pos = listed.iter().position(|p| p.name == "re-a").unwrap();
    assert!(b_pos < a_pos, "newer space sorts first");
}

// ── identity::ensure_identity_for_root (the single mint-once accessor) ──────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn add_is_idempotent_same_id_name_preserved() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = global_service(&tmp, kernel);

    let path = tmp.path().join("proj");
    std::fs::create_dir_all(&path).unwrap();
    let r1 = svc.create(&path, Some("test")).await.unwrap();
    let r2 = svc.create(&path, Some("test-renamed")).await.unwrap();
    assert_eq!(r1.space_id, r2.space_id, "id stable across re-add");
    assert_eq!(r2.name, "test", "name preserved on second add");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn add_derives_name_from_path_when_unnamed() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let path = tmp.path().join("my-space");
    std::fs::create_dir_all(&path).unwrap();
    let r = identity::ensure_identity_for_root(&kernel, &path, None).await.unwrap();
    assert_eq!(r.name, "my-space", "name derived from final path segment");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_mints_id_without_a_tree() {
    // `identity::ensure_identity_for_root` is the SOLE birthplace of a space
    // id: a bare directory (no `.ship/`, no `ship.jsonc`) opens and gets a
    // stable id. No write ever touches the project tree.
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let path = tmp.path().join("bare");
    std::fs::create_dir_all(&path).unwrap();

    let r = identity::ensure_identity_for_root(&kernel, &path, None)
        .await
        .unwrap();
    assert!(!r.space_id.trim().is_empty(), "open mints a non-empty id");
    assert!(
        !path.join(".ship").exists(),
        "open must not scaffold a project tree"
    );

    let r2 = identity::ensure_identity_for_root(&kernel, &path, None)
        .await
        .unwrap();
    assert_eq!(r.space_id, r2.space_id, "re-open returns the same id");
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_is_idempotent_by_canonical_path() {
    // The `spaces` projection keys by CANONICAL path: opening the same
    // directory through a symlinked alias must resolve to the existing
    // row, not mint a second id. The canonicalization is owned by the
    // identity accessor (the single mint-once site).
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let path = tmp.path().join("proj");
    std::fs::create_dir_all(&path).unwrap();
    let alias = tmp.path().join("alias");
    std::os::unix::fs::symlink(&path, &alias).unwrap();

    let r1 = identity::ensure_identity_for_root(&kernel, &path, Some("orig"))
        .await
        .unwrap();
    let r2 = identity::ensure_identity_for_root(&kernel, &alias, Some("renamed"))
        .await
        .unwrap();
    assert_eq!(r1.space_id, r2.space_id, "one canonical path → one id");
    assert_eq!(r2.name, "orig", "re-open keeps the original name");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn re_open_emits_space_seen() {
    // Re-opening an already-known space emits `space.seen` (bumps
    // `last_seen_at` via `SpacesProjection`) — never `space.opened` again.
    // The mint-once identity stays put.
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = global_service(&tmp, kernel.clone());
    let mut rx = kernel.subscribe();

    let path = tmp.path().join("p");
    std::fs::create_dir_all(&path).unwrap();
    let r = svc.create(&path, Some("p")).await.unwrap();

    // Drain the open event(s).
    tokio::time::sleep(Duration::from_millis(20)).await;
    while rx.try_recv().is_ok() {}

    // Re-open.
    let _ = svc.create(&path, Some("p")).await.unwrap();

    let seen = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    assert_eq!(seen.event_type, "space.seen");
    assert_eq!(seen.entity_id, r.space_id);
}

// ── resolve_active_space rides the spaces projection ─────────────────────────
//
// `worker_threads = 1` keeps `get_global_dir()`'s per-thread test isolation on
// ONE thread, so the kernel (projection writer) and `resolve_active_space`
// (projection reader) share the same runtime.db.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn resolve_active_space_reads_the_spaces_projection() {
    install_spaces_projection();
    let global = ship_substrate::project::get_global_dir().unwrap();
    std::fs::create_dir_all(&global).unwrap();
    let router = new_isolated_for_test(global).unwrap();
    let kernel = Arc::new(KernelLog::new(router));

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("proj");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("stack.config.ts"), "export default {};\n").unwrap(); // the marker
    std::fs::create_dir_all(root.join("src")).unwrap();

    // Discoverable but unopened → space-less.
    assert_eq!(
        ship_substrate::project::resolve_active_space(&root.join("src")),
        None,
        "unopened root resolves to no active space"
    );

    let canonical = std::fs::canonicalize(&root).unwrap();
    let rec = identity::ensure_identity_for_root(&kernel, &canonical, None)
        .await
        .unwrap();

    assert_eq!(
        ship_substrate::project::resolve_active_space(&root.join("src")),
        Some(rec.space_id.clone()),
        "resolution must read the spaces projection"
    );
    assert!(
        !&root.join(".ship").join("ship.jsonc").exists(),
        "resolution/opening must never write identity into the tree"
    );

    // Re-opening bumps `last_seen_at` (and emits `space.seen`) but the
    // resolver returns the same id.
    let _ = identity::ensure_identity_for_root(&kernel, &canonical, None)
        .await
        .unwrap();
    assert_eq!(
        ship_substrate::project::resolve_active_space(&root.join("src")),
        Some(rec.space_id),
        "re-open keeps the canonical identity"
    );
}

// ── Round-trip: space.opened event folds into the spaces table; both ────────
//    the L1 async read AND the L0 sync read see the new row.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn space_opened_event_is_visible_in_both_async_and_sync_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = global_service(&tmp, kernel.clone());
    let ship_dir = kernel.try_ship_dir().unwrap();

    let path = tmp.path().join("my-project");
    std::fs::create_dir_all(&path).unwrap();

    // Opening the directory emits `space.opened`; `SpacesProjection` folds
    // it into the `spaces` SQL table inside the kernel transaction.
    let record = svc.create(&path, Some("MyProject")).await.unwrap();

    // 1. Async registry read (L1 — reads the `spaces` SQL via kernel.ship_dir).
    let async_list = svc.list().unwrap();
    assert!(
        !async_list.is_empty(),
        "async spaces::list must see the registered space"
    );
    let async_record = async_list.iter().find(|r| r.space_id == record.space_id);
    assert!(
        async_record.is_some(),
        "async spaces::get must find the space by id"
    );
    assert_eq!(async_record.unwrap().name, "MyProject");

    // 2. Sync SQL read (L0 — the path the space-id resolver uses).
    let sync_by_id = ship_substrate::db::spaces::get_at(&ship_dir, &record.space_id)
        .expect("sync get_at should not fail");
    assert!(
        sync_by_id.is_some(),
        "sync SQL read (get_at) must see the registered space"
    );
    assert_eq!(sync_by_id.unwrap().name, "MyProject");

    let path_str = path.to_string_lossy().to_string();
    let sync_by_path = ship_substrate::db::spaces::find_by_path_at(&ship_dir, &path_str)
        .expect("sync find_by_path_at should not fail");
    assert!(
        sync_by_path.is_some(),
        "sync SQL read (find_by_path_at) must see the registered space"
    );
    assert_eq!(sync_by_path.unwrap().space_id, record.space_id);
}

// ── stack membership ──────────────────────────────────────────────────────────

/// Membership is REGISTRATION: the composed set is exactly the registered
/// rows — a stack sitting on disk beside a member never appears, a non-stack
/// dir is rejected at the marker, and rows are per-space.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stack_membership_is_registration_not_scanning() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = global_service(&tmp, kernel);

    let mk = |name: &str, marker: bool| {
        let d = tmp.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        if marker {
            std::fs::write(d.join("stack.config.ts"), "export default {}\n").unwrap();
        }
        d
    };
    let a = mk("a", true);
    let b = mk("b", true);
    let _bystander_on_disk = mk("c", true);
    let not_a_stack = mk("d", false);

    svc.register_stack("sp-1", &a).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    svc.register_stack("sp-1", &b).await.unwrap();

    let err = svc.register_stack("sp-1", &not_a_stack).await.unwrap_err();
    assert!(
        format!("{err:?}").contains("not a stack root"),
        "non-stack dir must be rejected at the marker: {err:?}"
    );

    let rows = svc.list_stacks("sp-1").unwrap();
    assert_eq!(rows.len(), 2, "exactly the registered set — c never composes");
    assert!(rows[0].root_path.ends_with("/a"));
    assert!(rows[1].root_path.ends_with("/b"));

    assert!(
        svc.list_stacks("sp-2").unwrap().is_empty(),
        "membership rows are per-space"
    );
}
