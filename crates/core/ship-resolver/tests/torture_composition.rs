//! T-RESOLVER-TORTURE — adversarial coverage for `roll_up_members` +
//! `stack_model::compose_space`. TEST-AUTHORING ONLY: this file proves or
//! breaks the resolver against the master properties (determinism,
//! collision correctness) and the five scenario classes in the job spec
//! (`dispatch-t-resolver-torture.md`). A genuine break is marked
//! `#[ignore]` with a precise repro and reported as a finding — the
//! resolver itself is never patched here.
//!
//! Scope: `ship-resolver`'s own fixtures (temp dirs), its own public API
//! (`roll_up_members`, `descriptor::*`). No other crate is touched.

use std::path::Path;
use std::time::Instant;

use ship_resolver::descriptor;
use stack_model::ExtSource;
use tempfile::TempDir;

// ─── Fixture helpers ────────────────────────────────────────────────────

/// A bare stack root: `stack.config.ts` with an empty `defineConfig({})`
/// and no other content. Callers add agents/skills/children on top.
fn write_bare_root(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(
        root.join("stack.config.ts"),
        "import { defineConfig } from \"ship\";\nexport default defineConfig({});\n",
    )
    .unwrap();
}

fn write_agent(root: &Path, file_stem: &str, id: &str, name: &str) {
    std::fs::create_dir_all(root.join("agents")).unwrap();
    std::fs::write(
        root.join("agents").join(format!("{file_stem}.ts")),
        format!(
            r#"import {{ defineAgent }} from "ship";
defineAgent({{ agent: {{ id: "{id}", name: "{name}" }} }});
"#
        ),
    )
    .unwrap();
}

fn write_skill(root: &Path, id: &str, body: &str) {
    let dir = root.join("skills").join(id);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("SKILL.md"),
        format!("---\nname: {id}\ndescription: {body}\n---\n{body}\n"),
    )
    .unwrap();
}

/// One locked+cached package exporting a single agent whose namespaced id
/// (after `load_cached_extensions` prefixes it with `pkg.path`) is exactly
/// `collide_id`. Appends the `packages` entry to `lock_entries` and writes
/// the cache object under `cache_root`; the caller still has to write the
/// resulting `ship.lock` (multiple packages accumulate into one lock).
fn cached_agent_object(
    cache_root: &Path,
    hex: &str,
    pkg_path: &str,
    local_agent_id: &str,
    name: &str,
) {
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        format!(
            r#"{{ "id": "p", "module": {{ "name": "{pkg_path}", "version": "1.0.0" }},
                 "exports": {{ "skills": [], "agents": ["agents/a.jsonc"] }} }}"#
        ),
    )
    .unwrap();
    std::fs::write(
        obj.join("agents/a.jsonc"),
        format!(r#"{{ "agent": {{ "id": "{local_agent_id}", "name": "{name}" }} }}"#),
    )
    .unwrap();
}

