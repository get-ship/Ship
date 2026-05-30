/// Integration tests — artifact promote (scope escalation).
///
/// Promote moves an artifact from a narrower scope (run) to a wider one
/// (project, user). Events and blob GC behavior are asserted here.
use anyhow::Result;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::events::filter::EventFilter;
use ship_types::event_types::ARTIFACT_MOVED;
use tempfile::tempdir;

use crate::artifacts::{ArtifactScope, ArtifactService};
use ship_substrate::data::DataService;
use ship_substrate::events::KernelRouter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_kernel(ship_dir: &std::path::Path) -> Arc<Mutex<KernelRouter>> {
    ship_substrate::db::ensure_db_at(ship_dir).unwrap();
    let owned: PathBuf = ship_dir.to_path_buf();
    let leaked = Box::leak(Box::new(owned));
    Arc::new(Mutex::new(KernelRouter::new(leaked.clone()).unwrap()))
}

/// Returns (svc, data, kernel, ship_dir, _tmp_keep_alive).
fn setup() -> (ArtifactService, DataService, Arc<Mutex<KernelRouter>>, PathBuf, tempfile::TempDir) {
    let tmp = tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    crate::install_defaults();
    let svc = ArtifactService::new(&ship_dir);
    let data = DataService::open(&ship_dir).unwrap();
    let kernel = make_kernel(&ship_dir);
    (svc, data, kernel, ship_dir, tmp)
}

fn project_scope(slug: &str) -> ArtifactScope {
    ArtifactScope::Project { space_id: slug.to_string() }
}

fn run_scope(slug: &str, run: &str) -> ArtifactScope {
    ArtifactScope::Run { space_id: slug.to_string(), run_id: run.to_string() }
}

fn user_scope(uid: &str) -> ArtifactScope {
    ArtifactScope::User { user_id: uid.to_string() }
}

fn query_events_at(
    ship_dir: &std::path::Path,
    event_type: &str,
) -> Vec<ship_types::envelope::EventEnvelope> {
    let store = SqliteEventStore::new_at(ship_dir).expect("open test event store");
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .expect("query events")
}

// ── scope transitions ─────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_run_to_project_clears_run_id() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let src = run_scope("prj", "r1");
    let rec = svc
        .store(&mut data, &kernel, "artifact.bin", "application/octet-stream", b"data",
               &src, None, None, None, None)
        .await?;

    let promoted = svc
        .promote(&mut data, &kernel, &rec.id, project_scope("prj"))
        .await?;

    assert_eq!(promoted.scope, "project");
    assert_eq!(promoted.space_id.as_deref(), Some("prj"));
    assert!(promoted.run_id.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_run_to_project_bytes_preserved() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let src = run_scope("p", "r");
    let rec = svc
        .store(&mut data, &kernel, "x.bin", "application/octet-stream", b"content",
               &src, None, None, None, None)
        .await?;

    let promoted = svc
        .promote(&mut data, &kernel, &rec.id, project_scope("p"))
        .await?;

    let (_, bytes) = svc.get(&mut data, &promoted.id)?.unwrap();
    assert_eq!(bytes, b"content");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_run_to_project_old_id_gone() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let src = run_scope("p", "r");
    let rec = svc
        .store(&mut data, &kernel, "y.bin", "application/octet-stream", b"y",
               &src, None, None, None, None)
        .await?;
    let old_id = rec.id.clone();

    svc.promote(&mut data, &kernel, &old_id, project_scope("p")).await?;

    assert!(
        svc.get_record(&mut data, &old_id)?.is_none(),
        "old metadata row must be deleted after promote"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_emits_artifact_moved() -> Result<()> {
    let (svc, mut data, kernel, ship_dir, _tmp) = setup();
    let src = run_scope("p", "r-evt");
    let rec = svc
        .store(&mut data, &kernel, "z.bin", "application/octet-stream", b"z",
               &src, None, None, None, None)
        .await?;

    let promoted = svc
        .promote(&mut data, &kernel, &rec.id, project_scope("p"))
        .await?;

    let events = query_events_at(&ship_dir, ARTIFACT_MOVED);
    assert!(
        events.iter().any(|e| e.entity_id == promoted.id),
        "expected artifact.moved event with entity_id={}; got {events:?}", promoted.id
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_project_to_user_scope() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let rec = svc
        .store(&mut data, &kernel, "report.pdf", "application/pdf", b"pdf",
               &project_scope("p"), None, None, None, None)
        .await?;

    let promoted = svc
        .promote(&mut data, &kernel, &rec.id, user_scope("bob"))
        .await?;

    assert_eq!(promoted.scope, "user");
    assert_eq!(promoted.user_id.as_deref(), Some("bob"));
    assert!(promoted.space_id.is_none());
    let (_, bytes) = svc.get(&mut data, &promoted.id)?.unwrap();
    assert_eq!(bytes, b"pdf");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_run_to_user_scope() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let rec = svc
        .store(&mut data, &kernel, "x.bin", "application/octet-stream", b"x",
               &run_scope("p", "r"), None, None, None, None)
        .await?;

    let promoted = svc
        .promote(&mut data, &kernel, &rec.id, user_scope("alice"))
        .await?;

    assert_eq!(promoted.scope, "user");
    assert_eq!(promoted.user_id.as_deref(), Some("alice"));
    assert!(promoted.run_id.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_missing_artifact_errors() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let err = svc
        .promote(&mut data, &kernel, "ghost-id", project_scope("p"))
        .await;
    assert!(err.is_err(), "promote on missing artifact must return Err");
    Ok(())
}

/// Promoting bytes that already exist in the target scope must dedup —
/// no duplicate blob file written.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_promote_dedup_at_target_scope() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let bytes = b"shared";

    // Pre-store the same bytes in the target scope.
    let existing = svc
        .store(&mut data, &kernel, "existing.bin", "application/octet-stream", bytes,
               &project_scope("p"), None, None, None, None)
        .await?;

    // Store in run scope and promote to project scope.
    let run_rec = svc
        .store(&mut data, &kernel, "run.bin", "application/octet-stream", bytes,
               &run_scope("p", "r"), None, None, None, None)
        .await?;
    let promoted = svc
        .promote(&mut data, &kernel, &run_rec.id, project_scope("p"))
        .await?;

    // Both the existing project-scope artifact and the promoted one should
    // share the same blob path (dedup at target scope).
    assert_eq!(
        promoted.path, existing.path,
        "promoted artifact should reuse existing blob at target scope"
    );
    Ok(())
}
