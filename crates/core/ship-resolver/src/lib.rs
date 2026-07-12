//! `ship-resolver` — the Phase-2 space-shape resolver.
//!
//! ## What this crate does
//!
//! Generalizes the Phase-0 resolver (compiler leaf + agent-config-eval) from
//! single-manifest to **space shape**: a `ship.config.ts` at the space root, a
//! `skills/` + `rules/` directory convention, the on-disk permissions +
//! hooks reads. The result is a fully-populated [`compiler::ExtensionLibrary`]
//! (disk state) and a [`compiler::ResolvedConfig`] (resolved state) — the
//! canonical descriptors the rest of the platform compiles and runs against.
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
//! The V8 TS-eval seam is `agent_config_eval::resolve_declarations_with_globals`
//! (every declare-form, not just `defineAgent` — format-migration Wave 3),
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
//!   Composes with the leaf; the leaf populates the `skill_paths` field
//!   with the single default entry (no `space.skill_paths` ship.jsonc
//!   read — that field died with the dot-ship close-out).
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
pub mod cache;
pub mod descriptor;
pub mod space_loader;
pub mod ts_resolver;

use std::collections::HashSet;
use std::path::Path;

use agent_config_eval::Declarations;
use anyhow::{Context, Result};
use compiler::{
    ComposedLibrary, ExtSource, ExtensionDescriptor, ExtensionLibrary, ResolvedConfig, RollupInputs,
};
use serde_json::Value;
use ship_substrate::ShipHome;
use ship_substrate::ports::{FsBroker, LocalFsBroker};

