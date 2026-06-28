//! Tests for the compile service — loader, parser, and pipeline.
//! Relocated from apps/ship-cli/src/loader_tests.rs and compile/tests*.rs.

use super::*;
use compiler::{HookTrigger, McpServerType};
use loader::parse_rule;
use skill_parser::parse_skill;
use std::path::Path;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// The project's own (on-disk) skills, excluding the embedded SDK capability-
/// skills the compile pool now always carries (`load_compile_skills`). These
/// loader tests assert disk-loading behavior, so they look past the embedded set.
fn disk_skills(skills: &[compiler::Skill]) -> Vec<&compiler::Skill> {
    skills
        .iter()
        .filter(|s| s.source != compiler::SkillSource::Builtin)
        .collect()
}

fn setup_minimal_project(tmp: &TempDir) {
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{ "mcp": { "servers": { "github": { "id": "github", "command": "npx", "args": ["-y", "@mcp/github"] } } } }"#,
    );
}

fn opts_for(tmp: &TempDir, harness: &str) -> CompileOptions {
    CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some(harness.to_string()),
        active_agent: None,
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    }
}

// ── load_library ─────────────────────────────────────────────────────────────

#[test]
fn empty_ship_dir_gives_default_library() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let lib = load_library(&ship_dir).unwrap();
    assert!(lib.mcp_servers.is_empty());
    assert!(disk_skills(&lib.skills).is_empty());
    assert!(lib.rules.is_empty());
}

#[test]
fn loads_stdio_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "mcp.jsonc",
        r#"{
  "mcp": {
    "servers": {
      "github": {
        "id": "github",
        "name": "GitHub",
        "command": "npx",
        "args": ["-y", "@mcp/github"]
      }
    }
  }
}"#,
    );
    let lib = load_library(&ship_dir).unwrap();
    assert_eq!(lib.mcp_servers.len(), 1);
    assert_eq!(lib.mcp_servers[0].id, "github");
    assert_eq!(lib.mcp_servers[0].command, "npx");
    assert!(matches!(lib.mcp_servers[0].server_type, McpServerType::Stdio));
}

#[test]
fn loads_http_mcp_server_by_url() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "mcp.jsonc",
        r#"{
  "mcp": {
    "servers": {
      "remote": {
        "id": "remote",
        "url": "https://api.example.com/mcp"
      }
    }
  }
}"#,
    );
    let lib = load_library(&ship_dir).unwrap();
    assert!(matches!(lib.mcp_servers[0].server_type, McpServerType::Http));
}

#[test]
fn loads_rules_sorted_alphabetically() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(&ship_dir, "rules/b-workflow.md", "Run tests first.");
    write(&ship_dir, "rules/a-style.md", "Use explicit types.");
    let lib = load_library(&ship_dir).unwrap();
    assert_eq!(lib.rules.len(), 2);
    assert_eq!(lib.rules[0].file_name, "a-style.md");
    assert_eq!(lib.rules[1].file_name, "b-workflow.md");
}

#[test]
fn rule_frontmatter_parsed() {
    let raw = "---\ndescription: \"Style guide\"\nalwaysApply: false\n---\n\nKeep it clean.";
    let rule = parse_rule("style.md", raw);
    assert_eq!(rule.description.as_deref(), Some("Style guide"));
    assert!(!rule.always_apply);
    assert_eq!(rule.content, "Keep it clean.");
}

#[test]
fn rule_without_frontmatter_uses_full_content() {
    let rule = parse_rule("plain.md", "Just plain content.");
    assert_eq!(rule.content, "Just plain content.");
    assert!(rule.always_apply);
    assert!(rule.description.is_none());
}

#[test]
fn loads_skill_from_skill_md() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        ".ship/skills/my-skill/SKILL.md",
        "---\nname: My Skill\ndescription: Does stuff\n---\n\nInstructions here.",
    );
    let lib = load_library(&ship_dir).unwrap();
    let skills = disk_skills(&lib.skills);
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].id, "my-skill");
    assert_eq!(skills[0].name, "My Skill");
    assert_eq!(skills[0].description.as_deref(), Some("Does stuff"));
    assert_eq!(skills[0].content, "Instructions here.");
}

