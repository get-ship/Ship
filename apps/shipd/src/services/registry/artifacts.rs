//! Single-source `artifacts.*` ops — descriptors + dispatch via `ship_ops!`.
//!
//! Reads route through the `ship-artifacts` service over the daemon's global
//! `SpaceContext` (C-2). The registry op is sync, so the async `list` bridges
//! via `block_in_place` + the current runtime handle. `sort`/`offset` are
//! dropped (C-2 Gap 2) — the only caller hardcoded `-created_at`, which the
//! ULID-ordered projection already yields newest-first.

use std::collections::BTreeMap;

use serde_json::Value;

use super::optional_usize;
use super::wire::ArtifactsListResponse;
use super::RegistryError;
use crate::services::schema::ArgDescriptor;

ship_ops! {
    ARTIFACTS_OPS;
    {
        name: "artifacts.list",
        desc: "Artifact metadata records, filterable by scope/mime/collection/item/owner.",
        perm: "ship.artifacts.read",
        args: &[
            ArgDescriptor { name: "scope", kind: "string?", required: false },
            ArgDescriptor { name: "mime", kind: "string?", required: false },
            ArgDescriptor { name: "collection_id", kind: "string?", required: false },
            ArgDescriptor { name: "item_id", kind: "string?", required: false },
            ArgDescriptor { name: "owner_actor", kind: "string?", required: false },
            ArgDescriptor { name: "content_hash", kind: "string?", required: false },
            ArgDescriptor { name: "limit", kind: "number?", required: false },
        ],
        ret: ArtifactsListResponse,
        run: |args, state, _files| {
            let scope = args.get("scope").and_then(|v| v.as_str()).unwrap_or("project").to_string();
            let mut filter: BTreeMap<String, Value> = BTreeMap::new();
            filter.insert("scope".into(), Value::String(scope.clone()));
            for key in ["mime", "collection_id", "item_id", "owner_actor", "content_hash"] {
                if let Some(v) = args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                    filter.insert(key.into(), Value::String(v.to_string()));
                }
            }
            let limit = optional_usize(args, "limit", 50) as u32;
            let svc = state.artifacts();
            let items = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(svc.list(filter, limit))
            })
            .map_err(|e| RegistryError::service(e.to_string()))?;
            let count = items.len();
            Ok(ArtifactsListResponse { items, count, scope })
        },
    },
}
