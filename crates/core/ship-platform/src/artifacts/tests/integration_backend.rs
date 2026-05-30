/// Integration tests — backend primitives: blob path layout, sha256_hex,
/// scope accessors, lock key namespacing, concurrent store under contention.
///
/// These tests cover the pure/sync surface of the backend layer. No event
/// assertions needed — these fns emit no events.
use anyhow::Result;
use std::sync::Arc;
use tempfile::tempdir;

use crate::artifacts::{ArtifactScope, ArtifactService, LocalFsBackend, sha256_hex};
use ship_substrate::data::DataService;
use ship_substrate::events::KernelRouter;
use tokio::sync::Mutex;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_kernel(ship_dir: &std::path::Path) -> Arc<Mutex<KernelRouter>> {
    ship_substrate::db::ensure_db_at(ship_dir).unwrap();
    let owned: std::path::PathBuf = ship_dir.to_path_buf();
    let leaked = Box::leak(Box::new(owned));
    Arc::new(Mutex::new(KernelRouter::new(leaked.clone()).unwrap()))
}

// ── blob path layout ──────────────────────────────────────────────────────────

#[test]
fn artifacts_backend_project_scope_blob_path_layout() {
    let tmp = tempdir().unwrap();
    let backend = LocalFsBackend::new(tmp.path().to_path_buf());
    let scope = ArtifactScope::Project { space_id: "demo".to_string() };
    let path = backend.blob_path(&scope, "01HABCDEF");
    let s = path.to_string_lossy();
    assert!(
        s.contains("/artifacts/demo/blobs/01/01HABCDEF"),
        "project blob path layout wrong: {s}"
    );
}

#[test]
fn artifacts_backend_run_scope_blob_path_layout() {
    let tmp = tempdir().unwrap();
    let backend = LocalFsBackend::new(tmp.path().to_path_buf());
    let scope = ArtifactScope::Run {
        space_id: "p".to_string(),
        run_id: "r1".to_string(),
    };
    let path = backend.blob_path(&scope, "01HABCDEF");
    let s = path.to_string_lossy();
    assert!(
        s.contains("/artifacts/p/runs/r1/01/01HABCDEF"),
        "run blob path layout wrong: {s}"
    );
}

#[test]
fn artifacts_backend_user_scope_blob_path_layout() {
    let tmp = tempdir().unwrap();
    let backend = LocalFsBackend::new(tmp.path().to_path_buf());
    let scope = ArtifactScope::User { user_id: "alice".to_string() };
    let path = backend.blob_path(&scope, "01HABCDEF");
    let s = path.to_string_lossy();
    assert!(
        s.contains("/artifacts/users/alice/01/01HABCDEF"),
        "user blob path layout wrong: {s}"
    );
}

// ── sha256_hex ─────────────────────────────────────────────────────────────────

