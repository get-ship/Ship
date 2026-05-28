// ─── Harness registry ────────────────────────────────────────────────────────
// Authoritative path reference, support matrix, and compatibility dates:
// → crates/core/compiler/HARNESSES.md
// Update that file before changing any HarnessDescriptor field.
//
// This table is the single source of truth for harness onboarding: data
// (paths, MCP keys, feature flags) AND behaviour (agent/MCP/settings
// compilation, carried as fn pointers). Adding a harness is one entry here
// plus its compile module; the compile flow never matches on harness id.

use serde_json::Value as Json;

use super::CompileOutput;
use crate::resolve::ResolvedConfig;
use crate::types::{AgentProfile, McpServerConfig};

/// Compile one agent profile into this harness's native agent-file body.
pub type CompileAgentFn = fn(&AgentProfile) -> String;
/// Build this harness's MCP server config block.
/// Args: descriptor, project MCP servers, resolved ship-home dir, whether this
/// is a daemon-spawned scoped launch (gates the authed-HTTP `ship` entry).
pub type BuildMcpFn =
    fn(&HarnessDescriptor, &[McpServerConfig], Option<&str>, bool) -> Json;
/// Populate this harness's native settings/policy/hook outputs.
pub type BuildSettingsFn = fn(&HarnessDescriptor, &ResolvedConfig, &mut CompileOutput);

/// How MCP servers are keyed in the target config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpKey {
    /// `"mcpServers"` — Claude, Cursor
    McpServers,
    /// `"mcp_servers"` — Codex/OpenAI
    McpServersUnderscored,
    /// `"mcp"` — OpenCode
    Mcp,
}

impl McpKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::McpServers => "mcpServers",
            Self::McpServersUnderscored => "mcp_servers",
            Self::Mcp => "mcp",
        }
    }
}

/// Where the context / system-instructions file is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextFile {
    /// `CLAUDE.md` — Claude Code
    ClaudeMd,
    /// `AGENTS.md` — Codex, Roo, Amp, Goose
    AgentsMd,
    /// Harness does not use a context file
    None,
}

impl ContextFile {
    pub fn file_name(self) -> Option<&'static str> {
        match self {
            Self::ClaudeMd => Some("CLAUDE.md"),
            Self::AgentsMd => Some("AGENTS.md"),
            Self::None => None,
        }
    }
}

/// Where native skill files are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillsDir {
    /// `.claude/skills/<id>/SKILL.md`
    Claude,
    /// `.agents/skills/<id>/SKILL.md` (Codex, Pi, Cursor fallback, universal)
    Agents,
    /// `.cursor/skills/<id>/SKILL.md`
    Cursor,
    /// `.opencode/skills/<id>/SKILL.md`
    OpenCode,
    None,
}

impl SkillsDir {
    pub fn base_path(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some(".claude/skills"),
            Self::Agents => Some(".agents/skills"),
            Self::Cursor => Some(".cursor/skills"),
            Self::OpenCode => Some(".opencode/skills"),
            Self::None => None,
        }
    }
}

/// Where harness-native subagent definition files are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentsDir {
    /// `.claude/agents/<id>.md`
    Claude,
    /// `.codex/agents/<id>.toml`
    Codex,
    /// `.cursor/agents/<id>.md`
    Cursor,
    /// `.opencode/agents/<id>.md` — markdown form (OpenCode also supports
    /// inline definition under `opencode.json` `agent` key; file form is
    /// chosen here to parallel the other harnesses).
    OpenCode,
    None,
}

impl AgentsDir {
    pub fn base_path(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some(".claude/agents"),
            Self::Codex => Some(".codex/agents"),
            Self::Cursor => Some(".cursor/agents"),
            Self::OpenCode => Some(".opencode/agents"),
            Self::None => None,
        }
    }

    pub fn ext(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some("md"),
            Self::Codex => Some("toml"),
            Self::Cursor => Some("md"),
            Self::OpenCode => Some("md"),
            Self::None => None,
        }
    }

    /// Build the project-relative path for an agent file: `<base>/<id>.<ext>`.
    pub fn agent_path(self, id: &str) -> Option<String> {
        let base = self.base_path()?;
        let ext = self.ext()?;
        Some(format!("{base}/{id}.{ext}"))
    }
}

