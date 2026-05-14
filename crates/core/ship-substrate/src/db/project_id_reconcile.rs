//! One-shot pre-migration fixup: rename `events.workspace_id` →
//! `events.project_id` on DBs that predate the column rename in commit
//! 0b52fa28.
//!
//! Why this lives outside `migrations/`: the rename was applied by
//! patching the 0001 baseline in place rather than introducing a versioned
//! migration. DBs created from the old baseline still carry
//! `workspace_id`; `sqlx::migrate!` sees migration 1 already applied and
//! does no further work, leaving `INSERT INTO events (... project_id ...)`
//! to fail at runtime with `no such column`.
//!
//! Running this before `sqlx::migrate!` keeps the fix idempotent and free
//! of new migration files (a new file would itself need conditional logic
//! since SQLite has no conditional DDL).
//!
//! Contract:
//! * No-op when the `events` table is absent (fresh DB — sqlx_migrate
//!   creates it from the canonical 0001 schema).
//! * No-op when `events.project_id` already exists.
//! * Renames `workspace_id` to `project_id` and rebuilds the supporting
//!   index when only the old column is present.
//! * Returns an error only if SQLite refuses the rename (e.g. version <
//!   3.25); we never silently mask schema corruption.

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::db::block_on;

pub fn reconcile_events_project_id_column(conn: &mut SqliteConnection) -> Result<()> {
    let table_present: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'",
        )
        .fetch_one(&mut *conn)
        .await
    })?;
    if table_present == 0 {
        return Ok(());
    }

    let has_project_id: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='project_id'",
        )
        .fetch_one(&mut *conn)
        .await
    })?;
    if has_project_id > 0 {
        return Ok(());
    }

    let has_workspace_id: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='workspace_id'",
        )
        .fetch_one(&mut *conn)
        .await
    })?;
    if has_workspace_id == 0 {
        // Table exists but has neither column — let sqlx_migrate surface
        // the real error; we don't want to silently mask schema corruption.
        return Ok(());
    }

    // The stale index is keyed on the old column name. Drop it before the
    // rename, then recreate it under the canonical name from 0001.
    block_on(async {
        sqlx::query("DROP INDEX IF EXISTS idx_events_workspace")
            .execute(&mut *conn)
            .await
    })?;
    block_on(async {
        sqlx::query("ALTER TABLE events RENAME COLUMN workspace_id TO project_id")
            .execute(&mut *conn)
            .await
    })?;
    block_on(async {
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_events_project ON events(project_id, created_at)",
        )
        .execute(&mut *conn)
        .await
    })?;
    tracing::info!("events.workspace_id renamed to events.project_id (pre-migration reconcile)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, sqlite::SqliteConnectOptions};
    use std::str::FromStr;
    use tempfile::tempdir;

    fn open(path: &std::path::Path) -> SqliteConnection {
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let opts = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
        block_on(async { SqliteConnection::connect_with(&opts).await }).unwrap()
    }

    #[test]
    fn noop_when_events_table_missing() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("noop.db");
        let mut conn = open(&db);
        // Should succeed without erroring or creating tables.
        reconcile_events_project_id_column(&mut conn).unwrap();
    }

    #[test]
    fn noop_when_project_id_already_present() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("fresh.db");
        let mut conn = open(&db);
        block_on(async {
            sqlx::query(
                "CREATE TABLE events (id TEXT PRIMARY KEY, project_id TEXT, created_at TEXT)",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();

        reconcile_events_project_id_column(&mut conn).unwrap();

        // Column unchanged.
        let has_pid: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='project_id'",
            )
            .fetch_one(&mut conn)
            .await
        })
        .unwrap();
        assert_eq!(has_pid, 1);
    }

    #[test]
    fn renames_workspace_id_to_project_id_and_preserves_rows() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("legacy.db");
        let mut conn = open(&db);
        block_on(async {
            sqlx::query(
                "CREATE TABLE events (id TEXT PRIMARY KEY, workspace_id TEXT, created_at TEXT)",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();
        block_on(async {
            sqlx::query("CREATE INDEX idx_events_workspace ON events(workspace_id, created_at)")
                .execute(&mut conn)
                .await
        })
        .unwrap();
        block_on(async {
            sqlx::query("INSERT INTO events (id, workspace_id, created_at) VALUES (?, ?, ?)")
                .bind("e1")
                .bind("ws-1")
                .bind("2026-05-14T00:00:00Z")
                .execute(&mut conn)
                .await
        })
        .unwrap();

        reconcile_events_project_id_column(&mut conn).unwrap();

        let pid: Option<String> = block_on(async {
            sqlx::query_scalar("SELECT project_id FROM events WHERE id='e1'")
                .fetch_one(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(pid.as_deref(), Some("ws-1"));

        let idx_count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_events_project'",
            )
            .fetch_one(&mut conn)
            .await
        })
        .unwrap();
        assert_eq!(idx_count, 1);

        // Idempotent on second call.
        reconcile_events_project_id_column(&mut conn).unwrap();
    }
}
