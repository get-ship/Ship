//! `.ship/` → [`ExtensionLibrary`] loader. Leaf, substrate-free.
//!
//! This is the LEAF loader — the same surface `agent-def::compile::loader`
//! served, minus every dep on `ship-substrate` / `ship-settings` / `ship-store`.
//! helmd depends on the compiler crate over this exact contract, so the leaf
//! must NOT pull substrate back in (the leaf-clean invariant `cargo tree`
//! proves; see `apps/helmd/Cargo.toml`).
//!
//! **Slice 3 close-out (A3+A4)**: the leaf no longer reads
//! `.ship/hooks.jsonc` or `.ship/permissions.jsonc`. Both files died with
//! the dot-ship close-out — hooks are on-disk files at
//! `<hooks_dir>/<event>.<ext>` (walked by the resolver), and named permission
//! presets are authored under
//! `defineConfig({ agents: { permissions: { presets: { ... } } } })` in
//! `stack.config.ts` (harvested by the resolver's `merge_ts_declarations`).
//! The leaf returns `permissions` + `hooks` + `permissions_presets` empty so
//! the resolver is the single source of truth for every noun post slice 3.
//!
//! mcp / rules / skills / agents used to be read from this leaf. They now
//! arrive via `defineMcpServer` / `defineRule` / `defineSkill` / `defineAgent`
//! declare-forms in the space's `stack.config.ts`, harvested by the
//! Phase-2 resolver (`space_resolver::merge_ts_declarations`). The leaf returns
//! them empty so the TS-resolver merge is the single source of truth — see
//! `census-format-migration.md` Section 4 row 3 + Wave 5.
//!
//! Missing files / dirs are silently skipped. An empty library is valid — this
//! is the existing agent-def loader contract; the leaf loader must match.
//!
//! (the dep-skill cache itself is a substrate concern, surfaced by
//! `ship_store::dep_skills::resolve_dep_skill` — the leaf no longer
//! reads `.ship/skills/` at all, so the dep-skill seam is moot here
//! post Wave 5).
use std::path::Path;

use anyhow::Result;

use crate::resolve::ExtensionLibrary;

// (`.ship/mcp.jsonc` and `.ship/rules/*.md` are no longer read by the
// leaf loader. Post format-migration Wave 4 phase A, MCP server and
// rule definitions arrive via `defineMcpServer` / `defineRule`
// declare-forms in the space's `stack.config.ts` and land in
// `ExtensionLibrary::mcp_servers` / `ExtensionLibrary::rules` through
// `space_resolver::merge_ts_declarations` — see
// `census-format-migration.md` Section 4 row 3.)
//
// (`.ship/agents/*.jsonc` and `.ship/skills/<id>/SKILL.md` are no
// longer read by the leaf loader. Post format-migration Wave 5,
// agent and skill definitions arrive via `defineAgent` / `defineSkill`
// declare-forms in the space's `stack.config.ts` and land in
// `ExtensionLibrary::agent_profiles` / `ExtensionLibrary::skills`
// through the same resolver merge. The duplicate walk the leaf kept
// for the disk-authoring fallback dies with this wave — see
// `census-format-migration.md` Section 4 row 3 (last paragraph).)
//
// (`.ship/hooks.jsonc` and `.ship/permissions.jsonc` are no longer read
// by the leaf loader. Post dot-ship close-out slice 3 (A3+A4, 2026-07-11),
// hooks arrive via the per-extension-project `<hooks_dir>/<event>.<ext>`
// convention walked by the resolver, and named permission presets arrive
// via `defineConfig({ agents: { permissions: { presets: { ... } } } })`
// in `stack.config.ts` and land in
// `ExtensionLibrary::permissions_presets` through
// `space_resolver::merge_ts_declarations`. The leaf's `load_hooks` +
// `load_permissions` (presence-only stub) die with this slice — see
// `killmap-dot-ship-closeout.md` rows A3 + A4.)

// ─── Top-level entry point ────────────────────────────────────────────────────

/// Load a [`ExtensionLibrary`] from a `.ship/` directory (flat layout).
///
/// Post dot-ship close-out slice 3: the leaf is a no-op. Every noun the
/// resolver composes (mcp_servers, rules, skills, agent_profiles, hooks,
/// permissions, permissions_presets) arrives via the space's
/// `stack.config.ts` declare-forms (`defineMcpServer`, `defineRule`,
/// `defineSkill`, `defineAgent`, `defineConfig({ agents: { mcp, permissions } })`)
/// and lands through `space_resolver::merge_ts_declarations`. The leaf
/// returns the disk-side bulk empty (only `skill_paths` keeps its
/// post-W5 single default entry — the metadata the resolver's
/// `walk_skills_dir` reads when no `defineConfig.skills_dir` override
/// is present). The TS-resolver merge is the single source of truth for
/// every noun — see `census-format-migration.md` Section 4 row 3
/// (last paragraph) + Wave 5 + slice 3 close-out.
///
/// Missing files and dirs are silently skipped — an empty library is
/// valid. The leaf has no dep on `ship-substrate`; this implementation
/// has no `std::fs` calls at all.
pub fn load_library(ship_dir: &Path) -> Result<ExtensionLibrary> {
    // Slice 3 close-out: the leaf is a no-op for permissions + hooks +
    // permissions_presets. Everything is sourced from the resolver
    // (TS declare-form harvest + hooks-dir walk). `ship_dir` stays in
    // the signature so the leaf's contract ("load from this .ship/")
    // is preserved at the call site; callers that need the disk
    // surface for these nouns must go through `space_resolver`.
    let _ = ship_dir;
    Ok(ExtensionLibrary {
        mcp_servers: Vec::new(),
        rules: Vec::new(),
        skills: Vec::new(),
        agent_profiles: Vec::new(),
        skill_paths: vec![DEFAULT_SPACE_SKILL_PATH.to_string()],
        ..Default::default()
    })
}

