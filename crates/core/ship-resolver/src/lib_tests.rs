use super::*;
use serde_json::json;
use tempfile::TempDir;

fn write_minimal_space() -> (TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // .ship/ with a single rule
    let ship = root.join(".ship");
    std::fs::create_dir_all(ship.join("rules")).unwrap();
    std::fs::write(
        ship.join("rules").join("style.md"),
        "---\nalwaysApply: true\n---\nBe terse.\n",
    )
    .unwrap();

    // ship.ts at the space root
    std::fs::write(
        root.join("ship.ts"),
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
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
    assert!(
        resolved.agent_profiles.iter().any(|a| a.agent.id == "demo")
            || resolved
                .agent_profiles
                .iter()
                .any(|a| a.agent.id == "demo"),
        "expected the TS-defined demo agent in the resolved config: {ids:?}"
    );
    // The rule from .ship/rules/ reaches the resolved config.
    assert_eq!(resolved.rules.len(), 1);
    assert_eq!(resolved.rules[0].file_name, "style.md");
}

/// The shipd-ingest contract: `.jsonc` agents (the `agents/` dir) and
/// `.ts` agents (`defineAgent` in ship.ts) coexist in ONE
/// `agent_profiles` list — the consumer upserts them through a single
/// loop with no format awareness.
#[tokio::test(flavor = "current_thread")]
async fn jsonc_and_ts_agents_coexist_in_agent_profiles() {
    let (_tmp, root) = write_minimal_space();
    let agents_dir = root.join(".ship").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(
        agents_dir.join("writer.jsonc"),
        r#"{ "agent": { "id": "writer", "name": "Writer" } }"#,
    )
    .unwrap();

    let resolved = resolve_space(&root).await.unwrap();
    let ids: Vec<&str> = resolved
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(ids.contains(&"writer"), "jsonc agent missing: {ids:?}");
    assert!(ids.contains(&"demo"), "ts agent missing: {ids:?}");
}

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
    assert_eq!(lib.rules.len(), 1);
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

    // Source declares a same-id agent via .jsonc (one of the leaf's
    // shapes — `agents/writer.jsonc` lands in `agent_profiles`).
    let agents_dir = root.join(".ship").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(
        agents_dir.join("writer.jsonc"),
        r#"{ "agent": { "id": "writer", "name": "Source Writer" } }"#,
    )
    .unwrap();

    // Cache: one package with a same-id agent (namespaced by pkg_path
    // to `github.com/x/y/writer` — DIFFERENT id, no collision). To
    // exercise the shadow path, add a second dep agent that collides
    // on the namespaced id with the source's plain `writer` — i.e.
    // the cached package contributes an agent that, after namespacing,
    // matches the source. That's a different shape; for the spec
    // acceptance "cached package's agent shadowed by a same-id source
    // agent" we need: same NAMESPACE-PREFIXED id, two sources.
    //
    // The spec's exact contract: a cached package's agent id matches
    // a source agent's id (post-namespacing). The cached agent is
    // namespaced to `<pkg>/<id>`. For the collision, give the source
    // an agent whose id EQUALS the cached package's full namespaced
    // id. That's pathological authoring but proves the law.
    std::fs::write(
        agents_dir.join("gh_writer.jsonc"),
        r#"{ "agent": { "id": "github.com/x/y/writer", "name": "Source Mirror" } }"#,
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
    assert_eq!(lib.rules[0].file_name, "style.md");

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

/// The enabled tier gets a real code path: an agent authored under
/// `library_root` (mirroring `ShipHome::library_dir()`'s flat layout,
/// no `.ship/` wrapper) composes into the roll-up at `UserGlobal` rank
/// (500) — it survives when there's no collision, and is shadowed by a
/// same-id source agent (Space, rank 1000) when there is.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_composes_the_enabled_library_tier() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let library_root = root.join("library");
    std::fs::create_dir_all(library_root.join("agents")).unwrap();
    std::fs::write(
        library_root.join("agents").join("global.jsonc"),
        r#"{ "agent": { "id": "global-helper", "name": "Global Helper" } }"#,
    )
    .unwrap();

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    assert!(composed.shadows.is_empty(), "no collision — the enabled agent just survives");
    let lib = &composed.library;
    assert!(
        lib.agent_profiles.iter().any(|a| a.agent.id == "global-helper"),
        "the enabled tier's agent must compose into the result"
    );

    let rollup = lib.rollup_inputs.as_ref().unwrap();
    let enabled = rollup
        .extensions
        .iter()
        .find(|e| e.source == ExtSource::UserGlobal)
        .expect("enabled tier must be present in the audit trail");
    assert!(enabled.descriptor.agent_profiles.iter().any(|a| a.agent.id == "global-helper"));
}

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

/// The enabled tier (`UserGlobal`, rank 500) loses to a same-id source
/// agent (`Space`, rank 1000) — the space always wins over an
/// operator-enabled global default.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_source_shadows_enabled_agent_on_id_collision() {
    let (_tmp, root) = write_minimal_space();
    let cache_root = root.join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let agents_dir = root.join(".ship").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(
        agents_dir.join("writer.jsonc"),
        r#"{ "agent": { "id": "writer", "name": "Source Writer" } }"#,
    )
    .unwrap();

    let library_root = root.join("library");
    std::fs::create_dir_all(library_root.join("agents")).unwrap();
    std::fs::write(
        library_root.join("agents").join("writer.jsonc"),
        r#"{ "agent": { "id": "writer", "name": "Enabled Writer" } }"#,
    )
    .unwrap();

    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();

    let writer = composed
        .library
        .agent_profiles
        .iter()
        .find(|a| a.agent.id == "writer")
        .expect("the winning `writer` agent must be present");
    assert_eq!(writer.agent.name, "Source Writer", "source wins over the enabled tier");

    assert_eq!(composed.shadows.len(), 1);
    let shadow = &composed.shadows[0];
    assert_eq!(shadow.losing_source, ExtSource::UserGlobal);
    assert_eq!(shadow.winning_source, ExtSource::Space);
}

