//! Tests for compile pipeline worktree output-root separation.

use super::*;
use std::path::Path;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn setup_minimal_space(tmp: &TempDir) {
    // Rules-format cut: the rule body lives on the agent composition
    // as `rules: { inline: "..." }`. The on-disk `rules/<id>.md`
    // convention died — the agent is the only authoring surface.
    // The MCP registration lives under
    // `defineConfig({ agents: { mcp: { ... } } })` in the manifest
    // (the per-extension-project config block — slice 2c's
    // new home for the per-call `defineMcpServer` form).
    write(
        tmp.path(),
        "stack.config.ts",
        r#"
import { defineConfig } from "ship";

export default defineConfig({
  agents: {
    mcp: {
      github: {
        id: "github",
        server_type: "stdio",
        command: "npx",
        args: ["-y", "@mcp/github"],
        env: {},
        url: null,
        scope: "project",
      },
    },
  },
});
"#,
    );
    write(
        tmp.path(),
        "agents/demo.ts",
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
  rules: { inline: "Use explicit types." },
});
"#,
    );
}

#[test]
fn compile_worktree_output_root_separates_from_space_root() {
    // Simulates the actual worktree case: .ship/ config lives in space_root
    // but compiled output (CLAUDE.md, .mcp.json) goes to a separate output_root.
    // SHIP_HOME pinned via the shared lock — config tests in another module
    // mutate the same env var; without coordination they'd race and the
    // emitted .mcp.json would carry whichever tempdir won.
    let home = TempDir::new().unwrap();
    let ship_home = home.path().join(".ship");
    std::fs::create_dir_all(&ship_home).unwrap();
    let _home_guard = crate::test_env::isolated_ship_home(&ship_home);

    let space = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    setup_minimal_space(&space);

    run_compile(CompileOptions {
        space_root: space.path(),
        output_root: Some(output.path()),
        harness: Some("claude"),
        dry_run: false,
        active_agent: None,
        extra_skills: vec![],
            plugins_installed: vec![],
            scoped_launch: false,
            default_effort: None,
    })
    .unwrap();

    // Output files must be in output_root, not space_root.
    assert!(output.path().join(".mcp.json").exists(), ".mcp.json must be written to output_root");
    assert!(!space.path().join(".mcp.json").exists(), ".mcp.json must NOT be written to space_root");
    assert!(output.path().join("CLAUDE.md").exists(), "CLAUDE.md must be written to output_root");
    assert!(!space.path().join("CLAUDE.md").exists(), "CLAUDE.md must NOT be written to space_root");

    // Verify .mcp.json content: the ship-internal MCP server is NOT injected
    // (ripped in #236 — MCP was culled). User-provided external servers
    // (github, from setup_minimal_space) must still compile through.
    let content = std::fs::read_to_string(output.path().join(".mcp.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        parsed["mcpServers"]["ship"].is_null(),
        "ship server must NOT be in .mcp.json after the MCP rip (#236)"
    );
    assert!(
        parsed["mcpServers"]["github"].is_object(),
        "external user MCP servers must still compile through"
    );
}
