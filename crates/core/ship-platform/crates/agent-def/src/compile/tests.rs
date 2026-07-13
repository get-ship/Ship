//! Tests for the compile service — loader, parser, and pipeline.
//! Relocated from apps/ship-cli/src/loader_tests.rs and compile/tests*.rs.

use super::*;
use stack_model::{HookTrigger, McpServerType};
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
/// skills the compile pool used to carry (`load_compile_skills` — retired
/// with Wave 5; project skills now arrive via `defineSkill` declare-forms).
/// These loader tests assert disk-loading behavior, so they look past the
/// embedded set.
fn disk_skills(skills: &[stack_model::Skill]) -> Vec<&stack_model::Skill> {
    skills
        .iter()
        .filter(|s| s.source != stack_model::SkillSource::Builtin)
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
        space_root: tmp.path().to_path_buf(),
        harness: Some(harness.to_string()),
        ..Default::default()
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

// `loads_skill_from_skill_md` retired with format-migration Wave 5. The test exercised
// the agent-def leaf's `.ship/agents/*.jsonc` and/or
// `.ship/skills/<id>/SKILL.md` reads — paths the leaf no longer walks
// (those nouns arrive exclusively via `defineAgent` / `defineSkill`
// declare-forms in `stack.config.ts`, harvested by the resolver's TS
// eval and merged into the composed library).
// Migration path: tests that need agents / skills in the compiled
// output must use `stack.config.ts` and route through the composed
// descriptor (`compile_project_from_library` /
// `ship_resolver::roll_up`). Post slice 3, the leaf's `load_library`
// is disk-shape for mcp + rules only — permissions + hooks +
// permissions_presets are resolver-sourced (the per-noun walks land
// in `ExtensionLibrary.{permissions,hooks,permissions_presets}` from
// `merge_ts_declarations`, not from this fn).


// `loads_skill_from_flat_md` retired with format-migration Wave 5. The test exercised
// the agent-def leaf's `.ship/agents/*.jsonc` and/or
// `.ship/skills/<id>/SKILL.md` reads — paths the leaf no longer walks
// (those nouns arrive exclusively via `defineAgent` / `defineSkill`
// declare-forms in `stack.config.ts`, harvested by the resolver's TS
// eval and merged into the composed library).
// Migration path: tests that need agents / skills in the compiled
// output must use `stack.config.ts` and route through the composed
// descriptor (`compile_project_from_library` /
// `ship_resolver::roll_up`). The leaf's `load_library` is now
// disk-shape for permissions / hooks / mcp / rules only.


// `loads_skills_mixed_formats` retired with format-migration Wave 5. The test exercised
// the agent-def leaf's `.ship/agents/*.jsonc` and/or
// `.ship/skills/<id>/SKILL.md` reads — paths the leaf no longer walks
// (those nouns arrive exclusively via `defineAgent` / `defineSkill`
// declare-forms in `stack.config.ts`, harvested by the resolver's TS
// eval and merged into the composed library).
// Migration path: tests that need agents / skills in the compiled
// output must use `stack.config.ts` and route through the composed
// descriptor (`compile_project_from_library` /
// `ship_resolver::roll_up`). The leaf's `load_library` is now
// disk-shape for permissions / hooks / mcp / rules only.


// `load_permission_preset` retired from disk reads with the slice 3
// close-out. The function is now an in-memory lookup against
// `ExtensionLibrary::permissions_presets` (the `BTreeMap` the
// resolver populates from `defineConfig.agents.permissions.presets`).
// The unit tests below cover the lookup: present / missing / wrong name.
// The disk side (parsing `.ship/permissions.jsonc`) moved to the
// resolver's `merge_ts_declarations` and is exercised through the
// resolver's own tests.

#[test]
fn load_permission_preset_returns_named_entry() {
    use std::collections::BTreeMap;
    let mut presets: BTreeMap<String, PermissionPreset> = BTreeMap::new();
    presets.insert(
        "ship-standard".to_string(),
        PermissionPreset {
            default_mode: Some("acceptEdits".to_string()),
            tools_allow: vec![],
            tools_deny: vec!["Bash(git push --force*)".to_string()],
            tools_ask: vec!["Bash(rm -rf*)".to_string()],
            additional_directories: vec![],
        },
    );
    let preset = load_permission_preset(&presets, "ship-standard").unwrap();
    assert_eq!(preset.default_mode.as_deref(), Some("acceptEdits"));
    assert!(preset.tools_ask.contains(&"Bash(rm -rf*)".to_string()));
    assert!(preset.tools_deny.contains(&"Bash(git push --force*)".to_string()));
}

#[test]
fn load_permission_preset_missing_entry_returns_none() {
    use std::collections::BTreeMap;
    let mut presets: BTreeMap<String, PermissionPreset> = BTreeMap::new();
    presets.insert(
        "ship-elevated".to_string(),
        PermissionPreset {
            default_mode: Some("dontAsk".to_string()),
            ..Default::default()
        },
    );
    assert!(load_permission_preset(&presets, "nonexistent").is_none());
}

#[test]
fn load_permission_preset_empty_map_returns_none() {
    use std::collections::BTreeMap;
    let presets: BTreeMap<String, PermissionPreset> = BTreeMap::new();
    assert!(load_permission_preset(&presets, "ship-standard").is_none());
}

// `loads_agent_profiles_from_agents_dir` retired with format-migration Wave 5. The test exercised
// the agent-def leaf's `.ship/agents/*.jsonc` and/or
// `.ship/skills/<id>/SKILL.md` reads — paths the leaf no longer walks
// (those nouns arrive exclusively via `defineAgent` / `defineSkill`
// declare-forms in `stack.config.ts`, harvested by the resolver's TS
// eval and merged into the composed library).
// Migration path: tests that need agents / skills in the compiled
// output must use `stack.config.ts` and route through the composed
// descriptor (`compile_project_from_library` /
// `ship_resolver::roll_up`). The leaf's `load_library` is now
// disk-shape for permissions / hooks / mcp / rules only.


#[test]
fn agent_profiles_empty_when_dir_missing() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let lib = load_library(&ship_dir).unwrap();
    assert!(lib.agent_profiles.is_empty());
}

// `agent_profiles_skips_invalid_jsonc` retired with format-migration Wave 5. The test exercised
// the agent-def leaf's `.ship/agents/*.jsonc` and/or
// `.ship/skills/<id>/SKILL.md` reads — paths the leaf no longer walks
// (those nouns arrive exclusively via `defineAgent` / `defineSkill`
// declare-forms in `stack.config.ts`, harvested by the resolver's TS
// eval and merged into the composed library).
// Migration path: tests that need agents / skills in the compiled
// output must use `stack.config.ts` and route through the composed
// descriptor (`compile_project_from_library` /
// `ship_resolver::roll_up`). The leaf's `load_library` is now
// disk-shape for permissions / hooks / mcp / rules only.