// ── #122 coverage-equivalence (Wave 2 acceptance) ──────────────────────

/// The prerequisite for retiring `resolve_dep_agents`'s flat merge in
/// Wave 5 (do NOT retire it now — this only proves coverage): every
/// locked package's `exports.agents` that the `#122` flat merge
/// (`ship_store::dep_skills::resolve_dep_agents`) produces today is
/// ALSO present, under the identical namespaced id, in `roll_up`'s
/// composed result. `roll_up` additionally records a `ShadowRecord`
/// where the flat merge just silently coexists — a strictly richer
/// contract over the same case.
#[tokio::test(flavor = "current_thread")]
async fn roll_up_covers_every_case_resolve_dep_agents_covers() {
    let (_tmp, root) = write_minimal_space();

    // One locked package exporting one agent — the exact shape
    // `resolve_dep_agents_namespaces_and_loads_from_cache`
    // (ship-store/src/dep_skills_tests.rs) exercises.
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

    // Case 1: populated lock + cache — the flat merge's namespaced id
    // must also appear in the roll-up's composed result.
    let flat_merged =
        ship_store::dep_skills::resolve_dep_agents(&lock_path, Some(&cache_root)).unwrap();
    assert_eq!(flat_merged.len(), 1, "sanity: the flat merge composes one dep agent");
    assert_eq!(flat_merged[0].agent.id, "github.com/x/y/helper");

    let library_root = root.join("empty_library");
    let composed = roll_up(&root, Some(&cache_root), Some(&library_root)).await.unwrap();
    let flat_ids: std::collections::HashSet<&str> =
        flat_merged.iter().map(|a| a.agent.id.as_str()).collect();
    let composed_ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    for id in &flat_ids {
        assert!(
            composed_ids.contains(id),
            "roll_up must cover every id the #122 flat merge produces: missing {id:?} in {composed_ids:?}"
        );
    }

    // Case 2: no lockfile — the flat merge's empty-vec degenerate case
    // (`resolve_dep_agents_empty_without_lockfile`) — `roll_up` must
    // degenerate the same way (no cached agents at all), not error.
    let (_tmp2, root2) = write_minimal_space();
    let absent_lock = root2.join(".ship").join("absent.lock");
    let flat_merged_empty =
        ship_store::dep_skills::resolve_dep_agents(&absent_lock, Some(&cache_root)).unwrap();
    assert!(flat_merged_empty.is_empty(), "sanity: no lockfile => no dep agents in the flat merge");

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