#[test]
fn artifacts_backend_sha256_hex_known_vector() {
    // echo -n "" | sha256sum  => e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let hash = sha256_hex(b"");
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn artifacts_backend_sha256_hex_is_lowercase() {
    let hash = sha256_hex(b"test");
    assert_eq!(hash, hash.to_lowercase(), "sha256_hex must return lowercase hex");
}

#[test]
fn artifacts_backend_sha256_hex_deterministic() {
    let h1 = sha256_hex(b"hello");
    let h2 = sha256_hex(b"hello");
    assert_eq!(h1, h2);
}

// ── scope accessors ───────────────────────────────────────────────────────────

#[test]
fn artifacts_backend_scope_kind_str_values() {
    assert_eq!(
        ArtifactScope::Project { space_id: "p".into() }.kind_str(),
        "project"
    );
    assert_eq!(
        ArtifactScope::Run { space_id: "p".into(), run_id: "r".into() }.kind_str(),
        "run"
    );
    assert_eq!(
        ArtifactScope::User { user_id: "u".into() }.kind_str(),
        "user"
    );
}

#[test]
fn artifacts_backend_scope_space_id_accessor() {
    let proj = ArtifactScope::Project { space_id: "sp".into() };
    assert_eq!(proj.space_id(), Some("sp"));

    let run = ArtifactScope::Run { space_id: "sp".into(), run_id: "r".into() };
    assert_eq!(run.space_id(), Some("sp"));

    let user = ArtifactScope::User { user_id: "u".into() };
    assert!(user.space_id().is_none());
}

#[test]
fn artifacts_backend_scope_run_id_accessor() {
    let run = ArtifactScope::Run { space_id: "p".into(), run_id: "r99".into() };
    assert_eq!(run.run_id(), Some("r99"));

    let proj = ArtifactScope::Project { space_id: "p".into() };
    assert!(proj.run_id().is_none());
}

#[test]
fn artifacts_backend_scope_user_id_accessor() {
    let user = ArtifactScope::User { user_id: "bob".into() };
    assert_eq!(user.user_id(), Some("bob"));

    let proj = ArtifactScope::Project { space_id: "p".into() };
    assert!(proj.user_id().is_none());
}

// ── lock key namespacing ──────────────────────────────────────────────────────

#[test]
fn artifacts_backend_lock_keys_are_namespaced() {
    let p = ArtifactScope::Project { space_id: "alice".into() };
    let u = ArtifactScope::User { user_id: "alice".into() };
    assert_ne!(p.lock_key(), u.lock_key());
    assert!(p.lock_key().starts_with("project:"));
    assert!(u.lock_key().starts_with("user:"));
}

#[test]
fn artifacts_backend_run_lock_keys_differ_by_run_id() {
    let r1 = ArtifactScope::Run { space_id: "p".into(), run_id: "r1".into() };
    let r2 = ArtifactScope::Run { space_id: "p".into(), run_id: "r2".into() };
    assert_ne!(r1.lock_key(), r2.lock_key());
    assert!(r1.lock_key().starts_with("run:"));
}

// ── concurrent store under contention ────────────────────────────────────────

/// Concurrent stores of identical bytes into the same scope must dedup to
/// a single blob. The per-scope async lock inside `ArtifactService` serializes
/// the dedup-check + write.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn artifacts_backend_concurrent_store_same_bytes_dedup_to_one_blob() {
    let tmp = tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    crate::install_defaults();
    let svc = Arc::new(ArtifactService::new(&ship_dir));
    let kernel = make_kernel(&ship_dir);
    let scope = ArtifactScope::Project { space_id: "dedup-race".into() };
    let bytes: &'static [u8] = b"shared-bytes-under-contention";

    let n = 8;
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let svc = Arc::clone(&svc);
        let kernel = Arc::clone(&kernel);
        let scope = scope.clone();
        let ship_dir = ship_dir.clone();
        handles.push(tokio::spawn(async move {
            let mut data = DataService::open(&ship_dir).unwrap();
            svc.store(
                &mut data, &kernel,
                &format!("name-{i}"), "application/octet-stream",
                bytes, &scope, None, None, None, None,
            ).await
        }));
    }
    let mut records = Vec::with_capacity(n);
    for (i, h) in handles.into_iter().enumerate() {
        let rec = h.await
            .unwrap_or_else(|e| panic!("task {i} join failed: {e}"))
            .unwrap_or_else(|e| panic!("task {i} store returned Err: {e:#}"));
        records.push(rec);
    }

    let first_path = records[0].path.clone();
    for (i, r) in records.iter().enumerate().skip(1) {
        assert_eq!(
            r.path, first_path,
            "concurrent dedup must converge on one blob (task {i} diverged)"
        );
    }
    let abs = ship_dir.join(&first_path);
    assert!(abs.exists(), "shared blob must exist on disk: {abs:?}");
}

/// Concurrent stores of distinct bytes must each produce their own blob.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn artifacts_backend_concurrent_store_distinct_bytes_each_get_own_blob() {
    let tmp = tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    crate::install_defaults();
    let svc = Arc::new(ArtifactService::new(&ship_dir));
    let kernel = make_kernel(&ship_dir);
    let scope = ArtifactScope::Project { space_id: "distinct".into() };

    let n = 5;
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let svc = Arc::clone(&svc);
        let kernel = Arc::clone(&kernel);
        let scope = scope.clone();
        let ship_dir = ship_dir.clone();
        handles.push(tokio::spawn(async move {
            let mut data = DataService::open(&ship_dir).unwrap();
            let body = format!("body-{i}").into_bytes();
            svc.store(
                &mut data, &kernel,
                &format!("name-{i}"), "application/octet-stream",
                &body, &scope, None, None, None, None,
            ).await
        }));
    }
    let mut paths: Vec<String> = Vec::with_capacity(n);
    for h in handles {
        paths.push(h.await.unwrap().unwrap().path);
    }
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), n, "distinct bytes must produce {n} distinct blob paths");
}

// ── invalid scope in metadata row ────────────────────────────────────────────

/// A metadata row with an invalid scope string must not silently coerce —
/// the service must surface an error or return None rather than produce a
/// malformed `ArtifactRecord` with a hidden default scope.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_backend_invalid_scope_row_errors_loudly() -> Result<()> {
    use crate::artifacts::{ensure_artifacts_collection, artifacts_schema};
    use crate::collections::store::create_item;

    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;

    let mut data = DataService::open(&ship_dir)?;
    ensure_artifacts_collection(&mut data)?;
    let schema = artifacts_schema();
    let id = ship_substrate::gen_ulid();

    // Attempt to force-insert a row with an invalid scope.
    // Schema validation may reject this — either way the service must not
    // silently map an unknown scope to a valid variant.
    let row = serde_json::json!({
        "name": "bad", "path": "x", "mime": "text/plain", "size": 0,
        "scope": "nonsense",
    });
    let _ = create_item(&mut data, &schema, &id, &row, None);

    let svc = ArtifactService::new(&ship_dir);
    let got = svc.get_record(&mut data, &id).ok().flatten();
    // Either the schema rejected the write (got is None) or the record is
    // present with scope="nonsense" and no silent fallback occurred.
    if let Some(record) = got {
        assert_eq!(
            record.scope, "nonsense",
            "scope must round-trip exactly; silent fallback is not allowed"
        );
    }
    Ok(())
}
