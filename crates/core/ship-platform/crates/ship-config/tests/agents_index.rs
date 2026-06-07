//! Read-only agent-def index proofs (S1 — agents arc).
//!
//! The agent def file (`.ship/agents/<id>.jsonc`) is TRUTH; the index is a
//! one-way read model derived FROM the files (mirrors `ship-collection`'s
//! DataStore-backed read model, but the source is FILES, not event payloads).
//! These proofs assert the spec's test-first contract:
//!   - write a def file → index row appears with correct fields + content_hash;
//!   - edit a def file → re-index reflects new content (new content_hash);
//!   - delete a def file → re-index drops the row;
//!   - query/filter by scope and by name returns the right rows;
//!   - the index exposes NO write-back-to-file path (read-only invariant).
//!
//! Every test binds the index to an in-memory-ish per-test SQLite db via the
//! substrate `DataStore` port; defs are authored through the SAME `FsBroker`
//! door the reader uses (zero raw fs in the crate under test).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ship_config::agents_index::{AgentDefIndex, AgentDefQuery};
use ship_substrate::data::DataService;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{
    DataStore, FsBroker, FsCap, FsMode, FsRoot, LocalFsBroker, SingleBackendDataStore,
    SpaceId as PortSpaceId,
};

// ── Fixtures ────────────────────────────────────────────────────────────────

/// Canonical project `.ship` dir under a fresh tempdir. `/tmp` is symlinked on
/// WSL/macOS and the FsBroker's symlink-escape guard canonicalizes the deepest
/// existing ancestor, so the cap root must be canonical — production dirs are.
fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    ShipHome::project_ship_dir(base)
}

/// The FsCap rooted at `<ship_dir>/agents` for authoring def files in tests —
/// the same confinement the index reader builds from.
fn agents_cap(ship_dir: &Path) -> (FsCap, PathBuf) {
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(PortSpaceId::new("config")),
        FsMode::ReadWrite,
    );
    (cap, dir)
}

/// An index bound to a fresh per-test db. The index is read-only over the FILES;
/// the db is only its derived query surface.
async fn make_index(ship_dir: &Path) -> AgentDefIndex {
    let db = ship_substrate::db::space_db_path(ship_dir, "idx-test");
    let svc = DataService::open_at(&db).unwrap();
    let store: Arc<dyn DataStore> = Arc::new(SingleBackendDataStore::new(svc, "idx-test"));
    let index = AgentDefIndex::new(store);
    index.ensure_table().await.unwrap();
    index
}

/// Author a canonical def file through the broker (no std::fs writes of content).
fn write_def(fs: &LocalFsBroker, cap: &FsCap, id: &str, doc: serde_json::Value) {
    fs.write_atomic(cap, &format!("{id}.jsonc"), doc.to_string().as_bytes())
        .unwrap();
}

// ── Test 1: write a def → index row appears with correct fields + hash ────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn write_def_yields_index_row_with_fields_and_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, _) = agents_cap(&dir);

    write_def(
        &fs,
        &cap,
        "code-reviewer",
        serde_json::json!({
            "agent": {
                "id": "code-reviewer",
                "name": "Code Reviewer",
                "role": "worker",
                "tags": ["review", "testing"],
                "harnesses": ["claude", "codex"]
            },
            "skills": { "refs": ["review-pr"] }
        }),
    );

    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 1, "exactly one def row");
    let row = &rows[0];
    assert_eq!(row.id, "code-reviewer");
    assert_eq!(row.name, "Code Reviewer");
    assert_eq!(row.scope, "project");
    assert_eq!(row.path, "code-reviewer.jsonc");
    assert_eq!(row.role.as_deref(), Some("worker"));
    assert_eq!(row.tags, vec!["review".to_string(), "testing".to_string()]);
    assert_eq!(row.harness, vec!["claude".to_string(), "codex".to_string()]);
    assert!(!row.content_hash.is_empty(), "content_hash must be derived from the file");
}

