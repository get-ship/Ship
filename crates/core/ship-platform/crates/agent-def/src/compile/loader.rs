//! Load a [`ExtensionLibrary`] from the `.ship/` directory tree.
//! No compilation or resolution occurs here — pure filesystem loading.
//!
//! **Slice 3 close-out (A3+A4)**: the leaf no longer reads
//! `.ship/hooks.jsonc` or `.ship/permissions.jsonc` (both files died
//! with the dot-ship close-out). `permissions` + `hooks` +
//! `permissions_presets` are populated by the resolver's
//! `merge_ts_declarations` (the TS declare-form harvest + the
//! on-disk `<hooks_dir>/<event>.<ext>` walk). The leaf's
//! `load_permission_preset` is now an in-memory lookup against
//! `ExtensionLibrary::permissions_presets` — the caller side
//! ([`apply_agent_permissions`]) passes the map, not a `ship_dir`.
//! See [`super::permissions`] for the consumer.

use anyhow::Result;
use stack_model::{
    McpServerConfig, McpServerType, ExtensionLibrary, Rule,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::permissions::PermissionPreset;

// ── MCP file types ────────────────────────────────────────────────────────────

/// A single MCP server entry from `mcp.jsonc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEntry {
    pub id: String,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_type: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
}

fn default_scope() -> String {
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

#[derive(Debug, Clone, Deserialize, Default)]
struct LegacyMcpFile {
    #[serde(default)]
    servers: Vec<McpEntry>,
}

/// The `.ship/mcp.jsonc` file. Supports both the canonical keyed format
/// (`{ "mcp": { "servers": { "<id>": {...} } } }`) and the legacy flat array
/// format (`{ "servers": [...] }`).
#[derive(Debug, Clone, Default)]
pub struct McpFile {
    pub servers: Vec<McpEntry>,
}

impl McpFile {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;

        if let Ok(raw) = stack_model::jsonc::from_jsonc_str::<RawMcpFile>(&text)
            && !raw.mcp.servers.is_empty()
        {
            let servers = raw
                .mcp
                .servers
                .into_iter()
                .map(|(key, mut entry)| {
                    if entry.id.is_empty() {
                        entry.id = key;
                    }
                    entry
                })
                .collect();
            return Ok(Self { servers });
        }
        if let Ok(legacy) = stack_model::jsonc::from_jsonc_str::<LegacyMcpFile>(&text) {
            return Ok(Self {
                servers: legacy.servers,
            });
        }
        Ok(Self::default())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut servers_map = serde_json::Map::new();
        for s in &self.servers {
            let val = serde_json::to_value(s)?;
            servers_map.insert(s.id.clone(), val);
        }
        let root = serde_json::json!({ "mcp": { "servers": servers_map } });
        std::fs::write(path, serde_json::to_string_pretty(&root)?)?;
        Ok(())
    }
}

// ── Top-level entry point ─────────────────────────────────────────────────────

/// Load a [`ExtensionLibrary`] from a `.ship/` directory (flat layout).
///
/// Post format-migration Wave 5: reads mcp, permissions, hooks, rules
/// from disk. Agent profiles and skills no longer arrive from disk —
/// they land via `ship.config.ts` declare-forms (`defineAgent` /
/// `defineSkill`) through the resolver's `merge_ts_declarations` step
/// (census-format-migration.md §4 row 3, last paragraph). The leaf
/// returns the two noun vectors empty; the roll-up is the only door
/// for them. Tests that need a pre-merge library should call this fn
/// to get the disk-side bulk (mcp/permissions/hooks/rules) and merge
/// in the TS-declared agents/skills separately.
///
/// Missing files and dirs are silently skipped — an empty library is
/// valid (the lenient contract this loader has had).
pub fn load_library(ship_dir: &Path) -> Result<ExtensionLibrary> {
    Ok(ExtensionLibrary {
        mcp_servers: load_mcp_servers(ship_dir)?,
        // Slice 3 close-out: permissions, hooks, permissions_presets
        // are resolver-sourced now (the leaf's per-file disk reads
        // died with this slice). The leaf returns defaults — the
        // resolver's `merge_ts_declarations` populates them with
        // values harvested from the `defineConfig` block + the
        // on-disk hooks-dir walk.
        permissions: stack_model::Permissions::default(),
        hooks: Vec::new(),
        permissions_presets: std::collections::BTreeMap::new(),
        rules: load_rules(ship_dir)?,
        skills: Vec::new(),
        agent_profiles: Vec::new(),
        ..Default::default()
    })
}