#[test]
fn loads_skill_from_flat_md() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        ".ship/skills/ship-coordination.md",
        "---\nname: Ship Coordination\ndescription: Coordination skill\n---\n\nContent here.",
    );
    let lib = load_library(&ship_dir).unwrap();
    let skills = disk_skills(&lib.skills);
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].id, "ship-coordination");
    assert_eq!(skills[0].name, "Ship Coordination");
}

#[test]
fn loads_skills_mixed_formats() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(&ship_dir, ".ship/skills/flat-skill.md", "---\nname: Flat\n---\n\nFlat content.");
    write(&ship_dir, ".ship/skills/dir-skill/SKILL.md", "---\nname: Dir\n---\n\nDir content.");
    let lib = load_library(&ship_dir).unwrap();
    assert_eq!(disk_skills(&lib.skills).len(), 2);
}

#[test]
fn loads_permissions() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "permissions.jsonc",
        r#"{ "tools": { "deny": ["Bash(rm -rf *)"] } }"#,
    );
    let lib = load_library(&ship_dir).unwrap();
    assert!(lib.permissions.tools.deny.contains(&"Bash(rm -rf *)".to_string()));
}

#[test]
fn loads_hooks() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "hooks.jsonc",
        r#"{
  "hooks": [
    { "id": "check", "trigger": "pre_tool_use", "command": "ship hooks run", "matcher": "Bash" }
  ]
}"#,
    );
    let lib = load_library(&ship_dir).unwrap();
    assert_eq!(lib.hooks.len(), 1);
    assert_eq!(lib.hooks[0].command, "ship hooks run");
    assert!(matches!(lib.hooks[0].trigger, HookTrigger::PreToolUse));
}

#[test]
fn unknown_hook_trigger_skipped() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "hooks.jsonc",
        r#"{ "hooks": [{ "id": "bad", "trigger": "unknown_event", "command": "echo hi" }] }"#,
    );
    let lib = load_library(&ship_dir).unwrap();
    assert!(lib.hooks.is_empty(), "unknown trigger must be silently dropped");
}

#[test]
fn load_permission_preset_reads_named_section() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "permissions.jsonc",
        r#"{
  "ship-standard": {
    "default_mode": "acceptEdits",
    "tools_ask": ["Bash(rm -rf*)"],
    "tools_deny": ["Bash(git push --force*)"]
  }
}"#,
    );
    let preset = load_permission_preset(&ship_dir, "ship-standard").unwrap();
    assert_eq!(preset.default_mode.as_deref(), Some("acceptEdits"));
    assert!(preset.tools_ask.contains(&"Bash(rm -rf*)".to_string()));
    assert!(preset.tools_deny.contains(&"Bash(git push --force*)".to_string()));
}

#[test]
fn load_permission_preset_missing_section_returns_none() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(&ship_dir, "permissions.jsonc", r#"{ "ship-elevated": { "default_mode": "dontAsk" } }"#);
    assert!(load_permission_preset(&ship_dir, "nonexistent").is_none());
}

#[test]
fn load_permission_preset_missing_file_returns_none() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    assert!(load_permission_preset(&ship_dir, "ship-standard").is_none());
}

#[test]
fn loads_agent_profiles_from_agents_dir() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "agents/web-lane.jsonc",
        r#"{
  "agent": { "id": "web-lane", "name": "Web Lane", "harnesses": ["claude"] },
  "skills": { "refs": ["tanstack-start"] },
  "permissions": { "preset": "ship-standard" }
}"#,
    );
    write(
        &ship_dir,
        "agents/server-lane.jsonc",
        r#"{ "agent": { "id": "server-lane", "name": "Server Lane", "harnesses": ["claude"] } }"#,
    );
    let lib = load_library(&ship_dir).unwrap();
    assert_eq!(lib.agent_profiles.len(), 2);
    assert_eq!(lib.agent_profiles[0].agent.id, "server-lane");
    assert_eq!(lib.agent_profiles[1].agent.id, "web-lane");
    assert_eq!(lib.agent_profiles[1].skills.refs, vec!["tanstack-start"]);
}

