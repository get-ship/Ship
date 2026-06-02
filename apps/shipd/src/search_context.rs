//! `ShipdSearchContext` — shipd's implementation of `ship_search::SearchContext`.
//!
//! Built once per Connect request from `ApiState` (which carries all service
//! handles). Every snapshot method returns data already filtered by what the
//! daemon can see (runtime state + DB rows the caller's token can reach).
//! The search service performs no further permission checks.

use std::collections::HashSet;

use async_trait::async_trait;
use ship_substrate::ports::Query;
use ship_search::SearchContext;
use ship_search::adapters::{
    agents::AgentSnapshot,
    artifacts::ArtifactSnapshot,
    collection_items::CollectionItemSnapshot,
    collections::CollectionSnapshot,
    sessions::SessionSnapshot,
    skills::SkillSnapshot,
    views::ViewSnapshot,
};

use crate::ship_runtime::ApiState;

/// Per-request search scope. Holds the runtime handle; snapshots are lazy.
pub struct ShipdSearchContext {
    state: ApiState,
}

impl ShipdSearchContext {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

/// Views hidden from user-facing pickers.
const HIDDEN_FROM_PICKERS: &[&str] = &["setup", "sessions", "mesh", "events", "specs"];

/// Cap on items sampled per collection.
const ITEMS_PER_COLLECTION_CAP: u32 = 200;

#[async_trait]
impl SearchContext for ShipdSearchContext {
    fn views(&self) -> Vec<ViewSnapshot> {
        use crate::views::discovery::view_search_dirs;
        use crate::views::resolve::read_raw_manifest;

        let mut seen: HashSet<String> = HashSet::new();
        let mut out: Vec<ViewSnapshot> = Vec::new();
        for dir in view_search_dirs() {
            let Ok(entries) = std::fs::read_dir(&dir.path) else { continue };
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if !seen.insert(name.clone()) {
                    continue;
                }
                if HIDDEN_FROM_PICKERS.contains(&name.as_str()) {
                    continue;
                }
                let rail = read_raw_manifest(&entry.path()).and_then(|m| m.rail);
                let description = rail.as_ref().and_then(|r| r.label.clone());
                let icon = rail.and_then(|r| r.icon);
                out.push(ViewSnapshot { name, description, icon });
            }
        }
        out
    }

    fn collections(&self) -> Vec<CollectionSnapshot> {
        self.state
            .collections()
            .list()
            .into_iter()
            .filter(|s| !s.system)
            .map(|s| CollectionSnapshot { id: s.id, name: s.title })
            .collect()
    }

    fn agents(&self) -> Vec<AgentSnapshot> {
        self.state
            .agents()
            .list()
            .into_iter()
            .map(|e| {
                let (name, description) = self
                    .state
                    .agents()
                    .get(&e.id, None)
                    .ok()
                    .and_then(|d| {
                        let agent = d.doc.get("agent")?;
                        let name = agent
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(str::to_string);
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

    fn skills(&self) -> Vec<SkillSnapshot> {
        self.state
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

    fn sessions(&self) -> Vec<SessionSnapshot> {
        use ship_platform::services::sessions::types::SessionStatus;
        let mut out: Vec<SessionSnapshot> = Vec::new();
        for meta in self.state.pty_sessions().list() {
            if matches!(meta.status, SessionStatus::Ended) {
                continue;
            }
            out.push(SessionSnapshot {
                id: meta.id,
                agent_id: meta.agent_id,
                workspace_id: meta.workspace_id,
                status: format!("{:?}", meta.status).to_lowercase(),
            });
        }
        for meta in self.state.actor_sessions().list() {
            if matches!(meta.status, SessionStatus::Ended) {
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

    async fn collection_items(&self) -> Vec<CollectionItemSnapshot> {
        let svc = self.state.collections();
        let schemas: Vec<_> = svc.list().into_iter().filter(|s| !s.system).collect();
        let mut out: Vec<CollectionItemSnapshot> = Vec::new();
        for schema in schemas {
            let q = Query {
                table: format!("collection_{}", schema.id),
                limit: ITEMS_PER_COLLECTION_CAP,
                ..Default::default()
            };
            let Ok((items, _)) = svc.query_items_by_query(q).await else {
                continue;
            };
            for item in items {
                let label = item
                    .data
                    .get("title")
                    .or_else(|| item.data.get("name"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let searchable_text =
            stringify_data(&serde_json::Value::Object(item.data.clone()));
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

    async fn artifacts(&self) -> Vec<ArtifactSnapshot> {
        self.state
            .artifacts()
            .list(std::collections::BTreeMap::new(), 200)
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
}

/// Flatten an item's JSON data into a single search haystack. Bounded to 4 KiB.
fn stringify_data(v: &serde_json::Value) -> String {
    fn walk(v: &serde_json::Value, out: &mut String) {
        if out.len() >= 4096 {
            return;
        }
        match v {
            serde_json::Value::String(s) => {
                out.push_str(s);
                out.push(' ');
            }
            serde_json::Value::Number(n) => {
                out.push_str(&n.to_string());
                out.push(' ');
            }
            serde_json::Value::Bool(b) => {
                out.push_str(if *b { "true " } else { "false " });
            }
            serde_json::Value::Array(arr) => {
                for x in arr {
                    walk(x, out);
                }
            }
            serde_json::Value::Object(map) => {
                for (_, x) in map {
                    walk(x, out);
                }
            }
            serde_json::Value::Null => {}
        }
    }
    let mut s = String::new();
    walk(v, &mut s);
    s.trim_end().to_string()
}
