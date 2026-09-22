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

use ship_settings::agents_index::{AgentDefIndex, AgentDefQuery};
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
    base.join(".ship")
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
    assert_eq!(row.tags, vec!["review".to_string(), "testing".to_string()]);
    assert_eq!(row.harness, vec!["claude".to_string(), "codex".to_string()]);
    assert!(
        !row.content_hash.is_empty(),
        "content_hash must be derived from the file"
    );
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
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }),
    );
    let index = make_index(&dir).await;
    index
        .reindex_path(&fs, &dir, "project", "alpha.jsonc")
        .await
        .unwrap();
    let before = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(before.len(), 1);
    let hash_before = before[0].content_hash.clone();
    assert_eq!(before[0].name, "Alpha");

    // Edit the file: change name. The hash MUST change with the content.
    write_def(
        &fs,
        &cap,
        "alpha",
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha Prime" } }),
    );
    index
        .reindex_path(&fs, &dir, "project", "alpha.jsonc")
        .await
        .unwrap();

    let after = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(after.len(), 1, "still one row (upsert, not duplicate)");
    assert_eq!(
        after[0].name, "Alpha Prime",
        "index reflects the edited content"
    );
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

    write_def(
        &fs,
        &cap,
        "keep",
        serde_json::json!({ "agent": { "id": "keep", "name": "Keep" } }),
    );
    write_def(
        &fs,
        &cap,
        "gone",
        serde_json::json!({ "agent": { "id": "gone", "name": "Gone" } }),
    );

    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();
    assert_eq!(
        index.query(AgentDefQuery::default()).await.unwrap().len(),
        2
    );

    // Delete the file on disk, then re-index that path: the row must drop.
    std::fs::remove_file(agents_dir.join("gone.jsonc")).unwrap();
    index
        .reindex_path(&fs, &dir, "project", "gone.jsonc")
        .await
        .unwrap();

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

    write_def(
        &fs,
        &cap,
        "alpha",
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }),
    );
    write_def(
        &fs,
        &cap,
        "beta",
        serde_json::json!({ "agent": { "id": "beta", "name": "Beta" } }),
    );

    let index = make_index(&dir).await;
    // Index the same two files under two scopes (project + user) to prove the
    // scope filter partitions the rows.
    index.reindex_dir(&fs, &dir, "project").await.unwrap();
    index.reindex_dir(&fs, &dir, "user").await.unwrap();

    // Filter by scope: only the user-scoped rows.
    let user_rows = index
        .query(AgentDefQuery {
            scope: Some("user".into()),
            name: None,
        })
        .await
        .unwrap();
    assert_eq!(user_rows.len(), 2);
    assert!(user_rows.iter().all(|r| r.scope == "user"));

    // Filter by name within a scope.
    let by_name = index
        .query(AgentDefQuery {
            scope: Some("project".into()),
            name: Some("Beta".into()),
        })
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

    write_def(
        &fs,
        &cap,
        "alpha",
        serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }),
    );
    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();

    // Capture the on-disk bytes BEFORE any index operation.
    let before = std::fs::read(agents_dir.join("alpha.jsonc")).unwrap();

    // Exercise every read path on the index — none of these may touch the file.
    let _ = index.query(AgentDefQuery::default()).await.unwrap();
    let _ = index
        .query(AgentDefQuery {
            scope: Some("project".into()),
            name: None,
        })
        .await
        .unwrap();
    index
        .reindex_path(&fs, &dir, "project", "alpha.jsonc")
        .await
        .unwrap();

    let after = std::fs::read(agents_dir.join("alpha.jsonc")).unwrap();
    assert_eq!(
        before, after,
        "index operations never write back to the def file"
    );
}

// ── Test 6: the domain owns its config — live READS serve from the snapshot ────
//
// After a boot rebuild, REMOVE the def files. `list_docs` (the list view) and
// `get_doc` (detail + session-spawn `rules.inline`) must still serve from the
// DataStore snapshot with the files gone — proving the AGENTS domain owns its
// config and `.ship/agents/*.jsonc` is only a cold-load seed.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn index_serves_docs_after_files_removed() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, agents_dir) = agents_cap(&dir);

    write_def(
        &fs,
        &cap,
        "code-reviewer",
        serde_json::json!({
            "agent": { "id": "code-reviewer", "name": "Code Reviewer", "description": "Reviews PRs" },
            "skills": { "refs": ["review-pr"] },
            "rules": { "inline": "Be terse and precise." }
        }),
    );
    write_def(
        &fs,
        &cap,
        "planner",
        serde_json::json!({ "agent": { "id": "planner", "name": "Planner" } }),
    );

    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();

    // Remove BOTH def files from disk — the cold-load seed is gone.
    std::fs::remove_file(agents_dir.join("code-reviewer.jsonc")).unwrap();
    std::fs::remove_file(agents_dir.join("planner.jsonc")).unwrap();
    assert!(!agents_dir.join("code-reviewer.jsonc").exists());

    // list_docs still returns both, projected from the snapshot.
    let mut docs = index.list_docs("project").await.unwrap();
    docs.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        docs.len(),
        2,
        "list serves from the index after files removed"
    );
    assert_eq!(docs[0].0, "code-reviewer");
    assert_eq!(docs[1].0, "planner");

    // get_doc serves the FULL doc — including rules.inline (session-spawn read).
    let doc = index
        .get_doc("code-reviewer", "project")
        .await
        .unwrap()
        .expect("indexed doc served from snapshot");
    assert_eq!(
        doc.get("agent")
            .and_then(|a| a.get("description"))
            .and_then(|v| v.as_str()),
        Some("Reviews PRs"),
        "detail fields serve from the snapshot"
    );
    assert_eq!(
        doc.get("rules")
            .and_then(|r| r.get("inline"))
            .and_then(|v| v.as_str()),
        Some("Be terse and precise."),
        "rules.inline serves from the snapshot, not the (deleted) file"
    );

    // A def that was never indexed is absent — the snapshot never fabricates.
    assert!(index.get_doc("ghost", "project").await.unwrap().is_none());
}
