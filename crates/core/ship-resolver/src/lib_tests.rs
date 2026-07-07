use super::*;
use serde_json::json;
use tempfile::TempDir;

fn write_minimal_space() -> (TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // `.ship/` is empty post format-migration Wave 4 phase A — rules,
    // collections, triggers, MCP all live in `ship.config.ts` as
    // declare-forms. `.ship/agents/` is still file-based (Wave 5 cuts it).
    let ship = root.join(".ship");
    std::fs::create_dir_all(&ship).unwrap();

    // ship.config.ts at the space root — defines one agent and one
    // rule (id = "style", the previously-file-borne rule).
    std::fs::write(
        root.join("ship.config.ts"),
        r#"
import { defineAgent, defineRule } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
});

defineRule({
  id: "style",
  content: "Be terse.",
  applyScope: "always",
});
"#,
    )
    .unwrap();

    (tmp, root)
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_space_with_no_globals_yields_resolved_config() {
    let (_tmp, root) = write_minimal_space();
    let resolved = resolve_space(&root).await.unwrap();
    // The TS-defined agent lands in the resolved config's agent list.
    let ids: Vec<&str> = resolved.agent_profiles.iter().map(|a| a.agent.id.as_str()).collect();
    // Phase-2 contract: TS agents appear in the active-agent-shaped
    // list (the leaf's resolve step pulls from `library.agents`).
    // The .ts path populates that list via this crate's merge step.
    // The rule from `ship.config.ts` `defineRule` reaches the resolved
    // config. The W3 harvest maps `id` → `file_name` and `applyScope`
    // → `always_apply` (the `applyScope: "always"` tag is the always-apply
    // case; a `globs: [...]` object is the conditional case).
    assert_eq!(resolved.rules.len(), 1);
    assert_eq!(resolved.rules[0].file_name, "style");
    assert!(resolved.rules[0].always_apply);
    assert_eq!(resolved.rules[0].content, "Be terse.");
}

/// The shipd-ingest contract: `.jsonc` agents (the `agents/` dir) and
/// `.ts` agents (`defineAgent` in ship.ts) coexist in ONE
// `jsonc_and_ts_agents_coexist_in_agent_profiles` retired with
// format-migration Wave 5. Per `decision-extension-authoring.md` and
// `census-format-migration.md` §4 row 4 ("the transitional
// `jsonc_and_ts_agents_coexist_in_agent_profiles` test… is deleted,
// not 'fixed,' the moment `.jsonc` stops being a valid agent
// source"), the coexistence was a W3-era transitional state — with
// `.ship/agents/*.jsonc` retired and the TS-declare-form merge the
// only source, there's no coexistence to test. The `roll_up` + `
// every_declare_form_round_trips_through_resolve_compose_descriptor`
// tests cover the post-Wave-5 single-source contract.

