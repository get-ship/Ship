use serde::{Deserialize, Serialize};
use specta::Type;

pub const PRIMARY_CONFIG_FILE: &str = "ship.jsonc";
pub const LEGACY_CONFIG_FILE: &str = "ship.toml";

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct StatusConfig {
    pub id: String,
    pub name: String,
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "gray".to_string()
}

/// Controls which parts of .ship/ are committed to git.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct GitConfig {
    /// Paths/globs that should be gitignored (relative to .ship/).
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Paths/globs that should be committed (relative to .ship/).
    #[serde(default)]
    pub commit: Vec<String>,
}

impl Default for GitConfig {
    fn default() -> Self {
        // Default:
        // - Keep project docs local by default.
        // - Keep core control-plane config and always-on rules tracked.
        Self {
            ignore: Vec::new(),
            commit: vec![
                "ship.jsonc".to_string(),
                "mcp".to_string(),
                "permissions".to_string(),
                "rules".to_string(),
            ],
        }
    }
}

/// Configuration for the AI pass-through CLI.
/// Ship does not call any AI APIs directly — it spawns the configured CLI binary.
/// Supported providers: "claude" (Claude Code CLI), "gemini", "codex".
/// "chatgpt" is still accepted as a backwards-compatible alias.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AiConfig {
    /// Which AI CLI to use. Defaults to "claude".
    pub provider: Option<String>,
    /// Optional model identifier for UI/agent selection context.
    pub model: Option<String>,
    /// Override the binary path if it's not on PATH. Defaults to the provider name.
    pub cli_path: Option<String>,
}

impl AiConfig {
    pub fn effective_provider(&self) -> &str {
        self.provider.as_deref().unwrap_or("claude")
    }

    /// The binary to invoke — cli_path override, or falls back to the provider name.
    pub fn effective_cli(&self) -> &str {
        self.cli_path
            .as_deref()
            .unwrap_or_else(|| self.effective_provider())
    }
}

/// Which lifecycle event triggers a hook.
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookTrigger {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest,
    PostToolUse,
    PostToolUseFailure,
    Notification,
    SubagentStart,
    Stop,
    SubagentStop,
    PreCompact,
    BeforeTool,
    AfterTool,
    BeforeAgent,
    AfterAgent,
    SessionEnd,
    BeforeModel,
    AfterModel,
    BeforeToolSelection,
}

/// A shell command executed on a lifecycle event.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    /// Glob/regex pattern to match tool name (e.g. "Bash", "mcp__*"). Empty = all tools.
    #[serde(default)]
    pub matcher: Option<String>,
    /// Optional timeout for the hook command in milliseconds.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Human-readable intent for editor UX and exported provider config.
    #[serde(default)]
    pub description: Option<String>,
    /// The shell command to run
    pub command: String,
}

/// Mode-scoped tool permission overrides.
/// These overlay canonical `.ship/permissions.jsonc` `tools.allow/deny`
/// when a mode is active.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct PermissionConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AgentLayerConfig {
    /// Skill IDs to load for all sessions in this project.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Legacy alias for global instruction skill IDs.
    /// Prompts are modeled as skills in Ship.
    #[serde(default)]
    pub prompts: Vec<String>,
    /// Context files/folders to preload for agents.
    #[serde(default)]
    pub context: Vec<String>,
}

pub fn is_agent_layer_empty(config: &AgentLayerConfig) -> bool {
    config.skills.is_empty() && config.prompts.is_empty() && config.context.is_empty()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(super) struct LegacyAgentsConfigFile {
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub active_agent: Option<String>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct NamespaceConfig {
    /// Stable namespace id (e.g. "project", "agents", "generated", "plugin:ghost-issues")
    pub id: String,
    /// Directory path relative to `.ship/`
    pub path: String,
    /// Owning module or family (e.g. "project", "agents", "runtime", "plugins")
    pub owner: String,
}

// ─── Artifacts settings ───────────────────────────────────────────────────────

/// Per-scope artifact storage limits.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ArtifactsScopeConfig {
    /// Maximum size per individual artifact in megabytes.
    #[serde(default)]
    pub size_max_mb: Option<u64>,
    /// TTL in hours for run-scope artifacts (only meaningful for `run` scope).
    #[serde(default)]
    pub ttl_hours: Option<u64>,
    /// Total storage cap for the scope in gigabytes.
    #[serde(default)]
    pub total_max_gb: Option<u64>,
}

impl Default for ArtifactsScopeConfig {
    fn default() -> Self {
        Self {
            size_max_mb: None,
            ttl_hours: None,
            total_max_gb: None,
        }
    }
}

/// Sync options (phase-3 placeholder — `enabled: false` in phase 1).
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct ArtifactsSyncConfig {
    #[serde(default)]
    pub enabled: bool,
}

/// Top-level `artifacts` block in ship.jsonc.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ArtifactsConfig {
    #[serde(default)]
    pub run: ArtifactsScopeConfig,
    #[serde(default)]
    pub project: ArtifactsScopeConfig,
    #[serde(default)]
    pub user: ArtifactsScopeConfig,
    /// MIME types to allow. `null` means all types are allowed.
    #[serde(default)]
    pub mime_allowlist: Option<Vec<String>>,
    /// MIME types to reject.
    #[serde(default)]
    pub mime_denylist: Vec<String>,
    #[serde(default)]
    pub sync: ArtifactsSyncConfig,
}

