//! End-to-end tests for the `ship` CLI binary.
//!
//! Each test runs the actual binary against an isolated temp directory.
//! The `ship init` command was cut (SpaceService territory post-pivot), so
//! tests that need a `.ship/` layout scaffold it manually via
//! `scaffold_ship_dir`. Surviving commands under test: `ship use`,
//! `ship compile`, `ship --help`.
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Bare CLI command — no DB isolation. Use `ship_in()` for tests that touch
/// the database (ship use writes the active_agent projection).
fn ship() -> Command {
    Command::cargo_bin("ship").unwrap()
}

/// CLI command with DB isolated to `tmp`'s temp dir.
/// All calls within the same test share state; different tests are isolated.
fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship").unwrap();
    cmd.env("SHIP_HOME", tmp.path());
    cmd
}

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// Manually scaffold the bare-minimum `.ship/` tree that `ship use` and
/// `ship compile` need. Mirrors what the (now-removed) `ship init` produced
/// for a fresh project. Adds a `dev` agent at `.ship/agents/dev.jsonc` so
/// `ship use dev` resolves.
fn scaffold_ship_dir(root: &Path) {
    write(
        root,
        ".ship/ship.jsonc",
        r#"{
  "$schema": "../schemas/ship.schema.json",
  "project": {
    "harnesses": ["claude"],
  },
}
"#,
    );
    write(
        root,
        ".ship/agents/dev.jsonc",
        r#"{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json",
  "agent": {
    "id": "dev",
    "name": "Dev",
    "version": "0.1.0",
    "description": "General-purpose development agent"
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": { "preset": "ship-standard" }
}
"#,
    );
}

// ── ship use ──────────────────────────────────────────────────────────────────

#[test]
fn use_activates_agent_writes_claude_md_and_state() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    scaffold_ship_dir(tmp.path());
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/my-agent.jsonc",
        r#"{ "agent": { "name": "My Agent", "id": "my-agent", "harnesses": ["claude"] } }"#,
    );

    ship_in(&tmp)
        .args(["use", "my-agent", "--path", p])
        .assert()
        .success()
        .stdout(predicate::str::contains("activated agent 'my-agent'"));

    assert!(
        tmp.path().join("CLAUDE.md").exists(),
        "CLAUDE.md must be written"
    );
}

#[test]
fn use_same_agent_twice_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    scaffold_ship_dir(tmp.path());
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/my-agent.jsonc",
        r#"{ "agent": { "name": "My Agent", "id": "my-agent", "harnesses": ["claude"] } }"#,
    );

    let args = ["use", "my-agent", "--path", p];
    ship_in(&tmp).args(args).assert().success();
    ship_in(&tmp).args(args).assert().success();
}

