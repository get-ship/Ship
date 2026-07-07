//! Agent discovery helpers for the `agents` daemon-rendered view.
//!
//! The live daemon path (`list_agents_from_index` / `load_agent_from_index`)
//! reads the project scope's composed roll-up snapshot from the in-memory
//! `AgentDefIndex` — see "Index-backed reads" below. `list_agents` /
//! `load_agent` / `agent_id_exists` are the no-index fallback for a daemon
//! (helmd) that constructs its `AgentsService` via `with_roots` and never
//! mounts an `AgentDefIndex`. Post format-migration Wave 5 there is no
//! `.ship/agents/*.jsonc` two-scope directory scan to fall back to — agents
//! only exist as `defineAgent` declare-forms in `ship.config.ts`, so the
//! no-index fallback resolves through the SAME composed world as the
//! index-backed path: `ship_resolver::roll_up_blocking` from the project
//! space root (the `compile_project` W5 pattern). The UserGlobal-tier
//! (operator-enabled library) rides the SAME roll-up (W2) — there is no
//! separate global-scope read here.
//!
//! `state::list` and `state::load_doc` (the C0 Get/List read surface the
//! `AgentsWire::list` / `AgentsWire::get` handlers ride) go through the
//! SAME composed roll-up — W5b follow-up. They share the projection
//! helpers and the resolve/compose helpers below (made `pub(crate)` for
//! that).
//!
//! Each summary/detail carries a `source` field so the view can group the
//! rail; for both the index-backed and no-index paths it is derived from
//! the roll-up's own precedence resolution, never an invented scope label.
//!
//! Projection helpers (avatar letter + collection-field array) live as
//! private fns in this module — the old sibling `agents_jsonc.rs` is
//! retired: its only consumers were the migrated paths (this module's
//! `summary_from_value` / `detail_from_value`).

use compiler::{ComposedLibrary, ExtSource};
use serde::Serialize;
use ship_config::agents_index::AgentDefIndex;
use std::path::{Path, PathBuf};

use super::state::ScopeTag;

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
/// Resolve the enclosing project's space root (the parent of its `.ship/`),
/// walking up from the current working directory. `None` outside a Ship
/// project — the no-index posture then has no space to compose (there is no
/// longer a "global-only, no project" case: the operator-enabled library
/// tier only reaches a caller THROUGH a space's roll-up).
pub(crate) fn resolve_space_root() -> Option<PathBuf> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    project_ship.parent().map(Path::to_path_buf)
}

/// Compose the project's space root into an [`ExtensionLibrary`][compiler::ExtensionLibrary]
/// snapshot — the same `ship_resolver::roll_up_blocking` call `compile_project`
/// makes (format-migration W5). This IS the no-index fallback's read: no
/// `.ship/agents/*.jsonc` directory scan survives it. `None` on a roll-up
/// failure (e.g. a TS eval error) — callers treat that the same as "nothing
/// composed", matching the old scan's "missing dir" tolerance.
pub(crate) fn compose_space_library(space_root: &Path) -> Option<ComposedLibrary> {
    ship_resolver::roll_up_blocking(space_root, None, None)
        .inspect_err(|e| tracing::warn!("agents view_state: roll_up failed for {space_root:?}: {e}"))
        .ok()
}

/// The rail's `source` label for `id` within a composed roll-up: re-applies
/// the SAME rank-then-declared-order rule `compiler::compose_space` used to
/// pick the winner (mirrors `ship_config::agents_index::provenance`'s
/// `agent_profile_owners`, which does the identical re-derivation for the
/// DataStore-backed index — no parallel precedence law invented here, just
/// the same read without a DataStore row to carry it in). `Space` (or no
/// roll-up-inputs audit trail at all) reads as project; every other tier
/// reads as the (now roll-up-composed) global/user tier.
///
/// The `ScopeTag` form is the typed return [`state::list`] /
/// [`state::load_doc`] use to populate `IndexEntry.scope` (and to enforce
/// the explicit-scope branch of [`state::load_doc`]) — they want the
/// rank-derived scope as a typed enum, not as a UI string. The
/// string-form `agent_source_label` derives from this.
pub(crate) fn agent_source_scope(composed: &ComposedLibrary, id: &str) -> ScopeTag {
    let Some(rollup) = composed.library.rollup_inputs.as_ref() else {
        return ScopeTag::Project;
    };
    let mut ranked: Vec<_> = rollup
        .extensions
        .iter()
        .filter_map(|ext| {
            rollup
                .precedence
                .iter()
                .find(|row| row.kind == ext.source.kind())
                .map(|row| (row.rank, ext))
        })
        .collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    let winner = ranked
        .into_iter()
        .filter(|(_, ext)| ext.descriptor.agents.iter().any(|a| a.agent.id == id))
        .next_back()
        .map(|(_, ext)| ext.source.clone());
    match winner {
        None | Some(ExtSource::Space) => ScopeTag::Project,
        Some(_) => ScopeTag::User,
    }
}

