//! MCP commands — external MCP-server config CRUD, delegated to the platform
//! `McpService` (`ctx.state.mcp()`). Reads are `mcp.list`/`mcp.get`; these are
//! the write half. Registering a server adds tools agents can call, so the gate
//! classifies `mcp.*` as restricted (ship-authoring) — see `services::gate`.
//!
//! JSON-valued args (`server`, RFC-6902 `patch`) arrive as JSON-string args
//! (`server_json` / `patch_json`), matching the mesh/view command convention.

use serde_json::Value;

use ship_platform::services::files::Scope;
use ship_platform::services::mcp::service::{CreateArgs, PutArgs};
use ship_platform::services::mcp::state;

use super::{req_str, CommandCtx, CommandError};

/// Parse the optional `scope` arg ("project" | "user"); absent walks scopes.
fn parse_scope(args: &Value) -> Result<Option<Scope>, CommandError> {
    match args.get("scope").and_then(|v| v.as_str()) {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(CommandError::bad_args(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

/// Parse a required JSON-string arg into a `Value`.
fn req_json(args: &Value, key: &str) -> Result<Value, CommandError> {
    let raw = req_str(args, key)?;
    serde_json::from_str(&raw)
        .map_err(|e| CommandError::bad_args(format!("`{key}` is not valid JSON: {e}")))
}

/// Map an mcp `state::Error` to the right command status.
fn map_err(e: state::Error) -> CommandError {
    match e {
        state::Error::NotFound(m) => CommandError::not_found(m),
        state::Error::AlreadyExists(m) => CommandError::bad_args(m),
        state::Error::BadId(m) | state::Error::Parse(m) | state::Error::Patch(m) => {
            CommandError::bad_args(m)
        }
        other => CommandError::service(other.to_string()),
    }
}

/// `{ ok, id, scope }` — the shape the typed `ScopedMutation` response decodes.
fn ok(id: &str, scope: Scope) -> Value {
    serde_json::json!({ "ok": true, "id": id, "scope": state::ScopeTag::from(scope) })
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let mcp = ctx.state.mcp();
    match name {
        "mcp.create" => {
            let id = req_str(args, "id")?;
            let server = req_json(args, "server_json")?;
            let scope = parse_scope(args)?;
            let resolved = mcp
                .create(CreateArgs { id: id.clone(), server, scope })
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "mcp.put" => {
            let id = req_str(args, "id")?;
            let server = req_json(args, "server_json")?;
            let scope = parse_scope(args)?;
            let resolved = mcp
                .put(PutArgs { id: id.clone(), server, scope })
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "mcp.patch" => {
            let id = req_str(args, "id")?;
            let patch: json_patch::Patch = serde_json::from_value(req_json(args, "patch_json")?)
                .map_err(|e| CommandError::bad_args(format!("bad patch shape: {e}")))?;
            let scope = parse_scope(args)?;
            let resolved = mcp.patch(&id, scope, &patch).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "mcp.delete" => {
            let id = req_str(args, "id")?;
            let scope = parse_scope(args)?;
            let resolved = mcp.delete(&id, scope).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        _ => Ok(None),
    }
}
