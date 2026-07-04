//! Tests for `views::agents_state`. Kept in a sibling file so the module
//! itself stays under the 300-line cap.
//!
//! These drive `scopes_from` + `*_with_scopes` variants directly so
//! parallel tests don't race on the process-global `SHIP_DEV_PROJECT_DIR` env var.

use crate::services::agents::view_state::*;
use std::path::Path;
use tempfile::tempdir;

/// User-scope agents live at `<ship>/library/agents/` per the home-layout
/// ADR. `scopes_from` resolves the global dir through `ShipHome::vector_dir`
/// which knows the layout; tests must write to the same location.
fn write_agent(ship_dir: &Path, id: &str, body: &str) {
    let d = ship_dir.join("data").join("library").join("agents");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join(format!("{id}.jsonc")), body).unwrap();
}

/// Project-scope agents (when caller passes a `project_ship_dir`) live at
/// `<project>/.ship/agents/` (no `library/` prefix at project scope, see
/// `scopes_from`). Tests that exercise the project scope use this helper.
fn write_project_agent(project_ship_dir: &Path, id: &str, body: &str) {
    let d = project_ship_dir.join("agents");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join(format!("{id}.jsonc")), body).unwrap();
}

#[test]
fn empty_dir_returns_empty_vec() {
    let tmp = tempdir().unwrap();
    let scopes = scopes_from(tmp.path(), None);
    assert!(list_agents_with_scopes(&scopes).is_empty());
}

#[test]
fn parses_canonical_nested_shape() {
    let tmp = tempdir().unwrap();
    // Mirrors .ship/agents/commander.jsonc — the canonical shape per
    // schemas/agent.schema.json: `agent` wrapper, nested collections.
    write_agent(
        tmp.path(),
        "commander",
        r#"{
            "agent": { "name": "Commander" },
            "skills": { "refs": ["cmdr", "write-adr", "ship-agents"] },
            "mcp":    { "servers": ["ship", "typescript"] },
            "rules":  { "refs": ["discipline"] },
            "permissions": { "preset": "ship-autonomous" }
        }"#,
    );
    let scopes = scopes_from(tmp.path(), None);
    let out = list_agents_with_scopes(&scopes);
    assert_eq!(out.len(), 1);
    let commander = &out[0];
    assert_eq!(commander.name, "Commander");
    assert_eq!(commander.skills_count, 3);
    assert_eq!(commander.mcp_count, 2);
    assert_eq!(commander.rules_count, 1);

    let detail = load_agent_with_scopes(&scopes, "commander").unwrap();
    assert_eq!(detail.skills.len(), 3);
    assert_eq!(detail.mcp_servers, vec!["ship", "typescript"]);
    assert_eq!(detail.permissions_tier, "autonomous");
}

#[test]
fn load_agent_returns_none_for_missing() {
    let tmp = tempdir().unwrap();
    let scopes = scopes_from(tmp.path(), None);
    assert!(load_agent_with_scopes(&scopes, "nope").is_none());
}

#[test]
fn load_agent_returns_detail_with_defaults() {
    let tmp = tempdir().unwrap();
    write_agent(
        tmp.path(),
        "coder",
        r#"{ "agent": { "id": "coder", "name": "Coder", "description": "Writes code." } }"#,
    );
    let scopes = scopes_from(tmp.path(), None);
    let d = load_agent_with_scopes(&scopes, "coder").unwrap();
    assert_eq!(d.name, "Coder");
    assert_eq!(d.permissions_tier, "standard");
    assert_eq!(d.letter, "C");
}

