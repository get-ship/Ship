//! Machine-wide daemon settings — `daemon_settings` table in runtime.db.
//!
//! Replaces the `settings` namespace in kv.db. Daemon-managed values are
//! per-machine execution state, so they live in the Runtime scope db
//! (`runtime.db`) alongside the actor/session/process projections — never in
//! the dead platform.db. KV is reserved for L2+ user-app state.
//!
//! API mirrors `db::kv` (get/set/delete/list_all) but takes only a key —
//! there's no namespace to register.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

use crate::db::{block_on, open_runtime_db};

/// Open the Runtime scope db backing `daemon_settings`. `open_runtime_db`
/// ensures the events + projection schema (which includes the
/// `daemon_settings` table) on a missing file, so callers need no separate
/// ensure step.
fn open_settings_db() -> Result<sqlx::SqliteConnection> {
    open_runtime_db(&crate::project::get_global_dir()?)
}

pub fn set(key: &str, value: &serde_json::Value) -> Result<()> {
    let mut conn = open_settings_db()?;
    let now = Utc::now().to_rfc3339();
    let value_json = serde_json::to_string(value)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO daemon_settings (key, value_json, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET
               value_json = excluded.value_json,
               updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(&value_json)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get(key: &str) -> Result<Option<serde_json::Value>> {
    let mut conn = open_settings_db()?;
    let row = block_on(async {
        sqlx::query("SELECT value_json FROM daemon_settings WHERE key = ?")
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

pub fn delete(key: &str) -> Result<()> {
    let mut conn = open_settings_db()?;
    block_on(async {
        sqlx::query("DELETE FROM daemon_settings WHERE key = ?")
            .bind(key)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn list_all() -> Result<Vec<(String, serde_json::Value)>> {
    let mut conn = open_settings_db()?;
    let rows = block_on(async {
        sqlx::query("SELECT key, value_json FROM daemon_settings ORDER BY key")
            .fetch_all(&mut conn)
            .await
    })?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let key: String = row.get(0);
        let json_str: String = row.get(1);
        let value: serde_json::Value = serde_json::from_str(&json_str)?;
        out.push((key, value));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::init_project_minimal as init_project;
    use serde_json::json;
    use tempfile::tempdir;

    fn setup() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        init_project(tmp.path()).unwrap();
        // get/set/delete self-provision runtime.db via open_settings_db.
        tmp
    }

    #[test]
    fn missing_returns_none() {
        let _tmp = setup();
        let key = format!("missing-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        assert!(get(&key).unwrap().is_none());
    }

    #[test]
    fn set_and_get_roundtrip() {
        let _tmp = setup();
        let key = format!("daemon.port-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        set(&key, &json!("9315")).unwrap();
        let val = get(&key).unwrap().unwrap();
        assert_eq!(val, json!("9315"));
    }

    #[test]
    fn set_overwrites() {
        let _tmp = setup();
        let key = format!("theme-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        set(&key, &json!("dark")).unwrap();
        set(&key, &json!("light")).unwrap();
        assert_eq!(get(&key).unwrap().unwrap(), json!("light"));
    }

    #[test]
    fn delete_removes_row() {
        let _tmp = setup();
        let key = format!("scratch-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        set(&key, &json!(true)).unwrap();
        delete(&key).unwrap();
        assert!(get(&key).unwrap().is_none());
    }
}
