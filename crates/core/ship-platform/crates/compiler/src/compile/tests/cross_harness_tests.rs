use crate::compile::harness::ContextFile;
use crate::compile::{CompileOutput, compile, list_harnesses};
use crate::resolve::ResolvedConfig;
use crate::types::{HookTrigger, Permissions, Rule, ToolPermissions};

use super::fixtures::*;

// ── Determinism ───────────────────────────────────────────────────────────────

/// Compiling the same input twice must produce identical output. `compile` is
/// pure, so the only way this regresses is a nondeterministically-serialized
/// collection (e.g. an unordered map) leaking into an output field — one
/// harness is enough to catch that; the property is harness-independent.
#[test]
fn compile_is_idempotent() {
    use crate::types::AgentLimits;
    let r = ResolvedConfig {
        mcp_servers: vec![make_server("github"), make_server("linear")],
        skills: vec![make_skill("rust-expert")],
        rules: vec![Rule {
            file_name: "style.md".into(),
            content: "Use explicit types.".into(),
            always_apply: true,
            globs: vec![],
            description: None,
        }],
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["Bash(rm -rf *)".to_string()],
                ..Default::default()
            },
            agent: AgentLimits {
                max_cost_per_session: Some(2.50),
                ..Default::default()
            },
            ..Default::default()
        },
        hooks: vec![make_hook(HookTrigger::PostToolUse, "ship log", Some("*"))],
        ..resolved(vec![])
    };

    let first = compile(&r, "claude").unwrap();
    let second = compile(&r, "claude").unwrap();

    assert_eq!(
        serde_json::to_string(&first.mcp_servers).unwrap(),
        serde_json::to_string(&second.mcp_servers).unwrap(),
        "MCP output must be identical across compilations"
    );
    assert_eq!(first.context_content, second.context_content);
    assert_eq!(first.skill_files, second.skill_files);
    assert_eq!(
        serde_json::to_string(&first.claude_settings_patch).unwrap(),
        serde_json::to_string(&second.claude_settings_patch).unwrap(),
        "Settings patch must be identical across compilations"
    );
}

// ── Conformance: properties every registered harness must satisfy ─────────────
//
// These iterate `list_harnesses()` so adding or dropping a harness updates
// coverage with zero test edits — the registry is the single source of truth.
// (Replaces the old hardcoded `["claude","codex","cursor"]` loops, which both
// duplicated the roster and re-asserted properties already covered elsewhere.)

