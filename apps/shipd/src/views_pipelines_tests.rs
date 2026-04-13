//! Tests for the pipelines view context + template rendering.
//!
//! These tests synthesize `PipelinesCtx` directly and call `templates::render`
//! — they do not need an `ApiState` nor a running runtime.

use super::*;
use crate::views::state::{Filter, PipelineDef};
use crate::views::templates;

fn empty_ctx(active_tab: &str) -> PipelinesCtx {
    PipelinesCtx {
        active_tab: active_tab.to_string(),
        filter: Filter::default(),
        stats: Stats {
            definitions: 0,
            active: 0,
            completed: 0,
            failed: 0,
            history: 0,
        },
        defs: vec![],
        active_runs: vec![],
        history_runs: vec![],
        daemon_origin: "http://localhost:9315".to_string(),
        token: "test-token".to_string(),
    }
}

fn sample_run(slug: &str, state: &'static str) -> RunView {
    RunView {
        job_id: format!("j-{slug}"),
        slug: slug.into(),
        agent: "coder".into(),
        state,
        tone: "info",
        duration: "1m 0s".into(),
        updated_at: "2025-01-02T03:04:05Z".into(),
        steps: vec![StepView {
            id: "impl".into(),
            step_type: "agent".into(),
            state: "running",
            agent: Some("coder".into()),
        }],
    }
}

#[test]
fn ctx_defaults_to_active_tab() {
    let params = PipelinesParams::default();
    assert_eq!(params.active_tab(), "active");
}

#[test]
fn filter_flattens_from_query() {
    // Deserialize from a JSON object, which `#[serde(flatten)]` handles the
    // same way as a flat query string but without requiring a URL-encoder
    // dep in this crate.
    let json = serde_json::json!({ "tab": "history", "status": "failed" });
    let params: PipelinesParams = serde_json::from_value(json).expect("parse");
    assert_eq!(params.active_tab(), "history");
    assert_eq!(params.filter.status.as_deref(), Some("failed"));
}

#[test]
fn shell_renders_as_fragment_with_root_id() {
    let ctx = empty_ctx("active");
    let html = templates::render("pipelines/shell.html.j2", &ctx).expect("render");
    assert!(
        !html.contains("<!DOCTYPE"),
        "shell must be a fragment (no doctype): {html}"
    );
    assert!(!html.contains("<html"), "shell must not contain <html>");
    assert!(!html.contains("<body"), "shell must not contain <body>");
    assert!(
        html.contains("id=\"pipelines-root\""),
        "missing root id: {html}"
    );
    assert!(html.contains("<ship-stat"), "missing ship-stat element: {html}");
    assert!(html.contains("<ship-tabs"), "missing ship-tabs element: {html}");
    assert!(
        html.contains("http://localhost:9315/api/runtime/view/pipelines"),
        "expected absolute daemon URL baked in: {html}"
    );
    assert!(
        html.contains("token=test-token"),
        "expected token threaded into URLs: {html}"
    );
}

#[test]
fn content_renders_active_tab_with_runs() {
    let mut ctx = empty_ctx("active");
    ctx.active_runs = vec![sample_run("fix-bug", "dispatched")];
    ctx.stats.active = 1;
    let html = templates::render("pipelines/content.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-pipeline-run"), "expected ship-pipeline-run: {html}");
    assert!(html.contains("fix-bug"));
}

#[test]
fn empty_state_when_no_active_runs() {
    let ctx = empty_ctx("active");
    let html = templates::render("pipelines/content.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-empty-state"), "expected empty state: {html}");
}

#[test]
fn definitions_tab_renders_def_cards() {
    let mut ctx = empty_ctx("definitions");
    ctx.defs = vec![PipelineDef {
        name: "tdd".into(),
        description: "test driven".into(),
        step_count: 3,
    }];
    let html = templates::render("pipelines/content.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-pipeline-definition"));
    assert!(html.contains("name=\"tdd\""));
}
