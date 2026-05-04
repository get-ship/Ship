//! SDK manifest — embedded at compile time. Parsed lazily; exposes read
//! surfaces for the per-view `/meta` endpoint, the full `/sdk` root, and
//! the fuzzy `/sdk/query` search endpoint.
//!
//! The manifest is produced by `just gen-sdk` (see `crates/xtask/`). This
//! module is strictly the daemon read-side: no writes, no regeneration.

use std::sync::OnceLock;

use axum::{Extension, Json, extract::Path as AxumPath, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth::{AuthContext, Scope, require_scope};

// ── Embedded manifest ───────────────────────────────────────────────────────

const SDK_MACROS: &str = include_str!("../../../../packages/ship-set/sdk/sdk-macros.json");
const SDK_URLS: &str = include_str!("../../../../packages/ship-set/sdk/sdk-urls.json");
const SDK_VIEW_CONTEXTS: &str =
    include_str!("../../../../packages/ship-set/sdk/sdk-view-contexts.json");
const SDK_VIEWS: &str = include_str!("../../../../packages/ship-set/sdk/sdk-views.json");
const SDK_COLLECTIONS: &str = include_str!("../../../../packages/ship-set/sdk/sdk-collections.json");
const SDK_TRIGGERS: &str = include_str!("../../../../packages/ship-set/sdk/sdk-triggers.json");

/// Parsed cache. Parse-on-first-access, reused for the life of the process.
struct Parsed {
    macros: Value,
    urls: Value,
    view_contexts: Value,
    views: Value,
    collections: Value,
    triggers: Value,
}

fn parsed() -> &'static Parsed {
    static P: OnceLock<Parsed> = OnceLock::new();
    P.get_or_init(|| Parsed {
        macros: serde_json::from_str(SDK_MACROS).unwrap_or_else(|_| Value::Null),
        urls: serde_json::from_str(SDK_URLS).unwrap_or_else(|_| Value::Null),
        view_contexts: serde_json::from_str(SDK_VIEW_CONTEXTS).unwrap_or_else(|_| Value::Null),
        views: serde_json::from_str(SDK_VIEWS).unwrap_or_else(|_| Value::Null),
        collections: serde_json::from_str(SDK_COLLECTIONS).unwrap_or_else(|_| Value::Null),
        triggers: serde_json::from_str(SDK_TRIGGERS).unwrap_or_else(|_| Value::Null),
    })
}

fn entries(v: &Value) -> Option<&serde_json::Map<String, Value>> {
    v.get("entries").and_then(|e| e.as_object())
}

// ── Types ───────────────────────────────────────────────────────────────────

