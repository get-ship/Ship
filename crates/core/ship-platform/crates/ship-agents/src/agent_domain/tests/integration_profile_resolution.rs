/// Integration tests — profile resolution: the 4-way merger at config.rs:104.
///
/// Exercises `resolve_harness_settings` and
/// `resolve_harness_settings_with_agent_override` against an ephemeral project
/// and asserts the merged `ProviderSettings` fields match the resolution
/// priority (workspace > agent-profile > project > defaults).
use anyhow::Result;
use tempfile::tempdir;

use crate::agent_domain::config::{
    ProviderSettings, WorkspaceAgentSettings, resolve_harness_settings,
    resolve_harness_settings_with_agent_override,
};
use crate::agent_domain::rule::create_rule;
use crate::agent_domain::skill::create_skill;
use ship_config::project::{AgentProfile, McpServerConfig, McpServerType, ProjectConfig, save_config};
use crate::testutil::init_project;
use std::collections::HashMap;

fn stdio_server(id: &str, command: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: command.to_string(),
        args: vec![],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    }
}

#[test]
fn agents_profile_resolution_defaults_to_claude_when_no_harness_configured() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let resolved: ProviderSettings = resolve_harness_settings(&ship_dir, None)?;
    assert_eq!(
        resolved.harnesses,
        vec!["claude".to_string()],
        "expected default harness [claude]; got {:?}",
        resolved.harnesses
    );
    assert!(resolved.active_agent.is_none());
    Ok(())
}

#[test]
fn agents_profile_resolution_workspace_harness_overrides_project() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    std::fs::write(
        ship_dir.join("ship.jsonc"),
        "{\"harnesses\": [\"claude\", \"cursor\"]}\n",
    )?;

    let wa = WorkspaceAgentSettings {
        harnesses: vec!["codex".to_string()],
        model: None,
        mcp_servers: vec![],
        skills: vec![],
    };
    let resolved = resolve_harness_settings(&ship_dir, Some(&wa))?;
    assert_eq!(resolved.harnesses, vec!["codex".to_string()]);
    Ok(())
}

#[test]
fn agents_profile_resolution_agent_profile_filters_mcp_servers() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let config = ProjectConfig {
        mcp_servers: vec![
            stdio_server("github", "github-bin"),
            stdio_server("linear", "linear-bin"),
        ],
        agents: vec![AgentProfile {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            mcp_servers: vec!["github".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };
    save_config(&config, Some(ship_dir.clone()))?;

    let resolved =
        resolve_harness_settings_with_agent_override(&ship_dir, None, Some("focus"))?;
    assert_eq!(resolved.mcp_servers.len(), 1);
    assert_eq!(resolved.mcp_servers[0].id, "github");
    Ok(())
}

#[test]
fn agents_profile_resolution_agent_profile_filters_skills() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "skill-a", "A", "body a")?;
    create_skill(&ship_dir, "skill-b", "B", "body b")?;

    let config = ProjectConfig {
        agents: vec![AgentProfile {
            id: "narrow".to_string(),
            name: "Narrow".to_string(),
            skills: vec!["skill-a".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };
    save_config(&config, Some(ship_dir.clone()))?;

    let resolved =
        resolve_harness_settings_with_agent_override(&ship_dir, None, Some("narrow"))?;
    let ids: Vec<&str> = resolved.skills.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["skill-a"], "profile should retain only skill-a");
    Ok(())
}

#[test]
fn agents_profile_resolution_rules_pass_through_when_no_filter() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_rule(ship_dir.clone(), "rule-a.md", "# A\n")?;
    create_rule(ship_dir.clone(), "rule-b.md", "# B\n")?;

    // No agent profile → no filter → both rules surface in resolved settings.
    let resolved = resolve_harness_settings(&ship_dir, None)?;
    let mut fnames: Vec<&str> =
        resolved.rules.iter().map(|r| r.file_name.as_str()).collect();
    fnames.sort();
    assert_eq!(fnames, vec!["rule-a.md", "rule-b.md"]);
    Ok(())
}

#[test]
fn agents_profile_resolution_unknown_agent_override_falls_back_to_no_active_agent() -> Result<()>
{
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let resolved =
        resolve_harness_settings_with_agent_override(&ship_dir, None, Some("nonexistent"))?;
    assert!(
        resolved.active_agent.is_none(),
        "unknown override must not set active_agent; got {:?}",
        resolved.active_agent
    );
    Ok(())
}

#[test]
fn agents_profile_resolution_workspace_model_propagates() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let wa = WorkspaceAgentSettings {
        model: Some("claude-opus-4".to_string()),
        harnesses: vec![],
        mcp_servers: vec![],
        skills: vec![],
    };
    let resolved = resolve_harness_settings(&ship_dir, Some(&wa))?;
    assert_eq!(resolved.model.as_deref(), Some("claude-opus-4"));
    Ok(())
}

#[test]
fn agents_profile_resolution_workspace_skill_filter_ignores_absent_ids() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "present", "Present", "body")?;

    let wa = WorkspaceAgentSettings {
        skills: vec!["present".to_string(), "missing-id".to_string()],
        harnesses: vec![],
        model: None,
        mcp_servers: vec![],
    };
    let resolved = resolve_harness_settings(&ship_dir, Some(&wa))?;
    let ids: Vec<&str> = resolved.skills.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["present"]);
    Ok(())
}