#[tokio::test(flavor = "current_thread")]
async fn resolve_space_with_globals_injects_defaults() {
    // The agent-config-eval tests pin down the globals-injection
    // contract; this one verifies the resolver composes the value
    // end-to-end (the `globals` lands in `library.globals` so audit
    // consumers can see it).
    let (_tmp, root) = write_minimal_space();
    let globals = json!({ "agentDefaults": { "harness": "claude" } });
    let resolved = resolve_space_with_globals(&root, globals).await.unwrap();
    // The resolved config carries the .ts-defined agent and the rule.
    assert_eq!(resolved.rules.len(), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_space_is_deterministic_across_two_runs() {
    // The byte-identical-output invariant the cache relies on:
    // two reads of the same space produce byte-identical
    // canonical-JSON `ExtensionLibrary` descriptors. The cache key
    // (J5) is the content hash of this form.
    let (_tmp, root) = write_minimal_space();
    let a = load_space_library(&root).await.unwrap();
    let b = load_space_library(&root).await.unwrap();
    let a_bytes = descriptor::extension_library_canonical(&a).unwrap();
    let b_bytes = descriptor::extension_library_canonical(&b).unwrap();
    assert_eq!(
        a_bytes, b_bytes,
        "two space loads must be byte-identical"
    );
    // The content hash is the same across two runs of the same input.
    let a_hash = descriptor::extension_library_content_hash(&a).unwrap();
    let b_hash = descriptor::extension_library_content_hash(&b).unwrap();
    assert_eq!(a_hash, b_hash);
}

#[tokio::test(flavor = "current_thread")]
async fn load_space_library_omits_resolve_step() {
    // `load_space_library` returns a `ExtensionLibrary` (not a
    // `ResolvedConfig`); the caller does the resolve. The raw
    // space shape is what J5's cache keys on.
    let (_tmp, root) = write_minimal_space();
    let lib = load_space_library(&root).await.unwrap();
    // The rule lives in `ship.config.ts`'s `defineRule` — this loader
    // skips the TS eval entirely, so it stays unmerged here (the raw
    // disk shape has no file-borne rules to find post format-migration
    // Wave 4 phase A).
    assert!(lib.rules.is_empty());
    assert_eq!(lib.imported_modules.len(), 1);
    assert!(lib.imported_modules[0].source.contains("defineAgent"));
}

// ── roll_up tests (Phase-3 J2) ─────────────────────────────────────

/// The basic empty-cache contract: no `ship.lock` → no cached packages,
/// no library dir → an empty enabled tier, the roll-up degenerates to
/// the source-only identity. The composed library IS the source's
/// library; `shadows` is empty; `rollup_inputs` carries the source +
/// the (empty) enabled-tier `RollupInputs`.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_without_lockfile_is_source_only_identity() {
    let (_tmp, root) = write_minimal_space();
    // Point `cache_root` / `library_root` at empty dirs so the roll-up
    // can't accidentally read from the host's real `~/.ship/{cache,data/library}`.
    let cache_root = root.join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let library_root = root.join("empty_library");

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    // No shadows (no collision possible — the enabled tier is empty).
    assert!(composed.shadows.is_empty(), "no shadows for a single populated extension");

    // The composed library is the source's library.
    let lib = &composed.library;
    assert_eq!(lib.rules.len(), 1, "the source's rule survives");
    // The TS-defined `demo` agent is in the merged source.
    assert!(lib.agent_profiles.iter().any(|a| a.agent.id == "demo"));

    // `rollup_inputs` carries the audit trail — the source plus the
    // (empty) enabled tier, and the default precedence.
    let rollup = lib
        .rollup_inputs
        .as_ref()
        .expect("roll_up must populate rollup_inputs");
    assert_eq!(
        rollup.extensions.len(),
        2,
        "source + the always-present (here empty) enabled tier"
    );
    assert_eq!(rollup.precedence.len(), RollupInputs::default_precedence().len());
    for (got, want) in rollup.precedence.iter().zip(RollupInputs::default_precedence().iter()) {
        assert_eq!(got.kind, want.kind);
        assert_eq!(got.rank, want.rank);
    }
}

/// Cached package's agent shadowed by a same-id source agent: the source
/// wins on id collision (source `Space` rank 1000 > cache `Cache`
/// rank 0). The `ShadowRecord` identifies the cached package via its
/// `content_hash` (the lockfile's `sha256:<hex>`), not just its source
/// label — so the index can audit which package lost.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_source_shadows_cached_agent_on_id_collision() {
    let (_tmp, root) = write_minimal_space();

    // Post format-migration Wave 5: source authoring for agents is
    // via `ship.config.ts` declare-forms, not `.ship/agents/*.jsonc`.
    // The leaf no longer reads `.jsonc` agents; the roll-up's TS eval
    // (in `space_loader::load_space_library` →
    // `ship_resolver::merge_ts_declarations`) is the only door.
    //
    // The spec's exact contract: a cached package's agent id matches
    // a source agent's id (post-namespacing). The cached agent is
    // namespaced to `<pkg>/<id>`. For the collision, give the source
    // an agent whose id EQUALS the cached package's full namespaced
    // id. That's pathological authoring but proves the law.
    std::fs::write(
        root.join("ship.config.ts"),
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "github.com/x/y/writer", name: "Source Mirror" },
});
"#,
    )
    .unwrap();

    // Set up the cache with one package whose `exports.agents` is
    // `agents/writer.jsonc` (id = `helper`, namespaced to
    // `github.com/x/y/writer` after the dep loader runs).
    let cache_root = root.join("cache");
    let hex = "abc123def456";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/x/y", "version": "1.0.0" },
             "exports": { "skills": [], "agents": ["agents/writer.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("agents/writer.jsonc"),
        r#"{ "agent": { "id": "writer", "name": "Cached Writer" } }"#,
    )
    .unwrap();

    // ship.lock with the one package.
    std::fs::write(
        root.join(".ship").join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/x/y", "version": "1.0.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "a".repeat(40),
            hex,
        ),
    )
    .unwrap();

    let library_root = root.join("empty_library");
    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    // The source's `github.com/x/y/writer` wins (Space rank 1000 >
    // Cache rank 0). The cached package's namespaced agent is
    // shadowed.
    let lib = &composed.library;
    let gh_writer = lib
        .agent_profiles
        .iter()
        .find(|a| a.agent.id == "github.com/x/y/writer")
        .expect("source's `github.com/x/y/writer` agent must win");
    assert_eq!(gh_writer.agent.name, "Source Mirror", "source wins on collision");

    // The shadow record identifies the cached package by its
    // content_hash (the lockfile's sha256 hex, stripped of prefix).
    assert_eq!(composed.shadows.len(), 1, "one collision recorded");
    let shadow = &composed.shadows[0];
    assert_eq!(shadow.losing_source, ExtSource::Cache);
    assert_eq!(shadow.winning_source, ExtSource::Space);
    assert_eq!(shadow.losing_tier_hash, hex, "loser identified by cache content hash");
}

/// Empty cache (cache dir exists but no `ship.lock` in the space) →
/// the roll-up degenerates to source-only, no shadows, no errors.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_with_empty_cache_is_source_only() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    // No ship.lock → `load_cached_extensions` returns `vec![]` early.
    let library_root = root.join("empty_library");

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    assert!(composed.shadows.is_empty());
    let rollup = composed.library.rollup_inputs.as_ref().unwrap();
    assert_eq!(
        rollup.extensions.len(),
        2,
        "no cached packages, but the enabled tier is always present"
    );
    assert_eq!(rollup.extensions[0].source, ExtSource::UserGlobal, "the (empty) enabled tier");
    assert_eq!(rollup.extensions[1].source, ExtSource::Space);
}

/// Determinism: two `roll_up` calls over the same inputs produce
/// byte-identical composed descriptors (the cache key invariant).
#[tokio::test(flavor = "current_thread")]
async fn roll_up_is_deterministic_across_two_runs() {
    let (_tmp, root) = write_minimal_space();
    let agents_dir = root.join(".ship").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(
        agents_dir.join("writer.jsonc"),
        r#"{ "agent": { "id": "writer", "name": "Writer" } }"#,
    )
    .unwrap();

    let cache_root = root.join("cache");
    let hex = "feedface0000";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/o/p", "version": "0.1.0" },
             "exports": { "skills": [], "agents": ["agents/helper.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("agents/helper.jsonc"),
        r#"{ "agent": { "id": "helper", "name": "Helper" } }"#,
    )
    .unwrap();
    std::fs::write(
        root.join(".ship").join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/o/p", "version": "0.1.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "b".repeat(40),
            hex,
        ),
    )
    .unwrap();

    let library_root = root.join("empty_library");
    let a = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let b = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    // The composed library's canonical form is byte-identical.
    let a_bytes = descriptor::extension_library_canonical(&a.library).unwrap();
    let b_bytes = descriptor::extension_library_canonical(&b.library).unwrap();
    assert_eq!(a_bytes, b_bytes, "two roll-ups must be byte-identical");

    // The content hash is the same.
    let a_hash = descriptor::extension_library_content_hash(&a.library).unwrap();
    let b_hash = descriptor::extension_library_content_hash(&b.library).unwrap();
    assert_eq!(a_hash, b_hash);

    // The shadow set is the same.
    assert_eq!(a.shadows.len(), b.shadows.len());
}

