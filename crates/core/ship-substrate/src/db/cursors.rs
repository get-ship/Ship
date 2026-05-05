//! Bookmarks — sync push/pull positions and per-projection rebuild cursors.
//!
//! Backed by the `cursors` table in `platform.db`. Names are opaque strings
//! the caller chooses; the table has no opinion. Conventions used today:
//! - `push:platform:{project_id}` — sync push cursor per project
//! - `pull:platform:{project_id}` — sync pull cursor per project
//! - `projection:{name}` — per-projection rebuild cursor
//!
//! Living in platform.db (next to the event log) means cursor advances can
//! be transactional with event reads/writes when needed.

use anyhow::Result;
use sqlx::Row;

use crate::db::{block_on, ensure_db, open_db};

/// Get the last-applied event id for a named cursor.
pub fn get(name: &str) -> Result<Option<String>> {
    ensure_db()?;
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query("SELECT last_event_id FROM cursors WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.map(|r| r.get::<String, _>(0)))
}

/// Upsert a cursor. `updated_at` is set to now() automatically.
pub fn set(name: &str, last_event_id: &str) -> Result<()> {
    ensure_db()?;
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query(
            "INSERT INTO cursors (name, last_event_id, updated_at)
             VALUES (?, ?, datetime('now'))
             ON CONFLICT(name) DO UPDATE SET
               last_event_id = excluded.last_event_id,
               updated_at    = excluded.updated_at",
        )
        .bind(name)
        .bind(last_event_id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

/// Delete a cursor. No-op if it doesn't exist.
pub fn delete(name: &str) -> Result<()> {
    ensure_db()?;
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("DELETE FROM cursors WHERE name = ?")
            .bind(name)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::init_project_minimal as init_project;
    use tempfile::tempdir;

    fn setup() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        tmp
    }

    #[test]
    fn missing_returns_none() {
        let _tmp = setup();
        assert!(get("push:platform:proj-999").unwrap().is_none());
    }

    #[test]
    fn set_and_get_roundtrip() {
        let _tmp = setup();
        set("push:platform:proj-1", "01J0000000000000000000000A").unwrap();
        assert_eq!(
            get("push:platform:proj-1").unwrap().unwrap(),
            "01J0000000000000000000000A"
        );
    }

    #[test]
    fn set_overwrites_previous() {
        let _tmp = setup();
        set("projection:auth_sessions", "01J0000000000000000000000A").unwrap();
        set("projection:auth_sessions", "01J0000000000000000000000B").unwrap();
        assert_eq!(
            get("projection:auth_sessions").unwrap().unwrap(),
            "01J0000000000000000000000B"
        );
    }

    #[test]
    fn names_are_isolated() {
        let _tmp = setup();
        set("push:platform:proj-1", "AAA").unwrap();
        set("projection:actors", "BBB").unwrap();
        assert_eq!(get("push:platform:proj-1").unwrap().unwrap(), "AAA");
        assert_eq!(get("projection:actors").unwrap().unwrap(), "BBB");
    }

    #[test]
    fn delete_removes_row() {
        let _tmp = setup();
        set("push:platform:proj-1", "AAA").unwrap();
        delete("push:platform:proj-1").unwrap();
        assert!(get("push:platform:proj-1").unwrap().is_none());
    }
}
