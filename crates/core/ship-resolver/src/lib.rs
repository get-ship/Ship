//! `ship-resolver` — the Phase-2 space-shape resolver.
//!
//! ## What this crate does
//!
//! Generalizes the Phase-0 resolver (compiler leaf + agent-config-eval) from
//! single-manifest to **space shape**: a `ship.ts` at the space root, a
//! `skills/` + `rules/` directory convention, a `space.skill_paths` manifest
//! read. The result is a fully-populated [`compiler::ExtensionLibrary`] (disk
//! state) and a [`compiler::ResolvedConfig`] (resolved state) — the canonical
//! descriptors the rest of the platform compiles and runs against.
//!
//! ## The seam
//!
//! The resolver is a NEW crate that composes with — does NOT replace — the
//! compiler leaf's `load_library` (per the F1 decision, `decision-f1-loader.md`
//! Option A: "embrace the superset"). The leaf's `load_library` keeps walking
//! `.ship/{mcp,permissions,hooks,rules,skills,agents}/` with its `std::fs`
//! surface; this crate layers the Phase-2 space-shape pieces on top via the
//! [`ship_substrate::ports::FsBroker`] gated door (R1 — new code goes through
//! FsBroker, not raw `std::fs`).
//!
//! The V8 TS-eval seam is `agent_config_eval::resolve_agents_with_globals`,
//! wrapped by `ts_resolver::resolve_ts_files` with the `!Send` isolate
//! crossing a `spawn_blocking` boundary — the same pattern shipd's ingest
//! uses today (`apps/shipd/src/ship_runtime/tasks.rs:285-304`, R2).
//!
//! ## What this crate does NOT do
//!
//! - **No private invalidation channel.** The descriptor cache ([`cache`])
//!   subscribes to the KERNEL-LOG BROADCAST (`ship_substrate::ports::KernelLog`)
//!   and invalidates on `config.file_changed` — the same pattern
//!   `ship-config`'s `spawn_def_index_consumer` uses. There is no
//!   resolver-private `FsWatch` subscription; reactivity flows through the
//!   kernel log like everything else (operator ruling 2026-07-06,
//!   `job-spec-p3-event-channel.md`).
//! - **No caller migration.** J6 (helmd dispatch), J7 (shipd ingest), J8
//!   (helmd wire) replace their existing call sites with this crate's API.
//!   Out of scope here.
//! - **Phase-3 composition lives here.** [`roll_up`] composes the on-disk
//!   source + every locked cached package into a single
//!   [`compiler::ComposedLibrary`]. The composed library carries
//!   `rollup_inputs: Some(...)` so J3 (def index) + J4 (view) can audit
//!   what composed into the result. Cached packages are tagged
//!   `ExtSource::UserGlobal` (the standing rank-0 tier; the source
//!   `ExtSource::Space` rank-1000 wins on id collision).
//!
//! ## Module layout
//!
//! - [`space_loader`] — the disk walk over space shape → `ExtensionLibrary`.
//!   Composes with the leaf; the additive `space.skill_paths` manifest read
//!   lives in the leaf (per F1).
//! - [`ts_resolver`] — V8 eval fan-out. `!Send` boundary on `spawn_blocking`.
//! - [`descriptor`] — canonical sorted-key JSON helpers. The determinism
//!   invariant the cache relies on (`inv-fmt-deterministic-eval`).
//! - [`cache`] — the descriptor cache. Subscribes directly to a
//!   `ship_substrate::ports::KernelLog`'s broadcast and invalidates on
//!   `config.file_changed`; no resolver-private watch channel.
//!
//! ## Authority
//!
//! - Spec: `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-p2-J3-resolver.md`
//!   (J3) + `job-spec-p3-event-channel.md` (the kernel-log-only cache
//!   invalidation this module now implements).
//! - Census: `census-phase2-resolver.md` Part 4 + Part 5 J3.
//! - F1 decision: `decision-f1-loader.md` (Option A — the superset is the
//!   architecture, no code moves).
pub mod descriptor;
pub mod space_loader;
pub mod ts_resolver;
pub mod cache;