#[derive(Serialize, utoipa::ToSchema)]
pub struct ActionUrl {
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ViewMeta {
    pub name: String,
    pub context: Value,
    pub manifest: Value,
    pub signals: Vec<String>,
    pub sse_topics: Vec<String>,
    pub action_urls: Vec<ActionUrl>,
    pub required_packs: Vec<String>,
    pub collections_referenced: Vec<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SdkRoot {
    pub macros: Value,
    pub routes: Value,
    pub view_contexts: Value,
    pub views: Value,
    pub collections: Value,
    pub triggers: Value,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SdkSearchHit {
    pub kind: &'static str,
    pub id: String,
    pub summary: String,
    pub path: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SdkSearchResponse {
    pub query: String,
    pub hits: Vec<SdkSearchHit>,
}

#[derive(Deserialize)]
pub struct SdkQuery {
    pub q: Option<String>,
}

// ── Extractors ──────────────────────────────────────────────────────────────

fn string_list(v: Option<&Value>) -> Vec<String> {
    v.and_then(|x| x.as_array())
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

/// Parse an endpoint key of the form "METHOD /path" into (method, path).
fn split_url_key(key: &str) -> Option<(&str, &str)> {
    let mut it = key.splitn(2, ' ');
    let m = it.next()?;
    let p = it.next()?;
    Some((m, p))
}

/// Extract action URLs referenced by a view manifest. We walk `manifest.data`
/// (each entry may have a `source` or `url`) and `manifest.on.*.do` strings
/// that look like `METHOD /path`. Anything that matches a known URL key in
/// sdk-urls is resolved to include its `purpose`.
fn extract_action_urls(manifest: &Value, urls: &Value) -> Vec<ActionUrl> {
    let mut out: Vec<ActionUrl> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    let mut push = |method: &str, path: &str| {
        let key = format!("{method} {path}");
        if !seen.insert(key.clone()) {
            return None;
        }
        let purpose = urls
            .get("entries")
            .and_then(|e| e.get(&key))
            .and_then(|e| e.get("purpose"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());
        Some(ActionUrl { method: method.to_string(), path: path.to_string(), purpose })
    };

    // Walk every string value in the manifest; look for "METHOD /path"
    // embedded as a discrete value. This is a deliberate structural sweep —
    // the sdk-views schema is still stabilizing, so we prefer a tolerant
    // extractor over a brittle key-by-key lookup.
    fn collect_strings<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
        match v {
            Value::String(s) => out.push(s.as_str()),
            Value::Array(a) => a.iter().for_each(|x| collect_strings(x, out)),
            Value::Object(o) => o.values().for_each(|x| collect_strings(x, out)),
            _ => {}
        }
    }
    let mut strings: Vec<&str> = Vec::new();
    collect_strings(manifest, &mut strings);
    for s in strings {
        if let Some((m, p)) = split_url_key(s) {
            if matches!(m, "GET" | "POST" | "PUT" | "DELETE" | "PATCH") && p.starts_with('/') {
                if let Some(au) = push(m, p) {
                    out.push(au);
                }
            }
        }
    }
    out
}

/// Collections referenced by a view: read `manifest.data.*.collection` keys
/// if present, plus any bare `collection: "name"` values anywhere in the tree.
fn extract_collections(manifest: &Value) -> Vec<String> {
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    fn walk(v: &Value, out: &mut std::collections::BTreeSet<String>) {
        match v {
            Value::Object(o) => {
                if let Some(Value::String(s)) = o.get("collection") {
                    out.insert(s.clone());
                }
                for x in o.values() {
                    walk(x, out);
                }
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    walk(manifest, &mut out);
    out.into_iter().collect()
}

/// Signals declared in the manifest (top-level `signals` object keys).
fn extract_signals(manifest: &Value) -> Vec<String> {
    manifest
        .get("signals")
        .and_then(|s| s.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

/// SSE topics — sdk-views currently emits `topics` and/or `events` arrays.
fn extract_sse_topics(manifest: &Value) -> Vec<String> {
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for key in ["topics", "events", "sse_topics"] {
        for s in string_list(manifest.get(key)) {
            out.insert(s);
        }
    }
    out.into_iter().collect()
}

// ── Handlers ────────────────────────────────────────────────────────────────

fn unauthorized_or<F: FnOnce() -> axum::response::Response>(
    auth: &AuthContext,
    otherwise: F,
) -> axum::response::Response {
    match require_scope(auth, Scope::ReadWorkspace) {
        Err(e) => e.into_response(),
        Ok(()) => otherwise(),
    }
}

/// GET /api/v1/views/{name}/meta
pub async fn get_view_meta(
    Extension(auth): Extension<AuthContext>,
    AxumPath(name): AxumPath<String>,
) -> axum::response::Response {
    unauthorized_or(&auth, || {
        let p = parsed();
        let views_entries = entries(&p.views);
        let manifest = views_entries.and_then(|e| e.get(&name));

        let Some(manifest) = manifest else {
            let available: Vec<&String> = views_entries
                .map(|m| m.keys().collect())
                .unwrap_or_default();
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "unknown view",
                    "available": available,
                })),
            )
                .into_response();
        };

        let context = entries(&p.view_contexts)
            .and_then(|e| e.get(&name))
            .cloned()
            .unwrap_or(Value::Null);

        let signals = extract_signals(manifest);
        let sse_topics = extract_sse_topics(manifest);
        let action_urls = extract_action_urls(manifest, &p.urls);
        let required_packs = string_list(manifest.get("packs"));
        let collections_referenced = extract_collections(manifest);

        Json(ViewMeta {
            name: name.clone(),
            context,
            manifest: manifest.clone(),
            signals,
            sse_topics,
            action_urls,
            required_packs,
            collections_referenced,
        })
        .into_response()
    })
}

/// GET /api/v1/sdk — full SDK root.
pub async fn get_sdk_root(
    Extension(auth): Extension<AuthContext>,
) -> axum::response::Response {
    unauthorized_or(&auth, || {
        let p = parsed();
        Json(SdkRoot {
            macros: p.macros.clone(),
            routes: p.urls.clone(),
            view_contexts: p.view_contexts.clone(),
            views: p.views.clone(),
            collections: p.collections.clone(),
            triggers: p.triggers.clone(),
        })
        .into_response()
    })
}

/// GET /api/v1/sdk/query?q=term
pub async fn query_sdk(
    Extension(auth): Extension<AuthContext>,
    axum::extract::Query(q): axum::extract::Query<SdkQuery>,
) -> axum::response::Response {
    unauthorized_or(&auth, || {
        let p = parsed();
        let query = q.q.unwrap_or_default();
        let hits = if query.trim().is_empty() {
            Vec::new()
        } else {
            search(&query, p)
        };
        Json(SdkSearchResponse { query, hits }).into_response()
    })
}

/// Case-insensitive substring match. Rank = match position (lower = better),
/// ties broken by id length. Cap at 20 results.
fn search(q: &str, p: &Parsed) -> Vec<SdkSearchHit> {
    let needle = q.to_ascii_lowercase();
    let mut ranked: Vec<(usize, usize, SdkSearchHit)> = Vec::new();

    let mut push = |kind: &'static str, id: &str, summary: String, path: String| {
        let hay = format!("{id} {summary}").to_ascii_lowercase();
        if let Some(pos) = hay.find(&needle) {
            ranked.push((
                pos,
                id.len(),
                SdkSearchHit { kind, id: id.to_string(), summary, path },
            ));
        }
    };

    if let Some(m) = entries(&p.macros) {
        for (id, v) in m {
            let summary = v
                .get("description")
                .and_then(|x| x.as_str())
                .or_else(|| v.get("signature").and_then(|x| x.as_str()))
                .unwrap_or("")
                .to_string();
            push("macro", id, summary, format!("sdk-macros.json#{id}"));
        }
    }
    if let Some(m) = entries(&p.urls) {
        for (id, v) in m {
            let summary = v
                .get("summary")
                .and_then(|x| x.as_str())
                .or_else(|| v.get("purpose").and_then(|x| x.as_str()))
                .unwrap_or("")
                .to_string();
            let path = v
                .get("path")
                .and_then(|x| x.as_str())
                .unwrap_or(id)
                .to_string();
            push("route", id, summary, path);
        }
    }
    if let Some(m) = entries(&p.views) {
        for (id, v) in m {
            let summary = v
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            push("view", id, summary, format!("/api/v1/view/{id}"));
        }
    }
    if let Some(m) = entries(&p.collections) {
        for (id, v) in m {
            let summary = v
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            push("collection", id, summary, format!("/api/v1/collections/{id}"));
        }
    }
    if let Some(m) = entries(&p.triggers) {
        for (id, v) in m {
            let summary = v
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            push("trigger", id, summary, format!("sdk-triggers.json#{id}"));
        }
    }

    ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    ranked.into_iter().take(20).map(|(_, _, h)| h).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded_manifest() {
        let p = parsed();
        // sanity: at least one known view exists
        assert!(entries(&p.views).is_some(), "sdk-views entries present");
    }

    #[test]
    fn extracts_signals_and_packs() {
        let manifest = serde_json::json!({
            "packs": ["git", "editor"],
            "signals": {"q": {"type": "string"}, "n": {"type": "number"}},
        });
        let signals = extract_signals(&manifest);
        assert!(signals.contains(&"q".to_string()));
        assert!(signals.contains(&"n".to_string()));
        assert_eq!(string_list(manifest.get("packs")).len(), 2);
    }

    #[test]
    fn extracts_collections_deeply() {
        let m = serde_json::json!({
            "data": {"items": {"collection": "tasks"}},
            "other": [{"collection": "notes"}],
        });
        let c = extract_collections(&m);
        assert_eq!(c, vec!["notes".to_string(), "tasks".to_string()]);
    }

    #[test]
    fn extracts_action_urls_from_strings() {
        let m = serde_json::json!({
            "on": {"click": {"do": "POST /api/v1/collections/tasks/items"}},
        });
        let urls = serde_json::json!({"entries": {
            "POST /api/v1/collections/tasks/items": {"purpose": "create-item"}
        }});
        let out = extract_action_urls(&m, &urls);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].method, "POST");
        assert_eq!(out[0].purpose.as_deref(), Some("create-item"));
    }

    #[test]
    fn search_empty_query_returns_none() {
        let p = parsed();
        assert!(search("", p).is_empty());
    }

    #[test]
    fn search_caps_at_20() {
        let p = parsed();
        // "a" is very common in descriptions — result set should cap at 20.
        let hits = search("a", p);
        assert!(hits.len() <= 20);
    }
}
