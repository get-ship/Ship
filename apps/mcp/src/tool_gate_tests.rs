use super::*;

// ── Preset tool sets ───────────────────────────────────────────────

#[test]
fn worker_preset_contains_platform_tools() {
    let tools = ShipServer::tools_for_preset("worker");
    assert!(tools.contains(&"log_progress"), "worker missing log_progress");
    assert!(tools.contains(&"update_job"), "worker missing update_job");
    assert!(tools.contains(&"step_done"), "worker missing step_done — pipelines depend on it");
}

#[test]
fn worker_preset_excludes_orchestration_tools() {
    let tools = ShipServer::tools_for_preset("worker");
    assert!(
        !tools.contains(&"discover_mesh_agents"),
        "worker must not have discover_mesh_agents"
    );
    assert!(
        !tools.contains(&"update_mesh_status"),
        "worker must not have update_mesh_status"
    );
}

#[test]
fn orchestrator_preset_inherits_worker() {
    let worker = ShipServer::tools_for_preset("worker");
    let orchestrator = ShipServer::tools_for_preset("orchestrator");
    for tool in worker {
        assert!(
            orchestrator.contains(&tool),
            "orchestrator should inherit worker tool {tool}"
        );
    }
    // ORCHESTRATION_TOOLS is empty since mesh observe/status moved to the
    // CLI — the orchestrator preset currently adds nothing over worker.
}

#[test]
fn unknown_preset_falls_back_to_worker() {
    let unknown = ShipServer::tools_for_preset("nonexistent");
    let worker = ShipServer::tools_for_preset("worker");
    assert_eq!(unknown, worker, "unknown preset should fall back to worker");
}

// ── Legacy role → preset mapping (back-compat) ─────────────────────

#[test]
fn legacy_role_worker_maps_to_worker_preset() {
    assert_eq!(ShipServer::effective_preset(None, Some("worker")), "worker");
}

#[test]
fn legacy_role_lead_maps_to_orchestrator_preset() {
    assert_eq!(ShipServer::effective_preset(None, Some("lead")), "orchestrator");
}

#[test]
fn legacy_role_commander_maps_to_orchestrator_preset() {
    assert_eq!(ShipServer::effective_preset(None, Some("commander")), "orchestrator");
}

#[test]
fn no_role_and_no_preset_defaults_to_worker() {
    assert_eq!(ShipServer::effective_preset(None, None), "worker");
}

#[test]
fn tools_preset_takes_precedence_over_legacy_role() {
    assert_eq!(
        ShipServer::effective_preset(Some("orchestrator"), Some("worker")),
        "orchestrator",
        "explicit tools_preset must win over legacy role"
    );
}

// ── Dangerous tools never in any preset ────────────────────────────

#[test]
fn dangerous_tools_excluded_from_all_presets() {
    let dangerous = ["emit_event", "send_mesh_message", "broadcast_mesh_message"];
    for preset in &["worker", "orchestrator"] {
        let tools = ShipServer::tools_for_preset(preset);
        for tool in &dangerous {
            assert!(
                !tools.contains(tool),
                "{tool} must not be in {preset} preset"
            );
        }
    }
}

// ── Dispatched-context shortcut ────────────────────────────────────

#[test]
fn is_platform_tool_recognizes_step_done() {
    assert!(ShipServer::is_platform_tool("step_done"));
    assert!(ShipServer::is_platform_tool("log_progress"));
    assert!(ShipServer::is_platform_tool("update_job"));
}

#[test]
fn is_platform_tool_rejects_orchestration() {
    assert!(!ShipServer::is_platform_tool("update_mesh_status"));
    assert!(!ShipServer::is_platform_tool("discover_mesh_agents"));
}

// ── Token-scope gate (daemon-hosted path) ──────────────────────────

