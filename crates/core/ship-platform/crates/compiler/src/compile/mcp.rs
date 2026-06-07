use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType};

use super::harness::HarnessDescriptor;

pub(super) fn build_mcp_servers(
    desc: &HarnessDescriptor,
    servers: &[McpServerConfig],
) -> Json {
    let mut map = serde_json::Map::new();

    for s in servers {
        if s.disabled {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_stdio_server_emits_command_and_args() {
        use crate::compile::harness::get_harness;
        use crate::types::{McpServerConfig, McpServerType};
        let desc = get_harness("claude").unwrap();
        let s = McpServerConfig {
            id: "context7".to_string(),
            name: "context7".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@upstash/context7-mcp".to_string()],
            env: Default::default(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        };
        let result = build_mcp_servers(desc, &[s]);
        let entry = &result["context7"];
        assert_eq!(entry["command"], "npx");
        assert_eq!(entry["args"][0], "-y");
        assert_eq!(entry["args"][1], "@upstash/context7-mcp");
        // No ship injection present.
        assert!(result.get("ship").is_none());
    }

    #[test]
    fn disabled_server_excluded() {
        use crate::compile::harness::get_harness;
        use crate::types::{McpServerConfig, McpServerType};
        let desc = get_harness("claude").unwrap();
        let s = McpServerConfig {
            id: "disabled-svc".to_string(),
            name: "disabled-svc".to_string(),
            command: "npx".to_string(),
            args: vec![],
            env: Default::default(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: true,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        };
        let result = build_mcp_servers(desc, &[s]);
        assert!(result.as_object().unwrap().is_empty());
    }

    #[test]
    fn http_server_uses_http_url_field() {
        use crate::compile::harness::get_harness;
        use crate::types::{McpServerConfig, McpServerType};
        let desc = get_harness("claude").unwrap();
        let s = McpServerConfig {
            id: "remote".to_string(),
            name: "remote".to_string(),
            command: String::new(),
            args: vec![],
            env: Default::default(),
            scope: "project".to_string(),
            server_type: McpServerType::Http,
            url: Some("https://api.example.com/mcp".to_string()),
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        };
        let result = build_mcp_servers(desc, &[s]);
        assert_eq!(result["remote"]["url"], "https://api.example.com/mcp");
        assert!(result["remote"].get("command").is_none());
    }
}
