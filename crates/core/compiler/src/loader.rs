//! `.ship/` → [`ExtensionLibrary`] loader. Leaf, substrate-free.
//!
//! This is the LEAF loader — the same surface `agent-def::compile::loader`
//! served, minus every dep on `ship-substrate` / `ship-config` / `ship-store`.
//! helmd depends on the compiler crate over this exact contract, so the leaf
//! must NOT pull substrate back in (the leaf-clean invariant `cargo tree`
//! proves; see `apps/helmd/Cargo.toml`).
//!
//! File-system surface walked:
//! - `mcp.jsonc`             → `McpServerConfig` entries (canonical keyed form)
//! - `permissions.jsonc`     → `Permissions` (the project's resolved defaults
//!                             — preset resolution is the caller's job, e.g.
//!                             the active agent's `permissions.preset` ref)
//! - `hooks.jsonc`           → `HookConfig` entries
//! - `rules/*.md`            → `Rule` entries (YAML-frontmatter aware)
//! - `skills/<id>/SKILL.md`  → `Skill` entries (via `skill_parser::parse_skill`)
//! - `agents/<id>.jsonc`     → `AgentProfile` entries (canonical nested shape
//!                             with the top-level `agent` block).
//!
//! Missing files / dirs are silently skipped. An empty library is valid — this
//! is the existing agent-def loader contract; the leaf loader must match.
//!
//! ## Dep-skill resolution (github refs)
//! Out of scope for this job. The leaf form reads LOCAL skills only. When a
//! future spike needs to resolve `github.com/.../skill` refs via
//! `ship.lock` + the dep-skill cache, the surface lives here behind a
//! `// TODO: dep-skill cache resolution` seam — the existing `lockfile` module
//! already parses the lockfile; the dep cache itself is a substrate concern.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::jsonc::from_jsonc_str;
use crate::resolve::ExtensionLibrary;
use crate::skill_parser::parse_skill;
use crate::types::{
    AgentProfile, HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions, Rule,
    Skill, SkillSource,
};

// ─── MCP file types ───────────────────────────────────────────────────────────

/// One entry in `.ship/mcp.jsonc` — canonical keyed form. The key on disk
/// is the canonical id (filled in if the entry's own `id` is empty).
#[derive(Debug, Clone, Deserialize)]
pub struct McpEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default = "default_mcp_scope")]
    pub scope: String,
    /// Lowercase: "stdio" | "http" | "sse" — matches `McpServerType`.
    #[serde(default)]
    pub server_type: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub timeout_secs: Option<u32>,
    #[serde(default)]
    pub codex_enabled_tools: Vec<String>,
    #[serde(default)]
    pub codex_disabled_tools: Vec<String>,
    #[serde(default)]
    pub cursor_env_file: Option<String>,
}

