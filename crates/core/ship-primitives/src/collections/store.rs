//! Collection storage — real tables with sea-query DDL, built on the data service.
//!
//! Each collection gets its own table (`collection_<id>`) with real columns
//! derived from the schema. Schema changes add columns via ALTER TABLE.

use anyhow::{Context, Result, anyhow};
use sea_query::{
    Alias, ColumnDef, Expr, Index, Query, SqliteQueryBuilder, Table,
};
use std::path::Path;

use ship_substrate::data::DataService;
use super::query::{self, QueryRequest, collection_table};
use super::schema::{CollectionSchema, FieldType};

/// System columns present on every collection table.
const SYSTEM_COLS: &[&str] = &["id", "created_by", "created_at", "updated_at"];

/// Open a DataService backed by `.ship/data.db`.
pub fn open_data_db(ship_dir: &Path) -> Result<DataService> {
    DataService::open(ship_dir)
}

/// Ensure a collection table exists matching the schema.
/// Creates the table if missing, adds columns for new fields.
pub fn ensure_collection_table(svc: &mut DataService, schema: &CollectionSchema) -> Result<()> {
    let table_name = format!("collection_{}", schema.id);

    // Check if table exists.
    let rows = svc.query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name = ?",
        &[ship_substrate::data::SqlParam::from(table_name.as_str())],
    )?;

    if rows.is_empty() {
        // Create table from schema.
        let ddl = build_create_table(schema);
        svc.execute(&ddl, &[])?;
    } else {
        // Table exists — add any missing columns.
        let existing_cols = get_existing_columns(svc, &table_name)?;
        for (field_name, _field_def) in &schema.fields {
            if !existing_cols.contains(&field_name.to_string()) {
                let alter = format!(
                    "ALTER TABLE `{table_name}` ADD COLUMN `{field_name}` TEXT"
                );
                svc.execute(&alter, &[])
                    .with_context(|| format!("adding column '{field_name}' to '{table_name}'"))?;
            }
        }
    }

    // Ensure indexes for filterable/enum fields.
    ensure_indexes(svc, schema)?;
    Ok(())
}

fn build_create_table(schema: &CollectionSchema) -> String {
    let table = collection_table(&schema.id);
    let mut stmt = Table::create();
    stmt.table(table)
        .if_not_exists()
        .col(ColumnDef::new(Alias::new("id")).text().primary_key().not_null())
        .col(ColumnDef::new(Alias::new("created_by")).text())
        .col(ColumnDef::new(Alias::new("created_at")).text().not_null())
        .col(ColumnDef::new(Alias::new("updated_at")).text().not_null());

    for (name, field) in &schema.fields {
        let mut col = ColumnDef::new(Alias::new(name.as_str()));
        match field.field_type {
            FieldType::Number => { col.double(); }
            FieldType::Boolean => { col.boolean(); }
            _ => { col.text(); }
        };
        if field.required {
            col.not_null();
        }
        stmt.col(&mut col);
    }

    stmt.to_string(SqliteQueryBuilder)
}

fn get_existing_columns(svc: &mut DataService, table_name: &str) -> Result<Vec<String>> {
    let rows = svc.query(
        &format!("PRAGMA table_info(`{table_name}`)"),
        &[],
    )?;
    Ok(rows.iter().filter_map(|r| r.get_str("name").map(str::to_string)).collect())
}

