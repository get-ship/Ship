//! End-to-end proof: `.ts` agent configs (authored with `defineAgent`) load into
//! the `AgentDefIndex` alongside `.jsonc` defs.
//!
//! The `.ts` path pulls V8 (agent-config-eval → ship-js isolate), so its eval
//! lives at the ingest layer, never in the V8-free config crate. This test drives
//! the SAME function chain shipd's `start_agent_def_index` runs:
//!   `read_agent_ts_sources` → `resolve_agents` → `AgentDefIndex::upsert_profile`
//! then asserts the read surface (`get_doc` / `list_docs`) serves the `.ts` agent
//! and that a `.jsonc` and a `.ts` agent coexist in one scope.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agent_config_eval::resolve_agents;
use ship_config::agents_ts::read_agent_ts_sources;
use ship_config::agents_index::AgentDefIndex;
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

/// FsCap rooted at `<ship_dir>/agents` for authoring config files in tests.
fn agents_cap(ship_dir: &Path) -> FsCap {
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    FsCap::new_unchecked(dir, FsRoot::Space(PortSpaceId::new("config")), FsMode::ReadWrite)
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

/// Ingest every `.ts` config under the agents dir the way shipd's
/// `start_agent_def_index` does: read source → eval → upsert each profile.
async fn ingest_ts(index: &AgentDefIndex, fs: &LocalFsBroker, dir: &Path) {
    for (_stem, source) in read_agent_ts_sources(fs, dir).unwrap() {
        let profiles = resolve_agents(&source).await.expect("`.ts` config evaluates");
        for profile in profiles {
            index.upsert_profile("project", &profile, None).await.unwrap();
        }
    }
}

/// A real `defineAgent` config carrying a TS type annotation (`const role: string`)
/// — the type is stripped before eval, the value survives into the profile.
const REVIEWER_TS: &str = r#"
import { defineAgent } from "ship";

const role: string = "reviewer";

defineAgent({
  agent: {
    id: "reviewer",
    name: "Reviewer",
    role,
    description: "Reviews diffs for correctness.",
  },
  skills: { refs: ["review-pr"] },
});
"#;

// ── Proof 1: a real `.ts` config loads into the index and serves via get_doc ──

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ts_agent_loads_into_index_and_serves_via_get_doc() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let cap = agents_cap(&dir);
    fs.write_atomic(&cap, "reviewer.ts", REVIEWER_TS.as_bytes()).unwrap();

    let index = make_index(&dir).await;
    ingest_ts(&index, &fs, &dir).await;

    let doc = index
        .get_doc("reviewer", "project")
        .await
        .unwrap()
        .expect("`.ts` agent indexed and served from the snapshot");
    assert_eq!(
        doc.get("agent").and_then(|a| a.get("id")).and_then(|v| v.as_str()),
        Some("reviewer"),
    );
    // The distinguishing field, carried through the TS-typed const → eval →
    // AgentProfile → index doc.
    assert_eq!(
        doc.get("agent").and_then(|a| a.get("role")).and_then(|v| v.as_str()),
        Some("reviewer"),
        "the TS-typed `role` survives eval → AgentProfile → index doc"
    );
}

// ── Proof 2: a `.jsonc` and a `.ts` agent coexist in the same agents dir ──────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ts_and_jsonc_agents_coexist_in_list_docs() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let cap = agents_cap(&dir);

    // A `.jsonc` agent and a `.ts` agent in the SAME `.ship/agents/` dir.
    fs.write_atomic(
        &cap,
        "planner.jsonc",
        serde_json::json!({ "agent": { "id": "planner", "name": "Planner" } })
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    fs.write_atomic(&cap, "reviewer.ts", REVIEWER_TS.as_bytes()).unwrap();

    let index = make_index(&dir).await;
    // `.jsonc` via the cold-load reindex; `.ts` via the eval ingest.
    index.reindex_dir(&fs, &dir, "project").await.unwrap();
    ingest_ts(&index, &fs, &dir).await;

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
        "both the `.jsonc` and `.ts` agents appear via list_docs"
    );
}