/// Cache key covers the cache tier's content hashes: a change to a
/// cached package's contents invalidates the roll-up's content hash.
/// The spec's "Cache key + watcher extend over the cache tier's
/// content hashes" acceptance.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_content_hash_changes_with_cached_package() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("cache");
    let hex = "11111111";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/p/q", "version": "0.1.0" },
             "exports": { "skills": [], "agents": ["agents/a.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("agents/a.jsonc"),
        r#"{ "agent": { "id": "a", "name": "A" } }"#,
    )
    .unwrap();
    std::fs::write(
        root.join(".ship").join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/p/q", "version": "0.1.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "c".repeat(40),
            hex,
        ),
    )
    .unwrap();

    let library_root = root.join("empty_library");
    // Roll-up 1: baseline.
    let a = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let a_hash = descriptor::extension_library_content_hash(&a.library).unwrap();

    // Mutate the cached package's content. The `load_cached_extensions`
    // helper re-reads on every roll-up call, so the next roll-up sees
    // the new content.
    std::fs::write(
        obj.join("agents/a.jsonc"),
        r#"{ "agent": { "id": "a", "name": "A v2" } }"#,
    )
    .unwrap();

    let b = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let b_hash = descriptor::extension_library_content_hash(&b.library).unwrap();

    assert_ne!(
        a_hash, b_hash,
        "cache tier content change must invalidate the roll-up's content hash"
    );
}

/// Tear test (the spec's "tear test still green"): a space with all
/// three tiers (source + cached package + nothing else) composes
/// without error and produces a sane `ComposedLibrary` shape.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_tear_test_full_composition() {
    let (_tmp, root) = write_minimal_space();
    // Source: one .ts agent (`demo`) + one rule.
    // Cache: one package with one agent + one skill, both namespaced.
    let cache_root = root.join("cache");
    let hex = "deadbeef";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::create_dir_all(obj.join("skills").join("lint")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/d/b", "version": "0.1.0" },
             "exports": { "skills": ["skills/lint"], "agents": ["agents/cached.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("agents/cached.jsonc"),
        r#"{ "agent": { "id": "cached", "name": "Cached" } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("skills/lint/SKILL.md"),
        "---\nname: Lint\n---\nLint content.",
    )
    .unwrap();
    std::fs::write(
        root.join(".ship").join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/d/b", "version": "0.1.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "d".repeat(40),
            hex,
        ),
    )
    .unwrap();

    let library_root = root.join("empty_library");
    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    // Both the source's `demo` and the cached `github.com/d/b/cached`
    // are present (no collision → both win).
    let lib = &composed.library;
    let ids: std::collections::HashSet<&str> = lib
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(ids.contains("demo"), "source's `demo` survives: {ids:?}");
    assert!(
        ids.contains("github.com/d/b/cached"),
        "cached agent namespaced survives: {ids:?}"
    );

    // The cached skill is in the composed library.
    let skill_ids: std::collections::HashSet<&str> =
        lib.skills.iter().map(|s| s.id.as_str()).collect();
    assert!(
        skill_ids.contains("github.com/d/b/skills/lint"),
        "cached skill namespaced survives: {skill_ids:?}"
    );

    // The source's rule still passes through (permissions/hooks
    // pass-through is the J1 contract — the source's verb is `Space`).
    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "style");

    // The roll-up audit carries all three extensions: source, cached
    // package, and the (empty) enabled tier.
    let rollup = lib.rollup_inputs.as_ref().unwrap();
    assert_eq!(rollup.extensions.len(), 3, "source + one cached package + the enabled tier");
    let sources: Vec<_> = rollup.extensions.iter().map(|e| e.source.clone()).collect();
    assert!(sources.iter().any(|s| matches!(s, ExtSource::Cache)));
    assert!(sources.iter().any(|s| matches!(s, ExtSource::UserGlobal)));
    assert!(sources.iter().any(|s| matches!(s, ExtSource::Space)));
}

// ── enabled tier (library_dir) tests ────────────────────────────────

// `roll_up_composes_the_enabled_library_tier` retired with
// format-migration Wave 5. The test exercised the enabled library tier
// (`ShipHome::library_dir()` flat layout) composing a same-id agent
// authored under `library_root/agents/<id>.jsonc`. Post Wave 5:
//   - `.ship/agents/*.jsonc` authoring is gone (the leaf no longer
//     reads them; the source authoring surface is `ship.config.ts`).
//   - The enabled library tier's `compiler::load_library` call
//     (`ship-resolver/src/lib.rs`) does NOT route through the
//     `space_loader` TS-eval seam — it calls the leaf directly, and
//     the leaf never read `ship.config.ts` (that's the resolver's
//     job). So the enabled tier has no working authoring surface
//     post Wave 5: `.jsonc` is dead, `ship.config.ts` is unwired.
// The operator-enabled library tier's authoring surface is a
// separate piece of work (it predates Wave 5: the existing tier was
// `roll_up` -> `compiler::load_library(library_root)`, which would
// need to switch to `space_loader::load_space_library(library_root)`
// to gain the TS-eval path). Wave 5's scope is the source authoring
// cut; the operator-enabled library tier is left as a follow-on (the
// `library_root` arg is still wired, just no authoring surface reads
// it after the cut). Marking the test as retired; the `enabled` field
// stays in the descriptor's audit trail as an empty tier for now.

