//! Tests for the specs view context + template rendering.
//!
//! Synthesize `SpecsCtx` directly and call `templates::render` — no
//! `ApiState` or running runtime required.

use super::*;
use crate::views::templates;

fn empty_ctx(active_tab: &str) -> SpecsCtx {
    SpecsCtx {
        active_tab: active_tab.to_string(),
        selected_id: None,
        counts: Counts::default(),
        specs: vec![],
        selected: None,
        pipelines: vec![],
        daemon_origin: "http://localhost:9315".to_string(),
        token: "test-token".to_string(),
    }
}

fn sample_summary(id: &str, status: &str) -> SpecSummary {
    SpecSummary {
        id: id.into(),
        title: format!("Spec {id}"),
        status: status.into(),
        pipeline: Some("tdd".into()),
        tone: tone_for(status),
        updated_at: "2025-01-02T03:04:05Z".into(),
    }
}

fn sample_detail(id: &str, status: &str) -> SpecDetail {
    SpecDetail {
        id: id.into(),
        title: format!("Spec {id}"),
        body: "# body".into(),
        status: status.into(),
        pipeline: Some("tdd".into()),
        tone: tone_for(status),
        title_js: format!("Spec {id}"),
        body_js: "# body".into(),
        pipeline_js: "tdd".into(),
        criteria_text_js: String::new(),
        scope_text_js: String::new(),
        vars_text_js: String::new(),
    }
}

#[test]
fn ctx_defaults_to_draft_tab() {
    let params = SpecsParams::default();
    assert_eq!(params.active_tab(), "draft");
    assert!(params.selected_id().is_none());
}

#[test]
fn filter_flattens_from_query() {
    let json = serde_json::json!({ "tab": "ready", "selected": "abc" });
    let params: SpecsParams = serde_json::from_value(json).expect("parse");
    assert_eq!(params.active_tab(), "ready");
    assert_eq!(params.selected_id().as_deref(), Some("abc"));
}

#[test]
fn shell_renders_as_fragment_with_root_id() {
    let ctx = empty_ctx("draft");
    let html = templates::render("specs/shell.html.j2", &ctx).expect("render");
    assert!(!html.contains("<!DOCTYPE"), "shell must be a fragment: {html}");
    assert!(!html.contains("<html"), "no <html> in fragment");
    assert!(!html.contains("<body"), "no <body> in fragment");
    assert!(html.contains("id=\"specs-root\""), "missing root id: {html}");
    assert!(html.contains("<ship-tabs"), "missing tabs");
    assert!(html.contains("specs-layout"), "missing layout");
    assert!(html.contains("id=\"specs-main\""), "missing main pane anchor");
    assert!(
        html.contains("http://localhost:9315/api/runtime/view/specs"),
        "expected absolute daemon URL: {html}"
    );
    assert!(
        html.contains("token=test-token"),
        "expected token threaded into URLs: {html}"
    );
}

#[test]
fn rail_renders_list_with_badges() {
    let mut ctx = empty_ctx("draft");
    ctx.specs = vec![sample_summary("s1", "draft"), sample_summary("s2", "draft")];
    ctx.counts.draft = 2;
    let html = templates::render("specs/partials/rail.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-list-item"), "expected list items: {html}");
    assert!(html.contains("<ship-badge"), "expected status badges");
    assert!(html.contains("Spec s1"));
    assert!(html.contains("Spec s2"));
}

#[test]
fn detail_renders_when_selected() {
    let mut ctx = empty_ctx("draft");
    ctx.selected_id = Some("s1".into());
    ctx.selected = Some(sample_detail("s1", "draft"));
    ctx.pipelines = vec!["tdd".into(), "ship-it".into()];
    let html = templates::render("specs/content.html.j2", &ctx).expect("render");
    assert!(html.contains("specs-detail"), "expected detail pane: {html}");
    assert!(html.contains("<ship-textarea"), "expected body editor");
    assert!(html.contains("Launch"), "expected launch button");
    assert!(html.contains("Save"), "expected save button");
}

#[test]
fn empty_state_when_no_specs() {
    let ctx = empty_ctx("draft");
    let html = templates::render("specs/partials/rail.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-empty-state"), "expected empty state: {html}");

    let content = templates::render("specs/content.html.j2", &ctx).expect("render content");
    assert!(content.contains("<ship-empty-state"), "expected empty main pane");
    assert!(content.contains("No spec selected"));
}
