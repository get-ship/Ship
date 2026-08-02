//! ARC A5 — JSONC / on-disk loaders for agent nouns.
//!
//! Agents, MCP, rules, and skills no longer arrive via `defineAgent` /
//! `defineConfig.agents.mcp` TS harvest. Plain-dir convention (aligned with
//! fleet `fleet.jsonc` defaults):
//!
//! - `agents/<id>.jsonc` → [`AgentProfile`]
//! - `mcp.jsonc` → MCP pool (`mcp.servers` map)
//! - `rules/<id>.md` (or `.jsonc`) → [`Rule`]
//! - `skills/<id>/SKILL.md` → still walked in [`super::merge_ts_declarations`]
//!
//! When `fleet.jsonc` is present at `space_root`, agent nouns are loaded via
//! **fleet `discover` → `rollup`** (include graph, fail-closed) instead of
//! plain-dir-only. That closes the dual-authority hole with bridge compile.
//!
//! Code nouns (pipelines / triggers / collections / views) stay on the TS
//! declare-form path — this module does not touch them.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use agent_schema::{AgentProfile, McpServerConfig, McpServerType, Rule};
use anyhow::{Context, Result};
use serde::Deserialize;
use ship_substrate::ports::{FsBroker, FsCap, FsError};
use stack_model::ExtensionLibrary;

