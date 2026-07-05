//! Agent discovery helpers for the `agents` daemon-rendered view.
//!
//! Reads agent configs from `<ship_dir>/agents/*.{jsonc,json}` across two
//! scopes and merges them — project (discovered from cwd via
//! `ship_substrate::project::get_project_dir`) wins over global on id collision.
//! Each summary/detail carries a `source` field so the view can group the
//! rail by scope.
//!
//! JSONC parsing lives in the sibling `agents_jsonc` module.

use super::agents_jsonc::{first_letter, resolve_array_field};
use serde::Serialize;
use ship_config::agents_index::AgentDefIndex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// Re-export `strip_jsonc` so the sibling test file can pull it in via
// `super::*` (the comment-stripping itself still delegates to
// `ship_substrate::fs_util::strip_jsonc_comments`).
#[cfg(test)]
pub(crate) use ship_files::jsonc::strip_jsonc;

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skills_count: usize,
    pub mcp_count: usize,
    pub rules_count: usize,
    /// First letter of `name` uppercased, for the avatar glyph.
    pub letter: String,
    pub source: String,
    /// Freeform tags from the agent jsonc (e.g. "workflow", "review").
    pub tags: Vec<String>,
    /// True when the agent is currently participating in an active session /
    /// job. Today unchecked — reserved for runtime wiring. Defaults false.
    pub active: bool,
}

/// Wire shape of `ship.agents.summaries()`.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AgentsSummariesResponse {
    pub items: Vec<AgentSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub letter: String,
    /// Optional icon name (lucide). Surfaced in detail header.
    pub icon: Option<String>,
    /// Optional semver-ish version string from `agent.version`.
    pub version: Option<String>,
    /// Tag strings from `agent.tags`.
    pub tags: Vec<String>,
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub rules: Vec<String>,
    /// Verbatim `rules.inline` text from the agent jsonc — surfaced in the
    /// detail editor so the inline rules can be read + edited in place.
    pub inline_rules: String,
    /// Legacy — canonicalized tier for existing UI that only shows a
    /// single "tier" (one of standard / autonomous / read-only). Kept
    /// for backward compat while the permissions panel migrates to
    /// the full editor; derived from `permissions.preset` by stripping
    /// a `ship-` prefix.
    pub permissions_tier: String,
    /// Full permissions shape matching the agent schema. Preset is the
    /// named entry from .ship/permissions.jsonc; the three tool-pattern
    /// lists are merged on top of the preset.
    pub permissions: Permissions,
    pub source: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Permissions {
    pub preset: String,
    pub tools_allow: Vec<String>,
    pub tools_ask: Vec<String>,
    pub tools_deny: Vec<String>,
}

/// Source label for project-scoped agents (enclosing `.ship/` of cwd).
pub const SOURCE_PROJECT: &str = "PROJECT AGENTS";
/// Source label for user-scope agents (`~/.ship/library/agents/`).
pub const SOURCE_GLOBAL: &str = "USER AGENTS";

/// Resolved search roots for the two scopes.
pub(crate) struct AgentScopes {
    pub global_dir: PathBuf,
    pub project_dir: Option<PathBuf>,
}

fn resolve_scopes(ship_dir: &Path) -> AgentScopes {
    let project_ship = ship_substrate::project::get_project_dir(None)
        .ok()
        .filter(|p| p != ship_dir);
    scopes_from(ship_dir, project_ship.as_deref())
}

/// Build scopes for an explicit (global, project) pair. Tests call this to
/// inject both sides without touching the process-global `SHIP_DEV_PROJECT_DIR` env var.
///
/// `ship_dir` is the user-scope home (`~/.ship/`); user-scope agents live
/// at `library/agents/` per the home-layout ADR. `project_ship_dir` is the
/// project-scope `.ship/` directly (no `library/` prefix at project scope).
pub(crate) fn scopes_from(ship_dir: &Path, project_ship_dir: Option<&Path>) -> AgentScopes {
    let global_dir = ship_substrate::ShipHome::at(ship_dir.to_path_buf()).vector_dir(
        ship_types::Scope::User,
        ship_types::Vector::Agents,
        None,
    );
    let project_dir = project_ship_dir
        .filter(|p| *p != ship_dir)
        .map(|p| p.join("agents"));
    AgentScopes {
        global_dir,
        project_dir,
    }
}

