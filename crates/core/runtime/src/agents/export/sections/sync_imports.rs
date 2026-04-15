// Included by crates/core/runtime/src/agents/export/mod.rs.
// Provider MCP server imports (JSON + TOML) and permission import bootstrap.

pub fn import_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<usize> {
    let desc = require_provider(provider_id)?;
    let (managed, _) =
        crate::db::managed_state::get_managed_state_db(provider_id).unwrap_or_default();

    let mut config = get_config(Some(project_dir.clone()))?;
    let mut added = 0usize;
    let import_paths = provider_import_paths(desc, &project_dir)?;

    for path in import_paths {
        if !path.exists() {
            continue;
        }
        let imported_scope = import_scope_for_path(desc, &project_dir, &path);
        let imported_servers = match desc.config_format {
            ConfigFormat::Json => import_mcp_servers_from_json(desc, &path)?,
            ConfigFormat::Toml => import_mcp_servers_from_toml(desc, &path)?,
        };

        for server in imported_servers {
            let Some(mut server) = normalize_imported_server(server) else {
                continue;
            };
            server.scope = imported_scope.clone();
            if managed.contains(&server.id) {
                continue;
            }
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
        crate::config::save_config(&config, Some(project_dir))?;
    }
    Ok(added)
}

fn import_scope_for_path(desc: &ProviderDescriptor, project_dir: &Path, path: &Path) -> String {
    let project_scope_path = project_dir
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

    // Ship server is runtime-managed and always injected on export.
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

/// Import provider-native permission settings into canonical
/// `.ship/agents/permissions.toml`.
///
/// Returns `true` when permissions were imported and saved, `false` when no
/// importable permissions were found for the provider.
pub fn import_permissions_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<bool> {
    let imported = match provider_id {
        "claude" => import_permissions_from_claude(&project_dir)?,
        "gemini" => import_permissions_from_gemini(&project_dir)?,
        "codex" => import_permissions_from_codex(&project_dir)?,
        _ => return Err(anyhow!("Unsupported provider '{}'", provider_id)),
    };

    if let Some(permissions) = imported {
        crate::permissions::save_permissions(project_dir, &permissions)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn provider_import_paths(desc: &ProviderDescriptor, project_dir: &Path) -> Result<Vec<PathBuf>> {
    let project_path_opt = project_dir
        .parent()
        .map(|project_root| project_root.join(desc.project_config));
    let global_path = home()?.join(desc.global_config);
    if let Some(project_path) = project_path_opt.as_ref()
        && project_path.exists()
    {
        // Prefer project config when present. Ship users typically commit to project-scoped
        // config ownership, so mixing in global provider state here is surprising.
        return Ok(vec![project_path.clone()]);
    }

    if global_path.exists() {
        return Ok(vec![global_path.clone()]);
    }

    // No files found yet; return candidate paths (project first, then global) for
    // callers that want to surface diagnostics or future file creation guidance.
    let mut candidates = Vec::new();
    if let Some(project_path) = project_path_opt {
        candidates.push(project_path);
    }
    if !candidates.iter().any(|path| path == &global_path) {
        candidates.push(global_path);
    }
    Ok(candidates)
}

fn import_mcp_servers_from_json(
    desc: &ProviderDescriptor,
    path: &Path,
) -> Result<Vec<McpServerConfig>> {
    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
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

fn import_mcp_servers_from_toml(
    desc: &ProviderDescriptor,
    path: &Path,
) -> Result<Vec<McpServerConfig>> {
    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let Some(mcp_table) = root.get(desc.mcp_key).and_then(|v| v.as_table()) else {
        return Ok(Vec::new());
    };

    let mut servers = Vec::new();
    for (id, entry) in mcp_table {
        let Some(table) = entry.as_table() else {
            continue;
        };

        let command = table
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = table
            .get(desc.http_url_field)
            .or_else(|| table.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let server_type = match table.get("type").and_then(|v| v.as_str()) {
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
            args: table
                .get("args")
                .and_then(|v| v.as_array())
                .map(|args| {
                    args.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            env: table
                .get("env")
                .and_then(|v| v.as_table())
                .map(|env| {
                    env.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: table
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or_default(),
            timeout_secs: table
                .get("startup_timeout_sec")
                .and_then(|v| v.as_integer())
                .and_then(|v| u32::try_from(v).ok()),
        });
    }
    Ok(servers)
}
