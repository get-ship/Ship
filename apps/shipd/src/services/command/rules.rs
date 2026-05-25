//! Rules commands — markdown behavior-config CRUD, delegated to the platform
//! `RulesService` (`ctx.state.rules()`). Reads are `rules.list`/`rules.get` in
//! the read registry; these are the write half. No event-building here.

use serde_json::Value;

use ship_platform::services::files::Scope;
use ship_platform::services::rules::service::{CreateArgs, PutArgs};
use ship_platform::services::rules::state;

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

/// Map a rules `state::Error` to the right command status.
fn map_err(e: state::Error) -> CommandError {
    match e {
        state::Error::NotFound(m) => CommandError::not_found(m),
        state::Error::AlreadyExists(m) => CommandError::bad_args(m),
        other => CommandError::service(other.to_string()),
    }
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let rules = ctx.state.rules();
    match name {
        "rules.create" => {
            let id = req_str(args, "id")?;
            let content = req_str(args, "content")?;
            let scope = parse_scope(args)?;
            let resolved = rules
                .create(CreateArgs { id: id.clone(), content, scope })
                .map_err(map_err)?;
            Ok(Some(serde_json::json!({ "ok": true, "id": id, "scope": format!("{resolved:?}") })))
        }
        "rules.put" => {
            let id = req_str(args, "id")?;
            let content = req_str(args, "content")?;
            let scope = parse_scope(args)?;
            let resolved = rules
                .put(PutArgs { id: id.clone(), content, scope })
                .map_err(map_err)?;
            Ok(Some(serde_json::json!({ "ok": true, "id": id, "scope": format!("{resolved:?}") })))
        }
        "rules.delete" => {
            let id = req_str(args, "id")?;
            let scope = parse_scope(args)?;
            let resolved = rules.delete(&id, scope).map_err(map_err)?;
            Ok(Some(serde_json::json!({ "ok": true, "id": id, "scope": format!("{resolved:?}") })))
        }
        _ => Ok(None),
    }
}