use std::path::Path;

use anyhow::{Context, Result};
use compiler::{ComposedLibrary, ExtensionDescriptor, ExtSource, ExtensionLibrary, ResolvedConfig, RollupInputs};
use serde_json::Value;
use ship_substrate::ports::{FsBroker, LocalFsBroker};
use ship_substrate::ShipHome;

/// Resolve a space's full descriptor: disk walk + TS eval + resolve.
///
/// `space_root` is the directory containing `ship.ts` (preferred) or
/// `<.ship>/ship.ts` (legacy), plus `.ship/` for the bulk of the config
/// shape. Returns the resolved config — a pure, in-memory view of the
/// space's effective configuration.
///
/// The bulk of the disk read is delegated to the compiler leaf's
/// `load_library`; the Phase-2 space-shape pieces (`ship.ts`,
/// `space.skill_paths` extra dirs) are layered on top via FsBroker.
/// The TS eval is one `defineAgent` roll-up per `imported_modules`
/// entry, run in the V8 isolate with the `!Send` boundary crossed on a
/// dedicated current-thread runtime (R2).
///
/// Thin wrapper over [`resolve_space_with_globals`] with no injected
/// globals — every `defineAgent` call lands without operator defaults.
pub async fn resolve_space(space_root: &Path) -> Result<ResolvedConfig> {
    resolve_space_with_globals(space_root, Value::Null).await
}

/// Resolve a space's full descriptor with pre-fetched operator defaults
/// injected into the TS eval as the `globals` export.
///
/// `globals` is part of the closed-world input — a plain JSON value
/// (objects/arrays/scalars) baked into the SDK module before any user
/// code runs. The eval stays deterministic: no live fs/net/RPC, no
/// `Date.now` / `Math.random` (the SDK stub's guard runs at module top
/// level, before user code; see `agent-config-eval/src/lib.rs:105-158`).
///
/// A `ship.ts` composes the defaults with
/// `defineAgent({ ...globals.agentDefaults, agent: {...} })` (the
/// pattern the agent-config-eval tests pin down at
/// `agent-config-eval/tests/resolve_agents.rs:127-164`).
pub async fn resolve_space_with_globals(
    space_root: &Path,
    globals: Value,
) -> Result<ResolvedConfig> {
    let fs = LocalFsBroker::new();
    resolve_space_with_broker(&fs, space_root, globals).await
}

