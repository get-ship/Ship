//! Registry dispatcher for `artifacts.*` ops.
//!
//! Sync mirror of the async `ArtifactsService::list` surface — opens the
//! data DB directly the same way `collections.list` does, so the registry
//! call doesn't need to bounce through the tokio runtime.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::{RegistryError, optional_usize};
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
        "artifacts.list" => {
            let scope = args.get("scope").and_then(|v| v.as_str())
                .unwrap_or("project").to_string();
            let mut filter: BTreeMap<String, Value> = BTreeMap::new();
            filter.insert("scope".into(), Value::String(scope.clone()));
            for key in ["mime", "collection_id", "item_id", "owner_actor", "content_hash"] {
                if let Some(v) = args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                    filter.insert(key.into(), Value::String(v.to_string()));
                }
            }
            let mut sort: BTreeMap<String, i8> = BTreeMap::new();
            let s = args.get("sort").and_then(|v| v.as_str()).unwrap_or("-created_at");
            let (field, dir) = if let Some(rest) = s.strip_prefix('-') { (rest, -1i8) } else { (s, 1i8) };
            if !field.is_empty() { sort.insert(field.to_string(), dir); }
            let limit = optional_usize(args, "limit", 50) as u32;
            let offset = optional_usize(args, "offset", 0) as u32;
            let req = ship_platform::collections::query::QueryRequest {
                collection: ship_platform::artifacts::ARTIFACTS_COLLECTION_ID.to_string(),
                filter, sort, limit, offset,
            };
            let mut svc = ship_platform::collections::store::open_data_db(state.ship_dir())
                .map_err(|e| RegistryError::service(e.to_string()))?;
            let runtime = ship_platform::artifacts::ArtifactService::new(state.ship_dir());
            let items = runtime.list(&mut svc, req)
                .map_err(|e| RegistryError::service(e.to_string()))?;
            let count = items.len();
            Ok(json!({
                "items": items,
                "count": count,
                "scope": scope,
            }))
        }
        _ => return Ok(None),
    };
    Ok(Some(value?))
}