#[test]
fn agent_profiles_empty_when_dir_missing() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let lib = load_library(&ship_dir).unwrap();
    assert!(lib.agent_profiles.is_empty());
}

#[test]
fn agent_profiles_skips_invalid_jsonc() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(&ship_dir, "agents/good.jsonc", r#"{ "agent": { "id": "good", "name": "Good" } }"#);
    write(&ship_dir, "agents/bad.jsonc", "this is not valid jsonc { { {");
    let lib = load_library(&ship_dir).unwrap();
    assert_eq!(lib.agent_profiles.len(), 1);
    assert_eq!(lib.agent_profiles[0].agent.id, "good");
}

#[test]
fn agent_profiles_ignores_legacy_subdir() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write(
        &ship_dir,
        "agents/profiles/legacy.jsonc",
        r#"{ "agent": { "id": "legacy", "name": "Legacy" } }"#,
    );
    let lib = load_library(&ship_dir).unwrap();
    assert!(lib.agent_profiles.is_empty());
}

// ── parse_skill ───────────────────────────────────────────────────────────────

#[test]
fn skill_parses_license_and_compatibility() {
    let raw = "---\nname: my-skill\ndescription: Does stuff\nlicense: MIT\ncompatibility: claude >= 3\n---\n\nInstructions.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.license.as_deref(), Some("MIT"));
    assert_eq!(skill.compatibility.as_deref(), Some("claude >= 3"));
}

#[test]
fn skill_parses_allowed_tools_hyphenated_key() {
    let raw = "---\nname: my-skill\ndescription: x\nallowed-tools: Read Edit Grep\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.allowed_tools, vec!["Read", "Edit", "Grep"]);
}

#[test]
fn skill_parses_metadata_block() {
    let raw = "---\nname: my-skill\ndescription: x\nmetadata:\n  team: infra\n  priority: high\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.metadata.get("team").map(String::as_str), Some("infra"));
    assert_eq!(skill.metadata.get("priority").map(String::as_str), Some("high"));
}

#[test]
fn skill_folds_legacy_version_author_into_metadata() {
    let raw = "---\nname: my-skill\ndescription: x\nversion: 1.2.3\nauthor: alice\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.metadata.get("version").map(String::as_str), Some("1.2.3"));
    assert_eq!(skill.metadata.get("author").map(String::as_str), Some("alice"));
}

#[test]
fn skill_parses_stable_id() {
    let raw = "---\nname: my-skill\nstable-id: commit\ndescription: x\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.stable_id.as_deref(), Some("commit"));
}

#[test]
fn skill_stable_id_invalid_is_ignored() {
    let raw = "---\nname: my-skill\nstable-id: ../evil\ndescription: x\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert!(skill.stable_id.is_none());
}

#[test]
fn skill_parses_grants_block() {
    let raw = "---\nname: ship-orchestration\ngrants: ship-orchestration\n---\n\nContent.";
    let skill = parse_skill("ship-orchestration", raw);
    assert_eq!(skill.grants, vec!["ship-orchestration"]);
}

#[test]
fn skill_grants_empty_when_absent() {
    let raw = "---\nname: my-skill\ndescription: x\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert!(skill.grants.is_empty());
}

#[test]
fn skill_parses_requires_block() {
    let raw = "---\nname: my-workflow\nrequires: ship-orchestration\n---\n\nContent.";
    let skill = parse_skill("my-workflow", raw);
    assert_eq!(skill.requires, vec!["ship-orchestration"]);
    assert!(skill.grants.is_empty());
}

// ── compile pipeline ──────────────────────────────────────────────────────────

#[test]
fn compile_writes_claude_md() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    compile_project(&opts_for(&tmp, "claude")).unwrap();
    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_writes_mcp_json() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    compile_project(&opts_for(&tmp, "claude")).unwrap();
    let content = std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed["mcpServers"]["ship"].is_null(), "ship server must not be in .mcp.json after rip");
    assert!(parsed["mcpServers"]["github"].is_object());
}

