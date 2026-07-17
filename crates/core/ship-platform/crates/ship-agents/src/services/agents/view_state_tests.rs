//! Tests for `views::agents_state`. Kept in a sibling file so the module
//! itself stays under the 300-line cap.
//!
//! These drive the `*_from_space` variants directly, passing an explicit
//! `space_root` so parallel tests don't race on the process-global
//! project-discovery walk `list_agents`/`load_agent`/`agent_id_exists` do.

use crate::services::agents::view_state::*;
use std::path::Path;
use tempfile::tempdir;

// `write_agent` / `write_project_agent` (the `.ship/agents/*.jsonc` and
// `<ship>/library/agents/*.jsonc` two-scope scan fixtures) retired with the
// format-migration Wave 5 follow-up (job-spec-fm-w5b-dispatch-index.md):
// the no-index posture no longer scans either directory — agents arrive
// exclusively as `defineAgent` declare-forms in `stack.config.ts`, composed
// via `ship_resolver::roll_up_blocking` (see `write_ts_space` below).

/// A minimal Ship space: `.ship/` (required for `ShipHome::project_ship_dir`
/// discovery) plus a `stack.config.ts` at the space root. `ts_body` is the
/// FULL `stack.config.ts` source (import + `defineAgent(...)` calls) — the
/// no-index posture's only agent-authoring surface post format-migration
/// Wave 5.
fn write_ts_space(root: &Path, ts_body: &str) {
    std::fs::create_dir_all(root.join(".ship")).unwrap();
    std::fs::write(root.join("stack.config.ts"), ts_body).unwrap();
}

/// Regression (job-spec-fm-w5b-dispatch-index.md acceptance): a `with_roots`
/// (no `AgentDefIndex`) service resolves an agent declared ONLY in
/// `stack.config.ts` — there is no `.ship/agents/` directory anywhere on
/// disk. Exercises the exact shape `agent_id_exists` (the dispatch-gate
/// check) and the summary/detail reads use.
#[test]
fn composed_space_resolves_agent_declared_only_in_ts_config() {
    let tmp = tempdir().unwrap();
    write_ts_space(
        tmp.path(),
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "commander", name: "Commander" },
  skills: { refs: ["cmdr", "write-adr", "ship-agents"] },
  mcp: { servers: ["ship", "typescript"] },
  rules: { refs: ["discipline"] },
  permissions: { preset: "ship-autonomous" },
});
"#,
    );
    assert!(
        !tmp.path().join(".ship").join("agents").exists(),
        "fixture must have no .ship/agents/ dir — the whole point of the regression"
    );

    assert!(agent_exists_in_space(tmp.path(), "commander"));
    assert!(!agent_exists_in_space(tmp.path(), "nope"));

    let out = list_agents_from_space(tmp.path());
    assert_eq!(out.len(), 1);
    let commander = &out[0];
    assert_eq!(commander.name, "Commander");
    assert_eq!(commander.skills_count, 3);
    assert_eq!(commander.mcp_count, 2);
    assert_eq!(commander.rules_count, 1);
    assert_eq!(commander.source, SOURCE_PROJECT);

    let detail = load_agent_from_space(tmp.path(), "commander").unwrap();
    assert_eq!(detail.skills.len(), 3);
    assert_eq!(detail.mcp_servers, vec!["ship", "typescript"]);
    assert_eq!(detail.permissions_tier, "autonomous");
}

#[test]
fn empty_space_returns_empty_vec() {
    let tmp = tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".ship")).unwrap();
    assert!(list_agents_from_space(tmp.path()).is_empty());
    assert!(!agent_exists_in_space(tmp.path(), "anything"));
}

#[test]
fn load_agent_returns_none_for_missing() {
    let tmp = tempdir().unwrap();
    write_ts_space(
        tmp.path(),
        r#"
import { defineAgent } from "ship";
defineAgent({ agent: { id: "coder", name: "Coder" } });
"#,
    );
    assert!(load_agent_from_space(tmp.path(), "nope").is_none());
}