#[test]
fn sui_tools_require_sui_authoring_scope() {
    use ship_platform::auth::scopes::{McpAuth, Scope};

    assert_eq!(
        ShipServer::tool_required_scope("sui"),
        Some(Scope::SuiAuthoring)
    );
    assert_eq!(
        ShipServer::tool_required_scope("view_compile_check"),
        Some(Scope::SuiAuthoring)
    );
    // Default surface — no scope required.
    assert_eq!(ShipServer::tool_required_scope("log_progress"), None);
    assert_eq!(ShipServer::tool_required_scope("chat_reply"), None);

    let designer = McpAuth {
        agent_id: "d".into(),
        scopes: vec![Scope::SuiAuthoring],
        workspace_id: None,
    };
    let worker = McpAuth {
        agent_id: "w".into(),
        scopes: vec![Scope::ReadWorkspace],
        workspace_id: None,
    };
    // Designer token reaches the gated group; worker token does not.
    assert!(ShipServer::enforce_scope_gate(&designer, "sui").is_ok());
    assert!(ShipServer::enforce_scope_gate(&worker, "sui").is_err());
    // The default surface is open to every authenticated token.
    assert!(ShipServer::enforce_scope_gate(&worker, "log_progress").is_ok());
}

#[test]
fn http_path_requires_scopes_no_trusted_local_bypass() {
    use super::tool_gate::GateOutcome;
    use ship_platform::auth::scopes::{McpAuth, Scope};

    // An HTTP connection (`over_http = true`) with NO injected identity must
    // FAIL CLOSED — never fall through to the trusted-local active-agent gate.
    // This is the silent network scope-bypass guard.
    assert!(matches!(
        ShipServer::classify_gate(true, None, "log_progress"),
        GateOutcome::Deny(_)
    ));
    assert!(matches!(
        ShipServer::classify_gate(true, None, "sui"),
        GateOutcome::Deny(_)
    ));

    // HTTP + designer token → the SuiAuthoring group is allowed.
    let designer = McpAuth {
        agent_id: "d".into(),
        scopes: vec![Scope::SuiAuthoring],
        workspace_id: None,
    };
    assert_eq!(
        ShipServer::classify_gate(true, Some(&designer), "sui"),
        GateOutcome::Allow
    );

    // HTTP + worker token → gated tool denied by scope; default tool allowed.
    let worker = McpAuth {
        agent_id: "w".into(),
        scopes: vec![Scope::ReadWorkspace],
        workspace_id: None,
    };
    assert!(matches!(
        ShipServer::classify_gate(true, Some(&worker), "sui"),
        GateOutcome::Deny(_)
    ));
    assert_eq!(
        ShipServer::classify_gate(true, Some(&worker), "log_progress"),
        GateOutcome::Allow
    );
}

#[test]
fn trusted_local_fallback_is_stdio_only() {
    use super::tool_gate::GateOutcome;
    // Only a non-HTTP (stdio, same-machine) connection without an injected
    // identity defers to the active-agent CWD gate. HTTP never does.
    assert_eq!(
        ShipServer::classify_gate(false, None, "anything"),
        GateOutcome::TrustedLocalFallback
    );
    assert!(!matches!(
        ShipServer::classify_gate(true, None, "anything"),
        GateOutcome::TrustedLocalFallback
    ));
}

#[test]
fn dispatched_context_detects_filesystem_marker() {
    use std::fs;
    use tempfile::tempdir;
    let tmp = tempdir().expect("tempdir");
    let wt = tmp.path().join("ship-dev-testslug");
    fs::create_dir_all(wt.join(".artifacts")).unwrap();
    fs::write(wt.join(".artifacts/.dispatched"), "ship-dev-testslug").unwrap();

    let _env =
        ship_substrate::testutil::EnvGuard::unset(ship_substrate::env::Var::ActorId);
    let detected = ShipServer::is_dispatched_context_at(Some(&wt));
    assert!(
        detected,
        "dispatched context must be detected via .artifacts/.dispatched marker \
         even when SHIP_ACTOR_ID is unset (codex strips env to MCP subprocesses)",
    );
}