fn write_lock(root: &Path, packages: &[(&str, &str, &str)]) {
    // packages: (pkg_path, version, hex)
    let entries: Vec<String> = packages
        .iter()
        .map(|(path, version, hex)| {
            format!(
                r#"{{ "path": "{path}", "version": "{version}", "commit": "{}", "hash": "sha256:{hex}" }}"#,
                "a".repeat(40)
            )
        })
        .collect();
    std::fs::write(
        root.join("ship.lock"),
        format!(r#"{{ "version": 1, "packages": [{}] }}"#, entries.join(",")),
    )
    .unwrap();
}

// ─── Scenario 1: adversarial collision precedence ──────────────────────

/// The three-tier adversarial case: the SAME agent id is authored at the
/// space root, in a cached package (namespaced to that exact id), AND in
/// a child extension nested under the root. Per `compose_space`'s law
/// (`stack-model/src/compose.rs`): rank Space(1000) > Cache(0), so the
/// space tier always beats the cache tier regardless of which Space
/// descriptor (root or child) ultimately owns the slot. This is the
/// property the dispatch's "space-authored wins over cached packages"
/// master property demands; it must hold for every noun family, not just
/// agents.
#[tokio::test(flavor = "current_thread")]
async fn three_way_collision_space_always_beats_cache() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    write_agent(&root, "root-agent", "collide", "Root Owns It");
    write_skill(&root, "collide-skill", "root skill body");

    // Child extension also claims both ids.
    let child = root.join("child-ext");
    write_bare_root(&child);
    write_agent(&child, "child-agent", "collide", "Child Owns It");
    write_skill(&child, "collide-skill", "child skill body");

    // Cached package claims the SAME final id via namespacing: the cache
    // loader prefixes with `pkg.path`, so pick a `pkg_path` of exactly
    // "collide" and a local agent id of "" is invalid — instead alias the
    // package path itself to match. We namespace `pkg_path/local_id`, so
    // to land on bare `collide` we'd need an empty pkg_path, which the
    // loader doesn't support. Cache ids are therefore always namespaced
    // and can never literally collide with an unnamespaced space id in
    // practice — so the adversarial construction targets the NAMESPACED
    // form the source can also author directly (matching the existing
    // `roll_up_source_shadows_cached_agent_on_id_collision` precedent).
    let cache_root = tmp.path().join("cache");
    let hex = "3wayhex000000000000000000000000000000000000000000000000000000";
    cached_agent_object(&cache_root, hex, "pkg.example", "writer", "Cached Writer");
    write_lock(&root, &[("pkg.example", "1.0.0", hex)]);

    // The source (root) ALSO authors the exact namespaced id the cache
    // would produce (`pkg.example/writer`), and so does the child.
    write_agent(&root, "ns-agent", "pkg.example/writer", "Root Namespaced");
    write_agent(&child, "ns-agent", "pkg.example/writer", "Child Namespaced");

    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();

    let winner = composed
        .library
        .agent_profiles
        .iter()
        .find(|a| a.agent.id == "pkg.example/writer")
        .expect("the collided id must resolve to exactly one winner");
    assert_ne!(
        winner.agent.name, "Cached Writer",
        "cache (rank 0) must never win over a Space-tier (rank 1000) author, \
         regardless of root vs. child: got {:?}",
        winner.agent.name
    );

    // Every shadow touching this id must show Cache losing to Space (or
    // Space losing to Space, for the root/child sub-collision) — Cache
    // must NEVER appear as `winning_source`.
    for shadow in &composed.shadows {
        if shadow.id == "pkg.example/writer" {
            assert_ne!(
                shadow.winning_source,
                ExtSource::Cache,
                "cache tier must never win an id collision: {shadow:?}"
            );
        }
    }

    // Exactly one composed entry per noun for the plain "collide" id too
    // (root vs. child collision, no cache involved) — never both, never
    // zero.
    let collide_agents: Vec<_> = composed
        .library
        .agent_profiles
        .iter()
        .filter(|a| a.agent.id == "collide")
        .collect();
    assert_eq!(
        collide_agents.len(),
        1,
        "root/child collision on `collide` must resolve to exactly one agent: {collide_agents:?}"
    );
    let collide_skills: Vec<_> = composed
        .library
        .skills
        .iter()
        .filter(|s| s.id == "collide-skill")
        .collect();
    assert_eq!(
        collide_skills.len(),
        1,
        "root/child collision on `collide-skill` must resolve to exactly one skill: {collide_skills:?}"
    );
}