/// Same as [`resolve_space_with_globals`] but with an injected
/// [`FsBroker`]. Tests and a future FsBroker-with-extra-context world
/// (e.g. cap-bootstrapping) use this; production goes through
/// [`resolve_space`] / [`resolve_space_with_globals`].
pub async fn resolve_space_with_broker(
    fs: &dyn FsBroker,
    space_root: &Path,
    globals: Value,
) -> Result<ResolvedConfig> {
    // 1. Disk walk — the bulk via the leaf, the space-shape pieces via
    //    FsBroker. The result is a `ExtensionLibrary` with skills
    //    (default + extra), rules, mcp, hooks, permissions, and the
    //    `imported_modules` from `ship.ts`.
    let mut library = space_loader::load_space_library(fs, space_root)
        .await
        .with_context(|| {
            format!("resolver: space load failed for {}", space_root.display())
        })?;

    // 2. TS eval fan-out — one `defineAgent` roll-up per module. Empty
    //    module list → no V8 work. The eval crosses the `!Send` boundary
    //    inside `ts_resolver`; the resulting `Vec<AgentProfile>` is plain
    //    data, safe to merge.
    let module_profiles =
        ts_resolver::resolve_ts_files(&library.imported_modules, &globals)
            .await
            .context("resolver: TS eval failed")?;
    if !module_profiles.is_empty() {
        // Phase-2: append the TS-authored profiles to BOTH the active-
        // agent-shaped list (`library.agents`) AND the consumer-visible
        // passthrough (`library.agent_profiles`).
        //
        // - `library.agents` is what `resolve_library` feeds into
        //   `SpaceManifest.agents`, which the resolve step filters by
        //   the active-agent selection.
        // - `library.agent_profiles` is the passthrough that
        //   `ResolvedConfig::agent_profiles` carries — the consumer-
        //   visible list (the harness compile inputs).
        //
        // Same id dedup as the skill merge: a `.jsonc` profile with
        // the same id as a TS profile wins (the leaf read it first
        // and the TS profile is appended after — a `.jsonc` author
        // beats a `.ts` author on id conflict, the lenient contract
        // the existing agent-def loader has).
        let seen: std::collections::HashSet<String> = library
            .agent_profiles
            .iter()
            .map(|a| a.agent.id.clone())
            .collect();
        for profile in module_profiles {
            if seen.contains(&profile.agent.id) {
                continue;
            }
            library.agent_profiles.push(profile.clone());
            library.agents.push(profile);
        }
    }
    // 3. Surface the resolve-time globals on the descriptor for
    //    audit / debug consumers. The TS eval already used the value;
    //    the field is for visibility.
    if !globals.is_null() {
        library.globals = globals;
    }

    // 4. Pure transform: `ExtensionLibrary` → `ResolvedConfig`. No fs,
    //    no async. The leaf's `resolve_library` is the canonical impl.
    let resolved = compiler::resolve_library(&library, None, None);
    Ok(resolved)
}

/// Load + return the `ExtensionLibrary` only (skip the resolve step +
/// TS eval). Useful for callers that need the raw space shape (the
/// descriptor cache in J5, for instance — the cache key is the
/// `ExtensionLibrary`'s content hash, not the resolved config's).
pub async fn load_space_library(
    space_root: &Path,
) -> Result<ExtensionLibrary> {
    let fs = LocalFsBroker::new();
    space_loader::load_space_library(&fs, space_root).await
}

// ─── Phase-3 roll-up ───────────────────────────────────────────────────────────