/// Load agent nouns from plain dirs under `space_root` and merge into `library`.
///
/// When `space_root/fleet.jsonc` exists, uses `agent_schema::discover` +
/// `rollup` (include graph) and injects agents / MCP / rules / skills —
/// **fail-closed** on discover/rollup errors.
///
/// Otherwise: first-wins on id collision with anything already in `library`
/// (same policy as the former TS merge). Stable sort by id after contributing
/// new entries. Missing dirs/files still yield empty contributions (`Ok`).
pub(crate) fn merge_jsonc_agent_nouns(
    library: &mut ExtensionLibrary,
    fs: &dyn FsBroker,
    cap: &FsCap,
    space_root: &Path,
) -> Result<()> {
    // Fleet include graph SSOT when fleet.jsonc is present (bridge parity).
    if space_root.join("fleet.jsonc").is_file() {
        return merge_from_fleet_graph(library, space_root);
    }

    let agents = load_agents_jsonc(fs, cap).with_context(|| {
        format!(
            "agents/*.jsonc load failed under {} (fail-closed)",
            space_root.display()
        )
    })?;
    merge_agents(library, agents);

    let servers = load_mcp_jsonc(fs, cap).with_context(|| {
        format!(
            "mcp.jsonc load failed under {} (fail-closed)",
            space_root.display()
        )
    })?;
    {
        let mut seen: HashSet<String> = library.mcp_servers.iter().map(|m| m.id.clone()).collect();
        let mut added = 0usize;
        for s in servers {
            if seen.insert(s.id.clone()) {
                library.mcp_servers.push(s);
                added += 1;
            }
        }
        if added > 0 {
            library.mcp_servers.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    let rules = load_rules_dir(fs, cap).with_context(|| {
        format!(
            "rules/ load failed under {} (fail-closed)",
            space_root.display()
        )
    })?;
    {
        let mut seen: HashSet<String> =
            library.rules.iter().map(|r| r.file_name.clone()).collect();
        let mut added = 0usize;
        for r in rules {
            if seen.insert(r.file_name.clone()) {
                library.rules.push(r);
                added += 1;
            }
        }
        if added > 0 {
            library.rules.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        }
    }

    Ok(())
}

/// Discover → rollup the fleet include graph and inject agent nouns.
///
/// Fail-closed: any discover/rollup error propagates (cycle, collision, loosen).
fn merge_from_fleet_graph(library: &mut ExtensionLibrary, space_root: &Path) -> Result<()> {
    let disc = agent_schema::discover(space_root).with_context(|| {
        format!(
            "fleet-config discover failed under {} (fail-closed)",
            space_root.display()
        )
    })?;
    let graph = agent_schema::rollup(&disc).with_context(|| {
        format!(
            "fleet-config rollup failed under {} (fail-closed)",
            space_root.display()
        )
    })?;

    // Agents (visibility already filtered by rollup).
    let agents: Vec<AgentProfile> = graph
        .agents
        .iter()
        .map(|a| a.profile.clone())
        .collect();
    merge_agents(library, agents);

    // MCP pool (stable id order from BTreeMap).
    {
        let mut seen: HashSet<String> = library.mcp_servers.iter().map(|m| m.id.clone()).collect();
        let mut added = 0usize;
        for (id, pooled) in &graph.mcp_servers {
            if seen.insert(id.clone()) {
                library.mcp_servers.push(pooled.server.clone());
                added += 1;
            }
        }
        if added > 0 {
            library.mcp_servers.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    // Rules.
    {
        let mut seen: HashSet<String> =
            library.rules.iter().map(|r| r.file_name.clone()).collect();
        let mut added = 0usize;
        for pooled in &graph.rules {
            if seen.insert(pooled.rule.file_name.clone()) {
                library.rules.push(pooled.rule.clone());
                added += 1;
            }
        }
        if added > 0 {
            library.rules.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        }
    }

    // Skills (also walked later from skills/; first-wins keeps graph entries).
    {
        let mut seen: HashSet<String> = library.skills.iter().map(|s| s.id.clone()).collect();
        let mut added = 0usize;
        for (id, pooled) in &graph.skills {
            if seen.insert(id.clone()) {
                library.skills.push(pooled.skill.clone());
                added += 1;
            }
        }
        if added > 0 {
            library.skills.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    Ok(())
}

fn merge_agents(library: &mut ExtensionLibrary, agents: Vec<AgentProfile>) {
    let mut seen: HashSet<String> = library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.clone())
        .collect();
    let mut added = 0usize;
    for profile in agents {
        if !seen.insert(profile.agent.id.clone()) {
            continue;
        }
        // Project inline rule body into the library rule pool (same
        // convention the retired defineAgent harvest used).
        if let Some(inline) = profile.rules.inline.as_deref() {
            let trimmed = inline.trim();
            if !trimmed.is_empty() {
                let file_name = format!("{}.md", profile.agent.id);
                if !library.rules.iter().any(|r| r.file_name == file_name) {
                    library.rules.push(Rule {
                        file_name,
                        content: trimmed.to_string(),
                        always_apply: true,
                        globs: vec![],
                        description: None,
                    });
                }
            }
        }
        library.agent_profiles.push(profile.clone());
        library.agents.push(profile);
        added += 1;
    }
    if added > 0 {
        library
            .agent_profiles
            .sort_by(|a, b| a.agent.id.cmp(&b.agent.id));
        library.agents.sort_by(|a, b| a.agent.id.cmp(&b.agent.id));
    }
}

fn load_agents_jsonc(fs: &dyn FsBroker, cap: &FsCap) -> Result<Vec<AgentProfile>> {
    let entries = match fs.list_dir(cap, "agents") {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".jsonc") && !name.ends_with(".json") {
            continue;
        }
        let rel = format!("agents/{name}");
        let raw = match fs.read(cap, &rel) {
            Ok(b) => b,
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        };
        let text = String::from_utf8_lossy(&raw);
        let json = agent_schema::jsonc::strip_jsonc_comments(&text);
        let profile: AgentProfile = serde_json::from_str(&json)
            .with_context(|| format!("parse agent doc {rel}"))?;
        out.push(profile);
    }
    out.sort_by(|a, b| a.agent.id.cmp(&b.agent.id));
    Ok(out)
}

/// Authoring shape for `mcp.jsonc` (fleet A3 / ship-meta SSOT).
#[derive(Debug, Deserialize)]
struct McpFileDoc {
    #[serde(default)]
    mcp: McpFileSection,
}

#[derive(Debug, Default, Deserialize)]
struct McpFileSection {
    #[serde(default)]
    servers: BTreeMap<String, McpServerDoc>,
}

/// One server entry — looser than [`McpServerConfig`] so HTTP entries may omit
/// `command` / use empty string, matching fleet schema defaults.
#[derive(Debug, Deserialize)]
struct McpServerDoc {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    server_type: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    timeout_secs: Option<u32>,
}

impl McpServerDoc {
    fn into_config(self, key: &str) -> McpServerConfig {
        let id = self
            .id
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| key.to_string());
        let name = self
            .name
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| id.clone());
        let server_type = match self.server_type.as_deref().unwrap_or("stdio") {
            "http" => McpServerType::Http,
            "sse" => McpServerType::Sse,
            _ => McpServerType::Stdio,
        };
        McpServerConfig {
            id,
            name,
            command: self.command.unwrap_or_default(),
            args: self.args,
            env: self.env,
            scope: self.scope.unwrap_or_else(|| "project".to_string()),
            server_type,
            url: self.url,
            disabled: self.disabled,
            timeout_secs: self.timeout_secs,
            codex_enabled_tools: Vec::new(),
            codex_disabled_tools: Vec::new(),
            cursor_env_file: None,
        }
    }
}

fn load_mcp_jsonc(fs: &dyn FsBroker, cap: &FsCap) -> Result<Vec<McpServerConfig>> {
    let raw = match fs.read(cap, "mcp.jsonc") {
        Ok(b) => b,
        Err(FsError::NotFound) => {
            // Also accept mcp.json
            match fs.read(cap, "mcp.json") {
                Ok(b) => b,
                Err(FsError::NotFound) => return Ok(Vec::new()),
                Err(e) => return Err(e.into()),
            }
        }
        Err(e) => return Err(e.into()),
    };
    let text = String::from_utf8_lossy(&raw);
    let json = agent_schema::jsonc::strip_jsonc_comments(&text);
    let doc: McpFileDoc =
        serde_json::from_str(&json).context("parse mcp.jsonc")?;
    let mut servers: Vec<McpServerConfig> = doc
        .mcp
        .servers
        .into_iter()
        .map(|(k, v)| v.into_config(&k))
        .collect();
    servers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(servers)
}

fn load_rules_dir(fs: &dyn FsBroker, cap: &FsCap) -> Result<Vec<Rule>> {
    let entries = match fs.list_dir(cap, "rules") {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let rel = format!("rules/{name}");
        if name.ends_with(".jsonc") || name.ends_with(".json") {
            let raw = match fs.read(cap, &rel) {
                Ok(b) => b,
                Err(FsError::NotFound) => continue,
                Err(e) => return Err(e.into()),
            };
            let text = String::from_utf8_lossy(&raw);
            let json = agent_schema::jsonc::strip_jsonc_comments(&text);
            let rule: Rule = serde_json::from_str(&json)
                .with_context(|| format!("parse rule doc {rel}"))?;
            out.push(rule);
            continue;
        }
        if !name.ends_with(".md") {
            continue;
        }
        let raw = match fs.read(cap, &rel) {
            Ok(b) => b,
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        };
        let text = String::from_utf8_lossy(&raw);
        out.push(rule_from_markdown(name, &text));
    }
    out.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(out)
}

/// Parse a rules/*.md file with optional YAML frontmatter (`alwaysApply`,
/// `globs`, `description`). Body is the markdown after the frontmatter fence.
fn rule_from_markdown(file_name: &str, raw: &str) -> Rule {
    let mut always_apply = true;
    let mut globs = Vec::new();
    let mut description = None;
    let content;

    let trimmed = raw.trim_start();
    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            let fm = &rest[..end];
            let body = rest[end + 4..].trim_start_matches('\n');
            for line in fm.lines() {
                let line = line.trim();
                if let Some(v) = line.strip_prefix("alwaysApply:") {
                    let v = v.trim();
                    always_apply = v == "true" || v == "yes";
                } else if let Some(v) = line.strip_prefix("description:") {
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if !v.is_empty() {
                        description = Some(v.to_string());
                    }
                } else if let Some(v) = line.strip_prefix("globs:") {
                    let v = v.trim();
                    if v.starts_with('[') {
                        // simple comma-split inside brackets
                        let inner = v.trim_start_matches('[').trim_end_matches(']');
                        globs = inner
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    } else if !v.is_empty() {
                        globs.push(v.trim_matches('"').trim_matches('\'').to_string());
                    }
                }
            }
            content = body.to_string();
        } else {
            content = raw.to_string();
        }
    } else {
        content = raw.to_string();
    }

    Rule {
        file_name: file_name.to_string(),
        content,
        always_apply,
        globs,
        description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::ports::{FsMode, FsRoot, LocalFsBroker, SpaceId};
    use std::fs;
    use tempfile::TempDir;

    fn cap_for(root: &Path) -> FsCap {
        FsCap::new_unchecked(
            root.to_path_buf(),
            FsRoot::Space(SpaceId::new("resolver")),
            FsMode::ReadOnly,
        )
    }

    #[test]
    fn loads_agents_mcp_rules_from_plain_dirs() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("agents")).unwrap();
        fs::create_dir_all(root.join("rules")).unwrap();
        fs::write(
            root.join("agents/demo.jsonc"),
            r#"{
  // agent
  "agent": { "id": "demo", "name": "Demo" },
  "skills": { "refs": ["tdd"] },
  "mcp": { "servers": ["ship"] },
  "rules": { "inline": "Be brief." },
}"#,
        )
        .unwrap();
        fs::write(
            root.join("mcp.jsonc"),
            r#"{
  "mcp": {
    "servers": {
      "ship": {
        "name": "Ship",
        "command": "ship",
        "args": ["mcp", "serve"],
        "server_type": "stdio",
        "scope": "project",
      },
      "helm": {
        "name": "helm",
        "command": "",
        "server_type": "http",
        "url": "http://localhost:3000/mcp",
        "scope": "project",
      },
    },
  },
}"#,
        )
        .unwrap();
        fs::write(
            root.join("rules/discipline.md"),
            "---\nalwaysApply: true\n---\n\n# Discipline\n\nNo placeholders.\n",
        )
        .unwrap();

        let fs_broker = LocalFsBroker::new();
        let cap = cap_for(root);
        let mut library = ExtensionLibrary::default();
        merge_jsonc_agent_nouns(&mut library, &fs_broker, &cap, root).expect("load ok");

        assert_eq!(library.agent_profiles.len(), 1);
        assert_eq!(library.agent_profiles[0].agent.id, "demo");
        assert_eq!(library.agents.len(), 1);
        assert_eq!(library.mcp_servers.len(), 2);
        assert_eq!(library.mcp_servers[0].id, "helm");
        assert_eq!(library.mcp_servers[1].id, "ship");
        // disk rule + inline agent rule
        assert!(
            library
                .rules
                .iter()
                .any(|r| r.file_name == "discipline.md" && r.content.contains("No placeholders"))
        );
        assert!(
            library
                .rules
                .iter()
                .any(|r| r.file_name == "demo.md" && r.content.contains("Be brief"))
        );
    }

    #[test]
    fn missing_dirs_yield_empty() {
        let tmp = TempDir::new().unwrap();
        let fs_broker = LocalFsBroker::new();
        let cap = cap_for(tmp.path());
        let mut library = ExtensionLibrary::default();
        merge_jsonc_agent_nouns(&mut library, &fs_broker, &cap, tmp.path()).expect("empty ok");
        assert!(library.agent_profiles.is_empty());
        assert!(library.mcp_servers.is_empty());
        assert!(library.rules.is_empty());
    }

    #[test]
    fn corrupt_agent_jsonc_fails_closed() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("agents")).unwrap();
        fs::write(root.join("agents/bad.jsonc"), r#"{ "agent": { not valid } }"#).unwrap();
        let fs_broker = LocalFsBroker::new();
        let cap = cap_for(root);
        let mut library = ExtensionLibrary::default();
        let err = merge_jsonc_agent_nouns(&mut library, &fs_broker, &cap, root)
            .expect_err("corrupt agent must fail closed");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("fail-closed") || msg.contains("parse"),
            "named fail-closed: {msg}"
        );
        assert!(library.agent_profiles.is_empty());
    }

    #[test]
    fn fleet_jsonc_uses_discover_rollup_for_agent_nouns() {
        let tmp = TempDir::new().unwrap();
        let org = tmp.path().join("org");
        let repo = tmp.path().join("repo");
        fs::create_dir_all(org.join("agents")).unwrap();
        fs::create_dir_all(org.join("skills/spawn")).unwrap();
        fs::create_dir_all(&repo).unwrap();
        fs::write(
            org.join("agents/commander.jsonc"),
            r#"{ "agent": { "id": "commander", "name": "Commander" }, "skills": { "refs": ["spawn"] } }"#,
        )
        .unwrap();
        fs::write(
            org.join("skills/spawn/SKILL.md"),
            "---\nname: spawn\ndescription: Spawn\n---\n\n# Spawn\n",
        )
        .unwrap();
        fs::write(
            org.join("mcp.jsonc"),
            r#"{ "mcp": { "servers": { "gh": { "name": "GH", "command": "npx" } } } }"#,
        )
        .unwrap();
        fs::write(repo.join("fleet.jsonc"), r#"{ "include": ["../org"] }"#).unwrap();

        let fs_broker = LocalFsBroker::new();
        let cap = cap_for(&repo);
        let mut library = ExtensionLibrary::default();
        merge_jsonc_agent_nouns(&mut library, &fs_broker, &cap, &repo).expect("fleet graph");

        assert!(
            library
                .agent_profiles
                .iter()
                .any(|p| p.agent.id == "commander"),
            "org agent via fleet include: {:?}",
            library
                .agent_profiles
                .iter()
                .map(|p| p.agent.id.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            library.skills.iter().any(|s| s.id == "spawn"),
            "org skill via fleet include"
        );
        assert!(
            library.mcp_servers.iter().any(|m| m.id == "gh"),
            "org mcp via fleet include"
        );
    }

    #[test]
    fn fleet_jsonc_cycle_fails_closed() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(a.join("fleet.jsonc"), r#"{ "include": ["../b"] }"#).unwrap();
        fs::write(b.join("fleet.jsonc"), r#"{ "include": ["../a"] }"#).unwrap();

        let fs_broker = LocalFsBroker::new();
        let cap = cap_for(&a);
        let mut library = ExtensionLibrary::default();
        let err = merge_jsonc_agent_nouns(&mut library, &fs_broker, &cap, &a)
            .expect_err("cycle must fail closed");
        let msg = format!("{err:#}").to_lowercase();
        assert!(
            msg.contains("cycle") || msg.contains("discover") || msg.contains("fail-closed"),
            "named cycle failure: {msg}"
        );
    }
}