/// Determinism under filesystem iteration order: two child extensions
/// with names that sort differently than they're created, both claiming
/// the same id. `discover_child_extension_roots` sorts child dir names
/// lexicographically before composing (space_loader.rs), so the winner
/// must be a pure function of dir NAME order, never creation order or
/// raw `read_dir` order. Run the same composition twice against fixtures
/// built with reversed creation order and assert byte-identical output.
#[tokio::test(flavor = "current_thread")]
async fn collision_winner_is_independent_of_directory_creation_order() {
    // Both variants reuse the SAME absolute root path (recreated fresh
    // between runs) — otherwise the two `TempDir`s' distinct absolute
    // paths leak into `rollup_inputs[].space_root`, and a raw byte
    // comparison would fail on path drift that has nothing to do with
    // the property under test (directory-creation-order independence).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    async fn build_and_compose(
        root: &Path,
        cache_root: &Path,
        create_zzz_first: bool,
    ) -> stack_model::ComposedLibrary {
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
        write_bare_root(root);

        let make_aaa = |root: &Path| {
            let c = root.join("aaa-ext");
            write_bare_root(&c);
            write_agent(&c, "shared", "shared", "AAA");
        };
        let make_zzz = |root: &Path| {
            let c = root.join("zzz-ext");
            write_bare_root(&c);
            write_agent(&c, "shared", "shared", "ZZZ");
        };
        if create_zzz_first {
            make_zzz(root);
            make_aaa(root);
        } else {
            make_aaa(root);
            make_zzz(root);
        }

        ship_resolver::roll_up_members(&[root.to_path_buf()], Some(cache_root))
            .await
            .unwrap()
    }

    let a = build_and_compose(&root, &cache_root, false).await;
    let b = build_and_compose(&root, &cache_root, true).await;

    let a_bytes = descriptor::extension_library_canonical(&a.library).unwrap();
    let b_bytes = descriptor::extension_library_canonical(&b.library).unwrap();
    assert_eq!(
        a_bytes, b_bytes,
        "composition must not depend on filesystem creation order, only sorted dir names"
    );

    let winner = a
        .library
        .agent_profiles
        .iter()
        .find(|ag| ag.agent.id == "shared")
        .unwrap();
    // Document the actual ruled winner (compose.rs: equal-rank ties
    // resolve by DECLARED order, later wins; `discover_child_extension_roots`
    // declares children in lexicographic dir-name order, so "zzz-ext" is
    // declared after "aaa-ext" and wins).
    assert_eq!(
        winner.agent.name, "ZZZ",
        "equal-rank tie-break is lexicographic-last-wins per compose.rs's documented law \
         (\"equal ranks resolve by declared extension order, later wins\"); if this fails \
         the tie-break rule changed or is non-deterministic — a FINDING either way"
    );
}

/// Multi-member roll-up: two independently registered member stacks (not
/// parent/child — siblings via `roll_up_members`' explicit membership)
/// both author the same skill id. Caller order is the only precedence
/// input (both are Space-rank); "later in the caller's member list wins"
/// must hold and must be stable across repeated runs.
#[tokio::test(flavor = "current_thread")]
async fn sibling_members_collision_resolves_by_caller_order_deterministically() {
    let tmp = TempDir::new().unwrap();
    let a_root = tmp.path().join("member-a");
    let b_root = tmp.path().join("member-b");
    write_bare_root(&a_root);
    write_bare_root(&b_root);
    write_skill(&a_root, "dup", "member A version");
    write_skill(&b_root, "dup", "member B version");

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let composed = ship_resolver::roll_up_members(&[a_root.clone(), b_root.clone()], Some(&cache_root))
        .await
        .unwrap();
    let dup: Vec<_> = composed.library.skills.iter().filter(|s| s.id == "dup").collect();
    assert_eq!(dup.len(), 1, "exactly one winner for `dup`: {dup:?}");

    // Repeat with the SAME order — must be stable (not just "a valid
    // winner" but the SAME winner every time).
    let composed2 = ship_resolver::roll_up_members(&[a_root.clone(), b_root.clone()], Some(&cache_root))
        .await
        .unwrap();
    assert_eq!(
        composed.library.skills.iter().find(|s| s.id == "dup").unwrap().description,
        composed2.library.skills.iter().find(|s| s.id == "dup").unwrap().description,
        "the same caller order must produce the same winner on every run"
    );
}

// ─── Scenario 2: deep + wide extension graphs ──────────────────────────

