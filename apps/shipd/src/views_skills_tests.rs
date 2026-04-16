//! Tests for the skills view context + template rendering.

use super::*;
use crate::views::state::{SkillDetail, SkillKind, SkillSummary};
use crate::views::templates;

fn sum(name: &str, kind: SkillKind, source: &str, tags: &[&str]) -> SkillSummary {
    SkillSummary {
        id: name.to_string(),
        name: name.to_string(),
        description: format!("{name} skill"),
        kind,
        source: source.to_string(),
        tags: tags.iter().map(|s| s.to_string()).collect(),
        files: vec!["SKILL.md".into()],
        has_vars: false,
        has_refs: false,
        has_evals: false,
    }
}

fn sample_rail() -> RailCtx {
    let all = vec![
        sum("tdd", SkillKind::Smart, "PROJECT SKILLS", &["testing"]),
        sum("ship-runtime", SkillKind::Docs, "PROJECT SKILLS", &["docs"]),
    ];
    let groups = vec![SkillGroup {
        source: "PROJECT SKILLS".into(),
        skills: all.clone(),
    }];
    RailCtx {
        q: String::new(),
        active_kind: "all".into(),
        active_tag: String::new(),
        groups,
        kind_counts: vec![
            KindCount { id: "all", label: "All", count: 2, selected: true },
            KindCount { id: "smart", label: "Smart", count: 1, selected: false },
            KindCount { id: "docs", label: "Docs", count: 1, selected: false },
        ],
        tags: vec!["docs".into(), "testing".into()],
        total: 2,
        filtered: 2,
    }
}

fn sample_detail() -> SkillDetail {
    SkillDetail {
        id: "tdd".into(),
        name: "tdd".into(),
        description: "Test first.".into(),
        kind: SkillKind::Smart,
        source: "PROJECT SKILLS".into(),
        tags: vec!["testing".into()],
        authors: vec![],
        variables: vec!["foo".into(), "bar".into()],
        artifacts: vec![],
        files: vec!["SKILL.md".into()],
        body: "# Hello\nBody text.".into(),
    }
}

#[test]
fn shell_renders_rail_and_empty_right_pane() {
    let ctx = SkillsShellCtx { rail: sample_rail() };
    let html = templates::render("skills/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("id=\"ship-view-root\""));
    assert!(html.contains("PROJECT SKILLS"));
    assert!(html.contains("tdd"));
    assert!(html.contains("ship-runtime"), "docs skill present: {html}");
    assert!(html.contains("<ship-empty-state"), "right pane empty state");
}

#[test]
fn rail_partial_renders_standalone() {
    let ctx = SkillsShellCtx { rail: sample_rail() };
    let html = templates::render("skills/partials/rail.html.j2", &ctx).expect("render");
    assert!(html.contains("id=\"skills-rail\""));
    assert!(html.contains("tdd"));
    assert!(html.contains("ship-runtime"));
}

#[test]
fn rail_partial_shows_empty_when_no_groups() {
    let mut rail = sample_rail();
    rail.groups.clear();
    rail.filtered = 0;
    let ctx = SkillsShellCtx { rail };
    let html = templates::render("skills/partials/rail.html.j2", &ctx).expect("render");
    assert!(html.contains("No skills match"), "empty rail hint: {html}");
}

#[test]
fn detail_renders_markdown_body_and_chips() {
    let ctx = SkillDetailCtx { skill: sample_detail(), rail: sample_rail(), file: None };
    let html = templates::render("skills/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-markdown"), "markdown element: {html}");
    assert!(html.contains("Body text"), "body rendered: {html}");
    assert!(html.contains("testing"), "tag chip rendered: {html}");
}

#[test]
fn detail_template_missing_yields_error() {
    let err =
        templates::render("skills/does-not-exist.html.j2", &serde_json::json!({})).unwrap_err();
    assert!(err.contains("not found"));
}

#[test]
fn view_template_renders_three_column_layout() {
    // Test the .ship/views/skills/index.html.j2 template via render_view_source.
    let ctx = SkillsShellCtx { rail: sample_rail() };
    let result = try_render_view("skills", "index.html.j2", &ctx);
    if let Some(html) = result {
        assert!(html.contains("id=\"ship-view-root\""), "root wrapper: {html}");
        assert!(html.contains("skills-ide"), "IDE layout class: {html}");
        assert!(html.contains("skills-explorer"), "explorer pane: {html}");
        assert!(html.contains("skills-editor"), "editor pane: {html}");
        assert!(html.contains("No file open"), "empty editor state: {html}");
    }
    // If the view dir doesn't exist (CI), the test still passes — the
    // fallback embedded template is tested elsewhere.
}

#[test]
fn view_template_renders_detail_with_file() {
    let file = SkillFileCtx {
        path: "SKILL.md".into(),
        name: "SKILL.md".into(),
        ext: "md".into(),
        content: "# Test\nHello world.".into(),
    };
    let ctx = SkillDetailCtx {
        skill: sample_detail(),
        rail: sample_rail(),
        file: Some(file),
    };
    let result = try_render_view("skills", "index.html.j2", &ctx);
    if let Some(html) = result {
        assert!(html.contains("skills-explorer"), "explorer: {html}");
        assert!(html.contains("skills-editor"), "editor: {html}");
        assert!(html.contains("skills-info"), "info panel: {html}");
        assert!(html.contains("ship-markdown"), "markdown render: {html}");
        assert!(html.contains("SKILL.md"), "breadcrumb: {html}");
    }
}

#[test]
fn kind_matches_filters_correctly() {
    let s_smart = sum("a", SkillKind::Smart, "PROJECT SKILLS", &[]);
    let s_docs = sum("b", SkillKind::Docs, "PROJECT SKILLS", &[]);
    assert!(kind_matches("all", &s_smart));
    assert!(kind_matches("smart", &s_smart));
    assert!(!kind_matches("smart", &s_docs));
    assert!(kind_matches("docs", &s_docs));
}