impl Default for ArtifactsConfig {
    fn default() -> Self {
        Self {
            run: ArtifactsScopeConfig {
                ttl_hours: Some(24),
                size_max_mb: Some(50),
                total_max_gb: None,
            },
            project: ArtifactsScopeConfig {
                size_max_mb: Some(100),
                total_max_gb: Some(5),
                ttl_hours: None,
            },
            user: ArtifactsScopeConfig {
                size_max_mb: Some(100),
                total_max_gb: Some(10),
                ttl_hours: None,
            },
            mime_allowlist: None,
            mime_denylist: vec![
                "application/x-msdownload".into(),
                "application/x-executable".into(),
            ],
            sync: ArtifactsSyncConfig { enabled: false },
        }
    }
}

impl ArtifactsConfig {
    /// Validate settings; return an error string describing the first invalid
    /// value found. Called at daemon startup.
    pub fn validate(&self) -> Result<(), String> {
        fn check_scope(name: &str, cfg: &ArtifactsScopeConfig) -> Result<(), String> {
            if let Some(mb) = cfg.size_max_mb {
                if mb == 0 {
                    return Err(format!("artifacts.{name}.size_max_mb must be > 0"));
                }
            }
            if let Some(gb) = cfg.total_max_gb {
                if gb == 0 {
                    return Err(format!("artifacts.{name}.total_max_gb must be > 0"));
                }
            }
            if let Some(h) = cfg.ttl_hours {
                if h == 0 {
                    return Err(format!("artifacts.{name}.ttl_hours must be > 0"));
                }
            }
            Ok(())
        }
        check_scope("run", &self.run)?;
        check_scope("project", &self.project)?;
        check_scope("user", &self.user)?;
        Ok(())
    }

    /// TTL for run-scope artifacts in hours (default 24).
    pub fn run_ttl_hours(&self) -> u64 {
        self.run.ttl_hours.unwrap_or(24)
    }

    /// Size cap for the given scope in bytes, or None if uncapped.
    pub fn size_max_bytes(&self, scope: &str) -> Option<u64> {
        let mb = match scope {
            "run" => self.run.size_max_mb,
            "project" => self.project.size_max_mb,
            "user" => self.user.size_max_mb,
            _ => None,
        }?;
        Some(mb * 1024 * 1024)
    }
}

#[cfg(test)]
mod artifacts_config_tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let cfg = ArtifactsConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.run_ttl_hours(), 24);
        assert_eq!(cfg.size_max_bytes("run"), Some(50 * 1024 * 1024));
        assert_eq!(cfg.size_max_bytes("project"), Some(100 * 1024 * 1024));
        assert_eq!(cfg.size_max_bytes("user"), Some(100 * 1024 * 1024));
    }

    #[test]
    fn parses_from_json() {
        let json = r#"{
            "run": { "ttl_hours": 48, "size_max_mb": 20 },
            "project": { "size_max_mb": 200, "total_max_gb": 10 },
            "user": {},
            "sync": { "enabled": true }
        }"#;
        let cfg: ArtifactsConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.run.ttl_hours, Some(48));
        assert_eq!(cfg.run.size_max_mb, Some(20));
        assert_eq!(cfg.project.total_max_gb, Some(10));
        assert!(cfg.sync.enabled);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn zero_size_max_rejected() {
        let mut cfg = ArtifactsConfig::default();
        cfg.run.size_max_mb = Some(0);
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn zero_ttl_hours_rejected() {
        let mut cfg = ArtifactsConfig::default();
        cfg.run.ttl_hours = Some(0);
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn absent_artifacts_block_uses_defaults() {
        let json = r#"{}"#;
        let cfg: ArtifactsConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.run_ttl_hours(), 24);
    }
}

// Used as a serde default in project.rs — must be pub.
pub fn default_namespaces() -> Vec<NamespaceConfig> {
    vec![
        NamespaceConfig {
            id: "project".to_string(),
            path: "project".to_string(),
            owner: "project".to_string(),
        },
        NamespaceConfig {
            id: "agents".to_string(),
            path: "agents".to_string(),
            owner: "agents".to_string(),
        },
    ]
}
