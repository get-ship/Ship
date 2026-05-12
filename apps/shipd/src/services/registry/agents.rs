//! Registry dispatcher for `agents.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, parse_scope, required_str};
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
        "agents.list" => Ok(json!({
            "items": ship_platform::services::agents::state::list(files),
        })),
        "agents.summaries" => Ok(json!({
            "items": ship_platform::services::agents::view_state::list_agents(state.ship_dir()),
        })),
        "agents.detail" => {
            let id = required_str(args, "id")?;
            let detail = ship_platform::services::agents::view_state::load_agent(state.ship_dir(), id)
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))?;
            serde_json::to_value(&detail)
                .map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.catalogs" => {
            let cats = ship_platform::services::agents::view_state::list_agent_catalogs(state.ship_dir());
            serde_json::to_value(&cats)
                .map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, doc) = ship_platform::services::agents::state::load_doc(files, id, scope)
                .map_err(|e| match e {
                    ship_platform::services::agents::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": ship_platform::services::agents::state::ScopeTag::from(resolved),
                "doc": doc,
            }))
        }
        "agents.wire" => {
            let id = required_str(args, "id")?;
            let ctx = ship_platform::services::agents::wire::build(id, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.settings" => {
            let id = required_str(args, "id")?;
            let ctx = ship_platform::services::agents::settings_view::build(id, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.templates" => Ok(json!({
            "items": ship_platform::services::agents::create::builtin_templates(),
        })),
        "agents.permission_presets" => Ok(json!({
            "items": ship_platform::services::agents::create::builtin_permission_presets(),
        })),
        "agents.providers" => Ok(json!({
            "items": ship_platform::services::agents::create::builtin_providers(),
        })),
        "agents.create_ctx" => {
            let ctx = ship_platform::services::agents::create::build_create_ctx();
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
