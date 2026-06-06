//! Inline diff annotations for the git view, backed by the system
//! `annotations` collection.
//!
//! Annotations are pinned to a diff line via the `selector` field, which
//! stores `"{sha_or_WORKING}:{line_number}"` — e.g. `"WORKING:42"` or
//! `"ceb06db:63"`. The `file_path` field scopes them to a specific file.
//! `view_id` is set to `"git"` (or `"git:{workspace_id}"` when a workspace
//! is active) so comments from other views never bleed in.

use std::sync::Arc;

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

use ship_collection::builtin::annotations_schema;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_substrate::access::SharedKernel;
use ship_substrate::ports::{
    KernelLog, Predicate, Query, SingleBackendDataStore, Sort, SpaceContext, SpaceId,
};

/// Built-in `annotations` collection id.
const ANNOTATIONS_ID: &str = "annotations";

#[derive(Clone, Serialize)]
pub struct GitAnnotation {
    pub id: String,
    pub selector: String,
    pub note: Option<String>,
    pub intent: Option<String>,
    pub author_kind: String,
    pub created_at: String,
}

/// Map from selector key (`"{sha}:{line}"`) to its annotations, ordered by
/// creation time. The template iterates this to render threaded comments
/// adjacent to the matching diff line.
pub type AnnotationsByLine = BTreeMap<String, Vec<GitAnnotation>>;

/// Query the annotations collection for all comments attached to `file_path`
/// in the given git view context. Synchronous wrapper over the async
/// `CollectionService`: the registry op runs on a sync dispatch path, so it
/// bridges to the async query via `block_in_place` + the current runtime
/// handle (C-2 Gap 5 — registry ops keep their sync signature, the read
/// routes through the new service over a `SpaceContext`).
pub fn load_file_annotations(
    kernel: SharedKernel,
    ship_dir: &Path,
    file_path: &str,
    workspace_id: Option<&str>,
) -> AnnotationsByLine {
    let data_svc = match ship_substrate::data::DataService::open(ship_dir) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "failed to open data store for git annotations");
            return BTreeMap::new();
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
        table: format!("collection_{ANNOTATIONS_ID}"),
        filters: vec![
            Predicate::eq("file_path", serde_json::Value::String(file_path.to_string())),
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
            tracing::warn!(error = %e, file_path, "failed to load git annotations");
            return BTreeMap::new();
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

    out
}