/// Exercises every output channel — MCP servers, a skill, a rule, a hook, and a
/// deny permission — enough to trigger each harness's owned patch outputs.
fn conformance_config() -> ResolvedConfig {
    ResolvedConfig {
        mcp_servers: vec![make_server("github")],
        skills: vec![{
            let mut s = make_skill("refactor");
            s.content = "Refactor carefully.".into();
            s
        }],
        rules: vec![Rule {
            file_name: "style.md".into(),
            content: "Write clean code.".into(),
            always_apply: true,
            globs: vec![],
            description: None,
        }],
        hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Bash(rm -rf *)".into()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    }
}

/// Each harness emits its declared context file (CLAUDE.md / AGENTS.md / …)
/// when rules are present. A harness with no context file (cursor) emits
/// per-file rules instead.
#[test]
fn conformance_context_output_matches_descriptor() {
    let r = conformance_config();
    for desc in list_harnesses() {
        let out = compile(&r, desc.id).unwrap();
        if desc.context_file == ContextFile::None {
            assert!(
                !out.rule_files.is_empty(),
                "{}: has no context file, so must emit per-file rules",
                desc.id
            );
        } else {
            assert!(
                out.context_content.is_some(),
                "{}: declares context file {:?} but emitted no context_content",
                desc.id,
                desc.context_file.file_name()
            );
        }
    }
}

/// Skills land in each harness's declared skills directory.
#[test]
fn conformance_skills_land_in_declared_dir() {
    let r = conformance_config();
    for desc in list_harnesses() {
        let Some(base) = desc.skills_dir.base_path() else {
            continue; // harness doesn't surface skills as on-disk files
        };
        let out = compile(&r, desc.id).unwrap();
        let expected = format!("{base}/refactor/SKILL.md");
        assert!(
            out.skill_files.contains_key(&expected),
            "{}: skill must land at {expected}; got {:?}",
            desc.id,
            out.skill_files.keys().collect::<Vec<_>>()
        );
    }
}

/// Owned patch outputs are exclusive: each appears only for the harness that
/// owns it. The owner is explicit (each output is harness-specific); the
/// "everyone else must NOT emit it" sweep is registry-driven, so a newly
/// registered harness is automatically held to the no-leak contract.
#[test]
fn conformance_patch_outputs_are_owner_exclusive() {
    let r = conformance_config();
    let outs: Vec<(&str, CompileOutput)> = list_harnesses()
        .iter()
        .map(|d| (d.id, compile(&r, d.id).unwrap()))
        .collect();

    let owned: &[(&str, &str, fn(&CompileOutput) -> bool)] = &[
        ("claude_settings_patch", "claude", |o| {
            o.claude_settings_patch.is_some()
        }),
        ("codex_config_patch", "codex", |o| {
            o.codex_config_patch.is_some()
        }),
        ("opencode_config_patch", "opencode", |o| {
            o.opencode_config_patch.is_some()
        }),
        ("cursor_hooks_patch", "cursor", |o| {
            o.cursor_hooks_patch.is_some()
        }),
        ("cursor_cli_permissions", "cursor", |o| {
            o.cursor_cli_permissions.is_some()
        }),
        ("rule_files", "cursor", |o| !o.rule_files.is_empty()),
    ];

    for (label, owner, present) in owned {
        for (id, out) in &outs {
            let got = present(out);
            let want = id == owner;
            assert_eq!(
                got, want,
                "{label}: emitted={got} for harness {id}, but it is owned by {owner} only"
            );
        }
    }
}

/// No harness injects a "ship" MCP entry — ship-internal MCP was retired (#236).
/// External servers pass through unchanged regardless of scoped_launch state.
#[test]
fn conformance_no_ship_injection_any_harness() {
    let r = ResolvedConfig {
        scoped_launch: true,
        ..conformance_config()
    };
    for desc in list_harnesses() {
        if !desc.features.supports_mcp {
            continue;
        }
        let out = compile(&r, desc.id).unwrap();
        assert!(
            out.mcp_servers.get("ship").is_none(),
            "{}: must not inject a 'ship' MCP entry (ship-internal MCP retired #236)",
            desc.id
        );
    }
}

/// compile_library_all via ProjectLibrary round-trip.
#[test]
fn library_round_trip_via_resolve() {
    use crate::resolve::{ProjectLibrary, resolve_library};

    let library = ProjectLibrary {
        mcp_servers: vec![make_server("github")],
        skills: vec![make_skill("workflow")],
        rules: vec![Rule {
            file_name: "style.md".into(),
            content: "Keep it clean.".into(),
            always_apply: true,
            globs: vec![],
            description: None,
        }],
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["Bash(sudo *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        hooks: vec![make_hook(
            HookTrigger::PreToolUse,
            "ship check",
            Some("Bash"),
        )],
        ..Default::default()
    };

    let resolved = resolve_library(&library, None, None);
    assert_eq!(
        resolved.hooks.len(),
        1,
        "hooks must survive resolve_library"
    );

    let out = compile(&resolved, "claude").unwrap();

    assert!(out.mcp_servers["github"].is_object());
    assert!(out.context_content.unwrap().contains("Keep it clean."));
    assert!(
        out.skill_files
            .contains_key(".claude/skills/workflow/SKILL.md")
    );

    let patch = out.claude_settings_patch.unwrap();
    assert_eq!(patch["permissions"]["deny"][0], "Bash(sudo *)");
    assert!(patch["hooks"]["PreToolUse"].is_array());
    assert_eq!(patch["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    assert_eq!(out.mcp_config_path.as_deref(), Some(".mcp.json"));
}
