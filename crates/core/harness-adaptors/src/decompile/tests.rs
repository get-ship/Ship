//! Decompile module tests.

use serde_json::json;
use tempfile::TempDir;

use super::claude::decompile_claude;
use super::codex::decompile_codex;
use super::opencode::decompile_opencode;
use super::test_helpers::write_file;
use super::{decompile_all, detect_harnesses};
use stack_model::types::HookTrigger;

// ── detect_harnesses ─────────────────────────────────────────────────────────

#[test]
fn detect_empty_dir() {
    let tmp = TempDir::new().unwrap();
    let detected = detect_harnesses(tmp.path());
    assert!(!detected.any());
    assert!(detected.as_list().is_empty());
}

#[test]
fn detect_claude_from_mcp_json() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, ".mcp.json", "{}");
    let detected = detect_harnesses(tmp.path());
    assert!(detected.claude);
    assert_eq!(detected.as_list(), vec!["claude"]);
}

#[test]
fn detect_claude_from_claude_dir() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
    let detected = detect_harnesses(tmp.path());
    assert!(detected.claude);
}

#[test]
fn detect_claude_from_claude_md() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "# Rules");
    let detected = detect_harnesses(tmp.path());
    assert!(detected.claude);
}

#[test]
fn detect_multiple_harnesses() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "# Claude rules");
    std::fs::create_dir_all(tmp.path().join(".codex")).unwrap();
    std::fs::create_dir_all(tmp.path().join(".opencode")).unwrap();
    let detected = detect_harnesses(tmp.path());
    assert_eq!(detected.as_list(), vec!["claude", "codex", "opencode"]);
}

// ══════════════════════════════════════════════════════════════════════════════
// Claude Code
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn claude_settings_permissions() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "permissions": {
            "allow": ["Read", "Write", "Bash(git *)"],
            "deny": ["Bash(rm -rf *)"],
            "ask": ["Bash(cargo publish)"],
            "defaultMode": "acceptEdits",
            "additionalDirectories": ["/home/user/shared"]
        }
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(
        lib.permissions.tools.allow,
        vec!["Read", "Write", "Bash(git *)"]
    );
    assert_eq!(lib.permissions.tools.deny, vec!["Bash(rm -rf *)"]);
    assert_eq!(lib.permissions.tools.ask, vec!["Bash(cargo publish)"]);
    assert_eq!(lib.permissions.default_mode.as_deref(), Some("acceptEdits"));
    assert_eq!(
        lib.permissions.additional_directories,
        vec!["/home/user/shared"]
    );
}

#[test]
fn claude_settings_hooks() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "hooks": {
            "Stop": [
                {"hooks": [{"type": "command", "command": "echo done"}]}
            ],
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [{"type": "command", "command": "echo pre-bash"}]
                }
            ]
        }
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.hooks.len(), 2);
    let stop = lib
        .hooks
        .iter()
        .find(|h| h.trigger == HookTrigger::Stop)
        .unwrap();
    assert_eq!(stop.command, "echo done");
    let pre = lib
        .hooks
        .iter()
        .find(|h| h.trigger == HookTrigger::PreToolUse)
        .unwrap();
    assert_eq!(pre.command, "echo pre-bash");
    assert_eq!(pre.matcher.as_deref(), Some("Bash"));
}

#[test]
fn claude_settings_model_and_env() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "model": "claude-sonnet-4-5-20250514",
        "env": {"API_KEY": "sk-test", "DEBUG": "1"},
        "availableModels": ["claude-sonnet-4-5-20250514", "claude-haiku-3-5-20241022"],
        "maxCostPerSession": 5.0,
        "maxTurns": 100
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("claude-sonnet-4-5-20250514"));
    assert_eq!(lib.env.get("API_KEY").map(String::as_str), Some("sk-test"));
    assert_eq!(lib.env.get("DEBUG").map(String::as_str), Some("1"));
    assert_eq!(lib.available_models.len(), 2);
    assert_eq!(lib.permissions.agent.max_cost_per_session, Some(5.0));
    assert_eq!(lib.permissions.agent.max_turns, Some(100));
}

#[test]
fn claude_settings_theme_and_scalar_fields() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "theme": "dark",
        "autoUpdates": false,
        "includeCoAuthoredBy": true
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.claude_theme.as_deref(), Some("dark"));
    assert_eq!(lib.claude_auto_updates, Some(false));
    assert_eq!(lib.claude_include_co_authored_by, Some(true));
}

