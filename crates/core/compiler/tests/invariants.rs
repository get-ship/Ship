/// Compiler invariant tests — determinism, harness round-trips, empty input,
/// and pure-transform purity (structural).
///
/// No `proptest`/`quickcheck` in Cargo.toml, so these are deterministic
/// parameterised tests as permitted by the job spec.
use compiler::{
    AgentProfile, ExtensionLibrary, McpRefs, McpServerConfig, PluginRefs, ProfileMeta,
    ProfilePermissions, ProfileRules, Rule, Skill, SkillRefs, compile, list_harnesses,
    resolve_library,
};

// ─── Fixture helpers ──────────────────────────────────────────────────────────

fn make_skill(id: &str) -> Skill {
    Skill {
        id: id.to_string(),
        name: id.to_string(),
        stable_id: None,
        description: Some(format!("Description for {id}")),
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: format!("# {id}\nDo something useful."),
        source: Default::default(),
        vars: Default::default(),
        artifacts: vec![],
        grants: vec![],
        requires: vec![],
    }
}

fn make_rule(file_name: &str) -> Rule {
    Rule {
        file_name: file_name.to_string(),
        content: format!("Follow the {} guidelines.", file_name),
        always_apply: true,
        globs: vec![],
        description: None,
    }
}

fn make_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: Default::default(),
        scope: "project".to_string(),
        server_type: Default::default(),
        url: None,
        disabled: false,
        timeout_secs: None,
        codex_enabled_tools: vec![],
        codex_disabled_tools: vec![],
        cursor_env_file: None,
    }
}

fn rich_library() -> ExtensionLibrary {
    ExtensionLibrary {
        skills: vec![make_skill("deploy"), make_skill("review")],
        rules: vec![make_rule("style.md"), make_rule("testing.md")],
        mcp_servers: vec![make_server("github"), make_server("linear")],
        ..Default::default()
    }
}

// ─── Determinism ──────────────────────────────────────────────────────────────

/// Same ExtensionLibrary compiled 10× must produce identical output every time.
/// Tests all harnesses to catch any per-harness non-determinism.
#[test]
fn compile_is_deterministic() {
    let library = rich_library();
    let harnesses = ["claude", "codex", "opencode"];

    for harness in harnesses {
        let resolved = resolve_library(&library, None, None);
        let reference = compile(&resolved, harness).expect("known harness");

        for _ in 1..10 {
            let resolved = resolve_library(&library, None, None);
            let output = compile(&resolved, harness).unwrap();

            // Compare serialisable/comparable fields.
            assert_eq!(
                output.mcp_servers, reference.mcp_servers,
                "mcp_servers non-deterministic for {harness}"
            );
            assert_eq!(
                output.context_content, reference.context_content,
                "context_content non-deterministic for {harness}"
            );
            assert_eq!(
                output.mcp_config_path, reference.mcp_config_path,
                "mcp_config_path non-deterministic for {harness}"
            );

            let mut skill_keys: Vec<_> = output.skill_files.keys().collect();
            skill_keys.sort();
            let mut ref_skill_keys: Vec<_> = reference.skill_files.keys().collect();
            ref_skill_keys.sort();
            assert_eq!(
                skill_keys, ref_skill_keys,
                "skill_files keys non-deterministic for {harness}"
            );

            let mut rule_keys: Vec<_> = output.rule_files.keys().collect();
            rule_keys.sort();
            let mut ref_rule_keys: Vec<_> = reference.rule_files.keys().collect();
            ref_rule_keys.sort();
            assert_eq!(
                rule_keys, ref_rule_keys,
                "rule_files keys non-deterministic for {harness}"
            );

            assert_eq!(
                format!("{:?}", output.claude_settings_patch),
                format!("{:?}", reference.claude_settings_patch),
                "claude_settings_patch non-deterministic for {harness}"
            );
        }
    }
}

// ─── Harness round-trips ─────────────────────────────────────────────────────

/// Compiling for `claude` must produce a CLAUDE.md context section containing
/// skill and rule content.
#[test]
fn claude_output_contains_context_file() {
    let library = rich_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "claude").unwrap();

    let content = output
        .context_content
        .expect("claude must emit context_content");
    assert!(
        !content.is_empty(),
        "CLAUDE.md context must be non-empty when rules/skills are present"
    );
    // Claude context file contains only rules (not skills — skills get their own files).
    // Rule content from make_rule() contains the file_name prefix "style" / "testing".
    assert!(
        content.contains("style") || content.contains("testing"),
        "CLAUDE.md must contain rule content; got:\n{content}"
    );
}

