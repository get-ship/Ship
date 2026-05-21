//! View shape for the `specs` collection: a typed projection of a
//! `store::Item` plus body-file helpers. Pure data — no service trait,
//! no DB locking. Routes and registry build their own queries against
//! `CollectionsService` / `ship_platform::collections::store` and pipe items
//! through `spec_from_item` to get this shape.
//!
//! Body content lives on disk at `.artifacts/specs/<id>.md`; the path
//! is held in the item's `body_path` field. Disk I/O is the caller's
//! responsibility — `save_body` / `load_body` are convenience wrappers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ship_platform::collections::store;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub body_path: String,
    pub status: String,
    #[serde(default)]
    pub pipeline: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub priority: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn spec_from_item(item: store::Item) -> Spec {
    let store::Item { id, data, created_at, updated_at, .. } = item;
    let title = string_field(&data, "title").unwrap_or_default();
    let body_path = string_field(&data, "body_path").unwrap_or_default();
    let status = string_field(&data, "status").unwrap_or_else(|| "draft".into());
    let pipeline = string_field(&data, "pipeline");
    let agent = string_field(&data, "agent");
    let vars = data
        .get("vars")
        .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v.clone()).ok())
        .unwrap_or_default();
    let criteria = data
        .get("criteria")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let scope = data
        .get("scope")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let priority = data
        .get("priority")
        .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
        .map(|p| p as u32);
    Spec {
        id,
        title,
        body_path,
        status,
        pipeline,
        agent,
        vars,
        criteria,
        scope,
        priority,
        created_at: parse_ts(&created_at),
        updated_at: parse_ts(&updated_at),
    }
}

fn string_field(data: &serde_json::Value, key: &str) -> Option<String> {
    data.get(key)
        .and_then(|v| v.as_str())
        .map(String::from)
        .filter(|s| !s.is_empty())
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Default body location for a new spec.
pub fn default_body_path(id: &str) -> String {
    format!(".artifacts/specs/{id}.md")
}

/// Resolve a `body_path` (relative or absolute) to an on-disk path.
/// Relative paths resolve against the project root (parent of `.ship/`).
pub fn resolve_body_path(body_path: &str) -> std::io::Result<PathBuf> {
    let p = Path::new(body_path);
    if p.is_absolute() {
        return Ok(p.to_path_buf());
    }
    let project_ship_dir = ship_substrate::project::get_project_dir(None)
        .map_err(|e| std::io::Error::other(format!("resolve project dir: {e}")))?;
    let project_root = project_ship_dir.parent().unwrap_or(&project_ship_dir);
    Ok(project_root.join(p))
}

pub fn save_body(body_path: &str, body: &str) -> std::io::Result<()> {
    let p = resolve_body_path(body_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, body)
}

pub fn load_body(body_path: &str) -> std::io::Result<String> {
    let p = resolve_body_path(body_path)?;
    std::fs::read_to_string(&p)
}

/// Build the JSON `data` blob for a new spec item.
#[allow(clippy::too_many_arguments)]
pub fn build_create_data(
    title: &str,
    body_path: &str,
    status: Option<&str>,
    pipeline: Option<&str>,
    agent: Option<&str>,
    vars: &HashMap<String, String>,
    criteria: &[String],
    scope: &[String],
    priority: Option<u32>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("title".into(), serde_json::Value::String(title.to_string()));
    map.insert("body_path".into(), serde_json::Value::String(body_path.to_string()));
    map.insert(
        "status".into(),
        serde_json::Value::String(status.unwrap_or("draft").to_string()),
    );
    if let Some(p) = pipeline {
        map.insert("pipeline".into(), serde_json::Value::String(p.to_string()));
    }
    if let Some(a) = agent {
        map.insert("agent".into(), serde_json::Value::String(a.to_string()));
    }
    map.insert("vars".into(), serde_json::to_value(vars).unwrap_or_default());
    map.insert("criteria".into(), serde_json::to_value(criteria).unwrap_or_default());
    map.insert("scope".into(), serde_json::to_value(scope).unwrap_or_default());
    if let Some(p) = priority {
        map.insert("priority".into(), serde_json::json!(p));
    }
    serde_json::Value::Object(map)
}

/// Build a partial-update JSON: only fields that are `Some` are written.
/// `pipeline: Some(None)` clears the field; `pipeline: None` leaves it
/// untouched. Same for `priority`.
#[allow(clippy::too_many_arguments)]
pub fn build_update_data(
    title: Option<&str>,
    status: Option<&str>,
    pipeline: Option<Option<&str>>,
    agent: Option<Option<&str>>,
    vars: Option<&HashMap<String, String>>,
    criteria: Option<&[String]>,
    scope: Option<&[String]>,
    priority: Option<Option<u32>>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(t) = title {
        map.insert("title".into(), serde_json::Value::String(t.to_string()));
    }
    if let Some(s) = status {
        map.insert("status".into(), serde_json::Value::String(s.to_string()));
    }
    if let Some(p) = pipeline {
        map.insert(
            "pipeline".into(),
            p.map(|s| serde_json::Value::String(s.to_string()))
                .unwrap_or(serde_json::Value::Null),
        );
    }
    if let Some(a) = agent {
        map.insert(
            "agent".into(),
            a.map(|s| serde_json::Value::String(s.to_string()))
                .unwrap_or(serde_json::Value::Null),
        );
    }
    if let Some(v) = vars {
        map.insert("vars".into(), serde_json::to_value(v).unwrap_or_default());
    }
    if let Some(c) = criteria {
        map.insert("criteria".into(), serde_json::to_value(c).unwrap_or_default());
    }
    if let Some(s) = scope {
        map.insert("scope".into(), serde_json::to_value(s).unwrap_or_default());
    }
    if let Some(p) = priority {
        map.insert(
            "priority".into(),
            p.map(|n| serde_json::json!(n)).unwrap_or(serde_json::Value::Null),
        );
    }
    serde_json::Value::Object(map)
}
