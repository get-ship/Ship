//! Key-value state store — namespaced, JSON values.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

use crate::db::{block_on, open_db};

pub fn set(namespace: &str, key: &str, value: &serde_json::Value) -> Result<()> {
    let mut conn = open_db()?;
    set_with_conn(&mut conn, namespace, key, value)
}

/// Same as [`set`] but reuses an existing connection. Use this when called
/// inside a projection transaction to avoid SQLite's "database is locked"
/// from a parallel writer.
pub fn set_with_conn(
    conn: &mut sqlx::SqliteConnection,
    namespace: &str,
    key: &str,
    value: &serde_json::Value,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let value_json = serde_json::to_string(value)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO kv_state (namespace, key, value_json, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(namespace, key) DO UPDATE SET
               value_json = excluded.value_json,
               updated_at = excluded.updated_at",
        )
        .bind(namespace)
        .bind(key)
        .bind(&value_json)
        .bind(&now)
        .execute(conn)
        .await
    })?;
    Ok(())
}

pub fn get(namespace: &str, key: &str) -> Result<Option<serde_json::Value>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query("SELECT value_json FROM kv_state WHERE namespace = ? AND key = ?")
            .bind(namespace)
            .bind(key)
            .fetch_optional(&mut conn)
            .await
    })?;
    match row {
        None => Ok(None),
        Some(r) => {
            let json_str: String = r.get(0);
            Ok(Some(serde_json::from_str(&json_str)?))
        }
    }
}

pub fn delete(namespace: &str, key: &str) -> Result<()> {
    let mut conn = open_db()?;
    delete_with_conn(&mut conn, namespace, key)
}

/// Same as [`delete`] but reuses an existing connection.
pub fn delete_with_conn(
    conn: &mut sqlx::SqliteConnection,
    namespace: &str,
    key: &str,
) -> Result<()> {
    block_on(async {
        sqlx::query("DELETE FROM kv_state WHERE namespace = ? AND key = ?")
            .bind(namespace)
            .bind(key)
            .execute(conn)
            .await
    })?;
    Ok(())
}

pub fn list_keys(namespace: &str) -> Result<Vec<String>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query("SELECT key FROM kv_state WHERE namespace = ? ORDER BY key")
            .bind(namespace)
            .fetch_all(&mut conn)
            .await
    })?;
    Ok(rows.iter().map(|r| r.get(0)).collect())
}

/// Return all (key, value) pairs in a namespace.
pub fn list_all(namespace: &str) -> Result<Vec<(String, serde_json::Value)>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query("SELECT key, value_json FROM kv_state WHERE namespace = ? ORDER BY key")
            .bind(namespace)
            .fetch_all(&mut conn)
            .await
    })?;
    let mut pairs = Vec::with_capacity(rows.len());
    for r in &rows {
        let key: String = r.get(0);
        let json_str: String = r.get(1);
        let value: serde_json::Value = serde_json::from_str(&json_str)?;
        pairs.push((key, value));
    }
    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_db;
    use crate::project::init_project;
    use serde_json::json;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn test_set_and_get() {
        let (_tmp, _ship_dir) = setup();
        set("agent", "active_workspace", &json!("ws-001")).unwrap();
        let val = get("agent", "active_workspace").unwrap().unwrap();
        assert_eq!(val, json!("ws-001"));
    }

    #[test]
    fn test_set_overwrites() {
        let (_tmp, _ship_dir) = setup();
        set("config", "theme", &json!("dark")).unwrap();
        set("config", "theme", &json!("light")).unwrap();
        let val = get("config", "theme").unwrap().unwrap();
        assert_eq!(val, json!("light"));
    }

    #[test]
    fn test_get_missing_returns_none() {
        let (_tmp, _ship_dir) = setup();
        assert!(get("ns", "missing").unwrap().is_none());
    }

    #[test]
    fn test_delete() {
        let (_tmp, _ship_dir) = setup();
        set("ns", "key", &json!(42)).unwrap();
        delete("ns", "key").unwrap();
        assert!(get("ns", "key").unwrap().is_none());
    }

    #[test]
    fn test_namespaces_are_isolated() {
        let (_tmp, _ship_dir) = setup();
        set("ns1", "key", &json!("a")).unwrap();
        set("ns2", "key", &json!("b")).unwrap();
        assert_eq!(get("ns1", "key").unwrap().unwrap(), json!("a"));
        assert_eq!(get("ns2", "key").unwrap().unwrap(), json!("b"));
    }

    #[test]
    fn test_list_keys() {
        let (_tmp, _ship_dir) = setup();
        set("app", "alpha", &json!(1)).unwrap();
        set("app", "beta", &json!(2)).unwrap();
        set("other", "gamma", &json!(3)).unwrap();
        let keys = list_keys("app").unwrap();
        assert_eq!(keys, vec!["alpha", "beta"]);
    }

}
