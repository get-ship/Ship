//! `GET /api/v1/search` — unified entity search. Cmdk's "find" mode
//! calls this; the response mixes views, collections, collection items,
//! agents, skills, active sessions, and artifacts ranked by score.
//!
//! Per-adapter logic lives in `ship_platform::search::adapters`; this
//! file is the thin transport that snapshots every source into adapter
//! inputs, runs `aggregator::run_search`, and JSON-wraps the result in
//! the existing `{ data: … }` envelope.

use std::sync::Arc;

use axum::{Extension, Json, extract::{Query, State}, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::auth::{AuthContext};

use ship_platform::collections::query::QueryRequest;
use ship_platform::search::{
    Cursor, EntityKindFilter, SearchOpts, adapter::EntitySearchAdapter, aggregator::run_search,
};
use ship_platform::search::adapters::{
    agents::{AgentSnapshot, AgentsAdapter},
    artifacts::{ArtifactSnapshot, ArtifactsAdapter},
    collection_items::{CollectionItemSnapshot, CollectionItemsAdapter},
    collections::{CollectionSnapshot, CollectionsAdapter},
    sessions::{SessionSnapshot, SessionsAdapter},
    skills::{SkillSnapshot, SkillsAdapter},
    views::{ViewSnapshot, ViewsAdapter},
};

use crate::routes::rest_api::ApiState;
use crate::views::discovery::view_search_dirs;
use crate::views::resolve::read_raw_manifest;

/// Cap on how many items we sample per collection when building
/// collection_item snapshots. Aggregator returns the top-K so over-large
/// fan-outs are pointless; this also bounds the DB cost per search.
const ITEMS_PER_COLLECTION_CAP: u32 = 200;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: Option<String>,
    /// Comma-separated kind filter tokens.
    #[serde(default)]
    pub types: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub cursor: Option<String>,
}

pub async fn search(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<SearchQuery>,
) -> axum::response::Response {
    let query = q.q.unwrap_or_default();
    let types: Option<Vec<EntityKindFilter>> = q.types.as_deref().map(parse_types_filter);
    let cursor = q.cursor.as_deref().and_then(Cursor::decode);
    let limit = q.limit.unwrap_or(20);

    let opts = SearchOpts { types: types.clone(), limit, cursor };
    let adapters = build_adapters(&state, types.as_deref()).await;

    let resp = run_search(&adapters, &query, opts).await;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "data": resp,
        })),
    ).into_response()
}

fn parse_types_filter(raw: &str) -> Vec<EntityKindFilter> {
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(EntityKindFilter::parse)
        .collect()
}

/// Build every adapter from request-time service snapshots. Disabled
/// kinds (per the `types` allowlist) get skipped here, not just in the
/// aggregator, so we don't load data we won't search.
async fn build_adapters(
    state: &ApiState,
    types: Option<&[EntityKindFilter]>,
) -> Vec<Arc<dyn EntitySearchAdapter>> {
    let enabled = |k: EntityKindFilter| -> bool {
        types.is_none_or(|list| list.contains(&k))
    };

    let mut adapters: Vec<Arc<dyn EntitySearchAdapter>> = Vec::new();

    if enabled(EntityKindFilter::View) {
        adapters.push(Arc::new(ViewsAdapter::new(snapshot_views())));
    }
    if enabled(EntityKindFilter::Collection) {
        adapters.push(Arc::new(CollectionsAdapter::new(snapshot_collections(state))));
    }
    if enabled(EntityKindFilter::CollectionItem) {
        adapters.push(Arc::new(CollectionItemsAdapter::new(
            snapshot_collection_items(state).await,
        )));
    }
    if enabled(EntityKindFilter::Agent) {
        adapters.push(Arc::new(AgentsAdapter::new(snapshot_agents(state))));
    }
    if enabled(EntityKindFilter::Skill) {
        adapters.push(Arc::new(SkillsAdapter::new(snapshot_skills(state))));
    }
    if enabled(EntityKindFilter::Session) {
        adapters.push(Arc::new(SessionsAdapter::new(snapshot_sessions(state))));
    }
    if enabled(EntityKindFilter::Artifact) {
        adapters.push(Arc::new(ArtifactsAdapter::new(snapshot_artifacts(state).await)));
    }

    adapters
}

// ── Snapshot helpers ─────────────────────────────────────────────────

/// Views the v1 shell never surfaces to end users — wizard, samples,
/// dashboards and dev-panel surfaces not yet promoted to first-party.
/// Hidden from /api/v1/search (cmdk find mode + view-picker both
/// consume this — single source of truth). When we add a manifest
/// `hidden: true` field, this list moves to per-view metadata.
const HIDDEN_FROM_PICKERS: &[&str] = &[
    "setup",
    "sessions", "mesh", "events",
    "specs",
];

fn snapshot_views() -> Vec<ViewSnapshot> {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<ViewSnapshot> = Vec::new();
    for dir in view_search_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir.path) else { continue };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue }
            let name = entry.file_name().to_string_lossy().to_string();
            if !seen.insert(name.clone()) { continue }
            if HIDDEN_FROM_PICKERS.contains(&name.as_str()) { continue }
            // Pull rail.label as the searchable "description" + rail.icon
            // so cmdk find-mode shows each view's actual icon (not the
            // generic app-window fallback).
            let rail = read_raw_manifest(&entry.path()).and_then(|m| m.rail);
            let description = rail.as_ref().and_then(|r| r.label.clone());
            let icon = rail.and_then(|r| r.icon);
            out.push(ViewSnapshot { name, description, icon });
        }
    }
    out
}

