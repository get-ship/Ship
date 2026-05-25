//! Agent profile authoring commands — CRUD over the platform `AgentsService`
//! (`ctx.state.agents()`). Note the prefix split: `agents.*` (plural) authors
//! profiles (this module); `agent.*` (singular) is runtime control
//! (dispatch/stop/steer — see command::agent). Authoring a profile defines an
//! agent's capability, so the gate classifies `agents.*` restricted
//! (ship-authoring), the same grant as skills. Reads are agents.list/get.
//!
//! Scope-resolving → `ScopedMutation`. The agent doc / RFC-6902 patch ride as
//! JSON-string args (`doc_json` / `patch_json`).

use serde_json::Value;

use ship_platform::services::agents::service::{CreateArgs, DuplicateArgs, PutArgs};
use ship_platform::services::agents::state;
use ship_platform::services::files::Scope;

use super::{req_str, CommandCtx, CommandError};

fn parse_scope(args: &Value, key: &str) -> Result<Option<Scope>, CommandError> {
    match args.get(key).and_then(|v| v.as_str()) {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(CommandError::bad_args(format!(
            "`{key}` must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn req_scope(args: &Value, key: &str) -> Result<Scope, CommandError> {
    parse_scope(args, key)?
        .ok_or_else(|| CommandError::bad_args(format!("`{key}` is required (project|user)")))
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
        state::Error::BadId(m) => CommandError::bad_args(m),
        other => CommandError::service(other.to_string()),
    }
}

/// `{ ok, id, scope }` — decodes into the typed `ScopedMutation`.
fn ok(id: &str, scope: Scope) -> Value {
    serde_json::json!({ "ok": true, "id": id, "scope": state::ScopeTag::from(scope) })
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let agents = ctx.state.agents();
    match name {
        "agents.create" => {
            let id = req_str(args, "id")?;
            let doc = req_json(args, "doc_json")?;
            let resolved = agents
                .create(CreateArgs { id: id.clone(), doc, scope: parse_scope(args, "scope")? })
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.put" => {
            let id = req_str(args, "id")?;
            let doc = req_json(args, "doc_json")?;
            let resolved = agents
                .put(PutArgs { id: id.clone(), doc, scope: parse_scope(args, "scope")? })
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.patch" => {
            let id = req_str(args, "id")?;
            let patch: json_patch::Patch = serde_json::from_value(req_json(args, "patch_json")?)
                .map_err(|e| CommandError::bad_args(format!("bad patch shape: {e}")))?;
            let resolved =
                agents.patch(&id, parse_scope(args, "scope")?, &patch).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.delete" => {
            let id = req_str(args, "id")?;
            let resolved = agents.delete(&id, parse_scope(args, "scope")?).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.rename" => {
            let id = req_str(args, "id")?;
            let new_id = req_str(args, "new_id")?;
            let resolved =
                agents.rename(&id, &new_id, parse_scope(args, "scope")?).map_err(map_err)?;
            Ok(Some(ok(&new_id, resolved)))
        }
        "agents.duplicate" => {
            let source_id = req_str(args, "id")?;
            let new_id = req_str(args, "new_id")?;
            let resolved = agents
                .duplicate(DuplicateArgs {
                    source_id,
                    source_scope: parse_scope(args, "scope")?,
                    new_id: new_id.clone(),
                    to_scope: parse_scope(args, "to")?,
                })
                .map_err(map_err)?;
            Ok(Some(ok(&new_id, resolved)))
        }
        "agents.move_scope" => {
            let id = req_str(args, "id")?;
            let from = req_scope(args, "from")?;
            let to = req_scope(args, "to")?;
            agents.move_scope(&id, from, to).map_err(map_err)?;
            Ok(Some(ok(&id, to)))
        }
        _ => Ok(None),
    }
}
