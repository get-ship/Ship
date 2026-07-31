//! MCP server registry CRUD — persists the server list to
//! `.ship/mcp.jsonc` and reads/writes through a small direct-file helper.
//!
//! Slice 2c landed these CRUD services on top of `ship_settings::project`'s
//! `SpaceManifest.mcp_servers` / `mcp.jsonc` IO. The A2 close-out
//! (2026-07-11) deleted the `project` module — its IO helpers `get_config` /
//! `save_config` were the project's last live consumer chain (see killmap
//! §A2). The CRUD public surface stays live for the slice-2c service
//! consumers (`agents/mcp CRUD` RPC), but the storage path is now a direct
//! read/write over `mcp.jsonc` instead of round-tripping through a
//! `SpaceManifest`. The MCP server types move inline.
//!
//! [`McpServerConfig`] is the same shape `harness_config_compiler::compile::harness` reads
//! from the resolved descriptor's `mcp_servers` field — duplicated here only
//! because `ship-settings` no longer `pub use stack_model::*` (the unified
//! descriptor surface lands with slice 5+).

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── MCP types (carried from the deleted `project` module) ────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    #[default]
    Stdio,
    Http,
}

impl std::fmt::Display for McpServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpServerType::Stdio => f.write_str("stdio"),
            McpServerType::Http => f.write_str("http"),
        }
    }
}
/// MCP server registration. The runtime stitches one of these into the
/// resolved descriptor's `mcp_servers`; `harness_config_compiler::compile::harness`
/// materializes it for the harness (`.mcp.json` for Claude, etc.).
///
/// A2: the legacy `name/scope/disabled/timeout_secs` UI-facing fields
/// stay for the slice-2c CRUD RPC surface (McpService RPC). The
/// resolved descriptor's `McpServerConfig` is the typed compile input;
/// the runtime CRUD layer carries the extra display knobs. Future
/// slices collapse both shapes once the descriptor surface unifies.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct McpServerConfig {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default, rename = "server_type")]
    pub server_type: McpServerType,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct McpSection {
    #[serde(default)]
    servers: HashMap<String, McpServerConfig>,
}
fn default_scope() -> String {
    "global".to_string()
}

/// On-disk mcp.jsonc envelope (private — the public surface is
/// the flat `McpServerConfig` list).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct McpConfig {
    #[serde(default)]
    mcp: McpSection,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn mcp_config_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("mcp.jsonc")
}

fn read_mcp_file(ship_dir: &Path) -> Result<Vec<McpServerConfig>> {
    use std::fs;
    let path = mcp_config_path(ship_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    let raw: McpConfig =
        serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(&content))?;
    let mut servers = Vec::new();
    for (id, mut server) in raw.mcp.servers {
        server.id = id;
        servers.push(server);
    }
    servers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(servers)
}

fn write_mcp_file(ship_dir: &Path, servers: &[McpServerConfig]) -> Result<()> {
    use ship_substrate::fs_util::write_atomic;
    use std::fs;
    let path = mcp_config_path(ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut by_id: HashMap<String, McpServerConfig> = HashMap::new();
    for server in servers {
        let mut cloned = server.clone();
        cloned.id.clear();
        by_id.insert(server.id.clone(), cloned);
    }
    let raw = McpConfig {
        mcp: McpSection { servers: by_id },
    };
    write_atomic(&path, serde_json::to_string_pretty(&raw)?)?;
    Ok(())
}

/// Create the standard `.ship/` subdirectories used by built-in modules.
/// Replaces the old configurable `namespaces` registry — every project
/// gets the same baseline tree.
pub fn ensure_standard_dirs(ship_path: &std::path::Path) -> Result<()> {
    let base = ship_substrate::project::stack_root_from_path(ship_path)
        .unwrap_or_else(|| ship_path.to_path_buf());
    for sub in ["project", "agents"] {
        std::fs::create_dir_all(base.join(sub))?;
    }
    Ok(())
}

// ── MCP Server Registry CRUD ─────────────────────────────────────────────────

pub fn add_mcp_server(ship_dir: Option<PathBuf>, server: McpServerConfig) -> Result<()> {
    let Some(ship_dir) = ship_dir else {
        return Err(anyhow!("add_mcp_server requires a project ship_dir"));
    };
    let mut servers = read_mcp_file(&ship_dir)?;
    if servers.iter().any(|s| s.id == server.id) {
        return Err(anyhow!("MCP server '{}' already exists", server.id));
    }
    servers.push(server);
    write_mcp_file(&ship_dir, &servers)
}

pub fn remove_mcp_server(ship_dir: Option<PathBuf>, id: &str) -> Result<()> {
    let Some(ship_dir) = ship_dir else {
        return Err(anyhow!("remove_mcp_server requires a project ship_dir"));
    };
    let mut servers = read_mcp_file(&ship_dir)?;
    servers.retain(|s| s.id != id);
    write_mcp_file(&ship_dir, &servers)
}

pub fn list_mcp_servers(ship_dir: Option<PathBuf>) -> Result<Vec<McpServerConfig>> {
    let Some(ship_dir) = ship_dir else {
        return Err(anyhow!("list_mcp_servers requires a project ship_dir"));
    };
    read_mcp_file(&ship_dir)
}
