//! Phase-3 spine smoke — the exemplar loop PROVEN over one real on-disk
//! fixture, not asserted piecemeal.
//!
//! Each piece here is independently proven elsewhere: `ship-resolver`'s own
//! `roll_up` tests (`crates/core/ship-resolver/src/lib.rs`), `ship-config`'s
//! `agents_index_provenance.rs` (`AgentDefIndex::upsert_composed`),
//! `ship-agents`' `view_state_tests.rs` (the index-backed view surfacing a
//! shadow), and `ship-resolver`'s `cache.rs` (kernel-log invalidation). This
//! file is the one place that walks all four across the SAME fixture, in
//! the SAME process, cross-crate, without the shipd boot wall:
//!
//!   disk (source + cached package) → `roll_up` (source wins, shadow
//!   recorded) → `AgentDefIndex::upsert_composed` (winner row + shadow row)
//!   → the agents view's index-backed reads (`ship_agents::services::agents::
//!   view_state`) → a `config.file_changed` kernel event → the descriptor
//!   cache invalidates → the next resolve recomposes with an edit visible.
//!
//! The fixture's rule body lives on the agent composition as
//! `defineAgent({ rules: { inline } })` (the rules-format cut
//! collapsed the on-disk `rules/<id>.md` convention — rules arrive
//! exclusively via the inline block on the agent). The change leg
//! mutates the agent's inline body to prove the recompose is
//! visible. This file does NOT touch the V8 isolate: the rule
//! body is plain `rs` text, not a TS source — the
//! `write_agent_ts_with_rule` helper writes the rendered TS to
//! disk on each change so the resolver's eval picks up the new
//! body on the next roll-up.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use harness_config_compiler::compile;
use stack_model::{ExtSource, resolve_library};
use ship_agents::services::agents::view_state::{list_agents_from_index, load_agent_from_index};
use ship_config::agents_index::{AgentDefIndex, AgentDefQuery};
use ship_resolver::cache::DescriptorCache;
use ship_resolver::descriptor::{extension_library_canonical, extension_library_content_hash};
use ship_resolver::roll_up_members;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;

use ship_substrate::ports::{DataStore, KernelLog, SingleBackendDataStore, SpaceId};
use ship_types::payload::ConfigFileChanged;
use ship_types::{EventEnvelope, event_types};

// ── Fixture ─────────────────────────────────────────────────────────────────

/// The cached package's lockfile path, `example.test` per RFC 2606 (a
/// reserved, unroutable domain — a neutral fixture id, not a real host).
const PKG_PATH: &str = "example.test/acme/kit";
const PKG_HEX: &str = "f00dcafe";
/// The cached package's bare agent id is `helper`; `load_cached_extensions`
/// namespaces it to `<PKG_PATH>/helper` on roll-up. The source authors an
/// agent under that EXACT post-namespace id to force the collision — the
/// same shape `ship-resolver`'s own
/// `roll_up_source_shadows_cached_agent_on_id_collision` test uses.
const COLLIDING_ID: &str = "example.test/acme/kit/helper";

const SPINE_CONFIG_TS_MANIFEST: &str = r#"import { defineConfig } from "ship";

