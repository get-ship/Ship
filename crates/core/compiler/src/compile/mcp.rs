use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType};

use super::harness::HarnessDescriptor;

pub(super) fn build_mcp_servers(
    desc: &HarnessDescriptor,
    servers: &[McpServerConfig],
    studio_url: Option<&str>,
    ship_home_dir: Option<&str>,
) -> Json {
    let mut map = serde_json::Map::new();

    // Ship's own self-hosted MCP server is always injected first.
    // When Studio integration is active, emit HTTP transport instead of stdio.
    let ship_entry = match studio_url {
        Some(url) => ship_server_entry_http(url, desc.emit_type_field),
        None => ship_server_entry(desc.emit_type_field, ship_home_dir),
    };
    map.insert("ship".to_string(), ship_entry);

    for s in servers {
        if s.disabled || s.id == "ship" {
            continue;
        }
        map.insert(s.id.clone(), server_entry(desc, s));
    }

    Json::Object(map)
}

pub(super) fn server_entry(desc: &HarnessDescriptor, s: &McpServerConfig) -> Json {
    let mut entry = serde_json::json!({});
    match s.server_type {
        McpServerType::Stdio => {
            entry["command"] = Json::String(s.command.clone());
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                entry["env"] = serde_json::json!(s.env);
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("stdio".to_string());
            }
        }
        McpServerType::Sse => {
            if let Some(url) = &s.url {
                entry[desc.sse_url_field] = Json::String(url.clone());
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("sse".to_string());
            }
        }
        McpServerType::Http => {
            if let Some(url) = &s.url {
                entry[desc.http_url_field] = Json::String(url.clone());
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("http".to_string());
            }
        }
    }
    if let Some(t) = s.timeout_secs {
        entry["startup_timeout_sec"] = serde_json::json!(t);
    }
    entry
}

pub(super) fn ship_server_entry(emit_type: bool, ship_home_dir: Option<&str>) -> Json {
    // The compiler is WASM-safe and cannot depend on `ship_substrate::ShipHome` or
    // read environment variables. Callers (CLI/daemon) inject the resolved
    // home path; when absent (e.g. WASM frontends), we omit the env block
    // entirely and let the spawned `ship mcp serve` resolve its own home via
    // the standard `SHIP_HOME` / default discovery path.
    let mut e = match ship_home_dir {
        Some(dir) => serde_json::json!({
            "command": "ship",
            "args": ["mcp", "serve"],
            "env": { "SHIP_HOME": dir }
        }),
        None => serde_json::json!({
            "command": "ship",
            "args": ["mcp", "serve"]
        }),
    };
    if emit_type {
        e["type"] = Json::String("stdio".to_string());
    }
    e
}

/// Emit an HTTP streamable-http transport entry pointing at Studio's agent endpoint.
pub(super) fn ship_server_entry_http(url: &str, emit_type: bool) -> Json {
    let mut e = serde_json::json!({ "url": url });
    if emit_type {
        e["type"] = Json::String("http".to_string());
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_server_entry_is_stdio() {
        let entry = ship_server_entry(false, None);
        assert_eq!(entry["command"], "ship");
        assert_eq!(entry["args"][0], "mcp");
        assert_eq!(entry["args"][1], "serve");
        assert!(entry.get("url").is_none());
        assert!(entry.get("env").is_none(), "env omitted when home not injected");
    }

    #[test]
    fn ship_server_entry_uses_injected_home_dir() {
        let entry = ship_server_entry(false, Some("/var/lib/ship"));
        assert_eq!(entry["env"]["SHIP_HOME"], "/var/lib/ship");
    }

    #[test]
    fn ship_server_entry_http_has_url_no_command() {
        let entry = ship_server_entry_http("http://localhost:51741/agent", false);
        assert_eq!(entry["url"], "http://localhost:51741/agent");
        assert!(entry.get("command").is_none());
        assert!(entry.get("args").is_none());
    }

    #[test]
    fn build_mcp_servers_no_studio_uses_stdio() {
        use crate::compile::harness::get_harness;
        let desc = get_harness("claude").unwrap();
        let result = build_mcp_servers(desc, &[], None, None);
        let ship = &result["ship"];
        assert_eq!(ship["command"], "ship");
    }

    #[test]
    fn build_mcp_servers_with_studio_uses_http() {
        use crate::compile::harness::get_harness;
        let desc = get_harness("claude").unwrap();
        let result = build_mcp_servers(desc, &[], Some("http://localhost:51741/agent"), None);
        let ship = &result["ship"];
        assert_eq!(ship["url"], "http://localhost:51741/agent");
        assert!(ship.get("command").is_none());
    }
}
