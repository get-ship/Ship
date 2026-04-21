//! Per-table migration helpers — notes, ADRs, specs from `platform.db`.
//!
//! All functions are `pub(super)` — consumed only by `migration.rs`.

use anyhow::Result;
use sqlx::{Row, SqliteConnection};

use super::schema::CollectionSchema;
use super::store;
use crate::data::{DataService, SqlParam};

// ── Notes ────────────────────────────────────────────────────────────────────

pub(super) fn migrate_notes(
    svc: &mut DataService,
    schema: &CollectionSchema,
    platform: &mut SqliteConnection,
) -> Result<usize> {
    if !table_exists(platform, "note")? {
        return Ok(0);
    }
    let rows = crate::db::block_on(async {
        sqlx::query("SELECT id, title, content, tags_json, branch FROM note")
            .fetch_all(&mut *platform)
            .await
    })?;
    let mut migrated = 0;
    for row in rows {
        let id: String = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let content: String = row.try_get("content").unwrap_or_default();
        let tags_json: String = row.try_get("tags_json").unwrap_or_else(|_| "[]".into());
        let branch: Option<String> = row.try_get("branch").ok();
        let tags = serde_json::from_str::<serde_json::Value>(&tags_json)
            .unwrap_or_else(|_| serde_json::json!([]));
        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        if !content.is_empty() {
            data.insert("content".into(), serde_json::Value::String(content));
        }
        data.insert("tags".into(), tags);
        if let Some(b) = branch {
            if !b.is_empty() {
                data.insert("category".into(), serde_json::Value::String(b));
            }
        }
        if item_exists(svc, &schema.id, &id)? {
            continue;
        }
        store::create_item(svc, schema, &id, &serde_json::Value::Object(data), None)?;
        migrated += 1;
    }
    Ok(migrated)
}

// ── ADRs ─────────────────────────────────────────────────────────────────────

pub(super) fn migrate_adrs(
    svc: &mut DataService,
    schema: &CollectionSchema,
    platform: &mut SqliteConnection,
) -> Result<usize> {
    if !table_exists(platform, "adr")? {
        return Ok(0);
    }
    let rows = crate::db::block_on(async {
        sqlx::query("SELECT id, title, status, context, decision, tags_json FROM adr")
            .fetch_all(&mut *platform)
            .await
    })?;
    let mut migrated = 0;
    for row in rows {
        let id: String = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let status: String = row.try_get("status").unwrap_or_else(|_| "proposed".into());
        let context: String = row.try_get("context").unwrap_or_default();
        let decision: String = row.try_get("decision").unwrap_or_default();
        let tags_json: String = row.try_get("tags_json").unwrap_or_else(|_| "[]".into());
        let tags = serde_json::from_str::<serde_json::Value>(&tags_json)
            .unwrap_or_else(|_| serde_json::json!([]));
        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        data.insert("status".into(), serde_json::Value::String(map_adr_status(&status)));
        if !context.is_empty() {
            data.insert("context".into(), serde_json::Value::String(context));
        }
        if !decision.is_empty() {
            data.insert("decision".into(), serde_json::Value::String(decision));
        }
        data.insert("tags".into(), tags);
        if item_exists(svc, &schema.id, &id)? {
            continue;
        }
        store::create_item(svc, schema, &id, &serde_json::Value::Object(data), None)?;
        migrated += 1;
    }
    Ok(migrated)
}

pub(super) fn map_adr_status(raw: &str) -> String {
    match raw.to_ascii_lowercase().as_str() {
        "proposed" | "draft" => "proposed",
        "accepted" | "active" | "final" => "accepted",
        "deprecated" | "retired" => "deprecated",
        "superseded" | "replaced" => "superseded",
        _ => "proposed",
    }
    .to_string()
}

// ── Specs (target + capability) ──────────────────────────────────────────────

