//! `.ship/` → [`ExtensionLibrary`] loader. Leaf, substrate-free.
//!
//! This is the LEAF loader — the same surface `agent-def::compile::loader`
//! served, minus every dep on `ship-substrate` / `ship-config` / `ship-store`.
//! helmd depends on the compiler crate over this exact contract, so the leaf
//! must NOT pull substrate back in (the leaf-clean invariant `cargo tree`
//! proves; see `apps/helmd/Cargo.toml`).
//!
//! File-system surface walked:
//! - `permissions.jsonc`     → `Permissions` (the project's resolved defaults
//!                             — preset resolution is the caller's job, e.g.
//!                             the active agent's `permissions.preset` ref)
//! - `hooks.jsonc`           → `HookConfig` entries
//!
//! mcp / rules / skills / agents used to be read from this leaf. They now
//! arrive via `defineMcpServer` / `defineRule` / `defineSkill` / `defineAgent`
//! declare-forms in the space's `ship.config.ts`, harvested by the
//! Phase-2 resolver (`ship_resolver::merge_ts_declarations`). The leaf returns
//! them empty so the TS-resolver merge is the single source of truth — see
//! `census-format-migration.md` Section 4 row 3 + Wave 5.
//!
//! Missing files / dirs are silently skipped. An empty library is valid — this
//! is the existing agent-def loader contract; the leaf loader must match.
//!
// (the dep-skill cache itself is a substrate concern, surfaced by
// `ship_store::dep_skills::resolve_dep_skill` — the leaf no longer
// reads `.ship/skills/` at all, so the dep-skill seam is moot here
// post Wave 5).
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::jsonc::from_jsonc_str;
use crate::resolve::ExtensionLibrary;
use crate::types::{
    HookConfig, HookTrigger, Permissions,
};

// (`.ship/mcp.jsonc` and `.ship/rules/*.md` are no longer read by the
// leaf loader. Post format-migration Wave 4 phase A, MCP server and
// rule definitions arrive via `defineMcpServer` / `defineRule`
// declare-forms in the space's `ship.config.ts` and land in
// `ExtensionLibrary::mcp_servers` / `ExtensionLibrary::rules` through
// `ship_resolver::merge_ts_declarations` — see
// `census-format-migration.md` Section 4 row 3.)
//
// (`.ship/agents/*.jsonc` and `.ship/skills/<id>/SKILL.md` are no
// longer read by the leaf loader. Post format-migration Wave 5,
// agent and skill definitions arrive via `defineAgent` / `defineSkill`
// declare-forms in the space's `ship.config.ts` and land in
// `ExtensionLibrary::agent_profiles` / `ExtensionLibrary::skills`
// through the same resolver merge. The duplicate walk the leaf kept
// for the disk-authoring fallback dies with this wave — see
// `census-format-migration.md` Section 4 row 3 (last paragraph).)