#[test]
fn mcp_json_does_not_include_ship_server() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    compile_project(&opts_for(&tmp, "claude")).unwrap();
    let content = std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let ship = &parsed["mcpServers"]["ship"];
    assert!(ship.is_null(), "ship server entry must not exist after rip");
}

#[test]
fn compile_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: None,
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: true,
    })
    .unwrap();
    assert!(!tmp.path().join("CLAUDE.md").exists(), "dry-run must not write files");
    assert!(!tmp.path().join(".mcp.json").exists());
}

#[test]
fn dry_run_outputs_present_in_summary() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    let summary = compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: None,
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: true,
    })
    .unwrap();
    assert!(!summary.outputs.is_empty(), "dry-run must populate outputs for printing");
}

#[test]
fn compile_codex_writes_agents_md_with_rules() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    compile_project(&opts_for(&tmp, "codex")).unwrap();
    let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_cursor_writes_mdc_rule_files() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    compile_project(&opts_for(&tmp, "cursor")).unwrap();
    let mdc = tmp.path().join(".cursor/rules/style.mdc");
    assert!(mdc.exists(), ".cursor/rules/style.mdc must be written");
    let content = std::fs::read_to_string(&mdc).unwrap();
    assert!(content.contains("Use explicit types."));
    assert!(content.starts_with("---\n"), "must have frontmatter");
}

#[test]
fn compile_with_deny_writes_claude_settings() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/permissions.jsonc", r#"{ "tools": { "deny": ["Bash(rm -rf *)"] } }"#);
    compile_project(&opts_for(&tmp, "claude")).unwrap();
    let settings_path = tmp.path().join(".claude/settings.json");
    assert!(settings_path.exists());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(v["permissions"]["deny"][0], "Bash(rm -rf *)");
}

#[test]
fn compile_with_mode_applies_permissions() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/readonly.jsonc",
        r#"{
  "agent": { "id": "readonly", "name": "ReadOnly", "harnesses": ["claude"] },
  "permissions": { "preset": "ship-readonly" }
}"#,
    );
    compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: Some("readonly".to_string()),
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let deny = v["permissions"]["deny"].as_array().unwrap();
    assert!(deny.iter().any(|d| d == "Write(*)"));
}

#[test]
fn compile_with_mode_inline_rules_adds_to_context() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/strict.jsonc",
        r#"{
  "agent": { "id": "strict", "name": "Strict", "harnesses": ["claude"] },
  "rules": { "inline": "Never delete files without explicit confirmation." }
}"#,
    );
    compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: Some("strict".to_string()),
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    })
    .unwrap();
    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Never delete files without explicit confirmation."));
}

#[test]
fn compile_with_mode_uses_permissions_jsonc_preset() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{
  "ship-fast": {
    "default_mode": "bypassPermissions",
    "tools_deny": ["Bash(git push --force*)"]
  }
}"#,
    );
    write(
        tmp.path(),
        ".ship/agents/fast.jsonc",
        r#"{ "agent": { "id": "fast", "name": "Fast", "harnesses": ["claude"] }, "permissions": { "preset": "ship-fast" } }"#,
    );
    compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: Some("fast".to_string()),
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["permissions"]["defaultMode"], "bypassPermissions");
    let deny = v["permissions"]["deny"].as_array().unwrap();
    assert!(deny.iter().any(|d| d == "Bash(git push --force*)"));
}

// ── extra skills (--with) ─────────────────────────────────────────────────────

