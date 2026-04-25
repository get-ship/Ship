//! Render tests for the first-run setup wizard.
//!
//! Each step partial and the full index render through the shared
//! minijinja environment used at runtime. No server is started.
//!
//! Run with: `cargo test -p shipd --test setup_tests`.

use serde::Serialize;
use shipd::views::templates::render_view_source;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct EmptyCtx {}

fn view_dir() -> PathBuf {
    // tests/ lives under apps/shipd/. The view files are at
    // <repo-root>/.ship/views/setup/.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("repo root above apps/shipd")
        .to_path_buf();
    runtime::ShipHome::project_ship_dir(repo_root).join("views/setup")
}

fn read_partial(name: &str) -> String {
    let path = view_dir().join("partials").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Render a partial in isolation. For partials that import SDK macros
/// (step-workspace / step-provider) the shared env already has
/// `_ship.html.j2` registered, so macro imports resolve at render time.
fn render_partial(name: &str) -> String {
    let source = read_partial(name);
    let template_name = format!("setup/partials/{}", name);
    render_view_source(&template_name, &source, &EmptyCtx {})
        .unwrap_or_else(|e| panic!("render {name}: {e}"))
}

// ─── Per-step render tests ───────────────────────────────────────────

#[test]
fn welcome_step_renders_headline_and_advances_step() {
    let html = render_partial("step-welcome.html.j2");
    assert!(html.contains("Welcome to Ship"), "missing headline: {html}");
    assert!(
        html.contains("$step = 2"),
        "CTA must advance step signal (got: {html})"
    );
    assert!(html.contains("Step 1 of 4"));
}

#[test]
fn workspace_step_has_both_tabs_and_wires_create_post() {
    let html = render_partial("step-workspace.html.j2");
    assert!(html.contains("Create new project"));
    assert!(html.contains("Open existing project"));
    // `bind('ws_name')` / `bind('ws_path')` macros expand to data-bind.
    assert!(
        html.contains("data-bind=\"ws_name\""),
        "ws_name field must bind signal"
    );
    assert!(
        html.contains("data-bind=\"ws_path\""),
        "ws_path field must bind signal"
    );
    // post macro expands to @post('/api/projects', {payload: …}).
    assert!(
        html.contains("@post('/api/projects'"),
        "submit must POST to /api/projects"
    );
    assert!(
        html.contains("data-show=\"$ws_error\""),
        "error slot must be gated on signal"
    );
    // Continue disables when the required field is empty.
    assert!(html.contains("data-attr:disabled="));
}

#[test]
fn provider_step_lists_four_cards_and_wires_test_and_save() {
    let html = render_partial("step-provider.html.j2");
    for name in ["claude", "gemini", "codex", "opencode"] {
        assert!(
            html.contains(&format!("$provider = '{name}'")),
            "provider card missing: {name}"
        );
    }
    assert!(
        html.contains("@post('/api/providers/test'"),
        "Test connection must POST to /api/providers/test"
    );
    assert!(
        html.contains("$step = 4"),
        "skip / save buttons must advance to step 4"
    );
    assert!(
        html.contains("$provider_test_result == 'ok'"),
        "green-check guard must reference signal"
    );
    assert!(
        html.contains("$provider_test_result != 'ok'"),
        "Save button must gate on successful test"
    );
}

#[test]
fn done_step_links_docs_community_store_and_redirects() {
    let html = render_partial("step-done.html.j2");
    assert!(html.contains("Ship is ready"));
    assert!(html.contains("getship.dev/docs"));
    assert!(html.contains("getship.dev/community"));
    assert!(html.contains("getship.dev/store"));
    assert!(
        html.contains("@post('/api/settings'"),
        "Open button must persist setup.completed via /api/settings"
    );
    assert!(
        html.contains("window.location.assign('/')"),
        "Open button must redirect to /"
    );
}

// ─── Index / manifest structure ───────────────────────────────────────

#[test]
fn full_index_renders_with_all_partials_spliced_in() {
    // The {% include %} chain in index.html.j2 needs sibling partial
    // resolution. render_view_source walks get_project_dir()/views/<name>,
    // which inside a git worktree may redirect to the main checkout.
    // SHIP_DIR overrides that, per runtime::project::discovery.
    let ship_dir = view_dir()
        .parent()
        .and_then(Path::parent)
        .expect("views/setup has .ship/ grandparent")
        .to_path_buf();
    let prior = runtime::env::read_raw(runtime::env::Var::Dir).ok();
    unsafe {
        runtime::env::set(runtime::env::Var::Dir, &ship_dir);
    }

    let index_path = view_dir().join("index.html.j2");
    let source = std::fs::read_to_string(&index_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", index_path.display()));
    let result = render_view_source("setup/index.html.j2", &source, &EmptyCtx {});

    unsafe {
        match prior {
            Some(v) => runtime::env::set(runtime::env::Var::Dir, v),
            None => runtime::env::remove(runtime::env::Var::Dir),
        }
    }

    let html = result.expect("full index must render with siblings resolved");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Set up Ship"), "expected <title>");
    assert!(html.contains("Welcome to Ship"));
    assert!(html.contains("Pick a project folder"));
    assert!(html.contains("Connect an AI provider"));
    assert!(html.contains("Ship is ready"));
    // Progress uses the ship-stepper primitive (not bespoke dots).
    assert!(
        html.contains("<ship-stepper"),
        "index must use <ship-stepper> for progress indicator"
    );
    assert!(
        html.contains("data-attr:current=\"$step\""),
        "stepper must bind current to $step signal"
    );
    // signal(…) macro outputs.
    assert!(html.contains("data-signals:step="));
    assert!(html.contains("data-signals:provider_test_result="));
}

#[test]
fn index_declares_every_required_signal() {
    let path = view_dir().join("index.html.j2");
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    for sig in [
        "step", "ws_mode", "ws_name", "ws_path", "provider",
        "api_key", "provider_test_result", "provider_test_error", "ws_error",
    ] {
        assert!(
            src.contains(&format!("signal('{sig}'")),
            "index must declare signal: {sig}"
        );
    }
    // Progress uses the <ship-stepper> primitive bound to $step.
    assert!(
        src.contains("ship-stepper") && src.contains("data-attr:current=\"$step\""),
        "index must use ship-stepper bound to $step"
    );
    for n in 1..=4 {
        assert!(
            src.contains(&format!("$step == {n}")),
            "step panel {n} must be gated on $step"
        );
    }
    for partial in ["step-welcome", "step-workspace", "step-provider", "step-done"] {
        assert!(
            src.contains(&format!("setup/partials/{partial}")),
            "index must include partial: {partial}"
        );
    }
}

#[test]
fn manifest_is_valid_jsonc_and_names_setup() {
    let path = view_dir().join("manifest.jsonc");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let stripped: String = raw
        .lines()
        .map(|l| {
            if let Some(ix) = l.find("//") {
                &l[..ix]
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let value: serde_json::Value =
        serde_json::from_str(&stripped).expect("manifest must parse as JSON after stripping //");
    assert_eq!(value["name"], "setup");
    assert_eq!(value["type"], "html");
    let signals = value["signals"]
        .as_object()
        .expect("signals block must be an object");
    for sig in [
        "step", "ws_mode", "ws_name", "ws_path", "provider",
        "api_key", "provider_test_result",
    ] {
        assert!(signals.contains_key(sig), "manifest missing signal: {sig}");
    }
    let actions = value["actions"]
        .as_object()
        .expect("actions block must be an object");
    for act in ["workspace.create", "settings.save", "provider.test", "setup.complete"] {
        assert!(actions.contains_key(act), "manifest missing action: {act}");
    }
}