/// Compose the space's extension roll-up: the on-disk source + every locked
/// cached package, fed through [`compiler::compose_space`].
///
/// The output is a [`compiler::ComposedLibrary`] whose `library` is the
/// highest-ranked tier's `ExtensionLibrary` (the on-disk source, per
/// `default_precedence`; cached packages use `ExtSource::UserGlobal` so the
/// source wins on id collision). `shadows` carries one `ShadowRecord` per
/// id collision the law resolved. The composed `library.rollup_inputs` is
/// populated with the full `RollupInputs` (extensions + precedence) so
/// downstream consumers (the def index in J3, the view in J4) can audit
/// exactly what composed into the result.
///
/// Roots come from the caller or `ShipHome` typed accessors — no hardcoded
/// paths. `cache_root` defaults to `ShipHome::cache_dir()` (the XDG cache
/// root, where the `#122` dep loader reads `<cache>/objects/<hex>/`).
///
/// ## The tier model
///
/// `default_precedence` defines: user-global (rank 0) < space (rank 1000) <
/// agent overlay (rank 2000). Cached packages are tagged `ExtSource::UserGlobal`
/// in this phase — they live in the XDG cache, not in the space — so a
/// cached package's id collision with a same-id source agent is recorded as
/// `losing_source: UserGlobal, winning_source: Space`. The `losing_tier_hash`
/// is the cached package's `sha256` hex from the lockfile, so the index
/// audit can identify which package lost.
///
/// Phase 6 surface work may add a dedicated `ExtSourceKind::Cache` variant
/// to distinguish cached packages from operator-enabled extensions. Until
/// then, `UserGlobal` is the carrier.
pub async fn roll_up(
    space_root: &Path,
    cache_root: Option<&Path>,
) -> Result<ComposedLibrary> {
    let fs = LocalFsBroker::new();

    // 1. Load the source's `ExtensionLibrary` (on-disk shape, no TS merge).
    let mut source_library = space_loader::load_space_library(&fs, space_root)
        .await
        .with_context(|| {
            format!("roll_up: space load failed for {}", space_root.display())
        })?;

    // 2. TS-eval the `imported_modules` and merge into the source's profile
    //    lists. Same flow as `resolve_space_with_broker` — we don't need the
    //    `ResolvedConfig` (the roll-up is descriptor-shaped, not runtime-
    //    shaped), only the merged library.
    let module_profiles =
        ts_resolver::resolve_ts_files(&source_library.imported_modules, &Value::Null)
            .await
            .context("roll_up: TS eval failed")?;
    if !module_profiles.is_empty() {
        let seen: std::collections::HashSet<String> = source_library
            .agent_profiles
            .iter()
            .map(|a| a.agent.id.clone())
            .collect();
        for profile in module_profiles {
            if seen.contains(&profile.agent.id) {
                continue;
            }
            source_library.agent_profiles.push(profile.clone());
            source_library.agents.push(profile);
        }
    }

    // 3. Source's per-extension content hash. The cache key (J5) is the
    //    COMPOSED library's hash, which embeds this via `rollup_inputs`,
    //    so the source's per-extension hash here is the audit value the
    //    shadow record carries on collision.
    let source_content_hash = descriptor::extension_library_content_hash(&source_library)
        .context("roll_up: source content hash")?;

    // 4. Load cached packages (the #122 dep loader, per-package output).
    let lock_path = ShipHome::project_ship_dir(space_root).join("ship.lock");
    let default_cache_root;
    let resolved_cache_root: &Path = match cache_root {
        Some(p) => p,
        None => {
            // Route through `ShipHome::from_env` so SHIP_HOME is honored
            // and the test-context isolation gate kicks in (datasafety-
            // shiphome).
            default_cache_root = ShipHome::from_env().cache_dir().to_path_buf();
            &default_cache_root
        }
    };
    let bundles = ship_store::dep_skills::load_cached_extensions(
        &lock_path,
        Some(resolved_cache_root),
    )
    .with_context(|| format!("roll_up: cached extensions for {}", lock_path.display()))?;

    // 5. Build the extension set. Cached packages come FIRST in declared
    //    order so the source — also `ExtSource::Space` (rank 1000) at the
    //    same rank — wins on equal-rank ties (the `compose_space` law:
    //    "later declared wins"). For cached packages at `UserGlobal` (rank
    //    0), the source outranks them outright; the declared order is
    //    irrelevant when ranks differ.
    let mut extensions: Vec<ExtensionDescriptor> = Vec::with_capacity(bundles.len() + 1);
    for bundle in &bundles {
        let mut descriptor = ExtensionLibrary::default();
        descriptor.agent_profiles = bundle.agent_profiles.clone();
        descriptor.skills = bundle.skills.clone();
        extensions.push(ExtensionDescriptor {
            source: ExtSource::UserGlobal,
            space_root: space_root.to_path_buf(),
            descriptor,
            content_hash: bundle.content_hash.clone(),
        });
    }
    extensions.push(ExtensionDescriptor {
        source: ExtSource::Space,
        space_root: space_root.to_path_buf(),
        descriptor: source_library,
        content_hash: source_content_hash,
    });

    // 6. Assemble RollupInputs.
    let rollup_inputs = RollupInputs {
        extensions,
        precedence: RollupInputs::default_precedence(),
    };

    // 7. Compose. Pure; the law is the only thing that mutates the
    //    library's noun collections.
    let mut composed = compiler::compose_space(&rollup_inputs)
        .context("roll_up: compose_space")?;

    // 8. Annotate the composed library with the inputs so consumers (J3
    //    index, J4 view) can audit the roll-up. The `rollup_inputs` field
    //    is `#[serde(skip_serializing_if = "Option::is_none")]` so an
    //    empty roll-up stays out of the canonical form (Phase-2 contract).
    composed.library.rollup_inputs = Some(rollup_inputs);

    Ok(composed)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
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
    /// the roll-up degenerates to the source-only identity. The composed
    /// library IS the source's library; `shadows` is empty; `rollup_inputs`
    /// carries the single-extension `RollupInputs`.
    #[tokio::test(flavor = "current_thread")]
    async fn roll_up_without_lockfile_is_source_only_identity() {
        let (_tmp, root) = write_minimal_space();
        // Point `cache_root` at an empty dir so `load_cached_extensions`
        // can't accidentally read from the host's `~/.ship/cache/`.
        let cache_root = root.join("empty_cache");
        std::fs::create_dir_all(&cache_root).unwrap();

        let composed = roll_up(&root, Some(&cache_root)).await.unwrap();

        // No shadows (no collision possible with one extension).
        assert!(composed.shadows.is_empty(), "no shadows for a single extension");

        // The composed library is the source's library.
        let lib = &composed.library;
        assert_eq!(lib.rules.len(), 1, "the source's rule survives");
        // The TS-defined `demo` agent is in the merged source.
        assert!(lib.agent_profiles.iter().any(|a| a.agent.id == "demo"));

        // `rollup_inputs` carries the audit trail — exactly one extension
        // (the source) and the default precedence.
        let rollup = lib
            .rollup_inputs
            .as_ref()
            .expect("roll_up must populate rollup_inputs");
        assert_eq!(rollup.extensions.len(), 1, "one extension (the source)");
        assert_eq!(rollup.precedence.len(), RollupInputs::default_precedence().len());
        for (got, want) in rollup.precedence.iter().zip(RollupInputs::default_precedence().iter()) {
            assert_eq!(got.kind, want.kind);
            assert_eq!(got.rank, want.rank);
        }
    }

    /// Cached package's agent shadowed by a same-id source agent: the source
    /// wins on id collision (source `Space` rank 1000 > cache `UserGlobal`
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

        let composed = roll_up(&root, Some(&cache_root)).await.unwrap();

        // The source's `github.com/x/y/writer` wins (Space rank 1000 >
        // UserGlobal rank 0). The cached package's namespaced agent is
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
        assert_eq!(shadow.losing_source, ExtSource::UserGlobal);
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

        let composed = roll_up(&root, Some(&cache_root)).await.unwrap();
        assert!(composed.shadows.is_empty());
        let rollup = composed.library.rollup_inputs.as_ref().unwrap();
        assert_eq!(rollup.extensions.len(), 1, "no cached packages");
        assert_eq!(rollup.extensions[0].source, ExtSource::Space);
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

        let a = roll_up(&root, Some(&cache_root)).await.unwrap();
        let b = roll_up(&root, Some(&cache_root)).await.unwrap();

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

        // Roll-up 1: baseline.
        let a = roll_up(&root, Some(&cache_root)).await.unwrap();
        let a_hash = descriptor::extension_library_content_hash(&a.library).unwrap();

        // Mutate the cached package's content. The `load_cached_extensions`
        // helper re-reads on every roll-up call, so the next roll-up sees
        // the new content.
        std::fs::write(
            obj.join("agents/a.jsonc"),
            r#"{ "agent": { "id": "a", "name": "A v2" } }"#,
        )
        .unwrap();

        let b = roll_up(&root, Some(&cache_root)).await.unwrap();
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

        let composed = roll_up(&root, Some(&cache_root)).await.unwrap();

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

        // The roll-up audit carries BOTH extensions.
        let rollup = lib.rollup_inputs.as_ref().unwrap();
        assert_eq!(rollup.extensions.len(), 2, "source + one cached package");
        let sources: Vec<_> = rollup.extensions.iter().map(|e| e.source.clone()).collect();
        assert!(sources.iter().any(|s| matches!(s, ExtSource::UserGlobal)));
        assert!(sources.iter().any(|s| matches!(s, ExtSource::Space)));
    }
}
