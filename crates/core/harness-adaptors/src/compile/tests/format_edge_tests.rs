//! Edge-case tests for format translation functions.
//!
//! These test the tricky corners of json→toml, glob→regex,
//! and permission translation that aren't covered by the
//! happy-path integration tests in each harness's test file.

use crate::compile::compile;
use stack_model::resolve::ResolvedConfig;
use stack_model::types::{Permissions, ToolPermissions};

use super::fixtures::*;

// ── json_to_toml edge cases (tested via Codex settings_extra) ───────────────

#[test]
fn codex_settings_extra_nested_object_becomes_toml_table() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            settings_extra: Some(serde_json::json!({
                "agents": { "max_threads": 8, "max_depth": 5 }
            })),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let agents = parsed["agents"].as_table().unwrap();
    assert_eq!(agents["max_threads"].as_integer().unwrap(), 8);
    assert_eq!(agents["max_depth"].as_integer().unwrap(), 5);
}

#[test]
fn codex_settings_extra_array_becomes_toml_array() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            settings_extra: Some(serde_json::json!({
                "allowed_hosts": ["github.com", "npmjs.org"]
            })),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let hosts = parsed["allowed_hosts"].as_array().unwrap();
    assert_eq!(hosts.len(), 2);
    assert_eq!(hosts[0].as_str().unwrap(), "github.com");
}

#[test]
fn codex_settings_extra_null_value_dropped() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            settings_extra: Some(serde_json::json!({
                "keep": "yes",
                "drop": null
            })),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    assert!(parsed.contains_key("keep"));
    assert!(
        !parsed.contains_key("drop"),
        "null values must not appear in TOML"
    );
}

#[test]
fn codex_settings_extra_float_preserved() {
    let r = ResolvedConfig {
        codex: crate::compile::CodexSettings {
            settings_extra: Some(serde_json::json!({ "temperature": 0.7 })),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let temp = parsed["temperature"].as_float().unwrap();
    assert!((temp - 0.7).abs() < 0.001);
}

// ── OpenCode permission translation edge cases ──────────────────────────────

#[test]
fn opencode_unknown_tool_pattern_dropped() {
    // A tool pattern that doesn't map to any known OpenCode permission key
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["UnknownTool".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![make_server("test")])
    };
    let out = compile(&r, "opencode").unwrap();
    let patch = out
        .opencode_config_patch
        .expect("servers must trigger patch");
    // Unknown patterns are silently dropped — permission key should not appear
    assert!(
        patch.get("permission").is_none(),
        "unknown tool pattern should not produce a permission entry"
    );
}

#[test]
fn opencode_mixed_allow_deny_same_tool_produces_granular() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["Bash(git commit *)".to_string()],
                deny: vec!["Bash(git push --force *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![make_server("test")])
    };
    let out = compile(&r, "opencode").unwrap();
    let patch = out.opencode_config_patch.unwrap();
    let bash = &patch["permission"]["bash"];
    assert!(bash.is_object(), "mixed bash patterns → granular object");
    assert_eq!(bash["git commit *"].as_str().unwrap(), "allow");
    assert_eq!(bash["git push --force *"].as_str().unwrap(), "deny");
}
