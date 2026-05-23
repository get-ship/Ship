//! Skill authoring commands — the core CRUD over the platform `SkillsService`
//! (`ctx.state.skills()`). Skills *are* capability groups, so the gate
//! classifies `skills.*` as restricted (ship-authoring) — only an agent holding
//! that grant (designer / operator) may author them. Reads are skills.list/get.
//!
//! Scope-resolving like rules/mcp, so all return `ScopedMutation {id, scope}`.
//! The open frontmatter payload rides as `frontmatter_json` (the established
//! convention). File-byte ops (write_file/delete_file) and KV vars are separate
//! follow-ups — different transport / error surfaces.

use serde_json::Value;

use ship_platform::services::files::Scope;
use ship_platform::services::skills::service::CreateArgs;
use ship_platform::services::skills::state;

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

/// A required scope arg (move_scope's from / to — no scope-walk default).
fn req_scope(args: &Value, key: &str) -> Result<Scope, CommandError> {
    match req_str(args, key)?.as_str() {
        "project" => Ok(Scope::Project),
        "user" => Ok(Scope::User),
        other => Err(CommandError::bad_args(format!(
            "`{key}` must be 'project' or 'user', got `{other}`"
        ))),
    }
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
    let skills = ctx.state.skills();
    match name {
        "skills.create" => {
            let id = req_str(args, "id")?;
            let scope = parse_scope(args)?;
            let frontmatter = match args.get("frontmatter_json").and_then(|v| v.as_str()) {
                None | Some("") => None,
                Some(raw) => Some(serde_json::from_str(raw).map_err(|e| {
                    CommandError::bad_args(format!("`frontmatter_json` is not valid JSON: {e}"))
                })?),
            };
            let body = args.get("body").and_then(|v| v.as_str()).map(str::to_string);
            let resolved = skills
                .create(CreateArgs { id: id.clone(), scope, frontmatter, body })
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "skills.delete" => {
            let id = req_str(args, "id")?;
            let resolved = skills.delete(&id, parse_scope(args)?).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "skills.rename" => {
            let id = req_str(args, "id")?;
            let new_id = req_str(args, "new_id")?;
            let resolved = skills.rename(&id, &new_id, parse_scope(args)?).map_err(map_err)?;
            Ok(Some(ok(&new_id, resolved)))
        }
        "skills.duplicate" => {
            let id = req_str(args, "id")?;
            let new_id = req_str(args, "new_id")?;
            let resolved = skills.duplicate(&id, &new_id, parse_scope(args)?).map_err(map_err)?;
            Ok(Some(ok(&new_id, resolved)))
        }
        "skills.write_frontmatter" => {
            let id = req_str(args, "id")?;
            let raw = req_str(args, "frontmatter_json")?;
            let frontmatter: Value = serde_json::from_str(&raw).map_err(|e| {
                CommandError::bad_args(format!("`frontmatter_json` is not valid JSON: {e}"))
            })?;
            let resolved = skills
                .write_frontmatter(&id, parse_scope(args)?, &frontmatter)
                .map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "skills.write_body" => {
            let id = req_str(args, "id")?;
            let body = req_str(args, "body")?;
            let resolved = skills.write_body(&id, parse_scope(args)?, &body).map_err(map_err)?;
            Ok(Some(ok(&id, resolved)))
        }
        "skills.move_scope" => {
            let id = req_str(args, "id")?;
            let from = req_scope(args, "from")?;
            let to = req_scope(args, "to")?;
            skills.move_scope(&id, from, to).map_err(map_err)?;
            Ok(Some(ok(&id, to)))
        }
        _ => Ok(None),
    }
}
