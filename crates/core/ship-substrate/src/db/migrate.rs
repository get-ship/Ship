//! Tolerant `platform.db` migration runner.
//!
//! `sqlx::migrate!`'s default runner is strict: it hard-errors when the
//! database's `_sqlx_migrations` log contains a version the current binary
//! doesn't embed ("migration N was previously applied but is missing in the
//! resolved migrations"). In Ship that failure is chronic, not exceptional —
//! `platform.db` is one shared store, and every dev worktree builds a
//! *different* version of Ship's own migration code and points it at that
//! store. Branch switches, agent-worktree dispatches, and migration-touching
//! merges all leave the DB log ahead of whichever binary boots next.
//!
//! The fix is to relax exactly that one failure mode. The runner's single
//! invariant is: **every migration the binary embeds is applied to the DB.**
//!
//! - embedded + applied, checksums match  -> skip
//! - embedded + applied, checksums differ -> hard error (an applied
//!   migration was edited — real corruption, not branch drift)
//! - embedded + not applied               -> apply, in version order
//! - applied + not embedded               -> tolerated no-op, logged DEBUG
//!
//! This behaviour is universally correct: a customer's DB is never ahead of
//! their binary, so the tolerance is inert for them — there is no `if dev`
//! fork. The relaxation is achieved with sqlx's own `Migrator` API
//! (`set_ignore_missing`), not a reimplementation of migration application.

use anyhow::{Result, anyhow};
use sqlx::SqliteConnection;
use sqlx::migrate::{Migrate, MigrateError};

/// Apply every embedded `platform.db` migration not yet recorded in the DB,
/// tolerating a database whose `_sqlx_migrations` log is *ahead* of this
/// binary. See the module docs for the exact invariant.
///
/// Errors only on genuine corruption: a dirty (partially-applied) migration,
/// or an applied migration whose checksum no longer matches the embedded
/// source.
pub async fn run_tolerant(conn: &mut SqliteConnection) -> Result<()> {
    // Owned, mutable copy of the compile-time migration set so we can flip
    // the one relax knob. `set_ignore_missing(true)` makes sqlx's runner
    // skip the "previously applied but missing" check — DB-log versions this
    // binary doesn't embed become inert instead of fatal. The checksum
    // check inside the run loop is untouched, so an edited applied migration
    // is still a hard `VersionMismatch`.
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);

    // Observability: `set_ignore_missing` silences DB-ahead versions
    // entirely, but an operator still wants the breadcrumb that a worktree's
    // binary is behind the shared store. Surface each one at DEBUG before
    // handing off to the runner.
    let embedded: std::collections::HashSet<i64> =
        migrator.iter().map(|m| m.version).collect();
    conn.ensure_migrations_table()
        .await
        .map_err(|e| anyhow!("ensure _sqlx_migrations table: {e}"))?;
    for applied in conn
        .list_applied_migrations()
        .await
        .map_err(|e| anyhow!("read _sqlx_migrations log: {e}"))?
    {
        if !embedded.contains(&applied.version) {
            tracing::debug!(
                version = applied.version,
                "platform.db migration log is ahead of this binary — version \
                 not in the resolved set; tolerated, no action taken"
            );
        }
    }

    migrator.run(&mut *conn).await.map_err(|e| match e {
        MigrateError::VersionMismatch(v) => anyhow!(
            "migration {v} checksum mismatch: an already-applied migration was \
             modified. Applied migrations are immutable — a change must be a \
             new migration, never an edit to a recorded one. Restore the \
             original file content, OR (pre-launch only, accepts data loss) \
             run `ship daemon repair-migrations --reset` to back up and \
             recreate platform.db so the next start re-applies migrations \
             from scratch."
        ),
        MigrateError::Dirty(v) => anyhow!(
            "migration {v} is dirty: a previous run failed partway through. \
             The database needs manual inspection — `ship daemon \
             repair-migrations` reports the recorded state."
        ),
        other => anyhow!("platform.db migration failed: {other}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ensure_db_at, open_db_at, platform_db_path};
    use crate::testutil::init_project_minimal;
    use tempfile::tempdir;

    /// `ensure_db_at` materializes `platform.db` (running the tolerant
    /// migrator once), then hands back a connection to it.
    fn fresh_db() -> (tempfile::TempDir, SqliteConnection) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project_minimal(tmp.path()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        let conn = open_db_at(&platform_db_path(&ship_dir)).unwrap();
        (tmp, conn)
    }

    /// A `platform.db` whose migration log carries an extra applied version
    /// the binary doesn't embed (the branch-switch / worktree-dispatch case)
    /// must migrate cleanly — the extra row is tolerated, not fatal.
    #[tokio::test(flavor = "multi_thread")]
    async fn db_ahead_of_binary_is_tolerated() {
        let (_tmp, mut conn) = fresh_db();

        // Inject a version far above anything this binary embeds, as if a
        // newer worktree's binary had applied it against the shared store.
        sqlx::query(
            "INSERT INTO _sqlx_migrations \
             (version, description, installed_on, success, checksum, execution_time) \
             VALUES (999999999999, 'from-the-future', CURRENT_TIMESTAMP, 1, X'00', 0)",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        // A strict runner would hard-error here; the tolerant one completes.
        run_tolerant(&mut conn).await.unwrap();
    }

    /// A `platform.db` missing an embedded migration gets it applied and
    /// recorded; a re-run is a no-op.
    #[tokio::test(flavor = "multi_thread")]
    async fn missing_migration_is_applied_and_recorded() {
        let (_tmp, mut conn) = fresh_db();

        let applied: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
                .fetch_one(&mut conn)
                .await
                .unwrap();
        assert!(applied > 0, "embedded migrations must be recorded");

        // Drop the highest-versioned migration's bookkeeping row so the
        // runner sees it as pending, then confirm it re-applies + re-records.
        let max_version: i64 =
            sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations")
                .fetch_one(&mut conn)
                .await
                .unwrap();
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = ?")
            .bind(max_version)
            .execute(&mut conn)
            .await
            .unwrap();

        run_tolerant(&mut conn).await.unwrap();
        let row = sqlx::query("SELECT version FROM _sqlx_migrations WHERE version = ?")
            .bind(max_version)
            .fetch_optional(&mut conn)
            .await
            .unwrap();
        assert!(row.is_some(), "pending migration must be re-applied + recorded");

        // Idempotent: a clean re-run changes nothing.
        run_tolerant(&mut conn).await.unwrap();
    }

    /// A version present in both the binary and the DB log but with a
    /// *different* checksum is real corruption — still a hard error.
    #[tokio::test(flavor = "multi_thread")]
    async fn checksum_mismatch_is_a_hard_error() {
        let (_tmp, mut conn) = fresh_db();

        // Corrupt the recorded checksum of an applied migration.
        let version: i64 =
            sqlx::query_scalar("SELECT MIN(version) FROM _sqlx_migrations")
                .fetch_one(&mut conn)
                .await
                .unwrap();
        sqlx::query("UPDATE _sqlx_migrations SET checksum = X'deadbeef' WHERE version = ?")
            .bind(version)
            .execute(&mut conn)
            .await
            .unwrap();

        let err = run_tolerant(&mut conn).await.unwrap_err();
        assert!(
            err.to_string().contains("checksum mismatch"),
            "expected a checksum-mismatch hard error, got: {err}"
        );
    }
}