#[test]
fn extra_skill_included_in_output() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/rust-lane.jsonc",
        r#"{ "agent": { "id": "rust-lane", "name": "Rust Lane" }, "skills": { "refs": ["base-skill"] } }"#,
    );
    write(tmp.path(), ".ship/skills/base-skill/SKILL.md", "---\nname: base-skill\n---\nBase skill content.");
    write(tmp.path(), ".ship/skills/compiler-docs/SKILL.md", "---\nname: compiler-docs\n---\nCompiler docs content.");
    compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: Some("rust-lane".to_string()),
        extra_skills: vec!["compiler-docs".to_string()],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    })
    .unwrap();
    let injected =
        std::fs::read_to_string(tmp.path().join(".claude/skills/compiler-docs/SKILL.md"))
            .expect("injected skill file must exist");
    assert!(injected.contains("Compiler docs content."), "injected skill must appear: {injected}");
    let base = std::fs::read_to_string(tmp.path().join(".claude/skills/base-skill/SKILL.md"))
        .expect("base skill must exist");
    assert!(base.contains("Base skill content."), "base skill must still appear: {base}");
}

#[test]
fn nonexistent_extra_skill_returns_clear_error() {
    let tmp = TempDir::new().unwrap();
    let err = compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: None,
        extra_skills: vec!["nonexistent".to_string()],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("skill not found: nonexistent"),
        "error message must identify the missing skill: {err}"
    );
}

#[test]
fn no_extra_skills_behaviour_unchanged() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/rust-lane.jsonc",
        r#"{ "agent": { "id": "rust-lane", "name": "Rust Lane" }, "skills": { "refs": ["base-skill"] } }"#,
    );
    write(tmp.path(), ".ship/skills/base-skill/SKILL.md", "---\nname: base-skill\n---\nBase skill content.");
    write(tmp.path(), ".ship/skills/other/SKILL.md", "---\nname: other\n---\nOther content.");
    compile_project(&CompileOptions {
        project_root: tmp.path().to_path_buf(),
        output_root: None,
        harness: Some("claude".to_string()),
        active_agent: Some("rust-lane".to_string()),
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    })
    .unwrap();
    let base = std::fs::read_to_string(tmp.path().join(".claude/skills/base-skill/SKILL.md"))
        .expect("base skill must exist");
    assert!(base.contains("Base skill content."), "base skill must appear: {base}");
    assert!(
        !tmp.path().join(".claude/skills/other/SKILL.md").exists(),
        "non-referenced skill must not be written when refs filter is active"
    );
}

// ── output_root separation ────────────────────────────────────────────────────

#[test]
fn compile_output_root_separates_from_project_root() {
    let project = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    setup_minimal_project(&project);
    compile_project(&CompileOptions {
        project_root: project.path().to_path_buf(),
        output_root: Some(output.path().to_path_buf()),
        harness: Some("claude".to_string()),
        active_agent: None,
        extra_skills: vec![],
        plugins_installed: vec![],
        scoped_launch: false,
        dry_run: false,
    })
    .unwrap();
    assert!(output.path().join(".mcp.json").exists(), ".mcp.json must be in output_root");
    assert!(!project.path().join(".mcp.json").exists(), ".mcp.json must NOT be in project_root");
    assert!(output.path().join("CLAUDE.md").exists(), "CLAUDE.md must be in output_root");
    assert!(!project.path().join("CLAUDE.md").exists(), "CLAUDE.md must NOT be in project_root");
}

// ── McpFile round-trip ────────────────────────────────────────────────────────

#[test]
fn mcp_file_round_trips() {
    use std::collections::HashMap;
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("mcp.jsonc");
    let original = McpFile {
        servers: vec![McpEntry {
            id: "linear".into(),
            name: Some("Linear".into()),
            command: Some("npx".into()),
            args: vec!["-y".into(), "@mcp/linear".into()],
            env: HashMap::new(),
            url: None,
            scope: "project".into(),
            server_type: Some("stdio".into()),
            disabled: false,
        }],
    };
    original.save(&path).unwrap();
    let back = McpFile::load(&path).unwrap();
    assert_eq!(back.servers[0].name.as_deref(), Some("Linear"));
}

#[test]
fn mcp_file_load_empty_when_no_file() {
    let tmp = TempDir::new().unwrap();
    let file = McpFile::load(&tmp.path().join("nonexistent.jsonc")).unwrap();
    assert!(file.servers.is_empty());
}