export default defineConfig({});
"#;
/// One space root with a colliding source + cached-package fixture:
/// - `stack.config.ts` (thin POINTER manifest: empty `defineConfig({})`).
/// - `agents/<id>.ts` — source's colliding `helper` agent, with
///   the inline `rules: { inline: "Be terse." }` block; the change
///   leg rewrites the agent's inline rule body to prove the
///   recompose is visible.
/// - `skills/review/SKILL.md` — the source's on-disk skill.
/// Write the source's agent (with the given rule body) into the
/// on-disk `agents/<id>.ts`. The id is the space-relative id of
/// the cached package's agent (per `COLLIDING_ID`); the body is
/// the same `defineAgent` shape the W4/W5 inline form used
/// (slice 2c's per-noun source dir convention). The rules-format
/// cut moved the rule body from the on-disk `rules/<id>.md`
/// convention to the agent's `rules: { inline: "..." }` block —
/// the change leg mutates that block to prove the recompose is
/// visible. The file name is sanitized for the FS (slashes → `__`).
fn write_agent_ts(space_root: &Path, id: &str) {
    write_agent_ts_with_rule(space_root, id, "Be terse.");
}
fn write_agent_ts_with_rule(space_root: &Path, id: &str, rule_body: &str) {
    let stem = id.replace('/', "__");
    let rendered = format!(
        r#"
import {{ defineAgent }} from "ship";

defineAgent({{
  agent: {{ id: "{id}", name: "Source Helper" }},
  rules: {{ inline: {rule_body:?} }},
}});
"#,
    );
    std::fs::write(space_root.join("agents").join(format!("{stem}.ts")), rendered).unwrap();
}


/// Author the source's skill on-disk (slice 2b close-out convention):
/// `skills/<id>/SKILL.md` is the per-extension-project source surface
/// for the skill shape, the same shape `defineSkill`'s
/// `{ frontmatter, body }` mapped onto `stack_model::Skill`. The
/// frontmatter keys here match `stack_model::skill_parser::parse_skill`'s
/// expected shape (name, description).
fn write_skill_md(space_root: &Path) {
    let dir = space_root.join("skills").join("review");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("SKILL.md"),
        "---\nname: review\ndescription: Review skill fixture\n---\nReview content.\n",
    )
    .unwrap();
}

fn write_colliding_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let space_root = std::fs::canonicalize(tmp.path()).unwrap();
    // Marker cutover: no .ship dir — the manifest marks the root and
    // ship.lock sits beside it.

    // 1. Thin pointer manifest.
    std::fs::write(
        space_root.join("stack.config.ts"),
        SPINE_CONFIG_TS_MANIFEST,
    )
    .unwrap();

    // 2. The colliding agent (the `helper` cached id, namespaced to
    //    the package's full id — per `COLLIDING_ID`).
    std::fs::create_dir_all(space_root.join("agents")).unwrap();
    write_agent_ts(&space_root, COLLIDING_ID);

    // 3. The source's on-disk skill (slice 2b close-out convention).
    write_skill_md(&space_root);



    let cache_root = space_root.join("cache");
    let obj = cache_root.join("objects").join(PKG_HEX);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        format!(
            r#"{{ "module": {{ "name": "{PKG_PATH}", "version": "1.0.0" }},
                 "exports": {{ "skills": [], "agents": ["agents/helper.jsonc"] }} }}"#
        ),
    )
    .unwrap();
    std::fs::write(
        obj.join("agents/helper.jsonc"),
        r#"{ "agent": { "id": "helper", "name": "Cached Helper" } }"#,
    )
    .unwrap();

    std::fs::write(
        space_root.join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "{PKG_PATH}", "version": "1.0.0",
                    "commit": "{commit}", "hash": "sha256:{PKG_HEX}" }}
             ] }}"#,
            commit = "c".repeat(40),
        ),
    )
    .unwrap();

    (tmp, space_root, cache_root)
}

/// Write the change-leg agent with the new rule body. The
/// `write_agent_ts_with_rule` helper renders a `defineAgent`
/// call whose `rules: { inline }` block carries the given body;
/// only the rule body mutates between the pre-edit and the
/// change-leg resolve, proving the recompose is visible. The
/// `COLLIDING_ID` is fixed across edits (the cached-package
/// collision shape `ship-resolver`'s own
/// `roll_up_source_shadows_cached_agent_on_id_collision` test
/// exercises); the agent + skill declarations are stable.


async fn make_index(ship_dir: &Path, db_name: &str) -> AgentDefIndex {
    let db = ship_substrate::db::space_db_path(ship_dir, db_name);
    let svc = DataService::open_at(&db).unwrap();
    let store: Arc<dyn DataStore> = Arc::new(SingleBackendDataStore::new(svc, db_name));
    let index = AgentDefIndex::new(store);
    index.ensure_table().await.unwrap();
    index
}