#[test]
fn use_fails_without_ship_dir() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp)
        .args(["use", "my-profile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".ship/ not found"));
}

#[test]
fn use_fails_for_unknown_agent() {
    let tmp = TempDir::new().unwrap();
    scaffold_ship_dir(tmp.path());

    ship()
        .args(["use", "nonexistent", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

// ── ship compile ──────────────────────────────────────────────────────────────

#[test]
fn compile_writes_claude_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args([
            "compile",
            "--harness",
            "claude",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args([
            "compile",
            "--harness",
            "claude",
            "--dry-run",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    assert!(
        !tmp.path().join("CLAUDE.md").exists(),
        "dry-run must not write any files"
    );
}

#[test]
fn compile_fails_without_ship_dir() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["compile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".ship/ not found"));
}

// ── ship compile --harness ───────────────────────────────────────────────────

#[test]
fn compile_harness_codex_writes_agents_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--harness", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(
        tmp.path().join("AGENTS.md").exists(),
        "AGENTS.md must be written for codex compile"
    );
    let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_harness_codex_writes_codex_config_toml() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--harness", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let path = tmp.path().join(".codex/config.toml");
    assert!(
        path.exists(),
        ".codex/config.toml must be written for codex compile"
    );
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("mcp_servers"),
        "config.toml must contain mcp_servers"
    );
}

// ── ship compile idempotency ───────────────────────────────────────────────────

#[test]
fn compile_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    let args = [
        "compile",
        "--harness",
        "claude",
        "--path",
        tmp.path().to_str().unwrap(),
    ];

    ship().args(args).assert().success();
    let first = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();

    ship().args(args).assert().success();
    let second = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();

    assert_eq!(
        first, second,
        "compile output must be identical on second run"
    );
}

// ── E2E integration tests ────────────────────────────────────────────────────

/// Full user story: scaffold .ship/ -> write agent JSONC with permissions/skills/hooks ->
/// write rules -> write MCP config -> write permissions preset -> use agent ->
/// compile for claude -> verify all output files.
#[test]
fn e2e_scaffold_create_agent_use_compile_full_flow() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    scaffold_ship_dir(tmp.path());

    // Write a realistic agent JSONC with permissions preset, skills ref, hooks
    write(
        tmp.path(),
        ".ship/agents/fullstack.jsonc",
        r#"{
  "agent": {
    "id": "fullstack",
    "name": "Fullstack Dev",
    "version": "0.1.0",
    "description": "Full-stack development agent",
    "harnesses": ["claude"]
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(docker rm*)"]
  },
  "rules": {
    "inline": "Always write tests before implementation."
  },
  "hooks": {
    "stop": "ship permissions sync"
  }
}"#,
    );

    // Write a rule file
    write(
        tmp.path(),
        ".ship/rules/style.md",
        "Prefer composition over inheritance. Use explicit return types.",
    );

    // Write MCP config with a server
    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{
  "mcp": {
    "servers": {
      "github": {
        "id": "",
        "name": "GitHub MCP",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-github"],
        "scope": "project",
        "server_type": "stdio",
        "disabled": false,
        "env": {}
      }
    }
  }
}"#,
    );

    // Write permissions.jsonc with ship-autonomous preset
    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{
  "ship-autonomous": {
    "default_mode": "dontAsk",
    "tools_allow": ["Read", "Write", "Edit", "Glob", "Grep", "Bash(*)"],
    "tools_deny": [
      "Bash(rm -rf *)",
      "Bash(git reset --hard*)",
      "Bash(git push*)"
    ]
  }
}"#,
    );

    // ship use <agent> (this compiles with the agent active)
    ship_in(&tmp)
        .args(["use", "fullstack", "--path", p])
        .assert()
        .success()
        .stdout(predicate::str::contains("activated agent 'fullstack'"));

    // Verify CLAUDE.md contains rule content and inline rule (written by ship use)
    let claude_md = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(
        claude_md.contains("Prefer composition over inheritance"),
        "CLAUDE.md must contain rule file content"
    );
    assert!(
        claude_md.contains("Always write tests before implementation"),
        "CLAUDE.md must contain inline rule from agent"
    );

    // Re-compile picks up active agent from DB state
    ship_in(&tmp)
        .args(["compile", "--harness", "claude", "--path", p])
        .assert()
        .success();

    // Verify .claude/settings.json contains permissions
    let settings_path = tmp.path().join(".claude/settings.json");
    assert!(settings_path.exists(), ".claude/settings.json must exist");
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(
        settings["permissions"]["defaultMode"], "dontAsk",
        "defaultMode must come from ship-autonomous preset"
    );
    let deny = settings["permissions"]["deny"].as_array().unwrap();
    assert!(
        deny.iter().any(|d| d == "Bash(rm -rf *)"),
        "deny list must contain preset entries"
    );

    // Verify .mcp.json contains the github server
    let mcp_json_path = tmp.path().join(".mcp.json");
    assert!(mcp_json_path.exists(), ".mcp.json must exist");
    let mcp: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&mcp_json_path).unwrap()).unwrap();
    assert!(
        mcp["mcpServers"]["github"].is_object(),
        "github server must appear in .mcp.json"
    );
}