/// Regression: agent profiles never carry a `harnesses` field on the
/// `agent` object (removed in the agent-profile-harness-cleanup arc).
/// The view-state structs must not surface one either — this test pins
/// both that the build doesn't panic when the field is absent and that
/// the serialized view shape excludes `harnesses` / `harnesses_count`.
#[test]
fn agent_without_harnesses_field_serializes_without_harnesses_keys() {
    let tmp = tempdir().unwrap();
    write_agent(
        tmp.path(),
        "lean",
        r#"{ "agent": { "id": "lean", "name": "Lean" } }"#,
    );
    let scopes = scopes_from(tmp.path(), None);

    let summaries = list_agents_with_scopes(&scopes);
    assert_eq!(summaries.len(), 1);
    let summary_json = serde_json::to_value(&summaries[0]).unwrap();
    assert!(
        summary_json.get("harnesses").is_none(),
        "AgentSummary must not surface a harnesses field: {summary_json}"
    );
    assert!(
        summary_json.get("harnesses_count").is_none(),
        "AgentSummary must not surface a harnesses_count field: {summary_json}"
    );

    let detail = load_agent_with_scopes(&scopes, "lean").unwrap();
    let detail_json = serde_json::to_value(&detail).unwrap();
    assert!(
        detail_json.get("harnesses").is_none(),
        "AgentDetail must not surface a harnesses field: {detail_json}"
    );
    assert!(
        detail_json.get("default_harness").is_none(),
        "AgentDetail must not surface a default_harness field: {detail_json}"
    );
}

#[test]
fn malformed_files_without_agent_wrapper_are_skipped() {
    let tmp = tempdir().unwrap();
    write_agent(tmp.path(), "flat", r#"{ "name": "Flat" }"#);
    let scopes = scopes_from(tmp.path(), None);
    assert!(list_agents_with_scopes(&scopes).is_empty());
    assert!(load_agent_with_scopes(&scopes, "flat").is_none());
}

#[test]
fn strip_jsonc_handles_comments() {
    let src = r#"{
        // line
        /* block */
        "x": 1,
    }"#;
    let v: serde_json::Value = serde_json::from_str(&strip_jsonc(src)).unwrap();
    assert_eq!(v["x"], 1);
}

#[test]
fn strip_jsonc_preserves_multibyte_utf8() {
    // Regression: the trailing-comma pass used to walk the output as
    // bytes and push `b as char`, which mangled any codepoint above
    // U+007F (e.g. emdash 0xE2 0x80 0x94 → U+00E2 U+0080 U+0094).
    let src = r#"{
        // description with an emdash
        "desc": "orchestrator — captures intent",
        "list": [1, 2,],
    }"#;
    let stripped = strip_jsonc(src);
    assert!(
        stripped.contains("orchestrator — captures intent"),
        "emdash must survive strip_jsonc verbatim: {stripped}",
    );
    let v: serde_json::Value = serde_json::from_str(&stripped).unwrap();
    assert_eq!(v["desc"], "orchestrator — captures intent");
    assert_eq!(v["list"], serde_json::json!([1, 2]));
}

#[test]
fn merges_project_and_global_scopes() {
    // Global ship_dir with one unique + one shared.
    let global_tmp = tempdir().unwrap();
    let global_ship = global_tmp.path().to_path_buf();
    write_agent(
        &global_ship,
        "g-only",
        r#"{ "agent": { "id": "g-only", "name": "Globalist" } }"#,
    );
    write_agent(
        &global_ship,
        "shared",
        r#"{ "agent": { "id": "shared", "name": "SharedGlobal", "description": "global copy" } }"#,
    );

    // Project ship_dir with one unique + one shared — project wins.
    let project_tmp = tempdir().unwrap();
    let project_root = project_tmp.path();
    let project_ship = ship_substrate::ShipHome::project_ship_dir(project_root);
    std::fs::create_dir_all(&project_ship).unwrap();
    std::fs::write(project_ship.join("ship.jsonc"), r#"{"id": "pr1"}"#).unwrap();
    write_project_agent(
        &project_ship,
        "p-only",
        r#"{ "agent": { "id": "p-only", "name": "Projector" } }"#,
    );
    write_project_agent(
        &project_ship,
        "shared",
        r#"{ "agent": { "id": "shared", "name": "SharedProject", "description": "project copy" } }"#,
    );

    let scopes = scopes_from(&global_ship, Some(&project_ship));
    let out = list_agents_with_scopes(&scopes);
    let detail = load_agent_with_scopes(&scopes, "shared").unwrap();

    let ids: Vec<&str> = out.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"g-only"), "global agent visible: {ids:?}");
    assert!(ids.contains(&"p-only"), "project agent visible: {ids:?}");
    assert_eq!(
        out.iter().filter(|a| a.id == "shared").count(),
        1,
        "dedup keeps one shared"
    );

    let g = out.iter().find(|a| a.id == "g-only").unwrap();
    assert_eq!(g.source, SOURCE_GLOBAL);
    let p = out.iter().find(|a| a.id == "p-only").unwrap();
    assert_eq!(p.source, SOURCE_PROJECT);
    let shared = out.iter().find(|a| a.id == "shared").unwrap();
    assert_eq!(shared.name, "SharedProject", "project copy wins");
    assert_eq!(shared.source, SOURCE_PROJECT);

    assert_eq!(detail.name, "SharedProject");
    assert_eq!(detail.source, SOURCE_PROJECT);
}

