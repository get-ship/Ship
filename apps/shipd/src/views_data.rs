//! Data browser view — schema-aware collection viewer with SSE live updates.
//!
//! Two endpoints:
//!
//! - `GET /api/runtime/view/data` — full-page shell with collection picker
//!   and table. Rendered via minijinja from templates/data/.
//! - `GET /api/runtime/view/data/sse` — Datastar SSE stream that pushes
//!   `#data-content` fragments when collection items change.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use runtime::collections::query::QueryRequest;
use runtime::collections::schema::{self, CollectionSchema};
use runtime::collections::store;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;
use crate::rest_api::ApiState;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DataParams {
    #[serde(default)]
    pub collection: Option<String>,
}

#[derive(Serialize)]
struct DataContext {
    collections: Vec<CollectionSummary>,
    active_collection: String,
    schema: Option<SchemaContext>,
    items: Vec<serde_json::Value>,
    total_items: usize,
}

#[derive(Serialize)]
struct CollectionSummary {
    id: String,
    name: Option<String>,
    field_count: usize,
}

#[derive(Serialize)]
struct SchemaContext {
    id: String,
    fields: BTreeMap<String, FieldContext>,
}

#[derive(Serialize)]
struct FieldContext {
    field_type: String,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<String>>,
}

fn build_context(ship_dir: &std::path::Path, collection: Option<&str>) -> DataContext {
    let schemas = schema::load_schemas(ship_dir).unwrap_or_default();

    let collections: Vec<CollectionSummary> = schemas
        .iter()
        .map(|s| CollectionSummary {
            id: s.id.clone(),
            name: s.name.clone(),
            field_count: s.fields.len(),
        })
        .collect();

    let active = collection
        .map(str::to_string)
        .or_else(|| collections.first().map(|c| c.id.clone()))
        .unwrap_or_default();

    let active_schema = schemas.iter().find(|s| s.id == active);

    let (schema_ctx, items) = match active_schema {
        Some(s) => {
            let schema_ctx = SchemaContext {
                id: s.id.clone(),
                fields: s
                    .fields
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            FieldContext {
                                field_type: format!("{:?}", v.field_type).to_lowercase(),
                                required: v.required,
                                values: v.values.clone(),
                            },
                        )
                    })
                    .collect(),
            };
            let items = load_items(ship_dir, s);
            (Some(schema_ctx), items)
        }
        None => (None, vec![]),
    };

    let total_items = items.len();
    DataContext {
        collections,
        active_collection: active,
        schema: schema_ctx,
        items,
        total_items,
    }
}

fn load_items(ship_dir: &std::path::Path, schema: &CollectionSchema) -> Vec<serde_json::Value> {
    let mut svc = match store::open_docs_db(ship_dir) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let req = QueryRequest {
        collection: schema.id.clone(),
        filter: BTreeMap::new(),
        sort: BTreeMap::new(),
        limit: 200,
        offset: 0,
    };
    match store::query_items(&mut svc, schema, &req) {
        Ok(items) => items.into_iter().map(|i| serde_json::to_value(i).unwrap_or_default()).collect(),
        Err(_) => vec![],
    }
}

/// GET /api/runtime/view/data — render the data browser shell.
pub async fn data_shell(
    State(state): State<ApiState>,
    Query(params): Query<DataParams>,
) -> Result<axum::response::Html<String>, (StatusCode, String)> {
    let ctx = build_context(state.ship_dir(), params.collection.as_deref());
    templates::render("data/shell.html.j2", &ctx)
        .map(axum::response::Html)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// GET /api/runtime/view/data/sse — SSE stream for live collection updates.
pub async fn data_sse(
    State(state): State<ApiState>,
    Query(params): Query<DataParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();
    let collection = params.collection;

    tokio::spawn(async move {
        // Initial content render.
        let ctx = build_context(&ship_dir, collection.as_deref());
        let html = match templates::render("data/content.html.j2", &ctx) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("data view: template render failed: {e}");
                return;
            }
        };
        if vs.patch("#data-content", html).await.is_err() {
            return;
        }

        // Subscribe to collection events and re-render on changes.
        let kernel = state.kernel().clone();
        let (_, mut mailbox) = {
            let config = runtime::events::ActorConfig {
                namespace: "view.data".to_string(),
                write_namespaces: vec![],
                read_namespaces: vec!["collection.".to_string()],
                subscribe_namespaces: vec!["collection.".to_string()],
            };
            match kernel.lock().await.spawn_actor("view.data", config) {
                Ok(pair) => pair,
                Err(e) => {
                    tracing::warn!("data view: failed to spawn actor: {e}");
                    return;
                }
            }
        };

        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        loop {
            match tokio::time::timeout(Duration::from_secs(30), mailbox.recv()).await {
                Ok(Some(event)) => {
                    // Check if this event is for our active collection.
                    if let Some(ref col) = collection {
                        let payload_str = &event.payload_json;
                        if !payload_str.contains(col) {
                            continue;
                        }
                    }

                    // Re-render full content on any change.
                    let ctx = build_context(&ship_dir, collection.as_deref());
                    let html = match templates::render("data/content.html.j2", &ctx) {
                        Ok(h) => h,
                        Err(e) => {
                            tracing::error!("data view: re-render failed: {e}");
                            continue;
                        }
                    };
                    if vs.patch("#data-content", html).await.is_err() {
                        break;
                    }
                }
                Ok(None) => break, // Mailbox closed.
                Err(_) => {
                    // Timeout — no events. Send heartbeat (handled by _hb).
                    continue;
                }
            }
        }

        // Cleanup: stop the ephemeral actor.
        let _ = kernel.lock().await.stop_actor("view.data");
    });

    sse
}
