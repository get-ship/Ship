//! Single-source `permissions.*` ops — descriptors + dispatch via `ship_ops!`.

use std::path::Path;

use super::{parse_scope, required_str, RegistryError};
use crate::services::schema::{ArgDescriptor, PROJECT_SCOPE};

use ship_platform::services::permissions::state;
use ship_platform::services::permissions::state::ScopeTag;
use ship_platform::services::permissions::wire_types::{
    PermissionGetResponse, PermissionsIndexResponse, ToolPattern, ToolPatternsResponse,
};

ship_ops! {
    PERMISSIONS_OPS;
    {
        name: "permissions.list",
        desc: "List permission presets across scopes.",
        perm: "ship.permissions.read",
        args: &[],
        ret: PermissionsIndexResponse,
        run: |_args, _state, files| Ok(PermissionsIndexResponse { items: state::list(files) }),
    },
    {
        name: "permissions.get",
        desc: "Raw permission preset at the requested scope.",
        perm: "ship.permissions.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        ret: PermissionGetResponse,
        run: |args, _state, files| {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = state::load_entry(files, id, scope).map_err(|e| match e {
                state::Error::NotFound(m) => RegistryError::not_found(m),
                other => RegistryError::service(other.to_string()),
            })?;
            Ok(PermissionGetResponse { id: id.to_string(), scope: ScopeTag::from(resolved), entry })
        },
    },
    {
        name: "permissions.tool_patterns",
        desc: "Enumerate tool-pattern capability names (for agent builders).",
        perm: "ship.permissions.read",
        args: &[],
        ret: ToolPatternsResponse,
        run: |_args, state, _files| Ok(ToolPatternsResponse { patterns: tool_patterns(state) }),
    },
}

/// Reassemble the tool-pattern catalog the `permissions` HTTP handler builds:
/// harness builtins + curated Bash shapes + per-server `mcp__<server>__*`
/// wildcards. Byte-identical to the prior anonymous `json!` rows.
fn tool_patterns(state: &crate::ship_runtime::ApiState) -> Vec<ToolPattern> {
    let mut out: Vec<ToolPattern> = Vec::new();
    for t in &[
        "Read", "Write", "Edit", "Glob", "Grep", "Bash", "NotebookEdit", "WebFetch", "WebSearch",
        "Task",
    ] {
        out.push(ToolPattern { pattern: t.to_string(), label: t.to_string(), source: "harness".into() });
    }
    for (p, l) in &[
        ("Bash(git *)", "Bash git · all git subcommands"),
        ("Bash(git reset*)", "Bash git · reset (destructive)"),
        ("Bash(git push --force*)", "Bash git · force push (destructive)"),
        ("Bash(npm *)", "Bash npm"),
        ("Bash(bun *)", "Bash bun"),
        ("Bash(cargo *)", "Bash cargo"),
        ("Bash(ship *)", "Bash ship"),
        ("Bash(just *)", "Bash just"),
        ("Bash(rm -rf *)", "Bash rm -rf (destructive)"),
    ] {
        out.push(ToolPattern { pattern: (*p).into(), label: (*l).into(), source: "bash".into() });
    }
    let project_root = state
        .ship_dir()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| state.ship_dir().to_path_buf());
    if let Ok(servers) = ship_platform::config::list_mcp_servers(Some(project_root)) {
        for server in servers {
            let sid = if server.id.is_empty() { server.name.clone() } else { server.id };
            out.push(ToolPattern {
                pattern: format!("mcp__{sid}__*"),
                label: format!("mcp · {sid} · all tools"),
                source: format!("mcp-{sid}"),
            });
        }
    }
    out
}