#[test]
fn scope_skipped_when_project_matches_global() {
    let tmp = tempdir().unwrap();
    write_agent(tmp.path(), "solo", r#"{ "agent": { "id": "solo", "name": "Solo" } }"#);
    let scopes = scopes_from(tmp.path(), Some(tmp.path()));
    let out = list_agents_with_scopes(&scopes);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].source, SOURCE_GLOBAL);
}

// ── Index-backed reads: the AGENTS domain owns its config ─────────────────────
//
// After a boot rebuild, remove the project `.ship/agents/*.jsonc` file. The
// list, detail, and session-spawn (`rules.inline`) reads must STILL serve —
// from the `AgentDefIndex` DataStore snapshot, not the (deleted) file.

use std::sync::Arc;
use ship_config::agents_index::AgentDefIndex;
use ship_substrate::data::DataService;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{DataStore, LocalFsBroker, SingleBackendDataStore};

/// Build an index over a fresh per-test space.db and reindex the project dir.
async fn build_project_index(ship_dir: &Path) -> AgentDefIndex {
    let db = ship_substrate::db::space_db_path(ship_dir, "vs-idx-test");
    let svc = DataService::open_at(&db).unwrap();
    let store: Arc<dyn DataStore> = Arc::new(SingleBackendDataStore::new(svc, "vs-idx-test"));
    let index = AgentDefIndex::new(store);
    index.ensure_table().await.unwrap();
    index
        .reindex_dir(&LocalFsBroker::new(), ship_dir, "project")
        .await
        .unwrap();
    index
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn index_backed_reads_serve_after_project_file_removed() {
    // Canonicalize: the FsBroker symlink-escape guard needs a canonical root.
    let tmp = tempdir().unwrap();
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    let ship_dir = ShipHome::project_ship_dir(base);
    let agents_dir = ship_dir.join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(
        agents_dir.join("reviewer.jsonc"),
        r#"{
            "agent": { "id": "reviewer", "name": "Reviewer", "description": "Reviews code" },
            "skills": { "refs": ["review-pr", "lint"] },
            "rules": { "inline": "Focus on correctness." }
        }"#,
    )
    .unwrap();

    let index = build_project_index(&ship_dir).await;

    // Remove the project def file — the cold-load seed is gone.
    std::fs::remove_file(agents_dir.join("reviewer.jsonc")).unwrap();
    assert!(!agents_dir.join("reviewer.jsonc").exists());

    // list serves from the index snapshot.
    let list = list_agents_from_index(&index, &ship_dir).await;
    assert_eq!(list.len(), 1, "list serves from index after file removed");
    assert_eq!(list[0].id, "reviewer");
    assert_eq!(list[0].skills_count, 2);
    assert_eq!(list[0].source, SOURCE_PROJECT);

    // detail serves from the index snapshot.
    let detail = load_agent_from_index(&index, &ship_dir, "reviewer")
        .await
        .expect("detail serves from index");
    assert_eq!(detail.name, "Reviewer");
    assert_eq!(detail.skills, vec!["review-pr", "lint"]);
    assert_eq!(detail.inline_rules, "Focus on correctness.");
    assert_eq!(detail.source, SOURCE_PROJECT);

    // session-spawn `rules.inline` serves from the index snapshot.
    let rules = crate::services::sessions::lifecycle::load_agent_rules(
        Some(&index),
        &ship_dir,
        "reviewer",
    )
    .await
    .expect("rules.inline serves from index");
    assert!(
        rules.contains("Focus on correctness."),
        "spawn preamble carries the index-served inline rules: {rules}"
    );
}