#[derive(Debug, Clone)]
pub struct HarnessDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub mcp_key: McpKey,
    pub context_file: ContextFile,
    pub skills_dir: SkillsDir,
    pub agents_dir: AgentsDir,
    /// Whether to emit `"type"` field in MCP server entries.
    /// All registered harnesses except OpenCode infer transport from field
    /// presence (command → stdio, url → SSE/HTTP); OpenCode opts in via
    /// `"type": "local"/"remote"`.
    pub emit_type_field: bool,
    /// Field name used for SSE transport URL entries ("url" for all current harnesses).
    pub sse_url_field: &'static str,
    /// Field name used for streamable HTTP transport URL entries
    /// ("url" for all current harnesses; parameterised so a future harness
    /// that distinguishes SSE vs HTTP at the field level can plug in here).
    pub http_url_field: &'static str,
    /// Whether this harness can authenticate to a streamable-HTTP MCP server
    /// from static config — i.e. it expresses request `headers` (for the
    /// `Authorization: Bearer` token) AND expands `${ENV}` references in config
    /// values. When true, the self-hosted `ship` server compiles to a
    /// daemon-hosted HTTP entry (`${SHIP_DAEMON_URL}/mcp` + `Bearer
    /// ${SHIP_TOKEN}`); when false it falls back to the local stdio shim.
    /// Verified per harness — default false (stdio) when unconfirmed, so an
    /// unverified harness never silently ships a broken authed-HTTP entry.
    pub supports_authed_http_mcp: bool,
    /// Project-relative path where the MCP config file is written.
    /// `None` when the MCP config is merged into a larger settings file.
    pub mcp_config_path: Option<&'static str>,
    /// What the compiler emits for this harness (MCP/hooks/permissions/
    /// memory). Lives on the descriptor so the one HARNESSES table is the
    /// single source of truth — no `match self.id` to keep in sync.
    pub features: ProviderFeatureFlags,
    /// Compile an agent profile into this harness's native agent file.
    pub compile_agent: CompileAgentFn,
    /// Build this harness's MCP server config (most use the shared builder;
    /// cursor has a per-server-field variant).
    pub build_mcp: BuildMcpFn,
    /// Populate this harness's settings/policy/hook outputs on CompileOutput.
    pub build_settings: BuildSettingsFn,
}