/// Same .ship/ configuration compiles correctly to all 4 harnesses.
/// Verifies that rule content appears in every harness's context file
/// and that each harness produces its expected artifacts.
#[test]
fn e2e_cross_harness_same_config() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    scaffold_ship_dir(tmp.path());

    // Set up a realistic .ship/ with agent, rules, permissions, MCP
    write(
        tmp.path(),
        ".ship/agents/cross.jsonc",
        r#"{
  "agent": {
    "id": "cross",
    "name": "Cross Harness",
    "version": "0.1.0",
    "harnesses": ["claude", "cursor", "codex"]
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {
    "preset": "ship-standard"
  },
  "rules": {}
}"#,
    );

    write(
        tmp.path(),
        ".ship/rules/architecture.md",
        "Keep modules under 300 lines. Prefer pure functions.",
    );

    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{ "ship-standard": { "default_mode": "default" } }"#,
    );

    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{
  "mcp": {
    "servers": {
      "test-server": {
        "id": "",
        "command": "echo",
        "args": ["hello"],
        "scope": "project",
        "server_type": "stdio",
        "disabled": false,
        "env": {}
      }
    }
  }
}"#,
    );

    let rule_text = "Keep modules under 300 lines";

    // Compile for claude
    ship()
        .args(["compile", "--harness", "claude", "--path", p])
        .assert()
        .success();
    assert!(
        tmp.path().join("CLAUDE.md").exists(),
        "CLAUDE.md must be written"
    );
    assert!(
        fs::read_to_string(tmp.path().join("CLAUDE.md"))
            .unwrap()
            .contains(rule_text),
        "CLAUDE.md must contain rule content"
    );
    assert!(
        tmp.path().join(".mcp.json").exists(),
        ".mcp.json must be written for claude"
    );

    // Compile for cursor
    ship()
        .args(["compile", "--harness", "cursor", "--path", p])
        .assert()
        .success();
    let cursor_rule = tmp.path().join(".cursor/rules/architecture.mdc");
    assert!(
        cursor_rule.exists(),
        ".cursor/rules/architecture.mdc must be written"
    );
    assert!(
        fs::read_to_string(&cursor_rule)
            .unwrap()
            .contains(rule_text),
        "cursor rule file must contain rule content"
    );

    // Compile for codex
    ship()
        .args(["compile", "--harness", "codex", "--path", p])
        .assert()
        .success();
    assert!(
        tmp.path().join("AGENTS.md").exists(),
        "AGENTS.md must be written"
    );
    assert!(
        fs::read_to_string(tmp.path().join("AGENTS.md"))
            .unwrap()
            .contains(rule_text),
        "AGENTS.md must contain rule content"
    );
    assert!(
        tmp.path().join(".codex/config.toml").exists(),
        ".codex/config.toml must be written"
    );
}

/// Validate that permission preset + agent-level overrides compose correctly.
/// The preset defines a deny list, the agent adds additional entries on top.
/// Both must appear in the compiled settings.json.
#[test]
fn e2e_permission_preset_plus_agent_override() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    scaffold_ship_dir(tmp.path());

    // permissions.jsonc defines ship-autonomous with specific deny list
    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{
  "ship-autonomous": {
    "default_mode": "dontAsk",
    "tools_allow": ["Read", "Write", "Edit", "Bash(*)"],
    "tools_deny": [
      "Bash(rm -rf *)",
      "Bash(git reset --hard*)",
      "Bash(git push*)"
    ]
  }
}"#,
    );

    // Agent uses preset AND adds its own tools_deny entries
    write(
        tmp.path(),
        ".ship/agents/strict-auto.jsonc",
        r#"{
  "agent": {
    "id": "strict-auto",
    "name": "Strict Autonomous",
    "version": "0.1.0",
    "harnesses": ["claude"]
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(docker rm*)", "Bash(cargo publish*)"]
  },
  "rules": {}
}"#,
    );

    // ship use compiles with the agent active, applying preset + overrides
    ship_in(&tmp)
        .args(["use", "strict-auto", "--path", p])
        .assert()
        .success();

    let settings_path = tmp.path().join(".claude/settings.json");
    assert!(settings_path.exists(), ".claude/settings.json must exist");
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();

    // Assert defaultMode matches the preset
    assert_eq!(
        settings["permissions"]["defaultMode"], "dontAsk",
        "defaultMode must come from ship-autonomous preset"
    );

    let deny = settings["permissions"]["deny"]
        .as_array()
        .expect("deny must be an array");

    // Assert preset deny entries are present
    assert!(
        deny.iter().any(|d| d == "Bash(rm -rf *)"),
        "preset deny entry 'Bash(rm -rf *)' must be present, got: {:?}",
        deny
    );
    assert!(
        deny.iter().any(|d| d == "Bash(git push*)"),
        "preset deny entry 'Bash(git push*)' must be present, got: {:?}",
        deny
    );

    // Assert agent-level deny additions are ALSO present
    assert!(
        deny.iter().any(|d| d == "Bash(docker rm*)"),
        "agent-level deny entry 'Bash(docker rm*)' must be present, got: {:?}",
        deny
    );
    assert!(
        deny.iter().any(|d| d == "Bash(cargo publish*)"),
        "agent-level deny entry 'Bash(cargo publish*)' must be present, got: {:?}",
        deny
    );
}

