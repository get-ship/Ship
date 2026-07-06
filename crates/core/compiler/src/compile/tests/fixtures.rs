use crate::resolve::ResolvedConfig;
use crate::types::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions, Rule, Skill,
};

/// Every registered harness id, sourced from the compiler registry — for tests
/// that assert a property across the whole roster. Add/drop a harness and these
/// loops update with no edits.
#[allow(dead_code)]
pub fn all_harness_ids() -> Vec<&'static str> {
    crate::compile::list_harnesses()
        .iter()
        .map(|d| d.id)
        .collect()
}

/// Every harness except `owner` — for "owned output must not leak to other
/// harnesses" checks. Registry-driven, so a newly registered harness is
/// automatically held to the no-leak contract (the old hardcoded subsets
/// silently skipped harnesses added after they were written).
pub fn others(owner: &str) -> Vec<&'static str> {
    all_harness_ids()
        .into_iter()
        .filter(|id| *id != owner)
        .collect()
}

pub fn make_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: "npx".to_string(),
        args: vec!["-y".to_string(), format!("@mcp/{}", id)],
        env: Default::default(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
        codex_enabled_tools: vec![],
        codex_disabled_tools: vec![],
        cursor_env_file: None,
    }
}

pub fn make_skill(id: &str) -> Skill {
    Skill {
        id: id.to_string(),
        name: id.to_string(),
        stable_id: None,
        description: Some(format!("{} skill", id)),
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: format!("# {}\n\nDo the thing.", id),
        source: Default::default(),
        vars: Default::default(),
        artifacts: vec![],
        grants: vec![],
        requires: vec![],
    }
}

pub fn make_hook(trigger: HookTrigger, command: &str, matcher: Option<&str>) -> HookConfig {
    HookConfig {
        id: "test-hook".to_string(),
        trigger,
        matcher: matcher.map(str::to_string),
        command: command.to_string(),
        cursor_event: None,
    }
}

pub fn make_rule(file_name: &str, content: &str) -> Rule {
    Rule {
        file_name: file_name.to_string(),
        content: content.to_string(),
        always_apply: true,
        globs: vec![],
        description: None,
    }
}

pub fn resolved(servers: Vec<McpServerConfig>) -> ResolvedConfig {
    ResolvedConfig {
        harnesses: vec!["claude".to_string()],
        model: None,
        max_cost_per_session: None,
        max_turns: None,
        mcp_servers: servers,
        skills: vec![],
        rules: vec![],
        permissions: Permissions::default(),
        hooks: vec![],
        active_agent: None,
        plugins: Default::default(),
        agent_profiles: vec![],
        env: Default::default(),
        available_models: vec![],
        claude: Default::default(),
        codex: Default::default(),
        opencode: Default::default(),
        requested_grants: vec![],
        unmet_capability_requirements: vec![],
        ship_home_dir: None,
        scoped_launch: false,
        imported_modules: vec![],
        rollup_inputs: None,
    }
}
