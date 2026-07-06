//! Phase-3 roll-up provenance proofs (P3-J3 — agents-domain index provenance,
//! F4 in `census-phase3-agents.md`).
//!
//! `AgentDefIndex::upsert_composed` ingests a `compiler::compose_space`
//! result: every winning `AgentProfile` becomes an ACTIVE row (provenance =
//! the extension that supplied it, `shadowed_by: None`); every
//! `AgentProfile`-noun `ShadowRecord` the law recorded becomes a SEPARATE row
//! carrying `shadowed_by` — the index audits both "where did this def come
//! from" and "why is it not active" without a second write path.
//!
//! These proofs build the roll-up fixture directly via `compiler::compose_space`
//! (a pure fn already exercised by `crates/core/compiler/tests/p3_compose.rs`)
//! rather than through `ship_resolver::roll_up`, since ship-resolver is out of
//! this job's file fence — `upsert_composed` only depends on the
//! `compiler::ComposedLibrary` shape, not on how it was produced.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use compiler::{
    compose_space, AgentProfile, ExtSource, ExtensionDescriptor, ExtensionLibrary, ProfileMeta,
    RollupInputs,
};
use ship_config::agents_index::{AgentDefIndex, AgentDefQuery};
use ship_substrate::data::DataService;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{
    DataStore, FsBroker, FsCap, FsMode, FsRoot, LocalFsBroker, SingleBackendDataStore,
    SpaceId as PortSpaceId,
};

// ── Fixtures ────────────────────────────────────────────────────────────────

fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    ShipHome::project_ship_dir(base)
}

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

async fn make_index(ship_dir: &Path) -> AgentDefIndex {
    let db = ship_substrate::db::space_db_path(ship_dir, "idx-test");
    let svc = DataService::open_at(&db).unwrap();
    let store: Arc<dyn DataStore> = Arc::new(SingleBackendDataStore::new(svc, "idx-test"));
    let index = AgentDefIndex::new(store);
    index.ensure_table().await.unwrap();
    index
}

fn agent(id: &str, name: &str) -> AgentProfile {
    AgentProfile {
        agent: ProfileMeta { id: id.to_string(), name: name.to_string(), ..Default::default() },
        ..Default::default()
    }
}

fn ext(source: ExtSource, hash: &str, descriptor: ExtensionLibrary) -> ExtensionDescriptor {
    ExtensionDescriptor {
        source,
        space_root: PathBuf::from("/fixture"),
        descriptor,
        content_hash: hash.to_string(),
    }
}

/// A cached-extension agent (`UserGlobal`, rank 0) colliding by id with a
/// same-id source agent (`Space`, rank 1000) — the source wins per the
/// standing precedence law (`RollupInputs::default_precedence`).
fn cached_vs_source_collision() -> compiler::ComposedLibrary {
    let cached = ExtensionLibrary {
        agent_profiles: vec![agent("widget", "Cached Widget")],
        ..Default::default()
    };
    let source = ExtensionLibrary {
        agent_profiles: vec![agent("widget", "Source Widget")],
        ..Default::default()
    };
    let inputs = RollupInputs {
        extensions: vec![
            ext(ExtSource::UserGlobal, "hash-cached", cached),
            ext(ExtSource::Space, "hash-source", source),
        ],
        precedence: RollupInputs::default_precedence(),
    };
    let mut composed = compose_space(&inputs).expect("compose must succeed");
    // `ship_resolver::roll_up` stamps `rollup_inputs` onto the composed
    // library after calling `compose_space`; replicate that here since this
    // test builds the fixture directly.
    composed.library.rollup_inputs = Some(inputs);
    composed
}

// ── Test 1: boot roll-up — source active, cached shadowed ──────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn roll_up_ingest_marks_source_active_and_cached_shadowed() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let index = make_index(&dir).await;
    let composed = cached_vs_source_collision();

    index.upsert_composed("project", &composed).await;

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 2, "one active row + one shadow row");

    let active = rows.iter().find(|r| r.shadowed_by.is_none()).expect("active row present");
    assert_eq!(active.name, "Source Widget", "the source wins on id collision");
    assert_eq!(active.ext_source.as_deref(), Some(serde_json::to_string(&ExtSource::Space).unwrap().as_str()));
    assert_eq!(active.tier_content_hash.as_deref(), Some("hash-source"));

    let shadow = rows.iter().find(|r| r.shadowed_by.is_some()).expect("shadow row present");
    assert_eq!(shadow.name, "Cached Widget", "the shadow row carries the LOSING content");
    assert_eq!(
        shadow.ext_source.as_deref(),
        Some(serde_json::to_string(&ExtSource::UserGlobal).unwrap().as_str())
    );
    assert_eq!(shadow.tier_content_hash.as_deref(), Some("hash-cached"));
    assert_eq!(
        shadow.shadowed_by.as_deref(),
        Some(serde_json::to_string(&ExtSource::Space).unwrap().as_str()),
        "the shadow names what beat it"
    );

    // `list_docs` — the live list-view read — serves ONLY the active row; the
    // shadow row must never surface as if it were a real, selectable def.
    let docs = index.list_docs("project").await.unwrap();
    assert_eq!(docs.len(), 1, "list_docs excludes the shadow row");
    assert_eq!(docs[0].0, "widget");
    assert_eq!(
        docs[0].1.get("agent").and_then(|a| a.get("name")).and_then(|v| v.as_str()),
        Some("Source Widget"),
    );

    // `get_doc` resolves to the ACTIVE doc, not the shadow — a collision on
    // (id, scope) must never surface ambiguous content.
    let doc = index.get_doc("widget", "project").await.unwrap().expect("active doc found");
    assert_eq!(
        doc.get("agent").and_then(|a| a.get("name")).and_then(|v| v.as_str()),
        Some("Source Widget"),
    );
    assert_eq!(
        doc.get("_provenance").and_then(|p| p.get("shadowed_by")),
        Some(&serde_json::Value::Null),
        "the active doc's own provenance carries no shadowed_by",
    );
}

// ── Test 2: rebuild is idempotent ───────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn roll_up_ingest_is_idempotent_on_rebuild() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let index = make_index(&dir).await;
    let composed = cached_vs_source_collision();

    index.upsert_composed("project", &composed).await;
    index.upsert_composed("project", &composed).await;

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 2, "re-running the SAME roll-up upserts, never duplicates");
}

// ── Test 3: plain file-event reindex still upserts/drops, provenance null ──

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn file_event_reindex_upserts_and_drops_with_null_provenance() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, agents_dir) = agents_cap(&dir);
    let index = make_index(&dir).await;

    fs.write_atomic(
        &cap,
        "keep.jsonc",
        serde_json::json!({ "agent": { "id": "keep", "name": "Keep" } })
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    fs.write_atomic(
        &cap,
        "gone.jsonc",
        serde_json::json!({ "agent": { "id": "gone", "name": "Gone" } })
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    index.reindex_dir(&fs, &dir, "project").await.unwrap();
    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter().all(|r| r.ext_source.is_none() && r.tier_content_hash.is_none() && r.shadowed_by.is_none()),
        "the plain per-scope reindex stamps no roll-up provenance"
    );

    // Delete + re-index that path: the row drops, no dangling provenance.
    std::fs::remove_file(agents_dir.join("gone.jsonc")).unwrap();
    index.reindex_path(&fs, &dir, "project", "gone.jsonc").await.unwrap();

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 1, "deleted def's row is dropped");
    assert_eq!(rows[0].id, "keep");
}
