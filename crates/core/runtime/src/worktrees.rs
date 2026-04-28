//! Worktree registry — kv-backed record of dev-flow worktree attachments.
//!
//! A worktree pairs a branch (or arbitrary key) with a filesystem path and
//! optional tmux session, used by the dev workflow to give each agent a
//! conflict-free checkout. Worktrees are NOT a runtime primitive; they're
//! an attachment that exists only when an actor wants one. The actor itself
//! owns isolation (cwd, permissions). This registry just lets us look up
//! "what path was attached to this branch."
//!
//! Storage is `kv_state` under namespace `worktree`. One row per branch.
//! Replaces the former `workspace` projection table; the workspace.* events
//! still drive writes here so the event log stays canonical.
//!
//! Workspace-retirement slice 05.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::kv;

const NAMESPACE: &str = "worktree";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeRecord {
    pub branch: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tmux_session_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Insert or update a worktree record. Idempotent.
pub fn upsert(record: &WorktreeRecord) -> Result<()> {
    let value = serde_json::to_value(record).context("serialize worktree record")?;
    kv::set(NAMESPACE, &record.branch, &value)
}

/// Same as [`upsert`] but reuses a caller-provided sqlite connection.
/// Used by the workspace projection to avoid lock contention with its
/// own write transaction.
pub fn upsert_with_conn(
    conn: &mut sqlx::SqliteConnection,
    record: &WorktreeRecord,
) -> Result<()> {
    let value = serde_json::to_value(record).context("serialize worktree record")?;
    kv::set_with_conn(conn, NAMESPACE, &record.branch, &value)
}

/// Same as [`forget`] but reuses a caller-provided sqlite connection.
pub fn forget_with_conn(conn: &mut sqlx::SqliteConnection, branch: &str) -> Result<()> {
    kv::delete_with_conn(conn, NAMESPACE, branch)
}

/// Read a worktree record using a caller-provided connection.
pub fn get_with_conn(
    conn: &mut sqlx::SqliteConnection,
    branch: &str,
) -> Result<Option<WorktreeRecord>> {
    use sqlx::Row;
    let row = crate::db::block_on(async {
        sqlx::query("SELECT value_json FROM kv_state WHERE namespace = ? AND key = ?")
            .bind(NAMESPACE)
            .bind(branch)
            .fetch_optional(&mut *conn)
            .await
    })?;
    match row {
        None => Ok(None),
        Some(r) => {
            let json_str: String = r.get(0);
            let value: serde_json::Value = serde_json::from_str(&json_str)?;
            Ok(Some(serde_json::from_value(value)?))
        }
    }
}

/// Get a worktree record by branch.
pub fn get(branch: &str) -> Result<Option<WorktreeRecord>> {
    let Some(value) = kv::get(NAMESPACE, branch)? else {
        return Ok(None);
    };
    let record: WorktreeRecord =
        serde_json::from_value(value).context("deserialize worktree record")?;
    Ok(Some(record))
}

/// Get a worktree record by id (or branch as fallback).
pub fn get_by_id_or_branch(id: &str) -> Result<Option<WorktreeRecord>> {
    if let Some(record) = get(id)? {
        return Ok(Some(record));
    }
    let entries = kv::list_all(NAMESPACE)?;
    for (_, value) in entries {
        if let Ok(record) = serde_json::from_value::<WorktreeRecord>(value)
            && record.id == id
        {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

/// List all worktree records, sorted by `created_at` desc.
pub fn list() -> Result<Vec<WorktreeRecord>> {
    let entries = kv::list_all(NAMESPACE)?;
    let mut out: Vec<WorktreeRecord> = entries
        .into_iter()
        .filter_map(|(_, v)| serde_json::from_value(v).ok())
        .collect();
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

/// Remove a worktree record. Does not delete on-disk artifacts.
pub fn forget(branch: &str) -> Result<()> {
    kv::delete(NAMESPACE, branch)
}

/// Patch the path on an existing worktree (no-op if absent).
pub fn set_path(branch: &str, path: Option<&str>) -> Result<()> {
    let Some(mut existing) = get(branch)? else {
        return Ok(());
    };
    existing.path = path.map(str::to_string);
    existing.updated_at = Utc::now().to_rfc3339();
    upsert(&existing)
}

/// Patch the tmux_session_name on an existing worktree (no-op if absent).
pub fn set_tmux_session(branch: &str, name: Option<&str>) -> Result<()> {
    let Some(mut existing) = get(branch)? else {
        return Ok(());
    };
    existing.tmux_session_name = name.map(str::to_string);
    existing.updated_at = Utc::now().to_rfc3339();
    upsert(&existing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh_db() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        crate::db::ensure_db().unwrap();
        tmp
    }

    fn record(branch: &str, id: &str) -> WorktreeRecord {
        let now = Utc::now().to_rfc3339();
        WorktreeRecord {
            branch: branch.to_string(),
            id: id.to_string(),
            path: None,
            tmux_session_name: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    #[test]
    fn upsert_and_get_round_trip() {
        let _tmp = fresh_db();
        upsert(&record("feat/x", "feat-x")).unwrap();
        let got = get("feat/x").unwrap().expect("present");
        assert_eq!(got.id, "feat-x");
    }

    #[test]
    fn set_path_updates_existing() {
        let _tmp = fresh_db();
        upsert(&record("feat/p", "feat-p")).unwrap();
        set_path("feat/p", Some("/worktrees/p")).unwrap();
        let got = get("feat/p").unwrap().expect("present");
        assert_eq!(got.path.as_deref(), Some("/worktrees/p"));
    }

    #[test]
    fn forget_removes_record() {
        let _tmp = fresh_db();
        upsert(&record("feat/g", "feat-g")).unwrap();
        forget("feat/g").unwrap();
        assert!(get("feat/g").unwrap().is_none());
    }

    #[test]
    fn get_by_id_falls_back_to_branch_lookup() {
        let _tmp = fresh_db();
        upsert(&record("feat/branch", "feat-branch")).unwrap();
        let by_id = get_by_id_or_branch("feat-branch").unwrap();
        assert!(by_id.is_some());
        let by_branch = get_by_id_or_branch("feat/branch").unwrap();
        assert!(by_branch.is_some());
    }
}
