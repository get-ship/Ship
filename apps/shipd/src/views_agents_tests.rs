//! Tests for the agents view context + template rendering.

use super::*;
use crate::views::state::{AgentDetail, AgentSummary};
use crate::views::templates;

fn sample_summary(name: &str, letter: &str) -> AgentSummary {
    AgentSummary {
        id: name.to_lowercase(),
        name: name.to_string(),
        description: format!("{name} does things."),
        skills_count: 3,
        providers_count: 2,
        mcp_count: 1,
        rules_count: 2,
        letter: letter.to_string(),
        source: "PROJECT AGENTS".into(),
    }
}

fn sample_detail() -> AgentDetail {
    AgentDetail {
        id: "coder".into(),
        name: "Coder".into(),
        description: "Writes code".into(),
        letter: "C".into(),
        skills: vec!["tdd".into(), "refactor".into()],
        mcp_servers: vec!["ship".into()],
        rules: vec!["kind".into()],
        providers: vec!["claude".into(), "codex".into()],
        default_provider: Some("claude".into()),
        permissions_tier: "standard".into(),
        source: "PROJECT AGENTS".into(),
    }
}

#[test]
fn shell_renders_card_grid_and_new_button() {
    let ctx = AgentsIndexCtx {
        agents: vec![sample_summary("Coder", "C"), sample_summary("Reviewer", "R")],
        total: 2,
    };
    let html = templates::render("agents/shell.html.j2", &ctx).expect("render");
    assert!(!html.contains("<!DOCTYPE"), "fragment only");
    assert!(html.contains("id=\"agents-root\""), "root id");
    assert!(html.contains("Coder"));
    assert!(html.contains("Reviewer"));
    assert!(html.contains("+ New Agent"));
    assert!(html.contains("2 agents") || html.contains("2 configured"));
}

#[test]
fn shell_empty_state_when_no_agents() {
    let ctx = AgentsIndexCtx { agents: vec![], total: 0 };
    let html = templates::render("agents/shell.html.j2", &ctx).expect("render");
    assert!(
        html.contains("<ship-empty-state"),
        "empty state element: {html}"
    );
}

#[test]
fn detail_renders_all_sections() {
    let ctx = AgentDetailCtx {
        agent: sample_detail(),
        tiers: build_tiers("standard"),
    };
    let html = templates::render("agents/detail.html.j2", &ctx).expect("render");
    for header in ["Skills", "MCP Servers", "Permissions", "Rules", "Providers"] {
        assert!(html.contains(header), "missing section {header}: {html}");
    }
    assert!(html.contains("tdd"));
    assert!(html.contains("claude"));
    // Tier selected state round-trips.
    assert!(
        html.contains("data-tier-selected=\"standard\"")
            || html.contains("aria-checked=\"true\""),
        "standard tier should be marked selected: {html}"
    );
}

#[test]
fn detail_renders_empty_lists_gracefully() {
    let mut agent = sample_detail();
    agent.skills.clear();
    agent.mcp_servers.clear();
    agent.rules.clear();
    agent.providers.clear();
    let ctx = AgentDetailCtx {
        agent,
        tiers: build_tiers("standard"),
    };
    let html = templates::render("agents/detail.html.j2", &ctx).expect("render");
    // Every empty-list section falls back to an empty-state chip.
    assert!(html.matches("No skills").count() >= 1, "skills empty: {html}");
}

#[test]
fn build_tiers_marks_selected() {
    let tiers = build_tiers("autonomous");
    assert_eq!(tiers.len(), 3);
    assert!(tiers.iter().find(|t| t.id == "autonomous").unwrap().selected);
    assert!(!tiers.iter().find(|t| t.id == "standard").unwrap().selected);
}

#[test]
fn detail_template_not_found_renders_error() {
    let err =
        templates::render("agents/does-not-exist.html.j2", &serde_json::json!({})).unwrap_err();
    assert!(err.contains("not found"));
}