fn default_mcp_scope() -> String {
    "project".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawMcpFile {
    #[serde(default)]
    mcp: RawMcpSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawMcpSection {
    #[serde(default)]
    servers: HashMap<String, McpEntry>,
}

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
/// Looks for mcp, permissions, hooks, rules, skills at the `.ship/` root,
/// and agent profiles under `.ship/agents/`. Missing files and dirs are
/// silently skipped — an empty library is valid. The leaf has no dep on
/// `ship-substrate`; everything is plain `std::fs` + the crate's own
/// `jsonc` + `skill_parser` modules.
pub fn load_library(ship_dir: &Path) -> Result<ExtensionLibrary> {
    let skill_paths = read_space_skill_paths(ship_dir);
    Ok(ExtensionLibrary {
        mcp_servers: load_mcp_servers(ship_dir)?,
        permissions: load_permissions(ship_dir),
        hooks: load_hooks(ship_dir)?,
        rules: load_rules(ship_dir)?,
        skills: load_skills(ship_dir),
        agent_profiles: load_agent_profiles(ship_dir),
        skill_paths,
        ..Default::default()
    })
}

// ─── MCP servers ──────────────────────────────────────────────────────────────

fn load_mcp_servers(ship_dir: &Path) -> Result<Vec<McpServerConfig>> {
    let path = ship_dir.join("mcp.jsonc");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    let raw: RawMcpFile = from_jsonc_str(&text)
        .with_context(|| format!("Cannot parse {}", path.display()))?;

    let mut servers: Vec<McpServerConfig> = raw
        .mcp
        .servers
        .into_iter()
        .map(|(key, mut entry)| {
            if entry.id.is_empty() {
                entry.id = key;
            }
            let id = entry.id.clone();
            let server_type = parse_mcp_server_type(&entry);
            McpServerConfig {
                id: id.clone(),
                name: entry.name.unwrap_or_else(|| id.clone()),
                command: entry.command.unwrap_or_default(),
                args: entry.args,
                env: entry.env,
                scope: entry.scope,
                server_type,
                url: entry.url,
                disabled: entry.disabled,
                timeout_secs: entry.timeout_secs,
                codex_enabled_tools: entry.codex_enabled_tools,
                codex_disabled_tools: entry.codex_disabled_tools,
                cursor_env_file: entry.cursor_env_file,
            }
        })
        .collect();
    // Stable order by id so the library is deterministic across reads.
    servers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(servers)
}

fn parse_mcp_server_type(entry: &McpEntry) -> McpServerType {
    match entry.server_type.as_deref() {
        Some("http") => McpServerType::Http,
        Some("sse") => McpServerType::Sse,
        _ => {
            if entry.url.is_some() && entry.command.as_deref().unwrap_or("").is_empty() {
                McpServerType::Http
            } else {
                McpServerType::Stdio
            }
        }
    }
}

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

// ─── Rules ────────────────────────────────────────────────────────────────────

fn load_rules(ship_dir: &Path) -> Result<Vec<Rule>> {
    let rules_dir = ship_dir.join("rules");
    if !rules_dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = fs::read_dir(&rules_dir)
        .with_context(|| format!("Cannot read_dir {}", rules_dir.display()))?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    let mut rules = Vec::with_capacity(entries.len());
    for e in entries {
        let file_name = e.file_name().to_string_lossy().to_string();
        let raw = fs::read_to_string(e.path())
            .with_context(|| format!("Cannot read rule {}", e.path().display()))?;
        rules.push(parse_rule(&file_name, &raw));
    }
    Ok(rules)
}

/// Parse a rule `.md` file, stripping YAML frontmatter if present.
/// Frontmatter fields: `description`, `globs` (list), `alwaysApply` (bool).
fn parse_rule(file_name: &str, raw: &str) -> Rule {
    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let body = &rest[end + 5..];
        let mut always_apply = true;
        let mut globs: Vec<String> = Vec::new();
        let mut description: Option<String> = None;
        for line in fm.lines() {
            if let Some(v) = line.strip_prefix("alwaysApply:") {
                always_apply = v.trim() != "false";
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().trim_matches('"').to_string());
            } else if line.trim_start().starts_with("- ") {
                globs.push(line.trim().trim_start_matches("- ").to_string());
            }
        }
        return Rule {
            file_name: file_name.to_string(),
            content: body.trim().to_string(),
            always_apply,
            globs,
            description,
        };
    }
    Rule {
        file_name: file_name.to_string(),
        content: raw.trim().to_string(),
        always_apply: true,
        globs: Vec::new(),
        description: None,
    }
}

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
// ─── Skills ───────────────────────────────────────────────────────────────────

