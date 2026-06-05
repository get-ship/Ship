//! Pipeline definition commands — doc CRUD over the platform `PipelinesService`
//! (`ctx.state.pipelines()`). Prefix split mirrors agents: `pipelines.*` (plural)
//! edits definitions (this module); `pipeline.*` (singular) is runtime lifecycle
//! (update_status/step_done — see command::pipeline). A pipeline is a workflow
//! composition over existing agents/skills, so editing it is ordinary workspace
//! config (C1 default) — running it dispatches, which is separately gated.
//!
//! Keyed by `name`; scope-resolving → `ScopedMutation` (name in the `id` field).
//! The doc / RFC-6902 patch ride as JSON-string args (doc_json / patch_json).

use serde_json::Value;

use ship_platform::services::files::Scope;
use ship_pipeline_runtime::pipelines::service::PutArgs;
use ship_pipeline_runtime::pipelines::state;

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
        state::Error::BadId(m) => CommandError::bad_args(m),
        other => CommandError::service(other.to_string()),
    }
}

/// `{ ok, id, scope }` — decodes into the typed `ScopedMutation` (id = name).
fn ok(name: &str, scope: Scope) -> Value {
    serde_json::json!({ "ok": true, "id": name, "scope": state::ScopeTag::from(scope) })
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let pipelines = ctx.state.pipelines();
    match name {
        "pipelines.put" => {
            let pname = req_str(args, "name")?;
            let doc = req_json(args, "doc_json")?;
            let resolved = pipelines
                .put(PutArgs { name: pname.clone(), doc, scope: parse_scope(args)? })
                .map_err(map_err)?;
            Ok(Some(ok(&pname, resolved)))
        }
        "pipelines.patch" => {
            let pname = req_str(args, "name")?;
            let patch: json_patch::Patch = serde_json::from_value(req_json(args, "patch_json")?)
                .map_err(|e| CommandError::bad_args(format!("bad patch shape: {e}")))?;
            let resolved = pipelines.patch(&pname, parse_scope(args)?, &patch).map_err(map_err)?;
            Ok(Some(ok(&pname, resolved)))
        }
        "pipelines.delete" => {
            let pname = req_str(args, "name")?;
            let resolved = pipelines.delete(&pname, parse_scope(args)?).map_err(map_err)?;
            Ok(Some(ok(&pname, resolved)))
        }
        _ => Ok(None),
    }
}
