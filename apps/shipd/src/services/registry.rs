//! Service registry — the single entry point for calling domain
//! services by name. The template SDK (`ship.*` minijinja globals),
//! the forthcoming MCP tool surface, and the TS SDK codegen all
//! resolve through this module.
//!
//! Registered names read like `<domain>.<op>` — `agents.list`,
//! `git.diff`, `permissions.tool_patterns`. Args are a JSON object;
//! unknown keys ignored, required-missing raise `BadArgs`.
//!
//! Read ops only. Writes are exposed as HTTP endpoints (POST/PATCH/
//! PUT/DELETE on each service), not registry entries — keeps the
//! security boundary between "reading" and "mutating" explicit.
//!
//! The registry is synchronous. Every domain state function we call
//! is sync (file I/O via FilesService, libgit2 direct, DB queries).
//! That lets us expose `call` to minijinja without a blocking bridge.

use serde_json::{Value, json};
use std::path::Path;

use crate::rest_api::ApiState;
use crate::services::files::{FilesService, Scope};

/// Structured error returned from a registry call. Rendered verbatim
/// into the template as an `{error, message}` object (per spec §11.3
/// — structured errors, template decides how to render).
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistryError {
    pub code: &'static str,
    pub message: String,
}

impl RegistryError {
    fn bad_args(msg: impl Into<String>) -> Self {
        Self { code: "bad_args", message: msg.into() }
    }
    fn unknown_service(name: &str) -> Self {
        Self { code: "unknown_service", message: format!("unknown service `{name}`") }
    }
    fn service(msg: impl Into<String>) -> Self {
        Self { code: "service_error", message: msg.into() }
    }
    fn not_found(msg: impl Into<String>) -> Self {
        Self { code: "not_found", message: msg.into() }
    }
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
impl std::error::Error for RegistryError {}

// ── Args helpers ─────────────────────────────────────────────────────

fn parse_scope(args: &Value) -> Result<Option<Scope>, RegistryError> {
    match args.get("scope").and_then(|v| v.as_str()) {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(RegistryError::bad_args(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn required_str<'a>(args: &'a Value, field: &str) -> Result<&'a str, RegistryError> {
    args.get(field)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| RegistryError::bad_args(format!("missing required arg `{field}`")))
}

fn optional_usize(args: &Value, field: &str, default: usize) -> usize {
    args.get(field)
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(default)
}

// ── Dispatch ─────────────────────────────────────────────────────────

/// Dispatch a registered service call. Synchronous — see module docs.
pub fn call(name: &str, args: &Value, state: &ApiState) -> Result<Value, RegistryError> {
    let files = FilesService::resolve_roots(state.ship_dir());
    match name {
        // ── agents ────────────────────────────────────────────────
        "agents.list" => Ok(json!({
            "items": crate::services::agents::state::list(&files),
        })),
        "agents.summaries" => Ok(json!({
            "items": crate::services::agents::view_state::list_agents(state.ship_dir()),
        })),
        "agents.detail" => {
            let id = required_str(args, "id")?;
            let detail = crate::services::agents::view_state::load_agent(state.ship_dir(), id)
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))?;
            serde_json::to_value(&detail)
                .map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.catalogs" => {
            let cats = crate::services::agents::view_state::list_agent_catalogs(state.ship_dir());
            serde_json::to_value(&cats)
                .map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, doc) = crate::services::agents::state::load_doc(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::agents::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::agents::state::ScopeTag::from(resolved),
                "doc": doc,
            }))
        }

        // ── skills ────────────────────────────────────────────────
        "skills.list" => Ok(json!({
            "items": crate::services::skills::state::list(&files),
        })),
        "skills.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let detail = crate::services::skills::state::load(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::skills::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))?)
        }

        // ── rules ─────────────────────────────────────────────────
        "rules.list" => Ok(json!({
            "items": crate::services::rules::state::list(&files),
        })),
        "rules.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, body) = crate::services::rules::state::load_body(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::rules::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::rules::state::ScopeTag::from(resolved),
                "body": body,
            }))
        }

        // ── mcp ───────────────────────────────────────────────────
        "mcp.list" => Ok(json!({
            "items": crate::services::mcp::state::list(&files),
        })),
        "mcp.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = crate::services::mcp::state::load_entry(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::mcp::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::mcp::state::ScopeTag::from(resolved),
                "entry": entry,
            }))
        }

        // ── permissions ───────────────────────────────────────────
        "permissions.list" => Ok(json!({
            "items": crate::services::permissions::state::list(&files),
        })),
        "permissions.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = crate::services::permissions::state::load_entry(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::permissions::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::permissions::state::ScopeTag::from(resolved),
                "entry": entry,
            }))
        }
        "permissions.tool_patterns" => {
            // Reassemble the composition the mod.rs handler does;
            // we can't call the axum handler directly from here.
            let mut out: Vec<serde_json::Value> = Vec::new();
            for t in &["Read", "Write", "Edit", "Glob", "Grep", "Bash",
                      "NotebookEdit", "WebFetch", "WebSearch", "Task"] {
                out.push(json!({ "pattern": t, "label": t, "source": "provider" }));
            }
            for (p, l) in &[
                ("Bash(git *)", "Bash git · all git subcommands"),
                ("Bash(git reset*)", "Bash git · reset (destructive)"),
                ("Bash(git push --force*)", "Bash git · force push (destructive)"),
                ("Bash(npm *)", "Bash npm"),
                ("Bash(pnpm *)", "Bash pnpm"),
                ("Bash(cargo *)", "Bash cargo"),
                ("Bash(ship *)", "Bash ship"),
                ("Bash(just *)", "Bash just"),
                ("Bash(rm -rf *)", "Bash rm -rf (destructive)"),
            ] {
                out.push(json!({ "pattern": p, "label": l, "source": "bash" }));
            }
            // Per-server `mcp__<server>__*` wildcards via McpService.
            let project_root = state.ship_dir().parent().map(Path::to_path_buf)
                .unwrap_or_else(|| state.ship_dir().to_path_buf());
            if let Ok(servers) = runtime::config::list_mcp_servers(Some(project_root)) {
                for server in servers {
                    let sid = if server.id.is_empty() { server.name.clone() } else { server.id };
                    out.push(json!({
                        "pattern": format!("mcp__{sid}__*"),
                        "label": format!("mcp · {sid} · all tools"),
                        "source": format!("mcp-{sid}"),
                    }));
                }
            }
            Ok(json!({ "patterns": out }))
        }

        // ── pipelines ─────────────────────────────────────────────
        "pipelines.list" => Ok(json!({
            "items": crate::services::pipelines::state::list(&files),
        })),
        "pipelines.get" => {
            let n = required_str(args, "name")?;
            let scope = parse_scope(args)?;
            let (resolved, ext, doc) = crate::services::pipelines::state::load_doc(&files, n, scope)
                .map_err(|e| match e {
                    crate::services::pipelines::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "name": n,
                "scope": crate::services::pipelines::state::ScopeTag::from(resolved),
                "format": ext,
                "doc": doc,
            }))
        }

        // ── git ───────────────────────────────────────────────────
        "git.status" => {
            let root = crate::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            Ok(serde_json::to_value(crate::services::git::read::load_status_buckets(&root))
                .map_err(|e| RegistryError::service(e.to_string()))?)
        }
        "git.log" => {
            let root = crate::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            let limit = optional_usize(args, "limit", 50);
            Ok(json!({ "commits": crate::services::git::read::load_log(&root, None, limit) }))
        }
        "git.branches" => {
            let root = crate::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            let (ahead, behind) = crate::services::git::read::load_ahead_behind(&root)
                .unwrap_or((0, 0));
            Ok(json!({
                "branches": crate::services::git::read::load_branches(&root),
                "ahead": ahead,
                "behind": behind,
            }))
        }

        other => Err(RegistryError::unknown_service(other)),
    }
}

/// Enumerate registered service names — drives SDK codegen, MCP tool
/// registration, lint diagnostics.
pub fn names() -> &'static [&'static str] {
    &[
        "agents.list", "agents.get", "agents.summaries", "agents.detail", "agents.catalogs",
        "skills.list", "skills.get",
        "rules.list", "rules.get",
        "mcp.list", "mcp.get",
        "permissions.list", "permissions.get", "permissions.tool_patterns",
        "pipelines.list", "pipelines.get",
        "git.status", "git.log", "git.branches",
    ]
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