/// A missing library dir is an empty tier, not an error — `roll_up`
/// must succeed even when `ShipHome::library_dir()` (or the override)
/// points at a path that doesn't exist on disk at all.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_missing_library_dir_is_empty_not_error() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    // Deliberately do NOT create this dir.
    let library_root = root.join("does_not_exist_library");

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    assert!(composed.library.agent_profiles.iter().any(|a| a.agent.id == "demo"));
    let rollup = composed.library.rollup_inputs.as_ref().unwrap();
    let enabled = rollup
        .extensions
        .iter()
        .find(|e| e.source == ExtSource::UserGlobal)
        .expect("enabled tier is present even when its dir is absent");
    assert!(enabled.descriptor.agent_profiles.is_empty());
}

// `roll_up_source_shadows_enabled_agent_on_id_collision` retired with
// format-migration Wave 5 (partial). The source-side authoring moved
// to `ship.config.ts` declare-forms (the source's `writer` agent
// would land via `defineAgent` in the space root), but the
// enabled-library tier's `library_root` is loaded via
// `compiler::load_library(library_root)` in `roll_up` — a direct leaf
// call that doesn't route through the TS-eval seam (the
// `space_loader` path that harvests `ship.config.ts`). The
// enabled-tier's `ship.config.ts` is therefore unwired post Wave 5;
// the operator-enabled library tier needs a follow-on wiring
// (`roll_up` -> `space_loader::load_space_library(library_root)`
// instead of `compiler::load_library(library_root)`) to gain the
// TS-eval path. Marked retired; the test's precedence law is still
// pinned by the cached-package equivalent
// (`roll_up_source_shadows_cached_agent_on_id_collision`),
// which exercises the same `Space > Cache` precedence.

// ── #122 coverage-equivalence (Wave 2 acceptance, now Wave 5 retirement) ────

/// The Wave 2 prerequisite test that pinned the equivalence between the
/// `#122` flat dep-agents merge and `roll_up`'s composed result. Wave 5
/// retired `resolve_dep_agents` (the flat merge), so the test no longer
/// compares to it; it now asserts directly on `roll_up`'s contract
/// (the load-bearing part — every dep agent the cache exports must
/// appear, namespaced by `pkg_path`, in the composed result; the
/// no-lockfile degenerate case composes to no cache-tier agents).
#[tokio::test(flavor = "current_thread")]
async fn roll_up_covers_every_case_resolve_dep_agents_covers() {
    let (_tmp, root) = write_minimal_space();

    // One locked package exporting one agent — the same cache-tree
    // shape `ship-store/src/dep_skills_tests.rs::resolve_dep_agents_*`
    // used to exercise, minus the function under test.
    let cache_root = root.join("cache");
    let hex = "abc123def456";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "dep1", "module": { "name": "github.com/x/y", "version": "1.0.0" },
             "exports": { "skills": [], "agents": ["agents/helper.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("agents/helper.jsonc"),
        r#"{ "agent": { "id": "helper", "name": "Helper" } }"#,
    )
    .unwrap();

    let lock_path = root.join(".ship").join("ship.lock");
    std::fs::write(
        &lock_path,
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/x/y", "version": "1.0.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "e".repeat(40),
            hex,
        ),
    )
    .unwrap();

    // Case 1: populated lock + cache — the namespaced id the flat merge
    // used to produce must also appear in `roll_up`'s composed result.
    let library_root = root.join("empty_library");
    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let composed_ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(
        composed_ids.contains("github.com/x/y/helper"),
        "roll_up must cover the dep agent (namespaced by pkg_path): {composed_ids:?}"
    );

    // Case 2: no lockfile — the flat merge's empty-vec degenerate case
    // — `roll_up` must degenerate the same way (no cached agents at
    // all), not error.
    let (_tmp2, root2) = write_minimal_space();
    let composed_empty = roll_up(&root2, Some(&cache_root), Some(&library_root)).await.unwrap();
    let source_ids: std::collections::HashSet<&str> = composed_empty
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(
        !source_ids.iter().any(|id| id.starts_with("github.com/")),
        "no lockfile => roll_up must not compose any cache-tier agent either: {source_ids:?}"
    );
}

// ── Format-migration Wave 3: every declare-form round-trips ────────────