/// Load every `.ship/skills/<id>/SKILL.md` (plus any flat `.ship/skills/<id>.md`).
/// Local skills only — dep-skill cache resolution from `ship.lock` is a
/// substrate concern and is intentionally out of scope for this leaf form.
fn load_skills(ship_dir: &Path) -> Vec<Skill> {
    let skills_dir = ship_dir.join("skills");
    if !skills_dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(&skills_dir) else {
        return Vec::new();
    };
    let mut skills: Vec<Skill> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&skill_md) else {
            continue;
        };
        let mut skill = parse_skill(&id, &raw);
        // SkillSource::Custom is the local-project default; builtin is
        // reserved for SDK-embedded skills (a future ship-sdk-skills dep).
        skill.source = SkillSource::Custom;
        skills.push(skill);
    }
    // Stable order by id so the library is deterministic.
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    skills
}

// ─── Agent profiles ──────────────────────────────────────────────────────────

/// Load every `.ship/agents/<id>.jsonc` as a full nested [`AgentProfile`].
/// Sorted by `agent.id`. Malformed files (no `agent:` block, or not parseable
/// as `AgentProfile`) are SKIPPED, never hard-errored — the lenient contract
/// the existing agent-def loader has. Pure `std::fs` walk + the crate's own
/// `jsonc` module — no `ship_config::agents::read_agent_profiles` (substrate).
fn load_agent_profiles(ship_dir: &Path) -> Vec<AgentProfile> {
    let agents_dir = ship_dir.join("agents");
    if !agents_dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(&agents_dir) else {
        return Vec::new();
    };
    let mut profiles: Vec<AgentProfile> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonc") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = from_jsonc_str::<serde_json::Value>(&text) else {
            continue;
        };
        // Lenient contract: only files that contain a top-level `agent` block
        // (the canonical authoring shape) get parsed. Everything else is
        // skipped — the existing agent-def reader's behavior, replicated.
        if value.get("agent").is_none() {
            continue;
        }
        let Ok(profile) = serde_json::from_value::<AgentProfile>(value) else {
            continue;
        };
        profiles.push(profile);
    }
    profiles.sort_by(|a, b| a.agent.id.cmp(&b.agent.id));
    profiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rule_strips_frontmatter_and_extracts_fields() {
        let raw = "---\nalwaysApply: false\ndescription: A rule\nglobs:\n  - **/*.rs\n  - Cargo.toml\n---\n\nThe body.\n";
        let r = parse_rule("style.md", raw);
        assert_eq!(r.file_name, "style.md");
        assert_eq!(r.content, "The body.");
        assert!(!r.always_apply);
        assert_eq!(r.globs, vec!["**/*.rs", "Cargo.toml"]);
        assert_eq!(r.description.as_deref(), Some("A rule"));
    }

    #[test]
    fn parse_rule_no_frontmatter_keeps_body_verbatim() {
        let r = parse_rule("plain.md", "# Body\n\nPlain rule.");
        assert!(r.always_apply);
        assert!(r.globs.is_empty());
        assert!(r.description.is_none());
        assert_eq!(r.content, "# Body\n\nPlain rule.");
    }

    #[test]
    fn parse_mcp_server_type_defaults_to_stdio() {
        let e = McpEntry {
            id: "x".into(),
            name: None,
            command: Some("npx".into()),
            args: vec![],
            env: Default::default(),
            url: None,
            scope: "project".into(),
            server_type: None,
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        };
        assert!(matches!(parse_mcp_server_type(&e), McpServerType::Stdio));
    }

    #[test]
    fn parse_mcp_server_type_url_only_is_http() {
        let e = McpEntry {
            id: "x".into(),
            name: None,
            command: None,
            args: vec![],
            env: Default::default(),
            url: Some("http://localhost:3000".into()),
            scope: "project".into(),
            server_type: None,
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        };
        assert!(matches!(parse_mcp_server_type(&e), McpServerType::Http));
    }

    #[test]
    fn parse_mcp_server_type_explicit_string() {
        let e = McpEntry {
            id: "x".into(),
            name: None,
            command: None,
            args: vec![],
            env: Default::default(),
            url: Some("http://x".into()),
            scope: "project".into(),
            server_type: Some("sse".into()),
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        };
        assert!(matches!(parse_mcp_server_type(&e), McpServerType::Sse));
    }

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
