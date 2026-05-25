//! MCP server management — add, list, remove entries in `.ship/mcp.jsonc`.

use anyhow::Result;
use std::collections::HashMap;

use crate::paths::mcp_path;

pub use ship_platform::services::compile::{McpEntry, McpFile};

pub fn add_http(id: &str, name: Option<String>, url: &str) -> Result<()> {
    let path = mcp_path();
    let mut file = McpFile::load(&path)?;
    if file.servers.iter().any(|s| s.id == id) {
        anyhow::bail!("MCP server '{}' already registered. Remove it first.", id);
    }
    file.servers.push(McpEntry {
        id: id.to_string(),
        name,
        command: None,
        args: vec![],
        env: HashMap::new(),
        url: Some(url.to_string()),
        scope: "project".into(),
        server_type: Some("http".into()),
        disabled: false,
    });
    file.save(&path)?;
    println!("✓ registered MCP server '{}'", id);
    Ok(())
}

pub fn add_stdio(id: &str, name: Option<String>, command: &str, args: Vec<String>) -> Result<()> {
    let path = mcp_path();
    let mut file = McpFile::load(&path)?;
    if file.servers.iter().any(|s| s.id == id) {
        anyhow::bail!("MCP server '{}' already registered. Remove it first.", id);
    }
    file.servers.push(McpEntry {
        id: id.to_string(),
        name,
        command: Some(command.to_string()),
        args,
        env: HashMap::new(),
        url: None,
        scope: "project".into(),
        server_type: Some("stdio".into()),
        disabled: false,
    });
    file.save(&path)?;
    println!("✓ registered MCP server '{}'", id);
    Ok(())
}

pub fn list() -> Result<()> {
    let path = mcp_path();
    let file = McpFile::load(&path)?;
    if file.servers.is_empty() {
        println!("No MCP servers configured.");
        println!("Add one with: ship mcp add-stdio <id> <command> [args...]");
        return Ok(());
    }
    println!("MCP servers:");
    for s in &file.servers {
        let transport = if s.url.is_some() { "http/sse" } else { "stdio" };
        let status = if s.disabled { " (disabled)" } else { "" };
        let name = s.name.as_deref().unwrap_or(&s.id);
        println!("  {} — {} [{}]{}", s.id, name, transport, status);
    }
    Ok(())
}

pub fn remove(id: &str) -> Result<()> {
    let path = mcp_path();
    let mut file = McpFile::load(&path)?;
    let before = file.servers.len();
    file.servers.retain(|s| s.id != id);
    if file.servers.len() == before {
        anyhow::bail!("MCP server '{}' not found", id);
    }
    file.save(&path)?;
    println!("✓ removed MCP server '{}'", id);
    Ok(())
}