/// `discover_child_extension_roots` is documented as depth-1 (immediate
/// children of a member root only — "no recursion past depth 1" is the
/// SPACE scan's rule; the per-member nested-extension scan in
/// `roll_up_members` calls it exactly once per member, never recursively
/// into a discovered child's own children). Assert that contract holds:
/// a grandchild extension (child-of-a-child) is NOT silently composed in
/// — either it's absent (bounded depth, by design) or present (deeper
/// recursion exists) but NEVER does composing it corrupt/hang/duplicate.
/// This pins current behavior so a future depth change is a deliberate,
/// visible diff here rather than a silent surface change.
#[tokio::test(flavor = "current_thread")]
async fn grandchild_extension_is_not_recursively_discovered() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    write_agent(&root, "root", "root-agent", "Root");

    let child = root.join("child");
    write_bare_root(&child);
    write_agent(&child, "c", "child-agent", "Child");

    let grandchild = child.join("grandchild");
    write_bare_root(&grandchild);
    write_agent(&grandchild, "g", "grandchild-agent", "Grandchild");

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();

    let ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(ids.contains("root-agent"));
    assert!(ids.contains("child-agent"));
    assert!(
        !ids.contains("grandchild-agent"),
        "current contract: `roll_up_members` discovers ONE level of child \
         extensions per member root and does not recurse into a child's own \
         children (`discover_child_extension_roots` is called exactly once \
         on the member root, never on discovered children — see \
         `members.rs::roll_up_members`). If this assertion fails, depth-1 \
         became recursive: verify cycle safety before relying on it. \
         composed ids = {ids:?}"
    );
}

/// Wide graph: one member root with N sibling child extensions (no
/// nesting), each contributing a unique agent plus a fixed id every
/// child ALSO defines (maximum collision pressure). Assert: every unique
/// id survives, the shared id resolves to exactly one winner, and
/// runtime scales roughly linearly (not exponentially) with N.
#[tokio::test(flavor = "current_thread")]
async fn wide_sibling_extension_graph_composes_correctly_and_boundedly() {
    const N: usize = 60;
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);

    for i in 0..N {
        let child = root.join(format!("ext-{i:03}"));
        write_bare_root(&child);
        write_agent(&child, "unique", &format!("agent-{i:03}"), &format!("Agent {i}"));
        write_agent(&child, "shared", "shared-id", &format!("Shared from {i}"));
    }

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let start = Instant::now();
    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    let ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    for i in 0..N {
        assert!(
            ids.contains(format!("agent-{i:03}").as_str()),
            "every unique per-extension agent must survive: missing agent-{i:03}"
        );
    }
    let shared_count = composed
        .library
        .agent_profiles
        .iter()
        .filter(|a| a.agent.id == "shared-id")
        .count();
    assert_eq!(
        shared_count, 1,
        "N-way collision across {N} extensions must resolve to exactly one winner"
    );
    assert_eq!(composed.library.agent_profiles.len(), N + 1, "N unique + 1 shared winner");
    assert!(
        elapsed.as_secs() < 30,
        "composing {N} sibling extensions took {elapsed:?} — looks exponential, not bounded"
    );
}

// ─── Scenario 3: cycles ─────────────────────────────────────────────────

/// A child extension directory that is a SYMLINK back to its own parent
/// (the member root) — the closest on-disk approximation of a
/// self-referential extension graph reachable through the real
/// `discover_child_extension_roots` walk (which never recurses past
/// depth 1; see `grandchild_extension_is_not_recursively_discovered`).
/// Because discovery is depth-1 only, a symlink cycle cannot actually be
/// walked into — this test proves that: roll-up completes promptly and
/// composes the space root's own content exactly once, never looping.
#[tokio::test(flavor = "current_thread")]
async fn symlinked_self_referential_child_does_not_hang_or_duplicate() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    write_agent(&root, "root", "root-agent", "Root");

    #[cfg(unix)]
    {
        let link = root.join("self-loop");
        std::os::unix::fs::symlink(&root, &link).unwrap();
    }

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let start = Instant::now();
    let composed = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root)),
    )
    .await
    .expect("roll_up_members must not hang on a self-referential symlinked child")
    .expect("roll_up_members must not error on a self-referential symlinked child");
    let elapsed = start.elapsed();

    let root_agents = composed
        .library
        .agent_profiles
        .iter()
        .filter(|a| a.agent.id == "root-agent")
        .count();
    assert_eq!(
        root_agents, 1,
        "the root's own agent must compose exactly once, not once per cycle traversal"
    );
    assert!(elapsed.as_secs() < 30, "composition must complete promptly: {elapsed:?}");
}

