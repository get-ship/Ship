//! Tests for the settings view context + template rendering.
//!
//! These synthesize `SettingsCtx` directly and call `templates::render` — no
//! `ApiState`, no runtime.

use super::*;
use crate::views::templates;

fn empty_ctx(active_tab: &str) -> SettingsCtx {
    SettingsCtx {
        active_tab: active_tab.to_string(),
        workspace_available: false,
        workspace_root: String::new(),
        defaults: Defaults {
            provider: "claude".into(),
        },
        project: ProjectView::default(),
        agents: vec![],
        providers: vec![],
        mcp_servers: vec![],
        permissions: PermissionsView::default(),
        integrations: vec![],
        daemon_origin: "http://localhost:9315".to_string(),
    }
}

#[test]
fn ctx_defaults_to_workspace_tab() {
    let params = SettingsParams::default();
    assert_eq!(params.active_tab(), "workspace");
}

#[test]
fn params_flatten_from_query() {
    let json = serde_json::json!({ "tab": "providers" });
    let params: SettingsParams = serde_json::from_value(json).expect("parse");
    assert_eq!(params.active_tab(), "providers");
}

#[test]
fn shell_renders_as_fragment_with_root_id_and_tabs() {
    let ctx = empty_ctx("workspace");
    let html = templates::render("settings/shell.html.j2", &ctx).expect("render");
    assert!(!html.contains("<!DOCTYPE"), "shell must be a fragment: {html}");
    assert!(!html.contains("<html"), "no <html>");
    assert!(!html.contains("<body"), "no <body>");
    assert!(html.contains("id=\"settings-root\""), "missing root id: {html}");
    assert!(html.contains("<ship-tabs"), "missing ship-tabs element");
    assert!(html.contains("name=\"workspace\""));
    assert!(html.contains("name=\"providers\""));
    assert!(html.contains("id=\"settings-panel\""));
    assert!(
        html.contains("http://localhost:9315/api/runtime/view/settings"),
        "expected absolute daemon URL: {html}"
    );
}

#[test]
fn workspace_panel_renders_fields() {
    let mut ctx = empty_ctx("workspace");
    ctx.workspace_available = true;
    ctx.providers = vec![ProviderView {
        id: "claude".into(),
        binary: "claude".into(),
        version: Some("1.2.3".into()),
        available: true,
    }];
    ctx.defaults.provider = "claude".into();
    let html = templates::render("settings/content.html.j2", &ctx).expect("render");
    assert!(
        html.contains("id=\"defaults-provider\""),
        "expected defaults-provider select: {html}"
    );
    assert!(html.contains("id=\"workspace-root\""));
}

#[test]
fn empty_state_when_tab_data_unavailable() {
    // workspace tab with workspace_available=false → placeholder empty-state.
    let ctx = empty_ctx("workspace");
    let html = templates::render("settings/content.html.j2", &ctx).expect("render");
    assert!(
        html.contains("<ship-empty-state"),
        "expected empty-state: {html}"
    );
    assert!(html.contains("Not configured"));
}
