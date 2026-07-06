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
//!   source + every locked cached package + the operator-enabled library
//!   tier (`ShipHome::library_dir()`) into a single
//!   [`compiler::ComposedLibrary`]. The composed library carries
//!   `rollup_inputs: Some(...)` so J3 (def index) + J4 (view) can audit
//!   what composed into the result. Cached packages are tagged
//!   `ExtSource::Cache` (rank 0), the enabled library tier
//!   `ExtSource::UserGlobal` (rank 500) — the source `ExtSource::Space`
//!   (rank 1000) wins on id collision over both.
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
/// `library_root` defaults to `ShipHome::library_dir()` (the per-user
/// globally-enabled extension tier — `data/library/{agents,skills,...}`).
///
/// ## The tier model
///
/// `default_precedence` defines: cache (rank 0) < user-global (rank 500) <
/// space (rank 1000) < agent overlay (rank 2000). Cached packages (locked
/// via `ship.lock`) are tagged `ExtSource::Cache`; the operator-enabled
/// library tier is tagged `ExtSource::UserGlobal` — the two were one
/// overloaded tier before the format-migration Wave 2 split (census Part
/// 3.4). A cached package's id collision with a same-id source agent is
/// recorded as `losing_source: Cache, winning_source: Space`. The
/// `losing_tier_hash` is the losing extension's content hash, so the index
/// audit can identify which tier lost.
pub async fn roll_up(
    space_root: &Path,
    cache_root: Option<&Path>,
    library_root: Option<&Path>,
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

    // 4b. Load the enabled tier: extensions the operator enabled globally,
    // via `ShipHome::library_dir()` (`data/library/{agents,skills,rules,...}`
    // — the same flat layout `compiler::load_library` reads for a space's
    // `.ship/`). A missing or empty library dir composes to an empty tier,
    // not an error — every sub-loader already treats a missing dir as empty.
    let default_library_root;
    let resolved_library_root: &Path = match library_root {
        Some(p) => p,
        None => {
            default_library_root = ShipHome::from_env().library_dir();
            &default_library_root
        }
    };
    let enabled_library = compiler::load_library(resolved_library_root).with_context(|| {
        format!(
            "roll_up: enabled-tier load failed for {}",
            resolved_library_root.display()
        )
    })?;
    let enabled_content_hash = descriptor::extension_library_content_hash(&enabled_library)
        .context("roll_up: enabled-tier content hash")?;

    // 5. Build the extension set. Cached packages come FIRST in declared
    //    order so the source — also `ExtSource::Space` (rank 1000) at the
    //    same rank — wins on equal-rank ties (the `compose_space` law:
    //    "later declared wins"). Every other tier has a distinct rank, so
    //    declared order is otherwise irrelevant.
    let mut extensions: Vec<ExtensionDescriptor> = Vec::with_capacity(bundles.len() + 2);
    for bundle in &bundles {
        let mut descriptor = ExtensionLibrary::default();
        descriptor.agent_profiles = bundle.agent_profiles.clone();
        descriptor.skills = bundle.skills.clone();
        extensions.push(ExtensionDescriptor {
            source: ExtSource::Cache,
            space_root: space_root.to_path_buf(),
            descriptor,
            content_hash: bundle.content_hash.clone(),
        });
    }
    extensions.push(ExtensionDescriptor {
        source: ExtSource::UserGlobal,
        space_root: space_root.to_path_buf(),
        descriptor: enabled_library,
        content_hash: enabled_content_hash,
    });
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

/// Synchronous bridge over [`roll_up`] for callers that cannot `.await` —
/// e.g. `ship-code`'s `AgentPlatform::compile_agent_config`, a sync trait
/// method the dispatch path calls from inside the daemon's (multi-thread)
/// Tokio runtime. Reuses the calling runtime via `block_in_place` when one
/// is already current; falls back to a throwaway current-thread runtime for
/// plain sync callers (e.g. a `#[test]` with no ambient runtime).
pub fn roll_up_blocking(
    space_root: &Path,
    cache_root: Option<&Path>,
    library_root: Option<&Path>,
) -> Result<ComposedLibrary> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(roll_up(space_root, cache_root, library_root))
        }),
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("roll_up_blocking: building fallback runtime")?
            .block_on(roll_up(space_root, cache_root, library_root)),
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
