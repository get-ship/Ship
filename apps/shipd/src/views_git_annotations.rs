//! Inline diff annotations for the git view, backed by the system
//! `annotations` collection.
//!
//! Annotations are pinned to a diff line via the `selector` field, which
//! stores `"{sha_or_WORKING}:{line_number}"` — e.g. `"WORKING:42"` or
//! `"ceb06db:63"`. The `file_path` field scopes them to a specific file.
//! `view_id` is set to `"git"` (or `"git:{workspace_id}"` when a workspace
//! is active) so comments from other views never bleed in.

use runtime::collections::{builtin::ANNOTATIONS_ID, query::QueryRequest, service};
use runtime::data::DataService;
use serde::Serialize;
use std::collections::BTreeMap;

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
/// in the given git view context. Returns a map keyed by selector.
pub fn load_file_annotations(
    svc: &mut DataService,
    file_path: &str,
    workspace_id: Option<&str>,
) -> AnnotationsByLine {
    let view_id = match workspace_id.filter(|s| !s.is_empty() && *s != "project") {
        Some(id) => format!("git:{id}"),
        None => "git".to_string(),
    };

    let schema = runtime::collections::builtin::annotations_schema();

    let mut filter = BTreeMap::new();
    filter.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
    filter.insert("view_id".to_string(), serde_json::Value::String(view_id));

    let req = QueryRequest {
        collection: ANNOTATIONS_ID.to_string(),
        filter,
        sort: {
            let mut s = BTreeMap::new();
            s.insert("created_at".to_string(), 1i8);
            s
        },
        limit: 200,
        offset: 0,
    };

    let items = match service::query_items(svc, &schema, &req) {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, file_path, "failed to load git annotations");
            return BTreeMap::new();
        }
    };

    let mut out: AnnotationsByLine = BTreeMap::new();

    for item in items {
        // Item.data holds the collection fields; item.id and item.created_at
        // are top-level.
        let data = &item.data;
        let selector = match data.get("selector").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let annotation = GitAnnotation {
            id: item.id.clone(),
            selector: selector.clone(),
            note: data
                .get("note")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            intent: data
                .get("intent")
                .and_then(|v| v.as_str())
                .map(str::to_string),
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
