//! Single-source `specs.*` ops — descriptors + dispatch via `ship_ops!`.
//!
//! Specs are a first-party collection (`SPECS_ID`); reads go through
//! `ship_platform::collections::store`.

use super::wire::{SpecGetResponse, SpecsListResponse};
use super::{required_str, RegistryError};
use crate::services::schema::ArgDescriptor;
use crate::services::specs::shape::{load_body, spec_from_item, Spec};

use ship_platform::collections::builtin::{specs_schema, SPECS_ID};
use ship_platform::collections::query::QueryRequest;
use ship_platform::collections::store;

ship_ops! {
    SPECS_OPS;
    {
        name: "specs.list",
        desc: "Raw spec records from the DB.",
        perm: "ship.specs.read",
        args: &[],
        ret: SpecsListResponse,
        run: |_args, state, _files| Ok(SpecsListResponse { items: list_specs(state)? }),
    },
    {
        name: "specs.get",
        desc: "Full spec record + body.",
        perm: "ship.specs.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        ret: SpecGetResponse,
        run: |args, state, _files| {
            let id = required_str(args, "id")?;
            let (spec, body) = get_spec_with_body(state, id)?;
            Ok(SpecGetResponse { spec, body })
        },
    },
}

fn list_specs(state: &crate::ship_runtime::ApiState) -> Result<Vec<Spec>, RegistryError> {
    let schema = specs_schema();
    let mut svc =
        store::open_data_db(state.ship_dir()).map_err(|e| RegistryError::service(e.to_string()))?;
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

fn get_spec_with_body(
    state: &crate::ship_runtime::ApiState,
    id: &str,
) -> Result<(Spec, String), RegistryError> {
    let schema = specs_schema();
    let mut svc =
        store::open_data_db(state.ship_dir()).map_err(|e| RegistryError::service(e.to_string()))?;
    store::ensure_collection_table(&mut svc, &schema)
        .map_err(|e| RegistryError::service(e.to_string()))?;
    let item = store::get_item(&mut svc, &schema, id)
        .map_err(|e| RegistryError::service(e.to_string()))?
        .ok_or_else(|| RegistryError::not_found(format!("spec '{id}' not found")))?;
    let spec = spec_from_item(item);
    let body = load_body(&spec.body_path).unwrap_or_default();
    Ok((spec, body))
}