// ── Test 2: edit a def → re-index reflects new content (new content_hash) ──────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn edit_def_reindex_reflects_new_content_and_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, _) = agents_cap(&dir);

    write_def(
        &fs,
        &cap,
        "alpha",
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha", "role": "worker" } }),
    );
    let index = make_index(&dir).await;
    index.reindex_path(&fs, &dir, "project", "alpha.jsonc").await.unwrap();
    let before = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(before.len(), 1);
    let hash_before = before[0].content_hash.clone();
    assert_eq!(before[0].name, "Alpha");

    // Edit the file: change name + role. The hash MUST change with the content.
    write_def(
        &fs,
        &cap,
        "alpha",
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha Prime", "role": "lead" } }),
    );
    index.reindex_path(&fs, &dir, "project", "alpha.jsonc").await.unwrap();

    let after = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(after.len(), 1, "still one row (upsert, not duplicate)");
    assert_eq!(after[0].name, "Alpha Prime", "index reflects the edited content");
    assert_eq!(after[0].role.as_deref(), Some("lead"));
    assert_ne!(
        after[0].content_hash, hash_before,
        "edited content yields a NEW content_hash"
    );
}

// ── Test 3: delete a def → re-index drops the row ─────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_def_reindex_drops_row() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, agents_dir) = agents_cap(&dir);

    write_def(&fs, &cap, "keep", serde_json::json!({ "agent": { "id": "keep", "name": "Keep" } }));
    write_def(&fs, &cap, "gone", serde_json::json!({ "agent": { "id": "gone", "name": "Gone" } }));

    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();
    assert_eq!(index.query(AgentDefQuery::default()).await.unwrap().len(), 2);

    // Delete the file on disk, then re-index that path: the row must drop.
    std::fs::remove_file(agents_dir.join("gone.jsonc")).unwrap();
    index.reindex_path(&fs, &dir, "project", "gone.jsonc").await.unwrap();

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 1, "deleted def's row is dropped");
    assert_eq!(rows[0].id, "keep");
}

// ── Test 4: query/filter by scope and by name returns the right rows ──────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_filters_by_scope_and_by_name() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, _) = agents_cap(&dir);

    write_def(&fs, &cap, "alpha", serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }));
    write_def(&fs, &cap, "beta", serde_json::json!({ "agent": { "id": "beta", "name": "Beta" } }));

    let index = make_index(&dir).await;
    // Index the same two files under two scopes (project + user) to prove the
    // scope filter partitions the rows.
    index.reindex_dir(&fs, &dir, "project").await.unwrap();
    index.reindex_dir(&fs, &dir, "user").await.unwrap();

    // Filter by scope: only the user-scoped rows.
    let user_rows = index
        .query(AgentDefQuery { scope: Some("user".into()), name: None })
        .await
        .unwrap();
    assert_eq!(user_rows.len(), 2);
    assert!(user_rows.iter().all(|r| r.scope == "user"));

    // Filter by name within a scope.
    let by_name = index
        .query(AgentDefQuery { scope: Some("project".into()), name: Some("Beta".into()) })
        .await
        .unwrap();
    assert_eq!(by_name.len(), 1);
    assert_eq!(by_name[0].id, "beta");
    assert_eq!(by_name[0].scope, "project");
}

// ── Test 5: read-only invariant — the index exposes no write-back-to-file path ─
//
// This is a *compile-time* proof rather than a runtime assertion: the index's
// public surface offers query + re-index-from-files only. There is no method
// that writes a def file. We assert the surface we DO have is read+derive, and
// that authoring still flows through the FsBroker (the reader/writer seam),
// never the index. The absence of a write-back method is enforced by the public
// API: `AgentDefIndex` has no `save`/`write`/`put_file` method to call here.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn index_is_read_only_over_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, agents_dir) = agents_cap(&dir);

    write_def(&fs, &cap, "alpha", serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }));
    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();

    // Capture the on-disk bytes BEFORE any index operation.
    let before = std::fs::read(agents_dir.join("alpha.jsonc")).unwrap();

    // Exercise every read path on the index — none of these may touch the file.
    let _ = index.query(AgentDefQuery::default()).await.unwrap();
    let _ = index
        .query(AgentDefQuery { scope: Some("project".into()), name: None })
        .await
        .unwrap();
    index.reindex_path(&fs, &dir, "project", "alpha.jsonc").await.unwrap();

    let after = std::fs::read(agents_dir.join("alpha.jsonc")).unwrap();
    assert_eq!(before, after, "index operations never write back to the def file");
}
