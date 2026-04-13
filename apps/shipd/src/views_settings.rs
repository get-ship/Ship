//! Settings view — minijinja shell + Datastar SSE content patches.
//!
//! Two endpoints:
//!
//! - `GET /api/runtime/view/settings` — full-page HTML shell with tab strip
//!   + embedded initial panel. Bootstraps the SSE subscription.
//! - `GET /api/runtime/view/settings/sse` — patches `#settings-panel` on
//!   workspace/config change events.
//!
//! All config reads go through the `runtime::config` public surface. No
//! mutations happen here: forms are wired to the existing `/api/settings`
//! and `/api/providers` endpoints where applicable, and rendered as
//! `disabled` placeholders otherwise (CLAUDE.md: no placeholders in code,
//! but rendered "not configured" is honest signal).

use axum::extract::{Query, State};
use axum::response::Html;
use runtime::config::{McpServerConfig, ProjectConfig};
use runtime::events::ActorConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SettingsParams {
    #[serde(default)]
    pub tab: Option<String>,
}

impl SettingsParams {
    pub fn active_tab(&self) -> String {
        self.tab
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "workspace".to_string())
    }
}

// ─── Context structs ──────────────────────────────────────────────────────────

#[derive(Serialize, Default)]
struct ProjectView {
    loaded: bool,
    id: String,
    name: String,
    description: String,
}

#[derive(Serialize, Default)]
struct Defaults {
    provider: String,
}

#[derive(Serialize)]
struct AgentView {
    name: String,
    role: String,
    description: String,
    is_active: bool,
}

#[derive(Serialize)]
struct ProviderView {
    id: String,
    binary: String,
    version: Option<String>,
    available: bool,
}

#[derive(Serialize)]
struct McpView {
    name: String,
    command: String,
    url: Option<String>,
    scope: String,
    server_type: String,
    disabled: bool,
}

#[derive(Serialize, Default)]
struct PermissionsView {
    loaded: bool,
    allow: Vec<String>,
    deny: Vec<String>,
}

#[derive(Serialize)]
struct IntegrationView {
    name: String,
    connected: bool,
    hint: String,
}

#[derive(Serialize)]
struct SettingsCtx {
    active_tab: String,
    workspace_available: bool,
    workspace_root: String,
    defaults: Defaults,
    project: ProjectView,
    agents: Vec<AgentView>,
    providers: Vec<ProviderView>,
    mcp_servers: Vec<McpView>,
    permissions: PermissionsView,
    integrations: Vec<IntegrationView>,
    daemon_origin: String,
}

// ─── State helpers (inline per scope rules) ───────────────────────────────────

fn load_project_config() -> Option<ProjectConfig> {
    runtime::config::get_effective_config(None).ok()
}

fn load_mcp_servers() -> Vec<McpServerConfig> {
    runtime::config::list_mcp_servers(None).unwrap_or_default()
}

fn detect_providers() -> Vec<ProviderView> {
    runtime::agent_export::PROVIDERS
        .iter()
        .map(|d| {
            let available = runtime::detect_binary(d.binary);
            let version = if available {
                runtime::detect_version(d.binary)
            } else {
                None
            };
            ProviderView {
                id: d.id.to_string(),
                binary: d.binary.to_string(),
                version,
                available,
            }
        })
        .collect()
}

fn detect_default_provider() -> String {
    runtime::config::get_setting("defaults.provider")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "claude".to_string())
}

fn workspace_root_display() -> String {
    runtime::project::get_global_dir()
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

fn detect_integrations() -> Vec<IntegrationView> {
    vec![
        IntegrationView {
            name: "Telegram".to_string(),
            connected: std::env::var("SHIP_TELEGRAM_TOKEN").is_ok()
                && std::env::var("SHIP_TELEGRAM_CHAT_ID").is_ok(),
            hint: "Set SHIP_TELEGRAM_TOKEN and SHIP_TELEGRAM_CHAT_ID to enable the human gateway."
                .to_string(),
        },
        IntegrationView {
            name: "GitHub".to_string(),
            connected: std::env::var("GITHUB_TOKEN").is_ok(),
            hint: "Set GITHUB_TOKEN for PR/issue integrations.".to_string(),
        },
    ]
}

fn build_ctx(params: &SettingsParams, daemon_origin: String) -> SettingsCtx {
    let active_tab = params.active_tab();
    let cfg = load_project_config();

    let project = cfg
        .as_ref()
        .map(|c| ProjectView {
            loaded: true,
            id: c.id.clone(),
            name: c.name.clone().unwrap_or_default(),
            description: c.description.clone().unwrap_or_default(),
        })
        .unwrap_or_default();

    let agents: Vec<AgentView> = cfg
        .as_ref()
        .map(|c| {
            c.modes
                .iter()
                .map(|m| AgentView {
                    name: m.name.clone(),
                    role: m.role.clone().unwrap_or_else(|| "worker".to_string()),
                    description: m.description.clone().unwrap_or_default(),
                    is_active: c.active_agent.as_deref() == Some(&m.id),
                })
                .collect()
        })
        .unwrap_or_default();

    let mcp_servers: Vec<McpView> = load_mcp_servers()
        .into_iter()
        .map(|s| McpView {
            name: s.name,
            command: s.command,
            url: s.url,
            scope: s.scope,
            server_type: format!("{:?}", s.server_type).to_lowercase(),
            disabled: s.disabled,
        })
        .collect();

    // Permissions: from active agent profile if present, otherwise unloaded.
    let permissions = cfg
        .as_ref()
        .and_then(|c| {
            let active = c.active_agent.as_ref()?;
            c.modes.iter().find(|m| &m.id == active).map(|m| PermissionsView {
                loaded: true,
                allow: m.permissions.allow.clone(),
                deny: m.permissions.deny.clone(),
            })
        })
        .unwrap_or_default();

    SettingsCtx {
        active_tab,
        workspace_available: cfg.is_some(),
        workspace_root: workspace_root_display(),
        defaults: Defaults {
            provider: detect_default_provider(),
        },
        project,
        agents,
        providers: detect_providers(),
        mcp_servers,
        permissions,
        integrations: detect_integrations(),
        daemon_origin,
    }
}

fn render_content(params: &SettingsParams, daemon_origin: String) -> String {
    let ctx = build_ctx(params, daemon_origin);
    templates::render("settings/content.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<div id=\"settings-panel\"><pre>{e}</pre></div>"))
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

/// GET /api/runtime/view/settings/sse — Datastar patches for `#settings-panel`
/// re-rendered on `workspace.config-changed` events.
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
    let config = ActorConfig {
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

        let html = render_content(&params, origin.clone());
        if vs.patch("#settings-panel", html).await.is_err() {
            return;
        }

        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let html = render_content(&params, origin.clone());
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