// ── MCP servers ───────────────────────────────────────────────────────────────

fn load_mcp_servers(ship_dir: &Path) -> Result<Vec<McpServerConfig>> {
    let path = ship_dir.join("mcp.jsonc");
    let file = McpFile::load(&path)?;
    if path.exists() && file.servers.is_empty() {
        eprintln!(
            "warning: {} exists but no MCP servers were loaded. Check JSONC syntax.",
            path.display()
        );
    }
    Ok(file
        .servers
        .into_iter()
        .map(|e| {
            let server_type = match e.server_type.as_deref() {
                Some("http") => McpServerType::Http,
                Some("sse") => McpServerType::Sse,
                _ => {
                    if e.url.is_some() && e.command.is_none() {
                        McpServerType::Http
                    } else {
                        McpServerType::Stdio
                    }
                }
            };
            McpServerConfig {
                id: e.id.clone(),
                name: e.name.unwrap_or_else(|| e.id.clone()),
                command: e.command.unwrap_or_default(),
                args: e.args,
                env: e.env,
                scope: e.scope,
                server_type,
                url: e.url,
                disabled: e.disabled,
                timeout_secs: None,
                codex_enabled_tools: vec![],
                codex_disabled_tools: vec![],
                cursor_env_file: None,
            }
        })
        .collect())
}

// ── Permission preset lookup (slice 3 close-out) ───────────────────────────

/// Look up a named permission preset in the descriptor's
/// `permissions_presets` map. Replaces the pre-slice-3
/// `load_permission_preset(ship_dir, name)` that read from
/// `.ship/permissions.jsonc` — the file died with the dot-ship
/// close-out, and the preset definitions now live in
/// `defineConfig({ agents: { permissions: { presets: { ... } } } })`
/// in `ship.config.ts` (harvested by the resolver into
/// `ExtensionLibrary::permissions_presets`).
///
/// Returns `None` if the map has no entry for `name` — the
/// consumer ([`super::permissions::apply_agent_permissions`])
/// treats that as "no preset contribution from the file, fall
/// through to the built-in presets or the agent's overrides."
pub fn load_permission_preset(
    presets: &std::collections::BTreeMap<String, PermissionPreset>,
    preset_name: &str,
) -> Option<PermissionPreset> {
    presets.get(preset_name).cloned()
}

// ── Rules ─────────────────────────────────────────────────────────────────────

fn load_rules(ship_dir: &Path) -> Result<Vec<Rule>> {
    let rules_dir = ship_dir.join("rules");
    if !rules_dir.exists() {
        return Ok(vec![]);
    }
    let mut rules = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&rules_dir)?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let file_name = e.file_name().to_string_lossy().to_string();
        let raw = std::fs::read_to_string(e.path())?;
        rules.push(parse_rule(&file_name, &raw));
    }
    Ok(rules)
}

/// Parse a rule `.md` file, stripping YAML frontmatter if present.
/// Frontmatter fields: `description`, `globs` (list), `alwaysApply` (bool).
pub fn parse_rule(file_name: &str, raw: &str) -> Rule {
    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let body = &rest[end + 5..];
        let mut always_apply = true;
        let mut globs = vec![];
        let mut description = None;
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
        globs: vec![],
        description: None,
    }
}

// `.ship/agents/*.jsonc` is no longer read by this loader. Post
// format-migration Wave 5, agent profiles arrive via `defineAgent`
// declare-forms in the space's `ship.config.ts` and land in
// `ExtensionLibrary::agent_profiles` through
// `ship_resolver::merge_ts_declarations` — see
// `census-format-migration.md` §4 row 3 (last paragraph). The
// ship_config reader (`ship_config::agents::read_agent_profiles`)
// stays in place — it is the S2 READ door for the agent-def write
// surface (`AgentsService` + the `agents_write` door), not a
// `.ship/`-shape loader anymore. Disk-authoring is gone.
