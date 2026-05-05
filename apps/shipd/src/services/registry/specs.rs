//! Registry dispatcher for `specs.*` ops. Specs are a first-party
//! collection (`SPECS_ID`); reads go through `ship_platform::collections::store`
//! directly, then pipe through `specs::shape` / `specs::view_state` for
//! the wire shapes the Studio specs view consumes.

use serde_json::{Value, json};

use super::{RegistryError, html_escape, optional_usize, parse_scope, required_str};
use crate::rest_api::ApiState;
use crate::services::files::{FilesService, Scope};
use crate::services::specs::shape::{Spec, load_body, spec_from_item};
use crate::services::specs::view_state;

use ship_platform::collections::builtin::{SPECS_ID, specs_schema};
use ship_platform::collections::query::QueryRequest;
use ship_platform::collections::store;

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "specs.list" => {
            let specs = list_specs(state)?;
            Ok(json!({ "items": specs }))
        }
        "specs.get" => {
            let id = required_str(args, "id")?;
            let (spec, body) = get_spec_with_body(state, id)?;
            Ok(json!({ "spec": spec, "body": body }))
        }
        "specs.summaries" => {
            let q = args.get("q").and_then(|v| v.as_str()).unwrap_or("");
            let selected = args
                .get("selected")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let specs = list_specs(state)?;
            let ctx = view_state::build_summaries(&specs, q, selected);
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "specs.detail" => {
            let id = required_str(args, "id")?;
            let (spec, body) = match get_spec_with_body(state, id) {
                Ok(pair) => pair,
                Err(e) if e.code == "not_found" => {
                    return Err(RegistryError::not_found(format!("spec `{id}`")));
                }
                Err(e) => return Err(e),
            };
            let detail = view_state::build_detail(&spec, &body);
            serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}

fn list_specs(state: &ApiState) -> Result<Vec<Spec>, RegistryError> {
    let schema = specs_schema();
    let mut svc = store::open_data_db(state.ship_dir())
        .map_err(|e| RegistryError::service(e.to_string()))?;
    store::ensure_collection_table(&mut svc, &schema)
        .map_err(|e| RegistryError::service(e.to_string()))?;
    let req = QueryRequest {
        collection: SPECS_ID.to_string(),
        filter: Default::default(),
        sort: {
            let mut s = std::collections::BTreeMap::new();
            s.insert("updated_at".into(), -1);
            s
        },
        limit: 1000,
        offset: 0,
    };
    let items = store::query_items(&mut svc, &schema, &req)
        .map_err(|e| RegistryError::service(e.to_string()))?;
    Ok(items.into_iter().map(spec_from_item).collect())
}

fn get_spec_with_body(state: &ApiState, id: &str) -> Result<(Spec, String), RegistryError> {
    let schema = specs_schema();
    let mut svc = store::open_data_db(state.ship_dir())
        .map_err(|e| RegistryError::service(e.to_string()))?;
    store::ensure_collection_table(&mut svc, &schema)
        .map_err(|e| RegistryError::service(e.to_string()))?;
    let item = store::get_item(&mut svc, &schema, id)
        .map_err(|e| RegistryError::service(e.to_string()))?
        .ok_or_else(|| RegistryError::not_found(format!("spec '{id}' not found")))?;
    let spec = spec_from_item(item);
    let body = load_body(&spec.body_path).unwrap_or_default();
    Ok((spec, body))
}
