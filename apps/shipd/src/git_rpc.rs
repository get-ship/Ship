//! Git RPC helpers — shipd-side functions the `GitWire` (ship-git crate) needs
//! injected as closures because they depend on shipd-owned state or crates that
//! ship-git cannot depend on directly (ship-collection, the active-space context).
//!
//! The `impl GitService for OpsImpl` and `gated_git_write` have moved to
//! `ship_git::wire::GitWire`. This file retains only the helpers that require
//! shipd's active-space context: `git_annotations_value`.

use serde_json::Value;

/// Build the git.annotations JSON value.
///
/// The annotations feature queries the built-in `annotations` collection for
/// comments pinned to diff lines in the git view. Relocated from
/// `ship_git::annotations` during the L0 collapse (ship-collection dep removed
/// from ship-git). shipd already depends on both ship-git and ship-collection,
/// so this is the natural L1 home. Exposed pub(crate) so `ops_rpc` can wrap it
/// as the `AnnotationsFn` closure supplied to `GitWire`.
pub(crate) fn git_annotations_value(
    state: &crate::ship_runtime::ApiState,
    workspace_id: Option<&str>,
    file: &str,
) -> Value {
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use ship_collection::builtin::annotations_schema;
    use ship_collection::projection::SchemaRegistry;
    use ship_collection::service::CollectionService;
    use ship_substrate::ports::{
        KernelLog, Predicate, Query, SingleBackendDataStore, Sort, SpaceContext, SpaceId,
    };
    use serde::Serialize;

    #[derive(Clone, Serialize)]
    struct GitAnnotation {
        id: String,
        selector: String,
        note: Option<String>,
        intent: Option<String>,
        author_kind: String,
        created_at: String,
    }
    type AnnotationsByLine = BTreeMap<String, Vec<GitAnnotation>>;

    let kernel = state.kernel().clone();
    let ship_dir = state.ship_dir().to_path_buf();

    let data_svc = match ship_substrate::data::DataService::open(&ship_dir) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "failed to open data store for git annotations");
            return Value::Null;
        }
    };
    let store = Arc::new(SingleBackendDataStore::new(data_svc, "global"));
    let kernel_log = Arc::new(KernelLog::new(kernel));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new("global"), store, kernel_log));
    let registry = SchemaRegistry::new();
    registry.insert(annotations_schema());
    let svc = CollectionService { ctx, schemas: Arc::new(registry) };

    let view_id = match workspace_id.filter(|s| !s.is_empty() && *s != "project") {
        Some(id) => format!("git:{id}"),
        None => "git".to_string(),
    };
    let q = Query {
        table: format!("collection_annotations"),
        filters: vec![
            Predicate::eq("file_path", serde_json::Value::String(file.to_string())),
            Predicate::eq("view_id", serde_json::Value::String(view_id)),
        ],
        sort: Some(Sort { column: "created_at".into(), descending: false }),
        limit: 200,
        after: None,
    };

    let items = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(svc.query_items_by_query(q))
    });
    let items = match items {
        Ok((rows, _)) => rows,
        Err(e) => {
            tracing::warn!(error = %e, file, "failed to load git annotations");
            return Value::Null;
        }
    };

    let mut out: AnnotationsByLine = BTreeMap::new();
    for item in items {
        let data = &item.data;
        let selector = match data.get("selector").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let annotation = GitAnnotation {
            id: item.id.clone(),
            selector: selector.clone(),
            note: data.get("note").and_then(|v| v.as_str()).map(str::to_string),
            intent: data.get("intent").and_then(|v| v.as_str()).map(str::to_string),
            author_kind: data
                .get("author_kind")
                .and_then(|v| v.as_str())
                .unwrap_or("human")
                .to_string(),
            created_at: item.created_at.clone(),
        };
        out.entry(selector).or_default().push(annotation);
    }
    serde_json::to_value(&out).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn git_op_policy_reads_are_c0_safe() {
        for path in [
            "ship.ops.v1.GitService/Status",
            "ship.ops.v1.GitService/Log",
            "ship.ops.v1.GitService/Branches",
            "ship.ops.v1.GitService/Overview",
            "ship.ops.v1.GitService/Diff",
            "ship.ops.v1.GitService/Annotations",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.git.read", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read, "{path}");
        }
    }

    #[test]
    fn git_op_policy_writes_are_c1_workspace() {
        for path in [
            "ship.ops.v1.GitService/Push",
            "ship.ops.v1.GitService/Fetch",
            "ship.ops.v1.GitService/Pull",
            "ship.ops.v1.GitService/Stage",
            "ship.ops.v1.GitService/Unstage",
            "ship.ops.v1.GitService/Commit",
            "ship.ops.v1.GitService/Checkout",
            "ship.ops.v1.GitService/BranchCreate",
            "ship.ops.v1.GitService/BranchDelete",
            "ship.ops.v1.GitService/Init",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.git.write", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space, "{path}");
        }
    }
}
