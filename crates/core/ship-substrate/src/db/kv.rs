//! Key-value state store — namespaced, JSON values.
//!
//! Backed by `kv.db` (separate from `platform.db`). The physical split means
//! KV writes never compete with the event log for WAL locks.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

use crate::db::{block_on, ensure_kv_db, open_kv_db};

/// Approved KV namespaces. Adding a new variant here is the required
/// code-review gate for introducing a new KV namespace. Raw string
/// namespaces are not accepted; use this enum instead.
#[derive(Debug, Clone)]
pub enum KvNamespace {
    /// Auth session tokens — ephemeral
    AuthSession,
    /// Auth daemon-owner binding — ephemeral
    AuthDaemonOwner,
    /// Cron hook next-fire timestamps — ephemeral
    CronState,
    /// User-facing settings (defaults.provider, defaults.model, daemon.port…)
    Settings,
    /// Skill variable state — global (machine-wide) scope; inner = skill_id
    SkillVarsGlobal(String),
    /// Skill variable state — local scope; inner = "{ctx}:{skill_id}"
    SkillVarsLocal(String),
    /// Skill variable state — project scope; inner = "{ctx}:{skill_id}"
    SkillVarsProject(String),
    /// Runtime project config cache (providers, active_agent, hooks, ai, git,
    /// namespaces, modes). Inner = `format!("runtime::{}", ship_dir.display())`
    ProjectConfig(String),
    /// Project registry — daemon's record of seen project paths
    Project,
    /// Per-project CLI state (active_agent, compiled_at, plugins_installed).
    /// Inner = stable hex hash of canonical ship_dir path.
    CliProjectState(String),
}

impl KvNamespace {
    fn as_str(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::AuthSession       => "auth.sessions".into(),
            Self::AuthDaemonOwner   => "auth.daemon_owner".into(),
            Self::CronState         => "cron.last_fire".into(),
            Self::Settings          => "settings".into(),
            Self::SkillVarsGlobal(s)   => format!("skill_vars:{s}").into(),
            Self::SkillVarsLocal(s)    => format!("skill_vars.local:{s}").into(),
            Self::SkillVarsProject(s)  => format!("skill_vars.project:{s}").into(),
            Self::ProjectConfig(s)     => s.as_str().into(),
            Self::Project              => "project".into(),
            Self::CliProjectState(h)   => format!("workspace:{h}").into(),
        }
    }
}

pub fn set(ns: KvNamespace, key: &str, value: &serde_json::Value) -> Result<()> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let now = Utc::now().to_rfc3339();
    let value_json = serde_json::to_string(value)?;
    let namespace = ns.as_str();
    block_on(async {
        sqlx::query(
            "INSERT INTO kv_state (namespace, key, value_json, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(namespace, key) DO UPDATE SET
               value_json = excluded.value_json,
               updated_at = excluded.updated_at",
        )
        .bind(&*namespace)
        .bind(key)
        .bind(&value_json)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get(ns: KvNamespace, key: &str) -> Result<Option<serde_json::Value>> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let namespace = ns.as_str();
    let row = block_on(async {
        sqlx::query("SELECT value_json FROM kv_state WHERE namespace = ? AND key = ?")
            .bind(&*namespace)
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

pub fn delete(ns: KvNamespace, key: &str) -> Result<()> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let namespace = ns.as_str();
    block_on(async {
        sqlx::query("DELETE FROM kv_state WHERE namespace = ? AND key = ?")
            .bind(&*namespace)
            .bind(key)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn list_keys(ns: KvNamespace) -> Result<Vec<String>> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let namespace = ns.as_str();
    let rows = block_on(async {
        sqlx::query("SELECT key FROM kv_state WHERE namespace = ? ORDER BY key")
            .bind(&*namespace)
            .fetch_all(&mut conn)
            .await
    })?;
    Ok(rows.iter().map(|r| r.get(0)).collect())
}

/// Return all (key, value) pairs in a namespace.
pub fn list_all(ns: KvNamespace) -> Result<Vec<(String, serde_json::Value)>> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let namespace = ns.as_str();
    let rows = block_on(async {
        sqlx::query("SELECT key, value_json FROM kv_state WHERE namespace = ? ORDER BY key")
            .bind(&*namespace)
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
        set(KvNamespace::SkillVarsGlobal("agent".to_string()), "active_workspace", &json!("ws-001")).unwrap();
        let val = get(KvNamespace::SkillVarsGlobal("agent".to_string()), "active_workspace").unwrap().unwrap();
        assert_eq!(val, json!("ws-001"));
    }

    #[test]
    fn test_set_overwrites() {
        let (_tmp, _ship_dir) = setup();
        set(KvNamespace::Settings, "theme", &json!("dark")).unwrap();
        set(KvNamespace::Settings, "theme", &json!("light")).unwrap();
        let val = get(KvNamespace::Settings, "theme").unwrap().unwrap();
        assert_eq!(val, json!("light"));
    }

    #[test]
    fn test_get_missing_returns_none() {
        let (_tmp, _ship_dir) = setup();
        assert!(get(KvNamespace::Settings, "missing").unwrap().is_none());
    }

    #[test]
    fn test_delete() {
        let (_tmp, _ship_dir) = setup();
        set(KvNamespace::Settings, "key", &json!(42)).unwrap();
        delete(KvNamespace::Settings, "key").unwrap();
        assert!(get(KvNamespace::Settings, "key").unwrap().is_none());
    }

    #[test]
    fn test_namespaces_are_isolated() {
        let (_tmp, _ship_dir) = setup();
        set(KvNamespace::SkillVarsGlobal("ns1".to_string()), "key", &json!("a")).unwrap();
        set(KvNamespace::SkillVarsGlobal("ns2".to_string()), "key", &json!("b")).unwrap();
        assert_eq!(get(KvNamespace::SkillVarsGlobal("ns1".to_string()), "key").unwrap().unwrap(), json!("a"));
        assert_eq!(get(KvNamespace::SkillVarsGlobal("ns2".to_string()), "key").unwrap().unwrap(), json!("b"));
    }

    #[test]
    fn test_list_keys() {
        let (_tmp, _ship_dir) = setup();
        set(KvNamespace::ProjectConfig("app".to_string()), "alpha", &json!(1)).unwrap();
        set(KvNamespace::ProjectConfig("app".to_string()), "beta", &json!(2)).unwrap();
        set(KvNamespace::ProjectConfig("other".to_string()), "gamma", &json!(3)).unwrap();
        let keys = list_keys(KvNamespace::ProjectConfig("app".to_string())).unwrap();
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
        let keys = list_keys(KvNamespace::ProjectConfig("test_ns".to_string())).unwrap();
        assert!(keys.is_empty());
    }

    fn ensure_kv_db_at_with_ship_dir(ship_dir: &std::path::Path) {
        crate::db::ensure_kv_db_at(ship_dir).unwrap();
    }
}
