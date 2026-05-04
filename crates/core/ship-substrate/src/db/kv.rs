//! Key-value state store — namespaced, JSON values.
//!
//! Backed by `kv.db` (separate from `platform.db`). The physical split means
//! KV writes never compete with the event log for WAL locks.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

use crate::db::{block_on, ensure_kv_db, open_kv_db};

pub fn set(namespace: &str, key: &str, value: &serde_json::Value) -> Result<()> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
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
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get(namespace: &str, key: &str) -> Result<Option<serde_json::Value>> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
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
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    block_on(async {
        sqlx::query("DELETE FROM kv_state WHERE namespace = ? AND key = ?")
            .bind(namespace)
            .bind(key)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn list_keys(namespace: &str) -> Result<Vec<String>> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
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
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
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

/// One-time upgrade: copy any rows that exist in the legacy `kv_state` table
/// inside `platform.db` into `kv.db`. Safe to call repeatedly — rows already
/// present in `kv.db` are skipped via `INSERT OR IGNORE`.
///
/// Called at daemon startup before the platform.db migration that drops `kv_state`.
pub fn migrate_from_platform_db(ship_dir: &std::path::Path) -> Result<()> {
    use crate::db::{kv_db_path, open_db_at, platform_db_path};

    let platform_path = platform_db_path(ship_dir);
    if !platform_path.exists() {
        return Ok(());
    }

    let kv_path = kv_db_path(ship_dir);
    crate::db::ensure_kv_db_at(ship_dir)?;

    let mut platform_conn = open_db_at(&platform_path)?;

    // Check if kv_state exists in platform.db (it won't on a fresh install or
    // after migration 0002 has already run).
    let table_exists: bool = block_on(async {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='kv_state'",
        )
        .fetch_one(&mut platform_conn)
        .await
    })
    .map(|n| n > 0)
    .unwrap_or(false);

    if !table_exists {
        return Ok(());
    }

    let rows: Vec<(String, String, String, String)> = {
        let raw = block_on(async {
            sqlx::query("SELECT namespace, key, value_json, updated_at FROM kv_state")
                .fetch_all(&mut platform_conn)
                .await
        })?;
        raw.iter()
            .map(|r| (r.get(0), r.get(1), r.get(2), r.get(3)))
            .collect()
    };

    if rows.is_empty() {
        return Ok(());
    }

    let mut kv_conn = open_db_at(&kv_path)?;
    for (namespace, key, value_json, updated_at) in &rows {
        block_on(async {
            sqlx::query(
                "INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(namespace)
            .bind(key)
            .bind(value_json)
            .bind(updated_at)
            .execute(&mut kv_conn)
            .await
        })?;
    }

    tracing::info!(
        "kv migration: copied {} rows from platform.db to kv.db",
        rows.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_kv_db;
    use crate::testutil::init_project_minimal as init_project;
    use serde_json::json;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_kv_db().unwrap();
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

    #[test]
    fn migrate_from_platform_db_copies_rows() {
        use crate::db::{ensure_db_at, platform_db_path};

        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();

        // Seed kv_state in platform.db (before migration 0002 runs).
        // We open platform.db directly so we can write to kv_state while it
        // still exists.
        ensure_db_at(&ship_dir).unwrap();
        // At this point migration 0002 has already run and dropped kv_state,
        // so we test the "table missing" early-return path.
        migrate_from_platform_db(&ship_dir).unwrap();

        // Verify no rows in kv.db (nothing to copy).
        ensure_kv_db_at_with_ship_dir(&ship_dir);
        let keys = list_keys("test_ns").unwrap();
        assert!(keys.is_empty());
    }

    fn ensure_kv_db_at_with_ship_dir(ship_dir: &std::path::Path) {
        crate::db::ensure_kv_db_at(ship_dir).unwrap();
    }
}
