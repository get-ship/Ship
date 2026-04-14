//! Collection storage — CRUD and queries backed by SQLite JSON columns.
//!
//! Items are stored in `.ship/docs.db` as JSON blobs. The schema validates
//! on write; the database enforces nothing beyond NOT NULL. Queries use
//! `json_extract()` to filter/sort by field values.

use anyhow::{Context, Result, anyhow};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};
use std::path::Path;
use std::str::FromStr;

use super::query::{QueryRequest, translate_query};
use super::schema::CollectionSchema;

/// Open or create `docs.db` at the given `.ship/` directory.
/// Creates the items table if it doesn't exist.
pub fn open_docs_db(ship_dir: &Path) -> Result<SqliteConnection> {
    let db_path = ship_dir.join("docs.db");
    let url = format!("sqlite://{}", db_path.display());
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(5));

    let mut conn = crate::db::block_on(async {
        SqliteConnection::connect_with(&opts).await
    })?;

    crate::db::block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL").execute(&mut conn).await
    })?;
    crate::db::block_on(async {
        sqlx::query("PRAGMA busy_timeout = 5000").execute(&mut conn).await
    })?;
    crate::db::block_on(async {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY,
                collection TEXT NOT NULL,
                data TEXT NOT NULL,
                created_by TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        ).execute(&mut conn).await
    })?;
    crate::db::block_on(async {
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_items_collection ON items(collection)"
        ).execute(&mut conn).await
    })?;

    Ok(conn)
}

/// Ensure indexes exist for filterable/enum fields in a schema.
pub fn ensure_indexes(conn: &mut SqliteConnection, schema: &CollectionSchema) -> Result<()> {
    for (name, field) in &schema.fields {
        let should_index = field.filterable
            || matches!(field.field_type, super::schema::FieldType::Enum);
        if !should_index {
            continue;
        }
        let idx_name = format!("idx_{}_{}", schema.id, name);
        let sql = format!(
            "CREATE INDEX IF NOT EXISTS {idx_name} ON items(json_extract(data, '$.{name}')) WHERE collection = '{}'",
            schema.id
        );
        crate::db::block_on(async {
            sqlx::query(&sql).execute(&mut *conn).await
        })
        .with_context(|| format!("creating index {idx_name}"))?;
    }
    Ok(())
}

/// A stored collection item.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    pub id: String,
    pub collection: String,
    pub data: serde_json::Value,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Insert a new item. Validates against the schema before writing.
