//! Agent discovery helpers for the `agents` daemon-rendered view.
//!
//! Reads agent configs from `<ship_dir>/agents/*.{jsonc,json}` across two
//! scopes and merges them — project (discovered from cwd via
//! `runtime::project::get_project_dir`) wins over global on id collision.
//! Each summary/detail carries a `source` field so the view can group the
//! rail by scope.
//!
//! JSONC parsing lives in the sibling `agents_jsonc` module.

use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::agents_jsonc::{first_letter, parse_file, resolve_array_field};

// Re-export helpers whose names tests refer to directly (via `super::*`
// in the sibling tests file). Canonical home is `agents_jsonc`.
#[cfg(test)]
pub(crate) use super::agents_jsonc::strip_jsonc;

#[derive(Clone, Debug, Serialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skills_count: usize,
    pub providers_count: usize,
    pub mcp_count: usize,
    pub rules_count: usize,
    /// First letter of `name` uppercased, for the avatar glyph.
    pub letter: String,
    pub source: String,
    /// Providers this agent compiles to (claude/codex/cursor/gemini/opencode).
    /// The list-view renders one tinted blob per provider.
    pub providers: Vec<String>,
    /// Freeform tags from the agent jsonc (e.g. "workflow", "review").
    pub tags: Vec<String>,
    /// True when the agent is currently participating in an active session /
    /// job. Today unchecked — reserved for runtime wiring. Defaults false.
    pub active: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub letter: String,
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub rules: Vec<String>,
    pub providers: Vec<String>,
    pub default_provider: Option<String>,
    pub permissions_tier: String,
    pub source: String,
}

/// Source label for project-scoped agents (enclosing `.ship/` of cwd).
pub const SOURCE_PROJECT: &str = "PROJECT AGENTS";
/// Source label for global-scoped agents (`~/.ship/agents/`).
pub const SOURCE_GLOBAL: &str = "USER AGENTS";

/// Resolved search roots for the two scopes.
pub(crate) struct AgentScopes {
    pub global_dir: PathBuf,
    pub project_dir: Option<PathBuf>,
}

fn resolve_scopes(ship_dir: &Path) -> AgentScopes {
    let project_ship = runtime::project::get_project_dir(None)
        .ok()
        .filter(|p| p != ship_dir);
    scopes_from(ship_dir, project_ship.as_deref())
}

/// Build scopes for an explicit (global, project) pair. Tests call this to
/// inject both sides without touching the process-global `SHIP_DIR` env var.
pub(crate) fn scopes_from(
    ship_dir: &Path,
    project_ship_dir: Option<&Path>,
) -> AgentScopes {
    let global_dir = ship_dir.join("agents");
    let project_dir = project_ship_dir
        .filter(|p| *p != ship_dir)
        .map(|p| p.join("agents"));
    AgentScopes { global_dir, project_dir }
}

fn scan_agents_dir(dir: &Path, source: &str) -> Vec<AgentSummary> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if !path.is_file() || !matches!(ext, Some("jsonc" | "json")) {
            continue;
        }
        let Some((id, parsed)) = parse_file(&path) else { continue };
        let agent_obj = parsed.get("agent").unwrap_or(&parsed);
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
        let skills = resolve_array_field(&parsed, &["skills"], &["refs", "ids"]);
        let providers = resolve_array_field(agent_obj, &["providers"], &["ids"]);
        let mcp = resolve_array_field(
            &parsed,
            &["mcp_servers", "mcpServers", "mcp"],
            &["servers", "refs"],
        );
        let rules = resolve_array_field(&parsed, &["rules"], &["refs", "ids"]);
        let tags = resolve_array_field(agent_obj, &["tags"], &[]);
        let letter = first_letter(&name);
        out.push(AgentSummary {
            id,
            name,
            description,
            skills_count: skills.len(),
            providers_count: providers.len(),
            mcp_count: mcp.len(),
            rules_count: rules.len(),
            letter,
            source: source.to_string(),
            providers,
            tags,
            active: false,
        });
    }
    out
}