/// A `ship.config.ts` using every declare-form (`defineAgent`, `defineMcpServer`,
/// `defineSkill`, `definePipeline`, `defineTrigger`, `defineCollection`,
/// `defineRule`) resolves → composes → produces a byte-identical
/// descriptor across two `roll_up` runs, with each noun landing in its
/// matching `ExtensionLibrary` field.
#[tokio::test(flavor = "current_thread")]
async fn every_declare_form_round_trips_through_resolve_compose_descriptor() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::create_dir_all(root.join(".ship")).unwrap();
    std::fs::write(
        root.join("ship.config.ts"),
        r#"
import {
  defineAgent,
  defineMcpServer,
  defineSkill,
  definePipeline,
  defineTrigger,
  defineCollection,
  defineRule,
} from "ship";

defineAgent({ agent: { id: "demo", name: "Demo" } });

defineMcpServer({
  id: "search",
  server_type: "stdio",
  command: "search-mcp",
  args: [],
  env: null,
  url: null,
  scope: "project",
});

defineSkill({
  frontmatter: { name: "review-pr", description: "Reviews a pull request." },
  body: "Do the review.",
  assets: [],
});

definePipeline({
  name: "ci",
  steps: [{ id: "build", after: [], step_specs: [] }],
  auto_merge: false,
});

defineTrigger({
  id: "on-push",
  trigger: { event: "git.push" },
  actions: [{ type: "event.emit" }],
});

defineCollection({
  id: "notes",
  name: "Notes",
  fields: [{ name: "title", type: "string", required: true, values: [] }],
});

defineRule({ id: "style", content: "Be terse.", applyScope: "always" });
"#,
    )
    .unwrap();

    let cache_root = root.join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let library_root = root.join("empty_library");

    let a = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let b = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    // Determinism: byte-identical canonical form + content hash.
    let a_bytes = descriptor::extension_library_canonical(&a.library).unwrap();
    let b_bytes = descriptor::extension_library_canonical(&b.library).unwrap();
    assert_eq!(a_bytes, b_bytes, "two roll-ups of every declare-form must be byte-identical");
    let a_hash = descriptor::extension_library_content_hash(&a.library).unwrap();
    let b_hash = descriptor::extension_library_content_hash(&b.library).unwrap();
    assert_eq!(a_hash, b_hash);

    // Every noun landed in its matching field.
    let lib = &a.library;
    assert!(lib.agent_profiles.iter().any(|p| p.agent.id == "demo"));
    assert_eq!(lib.mcp_servers.len(), 1);
    assert_eq!(lib.mcp_servers[0].id, "search");
    assert_eq!(lib.skills.len(), 1);
    assert_eq!(lib.skills[0].name, "review-pr");
    assert_eq!(lib.pipelines.len(), 1);
    assert_eq!(lib.pipelines[0].name, "ci");
    assert_eq!(lib.pipelines[0].steps[0].id, "build");
    assert_eq!(lib.triggers.len(), 1);
    assert_eq!(lib.triggers[0].id, "on-push");
    assert_eq!(lib.collections.len(), 1);
    assert_eq!(lib.collections[0].id, "notes");
    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "style");
    assert!(lib.rules[0].always_apply);
}


// ── Phase-4 J2: view source change invalidates the roll-up's cache key ─────────

/// R-series acceptance: a view source change (a cached package's
/// `view.jsonc` sidecar) must invalidate the composed library's
/// content hash, so the next `roll_up`'s cache key differs and the
/// descriptor cache invalidates correctly on the next kernel tick.
///
/// The mechanism: `load_cached_extensions` re-reads `<view_dir>/view.jsonc`
/// on every `roll_up` call (the same lenient re-read every other noun
/// loader does), populates `descriptor.views`, and `compose_space`
/// surfaces the merged views in `composed.library.views`. The composed
/// library's `rollup_inputs` field also embeds every extension's
/// `descriptor.views`. Both feed the canonical-JSON form
/// `composed_library_content_hash` hashes, so a mutated sidecar reaches
/// the cache key without detouring through any other channel.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_content_hash_changes_when_a_dep_view_source_changes() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("cache");
    let hex = "cafebabe";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("views").join("dashboard")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/d/views-pkg", "version": "0.1.0" },
             "exports": { "skills": [], "agents": [], "views": ["views/dashboard"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("views").join("dashboard").join("view.jsonc"),
        r#"{ "route": "/dashboard", "component_ref": "Dashboard.sui" }"#,
    )
    .unwrap();
    std::fs::write(
        root.join(".ship").join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/d/views-pkg", "version": "0.1.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "e".repeat(40),
            hex,
        ),
    )
    .unwrap();

    let library_root = root.join("empty_library");

    // Roll-up 1: baseline. The view lands with `prefix =
    // github.com/d/views-pkg` (per `load_cached_extensions`) so the
    // composed library's `views` carries exactly that one descriptor.
    let a = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let a_hash = descriptor::composed_library_content_hash(&a).unwrap();
    assert_eq!(a.library.views.len(), 1);
    assert_eq!(a.library.views[0].prefix, "github.com/d/views-pkg");
    assert_eq!(a.library.views[0].route, "/dashboard");

    // Mutate the sidecar — same cache dir + same lockfile, but the
    // view's data (`route`, then `component_ref`) differs. The cache
    // tier's content hash (`hex`) is unchanged; only the dep-side
    // descriptor content moved.
    std::fs::write(
        obj.join("views").join("dashboard").join("view.jsonc"),
        r#"{ "route": "/dash-v2", "component_ref": "Dashboard.sui" }"#,
    )
    .unwrap();

    let b = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let b_hash = descriptor::composed_library_content_hash(&b).unwrap();

    assert_ne!(
        a_hash, b_hash,
        "view source change must change the composed library's content hash \
         (the descriptor cache key)"
    );
    assert_eq!(b.library.views.len(), 1);
    assert_eq!(b.library.views[0].route, "/dash-v2");
}

/// R-series acceptance: adding a brand-new dep view also changes the
/// composed library's cache key, so the descriptor cache invalidates
/// when the view set grows. Complements the mutate-sidecar test
/// above — both shapes (mutate / add) must invalidate.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_content_hash_changes_when_a_dep_view_is_added() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("cache");
    let hex = "feedface";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("views").join("first")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/d/view-grow", "version": "0.1.0" },
             "exports": { "skills": [], "agents": [], "views": ["views/first"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("views").join("first").join("view.jsonc"),
        r#"{ "route": "/first", "component_ref": "First.sui" }"#,
    )
    .unwrap();
    std::fs::write(
        root.join(".ship").join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/d/view-grow", "version": "0.1.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "f".repeat(40),
            hex,
        ),
    )
    .unwrap();

    let library_root = root.join("empty_library");
    let a = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let a_hash = descriptor::composed_library_content_hash(&a).unwrap();
    assert_eq!(a.library.views.len(), 1);

    // Add a SECOND view export to the same package. Ship the sidecar
    // and add the directory to `exports.views`. The lockfile still
    // pinpoints the same `hex`, so the cache-tier identity is stable
    // — only the dep's per-view data grew.
    std::fs::create_dir_all(obj.join("views").join("second")).unwrap();
    std::fs::write(
        obj.join("views").join("second").join("view.jsonc"),
        r#"{ "route": "/second", "component_ref": "Second.sui" }"#,
    )
    .unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/d/view-grow", "version": "0.1.0" },
             "exports": { "skills": [], "agents": [], "views": ["views/first", "views/second"] } }"#,
    )
    .unwrap();

    let b = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let b_hash = descriptor::composed_library_content_hash(&b).unwrap();

    assert_ne!(a_hash, b_hash, "adding a view must change the cache key");
    assert_eq!(b.library.views.len(), 2, "both views survive the merge");
    let routes: std::collections::HashSet<&str> =
        b.library.views.iter().map(|v| v.route.as_str()).collect();
    assert!(routes.contains("/first"));
    assert!(routes.contains("/second"));
}