pub fn create_item(
    conn: &mut SqliteConnection,
    schema: &CollectionSchema,
    id: &str,
    data: &serde_json::Value,
    created_by: Option<&str>,
) -> Result<Item> {
    super::schema::validate_item(schema, data)
        .map_err(|errors| anyhow!("validation failed: {}", errors.join("; ")))?;

    let now = chrono::Utc::now().to_rfc3339();
    let data_str = serde_json::to_string(data)?;

    crate::db::block_on(async {
        sqlx::query(
            "INSERT INTO items (id, collection, data, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(&schema.id)
        .bind(&data_str)
        .bind(created_by)
        .bind(&now)
        .bind(&now)
        .execute(&mut *conn)
        .await
    })
    .with_context(|| format!("inserting item {id} into {}", schema.id))?;

    Ok(Item {
        id: id.to_string(),
        collection: schema.id.clone(),
        data: data.clone(),
        created_by: created_by.map(str::to_string),
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Get a single item by ID and collection.
pub fn get_item(
    conn: &mut SqliteConnection,
    collection: &str,
    id: &str,
) -> Result<Option<Item>> {
    let row = crate::db::block_on(async {
        sqlx::query("SELECT id, collection, data, created_by, created_at, updated_at FROM items WHERE id = ? AND collection = ?")
            .bind(id)
            .bind(collection)
            .fetch_optional(&mut *conn)
            .await
    })?;

    match row {
        Some(r) => {
            let data_str: String = r.get("data");
            Ok(Some(Item {
                id: r.get("id"),
                collection: r.get("collection"),
                data: serde_json::from_str(&data_str)?,
                created_by: r.get("created_by"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        }
        None => Ok(None),
    }
}

/// Update an item's fields. Merges `updates` into existing data.
/// Validates the merged result against the schema.
pub fn update_item(
    conn: &mut SqliteConnection,
    schema: &CollectionSchema,
    id: &str,
    updates: &serde_json::Value,
) -> Result<Item> {
    let existing = get_item(conn, &schema.id, id)?
        .ok_or_else(|| anyhow!("item '{}' not found in '{}'", id, schema.id))?;

    let mut merged = existing.data.clone();
    if let (Some(base), Some(patch)) = (merged.as_object_mut(), updates.as_object()) {
        for (k, v) in patch {
            base.insert(k.clone(), v.clone());
        }
    }

    super::schema::validate_item(schema, &merged)
        .map_err(|errors| anyhow!("validation failed after merge: {}", errors.join("; ")))?;

    let now = chrono::Utc::now().to_rfc3339();
    let data_str = serde_json::to_string(&merged)?;

    crate::db::block_on(async {
        sqlx::query("UPDATE items SET data = ?, updated_at = ? WHERE id = ? AND collection = ?")
            .bind(&data_str)
            .bind(&now)
            .bind(id)
            .bind(&schema.id)
            .execute(&mut *conn)
            .await
    })?;

    Ok(Item {
        id: id.to_string(),
        collection: schema.id.clone(),
        data: merged,
        created_by: existing.created_by,
        created_at: existing.created_at,
        updated_at: now,
    })
}

/// Delete an item by ID.
pub fn delete_item(
    conn: &mut SqliteConnection,
    collection: &str,
    id: &str,
) -> Result<bool> {
    let result = crate::db::block_on(async {
        sqlx::query("DELETE FROM items WHERE id = ? AND collection = ?")
            .bind(id)
            .bind(collection)
            .execute(&mut *conn)
            .await
    })?;
    Ok(result.rows_affected() > 0)
}

/// Execute a query against a collection. Returns matching items.
pub fn query_items(
    conn: &mut SqliteConnection,
    req: &QueryRequest,
) -> Result<Vec<Item>> {
    let (sql, params) = translate_query(req);

    let mut query = sqlx::query(&sql);
    for p in &params {
        query = match p {
            serde_json::Value::String(s) => query.bind(s.as_str()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    query.bind(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => query.bind(*b),
            _ => query.bind(p.to_string()),
        };
    }

    let rows = crate::db::block_on(async {
        query.fetch_all(&mut *conn).await
    })?;

    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        let data_str: String = r.get("data");
        items.push(Item {
            id: r.get("id"),
            collection: r.get("collection"),
            data: serde_json::from_str(&data_str)?,
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        });
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::schema::parse_schema;
    use serde_json::json;

    fn test_schema() -> CollectionSchema {
        parse_schema(r#"{
            "id": "task",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["todo", "doing", "done"] },
                "priority": { "type": "number" }
            }
        }"#).unwrap()
    }

    fn test_db() -> SqliteConnection {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        // Leak the tempdir so it lives for the test duration
        let ship_dir = Box::leak(Box::new(ship_dir));
        open_docs_db(ship_dir).unwrap()
    }

    #[test]
    fn create_and_get_item() {
        let mut conn = test_db();
        let schema = test_schema();
        let data = json!({ "title": "Write tests", "status": "todo", "priority": 1 });

        let item = create_item(&mut conn, &schema, "t1", &data, Some("user")).unwrap();
        assert_eq!(item.id, "t1");
        assert_eq!(item.data["title"], "Write tests");
        assert_eq!(item.created_by.as_deref(), Some("user"));

        let fetched = get_item(&mut conn, "task", "t1").unwrap().unwrap();
        assert_eq!(fetched.data["title"], "Write tests");
    }

    #[test]
    fn create_rejects_invalid_data() {
        let mut conn = test_db();
        let schema = test_schema();
        let data = json!({ "status": "todo" }); // missing required title

        let err = create_item(&mut conn, &schema, "t1", &data, None).unwrap_err();
        assert!(err.to_string().contains("'title' is required"), "{err}");
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let mut conn = test_db();
        let result = get_item(&mut conn, "task", "nope").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn update_merges_fields() {
        let mut conn = test_db();
        let schema = test_schema();
        create_item(&mut conn, &schema, "t1", &json!({ "title": "Original", "status": "todo" }), None).unwrap();

        let updated = update_item(&mut conn, &schema, "t1", &json!({ "status": "done" })).unwrap();
        assert_eq!(updated.data["title"], "Original"); // preserved
        assert_eq!(updated.data["status"], "done"); // updated
    }

    #[test]
    fn update_validates_merged_result() {
        let mut conn = test_db();
        let schema = test_schema();
        create_item(&mut conn, &schema, "t1", &json!({ "title": "Test", "status": "todo" }), None).unwrap();

        let err = update_item(&mut conn, &schema, "t1", &json!({ "status": "invalid" })).unwrap_err();
        assert!(err.to_string().contains("must be one of"), "{err}");
    }

    #[test]
    fn update_nonexistent_errors() {
        let mut conn = test_db();
        let schema = test_schema();
        let err = update_item(&mut conn, &schema, "nope", &json!({ "title": "X" })).unwrap_err();
        assert!(err.to_string().contains("not found"), "{err}");
    }

    #[test]
    fn delete_existing_item() {
        let mut conn = test_db();
        let schema = test_schema();
        create_item(&mut conn, &schema, "t1", &json!({ "title": "Delete me" }), None).unwrap();

        assert!(delete_item(&mut conn, "task", "t1").unwrap());
        assert!(get_item(&mut conn, "task", "t1").unwrap().is_none());
    }

    #[test]
    fn delete_nonexistent_returns_false() {
        let mut conn = test_db();
        assert!(!delete_item(&mut conn, "task", "nope").unwrap());
    }

    #[test]
    fn ensure_indexes_creates_for_enum() {
        let mut conn = test_db();
        let schema = test_schema();
        // Should not error
        ensure_indexes(&mut conn, &schema).unwrap();
    }
}