#[test]
fn claude_settings_unknown_keys_into_harness_defaults() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "model": "claude-sonnet-4-5-20250514",
        "customSetting": true,
        "experimentalFeatures": {"betaFlag": "on"}
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("claude-sonnet-4-5-20250514"));
    let defaults = lib.harness_defaults.get("claude").unwrap();
    assert_eq!(defaults["customSetting"], true);
    assert_eq!(defaults["experimentalFeatures"]["betaFlag"], "on");
    assert!(defaults.get("model").is_none());
}

#[test]
fn claude_md_imported_as_rule() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "# Project Rules\n\nAlways use TDD.\n");

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "CLAUDE.md");
    assert!(lib.rules[0].content.contains("Always use TDD."));
    assert!(lib.rules[0].always_apply);
}

#[test]
fn claude_md_empty_skipped() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "   \n  ");

    let lib = decompile_claude(tmp.path());
    assert!(lib.rules.is_empty());
}

#[test]
fn mcp_json_servers_parsed() {
    let tmp = TempDir::new().unwrap();
    let mcp = json!({
        "mcpServers": {
            "ship": {
                "command": "ship",
                "args": ["mcp", "serve"],
                "env": {"SHIP_PROJECT": "/home/user/project"}
            },
            "postgres": {
                "url": "http://localhost:5433/mcp",
                "disabled": true
            }
        }
    });
    write_file(&tmp, ".mcp.json", &serde_json::to_string(&mcp).unwrap());

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.mcp_servers.len(), 2);
    let ship = lib.mcp_servers.iter().find(|s| s.id == "ship").unwrap();
    assert_eq!(ship.command, "ship");
    assert_eq!(ship.args, vec!["mcp", "serve"]);
    let pg = lib.mcp_servers.iter().find(|s| s.id == "postgres").unwrap();
    assert_eq!(pg.url.as_deref(), Some("http://localhost:5433/mcp"));
    assert!(pg.disabled);
}

#[test]
fn full_claude_project_decompile() {
    let tmp = TempDir::new().unwrap();

    write_file(&tmp, "CLAUDE.md", "# Engineering\n\nWrite tests first.");
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&json!({
            "permissions": {
                "allow": ["Read", "Glob", "Grep"],
                "deny": ["Bash(rm -rf *)"],
                "defaultMode": "default"
            },
            "hooks": {
                "Stop": [{"hooks": [{"type": "command", "command": "ship log"}]}]
            },
            "model": "claude-opus-4-20250514",
            "theme": "dark",
            "autoMemoryEnabled": false,
            "customPlugin": {"enabled": true}
        }))
        .unwrap(),
    );
    write_file(
        &tmp,
        ".mcp.json",
        &serde_json::to_string(&json!({
            "mcpServers": {
                "ship": {"command": "ship", "args": ["mcp", "serve"]}
            }
        }))
        .unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.permissions.tools.allow, vec!["Read", "Glob", "Grep"]);
    assert_eq!(lib.hooks.len(), 1);
    assert_eq!(lib.model.as_deref(), Some("claude-opus-4-20250514"));
    assert_eq!(lib.mcp_servers.len(), 1);
    let defaults = lib.harness_defaults.get("claude").unwrap();
    assert_eq!(defaults["customPlugin"]["enabled"], true);
}

// ══════════════════════════════════════════════════════════════════════════════
// Codex CLI
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn codex_config_model_and_settings() {
    let tmp = TempDir::new().unwrap();
    let config = r#"
model = "o3"
approval_policy = "auto-edit"
sandbox_mode = "network-disabled"
model_reasoning_effort = "high"
shell_environment_policy = "inherit"

[agents]
max_threads = 4
max_depth = 3
run_max_runtime_seconds = 300
"#;
    write_file(&tmp, ".codex/config.toml", config);

    let lib = decompile_codex(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("o3"));
    assert_eq!(lib.codex_approval_policy.as_deref(), Some("auto_edit"));
    assert_eq!(lib.codex_sandbox.as_deref(), Some("network-only"));
    assert_eq!(lib.codex_reasoning_effort.as_deref(), Some("high"));
    assert_eq!(lib.codex_shell_env_policy.as_deref(), Some("inherit"));
    assert_eq!(lib.codex_max_threads, Some(4));
    assert_eq!(lib.codex_max_depth, Some(3));
    assert_eq!(lib.codex_job_max_runtime_seconds, Some(300));
}