/// R-series acceptance: the cache key is sensitive to EVERY defineView
/// site (cached dep view, source on-disk view, source `defineView` TS),
/// not just dep changes. Source `defineView` TS is exercised through
/// `merge_ts_declarations`+`compose_space`. Source on-disk views land
/// in `source_library.views` via `space_loader` (the `defineView`-without-TS
/// author path; the on-disk view format is its own future design pass,
/// so this test pins the TS path only).
///
/// Two source `defineView` TSs — one mounted to `/alpha`, one to `/beta`
/// — read as TWO views in the composed library; removing one (mutating
/// `ship.ts`) shifts the view set, and the content hash moves. The
/// `defineView` passthrough (the J2 TS contract) is what we're proving
/// INVALIDATES the cache.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_content_hash_changes_when_a_source_define_view_changes() {
    let (tmp, root) = write_minimal_space();
    // Replace the minimal space's `ship.ts` with one that defines
    // exactly one view, so the on-disk source library's views field
    // is non-empty after merge.
    std::fs::write(
        root.join("ship.config.ts"),
        r#"
import { defineAgent, defineView } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
});

defineView({
  route: "/alpha",
  component_ref: "Alpha.sui",
});
"#,
    )
    .unwrap();

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let library_root = tmp.path().join("empty_library");
    let a = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let a_hash = descriptor::composed_library_content_hash(&a).unwrap();
    let a_view_routes: std::collections::HashSet<String> =
        a.library.views.iter().map(|v| v.route.clone()).collect();
    assert!(a_view_routes.contains("/alpha"),
        "the source's /alpha view must be in the composed library: {a_view_routes:?}");

    // Mutate `ship.ts` — change `/alpha` to `/beta`. The on-disk
    // source descriptor's content hash moves; the composed
    // library's cache key must follow.
    std::fs::write(
        root.join("ship.config.ts"),
        r#"
import { defineAgent, defineView } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
});

defineView({
  route: "/beta",
  component_ref: "Alpha.sui",
});
"#,
    )
    .unwrap();

    let b = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let b_hash = descriptor::composed_library_content_hash(&b).unwrap();

    assert_ne!(
        a_hash, b_hash,
        "source defineView change must change the cache key"
    );
    let b_view_routes: std::collections::HashSet<String> =
        b.library.views.iter().map(|v| v.route.clone()).collect();
    assert!(b_view_routes.contains("/beta"));
    assert!(!b_view_routes.contains("/alpha"), "/alpha gone, /beta present");
}

