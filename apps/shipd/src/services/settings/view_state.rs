//! Settings view state — ctx builder for the Settings view template.
//! Called by the `settings.state` registry op; no HTTP handlers.

use serde::Serialize;

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ProviderRow {
    /// Stable id (e.g. "claude", "openai", "gemini").
    pub id: String,
    /// Human label.
    pub label: String,
    /// Single-glyph emblem to stand in for a logo.
    pub glyph: String,
    /// CLI binary we probe for.
    pub binary: String,
    /// Detected CLI version, if available.
    pub version: Option<String>,
    /// Whether the CLI is installed on PATH.
    pub available: bool,
    /// Whether this is the default harness per runtime settings.
    pub is_default: bool,
}

#[derive(Serialize, Default, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TerminalConfig {
    pub program: String,
    pub worktree_base: String,
    pub options: Vec<SelectOption>,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FeatureFlag {
    pub key: String,
    pub label: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Serialize, Default, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AboutInfo {
    pub ship_version: String,
    pub daemon_version: String,
    pub cli_version: String,
    pub license: String,
    pub homepage: String,
    pub docs: String,
    pub source: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SettingsCtx {
    pub harnesses: Vec<ProviderRow>,
    pub default_harness: String,
    pub default_model: String,
    pub terminal: TerminalConfig,
    pub advanced: Vec<FeatureFlag>,
    pub advanced_registry: String,
    pub daemon_endpoint: String,
    pub about: AboutInfo,
    pub harness_default: String,
    pub harness_default_model: String,
    pub agent_session_auto_resume: bool,
    /// Stringified for the `<sui-input>` value binding — the SUI compiler
    /// rejects scalar coercions like `String(...)` in renderable
    /// expressions, so number-shaped settings are serialised as strings.
    pub watcher_debounce_ms: String,
    pub git_log_limit: String,
    pub hooks_max_depth: String,
}

fn harness_meta(id: &str) -> (&'static str, &'static str) {
    match id {
        "claude" => ("Anthropic Claude", "\u{2726}"),
        "openai" | "codex" => ("OpenAI", "\u{25C6}"),
        "gemini" => ("Google Gemini", "\u{25C7}"),
        "grok" => ("xAI Grok", "\u{25C8}"),
        "opencode" => ("OpenCode", "\u{25C9}"),
        _ => ("Harness", "\u{25CB}"),
    }
}

fn detect_harnesses(default_id: &str) -> Vec<ProviderRow> {
    // Availability-only: `which <binary>` is sub-millisecond. Version
    // detection spawns the binary and blocks the render — keep off the
    // hot path.
    ship_platform::agents::harnesses::HARNESSES
        .iter()
        .map(|d| {
            let available = ship_platform::agents::harnesses::detect_binary(d.binary);
            let (label, glyph) = harness_meta(d.id);
            ProviderRow {
                id: d.id.to_string(),
                label: label.to_string(),
                glyph: glyph.to_string(),
                binary: d.binary.to_string(),
                version: None,
                available,
                is_default: d.id == default_id,
            }
        })
        .collect()
}

fn setting_string(key: &str) -> String {
    ship_platform::config::get_setting(key)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

fn build_terminal() -> TerminalConfig {
    TerminalConfig {
        program: setting_string("terminal.program"),
        worktree_base: setting_string("worktree.base_dir"),
        options: vec![
            ("tmux", "tmux"),
            ("iterm", "iTerm"),
            ("wezterm", "WezTerm"),
            ("alacritty", "Alacritty"),
            ("terminal", "Terminal.app"),
            ("wt", "Windows Terminal"),
            ("gnome", "GNOME Terminal"),
            ("vscode", "VS Code"),
        ]
        .into_iter()
        .map(|(v, l)| SelectOption {
            value: v.to_string(),
            label: l.to_string(),
        })
        .collect(),
    }
}

fn build_about() -> AboutInfo {
    let v = env!("CARGO_PKG_VERSION").to_string();
    AboutInfo {
        ship_version: v.clone(), daemon_version: v.clone(), cli_version: v,
        license: "MIT".into(), homepage: "https://getship.dev".into(),
        docs: "https://getship.dev/docs".into(),
        source: "https://github.com/madvib/Ship".into(),
    }
}

pub fn build_ctx() -> SettingsCtx {
    let default_harness = ship_platform::config::get_setting("defaults.harness")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "claude".to_string());
    let default_model = setting_string("defaults.model");
    let harnesses = detect_harnesses(&default_harness);
    let advanced_registry = setting_string("registry.endpoint");
    let daemon_port = setting_string("daemon.port");
    let daemon_endpoint = if daemon_port.is_empty() {
        "http://localhost:9315".to_string()
    } else {
        format!("http://localhost:{}", daemon_port)
    };

    let harness_default = setting_string("harness.default");
    let harness_default_model = setting_string("harness.default_model");
    let agent_session_auto_resume = ship_platform::config::resolve_bool(
        "agent_session.auto_resume",
        None,
        &ship_platform::config::ProjectConfig::default(),
    );
    let watcher_debounce_ms =
        ship_platform::config::get_setting_u64_or("watcher.debounce_ms", 200).to_string();
    let git_log_limit =
        ship_platform::config::get_setting_u64_or("git.log_limit", 50).to_string();
    let hooks_max_depth =
        (ship_platform::config::get_setting_u64_or("hooks.max_depth", 3) as u32).to_string();

    SettingsCtx {
        harnesses,
        default_harness,
        default_model,
        terminal: build_terminal(),
        advanced: Vec::new(),
        advanced_registry,
        daemon_endpoint,
        about: build_about(),
        harness_default,
        harness_default_model,
        agent_session_auto_resume,
        watcher_debounce_ms,
        git_log_limit,
        hooks_max_depth,
    }
}
