//! Tests for the specs view context + template rendering.

use super::*;
use crate::views::templates;

fn empty_ctx() -> SpecsCtx {
    SpecsCtx {
        selected_id: None,
        q: String::new(),
        counts: Counts::default(),
        groups: vec![],
        total: 0,
        selected: None,
        pipelines: vec![],
    }
}

fn sample_summary(id: &str, status: &str) -> SpecSummary {
    SpecSummary {
        id: id.into(),
        title: format!("Spec {id}"),
        status: status.into(),
        pipeline: Some("tdd".into()),
        tone: tone_for(status),
        when: "updated 3m ago".into(),
    }
}

fn sample_detail(id: &str, status: &str) -> SpecDetail {
    SpecDetail {
        id: id.into(),
        title: format!("Spec {id}"),
        body: "# body".into(),
        status: status.into(),
        status_label: "Draft".into(),
        pipeline: Some("tdd".into()),
        pipeline_label: "tdd".into(),
        agent: "test-writer".into(),
        vars_display: "—".into(),
        branch: "auto".into(),
        tone: tone_for(status),
        goal_md: "Do the thing.".into(),
        scope_md: "only this file.".into(),
        linked_md: String::new(),
        acceptance: vec![
            AcceptanceItem { text: "first".into(), done: true },
            AcceptanceItem { text: "second".into(), done: false },
        ],
        other_md: String::new(),
        launch_label: "Launch tdd → test-writer".into(),
        launch_hint: "Will create worktree …".into(),
        title_js: format!("Spec {id}"),
        body_js: "# body".into(),
        pipeline_js: "tdd".into(),
    }
}

#[test]
fn params_selected_and_query_flatten() {
    let json = serde_json::json!({ "selected": "abc", "q": "auth" });
    let params: SpecsParams = serde_json::from_value(json).expect("parse");
    assert_eq!(params.selected_id().as_deref(), Some("abc"));
    assert_eq!(params.query(), "auth");
}

#[test]
fn shell_renders_as_fragment_with_root_id() {
    let ctx = empty_ctx();
    let html = templates::render("specs/shell.html.j2", &ctx).expect("render");
    assert!(!html.contains("<!DOCTYPE"));
    assert!(html.contains("id=\"specs-root\""));
    assert!(html.contains("specs-grid"));
    assert!(html.contains("id=\"specs-rail-list\""));
    assert!(html.contains("Filter…"));
    assert!(!html.contains("token="), "cookie auth: URLs must not carry ?token=");
}

#[test]
fn rail_groups_and_items_render() {
    let mut ctx = empty_ctx();
    ctx.total = 2;
    ctx.counts.draft = 1;
    ctx.counts.ready = 1;
    ctx.groups = vec![
        RailGroup {
            key: "draft",
            label: "Draft",
            count: 1,
            items: vec![sample_summary("s1", "draft")],
        },
        RailGroup {
            key: "ready",
            label: "Ready",
            count: 1,
            items: vec![sample_summary("s2", "ready")],
        },
        RailGroup { key: "launched", label: "Launched", count: 0, items: vec![] },
        RailGroup { key: "completed", label: "Completed", count: 0, items: vec![] },
    ];
    let html = templates::render("specs/partials/rail.html.j2", &ctx).expect("render");
    assert!(html.contains("Draft"));
    assert!(html.contains("Ready"));
    assert!(html.contains("<ship-list-tile"));
    assert!(html.contains("<ship-status-dot"));
    assert!(html.contains("Spec s1"));
    assert!(html.contains("Spec s2"));
}

#[test]
fn rail_empty_state_when_no_specs() {
    let ctx = empty_ctx();
    let html = templates::render("specs/partials/rail.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-empty-state"));
    assert!(html.contains("No specs yet"));
}

#[test]
fn detail_renders_metachips_and_acceptance() {
    let mut ctx = empty_ctx();
    ctx.selected_id = Some("s1".into());
    ctx.selected = Some(sample_detail("s1", "draft"));
    let html = templates::render("specs/content.html.j2", &ctx).expect("render");
    assert!(html.contains("specs-metachips"));
    assert!(html.contains("<ship-metachip"));
    assert!(html.contains("label=\"Status\""));
    assert!(html.contains("label=\"Pipeline\""));
    assert!(html.contains("label=\"Agent\""));
    assert!(html.contains("label=\"Vars\""));
    assert!(html.contains("label=\"Branch\""));
    assert!(html.contains("specs-acceptance"));
    assert!(html.contains("<ship-checkbox"));
    assert!(html.contains("first"));
    assert!(html.contains("second"));
    assert!(html.contains("Launch tdd"));
    assert!(html.contains("specs-launch-bar"));
}

#[test]
fn content_empty_state_when_no_selection() {
    let ctx = empty_ctx();
    let html = templates::render("specs/content.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-empty-state"));
    assert!(html.contains("No spec selected"));
}

#[test]
fn parse_acceptance_extracts_checklist_items() {
    let body = "## Goal\n\nThe goal.\n\n## Acceptance\n- [x] first\n- [X] second\n- [ ] third\n- not a check\n\n## Scope\n- foo\n";
    let (sections, _prelude) = split_sections(body);
    let acc = sections.get("acceptance").expect("acc section");
    let items = parse_acceptance(acc);
    assert_eq!(items.len(), 3);
    assert!(items[0].done);
    assert_eq!(items[0].text, "first");
    assert!(items[1].done);
    assert!(!items[2].done);
    assert_eq!(items[2].text, "third");
    assert!(sections.contains_key("scope"));
    assert!(sections.contains_key("goal"));
}

#[test]
fn filter_query_matches_title_and_id() {
    use runtime::db::specs::Spec;
    let s = Spec {
        id: "abc-123".into(),
        title: "Auth pair flow".into(),
        body_path: String::new(),
        status: "draft".into(),
        pipeline: None,
        vars: Default::default(),
        criteria: vec![],
        scope: vec![],
        priority: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert!(matches_query(&s, ""));
    assert!(matches_query(&s, "auth"));
    assert!(matches_query(&s, "AUTH"));
    assert!(matches_query(&s, "abc"));
    assert!(!matches_query(&s, "xyz"));
}