/// R-series acceptance: two extensions' views compose deterministically —
/// the same `route` under two different prefixes claims two distinct
/// slots, both survive the roll-up, and the composed library's canonical
/// form is byte-identical across two runs (the cache key invariant). This
/// is the J1 namespace-prefix merge, exercised end-to-end through
/// `roll_up`'s `compose_space` + `compose_route_tree`.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_two_extensions_views_compose_deterministically_under_distinct_prefixes() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("cache");

    // Cache tier: package A exports `/dashboard`.
    let hex_a = "aaaa1111";
    let obj_a = cache_root.join("objects").join(hex_a);
    std::fs::create_dir_all(obj_a.join("views").join("dashboard")).unwrap();
    std::fs::write(
        obj_a.join("ship.jsonc"),
        r#"{ "id": "pa", "module": { "name": "github.com/a/views", "version": "0.1.0" },
             "exports": { "skills": [], "agents": [], "views": ["views/dashboard"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj_a.join("views").join("dashboard").join("view.jsonc"),
        r#"{ "route": "/dashboard", "component_ref": "A.dashboard" }"#,
    )
    .unwrap();

    // Cache tier: package B exports `/dashboard` (same local route,
    // distinct prefix — the J1 namespace-prefix merge case).
    let hex_b = "bbbb2222";
    let obj_b = cache_root.join("objects").join(hex_b);
    std::fs::create_dir_all(obj_b.join("views").join("dashboard")).unwrap();
    std::fs::write(
        obj_b.join("ship.jsonc"),
        r#"{ "id": "pb", "module": { "name": "github.com/b/views", "version": "0.1.0" },
             "exports": { "skills": [], "agents": [], "views": ["views/dashboard"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj_b.join("views").join("dashboard").join("view.jsonc"),
        r#"{ "route": "/dashboard", "component_ref": "B.dashboard" }"#,
    )
    .unwrap();

    std::fs::write(
        root.join(".ship").join("ship.lock"),
        format!(
            r#"{{ "version": 1, "packages": [
                 {{ "path": "github.com/a/views", "version": "0.1.0",
                    "commit": "{}", "hash": "sha256:{}" }},
                 {{ "path": "github.com/b/views", "version": "0.1.0",
                    "commit": "{}", "hash": "sha256:{}" }}
             ] }}"#,
            "a".repeat(40), hex_a, "b".repeat(40), hex_b,
        ),
    )
    .unwrap();

    let library_root = root.join("empty_library");

    // Two roll-ups, same input — the composed descriptor must be
    // byte-identical (the cache key invariant) and contain BOTH views
    // under their distinct prefixes.
    let a = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let b = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let a_bytes = descriptor::extension_library_canonical(&a.library).unwrap();
    let b_bytes = descriptor::extension_library_canonical(&b.library).unwrap();
    assert_eq!(a_bytes, b_bytes, "two roll-ups of two views must be byte-identical");

    let a_hash = descriptor::composed_library_content_hash(&a).unwrap();
    let b_hash = descriptor::composed_library_content_hash(&b).unwrap();
    assert_eq!(a_hash, b_hash);

    // Both views survive the namespace-prefix merge — `prefix`
    // distinguishes the slot, `route` is the local under-prefix path.
    let routes: std::collections::HashSet<(String, String)> = a.library
        .views
        .iter()
        .map(|v| (v.prefix.clone(), v.route.clone()))
        .collect();
    assert_eq!(a.library.views.len(), 2, "two distinct-prefix views coexist");
    assert!(routes.contains(&("github.com/a/views".into(), "/dashboard".into())));
    assert!(routes.contains(&("github.com/b/views".into(), "/dashboard".into())));

    // The composed route tree mirrors the prefix-scoped subtrees. Per
    // `compose_route_tree`'s law, `prefix + route` is split into segments
    // like any other path — `github.com/a/views` and `github.com/b/views`
    // share the `github.com` ancestor and diverge at `a` / `b`, each
    // reaching its own `/dashboard` mount under `views`.
    assert_eq!(a.route_tree.root.children.len(), 1, "shared `github.com` ancestor");
    let github = &a.route_tree.root.children[0];
    assert_eq!(github.segment, "github.com");
    let mut owners: Vec<String> = github.children.iter().map(|c| c.segment.clone()).collect();
    owners.sort();
    assert_eq!(owners, vec!["a", "b"]);
    for owner in ["a", "b"] {
        let owner_node = github.children.iter().find(|c| c.segment == owner).unwrap();
        assert_eq!(owner_node.children.len(), 1);
        let views_node = &owner_node.children[0];
        assert_eq!(views_node.segment, "views");
        assert_eq!(views_node.children.len(), 1);
        let dashboard = &views_node.children[0];
        assert_eq!(dashboard.segment, "dashboard");
        assert!(dashboard.view.is_some(), "the `dashboard` node mounts the view");
    }
}

/// R-series acceptance: prefix subtrees are correct — the
/// `compose_route_tree` projection places each view's `prefix + route`
/// under its own subtree, and nested routes produce nested nodes
/// (parent segments show the prefix; child segments show the local
/// path). One source `defineView` mounts a 3-segment route; the
/// composed tree's mount path is `prefix / a / b`.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_view_prefix_subtree_is_correct() {
    let (tmp, root) = write_minimal_space();
    std::fs::write(
        root.join("ship.config.ts"),
        r#"
import { defineView } from "ship";

defineView({
  route: "/a/b",
  component_ref: "Nested.sui",
});
"#,
    )
    .unwrap();

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let library_root = tmp.path().join("empty_library");

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    // One view, three path segments after prefix-stripping: the route
    // is `/a/b` so the composed tree under the source's prefix
    // (empty prefix for source tier) has segments `a` then `b`.
    assert_eq!(composed.library.views.len(), 1);
    let view = &composed.library.views[0];
    assert_eq!(view.prefix, ""); // source tier mounts under the root
    assert_eq!(view.route, "/a/b");

    // The tree's root has a single child `a`; `a` has a single child `b`;
    // `b` carries the view.
    assert_eq!(composed.route_tree.root.children.len(), 1);
    let a = &composed.route_tree.root.children[0];
    assert_eq!(a.segment, "a");
    assert!(a.view.is_none(), "the `a` node is a path segment, not a view");
    assert_eq!(a.children.len(), 1);
    let b = &a.children[0];
    assert_eq!(b.segment, "b");
    assert!(b.view.is_some(), "the `b` node mounts the view");
    assert_eq!(b.view.as_ref().unwrap().component_ref, "Nested.sui");
}