#[test]
fn dispatched_context_ignores_artifacts_without_marker() {
    use std::fs;
    use tempfile::tempdir;
    let tmp = tempdir().expect("tempdir");
    let wt = tmp.path().join("not-a-dispatch");
    fs::create_dir_all(wt.join(".artifacts")).unwrap();
    // `.artifacts/` exists with random contents but NO `.dispatched` marker.
    fs::write(wt.join(".artifacts/spec.md"), "random spec").unwrap();
    fs::write(wt.join(".artifacts/job-spec.md"), "old job spec").unwrap();

    let _env =
        ship_substrate::testutil::EnvGuard::unset(ship_substrate::env::Var::ActorId);
    let detected = ShipServer::is_dispatched_context_at(Some(&wt));
    assert!(
        !detected,
        "presence of .artifacts/ alone must not trigger dispatched mode — \
         only the .dispatched marker counts",
    );
}

// ── emit_event gating (unit) ───────────────────────────────────────
//
// emit_event is registered on every binary now (no compile-time gate). It is
// classified as core (bypasses agent-mode gating) but is never folded into a
// preset — clients reach it explicitly.

#[test]
fn emit_event_is_core_so_it_bypasses_mode_gate() {
    assert!(
        ShipServer::is_core_tool("emit_event"),
        "emit_event should be core (bypasses mode gating)"
    );
}

#[test]
fn emit_event_not_in_any_preset() {
    for preset in &["worker", "orchestrator"] {
        let tools = ShipServer::tools_for_preset(preset);
        assert!(
            !tools.contains(&"emit_event"),
            "emit_event must not be in {preset} preset"
        );
    }
}

// ── core tools bypass gating ───────────────────────────────────────

#[test]
fn core_tools_bypass_gating() {
    let core = ["set_skill_var", "chat_reply", "sdk_query"];
    for tool in core {
        assert!(
            ShipServer::is_core_tool(tool),
            "{tool} should be a core tool (bypass gating)"
        );
    }
}

#[test]
fn deprecated_tools_are_no_longer_core() {
    let removed = [
        "open_project",
        "activate_workspace",
        "list_workspaces",
        "set_agent",
        "start_session",
        "end_session",
        "get_session",
        "list_sessions",
        "list_skills",
        "get_skill_vars",
        "list_skill_vars",
        "list_jobs",
        "get_job",
        "list_stale_worktrees",
        "read_session_file",
        "list_session_files",
    ];
    for tool in removed {
        assert!(
            !ShipServer::is_core_tool(tool),
            "{tool} is deprecated and must not be a core tool"
        );
    }
}

// ── Preset + core effective tool set sizes ─────────────────────────

#[test]
fn worker_effective_tool_set_count() {
    let core: Vec<&str> = ShipServer::CORE_TOOLS.to_vec();
    let preset = ShipServer::tools_for_preset("worker");
    let mut all: Vec<&str> = core;
    all.extend(preset.iter().copied());
    all.sort_unstable();
    all.dedup();
    // 3 core (set_skill_var, chat_reply, sdk_query) + 3 protocol (step_done,
    // log_progress, update_job) + 5 view canvas (open/patch/append/signal/
    // close_view) = 11. CORE_TOOLS was pruned (view_lint/view_scaffold retired,
    // view_errors moved to the SuiAuthoring group).
    let expected = 11;
    assert_eq!(all.len(), expected, "worker effective set mismatch: {all:?}");
}

#[test]
fn orchestrator_effective_tool_set_count() {
    let core: Vec<&str> = ShipServer::CORE_TOOLS.to_vec();
    let preset = ShipServer::tools_for_preset("orchestrator");
    let mut all: Vec<&str> = core;
    all.extend(preset.iter().copied());
    all.sort_unstable();
    all.dedup();
    // Mesh observe/status tools left MCP for the CLI, so ORCHESTRATION_TOOLS
    // is empty and the orchestrator set equals the worker set (11).
    let expected = 11;
    assert_eq!(all.len(), expected, "orchestrator effective set mismatch: {all:?}");
}
