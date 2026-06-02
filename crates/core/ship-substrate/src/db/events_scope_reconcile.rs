//! One-shot pre-migration fixup: bring an existing `events` table up to the
//! `(scope, scope_key)` partition tuple (D0).
//!
//! Why this lives outside `migrations/`: the `events` partition column is part
//! of the *living* `0001` baseline — its shape is edited in place rather than
//! appended as a versioned migration, which would trip the strict runner's
//! "applied migration was modified" guard (see `db::migrate`). Existing DBs
//! created from an older baseline carry the legacy single column
//! (`space_id`, or the even older `project_id`/`workspace_id`); they need the
//! `scope`/`scope_key` columns added and backfilled before `sqlx::migrate!`
//! and the canonical `INSERT INTO events (... scope, scope_key ...)` run.
//!
//! Contract:
//! * No-op when the `events` table is absent (fresh DB — sqlx_migrate creates
//!   it from the canonical 0001 schema, which already has the tuple).
//! * No-op when `events.scope` already exists.
//! * Otherwise: add `scope` + `scope_key`, map the legacy key column
//!   (`space_id`/`project_id`/`workspace_id`) → `(Space, <key>)`, and infer the
//!   scope of keyless rows from the event-type registry (Account/System
//!   events; everything else fail-closed to Device). Rebuild the partition
//!   index.
//! * Returns an error only if SQLite refuses the DDL; we never silently mask
//!   schema corruption.

use anyhow::Result;
use sqlx::SqliteConnection;

use ship_types::event_registry::{self, EventScope};

use crate::db::block_on;

/// Names of every registered event whose scope is `target`. Used to backfill
/// the `scope` of keyless legacy rows from the event-type registry rather than
/// a hand-maintained list (drift-free).
fn registered_names(target: EventScope) -> Vec<&'static str> {
    event_registry::registry()
        .filter(|d| d.scope == target)
        .map(|d| d.name)
        .collect()
}

/// Build `event_type IN ('a','b',...)` for an UPDATE predicate. Names come from
/// the compile-time registry (fixed literals), never user input, so inlining is
/// safe. Returns `None` when the list is empty (no predicate to run).
fn event_type_in_clause(names: &[&str]) -> Option<String> {
    if names.is_empty() {
        return None;
    }
    let quoted: Vec<String> = names.iter().map(|n| format!("'{n}'")).collect();
    Some(format!("event_type IN ({})", quoted.join(", ")))
}

pub fn reconcile_events_scope_columns(conn: &mut SqliteConnection) -> Result<()> {
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

    let has_scope: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='scope'")
            .fetch_one(&mut *conn)
            .await
    })?;
    if has_scope > 0 {
        return Ok(());
    }

    // Identify the legacy single key column. Newest-first: `space_id`, then the
    // intermediate `project_id`, then the original `workspace_id`.
    let legacy_col = {
        let mut found = None;
        for candidate in ["space_id", "project_id", "workspace_id"] {
            let present: i64 = block_on(async {
                sqlx::query_scalar(&format!(
                    "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='{candidate}'"
                ))
                .fetch_one(&mut *conn)
                .await
            })?;
            if present > 0 {
                found = Some(candidate);
                break;
            }
        }
        found
    };

    // Add the tuple columns. `scope` defaults to 'device' (fail-closed) so the
    // NOT NULL canonical insert path is satisfied even for rows we don't reclassify.
    block_on(async {
        sqlx::query("ALTER TABLE events ADD COLUMN scope TEXT NOT NULL DEFAULT 'device'")
            .execute(&mut *conn)
            .await
    })?;
    block_on(async {
        sqlx::query("ALTER TABLE events ADD COLUMN scope_key TEXT")
            .execute(&mut *conn)
            .await
    })?;

    if let Some(col) = legacy_col {
        // Legacy key present → those rows are space-scoped: (Space, <key>).
        block_on(async {
            sqlx::query(&format!(
                "UPDATE events SET scope = 'space', scope_key = {col} WHERE {col} IS NOT NULL"
            ))
            .execute(&mut *conn)
            .await
        })?;
    }

    // Keyless rows default to Device; reclassify the Account/System families
    // from the registry so daemon/platform + account events land on the right
    // partition (matches `EventEnvelope::new`'s registry-stamped scope).
    for (scope, names) in [
        (EventScope::Account, registered_names(EventScope::Account)),
        (EventScope::System, registered_names(EventScope::System)),
    ] {
        let Some(in_clause) = event_type_in_clause(&names) else {
            continue;
        };
        block_on(async {
            sqlx::query(&format!(
                "UPDATE events SET scope = '{}' WHERE scope_key IS NULL AND {in_clause}",
                scope.as_str()
            ))
            .execute(&mut *conn)
            .await
        })?;
    }

    // Rebuild the partition index under its canonical name. Drop any stale
    // legacy index first (fixed literals — safe to inline).
    for stale in ["idx_events_project", "idx_events_workspace"] {
        block_on(async {
            sqlx::query(&format!("DROP INDEX IF EXISTS {stale}"))
                .execute(&mut *conn)
                .await
        })?;
    }
    block_on(async {
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_events_scope ON events(scope, scope_key, created_at)",
        )
        .execute(&mut *conn)
        .await
    })?;
    tracing::info!(
        legacy_col = legacy_col.unwrap_or("<none>"),
        "events table migrated to (scope, scope_key) tuple (pre-migration reconcile)"
    );
    Ok(())
}

