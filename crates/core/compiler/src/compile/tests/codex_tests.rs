use crate::compile::compile;
use crate::resolve::ResolvedConfig;

use super::fixtures::*;

/// Codex lifecycle hooks use the same pod/host hybrid command as claude:
/// curl the daemon over `$SHIP_DAEMON_URL` with the scoped `$SHIP_TOKEN`
/// inside a sandbox, fall back to `ship hook` on the host. One config,
/// both tiers — the isolation tier is only known per-spawn.
#[test]
fn codex_hooks_are_pod_and_host_hybrid() {
    let out = compile(&resolved(vec![]), "codex").unwrap();
    let hooks = out.codex_hooks_json.expect("codex emits hooks.json");
    // Find the PreToolUse entry's command.
    let cmd = hooks["hooks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event"] == "PreToolUse")
        .expect("PreToolUse hook")["hooks"][0]["command"]
        .as_str()
        .expect("command string");
    assert!(cmd.contains("$SHIP_DAEMON_URL/api/v1/hooks/before-tool"), "got: {cmd}");
    assert!(cmd.contains("Authorization: Bearer $SHIP_TOKEN"), "got: {cmd}");
    assert!(cmd.contains("ship hook before-tool"), "host fallback: {cmd}");
}

/// Source: https://developers.openai.com/codex/mcp
/// Codex config is TOML — codex_config_patch must be a valid TOML string.
#[test]
fn codex_produces_toml_patch_not_json() {
    let r = resolved(vec![make_server("github")]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out
        .codex_config_patch
        .expect("codex must emit a TOML config patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
    assert!(
        parsed.contains_key("mcp_servers"),
        "must have mcp_servers table"
    );
}

#[test]
fn codex_toml_no_ship_injection() {
    // Ship-internal MCP removed (#236): codex must not auto-inject a ship entry.
    let r = resolved(vec![make_server("github")]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let mcp = parsed["mcp_servers"].as_table().unwrap();
    assert!(
        !mcp.contains_key("ship"),
        "codex must not inject a 'ship' MCP entry"
    );
}

#[test]
fn codex_toml_stdio_entry_shape() {
    let mut s = make_server("context7");
    s.args = vec!["-y".into(), "@upstash/context7-mcp".into()];
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let entry = parsed["mcp_servers"]["context7"].as_table().unwrap();
    assert_eq!(entry["command"].as_str().unwrap(), "npx");
    assert_eq!(entry["args"].as_array().unwrap().len(), 2);
    assert!(entry.get("type").is_none(), "no type field in Codex TOML");
}

#[test]
fn codex_toml_disabled_server_excluded() {
    let mut s = make_server("disabled");
    s.disabled = true;
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let mcp = parsed["mcp_servers"].as_table().unwrap();
    assert!(!mcp.contains_key("disabled"));
}

#[test]
fn codex_user_defined_ship_server_passes_through() {
    // A user who defines their own "ship"-named external MCP gets it compiled
    // like any other server — no special-casing, no silent drop.
    let s = make_server("ship");
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let mcp = parsed["mcp_servers"].as_table().unwrap();
    // The user's entry must be present with their command (npx, not ship-internal).
    let ship = mcp["ship"].as_table().unwrap();
    assert_eq!(
        ship["command"].as_str().unwrap(),
        "npx",
        "user-defined ship server command must be preserved"
    );
}

#[test]
fn codex_toml_timeout_uses_startup_timeout_sec() {
    let mut s = make_server("slow");
    s.timeout_secs = Some(30);
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    assert_eq!(
        parsed["mcp_servers"]["slow"]["startup_timeout_sec"]
            .as_integer()
            .unwrap(),
        30
    );
}

#[test]
fn codex_no_json_mcp_key_confusion() {
    let r = resolved(vec![make_server("x")]);
    assert!(compile(&r, "codex").unwrap().codex_config_patch.is_some());
    assert!(compile(&r, "claude").unwrap().codex_config_patch.is_none());
    assert!(compile(&r, "cursor").unwrap().codex_config_patch.is_none());
}

#[test]
fn codex_model_emitted_in_config_patch() {
    let r = ResolvedConfig {
        model: Some("o3".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("model should trigger codex patch");
    assert!(
        patch.contains("model = \"o3\""),
        "codex TOML should contain model field: {}",
        patch
    );
}

// ── Codex sandbox mode (Phase 1B) ────────────────────────────────────────────

/// Phase 1B: key must be `sandbox_mode` (not `sandbox`), values translated.
#[test]
fn codex_sandbox_key_is_sandbox_mode() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            sandbox: Some("network-only".to_string()),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("sandbox should trigger codex patch");
    assert!(
        !patch.contains("sandbox ="),
        "old key 'sandbox' must NOT be emitted: {}",
        patch
    );
    assert!(
        patch.contains("sandbox_mode"),
        "new key 'sandbox_mode' must be emitted: {}",
        patch
    );
}

#[test]
fn codex_sandbox_values_translated_correctly() {
    let cases = [
        ("full", "danger-full-internet"),
        ("network-only", "network-disabled"),
        ("off", "disabled"),
    ];
    for (input, expected) in cases {
        let r = ResolvedConfig {
            codex: crate::compile::CodexSettings {
                sandbox: Some(input.to_string()),
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "codex").unwrap();
        let patch = out.codex_config_patch.as_ref().unwrap();
        assert!(
            patch.contains(&format!("sandbox_mode = \"{expected}\"")),
            "input '{}' should produce sandbox_mode = '{}': {}",
            input,
            expected,
            patch
        );
    }
}

#[test]
fn codex_sandbox_none_omitted() {
    let r = resolved(vec![]);
    let out = compile(&r, "codex").unwrap();
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("codex always emits a config patch");
    assert!(
        !patch.contains("sandbox_mode"),
        "codex TOML should not contain sandbox_mode when None: {}",
        patch
    );
    assert!(
        !patch.contains("sandbox ="),
        "codex TOML should not contain sandbox when None: {}",
        patch
    );
}

#[test]
fn codex_sandbox_not_emitted_for_other_harnesses() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            sandbox: Some("full".to_string()),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    for harness in others("codex") {
        let out = compile(&r, harness).unwrap();
        assert!(
            out.codex_config_patch.is_none(),
            "{harness} must not get codex config patch"
        );
    }
}

// ── Phase 4: new Codex fields ─────────────────────────────────────────────────

/// Model from library flows through to codex config as `model = "..."`.
#[test]
fn library_model_flows_to_codex_config() {
    use crate::resolve::{ProjectLibrary, resolve_library};
    let library = ProjectLibrary {
        model: Some("o4-mini".to_string()),
        ..Default::default()
    };
    let resolved = resolve_library(&library, None, None);
    let out = compile(&resolved, "codex").unwrap();
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("model must trigger codex patch");
    assert!(
        patch.contains("model = \"o4-mini\""),
        "model must be in codex config: {}",
        patch
    );
}

#[test]
fn codex_approval_policy_translated() {
    let cases = [
        ("default", "suggest"),
        ("auto_edit", "auto-edit"),
        ("plan", "full-auto"),
    ];
    for (input, expected) in cases {
        let r = ResolvedConfig {
            codex: crate::compile::CodexSettings {
                approval_policy: Some(input.to_string()),
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "codex").unwrap();
        let patch = out
            .codex_config_patch
            .as_ref()
            .expect("approval policy must trigger patch");
        assert!(
            patch.contains(&format!("approval_policy = \"{expected}\"")),
            "input '{}' should produce approval_policy = '{}': {}",
            input,
            expected,
            patch
        );
    }
}

#[test]
fn codex_reasoning_effort_emitted() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            reasoning_effort: Some("high".to_string()),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(
        patch.contains("model_reasoning_effort = \"high\""),
        "{}",
        patch
    );
}

#[test]
fn codex_agents_table_emitted() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            max_threads: Some(4),
            max_depth: Some(3),
            job_max_runtime_seconds: Some(300),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let agents = parsed.get("agents").expect("agents table must be present");
    assert_eq!(agents["max_threads"].as_integer().unwrap(), 4);
    assert_eq!(agents["max_depth"].as_integer().unwrap(), 3);
    assert_eq!(agents["run_max_runtime_seconds"].as_integer().unwrap(), 300);
}

#[test]
fn codex_shell_environment_policy_emitted() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            shell_env_policy: Some("inherit".to_string()),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(
        patch.contains("shell_environment_policy = \"inherit\""),
        "{}",
        patch
    );
}

#[test]
fn codex_notify_emitted() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            notify: Some(serde_json::json!(true)),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(patch.contains("notify = true"), "{}", patch);
}

#[test]
fn codex_mcp_server_tool_filters_emitted() {
    let mut s = make_server("filtered-server");
    s.codex_enabled_tools = vec!["read".to_string()];
    s.codex_disabled_tools = vec!["delete".to_string()];
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let server = &parsed["mcp_servers"]["filtered-server"];
    let enabled = server["enabled_tools"].as_array().unwrap();
    assert_eq!(enabled[0].as_str().unwrap(), "read");
    let disabled = server["disabled_tools"].as_array().unwrap();
    assert_eq!(disabled[0].as_str().unwrap(), "delete");
}

#[test]
fn codex_settings_extra_merged_verbatim() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            settings_extra: Some(serde_json::json!({ "custom_key": "custom_value" })),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(patch.contains("custom_key"), "{}", patch);
}