// ─── Hooks file types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
struct HooksFile {
    #[serde(default)]
    hooks: Vec<HooksEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct HooksEntry {
    id: String,
    trigger: String,
    command: String,
    #[serde(default)]
    matcher: Option<String>,
}

// ─── Top-level entry point ────────────────────────────────────────────────────

/// Load a [`ExtensionLibrary`] from a `.ship/` directory (flat layout).
///
/// Post format-migration Wave 5: the leaf only reads permissions +
/// hooks from disk. The four `ExtensionLibrary` noun vectors that
/// the resolver composes (`mcp_servers`, `rules`, `skills`,
/// `agent_profiles`) all arrive via the space's `ship.config.ts`
/// declare-forms (`defineMcpServer`, `defineRule`, `defineSkill`,
/// `defineAgent`) and land through
/// `ship_resolver::merge_ts_declarations`. The leaf returns them
/// empty so the TS-resolver merge is the single source of truth —
/// see `census-format-migration.md` Section 4 row 3 (last
/// paragraph) + Wave 5.
///
/// Missing files and dirs are silently skipped — an empty library is
/// valid. The leaf has no dep on `ship-substrate`; everything is
/// plain `std::fs` + the crate's own `jsonc` module.
pub fn load_library(ship_dir: &Path) -> Result<ExtensionLibrary> {
    let skill_paths = read_space_skill_paths(ship_dir);
    Ok(ExtensionLibrary {
        mcp_servers: Vec::new(),
        permissions: load_permissions(ship_dir),
        hooks: load_hooks(ship_dir)?,
        rules: Vec::new(),
        skills: Vec::new(),
        agent_profiles: Vec::new(),
        skill_paths,
        ..Default::default()
    })
}

// ─── Permissions ──────────────────────────────────────────────────────────────

// ─── Permissions ──────────────────────────────────────────────────────────────

/// Load the project's permission defaults. The file is a *presets* document
/// today (`{ "ship-readonly": {...}, "ship-standard": {...}, ... }`); the
/// leaf form returns a default `Permissions` and lets the active-agent's
/// preset resolution do the rest. The file's presence is also what we
/// check — an absent file is "no project presets" rather than a hard error.
fn load_permissions(ship_dir: &Path) -> Permissions {
    let path = ship_dir.join("permissions.jsonc");
    if !path.exists() {
        return Permissions::default();
    }
    // The presets file's shape does not match `Permissions` directly — it's
    // a map of named preset sections, not a single permissions blob. The
    // project-level defaults are injected by the preset-resolution layer
    // (caller side) using the active agent's `permissions.preset` ref. So
    // the leaf loader just confirms the file exists and returns defaults;
    // preset resolution lives in `agent-def` (out of leaf scope).
    let _ = path; // presence-checked; structure intentionally not deserialized here.
    Permissions::default()
}

// ─── Hooks ────────────────────────────────────────────────────────────────────

fn load_hooks(ship_dir: &Path) -> Result<Vec<HookConfig>> {
    let path = ship_dir.join("hooks.jsonc");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    let file: HooksFile = from_jsonc_str(&text)
        .with_context(|| format!("Cannot parse {}", path.display()))?;
    Ok(file
        .hooks
        .into_iter()
        .filter_map(|e| {
            let trigger = match e.trigger.as_str() {
                "pre_tool_use" | "PreToolUse" => HookTrigger::PreToolUse,
                "post_tool_use" | "PostToolUse" => HookTrigger::PostToolUse,
                "notification" | "Notification" => HookTrigger::Notification,
                "stop" | "Stop" => HookTrigger::Stop,
                "subagent_stop" | "SubagentStop" => HookTrigger::SubagentStop,
                "pre_compact" | "PreCompact" => HookTrigger::PreCompact,
                _ => return None, // unknown trigger — skip, do not fail
            };
            Some(HookConfig {
                id: e.id,
                trigger,
                command: e.command,
                matcher: e.matcher,
                cursor_event: None,
            })
        })
        .collect())
}

// `.ship/rules/*.md` is no longer read by the leaf loader. Post format-
// migration Wave 4 phase A, rule definitions arrive via `defineRule`
// declare-forms in the space's `ship.config.ts` and land in
// `ExtensionLibrary::rules` through
// `ship_resolver::merge_ts_declarations` — see
// `census-format-migration.md` Section 4 row 3. The compile path
// consumes the resulting `Vec<Rule>` from `library` without ever
// touching the source files, so the leaf's rules walk dies with the
// cutover.

// ── skill_paths (Phase-2 additive manifest read) ────────────────────────────────

/// The default skill dir path, relative to the space root (`<root>/.ship/skills/`).
///
/// Matches agent-def's [`read_skill_paths`](
/// ship_platform/crates/agent-def/src/skill_paths.rs) so a space that lands
/// `space.skill_paths` in its `ship.jsonc` sees the same effective skill
/// discovery the agent-def consumers do. The string is space-RELATIVE — the
/// resolver joins it onto the project root when walking.
pub const DEFAULT_SPACE_SKILL_PATH: &str = ".ship/skills/";

/// Read `space.skill_paths` from `<ship_dir>/ship.jsonc` and return the
/// space-relative paths as strings.
///
/// - Always includes the default `.ship/skills/` as the first entry (preserves
///   today's behavior: an empty manifest yields exactly the default).
/// - Custom paths from `space.skill_paths` are appended IN ORDER, duplicates
///   of the default skipped.
/// - Absolute paths are rejected and skipped (no fs escape).
/// - Missing or unreadable `ship.jsonc` → `vec![DEFAULT_SPACE_SKILL_PATH]`
///   (lenient contract — a missing manifest is "no project customization").
/// - JSONC comments are tolerated (the `ship.jsonc` file uses the same
///   format the agents config uses; the leaf's own `jsonc` module strips
///   `//` / `/* */` comments + trailing commas before parse).
pub fn read_space_skill_paths(ship_dir: &Path) -> Vec<String> {
    let mut result = vec![DEFAULT_SPACE_SKILL_PATH.to_string()];

    let primary = ship_dir.join("ship.jsonc");
    let content = match fs::read_to_string(&primary) {
        Ok(c) => c,
        Err(_) => return result,
    };
    let parsed: serde_json::Value = match from_jsonc_str(&content) {
        Ok(v) => v,
        Err(_) => return result,
    };
    let paths = parsed
        .get("space")
        .and_then(|p| p.get("skill_paths"))
        .and_then(|v| v.as_array());

    if let Some(arr) = paths {
        for v in arr {
            let Some(s) = v.as_str() else { continue };
            if std::path::Path::new(s).is_absolute() {
                // Absolute paths are rejected and skipped — same posture as
                // agent-def's `read_skill_paths` (the b2f75a7b2 bug class).
                continue;
            }
            if s == DEFAULT_SPACE_SKILL_PATH {
                // Already the first entry; skip the duplicate.
                continue;
            }
            result.push(s.to_string());
        }
    }

    result
}
// `.ship/skills/<id>/SKILL.md` and `.ship/agents/<id>.jsonc` are no
// longer read by the leaf loader. Post format-migration Wave 5,
// skill + agent definitions arrive via `defineSkill` / `defineAgent`
// declare-forms in the space's `ship.config.ts` and land in
// `ExtensionLibrary::skills` / `ExtensionLibrary::agent_profiles`
// through `ship_resolver::merge_ts_declarations` — see
// `census-format-migration.md` Section 4 row 3 (last paragraph).
// The leaf's `load_skills` + `load_agent_profiles` walks die with
// this wave; the leaf has no disk-authoring fallback for these
// nouns — the TS-resolver merge is the single source of truth.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_library_on_missing_dir_yields_empty_library() {
        let tmp = tempfile::tempdir().unwrap();
        let absent = tmp.path().join("nope");
        let lib = load_library(&absent).unwrap();
        assert!(lib.agent_profiles.is_empty());
        assert!(lib.mcp_servers.is_empty());
        assert!(lib.skills.is_empty());
        assert!(lib.rules.is_empty());
        assert!(lib.hooks.is_empty());
    }

    // ── read_space_skill_paths (Phase-2 additive manifest read) ──────────────

    #[test]
    fn read_skill_paths_defaults_to_ship_skills_when_manifest_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(paths, vec![".ship/skills/".to_string()]);
    }

    #[test]
    fn read_skill_paths_defaults_when_manifest_lacks_skill_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{"id": "demo", "space": {}}"#,
        )
        .unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(paths, vec![".ship/skills/".to_string()]);
    }

    #[test]
    fn read_skill_paths_appends_custom_space_relative_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{"space": {"skill_paths": ["docs/", "extra/skills/"]}}"#,
        )
        .unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(
            paths,
            vec![
                ".ship/skills/".to_string(),
                "docs/".to_string(),
                "extra/skills/".to_string(),
            ]
        );
    }

    #[test]
    fn read_skill_paths_rejects_absolute_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{"space": {"skill_paths": ["/abs/path", "rel/path/"]}}"#,
        )
        .unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(
            paths,
            vec![
                ".ship/skills/".to_string(),
                "rel/path/".to_string(),
            ]
        );
    }

    #[test]
    fn read_skill_paths_empty_array_falls_back_to_default() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{"space": {"skill_paths": []}}"#,
        )
        .unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(paths, vec![".ship/skills/".to_string()]);
    }

    #[test]
    fn read_skill_paths_duplicates_of_default_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{"space": {"skill_paths": [".ship/skills/", "other/"]}}"#,
        )
        .unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(
            paths,
            vec![
                ".ship/skills/".to_string(),
                "other/".to_string(),
            ]
        );
    }

    #[test]
    fn read_skill_paths_tolerates_jsonc_comments() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{
                // space-level skill discovery
                "space": {
                    "skill_paths": ["docs/"] /* extra skills */,
                }
            }"#,
        )
        .unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(
            paths,
            vec![".ship/skills/".to_string(), "docs/".to_string()]
        );
    }

    #[test]
    fn read_skill_paths_unreadable_manifest_falls_back_to_default() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(ship_dir.join("ship.jsonc"), "not valid jsonc {").unwrap();
        let paths = read_space_skill_paths(&ship_dir);
        assert_eq!(paths, vec![".ship/skills/".to_string()]);
    }

    #[test]
    fn load_library_populates_skill_paths_from_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{"space": {"skill_paths": ["docs/"]}}"#,
        )
        .unwrap();
        let lib = load_library(&ship_dir).unwrap();
        assert_eq!(
            lib.skill_paths,
            vec![".ship/skills/".to_string(), "docs/".to_string()]
        );
    }

    #[test]
    fn load_library_default_skill_paths_when_manifest_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let lib = load_library(&ship_dir).unwrap();
        assert_eq!(lib.skill_paths, vec![".ship/skills/".to_string()]);
    }
}