#[test]
fn codex_config_mcp_servers() {
    let tmp = TempDir::new().unwrap();
    let config = r#"
[mcp_servers.ship]
command = "ship"
args = ["mcp", "serve"]

[mcp_servers.pg]
url = "http://localhost:5433/mcp"

[mcp_servers.filtered]
command = "mcp-filtered"
enabled_tools = ["read", "write"]
disabled_tools = ["delete"]
"#;
    write_file(&tmp, ".codex/config.toml", config);

    let lib = decompile_codex(tmp.path());

    assert_eq!(lib.mcp_servers.len(), 3);
    let ship = lib.mcp_servers.iter().find(|s| s.id == "ship").unwrap();
    assert_eq!(ship.command, "ship");

    let filtered = lib.mcp_servers.iter().find(|s| s.id == "filtered").unwrap();
    assert_eq!(filtered.codex_enabled_tools, vec!["read", "write"]);
    assert_eq!(filtered.codex_disabled_tools, vec!["delete"]);
}

#[test]
fn codex_agents_md_imported() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "AGENTS.md", "# Agent Instructions\n\nFollow TDD.");
    std::fs::create_dir_all(tmp.path().join(".codex")).unwrap();

    let lib = decompile_codex(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "AGENTS.md");
    assert!(lib.rules[0].content.contains("Follow TDD."));
}

#[test]
fn codex_unknown_keys_into_harness_defaults() {
    let tmp = TempDir::new().unwrap();
    let config = r#"
model = "o3"
custom_flag = true
"#;
    write_file(&tmp, ".codex/config.toml", config);

    let lib = decompile_codex(tmp.path());

    let defaults = lib.harness_defaults.get("codex").unwrap();
    assert_eq!(defaults["custom_flag"], true);
    assert!(defaults.get("model").is_none());
}

// ══════════════════════════════════════════════════════════════════════════════
// decompile_all — multi-harness merge
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn decompile_all_merges_harnesses() {
    let tmp = TempDir::new().unwrap();

    // Claude
    write_file(&tmp, "CLAUDE.md", "# Claude rules");
    write_file(
        &tmp,
        ".mcp.json",
        &serde_json::to_string(&json!({
            "mcpServers": {
                "ship": {"command": "ship", "args": ["mcp", "serve"]}
            }
        }))
        .unwrap(),
    );

    // Codex
    write_file(&tmp, "AGENTS.md", "# Codex rules");
    std::fs::create_dir_all(tmp.path().join(".codex")).unwrap();
    write_file(
        &tmp,
        ".codex/config.toml",
        "[mcp_servers.ship]\ncommand = \"ship\"\nargs = [\"mcp\", \"serve\"]\n\n\
         [mcp_servers.codex-only]\ncommand = \"codex-mcp\"\n",
    );

    let lib = decompile_all(tmp.path());

    // Rules from both harnesses
    assert_eq!(lib.rules.len(), 2);
    assert!(lib.rules.iter().any(|r| r.file_name == "CLAUDE.md"));
    assert!(lib.rules.iter().any(|r| r.file_name == "AGENTS.md"));

    // MCP servers — "ship" deduped, "codex-only" kept
    assert_eq!(lib.mcp_servers.len(), 2);
    assert!(lib.mcp_servers.iter().any(|s| s.id == "ship"));
    assert!(lib.mcp_servers.iter().any(|s| s.id == "codex-only"));
}

// ══════════════════════════════════════════════════════════════════════════════
// OpenCode
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn opencode_config_model_and_mcp() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "model": "gpt-4o",
        "mcpServers": {
            "ship": {
                "command": "ship",
                "args": ["mcp", "serve"]
            },
            "remote": {
                "url": "http://localhost:8080/mcp"
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("gpt-4o"));
    assert_eq!(lib.mcp_servers.len(), 2);
    let ship = lib.mcp_servers.iter().find(|s| s.id == "ship").unwrap();
    assert_eq!(ship.command, "ship");
}

#[test]
fn opencode_unknown_keys_into_harness_defaults() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "model": "gpt-4o",
        "plugins": [{"name": "my-plugin"}],
        "theme": "dark"
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    let defaults = lib.harness_defaults.get("opencode").unwrap();
    assert!(defaults.get("plugins").is_some());
    assert_eq!(defaults["theme"], "dark");
    assert!(defaults.get("model").is_none());
}

#[test]
fn opencode_agents_md_imported() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "AGENTS.md", "# Instructions\n\nUse OpenCode.");
    write_file(&tmp, "opencode.json", "{}");

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "AGENTS.md");
}
