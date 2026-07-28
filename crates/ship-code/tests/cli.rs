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
/// the database (ship-code use writes the active_agent projection).
fn ship() -> Command {
    Command::cargo_bin("ship-code").unwrap()
}

/// CLI command with DB isolated to `tmp`'s temp dir.
/// All calls within the same test share state; different tests are isolated.
fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship-code").unwrap();
    cmd.env("SHIP_HOME", tmp.path());
    cmd
}

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// Scaffold the bare-minimum STACK that `ship use` and `ship compile`
/// need: the manifest marker. Agents ride `agents/<id>.jsonc` (written by
/// each test as needed).
fn scaffold_ship_dir(root: &Path) {
    write_ship_config(root, "");
}

fn write_ship_config(root: &Path, body: &str) {
    write(
        root,
        "stack.config.ts",
        &format!(
            r#"
import {{
  defineConfig,
}} from "ship";

{body}
"#
        ),
    );
}

/// Write the per-noun source surface slice 2c expects: an on-disk rule
/// at `rules/<id>.md`, an on-disk agent at `agents/<id>.ts`, and a stub
/// for the `ship use` activation gate (`.ship/agents/<id>.jsonc`).
/// The `stack.config.ts` is a thin pointer (`defineConfig({})`).
/// Rule bodies live on the agent composition as
/// `rules: { inline: "..." }` — the rules-format cut collapsed the
/// on-disk `rules/<id>.md` convention, so the rule content rides the
/// `defineAgent` call's inline block (the `rule_id` is the agent
/// id post-cut; the helper signature preserves the historical
/// `rule_id` parameter so the call sites still look like rule
/// authoring — the value is ignored except as an explanatory tag).
fn write_rule_and_agent(
    root: &Path,
    _rule_id: &str,
    rule_content: &str,
    agent_id: &str,
    agent_name: &str,
) {
    std::fs::create_dir_all(root.join("agents")).unwrap();
    write(
        root,
        &format!("agents/{agent_id}.ts"),
        &format!(
            r#"
import {{ defineAgent }} from "ship";

defineAgent({{
  agent: {{ id: "{agent_id}", name: "{agent_name}" }},
  rules: {{ inline: {rule_content:?} }},
}});
"#
        ),
    );
    write_agent_stub(root, agent_id, agent_name);
    write_ship_config(root, "");
}

/// `ship use <id>` still gates on a raw `.ship/agents/<id>.jsonc` file
/// (`crates/code/ship-code/src/bin/profile.rs::find_agent_file`/`load_agent_profile`) —
/// that lookup was never migrated onto the ts-declare-form roll-up the
/// compile pipeline itself uses. The stub only needs to exist for the
/// activation gate + plugin-lifecycle read; the agent's real skills/mcp/
/// rules/permissions effect comes from the matching `defineAgent` in
/// `agents/<id>.ts` (composed by `compile_project`'s roll-up).
fn write_agent_stub(root: &Path, id: &str, name: &str) {
    write(
        root,
        &format!("agents/{id}.jsonc"),
        &format!(r#"{{ "agent": {{ "id": "{id}", "name": "{name}" }} }}"#),
    );
}

// ── ship use ──────────────────────────────────────────────────────────────────

#[test]
fn use_activates_agent_writes_claude_md_and_state() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    scaffold_ship_dir(tmp.path());
    // Slice 2c: rule on disk at `rules/style.md`, agent on disk at
    // `agents/my-agent.ts`, manifest is a thin pointer. The
    // `.ship/agents/my-agent.jsonc` stub stays for the `ship use`
    // activation gate.
    write_rule_and_agent(tmp.path(), "style", "Use explicit types.", "my-agent", "My Agent");

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
    write_rule_and_agent(tmp.path(), "style", "Use explicit types.", "my-agent", "My Agent");

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
        .stderr(predicate::str::contains("no stack.config.ts found"));
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
    scaffold_ship_dir(tmp.path());
    // Rules-format cut: the rule body lives on the agent composition
    // as `rules: { inline: "..." }`. The on-disk `rules/<id>.md`
    // convention died — the agent is the only authoring surface.
    write_rule_and_agent(tmp.path(), "style", "Use explicit types.", "demo", "Demo");

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
    scaffold_ship_dir(tmp.path());
    write_rule_and_agent(tmp.path(), "style", "Use explicit types.", "demo", "Demo");

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
        .stderr(predicate::str::contains("no stack.config.ts found"));
}

// ── ship compile --harness ───────────────────────────────────────────────────

