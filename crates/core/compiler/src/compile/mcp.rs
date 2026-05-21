use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType};

use super::harness::HarnessDescriptor;

pub(super) fn build_mcp_servers(
    desc: &HarnessDescriptor,
    servers: &[McpServerConfig],
    ship_home_dir: Option<&str>,
    scoped_launch: bool,
) -> Json {
    let mut map = serde_json::Map::new();

    // Ship's own self-hosted MCP server is always injected first. On a
    // daemon-spawned scoped launch (dispatch/command-bar/sandbox) where the
    // spawner injects `$SHIP_TOKEN` + `$SHIP_DAEMON_URL` AND scope-enforcement
    // matters, a capable harness gets the daemon-hosted authed-HTTP entry. A
    // manual `ship use` (no spawner, human is the host) keeps the trusted-local
    // stdio shim. The env references are resolved by the harness at launch — no
    // live credential is written to disk.
    let ship_entry = if scoped_launch && desc.supports_authed_http_mcp {
        ship_server_entry_authed_http(desc)
    } else {
        ship_server_entry(desc.emit_type_field, ship_home_dir)
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

/// Emit a daemon-hosted streamable-HTTP entry for the self-hosted `ship`
/// server, authenticated by the scoped session token.
///
/// The url and token are `${ENV}` references the harness resolves at launch
/// from the environment the daemon/dispatcher injects (`SHIP_DAEMON_URL`,
/// `SHIP_TOKEN`) — one compiled shape for local + sandboxed agents, with no
/// live credential written to a config file. Only emitted for harnesses with
/// `supports_authed_http_mcp = true` (verified to express `headers` + `${ENV}`
/// expansion). `type: "http"` is required to declare the remote transport;
/// harnesses with a different remote-type token are not yet enabled.
pub(super) fn ship_server_entry_authed_http(desc: &HarnessDescriptor) -> Json {
    let mut e = serde_json::json!({
        "type": "http",
        "headers": { "Authorization": "Bearer ${SHIP_TOKEN}" },
    });
    e[desc.http_url_field] = Json::String("${SHIP_DAEMON_URL}/mcp".to_string());
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
    fn authed_http_entry_uses_env_references_no_disk_secret() {
        use crate::compile::harness::get_harness;
        let desc = get_harness("claude").unwrap();
        let entry = ship_server_entry_authed_http(desc);
        assert_eq!(entry["type"], "http");
        assert_eq!(entry["url"], "${SHIP_DAEMON_URL}/mcp");
        assert_eq!(entry["headers"]["Authorization"], "Bearer ${SHIP_TOKEN}");
        assert!(entry.get("command").is_none(), "authed HTTP entry has no command");
        // No live token baked into the config — only an env reference.
        let serialized = serde_json::to_string(&entry).unwrap();
        assert!(serialized.contains("${SHIP_TOKEN}"));
        assert!(!serialized.contains("ship_sess_"), "no minted token on disk");
    }

    #[test]
    fn scoped_launch_supported_harness_emits_daemon_hosted_http() {
        use crate::compile::harness::get_harness;
        // Claude on a scoped (daemon-spawned) launch → daemon-hosted HTTP.
        let desc = get_harness("claude").unwrap();
        let result = build_mcp_servers(desc, &[], Some("/var/lib/ship"), true);
        let ship = &result["ship"];
        assert_eq!(ship["url"], "${SHIP_DAEMON_URL}/mcp");
        assert!(ship.get("command").is_none());
    }

    #[test]
    fn manual_launch_uses_stdio_even_for_supported_harness() {
        use crate::compile::harness::get_harness;
        // Claude on a manual `ship use` (scoped_launch = false) → stdio shim.
        // The human is the host; there's no spawner to provision $SHIP_TOKEN.
        let desc = get_harness("claude").unwrap();
        let result = build_mcp_servers(desc, &[], Some("/var/lib/ship"), false);
        let ship = &result["ship"];
        assert_eq!(ship["command"], "ship");
        assert!(ship.get("headers").is_none());
    }

    #[test]
    fn unsupported_harness_falls_back_to_stdio_even_when_scoped() {
        use crate::compile::harness::get_harness;
        // Codex is not yet verified for authed HTTP → stdio even on a scoped launch.
        let desc = get_harness("codex").unwrap();
        let result = build_mcp_servers(desc, &[], Some("/var/lib/ship"), true);
        let ship = &result["ship"];
        assert_eq!(ship["command"], "ship");
        assert_eq!(ship["env"]["SHIP_HOME"], "/var/lib/ship");
        assert!(ship.get("headers").is_none());
    }
}
