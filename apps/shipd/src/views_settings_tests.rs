//! Tests for the settings view context + template rendering.
//!
//! These synthesize `SettingsCtx` directly and render templates — no
//! `ApiState`, no runtime initialization.

use super::*;
use crate::views::templates;

fn sample_ctx() -> SettingsCtx {
    SettingsCtx {
        providers: vec![
            ProviderRow {
                id: "claude".into(),
                label: "Anthropic Claude".into(),
                glyph: "✦".into(),
                binary: "claude".into(),
                version: Some("1.2.3".into()),
                available: true,
                is_default: true,
            },
            ProviderRow {
                id: "openai".into(),
                label: "OpenAI".into(),
                glyph: "◆".into(),
                binary: "codex".into(),
                version: None,
                available: false,
                is_default: false,
            },
        ],
        default_provider: "claude".into(),
        default_model: "sonnet".into(),
        terminal: TerminalConfig {
            program: "tmux".into(),
            worktree_base: "/tmp/worktrees".into(),
            options: vec![
                SelectOption { value: "tmux".into(), label: "tmux".into() },
                SelectOption { value: "iterm".into(), label: "iTerm".into() },
            ],
        },
        advanced: vec![],
        advanced_registry: "https://registry.getship.dev".into(),
        daemon_endpoint: "http://localhost:9315".into(),
        about: AboutInfo {
            ship_version: "0.0.1".into(),
            daemon_version: "0.0.1".into(),
            cli_version: "0.0.1".into(),
            license: "MIT".into(),
            homepage: "https://getship.dev".into(),
            docs: "https://getship.dev/docs".into(),
            source: "https://github.com/madvib/Ship".into(),
        },
    }
}

fn empty_ctx() -> SettingsCtx {
    SettingsCtx {
        providers: vec![],
        default_provider: "claude".into(),
        default_model: String::new(),
        terminal: TerminalConfig::default(),
        advanced: vec![],
        advanced_registry: String::new(),
        daemon_endpoint: "http://localhost:9315".into(),
        about: AboutInfo::default(),
    }
}

#[test]
fn ctx_has_providers_terminal_advanced_about() {
    let ctx = sample_ctx();
    assert_eq!(ctx.providers.len(), 2);
    assert_eq!(ctx.terminal.program, "tmux");
    assert_eq!(ctx.advanced.len(), 0);
    assert_eq!(ctx.about.license, "MIT");
}

#[test]
fn shell_renders_single_column_four_sections() {
    let ctx = sample_ctx();
    let html = templates::render("settings/shell.html.j2", &ctx).expect("render");

    // Fragment, not a full document.
    assert!(!html.contains("<!DOCTYPE"), "shell must be a fragment");
    assert!(!html.contains("<html"), "no <html>");
    // Root id + single column max-width 768.
    assert!(html.contains("id=\"ship-view-root\""), "missing root id");
    assert!(html.contains("max-width: 768px"), "missing 768px column");
    // Header content.
    assert!(html.contains(">Settings<"), "missing h1");
    assert!(html.contains("CLI and studio configuration"), "missing subtitle");
    // All four section headers present in order.
    let p = html.find("PROVIDERS").expect("PROVIDERS");
    let t = html.find("TERMINAL").expect("TERMINAL");
    let a = html.find("ADVANCED").expect("ADVANCED");
    let b = html.find("ABOUT").expect("ABOUT");
    assert!(p < t && t < a && a < b, "sections out of order");
    // No tabs control in the new IA.
    assert!(!html.contains("<ship-tabs"), "tabs control should be gone");
    assert!(html.contains("id=\"settings-panel\""), "missing sse target");
}

#[test]
fn providers_renders_provider_rows() {
    let ctx = sample_ctx();
    let html = templates::render("settings/partials/providers.html.j2", &ctx).expect("render");
    assert!(html.contains("Anthropic Claude"), "missing provider label");
    assert!(html.contains("OpenAI"), "missing second provider");
    assert!(html.contains("provider-dot on"), "installed provider should have on dot");
    assert!(html.contains("v1.2.3"), "version should render");
    assert!(html.contains("not found"), "unavailable provider should say 'not found'");
}

#[test]
fn empty_state_when_no_providers_configured() {
    let ctx = empty_ctx();
    let html = templates::render("settings/partials/providers.html.j2", &ctx).expect("render");
    assert!(
        html.contains("No provider CLIs found"),
        "expected empty-state message: {html}"
    );
    // Advanced also empty-states cleanly.
    let adv = templates::render("settings/partials/advanced.html.j2", &ctx).expect("render");
    assert!(adv.contains("No experimental features available."));
}

#[test]
fn url_token_baked_in() {
    let ctx = sample_ctx();
    let html = templates::render("settings/shell.html.j2", &ctx).expect("render");
    assert!(
        html.contains("/api/runtime/view/settings/sse"),
        "expected relative daemon SSE URL"
    );
    assert!(!html.contains("token="), "cookie auth: URLs must not carry ?token=");
}
