//! Tests for `compiler::load_library` — the LEAF `.ship/` → `ExtensionLibrary`
//! loader. The compiler crate is L0 / substrate-free, so this loader uses
//! only `std::fs` + the crate's own `jsonc` + `skill_parser` modules.
//!
//! RED → GREEN development: the test was authored before the loader
//! implementation, and asserted a known surface area (agents, mcp, skills,
//! rules). The implementation lives in `compiler::loader` and walks
//! `.ship/agents/`, `.ship/mcp.jsonc`, `.ship/skills/`, and `.ship/rules/`.

use std::path::PathBuf;

use compiler::{load_library, ExtensionLibrary};

fn fixture_ship_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR points at crates/core/compiler; the fixture sits
    // beside this test file under tests/load_library_fixture/.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("load_library_fixture")
        .join(".ship")
}

#[test]
fn load_library_reads_agents_mcp_skills_rules() {
    let ship_dir = fixture_ship_dir();
    let lib: ExtensionLibrary =
        load_library(&ship_dir).expect("load_library must succeed against the fixture .ship/");

    // 1. Agent profile loaded from .ship/agents/test-agent.jsonc.
    let agents = &lib.agent_profiles;
    assert_eq!(agents.len(), 1, "expected exactly one agent in the fixture");
    let agent = &agents[0];
    assert_eq!(agent.agent.id, "test-agent");
    assert_eq!(agent.agent.name, "Test Agent");
    assert_eq!(agent.skills.refs, vec!["example-skill".to_string()]);
    assert_eq!(agent.mcp.servers, vec!["test-server".to_string()]);
    assert_eq!(agent.rules.inline.as_deref(), Some("Test rule body."));

    // 2. MCP server loaded from .ship/mcp.jsonc.
    assert_eq!(lib.mcp_servers.len(), 1);
    let mcp = &lib.mcp_servers[0];
    assert_eq!(mcp.id, "test-server");
    assert_eq!(mcp.command, "npx");
    assert_eq!(mcp.args, vec!["-y", "@test/mcp"]);
    assert!(matches!(
        mcp.server_type,
        compiler::McpServerType::Stdio
    ));

    // 3. Skill loaded from .ship/skills/example-skill/SKILL.md.
    assert_eq!(lib.skills.len(), 1, "expected one skill in the fixture");
    let skill = &lib.skills[0];
    assert_eq!(skill.id, "example-skill");
    assert_eq!(skill.name, "example-skill");
    assert_eq!(
        skill.description.as_deref(),
        Some("Example skill for the load_library fixture.")
    );
    assert!(skill.content.contains("Example Skill"));

    // 4. Rule loaded from .ship/rules/01-style.md.
    assert_eq!(lib.rules.len(), 1);
    let rule = &lib.rules[0];
    assert_eq!(rule.file_name, "01-style.md");
    assert!(rule.content.contains("Style"));
    assert_eq!(
        rule.description.as_deref(),
        Some("A test rule".to_string()).as_deref()
    );

    // 5. Permissions loaded from .ship/permissions.jsonc if present
    //    (fixture does not include it; default empty Permissions is valid).
    // No assertion needed; the load call's success covers the path.

    // 6. Hooks loaded from .ship/hooks.jsonc (fixture does not include it).
    assert!(
        lib.hooks.is_empty(),
        "fixture has no hooks.jsonc, library must default to empty"
    );
}

#[test]
fn load_library_on_missing_dir_yields_empty_library() {
    // A non-existent .ship/ must NOT be a hard error — the existing
    // agent-def loader contract is "missing files are silently skipped;
    // an empty library is valid". The leaf loader must match.
    let tmp = tempfile::tempdir().expect("tempdir");
    let absent = tmp.path().join("does-not-exist");
    let lib = load_library(&absent).expect("load_library on a missing dir must succeed");
    assert!(lib.agent_profiles.is_empty());
    assert!(lib.mcp_servers.is_empty());
    assert!(lib.skills.is_empty());
    assert!(lib.rules.is_empty());
    assert!(lib.hooks.is_empty());
}
