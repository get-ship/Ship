//! Permissions commands — permission-preset config CRUD, delegated to the
//! platform `PermissionsService` (`ctx.state.permissions()`). Reads are
//! `permissions.list`/`get`/`tool_patterns`; these are the write half.
//!
//! A preset *is* the security policy for what a class of agents may do, so the
//! gate classifies `permissions.*` as **C2 (operator-only)** — agents never
//! edit the policy that governs them, regardless of grants. See `services::gate`.
//!
//! The JSON-valued `preset` / RFC-6902 `patch` travel as JSON-string args
//! (`preset_json` / `patch_json`).

use serde_json::Value;

use ship_platform::services::files::Scope;
use ship_platform::services::permissions::service::{CreateArgs, PutArgs};
use ship_platform::services::permissions::state;

use super::{req_str, CommandCtx, CommandError};

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

fn req_json(args: &Value, key: &str) -> Result<Value, CommandError> {
    let raw = req_str(args, key)?;
    serde_json::from_str(&raw)
        .map_err(|e| CommandError::bad_args(format!("`{key}` is not valid JSON: {e}")))
}

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
    let perms = ctx.state.permissions();
    match name {
        "permissions.create" => {
            let id = req_str(args, "id")?;
            let preset = req_json(args, "preset_json")?;
            let scope = parse_scope(args)?;
            let resolved = perms
                .create(CreateArgs { id: id.clone(), preset, scope })
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "permissions.put" => {
            let id = req_str(args, "id")?;
            let preset = req_json(args, "preset_json")?;
            let scope = parse_scope(args)?;
            let resolved = perms
                .put(PutArgs { id: id.clone(), preset, scope })
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "permissions.patch" => {
            let id = req_str(args, "id")?;
            let patch: json_patch::Patch = serde_json::from_value(req_json(args, "patch_json")?)
                .map_err(|e| CommandError::bad_args(format!("bad patch shape: {e}")))?;
            let scope = parse_scope(args)?;
            let resolved = perms.patch(&id, scope, &patch).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "permissions.delete" => {
            let id = req_str(args, "id")?;
            let scope = parse_scope(args)?;
            let resolved = perms.delete(&id, scope).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        _ => Ok(None),
    }
}