/// The string-form `source` label (rail-grouping string surfaced in the
/// `source` field of the wire response) — derived from
/// [`agent_source_scope`].
fn agent_source_label(composed: &ComposedLibrary, id: &str) -> &'static str {
    match agent_source_scope(composed, id) {
        ScopeTag::Project => SOURCE_PROJECT,
        ScopeTag::User => SOURCE_GLOBAL,
    }
}


/// First character of `s`, upper-cased (or `"?"` for an empty string).
/// Powers the agent-row "letter avatar" badge.
fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

/// Extract a flat `Vec<String>` from a JSON value: string entries of an
/// array, or empty for anything else.
fn extract_str_array(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a collection field from an agent config. Reads either the
/// top-level array (when `nested_keys` is empty) or a nested sub-key
/// (e.g. `skills.refs`, `mcp.servers`).
///
/// The `nested_keys` parameter lists candidate sub-keys to inspect when the
/// top-level value is an object; the first one that contains an array wins.
fn resolve_array_field(
    obj: &serde_json::Value,
    top_keys: &[&str],
    nested_keys: &[&str],
) -> Vec<String> {
    for key in top_keys {
        let Some(v) = obj.get(*key) else { continue };
        if v.is_array() {
            return extract_str_array(v);
        }
        if let Some(inner) = v.as_object() {
            for nk in nested_keys {
                if let Some(arr) = inner.get(*nk)
                    && arr.is_array()
                {
                    return extract_str_array(arr);
                }
            }
        }
    }
    Vec::new()
}
/// Project a raw agent-def value into a summary. The ONE projection shared by
/// the composed roll-up read and the index-backed read — the def value has
/// the same canonical shape whether it comes off a `serde_json::to_value`'d
/// [`compiler::AgentProfile`] or out of the index snapshot. `None` when the
/// value lacks the top-level `agent` block.
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

/// Project a raw agent-def value into a detail. Shared by the composed
/// roll-up read and the index-backed read (same canonical shape from either
/// source). `None` when the value lacks the top-level `agent` block.
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

/// List agents visible to the no-index posture: the project space's composed
/// roll-up (source + cache tier + operator-enabled library tier). `ship_dir`
/// stays a parameter (unused) only because the wire call site passes it —
/// same rationale as [`list_agents_from_index`]'s `_ship_dir`; the composed
/// world already covers the user/global tier (W2), so there is nothing left
/// for `ship_dir` to add.
pub fn list_agents(ship_dir: &Path) -> Vec<AgentSummary> {
    let _ = ship_dir;
    match resolve_space_root() {
        Some(root) => list_agents_from_space(&root),
        None => Vec::new(),
    }
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

/// Test/injection seam for [`list_agents`] — takes the space root directly
/// instead of walking up from the cwd, so parallel tests don't race on the
/// process-global project-discovery path.
pub(crate) fn list_agents_from_space(space_root: &Path) -> Vec<AgentSummary> {
    let Some(composed) = compose_space_library(space_root) else {
        return Vec::new();
    };
    let active = active_agent_id();
    let mut out: Vec<AgentSummary> = composed
        .library
        .agents
        .iter()
        .filter_map(|profile| {
            let source = agent_source_label(&composed, &profile.agent.id);
            let value = serde_json::to_value(profile).unwrap_or(serde_json::Value::Null);
            let mut a = summary_from_value(profile.agent.id.clone(), source, &value)?;
            a.active = active.as_deref() == Some(a.id.as_str());
            Some(a)
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load a single agent by id from the no-index posture's composed roll-up.
/// `ship_dir` is unused for the same reason as [`list_agents`].
pub fn load_agent(ship_dir: &Path, id: &str) -> Option<AgentDetail> {
    let _ = ship_dir;
    load_agent_from_space(&resolve_space_root()?, id)
}

/// Test/injection seam for [`load_agent`] — see [`list_agents_from_space`].
pub(crate) fn load_agent_from_space(space_root: &Path, id: &str) -> Option<AgentDetail> {
    let composed = compose_space_library(space_root)?;
    let profile = composed.library.agents.iter().find(|p| p.agent.id == id)?;
    let source = agent_source_label(&composed, id);
    let value = serde_json::to_value(profile).unwrap_or(serde_json::Value::Null);
    detail_from_value(id.to_string(), source, &value)
}

/// True when `id` resolves to a known agent in the no-index posture's
/// composed roll-up. Used by the dispatch wire to reject requests that name
/// an unknown agent BEFORE any side effect
/// (c-2026-07-05T12-33-00-000Z-a1b2c3d4). Final (no retry / degraded path;
/// an agent that doesn't exist cannot be dispatched).
pub fn agent_id_exists(ship_dir: &Path, id: &str) -> bool {
    let _ = ship_dir;
    let Some(root) = resolve_space_root() else {
        return false;
    };
    agent_exists_in_space(&root, id)
}

/// Test/injection seam for [`agent_id_exists`] — see [`list_agents_from_space`].
pub(crate) fn agent_exists_in_space(space_root: &Path, id: &str) -> bool {
    let Some(composed) = compose_space_library(space_root) else {
        return false;
    };
    composed.library.agents.iter().any(|p| p.agent.id == id)
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
// Served ENTIRELY from the `AgentDefIndex` DataStore snapshot — no re-read of
// `.ship/agents/*.jsonc` or `~/.ship/library/agents/*.jsonc` per call.
// `list_docs`'s "project" scope IS `compiler::compose_space`'s composed
// answer (`AgentDefIndex::upsert_composed`): a same-id loser is already
// excluded there, so no precedence law is re-derived here — id set and
// ranking are entirely the index's. `ship_dir` stays a parameter (unused)
// only because the wire call site passes it.

/// Map a doc's merged `_provenance.ext_source` (the `ExtSource` tag JSON
/// `list_docs`/`get_doc` merge in, e.g. `{"kind":"space"}`) to the rail's
/// `source` label — the extension kind verbatim, never an invented
/// "project"/"user-global" string. No provenance (pre-roll-up ingest) keeps
/// `fallback`.
fn source_from_provenance(doc: &serde_json::Value, fallback: &str) -> String {
    doc.get("_provenance")
        .and_then(|p| p.get("ext_source"))
        .and_then(|v| v.as_str())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("kind").and_then(|k| k.as_str()).map(str::to_string))
        .unwrap_or_else(|| fallback.to_string())
}

/// List agents from the project scope's composed roll-up snapshot.
pub async fn list_agents_from_index(
    index: &AgentDefIndex,
    _ship_dir: &Path,
) -> Vec<AgentSummary> {
    let active = active_agent_id();
    let Ok(docs) = index.list_docs("project").await else {
        return Vec::new();
    };
    let mut out: Vec<AgentSummary> = docs
        .into_iter()
        .filter_map(|(id, doc)| {
            let source = source_from_provenance(&doc, SOURCE_PROJECT);
            let mut a = summary_from_value(id, &source, &doc)?;
            a.active = active.as_deref() == Some(a.id.as_str());
            Some(a)
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load one agent by id from the project scope's composed roll-up snapshot.
pub async fn load_agent_from_index(
    index: &AgentDefIndex,
    _ship_dir: &Path,
    id: &str,
) -> Option<AgentDetail> {
    let doc = index.get_doc(id, "project").await.ok().flatten()?;
    let source = source_from_provenance(&doc, SOURCE_PROJECT);
    detail_from_value(id.to_string(), &source, &doc)
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
///
/// PRE-EXISTING BUG: this calls `super::state::list_jsonc` (the agents
/// domain's jsonc directory scan) instead of the rules service's own
/// state — the rules catalog gets populated with agent ids, not rule
/// ids. Predates this work (the original PR confused the two modules
/// when both got their `state` submodules). Out of scope here; we just
/// point it at the jsonc door (the new composed `state::list` would
/// return composed agent profiles, which is no closer to "rules"). A
/// future fix replaces this with the rules service's own reader.
fn list_rule_files(ship_dir: &Path) -> Vec<CatalogEntry> {
    let files = ship_files::files::FilesService::resolve_roots(ship_dir);
    super::state::list_jsonc(&files)
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
