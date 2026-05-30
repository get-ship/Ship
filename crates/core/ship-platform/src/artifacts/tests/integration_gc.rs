/// Integration tests — artifact GC (collect_expired).
///
/// All tests use hermetic per-test tempdir setups. Expired artifacts use
/// a past timestamp; non-expired use a far-future timestamp. GC runs
/// synchronously within the test (no shared scheduler state).
use anyhow::Result;
use tempfile::tempdir;

use crate::artifacts::{ArtifactScope, ArtifactService};
use crate::artifacts::gc::collect_expired;
use ship_substrate::data::DataService;
use ship_substrate::events::KernelRouter;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_kernel(ship_dir: &std::path::Path) -> Arc<Mutex<KernelRouter>> {
    ship_substrate::db::ensure_db_at(ship_dir).unwrap();
    let owned: std::path::PathBuf = ship_dir.to_path_buf();
    let leaked = Box::leak(Box::new(owned));
    Arc::new(Mutex::new(KernelRouter::new(leaked.clone()).unwrap()))
}

fn run_scope(slug: &str, run: &str) -> ArtifactScope {
    ArtifactScope::Run { space_id: slug.to_string(), run_id: run.to_string() }
}

fn project_scope(slug: &str) -> ArtifactScope {
    ArtifactScope::Project { space_id: slug.to_string() }
}

const PAST: &str = "2020-01-01T00:00:00Z";
const FUTURE: &str = "2099-01-01T00:00:00Z";

// ── GC behavior ───────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_gc_removes_expired_run_scope() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;
    crate::install_defaults();
    let svc = ArtifactService::new(&ship_dir);
    let mut data = DataService::open(&ship_dir)?;
    let kernel = make_kernel(&ship_dir);

    let scope = run_scope("proj", "run-001");
    let rec = svc
        .store(&mut data, &kernel, "tmp.txt", "text/plain", b"x", &scope,
               None, None, None, Some(PAST))
        .await?;

    let gc_count = collect_expired(&ship_dir, &mut data, &kernel).await?;
    assert_eq!(gc_count, 1);
    assert!(svc.get_record(&mut data, &rec.id)?.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_gc_skips_non_expired_run_scope() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;
    crate::install_defaults();
    let svc = ArtifactService::new(&ship_dir);
    let mut data = DataService::open(&ship_dir)?;
    let kernel = make_kernel(&ship_dir);

    let scope = run_scope("proj", "run-002");
    let rec = svc
        .store(&mut data, &kernel, "keep.txt", "text/plain", b"y", &scope,
               None, None, None, Some(FUTURE))
        .await?;

    let gc_count = collect_expired(&ship_dir, &mut data, &kernel).await?;
    assert_eq!(gc_count, 0);
    assert!(svc.get_record(&mut data, &rec.id)?.is_some());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_gc_skips_project_scope_even_with_past_expires() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;
    crate::install_defaults();
    let svc = ArtifactService::new(&ship_dir);
    let mut data = DataService::open(&ship_dir)?;
    let kernel = make_kernel(&ship_dir);

    // GC only targets run-scope artifacts.
    let rec = svc
        .store(&mut data, &kernel, "proj.txt", "text/plain", b"p",
               &project_scope("p"), None, None, None, Some(PAST))
        .await?;

    let gc_count = collect_expired(&ship_dir, &mut data, &kernel).await?;
    assert_eq!(gc_count, 0, "project-scope artifact must not be GC'd");
    assert!(svc.get_record(&mut data, &rec.id)?.is_some());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_gc_run_without_expires_at_is_skipped() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;
    crate::install_defaults();
    let svc = ArtifactService::new(&ship_dir);
    let mut data = DataService::open(&ship_dir)?;
    let kernel = make_kernel(&ship_dir);

    let scope = run_scope("proj", "run-no-expiry");
    let rec = svc
        .store(&mut data, &kernel, "no-expiry.txt", "text/plain", b"z", &scope,
               None, None, None, None)
        .await?;

    let gc_count = collect_expired(&ship_dir, &mut data, &kernel).await?;
    assert_eq!(gc_count, 0);
    assert!(svc.get_record(&mut data, &rec.id)?.is_some());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_gc_collects_multiple_expired() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;
    crate::install_defaults();
    let svc = ArtifactService::new(&ship_dir);
    let mut data = DataService::open(&ship_dir)?;
    let kernel = make_kernel(&ship_dir);

    for i in 0..3u8 {
        let scope = run_scope("proj", &format!("run-{i}"));
        svc.store(&mut data, &kernel, &format!("f{i}.txt"), "text/plain",
                  &[i], &scope, None, None, None, Some(PAST))
            .await?;
    }
    // One artifact that must survive.
    let keeper = svc
        .store(&mut data, &kernel, "keeper.txt", "text/plain", b"k",
               &run_scope("proj", "run-keep"), None, None, None, Some(FUTURE))
        .await?;

    let gc_count = collect_expired(&ship_dir, &mut data, &kernel).await?;
    assert_eq!(gc_count, 3);
    assert!(svc.get_record(&mut data, &keeper.id)?.is_some(), "keeper must survive");
    Ok(())
}
