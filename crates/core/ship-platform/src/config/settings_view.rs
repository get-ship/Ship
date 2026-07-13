//! Settings view state — ctx builder for the Settings view template.
//! Relocated from `shipd::services::settings::view_state` (L2 → L1).
//! The view-rendering `SettingsCtx` here is distinct from
//! `ship_config::SettingsCtx` (which is the service context with
//! kernel/account_store/space/is_operator). This one is a serializable
//! bag of display data for the Settings page.

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ProviderRow {
    /// Stable id (e.g. "claude", "codex", "cursor").
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
pub struct TerminalConfig {
    pub program: String,
    pub worktree_base: String,
    pub options: Vec<SelectOption>,
}

#[derive(Serialize, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Serialize, Clone)]
pub struct FeatureFlag {
    pub key: String,
    pub label: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Serialize, Default, Clone)]
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
pub struct SettingsViewCtx {
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
        "grok" => ("xAI Grok", "\u{25C8}"),
        "opencode" => ("OpenCode", "\u{25C9}"),
        _ => ("Harness", "\u{25CB}"),
    }
}

// A2: `agent_def::harnesses` (the manifest-CRUD chain to `ship.jsonc`)
// died with the dot-ship close-out. The Settings page's provider rows
// are a flat (id, label, glyph, binary) projection — synthesize the
// provider list here from the typed `HARNESSES` registry in
// `harness_config_compiler::compile::harness`. The compiler keeps the runtime source
// of truth for which harnesses exist; the settings page just lists
// them with metadata.
fn detect_harnesses(default_id: &str) -> Vec<ProviderRow> {
    // Static provider table mirroring the `harness_config_compiler::compile::harness`
    // runtime registry. Kept narrow (Settings view only) — the runtime
    // registry is the source of truth at dispatch time.
    const PROVIDERS: &[(&str, &str)] = &[
        ("claude", "claude"),
        ("codex", "codex"),
        ("cursor", "cursor"),
        ("opencode", "opencode"),
    ];
    let find_bin = |bin: &str| -> Option<std::path::PathBuf> {
        let path = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path) {
            let p = dir.join(bin);
            if p.is_file() {
                return Some(p);
            }
        }
        None
    };
    PROVIDERS
        .iter()
        .map(|(id, bin)| {
            let (label, glyph) = harness_meta(id);
            ProviderRow {
                id: (*id).to_string(),
                label: label.to_string(),
                glyph: glyph.to_string(),
                binary: (*bin).to_string(),
                version: None,
                available: find_bin(bin).is_some(),
                is_default: *id == default_id,
            }
        })
        .collect()
}

fn setting_string(key: &str) -> String {
    ship_config::settings::get_setting(key)
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
        source: "https://github.com/<ship-repo-org>/ship".into(),
    }
}

pub fn build_ctx() -> SettingsViewCtx {
    let default_harness = ship_config::settings::get_setting("defaults.harness")
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
    let agent_session_auto_resume =
        ship_config::settings::resolve_bool("agent_session.auto_resume", None);
    let watcher_debounce_ms =
        ship_config::settings::get_setting_u64_or("watcher.debounce_ms", 200).to_string();
    let git_log_limit =
        ship_config::settings::get_setting_u64_or("git.log_limit", 50).to_string();
    let hooks_max_depth =
        (ship_config::settings::get_setting_u64_or("hooks.max_depth", 3) as u32).to_string();

    SettingsViewCtx {
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
