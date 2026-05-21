use anyhow::Result;
use ship_platform::auth::scopes::{McpAuth, Scope};
use ship_platform::config::get_active_agent;
use std::path::Path;
use std::process::Command as ProcessCommand;

use super::ShipServer;

/// Outcome of [`ShipServer::classify_gate`].
#[derive(Debug, PartialEq, Eq)]
pub enum GateOutcome {
    /// Tool call permitted.
    Allow,
    /// Tool call refused, with a caller-facing reason.
    Deny(String),
    /// No injected identity on a local (stdio) connection — defer to the
    /// active-agent CWD gate. Never produced for an HTTP connection.
    TrustedLocalFallback,
}

impl ShipServer {
    /// Tools gated to the `SuiAuthoring` capability — the one designer
    /// tool-group. Keyed on canonical (post-normalize) names. Every other
    /// tool is in the default surface, visible to all agents. This list is
    /// the single source of truth for the token-scope gate on the
    /// daemon-hosted path.
    pub const SUI_AUTHORING_TOOLS: &[&str] = &[
        "view_compile_check",
        "sui",
        "get_view_errors",
        "open_view",
        "patch_view",
        "append_view",
        "signal_view",
        "close_view",
        "view_annotate",
        "view_write_source",
        "view_restore_snapshot",
        "view_collections_schema",
        "view_list_siblings",
        "view_theme_tokens",
        "view_preview_snapshot",
        "view_list_snapshots",
    ];

    /// The capability scope a tool requires, or `None` for the default
    /// surface (no scope needed — every authenticated agent may call it).
    pub fn tool_required_scope(tool_name: &str) -> Option<Scope> {
        let normalized = Self::normalize_mode_tool_id(tool_name);
        let canonical = crate::tools::stability::canonical_name(&normalized);
        if Self::SUI_AUTHORING_TOOLS.contains(&canonical) {
            Some(Scope::SuiAuthoring)
        } else {
            None
        }
    }

    /// Token-scope gate (daemon-hosted path). Authoritative when the
    /// connection carries `McpAuth`: a tool is allowed iff its required
    /// scope is granted by the session token. Default-surface tools (no
    /// required scope) are always allowed.
    pub fn enforce_scope_gate(auth: &McpAuth, tool_name: &str) -> Result<(), String> {
        match Self::tool_required_scope(tool_name) {
            Some(scope) if !auth.allows(scope) => Err(format!(
                "Tool '{tool_name}' requires the {scope:?} capability, which this \
                 session's token does not grant."
            )),
            _ => Ok(()),
        }
    }

    /// Resolve how a tool call should be gated, given the connection's
    /// transport and any injected identity.
    ///
    /// `over_http` is true on the daemon-hosted `/mcp` mount (rmcp injects
    /// the HTTP request `Parts` into the tool context). The stdio shim runs
    /// with no request parts → `false`.
    ///
    /// Security invariant: **the trusted-local fallback is local-only.** Over
    /// HTTP, an injected `McpAuth` (token scopes) is mandatory — a missing one
    /// fails closed (`Deny`), never falling through to the CWD active-agent
    /// gate. That CWD gate trusts the local filesystem's active-agent state,
    /// which is meaningless (and bypassable) for a network caller. Only a
    /// non-HTTP (stdio, same machine) connection may use it.
    pub fn classify_gate(
        over_http: bool,
        auth: Option<&McpAuth>,
        tool_name: &str,
    ) -> GateOutcome {
        match (over_http, auth) {
            // Authenticated (either transport): token scopes are authoritative.
            (_, Some(a)) => match Self::enforce_scope_gate(a, tool_name) {
                Ok(()) => GateOutcome::Allow,
                Err(e) => GateOutcome::Deny(e),
            },
            // HTTP without injected identity → fail closed. auth_middleware +
            // `inject_mcp_auth` should always populate it; if they didn't, deny
            // rather than trust local state over the network.
            (true, None) => GateOutcome::Deny(format!(
                "Tool '{tool_name}' denied: the daemon-hosted MCP path requires an \
                 authenticated session token; no scopes were presented."
            )),
            // Local stdio shim: defer to the active-agent (CWD) gate.
            (false, None) => GateOutcome::TrustedLocalFallback,
        }
    }
    pub fn normalize_mode_tool_id(raw: &str) -> String {
        let mut normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
        if let Some(stripped) = normalized.strip_prefix("ship_") {
            normalized = stripped.to_string();
        }
        if let Some(stripped) = normalized.strip_suffix("_tool") {
            normalized = stripped.to_string();
        }
        normalized
    }