fn load_from_dir(dir: &Path, id: &str, source: &str) -> Option<AgentDetail> {
    for ext in ["jsonc", "json"] {
        let path = dir.join(format!("{id}.{ext}"));
        if !path.is_file() {
            continue;
        }
        let (file_id, parsed) = parse_file(&path)?;
        let agent_obj = parsed.get("agent").unwrap_or(&parsed);
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
        let skills = resolve_array_field(&parsed, &["skills"], &["refs", "ids"]);
        let providers = resolve_array_field(agent_obj, &["providers"], &["ids"]);
        let mcp_servers = resolve_array_field(
            &parsed,
            &["mcp_servers", "mcpServers", "mcp"],
            &["servers", "refs"],
        );
        let rules = resolve_array_field(&parsed, &["rules"], &["refs", "ids"]);
        let default_provider = agent_obj
            .get("default_provider")
            .or_else(|| agent_obj.get("defaultProvider"))
            .or_else(|| agent_obj.get("provider"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        // Permissions tier lives under `permissions.tier` (flat) or — for
        // project configs that use the preset name — `permissions.preset`
        // with a `ship-<tier>` prefix. Strip the prefix to canonicalize.
        let permissions_obj = parsed.get("permissions");
        let permissions_tier = permissions_obj
            .and_then(|p: &serde_json::Value| p.get("tier").and_then(|t: &serde_json::Value| t.as_str()).map(|s: &str| s.to_string()))
            .or_else(|| {
                permissions_obj
                    .and_then(|p: &serde_json::Value| p.get("preset").and_then(|t: &serde_json::Value| t.as_str()))
                    .map(|s: &str| s.trim_start_matches("ship-").to_string())
            })
            .unwrap_or_else(|| "standard".to_string());
        return Some(AgentDetail {
            id: file_id,
            name: name.clone(),
            description,
            letter: first_letter(&name),
            skills,
            mcp_servers,
            rules,
            providers,
            default_provider,
            permissions_tier,
            source: source.to_string(),
        });
    }
    None
}

/// List agents from project (if inside a Ship project) and global scopes.
/// Project entries shadow global on id collision.
pub fn list_agents(ship_dir: &Path) -> Vec<AgentSummary> {
    list_agents_with_scopes(&resolve_scopes(ship_dir))
}

pub(crate) fn list_agents_with_scopes(scopes: &AgentScopes) -> Vec<AgentSummary> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    if let Some(dir) = scopes.project_dir.as_ref() {
        for a in scan_agents_dir(dir, SOURCE_PROJECT) {
            if seen.insert(a.id.clone()) {
                out.push(a);
            }
        }
    }
    for a in scan_agents_dir(&scopes.global_dir, SOURCE_GLOBAL) {
        if seen.insert(a.id.clone()) {
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

pub(crate) fn load_agent_with_scopes(
    scopes: &AgentScopes,
    id: &str,
) -> Option<AgentDetail> {
    if let Some(dir) = scopes.project_dir.as_ref() {
        if let Some(d) = load_from_dir(dir, id, SOURCE_PROJECT) {
            return Some(d);
        }
    }
    load_from_dir(&scopes.global_dir, id, SOURCE_GLOBAL)
}

// ── Create ────────────────────────────────────────────────────────────────────
//
// Minimal agent-file writer backing the create-wizard's submit. The wizard
// posts the raw payload; we validate against the schema at
// schemas/agent.schema.json, project-scope the write (project dir wins
// when the daemon is inside a Ship project; global dir otherwise), and
// refuse to overwrite. The written JSONC mirrors the schema contract
// exactly — no template/hue fields bleed in.

/// Payload accepted by the create-agent endpoint. Deliberately small; the
/// wizard's step-3 review is the source of truth.
#[derive(serde::Deserialize, Debug)]
pub struct CreateAgentRequest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub permissions: Option<PermissionsInput>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct PermissionsInput {
    #[serde(default)]
    pub preset: Option<String>,
}

#[derive(Debug)]
pub enum CreateAgentError {
    BadId,
    BadProvider(String),
    Exists(PathBuf),
    Io(std::io::Error),
}

impl std::fmt::Display for CreateAgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadId => write!(f, "id must match ^[a-z0-9-]+$"),
            Self::BadProvider(p) => write!(f, "unknown provider `{p}` — must be one of claude, codex, cursor, gemini, opencode"),
            Self::Exists(p) => write!(f, "{} already exists", p.display()),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CreateAgentResult {
    pub id: String,
    pub path: String,
    pub scope: String,
}

/// Write a new agent JSONC to the appropriate scope. Project-scoped when
/// the daemon is inside a Ship project (so the agent lives with the
/// project source); global otherwise. Refuses to overwrite.
pub fn create_agent(
    ship_dir: &Path,
    req: &CreateAgentRequest,
) -> Result<CreateAgentResult, CreateAgentError> {
    // id validation mirrors the schema's `pattern: ^[a-z0-9-]+$`.
    let id = req.id.trim();
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(CreateAgentError::BadId);
    }
    // providers enum
    const ALLOWED_PROVIDERS: &[&str] = &["claude", "codex", "cursor", "gemini", "opencode"];
    for p in &req.providers {
        if !ALLOWED_PROVIDERS.contains(&p.as_str()) {
            return Err(CreateAgentError::BadProvider(p.clone()));
        }
    }

    // Resolve write scope: project dir wins when available.
    let scopes = resolve_scopes(ship_dir);
    let (dir, scope_label) = match scopes.project_dir.as_ref() {
        Some(p) => (p.clone(), "project"),
        None => (scopes.global_dir, "global"),
    };
    std::fs::create_dir_all(&dir).map_err(CreateAgentError::Io)?;

    let path = dir.join(format!("{id}.jsonc"));
    if path.exists() {
        return Err(CreateAgentError::Exists(path));
    }

    // Build a schema-conformant JSONC document. Only fields the wizard
    // captures are emitted; template/hue don't belong in the file.
    let agent_block = {
        let mut o = serde_json::Map::new();
        o.insert("name".into(), serde_json::Value::String(req.name.clone()));
        o.insert("id".into(), serde_json::Value::String(id.to_string()));
        if let Some(d) = &req.description {
            if !d.trim().is_empty() {
                o.insert("description".into(), serde_json::Value::String(d.clone()));
            }
        }
        if !req.providers.is_empty() {
            o.insert(
                "providers".into(),
                serde_json::Value::Array(
                    req.providers.iter().map(|p| serde_json::Value::String(p.clone())).collect(),
                ),
            );
        }
        serde_json::Value::Object(o)
    };

    let mut doc = serde_json::Map::new();
    doc.insert(
        "$schema".into(),
        serde_json::Value::String(
            "https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json".into(),
        ),
    );
    doc.insert("schema_version".into(), serde_json::Value::String("1".into()));
    doc.insert("agent".into(), agent_block);
    if let Some(perms) = &req.permissions {
        if let Some(preset) = perms.preset.as_ref().filter(|s| !s.is_empty()) {
            let mut p = serde_json::Map::new();
            p.insert("preset".into(), serde_json::Value::String(preset.clone()));
            doc.insert("permissions".into(), serde_json::Value::Object(p));
        }
    }

    let body = serde_json::to_string_pretty(&serde_json::Value::Object(doc))
        .map_err(|e| CreateAgentError::Io(std::io::Error::other(e)))?;

    // Atomic-ish write: write to a sibling temp file then rename.
    let tmp = path.with_extension("jsonc.tmp");
    std::fs::write(&tmp, body).map_err(CreateAgentError::Io)?;
    std::fs::rename(&tmp, &path).map_err(CreateAgentError::Io)?;

    Ok(CreateAgentResult {
        id: id.to_string(),
        path: path.display().to_string(),
        scope: scope_label.into(),
    })
}

// ─── Patch agent ──────────────────────────────────────────────────────
//
// Atomic-ish patch of an existing agent's JSONC. Supports appending to
// the three array-typed slots the detail panels expose:
//   - skills.refs       — skill IDs to activate
//   - mcp.servers       — MCP server IDs to activate
//   - rules.refs        — rule file IDs to activate (extension of the
//                         schema's rules.inline; view already reads
//                         either shape via resolve_array_field)
//
// Duplicates are de-duped. Arrays are created if missing. The rest of
// the document — including comments, field order, and unrelated keys —
// survives only for keys we round-trip through serde_json; JSONC
// comments on the specific edited keys are lost. For comment-preserving
// edits we'd need a lossless JSONC round-trip (e.g. `jsonc-parser`
// crate with span retention), filed as a follow-up.

#[derive(serde::Deserialize, Debug)]
pub struct PatchAgentRequest {
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub mcp: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
}

#[derive(Debug)]
pub enum PatchAgentError {
    NotFound(String),
    Parse(String),
    Io(std::io::Error),
}

impl std::fmt::Display for PatchAgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "agent `{id}` not found"),
            Self::Parse(e) => write!(f, "couldn't parse agent JSONC: {e}"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

/// Locate the JSONC file for an agent id across project + global scopes.
fn find_agent_path(ship_dir: &Path, id: &str) -> Option<PathBuf> {
    let scopes = resolve_scopes(ship_dir);
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = scopes.project_dir {
        candidates.push(p.join(format!("{id}.jsonc")));
        candidates.push(p.join(format!("{id}.json")));
    }
    candidates.push(scopes.global_dir.join(format!("{id}.jsonc")));
    candidates.push(scopes.global_dir.join(format!("{id}.json")));
    candidates.into_iter().find(|p| p.exists())
}

/// Merge a string array into `doc.section.key` ensuring uniqueness.
fn merge_array(
    doc: &mut serde_json::Map<String, serde_json::Value>,
    section: &str,
    key: &str,
    adds: &[String],
) {
    if adds.is_empty() {
        return;
    }
    let section_entry = doc
        .entry(section.to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    let Some(section_obj) = section_entry.as_object_mut() else {
        return;
    };
    let arr_entry = section_obj
        .entry(key.to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let Some(arr) = arr_entry.as_array_mut() else {
        return;
    };
    let mut seen: HashSet<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    for v in adds {
        if seen.insert(v.clone()) {
            arr.push(serde_json::Value::String(v.clone()));
        }
    }
}

pub fn patch_agent(
    ship_dir: &Path,
    id: &str,
    req: &PatchAgentRequest,
) -> Result<(), PatchAgentError> {
    let path = find_agent_path(ship_dir, id).ok_or_else(|| PatchAgentError::NotFound(id.into()))?;
    let raw = std::fs::read_to_string(&path).map_err(PatchAgentError::Io)?;
    let stripped = super::agents_jsonc::strip_jsonc(&raw);
    let mut doc: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&stripped).map_err(|e| PatchAgentError::Parse(e.to_string()))?;

    merge_array(&mut doc, "skills", "refs", &req.skills);
    merge_array(&mut doc, "mcp", "servers", &req.mcp);
    merge_array(&mut doc, "rules", "refs", &req.rules);

    let body = serde_json::to_string_pretty(&serde_json::Value::Object(doc))
        .map_err(|e| PatchAgentError::Io(std::io::Error::other(e)))?;
    let tmp = path.with_extension("jsonc.tmp");
    std::fs::write(&tmp, body).map_err(PatchAgentError::Io)?;
    std::fs::rename(&tmp, &path).map_err(PatchAgentError::Io)?;
    Ok(())
}

// ─── Catalogs (available skills / mcp / rules) ────────────────────────
//
// The detail-panel pickers need to know what the user can select. We
// enumerate the three stores on disk:
//   - skills: reuse list_skills() → id + name
//   - mcp: runtime::config::list_mcp_servers()
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
}

fn list_rule_files(ship_dir: &Path) -> Vec<CatalogEntry> {
    let dir = runtime::project::rules_dir(ship_dir);
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        out.push(CatalogEntry {
            id: stem.to_string(),
            name: stem.to_string(),
            description: None,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn list_agent_catalogs(ship_dir: &Path) -> AgentCatalogs {
    let skills = super::skills_state::list_skills(ship_dir)
        .into_iter()
        .map(|s| CatalogEntry {
            id: s.id,
            name: s.name,
            description: if s.description.is_empty() { None } else { Some(s.description) },
        })
        .collect();
    let mcp = runtime::config::list_mcp_servers(Some(
        ship_dir.parent().map(Path::to_path_buf).unwrap_or_else(|| ship_dir.to_path_buf()),
    ))
    .unwrap_or_default()
    .into_iter()
    .map(|s| CatalogEntry {
        id: if s.id.is_empty() { s.name.clone() } else { s.id },
        name: s.name,
        description: None,
    })
    .collect();
    let rules = list_rule_files(ship_dir);
    AgentCatalogs { skills, mcp, rules }
}

#[cfg(test)]
#[path = "agents_state_tests.rs"]
mod tests;
