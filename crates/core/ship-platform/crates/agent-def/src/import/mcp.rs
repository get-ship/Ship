//! Import MCP server entries from a harness's config file (Claude / Cursor /
//! Codex). The source-directory file wins if it exists; the
//! global-scope fallback runs when no source file is present. Imported
//! servers are normalized (id trimmed, "ship" reserved, stdio vs http
//! split on `type`) and merged into the space's MCP config, skipping
//! ids that already exist.

use ship_config::project::{McpServerConfig, McpServerType, get_config};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::harnesses::{ConfigFormat, HarnessDescriptor, require_harness};
use super::home;

pub fn import_from_harness(harness_id: &str, source_dir: PathBuf) -> Result<usize> {
    let desc = require_harness(harness_id)?;
    let mut config = get_config(Some(source_dir.clone()))?;
    let mut added = 0usize;
    let import_paths = harness_import_paths(desc, &source_dir)?;

    for path in import_paths {
        if !path.exists() {
            continue;
        }
        let imported_scope = import_scope_for_path(desc, &source_dir, &path);
        let imported_servers = import_mcp_servers(desc, &path)?;

        for server in imported_servers {
            let Some(mut server) = normalize_imported_server(server) else {
                continue;
            };
            server.scope = imported_scope.clone();
            if config
                .mcp_servers
                .iter()
                .any(|existing| existing.id == server.id)
            {
                continue;
            }
            config.mcp_servers.push(server);
            added += 1;
        }
    }

    if added > 0 {
        ship_config::project::save_config(&config, Some(source_dir))?;
    }
    Ok(added)
}

fn import_scope_for_path(desc: &HarnessDescriptor, source_dir: &Path, path: &Path) -> String {
    let project_scope_path = source_dir
        .parent()
        .map(|root| root.join(desc.project_config));
    if project_scope_path.as_ref().is_some_and(|p| p == path) {
        "project".to_string()
    } else {
        "global".to_string()
    }
}

fn normalize_imported_server(mut server: McpServerConfig) -> Option<McpServerConfig> {
    server.id = server.id.trim().to_string();
    if server.id.is_empty() {
        return None;
    }

    if server.id.eq_ignore_ascii_case("ship") {
        return None;
    }

    if server.name.trim().is_empty() {
        server.name = server.id.clone();
    }

    match server.server_type {
        McpServerType::Stdio => {
            server.command = server.command.trim().to_string();
            if server.command.is_empty() {
                return None;
            }
            server.url = None;
        }
        McpServerType::Sse | McpServerType::Http => {
            let url = server
                .url
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())?
                .to_string();
            server.url = Some(url);
            server.command.clear();
            server.args.clear();
        }
    }

    Some(server)
}

pub(super) fn harness_import_paths(
    desc: &HarnessDescriptor,
    source_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let project_path_opt = source_dir
        .parent()
        .map(|space_root| space_root.join(desc.project_config));
    let global_path = home()?.join(desc.global_config);
    if let Some(project_path) = project_path_opt.as_ref()
        && project_path.exists()
    {
        return Ok(vec![project_path.clone()]);
    }

    if global_path.exists() {
        return Ok(vec![global_path.clone()]);
    }

    let mut candidates = Vec::new();
    if let Some(project_path) = project_path_opt {
        candidates.push(project_path);
    }
    if !candidates.iter().any(|path| path == &global_path) {
        candidates.push(global_path);
    }
    Ok(candidates)
}

/// Read a harness's MCP config (JSON for Claude/Cursor, TOML for Codex)
/// and return its server entries normalised into [`McpServerConfig`].
///
/// TOML harness files are converted to `serde_json::Value` at the boundary so
/// the field-extraction logic stays single-source. Previously this had two
/// near-identical 70-line implementations that drifted on minor details
/// (e.g. `timeout_secs` was only honoured on the JSON side).
fn import_mcp_servers(
    desc: &HarnessDescriptor,
    path: &Path,
) -> Result<Vec<McpServerConfig>> {
    let raw = fs::read_to_string(path)?;
    let root: serde_json::Value = match desc.config_format {
        ConfigFormat::Json => serde_json::from_str(&raw)?,
        ConfigFormat::Toml => {
            let tv: toml::Value = toml::from_str(&raw)?;
            serde_json::to_value(&tv)?
        }
    };
    let Some(mcp_obj) = root.get(desc.mcp_key).and_then(|v| v.as_object()) else {
        return Ok(Vec::new());
    };

    let mut servers = Vec::new();
    for (id, entry) in mcp_obj {
        let command = entry
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = entry
            .get(desc.http_url_field)
            .or_else(|| entry.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let server_type = match entry.get("type").and_then(|v| v.as_str()) {
            Some("sse") => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _ => {
                if command.is_empty() && url.is_some() {
                    McpServerType::Http
                } else {
                    McpServerType::Stdio
                }
            }
        };

        servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command,
            args: entry
                .get("args")
                .and_then(|v| v.as_array())
                .map(|args| {
                    args.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            env: entry
                .get("env")
                .and_then(|v| v.as_object())
                .map(|env| {
                    env.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: entry
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or_default(),
            timeout_secs: entry
                .get("startup_timeout_sec")
                .or_else(|| entry.get("timeout_secs"))
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok()),
        });
    }
    Ok(servers)
}
