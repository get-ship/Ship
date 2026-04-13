//! Settings view — minijinja shell + Datastar SSE content patches.
//!
//! Single endpoint pair:
//!
//! - `GET /api/runtime/view/settings` — full-page HTML shell, single column,
//!   four stacked sections (Providers / Terminal / Advanced / About).
//! - `GET /api/runtime/view/settings/sse` — patches `#settings-panel` on
//!   workspace / config-changed events.
//!
//! All reads go through the `runtime::config` public surface. Writes happen
//! via the existing `/api/settings` REST endpoint that forms post to.

use axum::extract::{Query, State};
use axum::response::Html;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SettingsParams {
    #[serde(default)]
    pub token: String,
}

// ─── Context structs ──────────────────────────────────────────────────────────

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
    pub about: AboutInfo,
    pub daemon_origin: String,
    pub token: String,
}

// ─── State helpers ────────────────────────────────────────────────────────────

fn provider_meta(id: &str) -> (&'static str, &'static str) {
    // (human label, glyph)
    match id {
        "claude" => ("Anthropic Claude", "✦"),
        "openai" | "codex" => ("OpenAI", "◆"),
        "gemini" => ("Google Gemini", "◇"),
        "grok" => ("xAI Grok", "◈"),
        "opencode" => ("OpenCode", "◉"),
        _ => ("Provider", "○"),
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
    let ship_version = env!("CARGO_PKG_VERSION").to_string();
    AboutInfo {
        ship_version: ship_version.clone(),
        daemon_version: ship_version.clone(),
        cli_version: ship_version,
        license: "MIT".to_string(),
        homepage: "https://getship.dev".to_string(),
        docs: "https://getship.dev/docs".to_string(),
        source: "https://github.com/madvib/Ship".to_string(),
    }
}

fn build_ctx(params: &SettingsParams, daemon_origin: String) -> SettingsCtx {
    let default_provider = runtime::config::get_setting("defaults.provider")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "claude".to_string());
    let default_model = setting_string("defaults.model");
    let providers = detect_providers(&default_provider);
    let advanced_registry = setting_string("registry.endpoint");

    SettingsCtx {
        providers,
        default_provider,
        default_model,
        terminal: build_terminal(),
        advanced: Vec::new(),
        advanced_registry,
        about: build_about(),
        daemon_origin,
        token: params.token.clone(),
    }
}

fn render_panel(params: &SettingsParams, daemon_origin: String) -> String {
    let ctx = build_ctx(params, daemon_origin);
    let inner = render_sections(&ctx);
    format!("<div id=\"settings-panel\">{inner}</div>")
}

fn render_sections(ctx: &SettingsCtx) -> String {
    let mut out = String::new();
    for tpl in [
        "settings/partials/providers.html.j2",
        "settings/partials/terminal.html.j2",
        "settings/partials/advanced.html.j2",
        "settings/partials/about.html.j2",
    ] {
        match templates::render(tpl, ctx) {
            Ok(s) => out.push_str(&s),
            Err(e) => out.push_str(&format!("<pre>{e}</pre>")),
        }
    }
    out
}

/// GET /api/runtime/view/settings — full shell, minijinja-rendered.
pub async fn settings_shell(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<SettingsParams>,
) -> Html<String> {
    let ctx = build_ctx(&params, state.daemon_origin());
    let html = templates::render("settings/shell.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<pre>shell render error: {e}</pre>"));
    Html(html)
}

/// GET /api/runtime/view/settings/sse — Datastar patches for `#settings-panel`.
pub async fn settings_sse(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<SettingsParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();

    let actor_id = format!(
        "settings-sse-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let config = runtime::events::ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["workspace.".to_string()],
    };
    let mailbox = state
        .kernel()
        .lock()
        .await
        .spawn_actor(&actor_id, config)
        .ok()
        .map(|(_store, mb)| mb);
    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };
    let origin = state.daemon_origin();

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        let html = render_panel(&params, origin.clone());
        if vs.patch("#settings-panel", html).await.is_err() {
            return;
        }

        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let html = render_panel(&params, origin.clone());
            if vs.patch("#settings-panel", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

#[cfg(test)]
#[path = "views_settings_tests.rs"]
mod tests;
