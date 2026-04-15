// Included by crates/core/runtime/src/agents/export/mod.rs.
// MCP per-server entry builders (ship self-entry + JSON/TOML).

fn ship_server_entry() -> (&'static str, serde_json::Value) {
    let command = if cfg!(debug_assertions) {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| "ship".to_string())
    } else {
        "ship".to_string()
    };
    let entry = serde_json::json!({
        "command": command,
        "args": ["mcp", "serve"],
        "type": "stdio",
        "_ship": { "managed": true }
    });
    ("ship", entry)
}

fn json_mcp_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> serde_json::Value {
    match s.server_type {
        McpServerType::Stdio => {
            let mut entry = serde_json::json!({ "command": s.command });
            if desc.emit_type_field {
                entry["type"] = serde_json::json!("stdio");
            }
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                entry["env"] = serde_json::json!(s.env);
            }
            entry
        }
        McpServerType::Http | McpServerType::Sse => {
            let mut entry = serde_json::json!({ desc.http_url_field: s.url });
            if desc.emit_type_field {
                let type_str = if matches!(s.server_type, McpServerType::Sse) {
                    "sse"
                } else {
                    "http"
                };
                entry["type"] = serde_json::json!(type_str);
            }
            if let Some(t) = s.timeout_secs {
                // Gemini timeout is in ms
                let key = "timeout";
                entry[key] = serde_json::json!(if desc.http_url_field == "httpUrl" {
                    t * 1000
                } else {
                    t
                });
            }
            entry
        }
    }
}

fn toml_mcp_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> toml::Value {
    let mut entry = toml::value::Table::new();
    match s.server_type {
        McpServerType::Stdio => {
            entry.insert("command".into(), toml::Value::String(s.command.clone()));
            if !s.args.is_empty() {
                entry.insert(
                    "args".into(),
                    toml::Value::Array(
                        s.args
                            .iter()
                            .map(|a| toml::Value::String(a.clone()))
                            .collect(),
                    ),
                );
            }
            if !s.env.is_empty() {
                let env: toml::value::Table = s
                    .env
                    .iter()
                    .map(|(k, v)| (k.clone(), toml::Value::String(v.clone())))
                    .collect();
                entry.insert("env".into(), toml::Value::Table(env));
            }
        }
        McpServerType::Http | McpServerType::Sse => {
            if let Some(url) = &s.url {
                entry.insert(desc.http_url_field.into(), toml::Value::String(url.clone()));
            }
            // Bearer token: if env has a *_TOKEN or *_KEY, surface it
            for k in s.env.keys() {
                if k.ends_with("_TOKEN") || k.ends_with("_KEY") {
                    entry.insert(
                        "bearer_token_env_var".into(),
                        toml::Value::String(k.clone()),
                    );
                    break;
                }
            }
        }
    }
    if let Some(t) = s.timeout_secs {
        entry.insert("startup_timeout_sec".into(), toml::Value::Integer(t as i64));
    }
    toml::Value::Table(entry)
}