fn scan_agents_dir(dir: &Path, source: &str) -> Vec<AgentSummary> {
    // THE agent-def reader (FsBroker, ship-config) is the one door — no raw
    // `std::fs` walk here. It yields `(id, value)` for each def; the summary
    // is projected from the raw value (fields the nested profile drops, e.g.
    // `rules.refs`, survive).
    let fs = ship_substrate::ports::LocalFsBroker::new();
    let Ok(docs) = ship_config::agents::read_agent_docs_in(&fs, dir) else {
        return Vec::new();
    };
    docs.into_iter()
        .filter_map(|(id, parsed)| summary_from_value(id, source, &parsed))
        .collect()
}

/// Project a raw agent-def value into a summary. The ONE projection shared by
/// the file scan and the index-backed read — the def value has the same
/// canonical shape whether it comes off disk or out of the index snapshot.
/// `None` when the value lacks the top-level `agent` block.
fn summary_from_value(
    id: String,
    source: &str,
    parsed: &serde_json::Value,
) -> Option<AgentSummary> {
    let agent_obj = parsed.get("agent")?;
    let name = agent_obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();
    let description = agent_obj
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let skills = resolve_array_field(parsed, &["skills"], &["refs"]);
    let mcp = resolve_array_field(parsed, &["mcp"], &["servers"]);
    // Schema/UI mismatch: schema says `rules.inline: string`; UI still
    // wants a list. Reading `rules.refs` until the UI redesign lands.
    let rules = resolve_array_field(parsed, &["rules"], &["refs"]);
    let tags = resolve_array_field(agent_obj, &["tags"], &[]);
    let letter = first_letter(&name);
    Some(AgentSummary {
        id,
        name,
        description,
        skills_count: skills.len(),
        mcp_count: mcp.len(),
        rules_count: rules.len(),
        letter,
        source: source.to_string(),
        tags,
        active: false,
    })
}

fn load_from_dir(dir: &Path, id: &str, source: &str) -> Option<AgentDetail> {
    // THE agent-def reader (FsBroker, ship-config) is the one door — no raw
    // `std::fs` lookup here. It resolves `<id>.jsonc` then `<id>.json` and
    // yields the raw value the detail projects from.
    let fs = ship_substrate::ports::LocalFsBroker::new();
    let (file_id, parsed) = ship_config::agents::read_agent_doc_in(&fs, dir, id)
        .ok()
        .flatten()?;
    detail_from_value(file_id, source, &parsed)
}