pub(super) static HARNESSES: &[HarnessDescriptor] = &[
    HarnessDescriptor {
        id: "claude",
        name: "Claude Code",
        mcp_key: McpKey::McpServers,
        context_file: ContextFile::ClaudeMd,
        skills_dir: SkillsDir::Claude,
        agents_dir: AgentsDir::Claude,
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        // Verified: .mcp.json supports { "type": "http", "url", "headers" }
        // and expands ${VAR} in values — so it carries Bearer ${SHIP_TOKEN}.
        supports_authed_http_mcp: true,
        // Project-level MCP for Claude Code lives in .mcp.json
        mcp_config_path: Some(".mcp.json"),
        features: ProviderFeatureFlags {
            supports_mcp: true,
            supports_hooks: true,
            supports_tool_permissions: true,
            supports_memory: true,
        },
        compile_agent: super::agents::compile_claude_agent,
        build_mcp: super::mcp::build_mcp_servers,
        build_settings: super::claude::build_settings,
    },
    HarnessDescriptor {
        id: "codex",
        name: "OpenAI Codex",
        // Source: https://developers.openai.com/codex/mcp
        // Codex MCP config is TOML ([mcp_servers.<name>] tables in ~/.codex/config.toml).
        // The mcp_key here reflects the TOML table key; the mcp_servers JSON output
        // is a known limitation — proper TOML serialisation is tracked as future work.
        mcp_key: McpKey::McpServersUnderscored,
        context_file: ContextFile::AgentsMd,
        skills_dir: SkillsDir::Agents,
        agents_dir: AgentsDir::Codex,
        // No "type" field in Codex TOML MCP entries either.
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        // Unverified: TOML config — static request headers + ${ENV} expansion
        // for HTTP MCP not confirmed. Defaults to the stdio shim until verified.
        supports_authed_http_mcp: false,
        mcp_config_path: Some(".codex/config.toml"),
        features: ProviderFeatureFlags {
            supports_mcp: true,
            supports_hooks: true,
            supports_tool_permissions: false,
            supports_memory: true,
        },
        compile_agent: super::agents::compile_codex_agent,
        build_mcp: super::mcp::build_mcp_servers,
        build_settings: super::codex::build_settings,
    },
    HarnessDescriptor {
        id: "cursor",
        name: "Cursor",
        // Source: https://cursor.com/docs/context/skills
        mcp_key: McpKey::McpServers,
        // Cursor uses per-file .mdc rules in .cursor/rules/ — not a single context file
        context_file: ContextFile::None,
        skills_dir: SkillsDir::Cursor,
        agents_dir: AgentsDir::Cursor,
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        // Unverified: mcp.json takes headers for remote servers, but ${ENV}
        // expansion in those values isn't confirmed. Stdio shim until verified.
        supports_authed_http_mcp: false,
        mcp_config_path: Some(".cursor/mcp.json"),
        features: ProviderFeatureFlags {
            supports_mcp: true,
            supports_hooks: true,
            supports_tool_permissions: true,
            supports_memory: false,
        },
        compile_agent: super::agents::compile_cursor_agent,
        build_mcp: super::cursor::build_cursor_mcp_servers,
        build_settings: super::cursor::build_settings,
    },
    HarnessDescriptor {
        id: "opencode",
        name: "OpenCode",
        // Source: https://opencode.ai/docs/config/
        // Config: opencode.json at project root. MCP servers nested under "mcp" key.
        // MCP uses type: "local"/"remote", command as array, "environment" not "env".
        mcp_key: McpKey::Mcp,
        context_file: ContextFile::AgentsMd,
        // Skills discovered from .opencode/skills/<name>/SKILL.md
        skills_dir: SkillsDir::OpenCode,
        // OpenCode supports both inline (opencode.json "agent" key) and
        // file-based (.opencode/agents/<id>.md) agent definitions. Ship
        // emits files to parallel the other four harnesses.
        agents_dir: AgentsDir::OpenCode,
        emit_type_field: true, // OpenCode uses "type": "local"/"remote"
        sse_url_field: "url",
        http_url_field: "url",
        // Unverified: type:"remote" entries take headers, but ${ENV} expansion
        // in opencode.json values isn't confirmed. Stdio shim until verified.
        supports_authed_http_mcp: false,
        // MCP is nested inside opencode.json — no separate file.
        mcp_config_path: None,
        features: ProviderFeatureFlags {
            supports_mcp: true,
            supports_hooks: true,
            supports_tool_permissions: true,
            supports_memory: true,
        },
        compile_agent: super::agents::compile_opencode_agent,
        build_mcp: super::mcp::build_mcp_servers,
        build_settings: super::opencode::build_settings,
    },
    HarnessDescriptor {
        id: "pi",
        name: "Pi",
        // Pi has no built-in MCP; this is unused (supports_mcp = false).
        mcp_key: McpKey::McpServers,
        // Pi reads AGENTS.md for context and .agents/skills for skills —
        // the same universal outputs Ship already emits. Source: pi.dev/docs.
        context_file: ContextFile::AgentsMd,
        skills_dir: SkillsDir::Agents,
        // Pi has no native subagent file format (uses TS extensions instead).
        agents_dir: AgentsDir::None,
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        // Pi has no MCP at all (supports_mcp = false) — it can't do authed HTTP.
        supports_authed_http_mcp: false,
        mcp_config_path: None,
        features: ProviderFeatureFlags {
            // No MCP, no compiled hooks/permissions file; model/tools are
            // CLI/env. Memory = context file (AGENTS.md).
            supports_mcp: false,
            supports_hooks: false,
            supports_tool_permissions: false,
            supports_memory: true,
        },
        compile_agent: super::pi::compile_pi_agent,
        build_mcp: super::mcp::build_mcp_servers,
        build_settings: super::pi::build_settings,
    },
];

/// Feature support flags for a harness.
///
/// These govern what the compiler emits for a given harness:
/// - `supports_mcp` — harness reads an MCP server config file
/// - `supports_hooks` — harness supports session hooks (Stop, PreToolUse, etc.)
/// - `supports_tool_permissions` — harness supports allow/deny tool lists
/// - `supports_memory` — harness has persistent memory / context file
///
/// See SPEC.md §Harness Feature Matrix for the full per-harness table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderFeatureFlags {
    /// Does this harness read an MCP server config file?
    pub supports_mcp: bool,
    /// Does this harness support session hooks?
    pub supports_hooks: bool,
    /// Can you configure allow/deny tool permission lists for this harness?
    pub supports_tool_permissions: bool,
    /// Does this harness have a persistent memory / context file?
    pub supports_memory: bool,
}

impl HarnessDescriptor {
    /// Return the feature flags for this harness. Stored on the descriptor —
    /// adding a harness sets them in the table entry, not in a match arm.
    pub fn feature_flags(&self) -> ProviderFeatureFlags {
        self.features
    }
}

pub fn get_harness(id: &str) -> Option<&'static HarnessDescriptor> {
    HARNESSES.iter().find(|p| p.id == id)
}

pub fn list_harnesses() -> &'static [HarnessDescriptor] {
    HARNESSES
}
