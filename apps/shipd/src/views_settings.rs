//! Settings view — shell + Datastar SSE patches.
//!
//! Templates now live in `.ship/views/settings/` with fallback to
//! embedded `apps/shipd/templates/settings/`. Rust handlers stay here
//! for context-building (providers, terminal, flags, about).

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use runtime::gen_ulid;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SettingsParams {}

#[derive(Serialize, Clone)]
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
    /// Whether this is the default provider per runtime settings.
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
pub struct SettingsCtx {
    pub providers: Vec<ProviderRow>,
    pub default_provider: String,
    pub default_model: String,
    pub terminal: TerminalConfig,
    pub advanced: Vec<FeatureFlag>,
    pub advanced_registry: String,
    pub daemon_endpoint: String,
    pub about: AboutInfo,
}

// ─── State helpers ────────────────────────────────────────────────────────────

fn provider_meta(id: &str) -> (&'static str, &'static str) {
    match id {
        "claude" => ("Anthropic Claude", "\u{2726}"),
        "openai" | "codex" => ("OpenAI", "\u{25C6}"),
        "gemini" => ("Google Gemini", "\u{25C7}"),
        "grok" => ("xAI Grok", "\u{25C8}"),
        "opencode" => ("OpenCode", "\u{25C9}"),
        _ => ("Provider", "\u{25CB}"),
    }
}

fn detect_providers(default_id: &str) -> Vec<ProviderRow> {
    // Availability-only: `which <binary>` is sub-millisecond. Version
    // detection spawns the binary (~1–2s each) and blocks the shell render;
    // defer it to the SSE path when/if we want to show versions live.
    runtime::agent_export::PROVIDERS
        .iter()
        .map(|d| {
            let available = runtime::detect_binary(d.binary);
            let (label, glyph) = provider_meta(d.id);
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
    runtime::config::get_setting(key)
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

pub fn build_ctx(_params: &SettingsParams) -> SettingsCtx {
    let default_provider = runtime::config::get_setting("defaults.provider")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "claude".to_string());
    let default_model = setting_string("defaults.model");
    let providers = detect_providers(&default_provider);
    let advanced_registry = setting_string("registry.endpoint");
    let daemon_port = setting_string("daemon.port");
    let daemon_endpoint = if daemon_port.is_empty() {
        "http://localhost:9315".to_string()
    } else {
        format!("http://localhost:{}", daemon_port)
    };

    SettingsCtx {
        providers,
        default_provider,
        default_model,
        terminal: build_terminal(),
        advanced: Vec::new(),
        advanced_registry,
        daemon_endpoint,
        about: build_about(),
    }
}

fn render_panel(params: &SettingsParams) -> Option<String> {
    let ctx = build_ctx(params);
    match render_content_partial(&ctx) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(error = %e, "settings panel render failed");
            None
        }
    }
}

/// Render content partial from `.ship/views/settings/` or fall back to embedded partials.
fn render_content_partial(ctx: &SettingsCtx) -> Result<String, String> {
    if let Some(view_dir) = resolve_settings_view_dir() {
        let partial = view_dir.join("partials/content.html.j2");
        if let Ok(source) = std::fs::read_to_string(&partial) {
            return templates::render_view_source("settings/partials/content", &source, ctx);
        }
    }
    // Fallback: compose from embedded templates.
    let mut out = String::new();
    for tpl in [
        "settings/partials/providers.html.j2",
        "settings/partials/terminal.html.j2",
        "settings/partials/advanced.html.j2",
        "settings/partials/about.html.j2",
    ] {
        out.push_str(&templates::render(tpl, ctx)?);
    }
    Ok(out)
}

/// Locate `.ship/views/settings/` in project or global dir.
fn resolve_settings_view_dir() -> Option<std::path::PathBuf> {
    if let Ok(d) = runtime::project::get_project_dir(None) {
        let c = d.join("views/settings");
        if c.is_dir() { return Some(c); }
    }
    if let Ok(d) = runtime::project::get_global_dir() {
        let c = d.join("views/settings");
        if c.is_dir() { return Some(c); }
    }
    None
}

/// GET /api/runtime/view/settings — full shell, minijinja-rendered.
///
/// Reads from `.ship/views/settings/index.html.j2` when available, falls
/// back to the embedded `settings/shell.html.j2` template.
pub async fn settings_shell(
    State(_state): State<crate::rest_api::ApiState>,
    Query(params): Query<SettingsParams>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_ctx(&params);
    let result = if let Some(view_dir) = resolve_settings_view_dir() {
        let index = view_dir.join("index.html.j2");
        if let Ok(source) = std::fs::read_to_string(&index) {
            templates::render_view_source("settings/index", &source, &ctx)
        } else {
            templates::render("settings/shell.html.j2", &ctx)
        }
    } else {
        templates::render("settings/shell.html.j2", &ctx)
    };
    result.map(Html).map_err(|e| {
        tracing::error!(error = %e, "settings shell render failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// GET /api/runtime/view/settings/sse — Datastar patches for `#settings-panel`.
pub async fn settings_sse(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<SettingsParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();

    let actor_id = format!("settings-sse-{}", gen_ulid());
    let config = runtime::events::ActorConfig {
        namespace: actor_id.clone(), write_namespaces: vec![],
        read_namespaces: vec![], subscribe_namespaces: vec!["workspace.".into()],
    };
    let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
        Ok((_store, mb)) => Some(mb),
        Err(e) => {
            tracing::error!(error = %e, actor = %actor_id, "spawn_actor failed for settings SSE");
            None
        }
    };
    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        if let Some(html) = render_panel(&params) {
            if vs.patch("#settings-content", html).await.is_err() {
                return;
            }
        }

        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let Some(html) = render_panel(&params) else { continue };
            if vs.patch("#settings-content", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

#[cfg(test)]
#[path = "views_settings_tests.rs"]
mod tests;