pub(super) fn migrate_specs(
    svc: &mut DataService,
    schema: &CollectionSchema,
    platform: &mut SqliteConnection,
) -> Result<usize> {
    if !table_exists(platform, "target")? {
        return Ok(0);
    }
    let rows = crate::db::block_on(async {
        sqlx::query(
            "SELECT id, title, description, status, body_markdown, file_scope_json FROM target",
        )
        .fetch_all(&mut *platform)
        .await
    })?;
    let has_capabilities = table_exists(platform, "capability")?;
    let mut migrated = 0;
    for row in rows {
        let id: String = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let description: Option<String> = row.try_get("description").ok();
        let status: String = row.try_get("status").unwrap_or_else(|_| "draft".into());
        let body_markdown: Option<String> = row.try_get("body_markdown").ok();
        let file_scope_json: Option<String> = row.try_get("file_scope_json").ok();
        let acceptance = if has_capabilities {
            read_capability_titles(platform, &id)?
        } else {
            vec![]
        };
        let scope: serde_json::Value = file_scope_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| serde_json::json!([]));
        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        data.insert("status".into(), serde_json::Value::String(map_spec_status(&status)));
        let description_text = match (description, body_markdown) {
            (Some(d), Some(b)) if !d.is_empty() && !b.is_empty() => format!("{d}\n\n{b}"),
            (Some(d), _) if !d.is_empty() => d,
            (_, Some(b)) if !b.is_empty() => b,
            _ => String::new(),
        };
        if !description_text.is_empty() {
            data.insert("description".into(), serde_json::Value::String(description_text));
        }
        if !acceptance.is_empty() {
            let bullets = acceptance.iter().map(|t| format!("- {t}")).collect::<Vec<_>>().join("\n");
            data.insert("acceptance".into(), serde_json::Value::String(bullets));
        }
        data.insert("scope".into(), scope);
        if item_exists(svc, &schema.id, &id)? {
            continue;
        }
        store::create_item(svc, schema, &id, &serde_json::Value::Object(data), None)?;
        migrated += 1;
    }
    Ok(migrated)
}

pub(super) fn map_spec_status(raw: &str) -> String {
    match raw.to_ascii_lowercase().as_str() {
        "draft" | "proposed" | "planned" => "draft",
        "active" | "in-progress" | "in_progress" | "doing" => "in-progress",
        "complete" | "completed" | "done" | "shipped" => "complete",
        "abandoned" | "cancelled" | "canceled" | "dropped" => "abandoned",
        _ => "draft",
    }
    .to_string()
}

// ── Shared ───────────────────────────────────────────────────────────────────

pub(super) fn table_exists(conn: &mut SqliteConnection, name: &str) -> Result<bool> {
    let row: Option<(String,)> = crate::db::block_on(async {
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name = ?")
            .bind(name)
            .fetch_optional(&mut *conn)
            .await
    })?;
    Ok(row.is_some())
}

pub(super) fn item_exists(svc: &mut DataService, collection: &str, id: &str) -> Result<bool> {
    let sql = format!("SELECT id FROM `collection_{collection}` WHERE id = ?");
    let rows = svc.query(&sql, &[SqlParam::from(id)])?;
    Ok(!rows.is_empty())
}

fn read_capability_titles(platform: &mut SqliteConnection, target_id: &str) -> Result<Vec<String>> {
    let rows = crate::db::block_on(async {
        sqlx::query(
            "SELECT title FROM capability WHERE target_id = ? ORDER BY priority DESC, created_at ASC",
        )
        .bind(target_id)
        .fetch_all(&mut *platform)
        .await
    })?;
    Ok(rows.iter().filter_map(|r| r.try_get::<String, _>("title").ok()).collect())
}

/// Delete all rows from `table` on `conn` if the table exists. Returns rows
/// affected.
pub(super) fn delete_if_table_exists(conn: &mut SqliteConnection, table: &str) -> Result<u64> {
    if !table_exists(conn, table)? {
        return Ok(0);
    }
    let sql = format!("DELETE FROM {table}");
    let result = crate::db::block_on(async { sqlx::query(&sql).execute(&mut *conn).await })?;
    Ok(result.rows_affected())
}
