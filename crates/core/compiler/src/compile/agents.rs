//! Subagent compiler — transforms `AgentProfile` TOML into harness-native agent files.
//!
//! Each harness has its own agent definition format:
//! - **Claude**: `.claude/agents/<id>.md` — YAML frontmatter + Markdown body
//! - **Gemini**: `.gemini/agents/<id>.md` — YAML frontmatter + Markdown body
//! - **Cursor**: `.cursor/agents/<id>.md` — YAML frontmatter + Markdown body
//! - **Codex**: `.codex/agents/<id>.toml` — TOML agent config
//! - **OpenCode**: `.opencode/agents/<id>.md` — YAML frontmatter + Markdown body
//!   (OpenCode also supports inline `agent` key in opencode.json; file form
//!   chosen here to parallel the other harnesses.)

use std::collections::HashMap;

use crate::compile::get_harness;
use crate::types::AgentProfile;

/// Compile agent profiles into harness-native agent files.
///
/// Returns a map of `relative_path → file_content` ready for `CompileOutput.agent_files`.
/// Every profile compiles for every harness — agent profiles are harness-agnostic.
/// Per-harness customization lives in `harness_settings.<harness>.<key>`.
pub fn compile_agent_profiles(
    profiles: &[AgentProfile],
    harness_id: &str,
) -> HashMap<String, String> {
    let desc = match get_harness(harness_id) {
        Some(d) => d,
        None => return HashMap::new(),
    };
    let mut out = HashMap::new();
    for profile in profiles {
        let path = match desc.agents_dir.agent_path(&profile.agent.id) {
            Some(p) => p,
            None => continue,
        };
        // Dispatch via the descriptor's fn pointer — no harness-id match.
        let content = (desc.compile_agent)(profile);
        out.insert(path, content);
    }
    out
}

// ─── Claude Code ─────────────────────────────────────────────────────────────
// Format: `.claude/agents/<id>.md`
// Frontmatter: name, description, model, tools, permissionMode, mcpServers, skills

pub(super) fn compile_claude_agent(profile: &AgentProfile) -> String {
    let mut fm = Vec::new();
    // Use id as name so Claude Code can match it as a subagent_type by filename
    fm.push(format!("name: {}", profile.agent.id));
    if let Some(desc) = &profile.agent.description {
        fm.push(format!("description: {}", yaml_quote(desc)));
    }

    // Model from harness_settings.claude or omit (inherit)
    if let Some(model) = claude_setting(profile, "model") {
        fm.push(format!("model: {model}"));
    }

    // Default tools — all tools available unless restricted
    fm.push("tools: \"*\"".to_string());

    // Permission mode
    if let Some(mode) = &profile.permissions.default_mode {
        fm.push(format!("permissionMode: {mode}"));
    }

    // Tools — map deny list to disallowedTools if present
    if !profile.permissions.tools_deny.is_empty() {
        let tools = profile
            .permissions
            .tools_deny
            .iter()
            .map(|t| yaml_quote(t))
            .collect::<Vec<_>>();
        fm.push(format!("disallowedTools:\n{}", yaml_list(&tools)));
    }

    // MCP servers
    if !profile.mcp.servers.is_empty() {
        let servers: Vec<String> = profile.mcp.servers.iter().map(|s| yaml_quote(s)).collect();
        fm.push(format!("mcpServers:\n{}", yaml_list(&servers)));
    }

    // Skills
    if !profile.skills.refs.is_empty() {
        let skills: Vec<String> = profile.skills.refs.iter().map(|s| yaml_quote(s)).collect();
        fm.push(format!("skills:\n{}", yaml_list(&skills)));
    }

    let body = profile.rules.inline.as_deref().unwrap_or_default();
    format!("---\n{}\n---\n\n{}\n", fm.join("\n"), body.trim())
}

// ─── Gemini CLI ──────────────────────────────────────────────────────────────
// Format: `.gemini/agents/<id>.md`
// Frontmatter: name, description, kind, tools, model, max_turns

pub(super) fn compile_gemini_agent(profile: &AgentProfile) -> String {
    let mut fm = Vec::new();
    fm.push(format!("name: {}", profile.agent.id));
    if let Some(desc) = &profile.agent.description {
        fm.push(format!("description: {}", yaml_quote(desc)));
    }
    fm.push("kind: local".to_string());

    // Model
    if let Some(model) = gemini_setting(profile, "model") {
        fm.push(format!("model: {model}"));
    }

    // Tools — Gemini uses tool name list; `*` = all, `mcp_*` = all MCP
    let mut tools = Vec::new();
    if !profile.mcp.servers.is_empty() {
        tools.push("mcp_*".to_string());
    }
    // Default to all tools if no restrictions specified
    if profile.permissions.tools_allow.is_empty() && tools.is_empty() {
        tools.push("*".to_string());
    } else if !profile.permissions.tools_allow.is_empty() {
        tools.extend(profile.permissions.tools_allow.iter().cloned());
    } else {
        tools.push("*".to_string());
    }
    let tool_strs: Vec<String> = tools.iter().map(|t| yaml_quote(t)).collect();
    fm.push(format!("tools:\n{}", yaml_list(&tool_strs)));

    let body = profile.rules.inline.as_deref().unwrap_or_default();
    format!("---\n{}\n---\n\n{}\n", fm.join("\n"), body.trim())
}

