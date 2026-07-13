//! `ship-resolver` — the Phase-2 space-shape resolver.
//!
//! **Tier: L1.** Orchestrates `ship_substrate` ports (FsBroker, KernelLog)
//! and `stack_model` (the descriptor types). Classified in place — lives
//! at `crates/core/ship-resolver/` rather than under `ship-platform/` for
//! cycle reasons (depends on ship-store/ship-substrate/compiler, NOT on
//! ship-platform).
//!
//! ## What this crate does

//! Generalizes the Phase-0 resolver (compiler leaf + agent-config-eval) from
//! single-manifest to **space shape**: a `stack.config.ts` at the space root, a
//! `skills/` directory convention, the on-disk permissions +
//! hooks reads. The result is a fully-populated [`stack_model::ExtensionLibrary`]
//! (disk state) and a [`stack_model::ResolvedConfig`] (resolved state) — the
//! canonical descriptors the rest of the platform compiles and runs against.
//!
//! ## The seam
//!
//! The resolver is a NEW crate that composes with — does NOT replace — the
//! compiler leaf's `load_library` (per the F1 decision, `decision-f1-loader.md`
//! Option A: "embrace the superset"). The leaf's `load_library` keeps walking
//! `.ship/{mcp,permissions,hooks,skills}/` with its `std::fs`
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
//!   [`stack_model::ComposedLibrary`]. The composed library carries
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
pub mod members;
pub mod descriptor;
pub mod space_loader;
pub mod ts_resolver;

pub use members::{
    resolve_member_roots, roll_up_members, roll_up_members_blocking, roll_up_registered,
    roll_up_registered_blocking,
};

use std::collections::HashSet;
use std::path::Path;
use agent_config_eval::Declarations;
use stack_model::ExtensionLibrary;

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
pub(crate) fn merge_ts_declarations(
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
    // `declarations.config` (root-last-wins, same as the `skills_dir` /
    // `hooks_dir` overrides below) and convert each entry via
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
    let mut mcp_from_config: Vec<stack_model::McpServerConfig> = Vec::new();
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

    // Per-extension-project on-disk skills walk (the resolution
    // layer's source for skills post slice-2b). The walk happens here
    // (not in the leaf) because the TS eval populates declared skills
    // via `defineSkill`; the disk walk duplicates the surface for
    // callers that author `SKILL.md` files directly.
    //
    // Rules no longer have an on-disk walk: the on-disk
    // `rules/<id>.md` convention died with the rules-format cut, so
    // rules arrive exclusively via `defineAgent({ rules: { inline: "..." } })`
    // (the active agent's inline rule body, the `apply_agent_to_library`
    // path in `agent-def`) — the `library.rules` vector this function
    // populates from `declarations.rules` is the only source for rules
    //
    // For skills:

    // - The leaf populates the default (`skill_paths: ["skills/"]`).
    // - `defineConfig({ skills_dir })` overrides via root-last-wins
    //   scalar.
    // - The walked records MERGE into `library.skills` with the same
    //   id-dedup policy as the TS-declared side: an entry already in
    //   the library wins.
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
        .unwrap_or_else(|| stack_model::loader::DEFAULT_SPACE_SKILL_PATH.to_string());
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
                        skill.source = stack_model::SkillSource::Custom;
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
    // ── Hooks: resolve `hooks_dir` from `defineConfig` (last wins)
    //    falling through to the unified default `"hooks/"`.
    //
    // Slice 3 close-out (A3): hooks arrive as on-disk files at
    // `<hooks_dir>/<event-stem>.<ext>`. The leaf no longer reads
    // `.ship/hooks.jsonc` — the resolver walks the per-extension-project
    // hooks dir, the same as skills. Filename → trigger mapping
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
    // in `stack.config.ts`. The per-module map is harvested by walking
    // `declarations.config` (root-last-wins, same as the skills/hooks
    // dir overrides), converting each `PermissionPresetDeclare` into
    // a `stack_model::PermissionPreset` via field-for-field clone, and
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
                .or_insert_with(|| stack_model::PermissionPreset {
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
// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