/// Project a raw agent-def value into a detail. Shared by the file lookup and
/// the index-backed read (same canonical shape from either source). `None` when
/// the value lacks the top-level `agent` block.
fn detail_from_value(
    file_id: String,
    source: &str,
    parsed: &serde_json::Value,
) -> Option<AgentDetail> {
    {
        let agent_obj = parsed.get("agent")?;
        let name = agent_obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&file_id)
            .to_string();
        let description = agent_obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let skills = resolve_array_field(parsed, &["skills"], &["refs"]);
        let mcp_servers = resolve_array_field(parsed, &["mcp"], &["servers"]);
        let rules = resolve_array_field(parsed, &["rules"], &["refs"]);
        let inline_rules = parsed
            .get("rules")
            .and_then(|r| r.get("inline"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let icon = agent_obj
            .get("icon")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let version = agent_obj
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let tags = resolve_array_field(agent_obj, &["tags"], &[]);
        // Permissions tier lives under `permissions.tier` (flat) or — for
        // project configs that use the preset name — `permissions.preset`
        // with a `ship-<tier>` prefix. Strip the prefix to canonicalize.
        let permissions_obj = parsed.get("permissions");
        let permissions_tier = permissions_obj
            .and_then(|p: &serde_json::Value| {
                p.get("tier")
                    .and_then(|t: &serde_json::Value| t.as_str())
                    .map(|s: &str| s.to_string())
            })
            .or_else(|| {
                permissions_obj
                    .and_then(|p: &serde_json::Value| {
                        p.get("preset").and_then(|t: &serde_json::Value| t.as_str())
                    })
                    .map(|s: &str| s.trim_start_matches("ship-").to_string())
            })
            .unwrap_or_else(|| "standard".to_string());
        let permissions = Permissions {
            preset: permissions_obj
                .and_then(|p| p.get("preset").and_then(|t| t.as_str()))
                .unwrap_or("")
                .to_string(),
            tools_allow: permissions_obj
                .and_then(|p| p.get("tools_allow").and_then(|v| v.as_array()))
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            tools_ask: permissions_obj
                .and_then(|p| p.get("tools_ask").and_then(|v| v.as_array()))
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            tools_deny: permissions_obj
                .and_then(|p| p.get("tools_deny").and_then(|v| v.as_array()))
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        };
        Some(AgentDetail {
            id: file_id,
            name: name.clone(),
            description,
            letter: first_letter(&name),
            icon,
            version,
            tags,
            skills,
            mcp_servers,
            rules,
            inline_rules,
            permissions_tier,
            permissions,
            source: source.to_string(),
        })
    }
}

/// List agents from project (if inside a Ship project) and global scopes.
/// Project entries shadow global on id collision.
pub fn list_agents(ship_dir: &Path) -> Vec<AgentSummary> {
    list_agents_with_scopes(&resolve_scopes(ship_dir))
}

/// Read the workspace-active agent id from `daemon_settings`. The
/// `/agents` list view dims everything but the active row; activation
/// happens through `POST /api/v1/settings { key: "shell.agent.active" }`.
pub fn active_agent_id() -> Option<String> {
    ship_config::settings::get_setting("shell.agent.active")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
}

pub(crate) fn list_agents_with_scopes(scopes: &AgentScopes) -> Vec<AgentSummary> {
    let active = active_agent_id();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    if let Some(dir) = scopes.project_dir.as_ref() {
        for mut a in scan_agents_dir(dir, SOURCE_PROJECT) {
            if seen.insert(a.id.clone()) {
                a.active = active.as_deref() == Some(a.id.as_str());
                out.push(a);
            }
        }
    }
    for mut a in scan_agents_dir(&scopes.global_dir, SOURCE_GLOBAL) {
        if seen.insert(a.id.clone()) {
            a.active = active.as_deref() == Some(a.id.as_str());
            out.push(a);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load a single agent by id. Project scope wins on collision.
pub fn load_agent(ship_dir: &Path, id: &str) -> Option<AgentDetail> {
    load_agent_with_scopes(&resolve_scopes(ship_dir), id)
}

pub(crate) fn load_agent_with_scopes(scopes: &AgentScopes, id: &str) -> Option<AgentDetail> {
    if let Some(dir) = scopes.project_dir.as_ref() {
        if let Some(d) = load_from_dir(dir, id, SOURCE_PROJECT) {
            return Some(d);
        }
    }
    load_from_dir(&scopes.global_dir, id, SOURCE_GLOBAL)
}

/// True when `id` resolves to a known agent in either scope (project wins).
/// Used by the dispatch wire to reject requests that name an unknown agent
/// BEFORE any side effect (c-2026-07-05T12-33-00-000Z-a1b2c3d4). The check
/// itself is cheap — a directory scan per scope — and final (no retry /
/// degraded path; an agent that doesn't exist cannot be dispatched).
pub fn agent_id_exists(ship_dir: &Path, id: &str) -> bool {
    let scopes = resolve_scopes(ship_dir);
    if let Some(dir) = scopes.project_dir.as_ref() {
        if load_from_dir(dir, id, SOURCE_PROJECT).is_some() {
            return true;
        }
    }
    load_from_dir(&scopes.global_dir, id, SOURCE_GLOBAL).is_some()
}

/// True when `id` resolves to a known agent in the project scope served from
/// the [`AgentDefIndex`] snapshot, or in the user scope served from files.
/// Companion to [`agent_id_exists`] for daemons that keep the project index
/// in-memory (so they don't have to re-read the on-disk config on every
/// dispatch validation).
pub async fn agent_id_exists_in_index(index: &AgentDefIndex, id: &str) -> bool {
    if let Ok(Some(_)) = index.get_doc(id, "project").await {
        return true;
    }
    if let Ok(Some(_)) = index.get_doc(id, "user").await {
        return true;
    }
    false
}

// ─── Index-backed reads (AGENTS domain owns its config) ───────────────────────
//
// The project scope is served from the `AgentDefIndex` DataStore snapshot — the
// read model kept live by `agent.def.updated` events — so a running daemon does
// NOT re-read `.ship/agents/*.jsonc` on every list/detail call. The user library
// scope has no index and stays on its file reader (those defs live under
// `~/.ship/library/agents/`, not the project `.ship/agents/` this invariant
// covers). Project shadows user on id collision, matching the file path.

/// List agents with the project scope served from the index snapshot and the
/// user scope from files. Project shadows user on id collision.
pub async fn list_agents_from_index(index: &AgentDefIndex, ship_dir: &Path) -> Vec<AgentSummary> {
    let scopes = resolve_scopes(ship_dir);
    let active = active_agent_id();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    if let Ok(docs) = index.list_docs("project").await {
        for (id, doc) in docs {
            if let Some(mut a) = summary_from_value(id, SOURCE_PROJECT, &doc) {
                if seen.insert(a.id.clone()) {
                    a.active = active.as_deref() == Some(a.id.as_str());
                    out.push(a);
                }
            }
        }
    }
    for mut a in scan_agents_dir(&scopes.global_dir, SOURCE_GLOBAL) {
        if seen.insert(a.id.clone()) {
            a.active = active.as_deref() == Some(a.id.as_str());
            out.push(a);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load one agent by id: project scope from the index snapshot, else the user
/// scope from files. Project wins on collision.
pub async fn load_agent_from_index(
    index: &AgentDefIndex,
    ship_dir: &Path,
    id: &str,
) -> Option<AgentDetail> {
    if let Ok(Some(doc)) = index.get_doc(id, "project").await {
        if let Some(d) = detail_from_value(id.to_string(), SOURCE_PROJECT, &doc) {
            return Some(d);
        }
    }
    let scopes = resolve_scopes(ship_dir);
    load_from_dir(&scopes.global_dir, id, SOURCE_GLOBAL)
}

// ─── Catalogs (available skills / mcp / rules) ────────────────────────
//
// The detail-panel pickers need to know what the user can select. We
// enumerate the three stores on disk:
//   - skills: reuse list_skills() → id + name
//   - mcp: ship_platform::config::list_mcp_servers()
//   - rules: scan .ship/rules/*.md

#[derive(Serialize, Debug)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct AgentCatalogs {
    pub skills: Vec<CatalogEntry>,
    pub mcp: Vec<CatalogEntry>,
    pub rules: Vec<CatalogEntry>,
    /// Named permission presets from `.ship/permissions.jsonc`.
    /// The editor's preset select populates from this list; falls back
    /// to the three canonical tiers when the file is absent.
    pub presets: Vec<CatalogEntry>,
    /// Known-good tool patterns the autocomplete suggests. Merged from
    /// harness builtins + Ship MCP tools + user-declared MCP
    /// `active_tools` + a small curated Bash-shape list. Free-form
    /// strings outside this list are still accepted in the editor.
    pub tool_patterns: Vec<ToolPatternEntry>,
}

#[derive(Serialize, Debug)]
pub struct ToolPatternEntry {
    pub pattern: String,
    /// Short label shown in the autocomplete dropdown.
    pub label: String,
    /// Where this pattern comes from — "harness", "mcp-ship",
    /// "mcp-<server>", "bash".
    pub source: String,
}

/// Enumerate rules via RuleService state. Returns project+user, project-wins.
fn list_rule_files(ship_dir: &Path) -> Vec<CatalogEntry> {
    let files = ship_files::files::FilesService::resolve_roots(ship_dir);
    super::state::list(&files)
        .into_iter()
        .map(|e| CatalogEntry {
            id: e.id.clone(),
            name: e.id,
            description: None,
        })
        .collect()
}

/// Enumerate permission presets via PermissionService state. Falls back to
/// the three canonical tiers when no file exists — so the UI always has
/// something to pick.
fn list_permission_presets(ship_dir: &Path) -> Vec<CatalogEntry> {
    let files = ship_files::files::FilesService::resolve_roots(ship_dir);
    let entries = crate::services::permissions::state::list(&files);
    if entries.is_empty() {
        return fallback_presets();
    }
    // Load each preset's doc so we can surface `description` (best-effort).
    let mut out: Vec<CatalogEntry> = Vec::new();
    for e in entries {
        let description = crate::services::permissions::state::load_entry(&files, &e.id, None)
            .ok()
            .and_then(|(_, v)| {
                v.get("description")
                    .and_then(|d| d.as_str())
                    .map(String::from)
            });
        out.push(CatalogEntry {
            id: e.id.clone(),
            name: e.id,
            description,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn fallback_presets() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry {
            id: "ship-standard".into(),
            name: "ship-standard".into(),
            description: Some("Human confirms risky actions before they run.".into()),
        },
        CatalogEntry {
            id: "ship-autonomous".into(),
            name: "ship-autonomous".into(),
            description: Some("Agent acts end-to-end; safety rails still apply.".into()),
        },
        CatalogEntry {
            id: "ship-readonly".into(),
            name: "ship-readonly".into(),
            description: Some("Agent inspects and reports. No writes.".into()),
        },
    ]
}

/// Build the tool-pattern autocomplete catalog. Intentionally small —
/// harness builtins + Ship MCP tool names + curated Bash shapes +
/// a wildcard hint. User-declared MCP server `active_tools` get
/// added when present.
fn list_tool_patterns(ship_dir: &Path) -> Vec<ToolPatternEntry> {
    let mut out: Vec<ToolPatternEntry> = Vec::new();

    // Harness builtins — invariant across projects.
    for t in &[
        "Read",
        "Write",
        "Edit",
        "Glob",
        "Grep",
        "Bash",
        "NotebookEdit",
        "WebFetch",
        "WebSearch",
        "Task",
    ] {
        out.push(ToolPatternEntry {
            pattern: t.to_string(),
            label: t.to_string(),
            source: "harness".into(),
        });
    }

    // Common Bash shapes — curated, not exhaustive.
    for (p, l) in &[
        ("Bash(git *)", "Bash git · all git subcommands"),
        ("Bash(git reset*)", "Bash git · reset (destructive)"),
        (
            "Bash(git push --force*)",
            "Bash git · force push (destructive)",
        ),
        ("Bash(npm *)", "Bash npm · all npm subcommands"),
        ("Bash(bun *)", "Bash bun · all bun subcommands"),
        ("Bash(cargo *)", "Bash cargo"),
        ("Bash(ship *)", "Bash ship · Ship CLI"),
        ("Bash(just *)", "Bash just"),
        ("Bash(rm -rf *)", "Bash rm · recursive delete (destructive)"),
    ] {
        out.push(ToolPatternEntry {
            pattern: (*p).into(),
            label: (*l).into(),
            source: "bash".into(),
        });
    }

    // MCP servers — wildcard entry per server. Delegates to McpService.
    let files = ship_files::files::FilesService::resolve_roots(ship_dir);
    for entry in crate::services::mcp::state::list(&files) {
        out.push(ToolPatternEntry {
            pattern: format!("mcp__{}__*", entry.id),
            label: format!("mcp · {} · all tools", entry.id),
            source: format!("mcp-{}", entry.id),
        });
    }

    out
}

pub fn list_agent_catalogs(ship_dir: &Path) -> AgentCatalogs {
    let skills = crate::services::skills::view_state::list_skills(ship_dir)
        .into_iter()
        .map(|s| CatalogEntry {
            id: s.id,
            name: s.name,
            description: if s.description.is_empty() {
                None
            } else {
                Some(s.description)
            },
        })
        .collect();
    let mcp_files = ship_files::files::FilesService::resolve_roots(ship_dir);
    let mcp: Vec<CatalogEntry> = crate::services::mcp::state::list(&mcp_files)
        .into_iter()
        .map(|e| {
            // Best-effort name: load entry and use `name` if present, else id.
            let name = crate::services::mcp::state::load_entry(&mcp_files, &e.id, None)
                .ok()
                .and_then(|(_, v)| v.get("name").and_then(|n| n.as_str()).map(String::from))
                .unwrap_or_else(|| e.id.clone());
            CatalogEntry {
                id: e.id,
                name,
                description: None,
            }
        })
        .collect();
    let rules = list_rule_files(ship_dir);
    let presets = list_permission_presets(ship_dir);
    let tool_patterns = list_tool_patterns(ship_dir);
    AgentCatalogs {
        skills,
        mcp,
        rules,
        presets,
        tool_patterns,
    }
}

#[cfg(test)]
#[path = "view_state_tests.rs"]
mod tests;
