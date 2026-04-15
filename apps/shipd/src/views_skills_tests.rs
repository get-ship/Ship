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
        body: "# Hello\nBody text.".into(),
    }
}

#[test]
fn shell_renders_rail_and_empty_right_pane() {
    let ctx = SkillsShellCtx { rail: sample_rail() };
    let html = templates::render("skills/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("id=\"skills-root\""));
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
    let ctx = SkillDetailCtx { skill: sample_detail(), rail: sample_rail() };
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
fn kind_matches_filters_correctly() {
    let s_smart = sum("a", SkillKind::Smart, "PROJECT SKILLS", &[]);
    let s_docs = sum("b", SkillKind::Docs, "PROJECT SKILLS", &[]);
    assert!(kind_matches("all", &s_smart));
    assert!(kind_matches("smart", &s_smart));
    assert!(!kind_matches("smart", &s_docs));
    assert!(kind_matches("docs", &s_docs));
}
