//! Read-only `platform.db` migration drift diagnostic.
//!
//! With the tolerant runner (`db::migrate`) a database whose `_sqlx_migrations`
//! log is *ahead* of the binary is no longer a failure — the runner skips the
//! extra versions. The mutating escape hatches that used to live here
//! (`mark_applied`, `prune_orphans`, and checksum patching) were load-bearing
//! ONLY because the old strict runner crashed on that drift. They are gone.
//!
//! What remains is a pure diagnostic: [`migration_drift`] reports how the
//! DB's `_sqlx_migrations` log relates to the migration set this binary
//! embeds, changing nothing. It is the tool for *understanding* a startup
//! migration error — the one drift the runner still rejects, an applied
//! migration whose checksum no longer matches its embedded source.

use anyhow::{Context, Result};
use sqlx::migrate::Migrator;
use sqlx::{Row, SqliteConnection};
use std::collections::{HashMap, HashSet};

/// The embedded migration set. Same source `db::migrate` runs.
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// How the DB's migration log relates to the binary's embedded set.
#[derive(Debug, Default)]
pub struct DriftReport {
    /// Versions applied to the DB and embedded in this binary, checksums
    /// matching — healthy.
    pub in_sync: Vec<i64>,
    /// Versions embedded in this binary but not yet applied. The next
    /// `ensure_db` applies them, in version order.
    pub pending: Vec<i64>,
    /// Versions in the DB log this binary does not embed. Tolerated by the
    /// migration runner (the DB is simply ahead) — informational only.
    pub db_ahead: Vec<i64>,
    /// Versions applied to the DB AND embedded, but with a differing
    /// checksum. This is the one drift the runner rejects: an applied
    /// migration was edited. Recovery is to restore the original file
    /// content (or, pre-launch, recreate the DB) — never to edit the log.
    pub checksum_mismatch: Vec<i64>,
}

impl DriftReport {
    /// True when the DB and binary are fully reconciled.
    pub fn is_clean(&self) -> bool {
        self.pending.is_empty()
            && self.db_ahead.is_empty()
            && self.checksum_mismatch.is_empty()
    }
}

/// Inspect the DB's `_sqlx_migrations` log against the embedded migration
/// set. Pure read — touches no table, mutates nothing.
pub async fn migration_drift(conn: &mut SqliteConnection) -> Result<DriftReport> {
    let mut embedded: HashMap<i64, Vec<u8>> = HashMap::new();
    for m in MIGRATOR.iter() {
        embedded.insert(m.version, m.checksum.to_vec());
    }

    let rows = sqlx::query("SELECT version, checksum FROM _sqlx_migrations ORDER BY version")
        .fetch_all(&mut *conn)
        .await
        .context("read _sqlx_migrations")?;

    let mut report = DriftReport::default();
    let mut applied: HashSet<i64> = HashSet::new();
    for row in rows {
        let version: i64 = row.try_get("version")?;
        let recorded: Vec<u8> = row.try_get("checksum")?;
        applied.insert(version);
        match embedded.get(&version) {
            None => report.db_ahead.push(version),
            Some(expected) if expected == &recorded => report.in_sync.push(version),
            Some(_) => report.checksum_mismatch.push(version),
        }
    }
    for &version in embedded.keys() {
        if !applied.contains(&version) {
            report.pending.push(version);
        }
    }
    report.pending.sort();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ensure_db_at, open_db_at, platform_db_path};
    use crate::testutil::init_project_minimal;
    use tempfile::tempdir;

    fn fresh_db() -> (tempfile::TempDir, SqliteConnection) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project_minimal(tmp.path()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        let conn = open_db_at(&platform_db_path(&ship_dir)).unwrap();
        (tmp, conn)
    }

    async fn insert_orphan(conn: &mut SqliteConnection, version: i64) {
        sqlx::query(
            "INSERT INTO _sqlx_migrations \
             (version, description, installed_on, success, checksum, execution_time) \
             VALUES (?, 'orphan', CURRENT_TIMESTAMP, 1, X'00', 0)",
        )
        .bind(version)
        .execute(&mut *conn)
        .await
        .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fresh_db_is_clean() {
        let (_tmp, mut conn) = fresh_db();
        let report = migration_drift(&mut conn).await.unwrap();
        assert!(report.is_clean(), "a freshly-migrated DB has no drift");
        assert!(!report.in_sync.is_empty(), "embedded migrations are recorded");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn db_ahead_versions_are_reported_not_mutated() {
        let (_tmp, mut conn) = fresh_db();
        insert_orphan(&mut conn, 9_000_000_001).await;
        insert_orphan(&mut conn, 9_000_000_002).await;

        let report = migration_drift(&mut conn).await.unwrap();
        assert!(report.db_ahead.contains(&9_000_000_001));
        assert!(report.db_ahead.contains(&9_000_000_002));
        assert!(!report.is_clean());

        // Pure diagnostic: the rows it reported are still present.
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE version IN (9000000001, 9000000002)",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 2, "migration_drift must not mutate the log");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pending_and_checksum_mismatch_are_distinguished() {
        let (_tmp, mut conn) = fresh_db();

        // Drop a row -> pending.
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

        // Corrupt a different row's checksum -> checksum_mismatch.
        let min_version: i64 =
            sqlx::query_scalar("SELECT MIN(version) FROM _sqlx_migrations")
                .fetch_one(&mut conn)
                .await
                .unwrap();
        sqlx::query("UPDATE _sqlx_migrations SET checksum = X'deadbeef' WHERE version = ?")
            .bind(min_version)
            .execute(&mut conn)
            .await
            .unwrap();

        let report = migration_drift(&mut conn).await.unwrap();
        assert!(report.pending.contains(&max_version));
        assert!(report.checksum_mismatch.contains(&min_version));
    }
}