/// The real resolve pipeline: `roll_up` → `resolve_library` → `compile` for
/// the `claude` harness. Used as the descriptor cache's `load_fn` in the
/// change leg — no stand-in, the production pipeline is what runs.
async fn resolve_claude_output(
    space_root: &Path,
    cache_root: &Path,
) -> anyhow::Result<harness_config_compiler::CompileOutput> {
    let composed = roll_up_members(&[space_root.to_path_buf()], Some(cache_root), None).await?;
    let resolved = resolve_library(&composed.library, None, None);
    compile(&resolved, "claude").ok_or_else(|| anyhow::anyhow!("compile: no output for claude"))
}

// ── Legs 1-4: fixture → roll_up → index → view ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spine_roll_up_index_and_view_agree_on_the_collision() {
    let (_tmp, space_root, cache_root) = write_colliding_fixture();
    let ship_dir = &space_root.join(".ship");

    // 1+2. `roll_up` composes: source wins, shadow record present.
    let composed = roll_up_members(&[space_root.clone()], Some(&cache_root), None).await.unwrap();
    assert_eq!(composed.shadows.len(), 1, "one collision recorded");
    let shadow = &composed.shadows[0];
    assert_eq!(shadow.id, COLLIDING_ID);
    assert_eq!(shadow.winning_source, ExtSource::Space);
    assert_eq!(shadow.losing_source, ExtSource::Cache);
    assert_eq!(shadow.losing_tier_hash, PKG_HEX);
    let winner = composed
        .library
        .agent_profiles
        .iter()
        .find(|a| a.agent.id == COLLIDING_ID)
        .expect("the source's colliding agent wins the roll-up");
    assert_eq!(winner.agent.name, "Source Helper");

    // 3. The `AgentDefIndex`, fed boot-style (`upsert_composed`): winner row
    //    with provenance, loser row with `shadowed_by`.
    let index = make_index(&ship_dir, "spine-idx").await;
    index.upsert_composed("project", &composed).await;

    let rows = index.query(AgentDefQuery::default()).await.unwrap();
    let active = rows
        .iter()
        .find(|r| r.id == COLLIDING_ID && r.shadowed_by.is_none())
        .expect("active row present");
    assert_eq!(active.name, "Source Helper");
    assert_eq!(
        active.ext_source.as_deref(),
        Some(serde_json::to_string(&ExtSource::Space).unwrap().as_str())
    );
    let shadow_row = rows
        .iter()
        .find(|r| r.id == COLLIDING_ID && r.shadowed_by.is_some())
        .expect("shadow row present");
    assert_eq!(shadow_row.name, "Cached Helper");
    assert_eq!(shadow_row.tier_content_hash.as_deref(), Some(PKG_HEX));

    // 4. The agents view state (`view_state` over the index) reflects the
    //    same truth — the collided id shows the winner and the shadowed row
    //    is still surfaceable through the index's audit surface.
    let list = list_agents_from_index(&index, &ship_dir).await;
    assert_eq!(list.len(), 1, "the shadow never surfaces as a selectable agent");
    assert_eq!(list[0].id, COLLIDING_ID);
    assert_eq!(list[0].name, "Source Helper");
    assert_eq!(list[0].source, "space", "the view's source label is the winning tier's kind");

    let detail = load_agent_from_index(&index, &ship_dir, COLLIDING_ID)
        .await
        .expect("detail serves the winner");
    assert_eq!(detail.name, "Source Helper");

    assert!(
        rows.iter().any(|r| r.id == COLLIDING_ID && r.shadowed_by.is_some()),
        "the shadowed row is queryable even though it never surfaces in list/detail"
    );
}

