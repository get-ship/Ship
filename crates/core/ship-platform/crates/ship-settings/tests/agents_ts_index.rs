//! ARC A5: agent nouns are JSONC-only. This test proves `.jsonc` agents
//! load into `AgentDefIndex` via reindex (the shipd cold-load path).
//! The retired `.ts`/`defineAgent` ingest path is no longer exercised.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ship_settings::agents_index::AgentDefIndex;
use ship_substrate::data::DataService;
use ship_substrate::ports::{
    DataStore, FsBroker, FsCap, FsMode, FsRoot, LocalFsBroker, SingleBackendDataStore,
    SpaceId as PortSpaceId,
};

fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    base.join(".ship")
}

fn agents_cap(ship_dir: &Path) -> FsCap {
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    FsCap::new_unchecked(
        dir,
        FsRoot::Space(PortSpaceId::new("config")),
        FsMode::ReadWrite,
    )
}

async fn make_index(ship_dir: &Path) -> AgentDefIndex {
    let db = ship_substrate::db::space_db_path(ship_dir, "idx-test");
    let svc = DataService::open_at(&db).unwrap();
    let store: Arc<dyn DataStore> = Arc::new(SingleBackendDataStore::new(svc, "idx-test"));
    let index = AgentDefIndex::new(store);
    index.ensure_table().await.unwrap();
    index
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn jsonc_agent_loads_into_index_and_serves_via_get_doc() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let cap = agents_cap(&dir);
    fs.write_atomic(
        &cap,
        "reviewer.jsonc",
        br#"{
  "agent": {
    "id": "reviewer",
    "name": "Reviewer",
    "description": "Reviews diffs for correctness."
  },
  "skills": { "refs": ["review-pr"] }
}"#,
    )
    .unwrap();

    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();

    let doc = index
        .get_doc("reviewer", "project")
        .await
        .unwrap()
        .expect("jsonc agent indexed and served from the snapshot");
    assert_eq!(
        doc.get("agent")
            .and_then(|a| a.get("id"))
            .and_then(|v| v.as_str()),
        Some("reviewer"),
    );
    assert_eq!(
        doc.get("agent")
            .and_then(|a| a.get("description"))
            .and_then(|v| v.as_str()),
        Some("Reviews diffs for correctness."),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_jsonc_agents_coexist_in_list_docs() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let cap = agents_cap(&dir);

    fs.write_atomic(
        &cap,
        "planner.jsonc",
        serde_json::json!({ "agent": { "id": "planner", "name": "Planner" } })
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    fs.write_atomic(
        &cap,
        "reviewer.jsonc",
        serde_json::json!({ "agent": { "id": "reviewer", "name": "Reviewer" } })
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    let index = make_index(&dir).await;
    index.reindex_dir(&fs, &dir, "project").await.unwrap();

    let mut ids: Vec<String> = index
        .list_docs("project")
        .await
        .unwrap()
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    ids.sort();
    assert_eq!(
        ids,
        vec!["planner".to_string(), "reviewer".to_string()],
        "both jsonc agents appear via list_docs"
    );
}