fn snapshot_collections(state: &ApiState) -> Vec<CollectionSnapshot> {
    state
        .collections()
        .list_user_schemas()
        .unwrap_or_default()
        .into_iter()
        .map(|s| CollectionSnapshot { id: s.id, name: s.name })
        .collect()
}

async fn snapshot_collection_items(state: &ApiState) -> Vec<CollectionItemSnapshot> {
    let schemas = state.collections().list_user_schemas().unwrap_or_default();
    let mut out: Vec<CollectionItemSnapshot> = Vec::new();
    for schema in schemas {
        // Skip artifacts here — they have their own adapter and a richer
        // hit shape (mime, scope). Duplicate hits across both adapters
        // would just clutter the result.
        if schema.id == ship_platform::artifacts::ARTIFACTS_COLLECTION_ID {
            continue;
        }
        let req = QueryRequest {
            collection: schema.id.clone(),
            filter: std::collections::BTreeMap::new(),
            sort: std::collections::BTreeMap::new(),
            limit: ITEMS_PER_COLLECTION_CAP,
            offset: 0,
        };
        let Ok(items) = state.collections().query_items(&schema.id, req).await else {
            continue;
        };
        for item in items {
            let label = item
                .data
                .get("title")
                .or_else(|| item.data.get("name"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let searchable_text = stringify_data(&item.data);
            out.push(CollectionItemSnapshot {
                collection_id: schema.id.clone(),
                item_id: item.id,
                label,
                searchable_text,
            });
        }
    }
    out
}

fn snapshot_agents(state: &ApiState) -> Vec<AgentSnapshot> {
    state
        .agents()
        .list()
        .into_iter()
        .map(|e| {
            // Load each agent's doc to pull name + description. Cheap —
            // < 50 agents typical, all on local disk.
            let (name, description) = state
                .agents()
                .get(&e.id, None)
                .ok()
                .and_then(|d| {
                    let agent = d.doc.get("agent")?;
                    let name = agent.get("name").and_then(|v| v.as_str()).map(str::to_string);
                    let desc = agent
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    Some((name, desc))
                })
                .unwrap_or((None, None));
            AgentSnapshot { id: e.id, name, description }
        })
        .collect()
}

fn snapshot_skills(state: &ApiState) -> Vec<SkillSnapshot> {
    state
        .skills()
        .list()
        .into_iter()
        .map(|e| SkillSnapshot {
            id: e.id,
            name: Some(e.name),
            description: Some(e.description),
            body: None,
        })
        .collect()
}

fn snapshot_sessions(state: &ApiState) -> Vec<SessionSnapshot> {
    let mut out: Vec<SessionSnapshot> = Vec::new();
    for meta in state.pty_sessions().list() {
        if matches!(meta.status, ship_platform::services::sessions::types::SessionStatus::Ended) {
            continue;
        }
        out.push(SessionSnapshot {
            id: meta.id,
            agent_id: meta.agent_id,
            workspace_id: meta.workspace_id,
            status: format!("{:?}", meta.status).to_lowercase(),
        });
    }
    for meta in state.actor_sessions().list() {
        if matches!(meta.status, ship_platform::services::sessions::types::SessionStatus::Ended) {
            continue;
        }
        out.push(SessionSnapshot {
            id: meta.id,
            agent_id: meta.agent_id,
            workspace_id: meta.workspace_id,
            status: format!("{:?}", meta.status).to_lowercase(),
        });
    }
    out
}

async fn snapshot_artifacts(state: &ApiState) -> Vec<ArtifactSnapshot> {
    let req = QueryRequest {
        collection: ship_platform::artifacts::ARTIFACTS_COLLECTION_ID.to_string(),
        filter: std::collections::BTreeMap::new(),
        sort: std::collections::BTreeMap::new(),
        limit: 200,
        offset: 0,
    };
    state
        .artifacts()
        .list(req)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| ArtifactSnapshot {
            id: r.id,
            name: r.name,
            mime: Some(r.mime),
            scope: Some(r.scope),
        })
        .collect()
}

/// Flatten an item's `data` JSON into a single search haystack — picks
/// every string value at any depth, joined with spaces. Bounded to
/// 4 KiB per item to keep the snapshot small.
fn stringify_data(v: &serde_json::Value) -> String {
    fn walk(v: &serde_json::Value, out: &mut String) {
        if out.len() >= 4096 { return }
        match v {
            serde_json::Value::String(s) => { out.push_str(s); out.push(' '); }
            serde_json::Value::Number(n) => { out.push_str(&n.to_string()); out.push(' '); }
            serde_json::Value::Bool(b) => { out.push_str(if *b { "true " } else { "false " }); }
            serde_json::Value::Array(arr) => { for x in arr { walk(x, out); } }
            serde_json::Value::Object(map) => { for (_, x) in map { walk(x, out); } }
            serde_json::Value::Null => {}
        }
    }
    let mut s = String::new();
    walk(v, &mut s);
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_types_filter_handles_comma_list() {
        let parsed = parse_types_filter("view,agent,nope,collection_item");
        assert_eq!(parsed.len(), 3);
        assert!(parsed.contains(&EntityKindFilter::View));
        assert!(parsed.contains(&EntityKindFilter::Agent));
        assert!(parsed.contains(&EntityKindFilter::CollectionItem));
    }

    #[test]
    fn stringify_data_walks_nested() {
        let v = serde_json::json!({
            "title": "Hello",
            "tags": ["alpha", "beta"],
            "meta": { "owner": "writer" },
            "count": 3,
        });
        let s = stringify_data(&v);
        for needle in ["Hello", "alpha", "beta", "writer", "3"] {
            assert!(s.contains(needle), "missing {needle}: {s}");
        }
    }
}