#[test]
fn compile_harness_codex_writes_agents_md() {
    let tmp = TempDir::new().unwrap();
    scaffold_ship_dir(tmp.path());
    write_rule_and_agent(tmp.path(), "style", "Use explicit types.", "demo", "Demo");

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
    scaffold_ship_dir(tmp.path());
    write_rule_and_agent(tmp.path(), "style", "Use explicit types.", "demo", "Demo");

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
    scaffold_ship_dir(tmp.path());
    write_rule_and_agent(tmp.path(), "style", "Use explicit types.", "demo", "Demo");

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

    // Rules-format cut: the rule body lives on the agent composition
    // as `rules: { inline: "..." }`. The on-disk `rules/<id>.md`
    // convention died — the agent is the only authoring surface.
    // The MCP server registration lives under
    // `defineConfig({ agents: { mcp: { ... } } })`; the permissions
    // preset stays in `.ship/permissions.jsonc` (untouched by the
    // migration).
    std::fs::create_dir_all(tmp.path().join("agents")).unwrap();
    write(
        tmp.path(),
        "agents/fullstack.ts",
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: {
    id: "fullstack",
    name: "Fullstack Dev",
    version: "0.1.0",
    description: "Full-stack development agent",
  },
  skills: { refs: [] },
  mcp: { servers: [] },
  permissions: {
    preset: "ship-autonomous",
    tools_deny: ["Bash(docker rm*)"],
  },
  rules: {
    inline: "Prefer composition over inheritance. Use explicit return types.\n\nAlways write tests before implementation.",
  },
  hooks: [
    { id: "perms-sync", trigger: "Stop", command: "ship permissions sync" },
  ],
});
"#,
    );
    // The MCP block + the named permission presets both live in the
    // manifest's `defineConfig({ agents: { mcp, permissions } })`
    // block (slice 2c added the MCP block; slice 3 close-out
    // added the permissions.presets block — the .ship/permissions.jsonc
    // file died with this slice).
    write_ship_config(
        tmp.path(),
        r#"export default defineConfig({
  agents: {
    mcp: {
      github: {
        id: "github",
        server_type: "stdio",
        command: "npx",
        args: ["-y", "@modelcontextprotocol/server-github"],
        env: {},
        url: null,
        scope: "project",
      },
    },
    permissions: {
      presets: {
        "ship-autonomous": {
          default_mode: "dontAsk",
          tools_allow: ["Read", "Write", "Edit", "Glob", "Grep", "Bash(*)"],
          tools_deny: [
            "Bash(rm -rf *)",
            "Bash(git reset --hard*)",
            "Bash(git push*)"
          ]
        }
      }
    },
  },
});"#,
    );
    write_agent_stub(tmp.path(), "fullstack", "Fullstack Dev");

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

    // Rules-format cut: the rule body lives on the agent composition
    // as `rules: { inline: "..." }`. The on-disk `rules/<id>.md`
    // convention died — the agent is the only authoring surface.
    write_rule_and_agent(
        tmp.path(),
        "architecture",
        "Keep modules under 300 lines. Prefer pure functions.",
        "demo",
        "Demo",
    );
    write_ship_config(
        tmp.path(),
        r#"export default defineConfig({
  agents: {
    mcp: {
      "test-server": {
        id: "test-server",
        server_type: "stdio",
        command: "echo",
        args: ["hello"],
        env: {},
        url: null,
        scope: "project",
      },
    },
  },
});"#,
    );

    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{ "ship-standard": { "default_mode": "default" } }"#,
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

    // The named permission preset lives in the manifest's
    // `defineConfig({ agents: { permissions: { presets: { ... } } })`
    // block (slice 3 close-out, A4 — the .ship/permissions.jsonc file
    // died with this slice).
    write_ship_config(
        tmp.path(),
        r#"export default defineConfig({
  agents: {
    permissions: {
      presets: {
        "ship-autonomous": {
          default_mode: "dontAsk",
          tools_allow: ["Read", "Write", "Edit", "Bash(*)"],
          tools_deny: [
            "Bash(rm -rf *)",
            "Bash(git reset --hard*)",
            "Bash(git push*)"
          ]
        }
      }
    },
  },
});"#,
    );

    // Agent uses preset AND adds its own tools_deny entries (on disk
    // at `agents/strict-auto.ts` per slice 2c).
    std::fs::create_dir_all(tmp.path().join("agents")).unwrap();
    write(
        tmp.path(),
        "agents/strict-auto.ts",
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "strict-auto", name: "Strict Autonomous", version: "0.1.0" },
  skills: { refs: [] },
  mcp: { servers: [] },
  permissions: {
    preset: "ship-autonomous",
    tools_deny: ["Bash(docker rm*)", "Bash(cargo publish*)"],
  },
});
"#,
    );
    write_agent_stub(tmp.path(), "strict-auto", "Strict Autonomous");

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

    // ship.lock with a matching entry
    write(
        tmp.path(),
        "ship.lock",
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

    // Agent referencing the dep skill (on disk at `agents/with-deps.ts`).
    std::fs::create_dir_all(tmp.path().join("agents")).unwrap();
    write(
        tmp.path(),
        "agents/with-deps.ts",
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "with-deps", name: "With Deps", version: "0.1.0" },
  skills: { refs: ["github.com/example/skills-pkg/skills/cool-skill"] },
  mcp: { servers: [] },
});
"#,
    );
    write_ship_config(tmp.path(), "");
    write_agent_stub(tmp.path(), "with-deps", "With Deps");

    // Simulate installed package cache at $HOME/.ship/cache/objects/<hex>/
    // The dep_skills resolver uses home::home_dir() for the cache path.
    // Set HOME to tmp so the cache is found at tmp/.ship/cache/objects/<hex>/
    write(
        tmp.path(),
        &format!("cache/objects/{}/skills/cool-skill/SKILL.md", hex),
        "---\nname: Cool Skill\ndescription: A cool dependency skill\n---\n\nYou are a cool skill. Follow best practices.",
    );

    // Use ship_in for DB isolation. SHIP_HOME is the `.ship` root (not its
    // parent), so the cache lives at <tmp>/cache/... — dep_skills now routes
    // through `ShipHome::from_env()` per datasafety-shiphome.
    ship_in(&tmp)
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