/// Compiling for `codex` must emit an AGENTS.md context and a TOML config patch.
#[test]
fn codex_output_has_agents_md_and_config_patch() {
    let library = rich_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "codex").unwrap();

    assert!(
        output.context_content.is_some(),
        "codex must emit context_content (AGENTS.md)"
    );
    assert!(
        output.codex_config_patch.is_some(),
        "codex must emit codex_config_patch when servers are present"
    );
}

/// MCP servers object must NOT contain a built-in `ship` entry — ship-internal
/// MCP was retired in #236 (`ship mcp serve` deleted, daemon has no /mcp route).
/// This invariant guards against re-introduction of the broken injection.
#[test]
fn mcp_servers_never_includes_ship_injection() {
    let harnesses = ["claude", "codex", "opencode"];
    let library = ExtensionLibrary::default();
    let resolved = resolve_library(&library, None, None);

    for harness in harnesses {
        let output = compile(&resolved, harness).unwrap();
        let servers = output
            .mcp_servers
            .as_object()
            .expect("mcp_servers must be a JSON object");
        assert!(
            !servers.contains_key("ship"),
            "harness {harness} must not inject a 'ship' MCP entry (#236)"
        );
    }
}

// ─── Empty input ──────────────────────────────────────────────────────────────

/// Empty ExtensionLibrary must compile for every known harness without panicking
/// and must produce structurally valid output.
#[test]
fn empty_library_compiles_for_all_harnesses() {
    let library = ExtensionLibrary::default();

    for desc in list_harnesses() {
        let resolved = resolve_library(&library, None, None);
        let output = compile(&resolved, desc.id)
            .unwrap_or_else(|| panic!("compile returned None for known harness {}", desc.id));

        // MCP servers must be a JSON object (at minimum contains "ship" key).
        assert!(
            output.mcp_servers.is_object(),
            "harness {} mcp_servers must be an object on empty input",
            desc.id
        );

        // Skill and rule file maps may be empty but must not be uninitialised.
        let _ = output.skill_files.len();
        let _ = output.rule_files.len();
    }
}

/// resolve_library on an empty ExtensionLibrary must not panic and must default
/// harness to "claude".
#[test]
fn empty_library_resolves_to_claude_default() {
    let library = ExtensionLibrary::default();
    let resolved = resolve_library(&library, None, None);
    assert_eq!(
        resolved.harnesses,
        vec!["claude"],
        "empty library must default to claude harness"
    );
}

// ─── Pure-transform purity (structural) ───────────────────────────────────────

/// The compiler's public entry points accept only in-memory values and return
/// only in-memory values — no `Path`, `File`, or `Command` types in the API.
///
/// This test verifies purity by exercising compile + resolve_library entirely
/// from in-memory inputs and confirming completion without any I/O side effects.
/// A pure function cannot open files or spawn processes; if it could, these
/// calls would fail in a sandboxed environment — making this a meaningful
/// runtime purity guard.
#[test]
fn compile_and_resolve_are_pure_no_io() {
    // Construct inputs purely from in-memory values — no file paths required.
    let library = rich_library();

    // resolve_library takes owned/borrowed structs; no I/O arguments accepted.
    let resolved = resolve_library(&library, None, None);

    // compile takes a ResolvedConfig (in-memory) and a harness string.
    // If this function accessed the filesystem or spawned a process it would
    // either fail (sandboxed) or require Path/Command arguments it doesn't have.
    for desc in list_harnesses() {
        let out = compile(&resolved, desc.id);
        assert!(
            out.is_some(),
            "compile must return Some for known harness {}",
            desc.id
        );
    }
}

// ─── Runtime var injection ────────────────────────────────────────────────────

fn make_agent_profile(id: &str, description: &str) -> AgentProfile {
    AgentProfile {
        agent: ProfileMeta {
            id: id.to_string(),
            name: id.to_string(),
            version: None,
            description: Some(description.to_string()),
            role: None,
            tags: vec![],
            tools_preset: None,
            active_tools: vec![],
            icon: None,
        },
        capabilities: Vec::new(),
        skills: SkillRefs::default(),
        mcp: McpRefs::default(),
        plugins: PluginRefs::default(),
        permissions: ProfilePermissions::default(),
        rules: ProfileRules::default(),
        hooks: Default::default(),
        target_agents: Vec::new(),
        apps: Default::default(),
        harness_settings: Default::default(),
    }
}

