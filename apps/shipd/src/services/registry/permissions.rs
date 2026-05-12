//! Registry dispatcher for `permissions.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, html_escape, optional_usize, parse_scope, required_str};
use crate::ship_runtime::ApiState;
use ship_platform::services::files::{FilesService, Scope};
use std::path::Path;

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "permissions.list" => Ok(json!({
            "items": ship_platform::services::permissions::state::list(&files),
        })),
        "permissions.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = ship_platform::services::permissions::state::load_entry(&files, id, scope)
                .map_err(|e| match e {
                    ship_platform::services::permissions::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": ship_platform::services::permissions::state::ScopeTag::from(resolved),
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
            if let Ok(servers) = ship_platform::config::list_mcp_servers(Some(project_root)) {
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

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
