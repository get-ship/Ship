use super::*;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn save_config_keeps_ship_toml_free_of_agent_sections() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    fs::create_dir_all(&ship_dir)?;

    let config = ProjectConfig {
        providers: vec!["claude".to_string(), "codex".to_string()],
        agent: AgentLayerConfig {
            skills: vec!["task-policy".to_string()],
            prompts: vec![],
            context: vec!["project/README.md".to_string()],
        },
        agents: vec![AgentProfile {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            ..Default::default()
        }],
        mcp_servers: vec![McpServerConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: Some(30),
        }],
        ..Default::default()
    };

    save_config(&config, Some(ship_dir.clone()))?;

    let ship_config = fs::read_to_string(ship_dir.join(PRIMARY_CONFIG_FILE))?;
    assert!(
        !ship_config.contains("\"agents\""),
        "ship.jsonc must not persist agent definitions"
    );
    assert!(
        !ship_config.contains("mcp_servers"),
        "ship.jsonc must not persist MCP servers"
    );
    assert!(
        !ship_config.contains("\"agent\""),
        "ship.jsonc must not persist agent block"
    );
    assert!(
        !ship_config.contains("\"providers\""),
        "ship.jsonc must not persist providers"
    );
    assert!(
        !ship_config.contains("active_agent"),
        "ship.jsonc must not persist active_agent"
    );
    assert!(
        !ship_config.contains("statuses"),
        "ship.jsonc must not persist statuses"
    );
    assert!(
        !ship_config.contains("\"git\""),
        "ship.jsonc must not persist git policy"
    );
    assert!(
        !ship_config.contains("\"ai\""),
        "ship.jsonc must not persist ai block"
    );
    assert!(
        !ship_config.contains("namespaces"),
        "ship.jsonc must not persist namespace claims"
    );

    // Legacy per-agent TOML config is no longer read or written — the
    // loader path was removed in the home-layout restructure.

    // Verify runtime settings stored in kv_state (project-scoped namespace)
    let providers: Vec<String> = ship_substrate::db::kv::get(ship_substrate::db::kv::KvNamespace::AgentsCache(format!("runtime::{}", ship_dir.display())), "providers")?
        .map(|v| serde_json::from_value(v).unwrap())
        .unwrap_or_default();
    assert_eq!(providers, vec!["claude".to_string(), "codex".to_string()]);

    // statuses, ai, and git fields were removed from ProjectConfig — no
    // KV checks for them anymore.

    // Verify agents stored in kv_state
    let agents_val = ship_substrate::db::kv::get(ship_substrate::db::kv::KvNamespace::AgentsCache(format!("runtime::{}", ship_dir.display())), "agents")?.expect("expected agents in kv");
    let agents: Vec<serde_json::Value> = serde_json::from_value(agents_val)?;
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["id"], "planning");

    let mcp_cfg = fs::read_to_string(ship_dir.join("mcp.jsonc"))?;
    assert!(mcp_cfg.contains("\"github\""));
    Ok(())
}

#[test]
fn get_config_round_trips_agent_sidecars() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    fs::create_dir_all(&ship_dir)?;

    let config = ProjectConfig {
        providers: vec!["gemini".to_string()],
        agents: vec![AgentProfile {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            mcp_servers: vec!["github".to_string()],
            ..Default::default()
        }],
        mcp_servers: vec![McpServerConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }],
        ..Default::default()
    };

    save_config(&config, Some(ship_dir.clone()))?;
    let loaded = get_config(Some(ship_dir))?;

    assert_eq!(loaded.providers, vec!["gemini".to_string()]);
    assert!(loaded.agent.skills.is_empty());
    assert_eq!(loaded.agents.len(), 1);
    assert_eq!(loaded.agents[0].id, "focus");
    assert_eq!(loaded.mcp_servers.len(), 1);
    assert_eq!(loaded.mcp_servers[0].id, "github");
    Ok(())
}