/// Test that installed dependency skills from the package cache appear in compiled output.
/// Simulates the cache directory structure that ship install would create.
#[test]
fn e2e_compile_with_dependency_skills() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    let hex = "a1b2c3d4e5f6";
    scaffold_ship_dir(tmp.path());

    // ship.jsonc referencing a dependency
    write(
        tmp.path(),
        ".ship/ship.jsonc",
        r#"{
  "$schema": "../schemas/ship.schema.json",
  "project": {
    "harnesses": ["claude"],
  },
}
"#,
    );

    // ship.lock with a matching entry
    write(
        tmp.path(),
        ".ship/ship.lock",
        &format!(
            r#"{{
  "version": 1,
  "packages": [
    {{
      "path": "github.com/example/skills-pkg",
      "version": "main",
      "commit": "{}",
      "hash": "sha256:{}"
    }}
  ]
}}"#,
            "a".repeat(40),
            hex,
        ),
    );

    // Agent referencing the dep skill
    write(
        tmp.path(),
        ".ship/agents/with-deps.jsonc",
        r#"{
  "agent": {
    "id": "with-deps",
    "name": "With Deps",
    "version": "0.1.0",
    "harnesses": ["claude"]
  },
  "skills": {
    "refs": ["github.com/example/skills-pkg/skills/cool-skill"]
  },
  "mcp": { "servers": [] },
  "permissions": {},
  "rules": {}
}"#,
    );

    // Simulate installed package cache at $HOME/.ship/cache/objects/<hex>/
    // The dep_skills resolver uses home::home_dir() for the cache path.
    // Set HOME to tmp so the cache is found at tmp/.ship/cache/objects/<hex>/
    write(
        tmp.path(),
        &format!(".ship/cache/objects/{}/skills/cool-skill/SKILL.md", hex),
        "---\nname: Cool Skill\ndescription: A cool dependency skill\n---\n\nYou are a cool skill. Follow best practices.",
    );

    // Use ship_in for DB isolation and set HOME for cache resolution.
    // Both SHIP_HOME (for the per-scope event dbs) and HOME (for cache) point to tmp.
    ship_in(&tmp)
        .env("HOME", tmp.path())
        .args(["use", "with-deps", "--path", p])
        .assert()
        .success();

    // Verify the dep skill is written as a skill file (skills are compiled
    // into .claude/skills/<id>/SKILL.md, not inlined into CLAUDE.md)
    let skill_path = tmp
        .path()
        .join(".claude/skills/github.com/example/skills-pkg/skills/cool-skill/SKILL.md");
    assert!(
        skill_path.exists(),
        "dep skill file must be written at {:?}",
        skill_path,
    );
    let skill_content = fs::read_to_string(&skill_path).unwrap();
    assert!(
        skill_content.contains("You are a cool skill"),
        "skill file must contain dep skill content, got: {}",
        skill_content
    );
}