// ── Leg 5: the change leg ─────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spine_config_file_changed_invalidates_cache_and_recompose_sees_the_edit() {
    let (_tmp, space_root, cache_root) = write_colliding_fixture();
    let ship_dir = &space_root.join(".ship");

    let kernel = Arc::new(KernelLog::new(new_isolated_for_test(ship_dir.clone()).unwrap()));
    let cache = DescriptorCache::open(space_root.join("resolver-cache"), Some(kernel.clone())).unwrap();
    let sub = cache.spawn_invalidation_subscriber();
    tokio::task::yield_now().await;
    let space_id = SpaceId::new("spine-smoke");

    // Resolve once, pre-edit. The load_fn IS the production pipeline
    // (roll_up → resolve_library → compile), not a stand-in.
    let hash_before = {
        let composed = roll_up_members(&[space_root.clone()], Some(&cache_root), None).await.unwrap();
        extension_library_content_hash(&composed.library).unwrap()
    };
    let before = cache
        .get_or_load(&space_id, &hash_before, "claude", || {
            resolve_claude_output(&space_root, &cache_root)
        })
        .await
        .unwrap();
    let before_context = before.context_content.clone().unwrap_or_default();
    assert!(before_context.contains("Be terse."), "{before_context}");
    assert!(!before_context.contains("Ship it."), "{before_context}");
    assert_eq!(cache._len().await, 1);

    // Edit the source's agent file — the change leg exercises the
    // recompose visibility by mutating the agent's inline rule body.
    // The rules-format cut moved the rule body from the on-disk
    // `rules/<id>.md` convention to the agent's `rules: { inline }`
    // block; the change surface is the agent's TS file.
    write_agent_ts_with_rule(&space_root, COLLIDING_ID, "Be terse. Ship it.");

    // Author + broadcast the synthetic `config.file_changed` — the keystone
    // pattern (no inotify, not flaky), the same shape `ship-resolver`'s own
    // `cache.rs` tests inject. The cache subscriber filters on event type
    // (not path); the path is metadata. The change surface is the
    // agent's TS file (the file we just rewrote); the event's
    // `path` field carries the manifest path for audit, but the
    // cache doesn't key on it.
    kernel.broadcast(
        EventEnvelope::new(
            event_types::CONFIG_FILE_CHANGED,
            "stack.config.ts",
            &ConfigFileChanged {
                domain: "source".to_string(),
                path: "stack.config.ts".to_string(),
                content_hash: "edited".to_string(),
            },
        )
        .unwrap(),
    );

    // The descriptor cache invalidates — poll briefly for the tick to land.
    let deadline = Instant::now() + Duration::from_millis(1000);
    while Instant::now() < deadline && cache._len().await != 0 {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(cache._len().await, 0, "the descriptor cache invalidates on config.file_changed");

    // The next resolve recomposes with the edit visible.
    let hash_after = {
        let composed = roll_up_members(&[space_root.clone()], Some(&cache_root), None).await.unwrap();
        extension_library_content_hash(&composed.library).unwrap()
    };
    assert_ne!(hash_after, hash_before, "the edit changes the roll-up's content hash");
    let after = cache
        .get_or_load(&space_id, &hash_after, "claude", || {
            resolve_claude_output(&space_root, &cache_root)
        })
        .await
        .unwrap();
    let after_context = after.context_content.clone().unwrap_or_default();
    assert!(after_context.contains("Ship it."), "recompose must reflect the edit: {after_context}");

    sub.abort();
}

// ── Leg 6: determinism ────────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn spine_roll_up_compose_is_deterministic_across_two_runs() {
    let (_tmp, space_root, cache_root) = write_colliding_fixture();

    let a = roll_up_members(&[space_root.clone()], Some(&cache_root), None).await.unwrap();
    let b = roll_up_members(&[space_root.clone()], Some(&cache_root), None).await.unwrap();

    let a_bytes = extension_library_canonical(&a.library).unwrap();
    let b_bytes = extension_library_canonical(&b.library).unwrap();
    assert_eq!(a_bytes, b_bytes, "two roll-ups over the same fixture must be byte-identical");
    assert_eq!(a.shadows.len(), b.shadows.len());
}
