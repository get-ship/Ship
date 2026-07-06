//! Decompile — reverse-compile harness-native configs into a [`ExtensionLibrary`].
//!
//! [`ExtensionLibrary`]. The caller merges results from multiple harnesses.

mod claude;
mod claude_mcp;
mod codex;
mod opencode;
mod opencode_agents;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "opencode_tests.rs"]
mod opencode_tests;

#[cfg(test)]
#[path = "roundtrip_tests.rs"]
mod roundtrip_tests;

#[cfg(test)]
pub(crate) mod test_helpers {
    // write_file moved here from three byte-identical copies (audit R-6c).
    use std::io::Write;
    use tempfile::TempDir;

    pub(crate) fn write_file(dir: &TempDir, path: &str, content: &str) {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(&full).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }
}

use std::path::Path;

use serde_json::Value as Json;

use crate::ExtensionLibrary;

/// Extract a JSON array of strings from an optional JSON value.
/// Shared by multiple decompile modules.
pub(super) fn json_string_array(val: Option<&Json>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub use claude::decompile_claude;
pub use codex::decompile_codex;
pub use opencode::decompile_opencode;

/// Which harnesses were detected in a project directory.
#[derive(Debug, Default)]
pub struct DetectedProviders {
    pub claude: bool,
    pub codex: bool,
    pub opencode: bool,
}

impl DetectedProviders {
    pub fn any(&self) -> bool {
        self.claude || self.codex || self.opencode
    }

    pub fn as_list(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.claude {
            v.push("claude");
        }
        if self.codex {
            v.push("codex");
        }
        if self.opencode {
            v.push("opencode");
        }
        v
    }
}

/// Detect which harness configs exist in a space directory.
pub fn detect_harnesses(space_root: &Path) -> DetectedProviders {
    DetectedProviders {
        claude: space_root.join(".claude").is_dir()
            || space_root.join("CLAUDE.md").exists()
            || space_root.join(".mcp.json").exists(),
        codex: space_root.join(".codex").is_dir() || space_root.join("AGENTS.md").exists(),
        opencode: space_root.join(".opencode").is_dir()
            || space_root.join("opencode.json").exists(),
    }
}

/// Decompile all detected harnesses in a directory into a single [`ExtensionLibrary`].
///
/// MCP servers, rules, permissions, and hooks are merged. Harness-specific
/// settings go into `harness_defaults`. The caller should write the result
/// to `.ship/`.
pub fn decompile_all(space_root: &Path) -> ExtensionLibrary {
    let detected = detect_harnesses(space_root);
    let mut library = ExtensionLibrary::default();

    if detected.claude {
        let partial = decompile_claude(space_root);
        merge_into(&mut library, partial, "claude");
    }

    if detected.codex {
        let partial = decompile_codex(space_root);
        merge_into(&mut library, partial, "codex");
    }

    if detected.opencode {
        let partial = decompile_opencode(space_root);
        merge_into(&mut library, partial, "opencode");
    }

    library
}

/// Merge a harness-specific partial library into the accumulator.
fn merge_into(target: &mut ExtensionLibrary, source: ExtensionLibrary, harness_id: &str) {
    // MCP servers — deduplicate by id
    for server in source.mcp_servers {
        if !target.mcp_servers.iter().any(|s| s.id == server.id) {
            target.mcp_servers.push(server);
        }
    }

    // Rules — append (no dedup; different harnesses may have different context files)
    target.rules.extend(source.rules);

    // Permissions — merge tool lists (union)
    merge_permissions(&mut target.permissions, &source.permissions);

    // Hooks — append, dedup by id
    for hook in source.hooks {
        if !target.hooks.iter().any(|h| h.id == hook.id) {
            target.hooks.push(hook);
        }
    }

    // Agent profiles — append, dedup by id
    for profile in source.agent_profiles {
        if !target
            .agent_profiles
            .iter()
            .any(|p| p.agent.id == profile.agent.id)
        {
            target.agent_profiles.push(profile);
        }
    }

    // Env — merge (source wins on conflict)
    target.env.extend(source.env);

    // Harness defaults — merge in
    for (k, v) in source.harness_defaults {
        target.harness_defaults.insert(k, v);
    }

    // Scalar fields — take from source if set
    if source.model.is_some() {
        target.model = source.model;
    }
    if !source.available_models.is_empty() {
        target.available_models = source.available_models;
    }

    // Claude-specific fields
    if harness_id == "claude" {
        target.claude_settings_extra = source.claude_settings_extra;
        target.claude_theme = source.claude_theme;
        target.claude_auto_updates = source.claude_auto_updates;
        target.claude_include_co_authored_by = source.claude_include_co_authored_by;
        target.claude_team_agents = source.claude_team_agents;
    }

    // Codex-specific fields
    if harness_id == "codex" {
        target.codex_settings_extra = source.codex_settings_extra;
        target.codex_sandbox = source.codex_sandbox;
        target.codex_approval_policy = source.codex_approval_policy;
        target.codex_reasoning_effort = source.codex_reasoning_effort;
        target.codex_max_threads = source.codex_max_threads;
        target.codex_max_depth = source.codex_max_depth;
        target.codex_job_max_runtime_seconds = source.codex_job_max_runtime_seconds;
        target.codex_shell_env_policy = source.codex_shell_env_policy;
        target.codex_notify = source.codex_notify;
    }

}

/// Merge source permissions into target. Appends tool lists rather than replacing.
fn merge_permissions(target: &mut crate::types::Permissions, source: &crate::types::Permissions) {
    // Tool lists — append unique entries
    for p in &source.tools.allow {
        if !target.tools.allow.contains(p) {
            target.tools.allow.push(p.clone());
        }
    }
    for p in &source.tools.deny {
        if !target.tools.deny.contains(p) {
            target.tools.deny.push(p.clone());
        }
    }
    for p in &source.tools.ask {
        if !target.tools.ask.contains(p) {
            target.tools.ask.push(p.clone());
        }
    }
    for d in &source.additional_directories {
        if !target.additional_directories.contains(d) {
            target.additional_directories.push(d.clone());
        }
    }
    if target.default_mode.is_none() {
        target.default_mode = source.default_mode.clone();
    }
    if target.agent.max_cost_per_session.is_none() {
        target.agent.max_cost_per_session = source.agent.max_cost_per_session;
    }
    if target.agent.max_turns.is_none() {
        target.agent.max_turns = source.agent.max_turns;
    }
}