    /// Core platform tools that always bypass gating.
    /// Only write operations that all roles need — reads are resources now.
    pub const CORE_TOOLS: &[&str] = &[
        "set_skill_var",
        // The chat protocol — every agent spawned into a Ship command-bar
        // session needs `chat_reply` to address the user, independent of
        // role/preset. Without it visible, the agent has no path back to
        // the UI bubble and falls through to stdout (which the parser
        // catches but loses structure).
        "chat_reply",
        // `query_sdk` is a harmless read-only SDK search every agent may use;
        // keep it ungated. (View-authoring tools moved to the SuiAuthoring
        // capability group; `view_lint`/`view_scaffold` were retired —
        // scaffolding is a skill now, linting is `sui`.)
        "sdk_query",
    ];

    /// Experimental tools that are always callable but advertised as
    /// experimental in their description. Listed here so dispatched/agent-
    /// gated contexts treat them as "core platform" tools (the experimental
    /// marker is informational, not a gating mechanism).
    const EXPERIMENTAL_CORE_TOOLS: &[&str] = &["emit_event"];

    // ── Tool tiers ──────────────────────────────────────────────────
    //
    // Two tiers, composed by preset. Replaces the old worker/lead/
    // commander roles (which conflated platform protocol with
    // orchestration capability).

    /// Platform protocol tools — every dispatched agent needs these to
    /// participate in the pipeline protocol. Not user-facing.
    pub const PLATFORM_TOOLS: &[&str] = &[
        "log_progress",
        "update_job",
        // Pipeline step-completion signal. Without step_done, pipelines
        // fall back to process-exit / activity-timeout signals.
        "step_done",
        // Canvas/view tools (experimental but registered everywhere).
        "open_view",
        "patch_view",
        "append_view",
        "signal_view",
        "close_view",
    ];

    /// Orchestration tools — historically the mesh observe/status tools.
    /// Those are no longer agent MCP tools (mesh probing moved to the CLI),
    /// so the orchestrator preset currently adds nothing over the worker set.
    /// Kept as the extension point for future orchestration-only tools.
    pub const ORCHESTRATION_TOOLS: &[&str] = &[];