/// Two members registered such that member A's directory contains a
/// symlinked child pointing at member B's root, and vice versa — an
/// attempt to construct a cross-member cycle. `roll_up_members` treats
/// membership as explicit (never scans siblings), so this must not
/// smuggle B's content into A's nested-child walk beyond what a normal
/// depth-1 scan would find, and must not hang.
#[tokio::test(flavor = "current_thread")]
#[cfg(unix)]
async fn cross_member_symlink_cycle_does_not_hang() {
    let tmp = TempDir::new().unwrap();
    let a_root = tmp.path().join("member-a");
    let b_root = tmp.path().join("member-b");
    write_bare_root(&a_root);
    write_bare_root(&b_root);
    write_agent(&a_root, "a", "a-agent", "A");
    write_agent(&b_root, "b", "b-agent", "B");

    std::os::unix::fs::symlink(&b_root, a_root.join("points-at-b")).unwrap();
    std::os::unix::fs::symlink(&a_root, b_root.join("points-at-a")).unwrap();

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let composed = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        ship_resolver::roll_up_members(&[a_root.clone(), b_root.clone()], Some(&cache_root)),
    )
    .await
    .expect("cross-member symlink cycle must not hang")
    .expect("cross-member symlink cycle must not error");

    let a_count = composed.library.agent_profiles.iter().filter(|x| x.agent.id == "a-agent").count();
    let b_count = composed.library.agent_profiles.iter().filter(|x| x.agent.id == "b-agent").count();
    assert_eq!(a_count, 1, "member A's agent composes exactly once");
    assert_eq!(b_count, 1, "member B's agent composes exactly once");
}

// ─── Scenario 4: malformed / missing manifests ─────────────────────────

/// Garbage (non-TS, non-JSON) content at `stack.config.ts`. Per
/// `real_manifest.rs`'s documented contract, `ts_resolver` tolerates a
/// failing module by warning and dropping ITS declarations — the roll-up
/// as a whole must still succeed (never panic, never error the whole
/// composition), just without whatever the garbage module would have
/// declared.
#[tokio::test(flavor = "current_thread")]
async fn garbage_stack_config_ts_does_not_panic_or_fail_the_rollup() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("stack.config.ts"), "{{{ not valid TS at all !!! ]][[").unwrap();

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root)).await;
    assert!(
        composed.is_ok(),
        "a syntactically-garbage stack.config.ts must degrade gracefully, not error the roll-up: {composed:?}"
    );
    assert!(composed.unwrap().library.agent_profiles.is_empty());
}

/// Empty `stack.config.ts` (zero bytes) — the degenerate "manifest
/// exists but declares nothing" case.
#[tokio::test(flavor = "current_thread")]
async fn empty_stack_config_ts_composes_to_empty_library() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("stack.config.ts"), "").unwrap();

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .expect("empty manifest must not error");
    assert!(composed.library.agent_profiles.is_empty());
}

/// A missing `stack.config.ts` entirely (member root exists, but the
/// manifest file is absent) — must not panic; `read_ship_config_ts`
/// returns `Ok(None)` per its documented contract.
#[tokio::test(flavor = "current_thread")]
async fn missing_manifest_file_does_not_panic() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    std::fs::create_dir_all(&root).unwrap();
    // No stack.config.ts written at all.

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root)).await;
    assert!(composed.is_ok(), "a member root with no manifest must not error: {composed:?}");
}

/// A member root that does not exist on disk AT ALL. `roll_up_members`
/// is handed an explicit path from the caller (e.g. a stale registry
/// row) — assert this degrades gracefully rather than panicking the
/// whole daemon.
#[tokio::test(flavor = "current_thread")]
async fn nonexistent_member_root_does_not_panic() {
    let tmp = TempDir::new().unwrap();
    let ghost = tmp.path().join("does-not-exist-at-all");
    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let result = ship_resolver::roll_up_members(&[ghost], Some(&cache_root)).await;
    // Graceful handling means EITHER a clean Err (surfaced, not a panic)
    // OR an empty-but-valid composition — never a panic/abort. Since this
    // runs in-process, a panic would abort the test with a different
    // failure signature (not a normal assertion failure); reaching this
    // line at all is the proof of "no panic".
    match result {
        Ok(composed) => assert!(composed.library.agent_profiles.is_empty()),
        Err(e) => assert!(!e.to_string().is_empty(), "error must carry context: {e}"),
    }
}