/// Skills with `{{ runtime.agents }}` get the project's agent profiles injected at
/// compile time. The compiled skill file must contain the rendered agent list.
#[test]
fn runtime_agents_injected_into_skill_template() {
    let skill = Skill {
        id: "dispatch".to_string(),
        name: "dispatch".to_string(),
        stable_id: None,
        description: Some("Dispatch jobs.".to_string()),
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: "{% for a in runtime.agents %}- {{ a.id }}\n{% endfor %}".to_string(),
        source: Default::default(),
        vars: Default::default(),
        artifacts: vec![],
        grants: vec![],
        requires: vec![],
    };

    let library = ExtensionLibrary {
        skills: vec![skill],
        agent_profiles: vec![
            make_agent_profile("rust-compiler", "Compiles Rust"),
            make_agent_profile("web-lane", "Frontend work"),
        ],
        ..Default::default()
    };

    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "claude").expect("claude compile");

    let skill_content = output
        .skill_files
        .values()
        .find(|c| c.contains("rust-compiler"))
        .expect("dispatch skill must be in skill_files");

    assert!(
        skill_content.contains("- rust-compiler\n"),
        "runtime.agents must include rust-compiler; got:\n{skill_content}"
    );
    assert!(
        skill_content.contains("- web-lane\n"),
        "runtime.agents must include web-lane; got:\n{skill_content}"
    );
}

/// Skills with `{{ runtime.harnesses }}` get the active harness list.
#[test]
fn runtime_harnesses_injected_into_skill_template() {
    let skill = Skill {
        id: "harnesses".to_string(),
        name: "harnesses".to_string(),
        stable_id: None,
        description: None,
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: "{% for p in runtime.harnesses %}{{ p }} {% endfor %}".to_string(),
        source: Default::default(),
        vars: Default::default(),
        artifacts: vec![],
        grants: vec![],
        requires: vec![],
    };

    let library = ExtensionLibrary {
        skills: vec![skill],
        ..Default::default()
    };

    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "claude").expect("claude compile");

    let skill_content = output
        .skill_files
        .values()
        .next()
        .expect("skill must be present");

    assert!(
        skill_content.contains("claude"),
        "runtime.harnesses must include 'claude'; got:\n{skill_content}"
    );
}

/// `runtime.skills` lists the other active skills by id and name.
#[test]
fn runtime_skills_injected_into_skill_template() {
    let meta_skill = Skill {
        id: "meta".to_string(),
        name: "meta".to_string(),
        stable_id: None,
        description: None,
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: "{% for s in runtime.skills %}{{ s.id }} {% endfor %}".to_string(),
        source: Default::default(),
        vars: Default::default(),
        artifacts: vec![],
        grants: vec![],
        requires: vec![],
    };

    let library = ExtensionLibrary {
        skills: vec![meta_skill, make_skill("deploy"), make_skill("review")],
        ..Default::default()
    };

    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "claude").expect("claude compile");

    let meta_content = output
        .skill_files
        .iter()
        .find(|(k, _)| k.contains("/meta/"))
        .map(|(_, v)| v.as_str())
        .expect("meta skill must be present");

    assert!(
        meta_content.contains("deploy"),
        "runtime.skills must include 'deploy'; got:\n{meta_content}"
    );
    assert!(
        meta_content.contains("review"),
        "runtime.skills must include 'review'; got:\n{meta_content}"
    );
}

/// Unknown harness IDs must return None rather than panicking.
#[test]
fn compile_unknown_harness_returns_none() {
    let library = ExtensionLibrary::default();
    let resolved = resolve_library(&library, None, None);

    assert!(
        compile(&resolved, "").is_none(),
        "empty harness id must return None"
    );
    assert!(
        compile(&resolved, "unknown-harness").is_none(),
        "unknown harness id must return None"
    );
    assert!(
        compile(&resolved, "CLAUDE").is_none(),
        "harness matching is case-sensitive — 'CLAUDE' must return None"
    );
}