    /// Resolve a preset name to its tool set. Unknown presets fall back
    /// to "worker" (platform tools only — the safe default).
    pub fn tools_for_preset(preset: &str) -> Vec<&'static str> {
        match preset {
            "orchestrator" => {
                let mut tools: Vec<&'static str> = Self::PLATFORM_TOOLS.to_vec();
                tools.extend_from_slice(Self::ORCHESTRATION_TOOLS);
                tools
            }
            _ => Self::PLATFORM_TOOLS.to_vec(),
        }
    }

    /// Resolve the effective preset for an agent profile:
    ///   1. Explicit `tools_preset` wins.
    ///   2. Otherwise map legacy `role`: worker→worker, lead|commander→orchestrator.
    ///   3. Otherwise "worker".
    pub fn effective_preset(tools_preset: Option<&str>, role: Option<&str>) -> &'static str {
        if let Some(p) = tools_preset {
            return match p {
                "orchestrator" => "orchestrator",
                _ => "worker",
            };
        }
        match role {
            Some("lead") | Some("commander") => "orchestrator",
            _ => "worker",
        }
    }

    /// True when this MCP server is serving a dispatched agent.
    ///
    /// Detection order:
    ///   1. SHIP_ACTOR_ID env var set (works when parent propagates env).
    ///   2. Current dir or any ancestor contains a `.artifacts/.dispatched`
    ///      marker — written by the isolation adapter on worktree prep.
    ///      Survives harnesses like codex that don't propagate env.
    ///
    /// Dispatched agents always get PLATFORM_TOOLS regardless of
    /// KV-stored active_agent state — dispatch context is the truth.
    pub fn is_dispatched_context() -> bool {
        Self::is_dispatched_context_at(std::env::current_dir().ok().as_deref())
    }

    /// Same as [`is_dispatched_context`], but resolves the worktree marker
    /// against an explicit starting directory instead of process cwd.
    /// Tests use this to avoid mutating the global cwd; production code
    /// passes `std::env::current_dir().ok()` via [`is_dispatched_context`].
    pub fn is_dispatched_context_at(start: Option<&std::path::Path>) -> bool {
        if ship_substrate::env::read_raw(ship_substrate::env::Var::ActorId)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
        {
            return true;
        }
        match start {
            Some(p) => Self::path_is_dispatched_worktree(p),
            None => false,
        }
    }

    /// Walk up from `start` looking for the explicit `.artifacts/.dispatched`
    /// marker file. Stops at the first match or at the filesystem root.
    /// The marker is only written by the isolation adapter during job
    /// dispatch, so the project root won't match even if it has
    /// `.artifacts/` artifacts from prior runs.
    pub(crate) const DISPATCH_MARKER: &str = ".artifacts/.dispatched";

    fn path_is_dispatched_worktree(start: &std::path::Path) -> bool {
        let mut cur = start.to_path_buf();
        loop {
            if cur.join(Self::DISPATCH_MARKER).is_file() {
                return true;
            }
            if !cur.pop() {
                return false;
            }
        }
    }

    /// True when `tool_name` is a platform-protocol tool (visible to every
    /// dispatched agent).
    pub fn is_platform_tool(tool_name: &str) -> bool {
        let n = Self::normalize_mode_tool_id(tool_name);
        let canonical = crate::tools::stability::canonical_name(&n);
        Self::PLATFORM_TOOLS.contains(&canonical)
    }

    pub fn is_core_tool(tool_name: &str) -> bool {
        let normalized = Self::normalize_mode_tool_id(tool_name);
        let canonical = crate::tools::stability::canonical_name(&normalized);
        if Self::CORE_TOOLS.contains(&canonical) {
            return true;
        }
        if Self::EXPERIMENTAL_CORE_TOOLS.contains(&canonical) {
            return true;
        }
        false
    }

    pub fn is_project_workspace_tool(_tool_name: &str) -> bool {
        false
    }

    pub fn mode_allows_tool(tool_name: &str, active_tools: &[String]) -> bool {
        if active_tools.is_empty() {
            return true;
        }
        let canonical_tool = crate::tools::stability::canonical_name(
            &Self::normalize_mode_tool_id(tool_name),
        )
        .to_string();
        active_tools.iter().any(|t| {
            let n = Self::normalize_mode_tool_id(t);
            crate::tools::stability::canonical_name(&n) == canonical_tool.as_str()
        })
    }

    pub fn enforce_mode_tool_gate(project_dir: &Path, tool_name: &str) -> Result<(), String> {
        if Self::is_core_tool(tool_name) {
            return Ok(());
        }
        if Self::is_project_workspace_tool(tool_name) {
            return Ok(());
        }
        // Dispatched agents (SHIP_ACTOR_ID set) always get platform tools.
        // This bypasses the KV-based active_agent lookup entirely — the
        // dispatch context is the source of truth.
        if Self::is_dispatched_context() && Self::is_platform_tool(tool_name) {
            return Ok(());
        }
        let active_agent =
            get_active_agent(Some(project_dir.to_path_buf())).map_err(|e| e.to_string())?;
        if let Some(ref mode) = active_agent {
            // Explicit active_tools is the complete allowlist when set —
            // lets profiles grant specific tools outside their preset
            // without adopting the whole preset.
            if !mode.active_tools.is_empty() {
                if Self::mode_allows_tool(tool_name, &mode.active_tools) {
                    return Ok(());
                }
                return Err(format!(
                    "Tool '{}' blocked by active mode '{}' (active_tools: {}).",
                    tool_name,
                    mode.id,
                    mode.active_tools.join(", ")
                ));
            }

            let preset = Self::effective_preset(mode.tools_preset.as_deref(), mode.role.as_deref());
            let preset_tools = Self::tools_for_preset(preset);
            let normalized = Self::normalize_mode_tool_id(tool_name);
            let canonical = crate::tools::stability::canonical_name(&normalized);
            if preset_tools.contains(&canonical) {
                return Ok(());
            }
            return Err(format!(
                "Tool '{}' blocked by active mode '{}' (preset '{}' allows: {}).",
                tool_name,
                mode.id,
                preset,
                preset_tools.join(", ")
            ));
        }
        Err(format!(
            "Tool '{}' is not in the core workflow surface. \
             Activate the service workspace ('ship') or a mode with this tool in its \
             active_tools list to use it.",
            tool_name
        ))
    }

    pub(crate) fn resolve_workspace_branch_for_project(
        project_dir: &Path,
        branch: Option<&str>,
    ) -> Result<String, String> {
        if let Some(b) = branch {
            let trimmed = b.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        let Some(root) = project_dir.parent() else {
            return Err("Error: Could not resolve project root".to_string());
        };
        current_branch(root).map_err(|e| e.to_string())
    }
}

pub(crate) fn current_branch(project_root: &Path) -> Result<String> {
    let output = ProcessCommand::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to determine current git branch");
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        anyhow::bail!("Current HEAD is detached; cannot map to a feature branch");
    }
    Ok(branch)
}
