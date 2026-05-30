//! Tests for compile pipeline worktree output-root separation.

use super::*;
use std::path::Path;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn setup_minimal_project(tmp: &TempDir) {
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{ "mcp": { "servers": { "github": { "id": "github", "command": "npx", "args": ["-y", "@mcp/github"] } } } }"#,
    );
}

#[test]
fn compile_worktree_output_root_separates_from_project_root() {
    // Simulates the actual worktree case: .ship/ config lives in project_root
    // but compiled output (CLAUDE.md, .mcp.json) goes to a separate output_root.
    // SHIP_HOME pinned via the shared lock — config tests in another module
    // mutate the same env var; without coordination they'd race and the
    // emitted .mcp.json would carry whichever tempdir won.
    let home = TempDir::new().unwrap();
    let ship_home = home.path().join(".ship");
    std::fs::create_dir_all(&ship_home).unwrap();
    let _home_guard = crate::test_env::isolated_ship_home(&ship_home);

    let project = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    setup_minimal_project(&project);

    run_compile(CompileOptions {
        project_root: project.path(),
        output_root: Some(output.path()),
        harness: Some("claude"),
        dry_run: false,
        active_agent: None,
        extra_skills: vec![],
            plugins_installed: vec![],
            scoped_launch: false,
    })
    .unwrap();

    // Output files must be in output_root, not project_root.
    assert!(output.path().join(".mcp.json").exists(), ".mcp.json must be written to output_root");
    assert!(!project.path().join(".mcp.json").exists(), ".mcp.json must NOT be written to project_root");
    assert!(output.path().join("CLAUDE.md").exists(), "CLAUDE.md must be written to output_root");
    assert!(!project.path().join("CLAUDE.md").exists(), "CLAUDE.md must NOT be written to project_root");

    // Verify .mcp.json content: the ship-internal MCP server is NOT injected
    // (ripped in #236 — MCP was culled). User-provided external servers
    // (github, from setup_minimal_project) must still compile through.
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
