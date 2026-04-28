use super::*;
use crate::project::init_project;
use std::collections::HashMap;
use tempfile::tempdir;

// ── MCP server CRUD ────────────────────────────────────────────────────────

#[test]
fn add_and_list_mcp_server() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let dir = init_project(tmp.path().to_path_buf())?;
    let server = McpServerConfig {
        id: "github".to_string(),
        name: "GitHub".to_string(),
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@mcp/github".to_string()],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    };
    add_mcp_server(Some(dir.clone()), server)?;
    let servers = list_mcp_servers(Some(dir))?;
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].id, "github");
    Ok(())
}

#[test]
fn remove_mcp_server_works() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let dir = init_project(tmp.path().to_path_buf())?;
    let server = McpServerConfig {
        id: "to-remove".to_string(),
        name: "Remove Me".to_string(),
        command: "rm".to_string(),
        args: vec![],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    };
    add_mcp_server(Some(dir.clone()), server)?;
    remove_mcp_server(Some(dir.clone()), "to-remove")?;
    let servers = list_mcp_servers(Some(dir))?;
    assert!(servers.is_empty());
    Ok(())
}

#[test]
fn duplicate_mcp_server_rejected() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let dir = init_project(tmp.path().to_path_buf())?;
    let server = McpServerConfig {
        id: "dup".to_string(),
        name: "Dup".to_string(),
        command: "x".to_string(),
        args: vec![],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    };
    add_mcp_server(Some(dir.clone()), server.clone())?;
    let result = add_mcp_server(Some(dir), server);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
    Ok(())
}

// ── Hook CRUD ──────────────────────────────────────────────────────────────

#[test]
fn add_and_list_hook() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let dir = init_project(tmp.path().to_path_buf())?;
    let hook = HookConfig {
        id: "log-tools".to_string(),
        trigger: HookTrigger::PostToolUse,
        matcher: Some("Bash".to_string()),
        timeout_ms: None,
        description: None,
        command: "echo 'tool used'".to_string(),
    };
    add_hook(Some(dir.clone()), hook)?;
    let hooks = list_hooks(Some(dir))?;
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].id, "log-tools");
    assert_eq!(hooks[0].trigger, HookTrigger::PostToolUse);
    Ok(())
}

#[test]
fn remove_hook_works() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let dir = init_project(tmp.path().to_path_buf())?;
    let hook = HookConfig {
        id: "bye".to_string(),
        trigger: HookTrigger::Stop,
        matcher: None,
        timeout_ms: None,
        description: None,
        command: "echo bye".to_string(),
    };
    add_hook(Some(dir.clone()), hook)?;
    remove_hook(Some(dir.clone()), "bye")?;
    assert!(list_hooks(Some(dir))?.is_empty());
    Ok(())
}

#[test]
fn duplicate_hook_rejected() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let dir = init_project(tmp.path().to_path_buf())?;
    let hook = HookConfig {
        id: "dup".to_string(),
        trigger: HookTrigger::PreToolUse,
        matcher: None,
        timeout_ms: None,
        description: None,
        command: "x".to_string(),
    };
    add_hook(Some(dir.clone()), hook.clone())?;
    let result = add_hook(Some(dir), hook);
    assert!(result.is_err());
    Ok(())
}

// ── Mode CRUD ──────────────────────────────────────────────────────────────

// Tests for add_agent / remove_agent / set_active_agent removed
// alongside the dead `ship mode` chain. Rebuild when a real
// active-agent UX lands.

#[test]
fn mcp_server_type_serialization_round_trips() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let dir = init_project(tmp.path().to_path_buf())?;
    let http_server = McpServerConfig {
        id: "http-svc".to_string(),
        name: "HTTP".to_string(),
        command: String::new(),
        args: vec![],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Http,
        url: Some("http://localhost:8080".to_string()),
        disabled: false,
        timeout_secs: Some(30),
    };
    add_mcp_server(Some(dir.clone()), http_server)?;
    let servers = list_mcp_servers(Some(dir))?;
    assert_eq!(servers[0].server_type, McpServerType::Http);
    assert_eq!(servers[0].url.as_deref(), Some("http://localhost:8080"));
    assert_eq!(servers[0].timeout_secs, Some(30));
    Ok(())
}
