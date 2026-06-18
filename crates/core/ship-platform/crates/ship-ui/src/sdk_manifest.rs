//! SDK manifest — embedded at compile time. Parsed lazily; exposes read
//! surfaces for the per-view `/meta` endpoint, the full `/sdk` root, and
//! the fuzzy `/sdk/query` search endpoint.
//!
//! Relocated from `apps/shipd/src/routes/sdk_manifest.rs` (Batch C routes-cull).
//! No daemon state required — pure read over embedded JSON.
//!
//! The manifest is produced by `just gen-sdk` (see `crates/xtask/`). This
//! module is strictly the read-side: no writes, no regeneration.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Embedded manifest ─────────────────────────────────────────────────────────
//
// Paths are relative to this source file, which lives at:
//   crates/core/ship-platform/crates/ship-ui/src/sdk_manifest.rs
// Six levels up reaches the workspace root.

const SDK_URLS: &str = include_str!(
    "../../../../../../packages/deck/sdk/src/generated/sdk-urls.json"
);
const SDK_VIEW_CONTEXTS: &str = include_str!(
    "../../../../../../packages/deck/sdk/src/generated/sdk-view-contexts.json"
);
const SDK_VIEWS: &str = include_str!(
    "../../../../../../packages/deck/sdk/src/generated/sdk-views.json"
);
const SDK_COLLECTIONS: &str = include_str!(
    "../../../../../../packages/deck/sdk/src/generated/sdk-collections.json"
);
const SDK_TRIGGERS: &str = include_str!(
    "../../../../../../packages/deck/sdk/src/generated/sdk-triggers.json"
);

struct Parsed {
    urls: Value,
    view_contexts: Value,
    views: Value,
    collections: Value,
    triggers: Value,
}

fn parsed() -> &'static Parsed {
    static P: OnceLock<Parsed> = OnceLock::new();
    P.get_or_init(|| Parsed {
        urls: serde_json::from_str(SDK_URLS).unwrap_or(Value::Null),
        view_contexts: serde_json::from_str(SDK_VIEW_CONTEXTS).unwrap_or(Value::Null),
        views: serde_json::from_str(SDK_VIEWS).unwrap_or(Value::Null),
        collections: serde_json::from_str(SDK_COLLECTIONS).unwrap_or(Value::Null),
        triggers: serde_json::from_str(SDK_TRIGGERS).unwrap_or(Value::Null),
    })
}

fn entries(v: &Value) -> Option<&serde_json::Map<String, Value>> {
    v.get("entries").and_then(|e| e.as_object())
}

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ActionUrl {
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct SdkRoot {
    pub routes: Value,
    pub view_contexts: Value,
    pub views: Value,
    pub collections: Value,
    pub triggers: Value,
}

#[derive(Serialize)]
pub struct SdkSearchHit {
    pub kind: &'static str,
    pub id: String,
    pub summary: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct SdkSearchResponse {
    pub query: String,
    pub hits: Vec<SdkSearchHit>,
}

#[derive(Deserialize)]
pub struct SdkQuery {
    pub q: Option<String>,
}

/// Not-found response shape for `get_view_meta`.
pub struct ViewMetaNotFound {
    pub available: Vec<String>,
}

// ── Domain functions ──────────────────────────────────────────────────────────

/// Look up a view by name and return its `ViewMeta`. Returns `None` when the
/// view is not in the manifest; callers map this to a 404 + available list.
pub fn view_meta(name: &str) -> Result<ViewMeta, ViewMetaNotFound> {
    let p = parsed();
    let views_entries = entries(&p.views);
    let manifest = views_entries.and_then(|e| e.get(name));

    let Some(manifest) = manifest else {
        return Err(ViewMetaNotFound {
            available: views_entries
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default(),
        });
    };

    let context = entries(&p.view_contexts)
        .and_then(|e| e.get(name))
        .cloned()
        .unwrap_or(Value::Null);

    Ok(ViewMeta {
        name: name.to_string(),
        context,
        manifest: manifest.clone(),
        signals: extract_signals(manifest),
        sse_topics: extract_sse_topics(manifest),
        action_urls: extract_action_urls(manifest, &p.urls),
        required_packs: string_list(manifest.get("packs")),
        collections_referenced: extract_collections(manifest),
    })
}

/// Return the full SDK root (all embedded catalogs).
pub fn sdk_root() -> SdkRoot {
    let p = parsed();
    SdkRoot {
        routes: p.urls.clone(),
        view_contexts: p.view_contexts.clone(),
        views: p.views.clone(),
        collections: p.collections.clone(),
        triggers: p.triggers.clone(),
    }
}

/// Fuzzy search across routes, views, collections, and triggers.
/// Returns up to 20 results ranked by match position.
pub fn query_sdk(q: Option<String>) -> SdkSearchResponse {
    let query = q.unwrap_or_default();
    let hits = if query.trim().is_empty() { Vec::new() } else { search(&query, parsed()) };
    SdkSearchResponse { query, hits }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn string_list(v: Option<&Value>) -> Vec<String> {
    v.and_then(|x| x.as_array())
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

fn split_url_key(key: &str) -> Option<(&str, &str)> {
    key.split_once(' ')
}

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
        if let Some((m, p)) = split_url_key(s)
            && matches!(m, "GET" | "POST" | "PUT" | "DELETE" | "PATCH")
            && p.starts_with('/')
            && let Some(au) = push(m, p)
        {
            out.push(au);
        }
    }
    out
}

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

fn extract_signals(manifest: &Value) -> Vec<String> {
    manifest
        .get("signals")
        .and_then(|s| s.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

fn extract_sse_topics(manifest: &Value) -> Vec<String> {
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for key in ["topics", "events", "sse_topics"] {
        for s in string_list(manifest.get(key)) {
            out.insert(s);
        }
    }
    out.into_iter().collect()
}

fn search(q: &str, p: &Parsed) -> Vec<SdkSearchHit> {
    let needle = q.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut ranked: Vec<(usize, usize, SdkSearchHit)> = Vec::new();

    let mut push = |kind: &'static str, id: &str, summary: String, path: String| {
        let hay = format!("{id} {summary}").to_ascii_lowercase();
        if let Some(pos) = hay.find(&needle) {
            ranked.push((pos, id.len(), SdkSearchHit { kind, id: id.to_string(), summary, path }));
        }
    };

    if let Some(m) = entries(&p.urls) {
        for (id, v) in m {
            let summary = v
                .get("summary")
                .and_then(|x| x.as_str())
                .or_else(|| v.get("purpose").and_then(|x| x.as_str()))
                .unwrap_or("")
                .to_string();
            let path =
                v.get("path").and_then(|x| x.as_str()).unwrap_or(id).to_string();
            push("route", id, summary, path);
        }
    }
    if let Some(m) = entries(&p.views) {
        for (id, v) in m {
            let summary =
                v.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string();
            push("view", id, summary, format!("/api/v1/view/{id}"));
        }
    }
    if let Some(m) = entries(&p.collections) {
        for (id, v) in m {
            let summary =
                v.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string();
            push("collection", id, summary, format!("/api/v1/collections/{id}"));
        }
    }
    if let Some(m) = entries(&p.triggers) {
        for (id, v) in m {
            let summary =
                v.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string();
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
        let hits = search("a", p);
        assert!(hits.len() <= 20);
    }
}
