use crate::compile::agents::compile_agent_profiles;
use crate::types::AgentProfile;
use crate::types::agent_profile::*;

fn make_profile(id: &str, name: &str) -> AgentProfile {
    AgentProfile {
        agent: ProfileMeta {
            id: id.to_string(),
            name: name.to_string(),
            version: None,
            description: Some(format!("{name} agent")),
            role: None,
            tags: vec![],
            tools_preset: None,
            active_tools: vec![],
            icon: None,
        },
        skills: SkillRefs {
            refs: vec!["skill-a".to_string()],
        },
        mcp: McpRefs {
            servers: vec!["ship".to_string()],
        },
        plugins: PluginRefs::default(),
        permissions: ProfilePermissions {
            preset: Some("ship-standard".to_string()),
            tools_deny: vec!["Bash(rm -rf *)".to_string()],
            default_mode: Some("acceptEdits".to_string()),
            ..Default::default()
        },
        rules: ProfileRules {
            inline: Some("You are a specialist.".to_string()),
        },
        hooks: Default::default(),
        apps: Default::default(),
        harness_settings: Default::default(),
    }
}

#[test]
fn claude_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer");
    let files = compile_agent_profiles(&[p], "claude");
    assert_eq!(files.len(), 1);
    let content = &files[".claude/agents/reviewer.md"];
    assert!(content.starts_with("---\n"));
    assert!(content.contains("name: reviewer"));
    assert!(content.contains("description: Code Reviewer agent"));
    assert!(content.contains("permissionMode: acceptEdits"));
    assert!(content.contains("mcpServers:"));
    assert!(content.contains("  - ship"));
    assert!(content.contains("skills:"));
    assert!(content.contains("  - skill-a"));
    assert!(content.contains("disallowedTools:"));
    assert!(content.contains("You are a specialist."));
}

#[test]
fn cursor_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer");
    let files = compile_agent_profiles(&[p], "cursor");
    assert_eq!(files.len(), 1);
    let content = &files[".cursor/agents/reviewer.md"];
    assert!(content.starts_with("---\n"));
    assert!(content.contains("name: reviewer"));
    assert!(content.contains("description: Code Reviewer agent"));
    assert!(content.contains("You are a specialist."));
}

#[test]
fn codex_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer");
    let files = compile_agent_profiles(&[p], "codex");
    assert_eq!(files.len(), 1);
    let content = &files[".codex/agents/reviewer.toml"];
    assert!(content.contains("name = \"Code Reviewer\""));
    assert!(content.contains("description = \"Code Reviewer agent\""));
    assert!(content.contains("[mcp_servers.ship]"));
}

#[test]
fn opencode_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer");
    let files = compile_agent_profiles(&[p], "opencode");
    assert_eq!(files.len(), 1);
    let content = &files[".opencode/agents/reviewer.md"];
    assert!(content.starts_with("---\n"));
    assert!(content.contains("description: Code Reviewer agent"));
    assert!(content.contains("mode: subagent"));
    assert!(content.contains("You are a specialist."));
}

#[test]
fn multiple_profiles() {
    let profiles = vec![
        make_profile("alpha", "Alpha"),
        make_profile("beta", "Beta"),
    ];
    let files = compile_agent_profiles(&profiles, "claude");
    assert_eq!(files.len(), 2);
    assert!(files.contains_key(".claude/agents/alpha.md"));
    assert!(files.contains_key(".claude/agents/beta.md"));
}

#[test]
fn yaml_quoting() {
    use crate::compile::agents::yaml_quote;
    assert_eq!(yaml_quote("simple"), "simple");
    assert_eq!(yaml_quote("has: colon"), "\"has: colon\"");
    assert_eq!(yaml_quote("Bash(rm -rf *)"), "\"Bash(rm -rf *)\"");
}

// ── Harness-agnostic baseline: a profile with no harnesses field compiles
// for every harness in the project's list (acceptance criterion #6).

#[test]
fn profile_with_no_harnesses_field_compiles_for_every_harness() {
    let p = make_profile("anything", "Anything");
    for harness in ["claude", "cursor", "codex", "opencode"] {
        let files = compile_agent_profiles(std::slice::from_ref(&p), harness);
        assert_eq!(
            files.len(),
            1,
            "expected harness '{harness}' to emit one agent file"
        );
    }
}

// ── harness_settings is the per-agent OVERRIDE mechanism (acceptance #7).
// A profile with `harness_settings.codex.model = "X"` emits `model: X`
// ONLY in the codex file — not claude/cursor/opencode.

#[test]
fn harness_settings_model_override_only_lands_in_matching_harness() {
    let mut p = make_profile("specialist", "Specialist");
    p.harness_settings.insert(
        "codex".to_string(),
        serde_json::json!({ "model": "codex-test-model" }),
    );

    // Codex receives the model override.
    let codex_files = compile_agent_profiles(&[p.clone()], "codex");
    let codex = &codex_files[".codex/agents/specialist.toml"];
    assert!(
        codex.contains("model = \"codex-test-model\""),
        "codex file must carry the model override; got:\n{codex}"
    );

    // Other harnesses MUST NOT carry the codex-only override.
    let expected_paths: &[(&str, &str)] = &[
        ("claude", ".claude/agents/specialist.md"),
        ("cursor", ".cursor/agents/specialist.md"),
        ("opencode", ".opencode/agents/specialist.md"),
    ];
    for (harness, path) in expected_paths {
        let files = compile_agent_profiles(&[p.clone()], harness);
        let content = files
            .get(*path)
            .unwrap_or_else(|| panic!("expected {path} to exist for harness {harness}"));
        assert!(
            !content.contains("codex-test-model"),
            "harness '{harness}' must NOT receive the codex-only model override; \
             content was:\n{content}"
        );
    }
}