// ─── Cursor ──────────────────────────────────────────────────────────────────
// Format: `.cursor/agents/<id>.md`
// Frontmatter: name, description, model

pub(super) fn compile_cursor_agent(profile: &AgentProfile) -> String {
    let mut fm = Vec::new();
    fm.push(format!("name: {}", profile.agent.id));
    if let Some(desc) = &profile.agent.description {
        fm.push(format!("description: {}", yaml_quote(desc)));
    }

    // Model — Cursor uses "fast" | "default" or model name
    if let Some(model) = cursor_setting(profile, "model") {
        fm.push(format!("model: {model}"));
    }

    let body = profile.rules.inline.as_deref().unwrap_or_default();
    format!("---\n{}\n---\n\n{}\n", fm.join("\n"), body.trim())
}

// ─── Codex CLI ───────────────────────────────────────────────────────────────
// Format: `.codex/agents/<id>.toml`
// Schema: name, model, description + config keys

pub(super) fn compile_codex_agent(profile: &AgentProfile) -> String {
    let mut lines = Vec::new();
    lines.push(format!("name = {}", toml_quote(&profile.agent.name)));
    if let Some(desc) = &profile.agent.description {
        lines.push(format!("description = {}", toml_quote(desc)));
    }

    // Model
    if let Some(model) = codex_setting(profile, "model") {
        lines.push(format!("model = {}", toml_quote(&model)));
    }

    // MCP servers
    if !profile.mcp.servers.is_empty() {
        lines.push(String::new());
        for server in &profile.mcp.servers {
            lines.push(format!("[mcp_servers.{}]", server));
            // Codex agent TOML references server by name; actual config comes from main config
            lines.push("enabled = true".to_string());
        }
    }

    lines.join("\n") + "\n"
}

// ─── OpenCode ────────────────────────────────────────────────────────────────
// Format: `.opencode/agents/<id>.md`
// Frontmatter: description (required), mode, model, permission, temperature
// Source: https://opencode.ai/docs/agents
// Ship emits `mode: subagent` so the parent OpenCode harness exposes these
// agents under `@`-autocomplete as subagents (invokable from a primary
// agent), matching the Claude `.claude/agents/<id>.md` semantics.

pub(super) fn compile_opencode_agent(profile: &AgentProfile) -> String {
    let mut fm = Vec::new();
    // `description` is required by OpenCode's schema; fall back to the
    // agent id when no description is set in the profile.
    let desc = profile
        .agent
        .description
        .clone()
        .unwrap_or_else(|| profile.agent.id.clone());
    fm.push(format!("description: {}", yaml_quote(&desc)));
    fm.push("mode: subagent".to_string());

    if let Some(model) = opencode_setting(profile, "model") {
        fm.push(format!("model: {model}"));
    }

    let body = profile.rules.inline.as_deref().unwrap_or_default();
    format!("---\n{}\n---\n\n{}\n", fm.join("\n"), body.trim())
}

// ─── Harness setting helpers ────────────────────────────────────────────────

fn claude_setting(profile: &AgentProfile, key: &str) -> Option<String> {
    harness_setting(profile, "claude", key)
}

fn gemini_setting(profile: &AgentProfile, key: &str) -> Option<String> {
    harness_setting(profile, "gemini", key)
}

fn cursor_setting(profile: &AgentProfile, key: &str) -> Option<String> {
    harness_setting(profile, "cursor", key)
}

fn codex_setting(profile: &AgentProfile, key: &str) -> Option<String> {
    harness_setting(profile, "codex", key)
}

fn opencode_setting(profile: &AgentProfile, key: &str) -> Option<String> {
    harness_setting(profile, "opencode", key)
}

fn harness_setting(profile: &AgentProfile, harness: &str, key: &str) -> Option<String> {
    profile
        .harness_settings
        .get(harness)
        .and_then(|v| v.get(key))
        .and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        })
}

// ─── YAML / TOML formatting helpers ─────────────────────────────────────────

/// Quote a YAML string value. Uses double-quotes if it contains special chars.
pub(super) fn yaml_quote(s: &str) -> String {
    if s.contains(':')
        || s.contains('#')
        || s.contains('\'')
        || s.contains('"')
        || s.contains('\n')
        || s.contains('*')
        || s.starts_with(' ')
        || s.ends_with(' ')
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// Render a YAML list block (each item on its own `  - ` line).
fn yaml_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("  - {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Quote a TOML string value.
fn toml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