// `.ship/rules/*.md` is no longer read by the leaf loader. Post format-
// migration Wave 4 phase A, rule definitions arrive via `defineRule`
// declare-forms in the space's `stack.config.ts` and land in
// `ExtensionLibrary::rules` through
// `space_resolver::merge_ts_declarations` — see
// `census-format-migration.md` Section 4 row 3. The compile path
// consumes the resulting `Vec<Rule>` from `library` without ever
// touching the source files, so the leaf's rules walk dies with the
// cutover.

// ── skill_paths ──────────────────────────────────────────────────────────────

/// The default skill dir path, relative to the space root (`<root>/skills/`).
///
/// The skill dir sits at the space root, NOT under `.ship/`. Authoring is
/// exclusively via `defineSkill` declare-forms in `stack.config.ts` (Wave 5);
/// the on-disk convention exists for compile-pipeline skill-VAR resolution
/// (`agent_def::skill_vars`) and the skills view (`ship_agents::view_state`).
/// agent-def's [`read_skill_paths`](ship_platform/crates/agent-def/src/skill_paths.rs)
/// imports this constant as the single source of truth.
pub const DEFAULT_SPACE_SKILL_PATH: &str = "skills/";
// `.ship/skills/<id>/SKILL.md` and `.ship/agents/<id>.jsonc` are no
// longer read by the leaf loader. Post format-migration Wave 5,
// skill + agent definitions arrive via `defineSkill` / `defineAgent`
// declare-forms in the space's `stack.config.ts` and land in
// `ExtensionLibrary::skills` / `ExtensionLibrary::agent_profiles`
// through `space_resolver::merge_ts_declarations` — see
// `census-format-migration.md` Section 4 row 3 (last paragraph).
// The leaf's `load_skills` + `load_agent_profiles` walks die with
// this wave; the leaf has no disk-authoring fallback for these
// nouns — the TS-resolver merge is the single source of truth.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_library_returns_empty_noun_vectors_post_wave5() {
        // No `.ship/` fixture: the leaf no longer reads any noun from disk
        // — every compose-time noun vector (mcp_servers, rules, skills,
        // agent_profiles, hooks, permissions, permissions_presets) arrives
        // via `stack.config.ts` declare-forms or the on-disk hooks-dir walk
        // in the resolver. A `.ship/` with nothing in it is the common
        // post-slice-3 case: the resolver still produces a non-empty
        // library when `stack.config.ts` carries agents + mcp + permissions.
        let tmp = tempfile::tempdir().expect("tempdir");
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let lib: ExtensionLibrary =
            load_library(&ship_dir).expect("load_library on a fresh .ship/ must succeed");

        assert!(
            lib.agent_profiles.is_empty(),
            "agent_profiles must be empty — the leaf no longer reads .ship/agents/*.jsonc (Wave 5)"
        );
        assert!(
            lib.mcp_servers.is_empty(),
            "mcp_servers must be empty — the leaf no longer reads .ship/mcp.jsonc (Wave 4)"
        );
        assert!(
            lib.rules.is_empty(),
            "rules must be empty — the leaf no longer reads .ship/rules/*.md (Wave 4)"
        );
        assert!(
            lib.skills.is_empty(),
            "skills must be empty — the leaf no longer reads .ship/skills/<id>/SKILL.md (Wave 5)"
        );
        assert!(
            lib.hooks.is_empty(),
            "hooks must be empty — the leaf no longer reads .ship/hooks.jsonc (slice 3)"
        );
        assert!(
            lib.permissions_presets.is_empty(),
            "permissions_presets must be empty — the leaf no longer reads .ship/permissions.jsonc (slice 3)"
        );
    }

    #[test]
    fn load_library_on_missing_dir_yields_empty_library() {
        // A non-existent .ship/ must NOT be a hard error — the existing
        // agent-def loader contract is "missing files are silently skipped;
        // an empty library is valid". The leaf loader must match.
        let tmp = tempfile::tempdir().expect("tempdir");
        let absent = tmp.path().join("does-not-exist");
        let lib = load_library(&absent).expect("load_library on a missing dir must succeed");
        assert!(lib.agent_profiles.is_empty());
        assert!(lib.mcp_servers.is_empty());
        assert!(lib.skills.is_empty());
        assert!(lib.rules.is_empty());
        assert!(lib.hooks.is_empty());
        assert!(lib.permissions_presets.is_empty());
    }
}