/// Idempotent reconcile for the Phase-2 author-identity + signature columns.
///
/// Mirrors [`reconcile_events_scope_columns`]: the three columns live in the
/// living 0001 baseline (additive, edited in place — not a forward migration,
/// which would trip the strict runner's checksum guard). A pre-existing DB
/// (15-col events table) gets `author_account_id`, `author_device_id`,
/// `signature` added if absent; a fresh DB already has them from the DDL.
/// No-op when the `events` table is absent or the columns already exist.
pub fn reconcile_events_author_columns(conn: &mut SqliteConnection) -> Result<()> {
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

    for col in ["author_account_id", "author_device_id", "signature"] {
        let present: i64 = block_on(async {
            sqlx::query_scalar(&format!(
                "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='{col}'"
            ))
            .fetch_one(&mut *conn)
            .await
        })?;
        if present == 0 {
            block_on(async {
                sqlx::query(&format!("ALTER TABLE events ADD COLUMN {col} TEXT"))
                    .execute(&mut *conn)
                    .await
            })?;
        }
    }
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
        reconcile_events_scope_columns(&mut conn).unwrap();
    }

    #[test]
    fn noop_when_scope_already_present() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("fresh.db");
        let mut conn = open(&db);
        block_on(async {
            sqlx::query(
                "CREATE TABLE events (id TEXT PRIMARY KEY, scope TEXT, scope_key TEXT, \
                 event_type TEXT, created_at TEXT)",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();

        reconcile_events_scope_columns(&mut conn).unwrap();

        let has_scope: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='scope'")
                .fetch_one(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(has_scope, 1);
    }

    #[test]
    fn splits_space_id_into_scope_tuple_and_infers_keyless() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("legacy.db");
        let mut conn = open(&db);
        block_on(async {
            sqlx::query(
                "CREATE TABLE events (id TEXT PRIMARY KEY, event_type TEXT, space_id TEXT, \
                 created_at TEXT)",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();
        // A space-scoped row (carries a key), an account row, a system row, and
        // a device row — the last three keyless.
        for (id, et, space) in [
            ("e1", "artifact.created", Some("sp-1")),
            ("e2", "account.login_completed", None),
            ("e3", "daemon.startup_completed", None),
            ("e4", "actor.created", None),
        ] {
            block_on(async {
                sqlx::query("INSERT INTO events (id, event_type, space_id, created_at) VALUES (?,?,?,?)")
                    .bind(id)
                    .bind(et)
                    .bind(space)
                    .bind("2026-05-14T00:00:00Z")
                    .execute(&mut conn)
                    .await
            })
            .unwrap();
        }

        reconcile_events_scope_columns(&mut conn).unwrap();

        let row = |id: &str, conn: &mut SqliteConnection| -> (String, Option<String>) {
            let id = id.to_string();
            block_on(async {
                sqlx::query_as::<_, (String, Option<String>)>(
                    "SELECT scope, scope_key FROM events WHERE id = ?",
                )
                .bind(id)
                .fetch_one(&mut *conn)
                .await
            })
            .unwrap()
        };
        assert_eq!(row("e1", &mut conn), ("space".into(), Some("sp-1".into())));
        assert_eq!(row("e2", &mut conn), ("account".into(), None));
        assert_eq!(row("e3", &mut conn), ("system".into(), None));
        assert_eq!(row("e4", &mut conn), ("device".into(), None));

        // Idempotent on second call.
        reconcile_events_scope_columns(&mut conn).unwrap();
    }

    // T7 (reconcile path) — a pre-existing 15-col events table gets the three
    // author/signature columns added idempotently; existing rows read back
    // with NULL for the new columns.
    #[test]
    fn author_reconcile_adds_columns_idempotently() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("author.db");
        let mut conn = open(&db);
        block_on(async {
            sqlx::query(
                "CREATE TABLE events (id TEXT PRIMARY KEY, event_type TEXT, scope TEXT, \
                 scope_key TEXT, created_at TEXT)",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();
        block_on(async {
            sqlx::query(
                "INSERT INTO events (id, event_type, scope, scope_key, created_at) \
                 VALUES ('e1','artifact.created','space','sp-1','2026-05-14T00:00:00Z')",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();

        reconcile_events_author_columns(&mut conn).unwrap();
        // Idempotent second call.
        reconcile_events_author_columns(&mut conn).unwrap();

        for col in ["author_account_id", "author_device_id", "signature"] {
            let present: i64 = block_on(async {
                sqlx::query_scalar(&format!(
                    "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name='{col}'"
                ))
                .fetch_one(&mut conn)
                .await
            })
            .unwrap();
            assert_eq!(present, 1, "{col} must be present after reconcile");
        }
        let sig: Option<String> = block_on(async {
            sqlx::query_scalar("SELECT signature FROM events WHERE id='e1'")
                .fetch_one(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(sig, None);
    }

    #[test]
    fn renames_legacy_workspace_id_into_scope_tuple() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("oldest.db");
        let mut conn = open(&db);
        block_on(async {
            sqlx::query(
                "CREATE TABLE events (id TEXT PRIMARY KEY, event_type TEXT, workspace_id TEXT, \
                 created_at TEXT)",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();
        block_on(async {
            sqlx::query("INSERT INTO events (id, event_type, workspace_id, created_at) VALUES ('e1','artifact.created','ws-1','2026-05-14T00:00:00Z')")
                .execute(&mut conn)
                .await
        })
        .unwrap();

        reconcile_events_scope_columns(&mut conn).unwrap();

        let (scope, key): (String, Option<String>) = block_on(async {
            sqlx::query_as("SELECT scope, scope_key FROM events WHERE id='e1'")
                .fetch_one(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(scope, "space");
        assert_eq!(key.as_deref(), Some("ws-1"));
    }
}
