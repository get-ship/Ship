pub mod agents;
pub mod harness;

mod claude;
mod codex;
mod context;
mod mcp;
mod opencode;
mod pi;

mod plugins;
mod skills;

#[cfg(test)]
mod tests;

// Per-harness settings types live in their harness modules (cohesion);
// re-export so the compile/tests fixtures can build `ResolvedConfig` values
// via `crate::compile::{Claude,Codex,OpenCode}Settings`.
#[cfg(test)]
pub(crate) use claude::ClaudeSettings;
#[cfg(test)]
pub(crate) use codex::CodexSettings;
#[cfg(test)]
pub(crate) use opencode::OpenCodeSettings;

use stack_model::resolve::ResolvedConfig;

// ─── Re-exports for lib.rs public surface ─────────────────────────────────────

pub use claude::build_claude_settings_patch;
pub use harness::{
    AgentsDir, ContextFile, HarnessDescriptor, McpKey, ProviderFeatureFlags, SkillsDir,
    get_harness, list_harnesses,
};

// ─── Shared hook command ──────────────────────────────────────────────────────

/// A lifecycle hook command that works on the host *and* inside a sandbox,
/// from one compiled config. The compiler can't know at compile time
/// whether an agent will run sandboxed (the isolation tier is resolved
/// per-spawn), so the command branches at run time on `$SHIP_DAEMON_URL`:
///
/// - **In a sandbox** the supervisor injects `$SHIP_DAEMON_URL` (rewritten
///   to a container-reachable host) + a scoped `$SHIP_TOKEN`; the hook
///   `curl`s its harness-native payload straight at the daemon — no `ship`
///   CLI in the container. The daemon derives identity from the token and
///   deserializes the raw payload via `ShipHookEvent`'s field aliases.
/// - **On the host** `$SHIP_DAEMON_URL` is unset → the hook is a NO-OP
///   (`true`). No `hook` subcommand exists in ship-cli; a failing lifecycle
///   hook blocks the harness's turn-end and death-spirals the agent
///   (operator ruling 2026-07-06: emit nothing that can fail until the
///   hooks mechanism is redesigned).
///
/// Shared by every harness that emits command-style lifecycle hooks
/// (claude `.mcp`/settings, codex `hooks.json`). Note: `before-tool`
/// *blocking* is observability-only in the curl branch (raw curl can't
/// translate the daemon's allow/block into a harness hook-block protocol).
pub(super) fn ship_hook_command(endpoint: &str) -> String {
    format!(
        "if [ -n \"$SHIP_DAEMON_URL\" ]; then \
curl -sS -X POST \"$SHIP_DAEMON_URL/hooks/{endpoint}\" \
-H \"Authorization: Bearer $SHIP_TOKEN\" -H \"Content-Type: application/json\" \
--data-binary @- >/dev/null 2>&1; \
else true; fi"
    )
}

// ─── Output ───────────────────────────────────────────────────────────────────

pub use stack_model::CompileOutput;

// ─── Main entry point ─────────────────────────────────────────────────────────

/// Compile a resolved agent config into harness-ready content.
/// Returns `None` if the harness ID is not recognised.
pub fn compile(resolved: &ResolvedConfig, harness_id: &str) -> Option<CompileOutput> {
    let desc = get_harness(harness_id)?;
    let flags = desc.feature_flags();
    let mut out = CompileOutput::default();

    // Only emit MCP config for harnesses that support it.
    if flags.supports_mcp {
        // MCP builder is carried on the descriptor; all current harnesses
        // share mcp::build_mcp_servers.
        out.mcp_servers = (desc.build_mcp)(desc, &resolved.mcp_servers);
        out.mcp_config_path = desc.mcp_config_path.map(String::from);
    } else {
        out.mcp_servers = serde_json::json!({});
    }
    out.context_content = context::build_context_content(desc, resolved);
    out.skill_files = skills::build_skill_files(desc, resolved);
    out.event_subscriptions = skills::resolve_event_subscriptions(&resolved.skills);

    // Compile agent profiles for this harness
    let profile_agents = agents::compile_agent_profiles(&resolved.agent_profiles, harness_id);
    out.agent_files.extend(profile_agents);

    // Harness-native settings/policy/hook outputs — dispatched via the
    // descriptor's fn pointer, no harness-id branch. Runs after agent
    // profiles so claude's team-agent passthrough lands in out.agent_files.
    (desc.build_settings)(desc, resolved, &mut out);

    out.plugins_manifest = plugins::build_plugins_manifest(&resolved.plugins, harness_id);

    Some(out)
}