/// Ensure indexes for filterable/enum fields.
pub fn ensure_indexes(svc: &mut DataService, schema: &CollectionSchema) -> Result<()> {
    let table_name = format!("collection_{}", schema.id);
    for (name, field) in &schema.fields {
        let should_index = field.filterable
            || matches!(field.field_type, FieldType::Enum);
        if !should_index {
            continue;
        }
        if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
            continue;
        }
        let idx = Index::create()
            .name(&format!("idx_{}_{}", schema.id, name))
            .table(Alias::new(table_name.as_str()))
            .col(Alias::new(name.as_str()))
            .if_not_exists()
            .to_string(SqliteQueryBuilder);
        svc.execute(&idx, &[])?;
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

/// Insert a new item.
pub fn create_item(
    svc: &mut DataService,
    schema: &CollectionSchema,
    id: &str,
    data: &serde_json::Value,
    created_by: Option<&str>,
) -> Result<Item> {
    super::schema::validate_item(schema, data)
        .map_err(|errors| anyhow!("validation failed: {}", errors.join("; ")))?;

    let now = chrono::Utc::now().to_rfc3339();
    let table = collection_table(&schema.id);
    let obj = data.as_object().ok_or_else(|| anyhow!("data must be an object"))?;

    // Build INSERT with sea-query.
    let mut cols = vec![
        Alias::new("id"), Alias::new("created_by"),
        Alias::new("created_at"), Alias::new("updated_at"),
    ];
    let mut vals: Vec<sea_query::SimpleExpr> = vec![
        id.into(),
        created_by.map(|s| s.to_string()).into(),
        now.clone().into(),
        now.clone().into(),
    ];

    for (field_name, field_val) in obj {
        cols.push(Alias::new(field_name.as_str()));
        vals.push(json_to_sql_value(field_val).into());
    }

    let (sql, values) = Query::insert()
        .into_table(table)
        .columns(cols)
        .values_panic(vals)
        .build(SqliteQueryBuilder);

    svc.execute_sea(&sql, values)
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

/// Get a single item by ID.
pub fn get_item(
    svc: &mut DataService,
    schema: &CollectionSchema,
    id: &str,
) -> Result<Option<Item>> {
    let table = collection_table(&schema.id);
    let (sql, values) = Query::select()
        .expr(Expr::asterisk())
        .from(table)
        .and_where(Expr::col(Alias::new("id")).eq(id))
        .build(SqliteQueryBuilder);

    let rows = svc.query_sea(&sql, values)?;
    if rows.is_empty() {
        return Ok(None);
    }

    Ok(Some(row_to_item(&rows[0], schema)))
}

/// Update item fields (partial merge).
pub fn update_item(
    svc: &mut DataService,
    schema: &CollectionSchema,
    id: &str,
    updates: &serde_json::Value,
) -> Result<Item> {
    let existing = get_item(svc, schema, id)?
        .ok_or_else(|| anyhow!("item '{}' not found in '{}'", id, schema.id))?;

    // Merge for validation.
    let mut merged = existing.data.clone();
    if let (Some(base), Some(patch)) = (merged.as_object_mut(), updates.as_object()) {
        for (k, v) in patch {
            base.insert(k.clone(), v.clone());
        }
    }

    super::schema::validate_item(schema, &merged)
        .map_err(|errors| anyhow!("validation failed after merge: {}", errors.join("; ")))?;

    let table = collection_table(&schema.id);
    let now = chrono::Utc::now().to_rfc3339();

    let mut stmt = Query::update();
    stmt.table(table)
        .value(Alias::new("updated_at"), now.clone())
        .and_where(Expr::col(Alias::new("id")).eq(id));

    if let Some(obj) = updates.as_object() {
        for (field_name, field_val) in obj {
            stmt.value(Alias::new(field_name.as_str()), json_to_sql_value(field_val));
        }
    }

    let (sql, values) = stmt.build(SqliteQueryBuilder);
    svc.execute_sea(&sql, values)?;

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
    svc: &mut DataService,
    collection: &str,
    id: &str,
) -> Result<bool> {
    let table = collection_table(collection);
    let (sql, values) = Query::delete()
        .from_table(table)
        .and_where(Expr::col(Alias::new("id")).eq(id))
        .build(SqliteQueryBuilder);

    let affected = svc.execute_sea(&sql, values)?;
    Ok(affected > 0)
}

/// Query items using structured filters.
pub fn query_items(
    svc: &mut DataService,
    schema: &CollectionSchema,
    req: &QueryRequest,
) -> Result<Vec<Item>> {
    let (sql, values) = query::build_select(req);
    let rows = match svc.query_sea(&sql, values) {
        Ok(r) => r,
        Err(e) => {
            if format!("{e:#}").contains("no such table") {
                return Ok(vec![]);
            }
            return Err(e);
        }
    };

    Ok(rows.iter().map(|r| row_to_item(r, schema)).collect())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Reconstruct an Item from a DataRow, packing schema fields into the data JSON object.
/// Applies type coercion for boolean fields (SQLite stores as 0/1) and default values.
fn row_to_item(row: &ship_substrate::data::DataRow, schema: &CollectionSchema) -> Item {
    let mut data = serde_json::Map::new();
    for (field_name, field_def) in &schema.fields {
        if let Some(val) = row.get(field_name) {
            if val.is_null() {
                // Apply default if defined.
                if let Some(ref default) = field_def.default {
                    data.insert(field_name.clone(), default.clone());
                }
                continue;
            }
            // Coerce SQLite integer 0/1 back to boolean for boolean fields.
            let coerced = if field_def.field_type == FieldType::Boolean {
                match val.as_i64() {
                    Some(0) => serde_json::Value::Bool(false),
                    Some(1) => serde_json::Value::Bool(true),
                    _ => val.clone(),
                }
            } else {
                val.clone()
            };
            data.insert(field_name.clone(), coerced);
        } else if let Some(ref default) = field_def.default {
            data.insert(field_name.clone(), default.clone());
        }
    }

    Item {
        id: row.get_str("id").unwrap_or("").to_string(),
        collection: schema.id.clone(),
        data: serde_json::Value::Object(data),
        created_by: row.get_str("created_by").map(str::to_string),
        created_at: row.get_str("created_at").unwrap_or("").to_string(),
        updated_at: row.get_str("updated_at").unwrap_or("").to_string(),
    }
}

fn json_to_sql_value(v: &serde_json::Value) -> sea_query::Value {
    match v {
        serde_json::Value::String(s) => s.clone().into(),
        serde_json::Value::Number(n) => {
            // Always store as double to match REAL column affinity.
            if let Some(f) = n.as_f64() { f.into() }
            else { n.to_string().into() }
        }
        serde_json::Value::Bool(b) => (*b).into(),
        serde_json::Value::Null => sea_query::Value::String(None),
        // Arrays/objects stored as JSON strings.
        other => other.to_string().into(),
    }
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

    fn test_db() -> (DataService, CollectionSchema, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = open_data_db(&ship_dir).unwrap();
        let schema = test_schema();
        ensure_collection_table(&mut svc, &schema).unwrap();
        (svc, schema, tmp)
    }

    #[test]
    fn create_and_get_item() {
        let (mut svc, schema, _tmp) = test_db();
        let data = json!({ "title": "Write tests", "status": "todo", "priority": 1 });

        let item = create_item(&mut svc, &schema, "t1", &data, Some("user")).unwrap();
        assert_eq!(item.id, "t1");
        assert_eq!(item.data["title"], "Write tests");
        assert_eq!(item.created_by.as_deref(), Some("user"));

        let fetched = get_item(&mut svc, &schema, "t1").unwrap().unwrap();
        assert_eq!(fetched.data["title"], "Write tests");
    }

    #[test]
    fn create_rejects_invalid_data() {
        let (mut svc, schema, _tmp) = test_db();
        let data = json!({ "status": "todo" });

        let err = create_item(&mut svc, &schema, "t1", &data, None).unwrap_err();
        assert!(err.to_string().contains("'title' is required"), "{err}");
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let (mut svc, schema, _tmp) = test_db();
        let result = get_item(&mut svc, &schema, "nope").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn update_merges_fields() {
        let (mut svc, schema, _tmp) = test_db();
        create_item(&mut svc, &schema, "t1", &json!({ "title": "Original", "status": "todo" }), None).unwrap();

        let updated = update_item(&mut svc, &schema, "t1", &json!({ "status": "done" })).unwrap();
        assert_eq!(updated.data["title"], "Original");
        assert_eq!(updated.data["status"], "done");
    }

    #[test]
    fn update_validates_merged_result() {
        let (mut svc, schema, _tmp) = test_db();
        create_item(&mut svc, &schema, "t1", &json!({ "title": "Test", "status": "todo" }), None).unwrap();

        let err = update_item(&mut svc, &schema, "t1", &json!({ "status": "invalid" })).unwrap_err();
        assert!(err.to_string().contains("must be one of"), "{err}");
    }

    #[test]
    fn update_nonexistent_errors() {
        let (mut svc, schema, _tmp) = test_db();
        let err = update_item(&mut svc, &schema, "nope", &json!({ "title": "X" })).unwrap_err();
        assert!(err.to_string().contains("not found"), "{err}");
    }

    #[test]
    fn delete_existing_item() {
        let (mut svc, schema, _tmp) = test_db();
        create_item(&mut svc, &schema, "t1", &json!({ "title": "Delete me" }), None).unwrap();

        assert!(delete_item(&mut svc, "task", "t1").unwrap());
        assert!(get_item(&mut svc, &schema, "t1").unwrap().is_none());
    }

    #[test]
    fn delete_nonexistent_returns_false() {
        let (mut svc, _, _tmp) = test_db();
        assert!(!delete_item(&mut svc, "task", "nope").unwrap());
    }

    #[test]
    fn schema_migration_adds_column() {
        let (mut svc, _, _tmp) = test_db();

        // Create an item with the original schema.
        let schema = test_schema();
        create_item(&mut svc, &schema, "t1", &json!({ "title": "Before migration" }), None).unwrap();

        // "Migrate" — add a new field to the schema.
        let schema_v2 = parse_schema(r#"{
            "id": "task",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["todo", "doing", "done"] },
                "priority": { "type": "number" },
                "assignee": { "type": "string" }
            }
        }"#).unwrap();
        ensure_collection_table(&mut svc, &schema_v2).unwrap();

        // Old item should still be readable.
        let item = get_item(&mut svc, &schema_v2, "t1").unwrap().unwrap();
        assert_eq!(item.data["title"], "Before migration");

        // New field should work.
        create_item(&mut svc, &schema_v2, "t2",
            &json!({ "title": "After migration", "assignee": "alice" }), None).unwrap();
        let item2 = get_item(&mut svc, &schema_v2, "t2").unwrap().unwrap();
        assert_eq!(item2.data["assignee"], "alice");
    }
}
