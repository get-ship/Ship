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

        // Glue legacy description + body_markdown + acceptance bullets into
        // a single markdown body that lands at .artifacts/specs/<id>.md —
        // the storage shape the new specs schema expects.
        let body_inline = match (&description, &body_markdown) {
            (Some(d), Some(b)) if !d.is_empty() && !b.is_empty() => format!("{d}\n\n{b}"),
            (Some(d), _) if !d.is_empty() => d.clone(),
            (_, Some(b)) if !b.is_empty() => b.clone(),
            _ => String::new(),
        };
        let body_text = if acceptance.is_empty() {
            body_inline
        } else {
            let bullets = acceptance
                .iter()
                .map(|t| format!("- {t}"))
                .collect::<Vec<_>>()
                .join("\n");
            if body_inline.is_empty() {
                format!("## Acceptance\n\n{bullets}\n")
            } else {
                format!("{body_inline}\n\n## Acceptance\n\n{bullets}\n")
            }
        };
        let body_path = format!(".artifacts/specs/{id}.md");
        // Best-effort write; if the project root isn't resolvable the
        // body file is missing but the row still lands. Authors can
        // re-author the body via the UI.
        if !body_text.is_empty()
            && let Ok(project_ship) = crate::project::get_project_dir(None)
            && let Some(root) = project_ship.parent()
        {
            let abs = root.join(&body_path);
            if let Some(dir) = abs.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(&abs, &body_text);
        }

        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        data.insert("body_path".into(), serde_json::Value::String(body_path));
        data.insert("status".into(), serde_json::Value::String(map_spec_status(&status)));
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
    // Map legacy `target.status` values onto the current specs enum
    // (draft / ready / launched / archived). The legacy active/complete
    // states fold into "launched" since they imply the work was kicked
    // off; abandoned folds into "archived".
    match raw.to_ascii_lowercase().as_str() {
        "draft" | "proposed" | "planned" => "draft",
        "ready" | "queued" => "ready",
        "active" | "in-progress" | "in_progress" | "doing" | "complete" | "completed"
        | "done" | "shipped" | "launched" => "launched",
        "abandoned" | "cancelled" | "canceled" | "dropped" | "archived" => "archived",
        _ => "draft",
    }
    .to_string()
}

// ── Spec table → specs collection ────────────────────────────────────────────
//
// Migration 0007 drops the dedicated `spec` table that 0002 added. Before
// the drop fires, copy every row into the `specs` collection so users who
// authored specs through the old API don't lose data. Idempotent: skipped
// when the source table is missing or when the destination already has a
// row with the matching id.

pub(super) fn migrate_spec_table(
    svc: &mut DataService,
    schema: &CollectionSchema,
    platform: &mut SqliteConnection,
) -> Result<usize> {
    if !table_exists(platform, "spec")? {
        return Ok(0);
    }
    let rows = crate::db::block_on(async {
        sqlx::query(
            "SELECT id, title, body_path, status, pipeline, vars_json, criteria_json, scope_json, priority FROM spec",
        )
        .fetch_all(&mut *platform)
        .await
    })?;
    let mut migrated = 0;
    for row in rows {
        let id: String = row.try_get("id")?;
        if item_exists(svc, &schema.id, &id)? {
            continue;
        }
        let title: String = row.try_get("title")?;
        let body_path: String = row.try_get("body_path")?;
        let status: String = row.try_get("status").unwrap_or_else(|_| "draft".into());
        let pipeline: Option<String> = row.try_get("pipeline").ok();
        let vars_json: String = row.try_get("vars_json").unwrap_or_else(|_| "{}".into());
        let criteria_json: String = row.try_get("criteria_json").unwrap_or_else(|_| "[]".into());
        let scope_json: String = row.try_get("scope_json").unwrap_or_else(|_| "[]".into());
        let priority: Option<i64> = row.try_get("priority").ok();

        let vars: serde_json::Value =
            serde_json::from_str(&vars_json).unwrap_or_else(|_| serde_json::json!({}));
        let criteria: serde_json::Value =
            serde_json::from_str(&criteria_json).unwrap_or_else(|_| serde_json::json!([]));
        let scope: serde_json::Value =
            serde_json::from_str(&scope_json).unwrap_or_else(|_| serde_json::json!([]));

        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        data.insert("body_path".into(), serde_json::Value::String(body_path));
        data.insert("status".into(), serde_json::Value::String(status));
        if let Some(p) = pipeline.filter(|s| !s.is_empty()) {
            data.insert("pipeline".into(), serde_json::Value::String(p));
        }
        data.insert("vars".into(), vars);
        data.insert("criteria".into(), criteria);
        data.insert("scope".into(), scope);
        if let Some(p) = priority {
            data.insert("priority".into(), serde_json::json!(p as u32));
        }
        store::create_item(svc, schema, &id, &serde_json::Value::Object(data), None)?;
        migrated += 1;
    }
    Ok(migrated)
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