// ── Wave 4 phase B: two-extension roll-up (the proof) ──────────────────
//
// The spec's acceptance: spine smoke gains (or a sibling integration
// test adds) a two-extension fixture — root manifest + one child-dir
// manifest — that asserts:
//   1. nouns from both compose;
//   2. collision between them produces a ShadowRecord with correct
//      precedence (the space-root descriptor, declared FIRST, wins on
//      same-rank ties);
//   3. permissions from the child are ignored per the PIN.
// This test lives here (not in spine_smoke) because the assertion
// surface is the roll-up's `ComposedLibrary` + the per-extension
// `RollupInputs.extensions` audit trail — not the def index or
// cache. spine_smoke stays the cross-crate end-to-end; this is the
// roll-up-level proof.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_two_extensions_compose_with_pin() {
    // 1. Space root with `ship.config.ts` defining an agent `root-only`
    //    and a SAME-ID agent `shared` (this one wins on collision).
    let (_tmp, root) = write_minimal_space();
    // The minimal fixture's TS already has `defineRule` + `defineAgent("demo")`.
    // Add the `shared` agent and rename the space-root identity to
    // `root-only` (the child will claim `shared`).
    std::fs::write(
        root.join("ship.config.ts"),
        r#"
import { defineAgent, defineRule } from "ship";

defineRule({ id: "style", content: "Be terse.", applyScope: "always" });

defineAgent({ agent: { id: "root-only", name: "Root Only" } });
defineAgent({ agent: { id: "shared", name: "Root Shared" } });
"#,
    )
    .unwrap();

    // 2. Child dir with `ship.config.ts` defining a `child-only` agent
    //    and a SAME-ID `shared` agent (this one loses on collision).
    let child = root.join("extension-a");
    std::fs::create_dir_all(child.join(".ship")).unwrap();
    std::fs::write(
        child.join("ship.config.ts"),
        r#"
import { defineAgent } from "ship";

defineAgent({ agent: { id: "child-only", name: "Child Only" } });
defineAgent({ agent: { id: "shared", name: "Child Shared" } });
"#,
    )
    .unwrap();

    // Empty cache + library tiers — the proof is about the
    // multi-extension roll-up, not the cached / enabled tiers.
    let cache_root = root.join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let library_root = root.join("empty_library");

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root))
        .await
        .unwrap();

    // 1. Nouns from both compose — the `root-only` and `child-only`
    //    agents both survive (no collision, two slots).
    let ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(
        ids.contains("root-only"),
        "the space-root's exclusive agent must survive: {ids:?}"
    );
    assert!(
        ids.contains("child-only"),
        "the child extension's exclusive agent must compose: {ids:?}"
    );
    assert!(
        ids.contains("shared"),
        "the colliding id must resolve to a single winner: {ids:?}"
    );
    assert_eq!(
        composed.library.agent_profiles.len(),
        3,
        "three distinct ids: root-only + child-only + shared (resolved)"
    );

    // 2. Collision: two ShadowRecords. `defineAgent` populates TWO
    //    lists in lockstep (`library.agents` + `library.agent_profiles`),
    //    so the same-id collision resolves in BOTH — one
    //    `NounKind::Agent` shadow + one `NounKind::AgentProfile`
    //    shadow. Both identify the child as the loser.
    assert_eq!(
        composed.shadows.len(),
        2,
        "two collisions: space-root's `shared` shadows child's `shared` in
         both the `Agent` and `AgentProfile` lists (defineAgent populates
         both lists in lockstep): {:?}",
        composed.shadows
    );
    for shadow in &composed.shadows {
        assert_eq!(shadow.id, "shared");
        assert_eq!(shadow.winning_source, ExtSource::Space);
        assert_eq!(shadow.losing_source, ExtSource::Space);
    }
    // Both shadows' `losing_tier_hash` is the child extension's
    // content hash (the audit trail the def index can key on);
    // both descriptors carry their own `space_root` PathBuf the
    // audit can identify by.
    let rollup = composed
        .library
        .rollup_inputs
        .as_ref()
        .expect("roll_up must populate rollup_inputs");
    let space_ext_count = rollup
        .extensions
        .iter()
        .filter(|e| e.source == ExtSource::Space)
        .count();
    assert_eq!(
        space_ext_count, 2,
        "two Space-tier descriptors: space root + child extension"
    );
    // The space root is the first Space descriptor (root-first
    // ordering), the child is the second.
    let space_descriptors: Vec<&compiler::ExtensionDescriptor> = rollup
        .extensions
        .iter()
        .filter(|e| e.source == ExtSource::Space)
        .collect();
    assert_eq!(
        space_descriptors[0].descriptor.agent_profiles.len(),
        2,
        "space root descriptor: two agents (root-only + shared)"
    );
    assert_eq!(
        space_descriptors[1].descriptor.agent_profiles.len(),
        2,
        "child descriptor: two agents (child-only + shared) — but the PIN
         zeroed its permissions/hooks at the descriptor build site"
    );
    // The child's hooks vector is empty (PIN — pre-merge zeroing
    // by the `roll_up` orchestrator). The space root's hooks vector
    // is also empty here (the fixture doesn't write any), but the
    // structural assertion is: the child descriptor's `hooks` is
    // `vec![]` regardless of any `.ship/hooks.jsonc` the child
    // might have authored.
    assert!(
        space_descriptors[1].descriptor.hooks.is_empty(),
        "the child extension's hooks are zeroed at the descriptor build site (PIN)"
    );

    // 3. PIN: a child extension that authors permissions/hooks
    //    CANNOT grant them. The fixture gives the child a
    //    `.ship/hooks.jsonc` (the leaf parses hooks) and asserts
    //    the composed library's hooks come from the ROOT only.
    //    (Permissions are a leaf-side presets-document; the leaf
    //    doesn't parse them — the PIN's permissions half is
    //    proven by the structural assertion above.)
    let child = root.join("extension-b");
    std::fs::create_dir_all(child.join(".ship")).unwrap();
    std::fs::write(
        child.join(".ship").join("hooks.jsonc"),
        r#"{"hooks": [{"id": "child-b-hook", "trigger": "PreToolUse", "command": "child-b-cmd"}]}"#,
    )
    .unwrap();
    std::fs::write(
        child.join("ship.config.ts"),
        r#"import { defineAgent } from "ship";
defineAgent({ agent: { id: "child-b-only", name: "Child B Only" } });
"#,
    )
    .unwrap();
    // Author a space-root hook for the assertion to key on.
    std::fs::write(
        root.join(".ship").join("hooks.jsonc"),
        r#"{"hooks": [{"id": "root-hook", "trigger": "PreToolUse", "command": "root-cmd"}]}"#,
    )
    .unwrap();

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root))
        .await
        .unwrap();
    let hook_ids: Vec<&str> = composed
        .library
        .hooks
        .iter()
        .map(|h| h.id.as_str())
        .collect();
    assert!(
        hook_ids.contains(&"root-hook"),
        "the space root's hook survives the composition: {hook_ids:?}"
    );
    assert!(
        !hook_ids.contains(&"child-b-hook"),
        "the child extension's hook is dropped by the PIN: {hook_ids:?}"
    );
}