/// Merge one eval's harvested [`Declarations`] into `library`'s noun
/// collections — the shared merge both [`resolve_space_with_broker`] and
/// [`roll_up`] apply after their TS eval fan-out.
///
/// Agents append to TWO lists in lockstep: `library.agents` (what
/// `resolve_library` feeds into `SpaceManifest.agents`, filtered by the
/// active-agent selection) and `library.agent_profiles` (the consumer-
/// visible passthrough `ResolvedConfig::agent_profiles` carries). Every
/// other noun has exactly one `ExtensionLibrary` field to land in.
///
/// Every noun uses the same id-collision policy: a `.jsonc`/`.md`-authored
/// entry (already in `library` from the leaf's disk walk, which runs
/// before this merge) wins over a same-id `.ts` declare-form — the
/// existing agent-def loader's lenient "space format read first wins"
/// contract, generalized to every noun this crate now harvests from TS.
///
/// Views are the one exception: the composition law keys on
/// `(prefix, route)`, not a bare id, so the merge can simply APPEND
/// harvested views to `library.views` — the same prefix's duplicate
/// `route` and a different prefix's identical `route` are both
/// legitimate cases, and dedup by anything narrower would silently
/// drop one. `compose_space`'s `compose_views` is the single source
/// of truth for view resolution; the merge here just gathers.
fn merge_ts_declarations(
    library: &mut ExtensionLibrary,
    declarations: Declarations,
    fs: &dyn ship_substrate::ports::FsBroker,
    space_root: &Path,
) {
    if !declarations.agents.is_empty() {
        let mut seen: HashSet<String> = library
            .agent_profiles
            .iter()
            .map(|a| a.agent.id.clone())
            .collect();
        for profile in declarations.agents {
            if !seen.insert(profile.agent.id.clone()) {
                continue;
            }
            library.agent_profiles.push(profile.clone());
            library.agents.push(profile);
        }
    }
    // Slice 2c close-out: MCP server registrations move to
    // `defineConfig({ agents: { mcp: { <name>: {…} } } })`. The per-call
    // `defineMcpServer` form is gone; the `agents.mcp` map is the only
    // source for MCP servers in the post-slice-2c surface. Walk
    // `declarations.config` (root-last-wins, same as `skills_dir` /
    // `rules_dir` overrides below) and convert each entry via
    // `McpServerDeclare::into_mcp_server_config`. The id-collision
    // policy matches the dead per-call form: the existing
    // `library.mcp_servers` (populated by the leaf's `load_mcp_servers`
    // for any pre-W4 `.ship/mcp.jsonc` entries) wins, then the FIRST
    // `defineConfig.agents.mcp.<id>` claim wins — `merge_by_key`'s
    // "first wins" semantics apply identically to the new path.
    // Slice 2c close-out determinism: `agents.mcp` is a `HashMap`,
    // and `HashMap` iteration order is randomized per process (the
    // default hasher seeds from the OS). The compile output's byte
    // sequence depends on the `mcp_servers` order in the canonical
    // `ExtensionLibrary`, so non-deterministic order = non-deterministic
    // output = parity-capture failures. Sort by `id` (the canonical
    // key, sorted ascending) before the `merge_by_key` first-wins pass.
    let mut mcp_from_config: Vec<compiler::McpServerConfig> = Vec::new();
    for cfg in &declarations.config {
        if let Some(agents) = &cfg.agents {
            for declared in agents.mcp.values() {
                mcp_from_config.push(declared.clone().into_mcp_server_config());
            }
        }
    }
    mcp_from_config.sort_by(|a, b| a.id.cmp(&b.id));
    merge_by_key(&mut library.mcp_servers, mcp_from_config, |m| m.id.as_str());
    merge_by_key(&mut library.skills, declarations.skills, |s| s.id.as_str());
    merge_by_key(&mut library.rules, declarations.rules, |r| {
        r.file_name.as_str()
    });
    merge_by_key(&mut library.pipelines, declarations.pipelines, |p| {
        p.name.as_str()
    });
    merge_by_key(&mut library.triggers, declarations.triggers, |t| {
        t.id.as_str()
    });
    merge_by_key(&mut library.collections, declarations.collections, |c| {
        c.id.as_str()
    });
    library.views.extend(declarations.views);

    // Per-extension-project on-disk skills + rules walks (the
    // resolution layer's source for these nouns post slice-2b).
    //
    // The walk happens here (not in the leaf) because the TS eval
    // already populated declared skills via `defineSkill` and
    // declared rules via `defineRule`. The disk walk duplicates the
    // surface for callers that author SKILL.md / rule `.md` files
    // directly — the dot-ship close-out's correct posture per
    // `data/blueprint/subsystems/ship-stack-format.md` (skills + rules
    // are on-disk directory conventions detected by the resolver; the
    // manifest is just a pointer via `defineConfig({ skills_dir,
    // rules_dir })`).
    //
    // For each path:
    // - The leaf populates the default (`skill_paths: ["skills/"]`,
    //   no leaf-side equivalent for `rules_dir` yet — the resolver
    //   applies the unified default "rules/" when no override).
    // - `defineConfig({ skills_dir | rules_dir })` overrides via
    //   root-last-wins scalar.
    // - The walked records MERGE into `library.skills` / `.rules`
    //   with the same id-dedup policy as the TS-declared side: an
    //   entry already in the library wins (the leaf's existing
    //   contract generalized to every noun this crate harvests).
    //
    // Without these walks the post-slice-2b close-out (skills +
    // rules un-flattened from `defineSkill` / `defineRule` calls
    // into on-disk files) would deliver an empty resolved surface —
    // these walks ARE the on-disk authority.
    let cap = ship_substrate::ports::FsCap::new_unchecked(
        space_root.to_path_buf(),
        ship_substrate::ports::FsRoot::Space(ship_substrate::ports::SpaceId::new("resolver")),
        ship_substrate::ports::FsMode::ReadOnly,
    );

    // ── Skills: resolve `skills_dir` from `defineConfig` (last wins)
    //    falling through to the leaf-populated `library.skill_paths[0]`
    //    (the post-W5 unified default `"skills/"`).
    //
    // The descriptor-level `skill_paths` field preserves the form the
    // source declared (the leaf default is slash-terminated,
    // `defineConfig`'s value is verbatim); `walk_skills_dir` strips
    // the slash internally so both forms produce identical walks.
    let resolved_skills_dir: String = declarations
        .config
        .iter()
        .filter_map(|c| c.skills_dir.clone())
        .next_back()
        .or_else(|| library.skill_paths.first().cloned())
        .unwrap_or_else(|| compiler::loader::DEFAULT_SPACE_SKILL_PATH.to_string());
    // Keep `library.skill_paths` tracking the resolved path (single
    // entry — the leaf populates it with the unified default; the
    // resolver-level override (per-extension-project
    // `defineConfig({ skills_dir })`) REPLACES that single entry).
    library.skill_paths = vec![resolved_skills_dir.clone()];
    {
        let mut seen: HashSet<String> = library.skills.iter().map(|s| s.id.clone()).collect();
        let mut added_skills: Vec<String> = Vec::new();
        match space_loader::walk_skills_dir(fs, &cap, &resolved_skills_dir) {
            Ok(found) => {
                for mut skill in found {
                    if seen.insert(skill.id.clone()) {
                        skill.source = compiler::SkillSource::Custom;
                        added_skills.push(skill.id.clone());
                        library.skills.push(skill);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "resolver: walk_skills_dir failed for {}: {e}",
                    resolved_skills_dir
                );
            }
        }
        // Stable order — the descriptor cache's content hash depends
        // on a stable iteration order. Sort ONLY when the walk
        // contributed new entries (declared skills already arrive
        // in stable call order; sorting unconditionally would
        // re-order them and break byte-parity with the prior W5
        // surface). Slice 2b close-out.
        if !added_skills.is_empty() {
            library.skills.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    // ── Rules: resolve `rules_dir` from `defineConfig` (last wins)
    //    falling through to the unified default `"rules/"`.
    let resolved_rules_dir: String = declarations
        .config
        .iter()
        .filter_map(|c| c.rules_dir.clone())
        .next_back()
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "rules".to_string());
    {
        let mut seen: HashSet<String> = library.rules.iter().map(|r| r.file_name.clone()).collect();
        let mut added_rules: Vec<String> = Vec::new();
        match space_loader::walk_rules_dir(fs, &cap, &resolved_rules_dir) {
            Ok(found) => {
                for rule in found {
                    if seen.insert(rule.file_name.clone()) {
                        added_rules.push(rule.file_name.clone());
                        library.rules.push(rule);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "resolver: walk_rules_dir failed for {}: {e}",
                    resolved_rules_dir
                );
            }
        }
        // Stable order — same conditional-sort policy as skills:
        // declared rules arrive in call order; only re-sort when the
        // on-disk walk contributed new entries. Slice 2b close-out.
        if !added_rules.is_empty() {
            library.rules.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        }
    }

    // ── Hooks: resolve `hooks_dir` from `defineConfig` (last wins)
    //    falling through to the unified default `"hooks/"`.
    //
    // Slice 3 close-out (A3): hooks arrive as on-disk files at
    // `<hooks_dir>/<event-stem>.<ext>`. The leaf no longer reads
    // `.ship/hooks.jsonc` — the resolver walks the per-extension-project
    // hooks dir, same as skills + rules. Filename → trigger mapping
    // happens in `space_loader::walk_hooks_dir`; the resolver is just
    // the side that decides WHERE the dir is (the `defineConfig`
    // pointer, falling through to the unified default).
    //
    // The walked records MERGE into `library.hooks` with the same
    // first-wins id-collision policy as every other noun this crate
    // harvests — an entry already in the library (the leaf's
    // presence-only stub, post slice 3 always empty) wins, then the
    // first on-disk file claim wins.
    let resolved_hooks_dir: String = declarations
        .config
        .iter()
        .filter_map(|c| c.hooks_dir.clone())
        .next_back()
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "hooks".to_string());
    // Track the resolved dir on the descriptor (`library.hooks_dir`)
    // so downstream consumers + diff tests see the same scalar the
    // walk used. `None` means "use the default" — the leaf's
    // post-slice-3 contract is "empty unless the resolver sets it",
    // so we always set it after the resolution step.
    library.hooks_dir = Some(resolved_hooks_dir.clone());
    {
        let mut seen: HashSet<String> = library.hooks.iter().map(|h| h.id.clone()).collect();
        let mut added_hooks: Vec<String> = Vec::new();
        match space_loader::walk_hooks_dir(fs, &cap, &resolved_hooks_dir) {
            Ok(found) => {
                for hook in found {
                    if seen.insert(hook.id.clone()) {
                        added_hooks.push(hook.id.clone());
                        library.hooks.push(hook);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "resolver: walk_hooks_dir failed for {}: {e}",
                    resolved_hooks_dir
                );
            }
        }
        // Stable order — same conditional-sort policy as skills +
        // rules. The descriptor cache's content hash depends on a
        // stable iteration order.
        if !added_hooks.is_empty() {
            library.hooks.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    // ── Permission presets: harvest `defineConfig.agents.permissions.presets`
    //    and seed `library.permissions_presets`.
    //
    // Slice 3 close-out (A4): the leaf no longer reads
    // `.ship/permissions.jsonc`. The named-preset layer
    // (`ship-readonly` / `ship-standard` / `ship-autonomous` /
    // `ship-elevated`) moves to
    // `defineConfig({ agents: { permissions: { presets: { ... } } } })`
    // in `ship.config.ts`. The per-module map is harvested by walking
    // `declarations.config` (root-last-wins, same as the skills/rules
    // dir overrides), converting each `PermissionPresetDeclare` into
    // a `compiler::PermissionPreset` via field-for-field clone, and
    // inserting into the `BTreeMap`-keyed library field. First-wins
    // on id collision (two extensions declaring the same preset name
    // → the first one in declared order wins, mirroring the
    // `agents.mcp` collision policy the slice-2c close-out set).
    //
    // The map is the descriptor's only on-disk authority for presets
    // post slice 3 — the leaf's `load_permission_preset` (disk read)
    // is replaced by an in-memory lookup against this map, which the
    // compile path (agent-def's `apply_agent_permissions`) consumes
    // directly. See `agent_def::compile::permissions::apply_agent_permissions`.
    for cfg in &declarations.config {
        let Some(agents) = &cfg.agents else { continue };
        let Some(perms) = &agents.permissions else {
            continue;
        };
        for (name, declared) in &perms.presets {
            // First-wins: the entry already in the map (from a
            // prior `defineConfig` call in declared order) wins.
            // `BTreeMap::entry()` gives us the same semantics in
            // one call: `or_insert_with` is no-op when the key is
            // already present.
            library
                .permissions_presets
                .entry(name.clone())
                .or_insert_with(|| compiler::PermissionPreset {
                    default_mode: declared.default_mode.clone(),
                    tools_allow: declared.tools_allow.clone(),
                    tools_deny: declared.tools_deny.clone(),
                    tools_ask: declared.tools_ask.clone(),
                    additional_directories: declared.additional_directories.clone(),
                });
        }
    }
}

/// Append `incoming` items into `existing`, skipping any whose key is
/// already taken — by an `existing` entry OR by an earlier item in
/// `incoming` itself (two declare-forms with the same id in one eval;
/// the first wins).
fn merge_by_key<T>(existing: &mut Vec<T>, incoming: Vec<T>, key: impl Fn(&T) -> &str) {
    let mut seen: HashSet<String> = existing.iter().map(|item| key(item).to_string()).collect();
    for item in incoming {
        if !seen.insert(key(&item).to_string()) {
            continue;
        }
        existing.push(item);
    }
}

/// Resolve a space's full descriptor: disk walk + TS eval + resolve.
///
/// `space_root` is the directory containing `ship.config.ts` (preferred) or
/// `<.ship>/ship.ts` (legacy), plus `.ship/` for the bulk of the config
/// shape. Returns the resolved config — a pure, in-memory view of the
/// space's effective configuration.
///
/// The bulk of the disk read is delegated to the compiler leaf's
/// `load_library`; the Phase-2 space-shape piece (`ship.config.ts`)
/// is layered on top via FsBroker.
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
/// A `ship.config.ts` composes the defaults with
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
    //    `imported_modules` from `ship.config.ts`.
    let mut library = space_loader::load_space_library(fs, space_root)
        .await
        .with_context(|| format!("resolver: space load failed for {}", space_root.display()))?;

    // 2. TS eval fan-out — every declare-form roll-up per module (Wave 3:
    //    agents, mcp servers, skills, rules, pipelines, triggers,
    //    collections). Empty module list → no V8 work. The eval crosses
    //    the `!Send` boundary inside `ts_resolver`; the resulting
    //    `Declarations` is plain data, safe to merge.
    let declarations = ts_resolver::resolve_ts_files(&library.imported_modules, &globals)
        .await
        .context("resolver: TS eval failed")?;
    merge_ts_declarations(&mut library, declarations, fs, space_root);
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
pub async fn load_space_library(space_root: &Path) -> Result<ExtensionLibrary> {
    let fs = LocalFsBroker::new();
    space_loader::load_space_library(&fs, space_root).await
}

// ─── Phase-3 roll-up ───────────────────────────────────────────────────────────

/// Compose the space's extension roll-up: the discovered extension roots
/// (one per `ship.config.ts`-bearing directory under the space root) +
/// every locked cached package, fed through [`compiler::compose_space`].
///
/// The output is a [`compiler::ComposedLibrary`] whose `library` is the
/// composed result of every extension's noun collections under the
/// precedence law. `shadows` carries one `ShadowRecord` per id
/// collision the law resolved. The composed `library.rollup_inputs`
/// is populated with the full `RollupInputs` (extensions + precedence)
/// so downstream consumers (the def index in J3, the view in J4) can
/// audit exactly what composed into the result.
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
/// library tier is tagged `ExtSource::UserGlobal`. Every discovered
/// extension root in the space (`ship.config.ts` at the space root +
/// every direct child dir carrying `ship.config.ts`) is tagged
/// `ExtSource::Space` and forms its own `ExtensionDescriptor` — the
/// multi-extension shape Wave 4 phase B introduced
/// (`job-spec-fm-w4b-space-scan.md`).
///
/// ## The permissions/hooks fence (the PIN)
///
/// Only the space-root extension's `permissions` + `hooks` survive the
/// composition — every child-dir extension's contribution is zeroed at
/// the per-root identity source (the descriptor build site below) before
/// `compose_space` runs. The fence is encoded at the source where the
/// per-root identity is known, not inferred by `compose_space` (an
/// extension must not grant itself). The `compose_space` "first Space
/// wins" rule plus the deterministic root-first ordering ensure the
/// space root's `permissions` / `hooks` are the composed base; child
/// descriptors arrive in declared order after, with their
/// `permissions` / `hooks` already zeroed.
///
/// ## Determinism
///
/// Extension roots are enumerated in a stable order — the space root
/// first, then children in lexicographic order of their dir name. The
/// `RollupInputs.extensions` list mirrors this order verbatim, so
/// compose precedence and the `ShadowRecord` list are reproducible
/// run-to-run (the cache key invariant).
pub async fn roll_up(
    space_root: &Path,
    cache_root: Option<&Path>,
    library_root: Option<&Path>,
) -> Result<ComposedLibrary> {
    let fs = LocalFsBroker::new();

    // 1. Enumerate every extension root in the space (the W4 phase B
    //    shape: space root + every direct child dir carrying
    //    `ship.config.ts`). Each becomes its own `ExtSource::Space`
    //    descriptor with stable identity (the root's name).
    let roots = space_loader::enumerate_extension_roots(space_root)
        .context("roll_up: enumerate extension roots")?;

    // 2. Per-extension TS eval: collect the union of every root's
    //    `imported_modules` (each root contributes one module — the
    //    root's `ship.config.ts`). The TS fan-out
    //    (`ts_resolver::resolve_ts_files`) iterates in declared order
    //    and merges; the merged `Declarations` lands in
    //    `source_library` below (the merged source shape that
    //    `compose_space` consumes).
    let mut per_root_descriptors: Vec<ExtensionDescriptor> = Vec::with_capacity(roots.len());
    let mut space_root_descriptor: Option<ExtensionDescriptor> = None;
    for root in &roots {
        let mut per_root = space_loader::load_root_library(&fs, &root.path)
            .await
            .with_context(|| {
                format!(
                    "roll_up: load root library for {} ({})",
                    root.path.display(),
                    root.identity
                )
            })?;
        // TS-eval THIS root's manifest and merge into the per-root
        // library. The `imported_modules` is consumed in the process —
        // the per-root descriptor carries the merged library
        // (declare-forms as if they were file-borne).
        let declarations = ts_resolver::resolve_ts_files(&per_root.imported_modules, &Value::Null)
            .await
            .with_context(|| {
                format!(
                    "roll_up: TS eval for root {} ({})",
                    root.path.display(),
                    root.identity
                )
            })?;
        merge_ts_declarations(&mut per_root, declarations, &fs, &root.path);
        // Drain the now-consumed modules so they don't double-eval
        // via the merged `source_library` below.
        per_root.imported_modules.clear();

        let per_root_content_hash = descriptor::extension_library_content_hash(&per_root)
            .with_context(|| {
                format!("roll_up: per-root content hash for {}", root.path.display())
            })?;

        if root.identity == space_loader::SPACE_ROOT_IDENTITY {
            // The space root owns permissions + hooks. The PIN:
            // this descriptor's `permissions` + `hooks` are the
            // composed base (the rest are dropped pre-merge).
            space_root_descriptor = Some(ExtensionDescriptor {
                source: ExtSource::Space,
                space_root: root.path.clone(),
                descriptor: per_root,
                content_hash: per_root_content_hash,
            });
        } else {
            // Child-dir extension: zero out permissions + hooks
            // BEFORE the descriptor is built. The fence is encoded
            // at the per-root identity source — a child that
            // misconfigures its `.ship/permissions.jsonc` cannot
            // escalate the space's effective grants.
            per_root.permissions = compiler::Permissions::default();
            per_root.hooks = Vec::new();
            per_root_descriptors.push(ExtensionDescriptor {
                source: ExtSource::Space,
                space_root: root.path.clone(),
                descriptor: per_root,
                content_hash: per_root_content_hash,
            });
        }
    }
    // The merged source library is the concatenation of every root's
    // post-TS-eval library. This is the shape
    // `load_space_library` returns for the resolver's single-library
    // API consumers (TS fan-out, canonical-form hashing, etc.); the
    // roll-up below uses the per-root descriptors to feed
    // `compose_space`.
    //
    // We rebuild this from the per-root descriptors to keep one
    // source of truth (no double-eval, no drift between the merged
    // library and the descriptor set).
    let mut source_library = ExtensionLibrary::default();
    if let Some(space_root_desc) = &space_root_descriptor {
        // Space root keeps its permissions + hooks (the PIN base);
        // every other field lands in the merged library.
        source_library = space_root_desc.descriptor.clone();
    }
    for child_desc in &per_root_descriptors {
        merge_ts_root_into(&mut source_library, &child_desc.descriptor);
    }

    // 3. The merged `ExtensionLibrary`'s content hash. Surfaced for
    //    callers that key on the single-library canonical form. The
    //    `compose_space` step below uses the per-extension hashes
    //    (the descriptor's `content_hash` field) as the audit
    //    values for `ShadowRecord`.
    let source_content_hash = descriptor::extension_library_content_hash(&source_library)
        .context("roll_up: source content hash")?;

    // 4. Load cached packages (the #122 dep loader, per-package output).
    // ship.lock lives at the STACK ROOT, beside ship.config.ts (the
    // lockfile-beside-manifest convention; ruled 2026-07-12).
    let lock_path = space_root.join("ship.lock");
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
    let bundles =
        ship_store::dep_skills::load_cached_extensions(&lock_path, Some(resolved_cache_root))
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

    // 5. Build the extension set. Cached packages come FIRST in
    //    declared order so the source extensions — also
    //    `ExtSource::Space` (rank 1000) at the same rank — win on
    //    equal-rank ties (the `compose_space` law: "later declared
    //    wins"). Every other tier has a distinct rank, so declared
    //    order is otherwise irrelevant. The Space-tier set is the
    //    space-root descriptor (if it qualifies) followed by every
    //    child-dir descriptor in their lexicographic order (the
    //    order `enumerate_extension_roots` returns them in).
    let mut extensions: Vec<ExtensionDescriptor> =
        Vec::with_capacity(bundles.len() + 2 + per_root_descriptors.len());
    for bundle in &bundles {
        extensions.push(ExtensionDescriptor {
            source: ExtSource::Cache,
            space_root: space_root.to_path_buf(),
            descriptor: ExtensionLibrary {
                agent_profiles: bundle.agent_profiles.clone(),
                skills: bundle.skills.clone(),
                // Dep views: each bundle's `views` are already prefixed with
                // `pkg_path` by `load_cached_extensions`, so they're ready for
                // `compose_views`'s namespace-prefix merge. An empty vec is
                // the common case (no view exports in this package) —
                // composing an empty collection is a no-op, not an error.
                views: bundle.views.clone(),
                ..Default::default()
            },
            content_hash: bundle.content_hash.clone(),
        });
    }
    extensions.push(ExtensionDescriptor {
        source: ExtSource::UserGlobal,
        space_root: space_root.to_path_buf(),
        descriptor: enabled_library,
        content_hash: enabled_content_hash,
    });
    if let Some(space_root_desc) = space_root_descriptor {
        extensions.push(space_root_desc);
    }
    extensions.extend(per_root_descriptors);

    // 6. Assemble RollupInputs.
    let rollup_inputs = RollupInputs {
        extensions,
        precedence: RollupInputs::default_precedence(),
    };

    // 7. Compose. Pure; the law is the only thing that mutates the
    //    library's noun collections.
    let mut composed = compiler::compose_space(&rollup_inputs).context("roll_up: compose_space")?;

    // 8. Annotate the composed library with the inputs so consumers (J3
    //    index, J4 view) can audit the roll-up. The `rollup_inputs` field
    //    is `#[serde(skip_serializing_if = "Option::is_none")]` so an
    //    empty roll-up stays out of the canonical form (Phase-2 contract).
    composed.library.rollup_inputs = Some(rollup_inputs);

    // `source_content_hash` is intentionally surfaced for callers
    // that key on the single-library canonical form; the per-extension
    // hashes are the audit values `compose_space` records on shadow.
    let _ = source_content_hash;

    Ok(composed)
}

/// Append one root's post-TS-eval `ExtensionLibrary` into the merged
/// accumulator (the single-library view `load_space_library`-shaped
/// callers consume). Permissions + hooks are intentionally NOT
/// merged here — the PIN has them sourced exclusively from the space
/// root, and the roll-up's per-root descriptors carry their own
/// scoped values. The merged library is the resolver's
/// `load_space_library` API contract shape; the roll-up feeds
/// `compose_space` from the per-root descriptors directly.
fn merge_ts_root_into(merged: &mut ExtensionLibrary, lib: &ExtensionLibrary) {
    merged.agent_profiles.extend(lib.agent_profiles.clone());
    merged.agents.extend(lib.agents.clone());
    merged.mcp_servers.extend(lib.mcp_servers.clone());
    merged.skills.extend(lib.skills.clone());
    merged.rules.extend(lib.rules.clone());
    merged.pipelines.extend(lib.pipelines.clone());
    merged.triggers.extend(lib.triggers.clone());
    merged.collections.extend(lib.collections.clone());
    merged.views.extend(lib.views.clone());
    merged.skill_paths.extend(lib.skill_paths.clone());
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
