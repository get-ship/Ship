use std::collections::HashMap;

use serde_json::Value as Json;

use crate::types::{HookConfig, HookTrigger, Permissions};

/// Build the `.claude/settings.json` patch from permissions, hooks, model, and harness settings.
///
/// Always returns `Some` — Ship unconditionally sets `autoMemoryEnabled: false`
/// so a settings patch is always written. Callers must always write the returned value.
#[allow(clippy::too_many_arguments)]
pub fn build_claude_settings_patch(
    permissions: &Permissions,
    hooks: &[HookConfig],
    model: Option<&str>,
    extra: Option<&Json>,
    env: &std::collections::HashMap<String, String>,
    available_models: &[String],
    theme: Option<&str>,
    auto_updates: Option<bool>,
    include_co_authored_by: Option<bool>,
) -> Option<Json> {
    let has_perms = has_permission_overrides(permissions);
    let has_hooks = !hooks.is_empty();
    let has_env = !env.is_empty();
    let has_available_models = !available_models.is_empty();

    let mut patch = serde_json::json!({});

    // Tool permissions — only emit when the user has deliberately configured them.
    // Claude Code interprets an explicit `allow` list as a strict allowlist, so we
    // only write it when the user has moved away from the "allow all" default.
    if has_perms {
        let mut perms = serde_json::json!({});
        // Only include allow if the user has a non-default allowlist.
        // Default ("*" or empty) → omit → Claude Code uses its own defaults.
        let non_default_allow = !(permissions.tools.allow.is_empty()
            || permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*");
        if non_default_allow {
            perms["allow"] = serde_json::json!(permissions.tools.allow);
        }
        if !permissions.tools.ask.is_empty() {
            perms["ask"] = serde_json::json!(permissions.tools.ask);
        }
        if !permissions.tools.deny.is_empty() {
            perms["deny"] = serde_json::json!(permissions.tools.deny);
        }
        if let Some(ref mode) = permissions.default_mode {
            perms["defaultMode"] = serde_json::json!(mode);
        }
        if !permissions.additional_directories.is_empty() {
            perms["additionalDirectories"] = serde_json::json!(permissions.additional_directories);
        }
        patch["permissions"] = perms;
    } else {
        // Even without tool-level overrides, emit permissions block if defaultMode or additionalDirectories set
        let mut perms = serde_json::json!({});
        let mut has_extra_perms = false;
        if let Some(ref mode) = permissions.default_mode {
            perms["defaultMode"] = serde_json::json!(mode);
            has_extra_perms = true;
        }
        if !permissions.additional_directories.is_empty() {
            perms["additionalDirectories"] = serde_json::json!(permissions.additional_directories);
            has_extra_perms = true;
        }
        if has_extra_perms {
            patch["permissions"] = perms;
        }
    }

    // Hooks — grouped by trigger type, matching Claude Code's expected structure.
    if has_hooks {
        let mut by_trigger: HashMap<&str, Vec<Json>> = HashMap::new();
        for hook in hooks {
            let key = match hook.trigger {
                HookTrigger::PreToolUse => "PreToolUse",
                HookTrigger::PostToolUse => "PostToolUse",
                HookTrigger::Notification => "Notification",
                HookTrigger::Stop => "Stop",
                HookTrigger::SubagentStop => "SubagentStop",
                HookTrigger::PreCompact => "PreCompact",
            };
            let hook_obj = serde_json::json!({ "type": "command", "command": hook.command });
            let mut entry = serde_json::json!({ "hooks": [hook_obj] });
            if let Some(m) = &hook.matcher {
                entry["matcher"] = Json::String(m.clone());
            }
            by_trigger.entry(key).or_default().push(entry);
        }
        patch["hooks"] = serde_json::json!(by_trigger);
    }

    // Agent limits.
    if let Some(cost) = permissions.agent.max_cost_per_session {
        patch["maxCostPerSession"] = serde_json::json!(cost);
    }
    if let Some(turns) = permissions.agent.max_turns {
        patch["maxTurns"] = serde_json::json!(turns);
    }

    // Model override.
    if let Some(m) = model {
        patch["model"] = serde_json::json!(m);
    }

    // Environment variables — KEY=VALUE pairs in Claude's `env` field.
    if has_env {
        let env_obj: serde_json::Map<String, Json> = env
            .iter()
            .map(|(k, v)| (k.clone(), Json::String(v.clone())))
            .collect();
        patch["env"] = Json::Object(env_obj);
    }

    // Model allowlist — restricts the model picker in Claude Code.
    if has_available_models {
        patch["availableModels"] = serde_json::json!(available_models);
    }

    // Theme.
    if let Some(t) = theme {
        patch["theme"] = Json::String(t.to_string());
    }

    // Auto-updates.
    if let Some(au) = auto_updates {
        patch["autoUpdates"] = serde_json::json!(au);
    }

    // Include co-authored-by.
    if let Some(co) = include_co_authored_by {
        patch["includeCoAuthoredBy"] = serde_json::json!(co);
    }

    // Ship is the memory layer. Always disable Claude's built-in memories.
    patch["autoMemoryEnabled"] = serde_json::json!(false);

    // Extra harness-specific settings — pass through verbatim (must be last).
    // Source: `[harness_settings.claude]` in the active preset TOML.
    if let Some(extra_obj) = extra.and_then(|v| v.as_object()) {
        for (k, v) in extra_obj {
            patch[k] = v.clone();
        }
    }

    Some(patch)
}

use super::ship_hook_command;

/// Inject Ship lifecycle hooks for the daemon into Claude's hooks patch.
///
/// Emits hybrid command hooks (see `ship_hook_command`) that POST to the
/// daemon over HTTP inside a sandbox and fall back to the `ship hook` CLI
/// on the host — one config, stable across daemon restarts and tiers.
pub fn inject_ship_lifecycle_hooks(patch: &mut Json) {
    let Some(obj) = patch.as_object_mut() else {
        return;
    };
    let hooks_obj = obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    // Claude Code hook events used by Ship's lifecycle wiring:
    // - SessionStart: fires when Claude opens a session. Drives the
    //   hook.session_started kernel event for analytics. UNLIKE the
    //   per-event hooks (PreToolUse / PostToolUse / Stop), SessionStart
    //   requires an explicit `matcher` naming the source (`startup`,
    //   `resume`, `clear`, `compact`). Without a matcher the hook
    //   silently fails to register and never fires. We register one
    //   entry per source so any kind of session boot reaches the
    //   daemon. Source: claude --debug hooks emits "no matching hook"
    //   when matcher is absent.
    // - PreToolUse / PostToolUse: per-tool bracketing. Drives the
    //   hook.tool_use_* kernel events. No matcher needed (default = all).
    // - Stop: fires on EVERY turn end (not actual session end —
    //   Claude's Stop semantics). Wired to `ship hook session-end`
    //   for historical reasons; the per-turn semantics are documented
    //   in hooks_api.rs::hook_session_end. A future job will split
    //   turn-end from session-end into separate Ship signals.
    let lifecycle = [
        ("PreToolUse", "before-tool"),
        ("PostToolUse", "after-tool"),
        ("Stop", "session-end"),
    ];

    for (trigger, endpoint) in lifecycle {
        let command = ship_hook_command(endpoint);
        let hook_obj = serde_json::json!({ "type": "command", "command": command });
        let entry = serde_json::json!({ "hooks": [hook_obj] });
        if let Some(arr) = hooks_obj.get_mut(trigger).and_then(|v| v.as_array_mut()) {
            arr.push(entry);
        } else {
            hooks_obj[trigger] = serde_json::json!([entry]);
        }
    }

    // SessionStart needs one entry per source matcher to register.
    let session_start_cmd = ship_hook_command("session-start");
    let session_start_sources = ["startup", "resume", "clear", "compact"];
    let session_start_entries: Vec<Json> = session_start_sources
        .iter()
        .map(|src| {
            serde_json::json!({
                "matcher": src,
                "hooks": [{ "type": "command", "command": session_start_cmd }],
            })
        })
        .collect();
    if let Some(arr) = hooks_obj.get_mut("SessionStart").and_then(|v| v.as_array_mut()) {
        arr.extend(session_start_entries);
    } else {
        hooks_obj["SessionStart"] = serde_json::json!(session_start_entries);
    }
}

/// Claude Code LSP plugin manifest. Written to `.claude/lsp.json`.
///
/// Manifest shape: https://code.claude.com/docs/en/plugins-reference#lsp-servers
pub fn build_claude_lsp_config() -> Json {
    serde_json::json!({
        "ship": {
            "command": "ship",
            "args": ["lsp"],
            "extensionToLanguage": {
                ".sui": "sui",
                ".controller.ts": "typescript-sui",
                ".html.j2": "jinja-ship",
                ".ship": "jinja-ship"
            }
        }
    })
}

/// Returns `true` when the permissions object contains any tool-level overrides
/// that deviate from "allow everything" defaults. Filesystem, command, network,
/// and agent limits are checked separately in the caller.
pub(super) fn has_permission_overrides(p: &Permissions) -> bool {
    let allow_is_default =
        p.tools.allow.is_empty() || (p.tools.allow.len() == 1 && p.tools.allow[0] == "*");
    !allow_is_default
        || !p.tools.ask.is_empty()
        || !p.tools.deny.is_empty()
        || p.default_mode.is_some()
        || !p.additional_directories.is_empty()
}

/// Populate Claude-native outputs (settings patch + lifecycle hooks, LSP
/// config, legacy team agents). Invoked via the descriptor's build_settings
/// fn pointer — no harness-id branch in the compile flow.
pub(super) fn build_settings(
    desc: &super::harness::HarnessDescriptor,
    resolved: &crate::resolve::ResolvedConfig,
    out: &mut super::CompileOutput,
) {
    out.claude_settings_patch = build_claude_settings_patch(
        &resolved.permissions,
        &resolved.hooks,
        resolved.model.as_deref(),
        resolved.claude.settings_extra.as_ref(),
        &resolved.env,
        &resolved.available_models,
        resolved.claude.theme.as_deref(),
        resolved.claude.auto_updates,
        resolved.claude.include_co_authored_by,
    );
    if let Some(ref mut patch) = out.claude_settings_patch {
        inject_ship_lifecycle_hooks(patch);
    }
    out.claude_lsp_config = Some(build_claude_lsp_config());
    if let Some(base) = desc.agents_dir.base_path() {
        for (filename, content) in &resolved.claude.team_agents {
            out.agent_files
                .insert(format!("{base}/{filename}"), content.clone());
        }
    }
}

/// Claude's typed settings delta from the Ship agent.
#[derive(Debug, Clone, Default)]
pub struct ClaudeSettings {
    pub settings_extra: Option<serde_json::Value>,
    /// Legacy team-agent passthrough: `.ship/agents/teams/claude/<n>.md`.
    pub team_agents: Vec<(String, String)>,
    pub theme: Option<String>,
    pub auto_updates: Option<bool>,
    pub include_co_authored_by: Option<bool>,
}
