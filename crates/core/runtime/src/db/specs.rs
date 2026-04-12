//! Spec CRUD — platform DB backing store for the Specs view.
//!
//! Body is stored on disk at `body_path` (typically `.artifacts/specs/<id>.md`).
//! Metadata (title, status, pipeline, vars, criteria, scope) lives in the DB.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::collections::HashMap;
use std::path::Path;

use crate::db::{block_on, open_db};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub body_path: String,
    pub status: String,
    #[serde(default)]
    pub pipeline: Option<String>,
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

fn row_to_spec(row: sqlx::sqlite::SqliteRow) -> Result<Spec> {
    let vars_json: String = row.try_get("vars_json")?;
    let criteria_json: String = row.try_get("criteria_json")?;
    let scope_json: String = row.try_get("scope_json")?;
    let created_at: String = row.try_get("created_at")?;
    let updated_at: String = row.try_get("updated_at")?;
    let priority: Option<i64> = row.try_get("priority").ok();

    Ok(Spec {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        body_path: row.try_get("body_path")?,
        status: row.try_get("status")?,
        pipeline: row.try_get("pipeline").ok(),
        vars: serde_json::from_str(&vars_json).unwrap_or_default(),
        criteria: serde_json::from_str(&criteria_json).unwrap_or_default(),
        scope: serde_json::from_str(&scope_json).unwrap_or_default(),
        priority: priority.map(|p| p as u32),
        created_at: DateTime::parse_from_rfc3339(&created_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

pub fn list_specs() -> Result<Vec<Spec>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, title, body_path, status, pipeline, vars_json, criteria_json,
                    scope_json, priority, created_at, updated_at
             FROM spec ORDER BY updated_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    rows.into_iter().map(row_to_spec).collect()
}

pub fn get_spec(id: &str) -> Result<Option<Spec>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, title, body_path, status, pipeline, vars_json, criteria_json,
                    scope_json, priority, created_at, updated_at
             FROM spec WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    match row {
        Some(r) => Ok(Some(row_to_spec(r)?)),
        None => Ok(None),
    }
}

pub fn upsert_spec(spec: &Spec) -> Result<()> {
    let mut conn = open_db()?;
    let vars_json = serde_json::to_string(&spec.vars)?;
    let criteria_json = serde_json::to_string(&spec.criteria)?;
    let scope_json = serde_json::to_string(&spec.scope)?;
    let created = spec.created_at.to_rfc3339();
    let updated = spec.updated_at.to_rfc3339();
    let priority = spec.priority.map(|p| p as i64);
    block_on(async {
        sqlx::query(
            "INSERT INTO spec (id, title, body_path, status, pipeline, vars_json,
                               criteria_json, scope_json, priority, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title,
               body_path = excluded.body_path,
               status = excluded.status,
               pipeline = excluded.pipeline,
               vars_json = excluded.vars_json,
               criteria_json = excluded.criteria_json,
               scope_json = excluded.scope_json,
               priority = excluded.priority,
               updated_at = excluded.updated_at",
        )
        .bind(&spec.id)
        .bind(&spec.title)
        .bind(&spec.body_path)
        .bind(&spec.status)
        .bind(&spec.pipeline)
        .bind(&vars_json)
        .bind(&criteria_json)
        .bind(&scope_json)
        .bind(priority)
        .bind(&created)
        .bind(&updated)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn delete_spec(id: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("DELETE FROM spec WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

/// Read the markdown body from disk. `body_path` is resolved relative to the
/// project root when not absolute.
pub fn load_body(body_path: &str) -> Result<String> {
    let p = resolve_body_path(body_path)?;
    std::fs::read_to_string(&p)
        .with_context(|| format!("read spec body at {}", p.display()))
}

/// Write the markdown body to disk. Creates parent directories as needed.
pub fn save_body(body_path: &str, body: &str) -> Result<()> {
    let p = resolve_body_path(body_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create spec dir {}", parent.display()))?;
    }
    std::fs::write(&p, body)
        .with_context(|| format!("write spec body at {}", p.display()))?;
    Ok(())
}

fn resolve_body_path(body_path: &str) -> Result<std::path::PathBuf> {
    let p = Path::new(body_path);
    if p.is_absolute() {
        return Ok(p.to_path_buf());
    }
    let project_dir = crate::project::get_project_dir(None)?;
    // project_dir is the .ship dir — artifacts live in its parent (project root).
    let project_root = project_dir.parent().unwrap_or(&project_dir);
    Ok(project_root.join(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::init_project;
    use tempfile::tempdir;

    fn sample(id: &str) -> Spec {
        let now = Utc::now();
        let mut vars = HashMap::new();
        vars.insert("k".into(), "v".into());
        Spec {
            id: id.into(),
            title: "T".into(),
            body_path: format!(".artifacts/specs/{id}.md"),
            status: "draft".into(),
            pipeline: Some("tdd".into()),
            vars,
            criteria: vec!["c1".into()],
            scope: vec!["crates/core/".into()],
            priority: Some(3),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn crud_roundtrip() {
        let tmp = tempdir().unwrap();
        let _ = init_project(tmp.path().to_path_buf()).unwrap();
        crate::db::ensure_db().unwrap();

        let s = sample("s1");
        upsert_spec(&s).unwrap();

        let got = get_spec("s1").unwrap().unwrap();
        assert_eq!(got.title, "T");
        assert_eq!(got.pipeline.as_deref(), Some("tdd"));
        assert_eq!(got.criteria, vec!["c1".to_string()]);
        assert_eq!(got.vars.get("k").map(|v| v.as_str()), Some("v"));
        assert_eq!(got.priority, Some(3));

        let all = list_specs().unwrap();
        assert_eq!(all.len(), 1);

        let mut updated = got.clone();
        updated.title = "T2".into();
        updated.updated_at = Utc::now();
        upsert_spec(&updated).unwrap();
        assert_eq!(get_spec("s1").unwrap().unwrap().title, "T2");

        delete_spec("s1").unwrap();
        assert!(get_spec("s1").unwrap().is_none());
    }

    #[test]
    fn body_io_roundtrip() {
        let tmp = tempdir().unwrap();
        let _ = init_project(tmp.path().to_path_buf()).unwrap();

        save_body(".artifacts/specs/abc.md", "# Hello\nbody").unwrap();
        let back = load_body(".artifacts/specs/abc.md").unwrap();
        assert_eq!(back, "# Hello\nbody");
    }

    #[test]
    fn load_missing_body_errors() {
        let tmp = tempdir().unwrap();
        let _ = init_project(tmp.path().to_path_buf()).unwrap();
        assert!(load_body(".artifacts/specs/missing.md").is_err());
    }
}
