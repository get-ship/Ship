//! Registry dispatcher for `pipelines.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, optional_usize, parse_scope, required_str};
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "pipelines.list" => Ok(json!({
            "items": ship_platform::services::pipelines::state::list(files),
        })),
        "pipelines.get" => {
            let n = required_str(args, "name")?;
            let scope = parse_scope(args)?;
            let (resolved, ext, doc) = ship_platform::services::pipelines::state::load_doc(files, n, scope)
                .map_err(|e| match e {
                    ship_platform::services::pipelines::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "name": n,
                "scope": ship_platform::services::pipelines::state::ScopeTag::from(resolved),
                "format": ext,
                "doc": doc,
            }))
        }
        "pipelines.summaries" => {
            let filter = crate::views::state::Filter {
                q: args.get("q").and_then(|v| v.as_str()).map(String::from),
                agent: args.get("agent").and_then(|v| v.as_str()).map(String::from),
                pipeline: args.get("pipeline").and_then(|v| v.as_str()).map(String::from),
                status: args.get("status").and_then(|v| v.as_str()).map(String::from),
            };
            let tab = args.get("tab").and_then(|v| v.as_str()).map(String::from);
            let ctx = crate::services::pipelines::summaries::build(state.ship_dir(), filter, tab);
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "pipelines.focus" => {
            let n = required_str(args, "name")?;
            let ctx = crate::services::pipelines::focus::build(n, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("pipeline `{n}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "pipelines.run" => {
            let slug = required_str(args, "slug")?;
            let ctx = crate::services::pipelines::run_detail::build(slug, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("run `{slug}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "pipelines.history" => {
            let q = args.get("q").and_then(|v| v.as_str()).map(String::from);
            let pipeline = args.get("pipeline").and_then(|v| v.as_str()).map(String::from);
            let offset = optional_usize(args, "offset", 0);
            let limit = optional_usize(args, "limit", 50);
            let ctx = crate::services::pipelines::history::build(
                state.ship_dir(), q, pipeline, offset, limit,
            );
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
