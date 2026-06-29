use serde::{Deserialize, Serialize};

/// An agent parsed from `.ship/agents/<id>.jsonc`.
///
/// Defines a specialist agent that the compiler emits as a
/// harness-native subagent definition (`.claude/agents/`,
/// `.codex/agents/`, `.opencode/agents/`). The
/// canonical schema lives in `schemas/agent.schema.json` — every
/// JSONC field declared there should have a matching Rust field
/// here. The top-level key is `agent`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent: ProfileMeta,
    /// Authority grants for the ship-op control plane (`{domain, blast}` pairs).
    /// Gate-enforced; replaces the dying `role`/`tools_preset`. C0/C1 are an
    /// implicit baseline; C3 is operator-only and never agent-settable.
    #[serde(default)]
    pub capabilities: Vec<super::capability::Capability>,
    #[serde(default)]
    pub skills: SkillRefs,
    #[serde(default)]
    pub mcp: McpRefs,
    #[serde(default)]
    pub plugins: PluginRefs,
    #[serde(default)]
    pub permissions: ProfilePermissions,
    #[serde(default)]
    pub rules: ProfileRules,
    #[serde(default)]
    pub hooks: ProfileHooks,
    /// App integrations for this agent profile.
    #[serde(default)]
    pub apps: ProfileApps,
    /// Harness-specific settings merged verbatim into the harness's config
    /// file. `harness_settings.claude` → `.claude/settings.json`. Any
    /// key/value valid in that file works here — no code change required.
    #[serde(default)]
    pub harness_settings: std::collections::HashMap<String, serde_json::Value>,
}

