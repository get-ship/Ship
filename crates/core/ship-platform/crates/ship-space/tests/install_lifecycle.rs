//! Integration tests for the library projection.
//!
//! Drives the `LibraryProjection` directly: emit `stack.installed` /
//! `stack.unregistered` events into the kernel + apply the projection
//! against the runtime.db, then query the `library` table.

use ship_space::LibraryProjection;
use ship_substrate::projections::registry::Projection;
use ship_substrate::testutil::EnvGuard;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{STACK_INSTALLED, STACK_UNREGISTERED};
use ship_types::payload::{StackInstalled, StackUnregistered};

fn fold(
    projection: &LibraryProjection,
    env: &EventEnvelope,
    conn: &mut sqlx::SqliteConnection,
) {
    projection.apply(env, conn).unwrap();
}

fn runtime_conn(data_dir: &std::path::Path) -> sqlx::SqliteConnection {
    // `open_runtime_db` blocks; the `tokio::test(flavor="multi_thread")`
    // attribute below satisfies the runtime requirement.
    ship_substrate::db::open_runtime_db(data_dir).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn library_projection_inserts_then_deletes() {
    let ship_home = tempfile::tempdir().unwrap();
    let _guard = EnvGuard::set(
        ship_substrate::env::Var::Home,
        ship_home.path().to_str().unwrap(),
    );
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();
    // Self-ensure the runtime.db exists.
    {
        let _ = runtime_conn(&data_dir);
    }

    let projection = LibraryProjection;

    // Install.
    let installed = StackInstalled {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        root_path: "/data/library/github.com%2Fship%2Ftest-stack/1.2.3".into(),
        content_hash: "sha256:abc".into(),
        installed_at: "2026-07-13T00:00:00Z".into(),
    };
    let env_install = EventEnvelope::new(STACK_INSTALLED, "library", &installed).unwrap();
    {
        let mut conn = runtime_conn(&data_dir);
        fold(&projection, &env_install, &mut conn);
    }
    let rows = ship_substrate::db::spaces::list_installed_at(&data_dir).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].path, "github.com/ship/test-stack");
    assert_eq!(rows[0].version, "1.2.3");
    assert_eq!(rows[0].content_hash, "sha256:abc");
    assert_eq!(
        rows[0].root_path,
        "/data/library/github.com%2Fship%2Ftest-stack/1.2.3"
    );
    assert!(rows[0].enabled);

    // Unregister.
    let unregistered = StackUnregistered {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        root_path: "/data/library/github.com%2Fship%2Ftest-stack/1.2.3".into(),
        unregistered_at: "2026-07-13T01:00:00Z".into(),
    };
    let env_unreg = EventEnvelope::new(STACK_UNREGISTERED, "library", &unregistered).unwrap();
    {
        let mut conn = runtime_conn(&data_dir);
        fold(&projection, &env_unreg, &mut conn);
    }
    let rows = ship_substrate::db::spaces::list_installed_at(&data_dir).unwrap();
    assert_eq!(rows.len(), 0, "stack.unregistered deletes the row");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn library_projection_installed_is_idempotent_on_resync() {
    let ship_home = tempfile::tempdir().unwrap();
    let _guard = EnvGuard::set(
        ship_substrate::env::Var::Home,
        ship_home.path().to_str().unwrap(),
    );
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();
    let _ = runtime_conn(&data_dir);

    let projection = LibraryProjection;
    let installed = StackInstalled {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        root_path: "/data/library/github.com%2Fship%2Ftest-stack/1.2.3".into(),
        content_hash: "sha256:abc".into(),
        installed_at: "2026-07-13T00:00:00Z".into(),
    };
    let env = EventEnvelope::new(STACK_INSTALLED, "library", &installed).unwrap();

    let mut conn = runtime_conn(&data_dir);
    fold(&projection, &env, &mut conn);
    fold(&projection, &env, &mut conn);
    let rows = ship_substrate::db::spaces::list_installed_at(&data_dir).unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn library_projection_unregistered_for_missing_path_is_noop() {
    let ship_home = tempfile::tempdir().unwrap();
    let _guard = EnvGuard::set(
        ship_substrate::env::Var::Home,
        ship_home.path().to_str().unwrap(),
    );
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();
    let _ = runtime_conn(&data_dir);

    let projection = LibraryProjection;
    let unregistered = StackUnregistered {
        path: "github.com/ship/never-installed".into(),
        version: "1.0.0".into(),
        root_path: "/data/library/never-installed/1.0.0".into(),
        unregistered_at: "2026-07-13T01:00:00Z".into(),
    };
    let env = EventEnvelope::new(STACK_UNREGISTERED, "library", &unregistered).unwrap();

    let mut conn = runtime_conn(&data_dir);
    fold(&projection, &env, &mut conn);
    let rows = ship_substrate::db::spaces::list_installed_at(&data_dir).unwrap();
    assert_eq!(
        rows.len(),
        0,
        "unregistered for never-installed path stays empty"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn library_projection_recognises_only_its_event_types() {
    let ship_home = tempfile::tempdir().unwrap();
    let _guard = EnvGuard::set(
        ship_substrate::env::Var::Home,
        ship_home.path().to_str().unwrap(),
    );
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();
    let _ = runtime_conn(&data_dir);

    let projection = LibraryProjection;
    assert_eq!(projection.event_types(), &[STACK_INSTALLED, STACK_UNREGISTERED]);
    assert_eq!(projection.name(), "library");
}