/// Corrupt `ship.lock` (unparseable JSON) — must surface as a clean
/// error from `roll_up_members`, not a panic.
#[tokio::test(flavor = "current_thread")]
async fn corrupt_ship_lock_does_not_panic() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    std::fs::write(root.join("ship.lock"), "this is not { valid json").unwrap();

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    // Reaching this assertion (rather than a test-process abort) is the
    // "no panic" proof; a clean Err is acceptable graceful handling.
    let _ = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root)).await;
}

/// `ship.lock` references a package hash with no matching cache object
/// on disk (partially-installed dependency). Per `load_cached_extensions`'s
/// documented contract this is a silent skip, not an error.
#[tokio::test(flavor = "current_thread")]
async fn lockfile_referencing_missing_cache_object_is_skipped_not_errored() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    write_lock(&root, &[("github.com/ghost/pkg", "1.0.0", "deadnotexist00000000")]);

    let cache_root = tmp.path().join("cache");
    std::fs::create_dir_all(&cache_root).unwrap(); // exists, but no `objects/<hex>` dir

    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .expect("a locked-but-uncached package must be silently skipped, not error");
    assert!(composed.library.agent_profiles.is_empty());
}

/// Garbage content inside a cached package's `agents/*.jsonc` file — the
/// documented contract (`dep_skills.rs::load_cached_extensions`) is
/// warn-and-skip per malformed entry; the rest of the roll-up (including
/// other, valid entries in the SAME package) must still compose.
#[tokio::test(flavor = "current_thread")]
async fn malformed_cached_agent_entry_is_skipped_valid_siblings_survive() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);

    let cache_root = tmp.path().join("cache");
    let hex = "malformedcase0000000000000000000000000000000000000000000000000";
    let obj = cache_root.join("objects").join(hex);
    std::fs::create_dir_all(obj.join("agents")).unwrap();
    std::fs::write(
        obj.join("ship.jsonc"),
        r#"{ "id": "p", "module": { "name": "github.com/g/h", "version": "1.0.0" },
             "exports": { "skills": [], "agents": ["agents/broken.jsonc", "agents/ok.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(obj.join("agents/broken.jsonc"), "{ not valid jsonc at all [[[").unwrap();
    std::fs::write(
        obj.join("agents/ok.jsonc"),
        r#"{ "agent": { "id": "ok", "name": "OK" } }"#,
    )
    .unwrap();
    write_lock(&root, &[("github.com/g/h", "1.0.0", hex)]);

    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .expect("malformed sibling entry must not fail the whole roll-up");
    let ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(
        ids.contains("github.com/g/h/ok"),
        "the valid sibling entry must still compose: {ids:?}"
    );
}

// ─── Scenario 5: scale ──────────────────────────────────────────────────

/// Hundreds of agents/skills across many extensions, all with unique ids
/// (no collision pressure — this isolates raw scale from the collision
/// path exercised by scenario 1/2). Assert full correctness (every id
/// present, none duplicated) and a sane wall-clock bound.
#[tokio::test(flavor = "current_thread")]
async fn hundreds_of_unique_entries_across_many_extensions_compose_correctly() {
    const EXTENSIONS: usize = 25;
    const AGENTS_PER_EXT: usize = 8;
    const SKILLS_PER_EXT: usize = 8;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);

    for e in 0..EXTENSIONS {
        let child = root.join(format!("ext-{e:03}"));
        write_bare_root(&child);
        for a in 0..AGENTS_PER_EXT {
            write_agent(
                &child,
                &format!("agent-{a}"),
                &format!("e{e:03}-a{a}"),
                &format!("Agent {e}-{a}"),
            );
        }
        for s in 0..SKILLS_PER_EXT {
            write_skill(&child, &format!("e{e:03}-s{s}"), "body");
        }
    }

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();

    let start = Instant::now();
    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    let expected_agents = EXTENSIONS * AGENTS_PER_EXT;
    let expected_skills = EXTENSIONS * SKILLS_PER_EXT;
    assert_eq!(
        composed.library.agent_profiles.len(),
        expected_agents,
        "every unique agent across {EXTENSIONS} extensions must survive with none dropped or duplicated"
    );
    assert_eq!(
        composed.library.skills.len(),
        expected_skills,
        "every unique skill across {EXTENSIONS} extensions must survive with none dropped or duplicated"
    );

    let agent_ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert_eq!(agent_ids.len(), expected_agents, "no duplicate agent ids sneak in");

    assert!(
        elapsed.as_secs() < 60,
        "composing {expected_agents} agents + {expected_skills} skills across \
         {EXTENSIONS} extensions took {elapsed:?} — not bounded"
    );

    // Determinism holds at scale too — re-run and compare canonical bytes.
    let composed2 = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();
    let bytes1 = descriptor::extension_library_canonical(&composed.library).unwrap();
    let bytes2 = descriptor::extension_library_canonical(&composed2.library).unwrap();
    assert_eq!(bytes1, bytes2, "scale must not break determinism");
}

/// Many cached packages (the cache tier scaled up) alongside the source
/// tier — proves the `load_cached_extensions` fan-out + `compose_space`
/// stay correct and bounded when the cache side, not just the source
/// side, is large.
#[tokio::test(flavor = "current_thread")]
async fn many_cached_packages_compose_correctly_and_boundedly() {
    const PACKAGES: usize = 100;
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);

    let cache_root = tmp.path().join("cache");
    let mut lock_entries: Vec<(String, String, String)> = Vec::new();
    for i in 0..PACKAGES {
        let hex = format!("{i:064x}");
        let pkg_path = format!("github.com/scale/pkg{i:03}");
        cached_agent_object(&cache_root, &hex, &pkg_path, "member", &format!("Member {i}"));
        lock_entries.push((pkg_path, "1.0.0".to_string(), hex));
    }
    let refs: Vec<(&str, &str, &str)> = lock_entries
        .iter()
        .map(|(p, v, h)| (p.as_str(), v.as_str(), h.as_str()))
        .collect();
    write_lock(&root, &refs);

    let start = Instant::now();
    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    assert_eq!(
        composed.library.agent_profiles.len(),
        PACKAGES,
        "every cached package's namespaced agent must survive, none collide (distinct pkg_path prefixes)"
    );
    assert!(
        elapsed.as_secs() < 60,
        "composing {PACKAGES} cached packages took {elapsed:?} — not bounded"
    );
}

// ─── Extra collision coverage: views, rules, mcp (every noun, not just
//     agents/skills) ────────────────────────────────────────────────────

fn write_view(root: &Path, file_stem: &str, route: &str, component: &str) {
    std::fs::create_dir_all(root.join("views")).unwrap();
    // Views arrive via `defineView` in `stack.config.ts` (no dedicated
    // per-noun dir); append to the existing manifest instead of writing a
    // separate `views/*.ts` file, since `PER_NOUN_SOURCE_DIRS` doesn't
    // include `views`.
    let manifest = root.join("stack.config.ts");
    let existing = std::fs::read_to_string(&manifest).unwrap_or_default();
    let addition = format!(
        r#"
import {{ defineView }} from "ship";
defineView({{ route: "{route}", component_ref: "{component}" }}); // {file_stem}
"#
    );
    std::fs::write(&manifest, format!("{existing}\n{addition}")).unwrap();
}

/// Same-prefix view route collision between space root and a child
/// extension: `compose_views`' key is `(prefix, route)`, and BOTH tiers
/// mount under the empty (source) prefix, so a same-route collision
/// between root and child must resolve to exactly one winner (never
/// both surviving side by side — that's reserved for DISTINCT prefixes).
#[tokio::test(flavor = "current_thread")]
async fn same_prefix_view_route_collision_resolves_to_one_winner() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    write_view(&root, "root-view", "/dashboard", "RootDashboard.sui");

    let child = root.join("child-ext");
    write_bare_root(&child);
    write_view(&child, "child-view", "/dashboard", "ChildDashboard.sui");

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();

    let matches: Vec<_> = composed
        .library
        .views
        .iter()
        .filter(|v| v.prefix == "" && v.route == "/dashboard")
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "same (prefix, route) from root + child must resolve to exactly one view, never both: {matches:?}"
    );
}

/// Rules collide via the agent's inline `rules: { inline }` body, which
/// harvests into `library.rules` keyed by `file_name = {agent_id}.md`.
/// Two agents with the SAME id (one at root, one in a child) each
/// declaring different inline rule text: the rule collision must track
/// the same winner as the agent collision (both keyed on `demo`/`demo.md`),
/// never a split where the agent picks one tier and the rule picks
/// another.
#[tokio::test(flavor = "current_thread")]
async fn rule_and_agent_collision_winners_stay_consistent_across_nouns() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    std::fs::create_dir_all(root.join("agents")).unwrap();
    std::fs::write(
        root.join("agents/demo.ts"),
        r#"import { defineAgent } from "ship";
defineAgent({ agent: { id: "demo", name: "Root Demo" }, rules: { inline: "Root rule body." } });
"#,
    )
    .unwrap();

    let child = root.join("child-ext");
    write_bare_root(&child);
    std::fs::create_dir_all(child.join("agents")).unwrap();
    std::fs::write(
        child.join("agents/demo.ts"),
        r#"import { defineAgent } from "ship";
defineAgent({ agent: { id: "demo", name: "Child Demo" }, rules: { inline: "Child rule body." } });
"#,
    )
    .unwrap();

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .unwrap();

    let agent = composed
        .library
        .agent_profiles
        .iter()
        .find(|a| a.agent.id == "demo")
        .expect("exactly one `demo` agent must survive");
    let rules: Vec<_> = composed.library.rules.iter().filter(|r| r.file_name == "demo.md").collect();
    assert_eq!(rules.len(), 1, "exactly one `demo.md` rule must survive: {rules:?}");

    let agent_is_root = agent.agent.name == "Root Demo";
    let rule_is_root = rules[0].content == "Root rule body.";
    assert_eq!(
        agent_is_root, rule_is_root,
        "the agent collision winner and the rule collision winner must be the SAME tier \
         (agent={:?} rule={:?}) — a split would mean an agent's identity and its own \
         inline rule body came from two different, inconsistent sources",
        agent.agent.name, rules[0].content
    );
}

/// One malformed `.ts` file among many valid ones in the SAME per-noun
/// source dir (`agents/`): per `ts_resolver::resolve_ts_files`'s
/// documented per-module leniency (warn + drop that module's
/// declarations, fan-out continues), the malformed file's agent must be
/// silently absent while every other sibling agent in the same
/// directory still composes. This is the single-extension analogue of
/// `malformed_cached_agent_entry_is_skipped_valid_siblings_survive`,
/// but for the SOURCE tier's TS eval path rather than the cache's JSONC
/// parse path — the two failure surfaces are handled by different code
/// (`ts_resolver` vs `dep_skills`) and both must be proven leniently.
#[tokio::test(flavor = "current_thread")]
async fn one_malformed_source_agent_file_does_not_drop_its_siblings() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("space");
    write_bare_root(&root);
    std::fs::create_dir_all(root.join("agents")).unwrap();
    std::fs::write(
        root.join("agents/broken.ts"),
        "this is not valid TypeScript at all {{{ [[[ ???",
    )
    .unwrap();
    write_agent(&root, "healthy", "healthy-agent", "Healthy");

    let cache_root = tmp.path().join("empty_cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    let composed = ship_resolver::roll_up_members(&[root.clone()], Some(&cache_root))
        .await
        .expect("one malformed sibling module must not fail the whole roll-up");

    let ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(
        ids.contains("healthy-agent"),
        "the healthy sibling agent must survive a malformed sibling module: {ids:?}"
    );
}