/// Declares which Ship apps an agent integrates with.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileApps {
    /// When true, the compiler emits an HTTP MCP transport pointing at Studio's
    /// local endpoint instead of a stdio `ship mcp serve` entry.
    #[serde(default)]
    pub studio: bool,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Role for tool gating defaults. Maps to a `tools_preset` when unset:
    ///   worker → preset "worker"
    ///   lead | commander → preset "orchestrator"
    /// `tools_preset` takes precedence when both are set.
    #[serde(default)]
    pub role: Option<String>,
    /// Searchable tags for registry discovery (e.g. workflow, review, testing).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Tool preset name controlling which platform-shipped tools this agent
    /// sees. Known values: `worker`, `orchestrator`. When unset, falls back
    /// to the `role`-based default.
    #[serde(default)]
    pub tools_preset: Option<String>,
    /// Explicit tool IDs visible to this agent. When non-empty, replaces the
    /// preset's tool list (escape hatch for custom agents).
    #[serde(default)]
    pub active_tools: Vec<String>,
    /// Display icon — emoji or icon name (e.g. "🧪", "test-tube").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillRefs {
    #[serde(default)]
    pub refs: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpRefs {
    #[serde(default)]
    pub servers: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginRefs {
    #[serde(default)]
    pub install: Vec<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfilePermissions {
    /// Permission preset name (e.g. "ship-standard", "ship-autonomous").
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub tools_allow: Vec<String>,
    #[serde(default)]
    pub tools_ask: Vec<String>,
    #[serde(default)]
    pub tools_deny: Vec<String>,
    /// Permission mode. One of the five authoritative modes (see
    /// `permissions.schema.json` / `agent.schema.json`): "default", "plan",
    /// "acceptEdits", "dontAsk", "bypassPermissions". The schema is the
    /// validating contract; this stays an untyped `Option<String>`.
    #[serde(default)]
    pub default_mode: Option<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileRules {
    /// Inline rules body — becomes the agent's system prompt.
    #[serde(default)]
    pub inline: Option<String>,
}

/// Hook commands declared on the agent. Compile emits each into the active
/// harness's hook registration (e.g. `.claude/settings.json:hooks`).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileHooks {
    /// Command to run when the agent session ends (Claude Code Stop hook).
    #[serde(default)]
    pub stop: Option<String>,
    /// Command to run when a sub-agent session ends (SubagentStop hook).
    #[serde(default)]
    pub subagent_stop: Option<String>,
}

impl AgentProfile {
    /// Project this nested file-format profile to the flat runtime form
    /// the resolver and harness compilers consume (`compiler::AgentConfig`).
    /// This is the single canonical nested → flat mapping.
    pub fn to_config(&self) -> super::AgentConfig {
        use super::config::PermissionConfig;
        super::AgentConfig {
            id: self.agent.id.clone(),
            name: self.agent.name.clone(),
            description: self.agent.description.clone(),
            role: self.agent.role.clone(),
            tools_preset: self.agent.tools_preset.clone(),
            active_tools: self.agent.active_tools.clone(),
            // Floki D2b: thread capabilities through — do NOT cement the loss.
            capabilities: self.capabilities.clone(),
            mcp_servers: self.mcp.servers.clone(),
            skills: self.skills.refs.clone(),
            rules: Vec::new(),
            hooks: Vec::new(),
            permissions: PermissionConfig::default(),
            target_agents: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_jsonc(src: &str) -> AgentProfile {
        let stripped = ship_substrate_strip(src);
        serde_json::from_str(&stripped).expect("parse")
    }

    // Local strip wrapper — compiler is leaf-ish and can't pull substrate
    // (pure-transform constraint). Tests run native, so we mimic the canonical
    // stripper's behaviour with a tiny inline copy via crate::jsonc.
    fn ship_substrate_strip(src: &str) -> String {
        crate::jsonc::strip_jsonc_comments(src)
    }

    #[test]
    fn round_trip_full_profile() {
        let src = r#"{
  "agent": {
    "id": "code-reviewer",
    "name": "Code Reviewer",
    "version": "1.2.0",
    "description": "Reviews pull requests for style, correctness, and test coverage.",
    "role": "worker",
    "tags": ["review", "testing"]
  },
  "skills": { "refs": ["review-pr", "lint-fix", "test-runner"] },
  "mcp": { "servers": ["github", "linear"] },
  "plugins": { "install": ["eslint-plugin"], "scope": "project" },
  "permissions": {
    "preset": "ship-standard",
    "tools_allow": ["Read", "Grep", "Bash"],
    "tools_ask": ["Edit", "Write"],
    "tools_deny": ["WebFetch"],
    "default_mode": "acceptEdits"
  },
  "rules": { "inline": "You are a senior code reviewer.\nFocus on correctness." },
  "hooks": { "stop": "ship session end" },
  "harness_settings": {
    "claude": { "model": "opus", "temperature": 0.2 }
  }
}"#;
        let profile = parse_jsonc(src);

        assert_eq!(profile.agent.id, "code-reviewer");
        assert_eq!(profile.agent.role.as_deref(), Some("worker"));
        assert_eq!(profile.agent.tags, vec!["review", "testing"]);
        assert_eq!(profile.skills.refs, vec!["review-pr", "lint-fix", "test-runner"]);
        assert_eq!(profile.mcp.servers, vec!["github", "linear"]);
        assert_eq!(profile.permissions.preset.as_deref(), Some("ship-standard"));
        assert_eq!(profile.hooks.stop.as_deref(), Some("ship session end"));
        let claude = profile.harness_settings.get("claude").expect("claude");
        assert_eq!(claude.get("model").and_then(|v| v.as_str()), Some("opus"));
    }

    #[test]
    fn legacy_profile_key_is_rejected() {
        // The `profile` alias was removed; only `agent` is accepted.
        let src = r#"{
  "profile": { "id": "legacy", "name": "Legacy" }
}"#;
        assert!(serde_json::from_str::<AgentProfile>(src).is_err());
    }

    #[test]
    fn apps_studio_parses_and_defaults_false() {
        let src = r#"{
  "agent": { "id": "design-agent", "name": "Design Agent" },
  "apps": { "studio": true }
}"#;
        let profile = parse_jsonc(src);
        assert!(profile.apps.studio);

        let src2 = r#"{ "agent": { "id": "headless", "name": "Headless" } }"#;
        let profile2: AgentProfile = serde_json::from_str(src2).expect("parse");
        assert!(!profile2.apps.studio);
    }

    #[test]
    fn minimal_profile_round_trips() {
        let src = r#"{ "agent": { "id": "basic", "name": "Basic" } }"#;
        let profile: AgentProfile = serde_json::from_str(src).expect("parse");
        assert_eq!(profile.agent.id, "basic");
        assert_eq!(profile.agent.version, None);
        assert!(profile.skills.refs.is_empty());
        assert!(profile.mcp.servers.is_empty());
        assert!(profile.permissions.tools_allow.is_empty());
        assert!(profile.harness_settings.is_empty());
    }

    #[test]
    fn to_config_projects_nested_to_flat() {
        let src = r#"{
  "agent": { "id": "rev", "name": "Reviewer", "role": "worker" },
  "skills": { "refs": ["a", "b"] },
  "mcp": { "servers": ["github"] }
}"#;
        let profile: AgentProfile = serde_json::from_str(src).expect("parse");
        let config = profile.to_config();
        assert_eq!(config.id, "rev");
        assert_eq!(config.name, "Reviewer");
        assert_eq!(config.role.as_deref(), Some("worker"));
        assert_eq!(config.skills, vec!["a", "b"]);
        assert_eq!(config.mcp_servers, vec!["github"]);
        assert!(config.target_agents.is_empty());
    }
}