#[test]
fn load_agent_returns_detail_with_defaults() {
    let tmp = tempdir().unwrap();
    write_ts_space(
        tmp.path(),
        r#"
import { defineAgent } from "ship";
defineAgent({
  agent: { id: "coder", name: "Coder", description: "Writes code." },
});
"#,
    );
    let d = load_agent_from_space(tmp.path(), "coder").unwrap();
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
    write_ts_space(
        tmp.path(),
        r#"
import { defineAgent } from "ship";
defineAgent({ agent: { id: "lean", name: "Lean" } });
"#,
    );

    let summaries = list_agents_from_space(tmp.path());
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

    let detail = load_agent_from_space(tmp.path(), "lean").unwrap();
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

// `malformed_files_without_agent_wrapper_are_skipped` retired with the
// format-migration Wave 5 follow-up: it pinned that a `.jsonc` file lacking
// the `agent` wrapper is silently skipped by the directory scan. There is
// no directory scan left to skip anything in — `AgentProfile` deserializes
// straight off the `defineAgent(...)` call's JSON argument (a malformed
// shape is a TS-eval-time deserialize error, `ship_resolver`'s concern, not
// this module's projection layer). `summary_from_value` / `detail_from_value`
// still guard on the `agent` key for the shared index-backed-read path
// (`composed_row_source_reflects_provenance_and_shadow_is_queryable` below
// exercises that guard through the index).

// `merges_project_and_global_scopes` / `scope_skipped_when_project_matches_global`
// retired with the format-migration Wave 5 follow-up: they pinned the
// two-scope (`<ship>/library/agents/`, `<project>/.ship/agents/`) directory
// scan-and-merge precedence. That precedence law is now
// `stack_model::compose_space`'s (Space > Cache, `RollupInputs::
// default_precedence`) — already covered by `ship-resolver`'s own roll-up
// tests (out of this job's file fence) and by
// `composed_row_source_reflects_provenance_and_shadow_is_queryable` below,
// which exercises the identical precedence law through the index-backed
// read this module's no-index path is required to match.

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

// ── Index-backed reads: the AGENTS domain owns its config ─────────────────────
//
// After a boot rebuild, remove the project `.ship/agents/*.jsonc` file. The
// list, detail, and session-spawn (`rules.inline`) reads must STILL serve —
// from the `AgentDefIndex` DataStore snapshot, not the (deleted) file.

use std::sync::Arc;
use ship_config::agents_index::{AgentDefIndex, AgentDefQuery};
use ship_substrate::data::DataService;
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
    let ship_dir = base.join(".ship");
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

// ── `source` from provenance (unit) ────────────────────────────────────────

#[test]
fn source_from_provenance_prefers_ext_source_kind_over_fallback() {
    let doc = serde_json::json!({
        "agent": { "name": "X" },
        "_provenance": {
            "ext_source": serde_json::to_string(&serde_json::json!({"kind": "cache"})).unwrap(),
        },
    });
    assert_eq!(source_from_provenance(&doc, SOURCE_PROJECT), "cache");
}

#[test]
fn source_from_provenance_falls_back_when_no_provenance() {
    let doc = serde_json::json!({ "agent": { "name": "X" } });
    assert_eq!(source_from_provenance(&doc, SOURCE_PROJECT), SOURCE_PROJECT);
}

// ── Shadowing surfaced from provenance ──────────────────────────────────────
//
// A same-id collision across two roll-up tiers: the composed index row set
// carries the WINNER as the active row (what list/detail serve) and the
// LOSER as a separate shadow row (`AgentDefIndex::query`, the audit surface
// `ship-config` already exposes — no new read path added here). Built
// directly via `stack_model::compose_space`, the same fixture shape
// `ship-config`'s `agents_index_provenance.rs` uses (ship-resolver is out of
// this job's file fence).

use stack_model::{
    compose_space, AgentProfile, ExtSource, ExtensionDescriptor, ExtensionLibrary, ProfileMeta,
    RollupInputs,
};

fn agent_profile(id: &str, name: &str) -> AgentProfile {
    AgentProfile {
        agent: ProfileMeta { id: id.to_string(), name: name.to_string(), ..Default::default() },
        ..Default::default()
    }
}

fn ext_descriptor(
    source: ExtSource,
    hash: &str,
    descriptor: ExtensionLibrary,
) -> ExtensionDescriptor {
    ExtensionDescriptor {
        source,
        space_root: std::path::PathBuf::from("/fixture"),
        descriptor,
        content_hash: hash.to_string(),
    }
}

/// A cached-package agent (`Cache`, rank 0) colliding by id with the
/// space's own same-id agent (`Space`, rank 1000) — the space wins per
/// `RollupInputs::default_precedence`.
fn space_beats_cached_package() -> stack_model::ComposedLibrary {
    let cached = ExtensionLibrary {
        agent_profiles: vec![agent_profile("reviewer", "Cached Reviewer")],
        ..Default::default()
    };
    let source = ExtensionLibrary {
        agent_profiles: vec![agent_profile("reviewer", "Space Reviewer")],
        ..Default::default()
    };
    let inputs = RollupInputs {
        extensions: vec![
            ext_descriptor(ExtSource::Cache, "hash-cached", cached),
            ext_descriptor(ExtSource::Space, "hash-source", source),
        ],
        precedence: RollupInputs::default_precedence(),
    };
    let mut composed = compose_space(&inputs).expect("compose must succeed");
    composed.library.rollup_inputs = Some(inputs);
    composed
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn composed_row_source_reflects_provenance_and_shadow_is_queryable() {
    let tmp = tempdir().unwrap();
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    let ship_dir = base.join(".ship");

    let db = ship_substrate::db::space_db_path(&ship_dir, "vs-idx-shadow-test");
    let svc = DataService::open_at(&db).unwrap();
    let store: Arc<dyn DataStore> = Arc::new(SingleBackendDataStore::new(svc, "vs-idx-shadow-test"));
    let index = AgentDefIndex::new(store);
    index.ensure_table().await.unwrap();
    index.upsert_composed("project", &space_beats_cached_package()).await;

    // The rail shows only the winner — its `source` reflects the winning
    // extension's kind, not an invented "project" label.
    let list = list_agents_from_index(&index, &ship_dir).await;
    assert_eq!(list.len(), 1, "the shadow row never surfaces as a selectable agent");
    assert_eq!(list[0].name, "Space Reviewer", "the source tier wins on id collision");
    assert_eq!(list[0].source, "space");

    let detail = load_agent_from_index(&index, &ship_dir, "reviewer")
        .await
        .expect("detail serves the winner");
    assert_eq!(detail.name, "Space Reviewer");
    assert_eq!(detail.source, "space");

    // The shadowed row's existence is still surfaceable from provenance —
    // via the SAME `AgentDefIndex` audit surface this file already imports,
    // not a parallel merge law re-derived here.
    let rows = index
        .query(AgentDefQuery { scope: Some("project".to_string()), name: None })
        .await
        .unwrap();
    let shadow = rows
        .iter()
        .find(|r| r.id == "reviewer" && r.shadowed_by.is_some())
        .expect("the losing tier's row is retained, not discarded");
    assert_eq!(shadow.name, "Cached Reviewer");
    assert_eq!(
        shadow.ext_source.as_deref().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v.get("kind").and_then(|k| k.as_str()).map(str::to_string)),
        Some("cache".to_string()),
    );
}
