//! Non-destructive migration repair.
//!
//! `sqlx::migrate!` records a SHA-384 of every applied migration and
//! refuses to run when the on-disk file's hash diverges. The protection
//! is real but the recovery path was missing — until now the only escape
//! was to delete platform.db and lose data.
//!
//! `repair_migrations` reconciles the recorded checksums against the
//! current files. For every migration the database knows about, it
//! recomputes the file hash and `UPDATE`s the recorded value. It does
//! NOT touch any other table; data is preserved in full.
//!
//! Use cases:
//! - merging a branch that intentionally edited a baseline migration
//!   (the freeze policy explicitly grandfathers 0001-0006)
//! - renaming a migration file (timestamp adoption, collision resolution)
//! - whitespace / comment-only edits that drift the hash without changing
//!   the schema
//!
//! NOT a substitute for adding new migrations when the schema actually
//! changes. If a recorded migration no longer has a matching file, the
//! repair leaves the row alone and surfaces it in the report — that's
//! the case where you DO need to think.

use anyhow::{Context, Result};
use sqlx::migrate::Migrator;
use sqlx::{Row, SqliteConnection};
use std::collections::HashMap;

/// The embedded migration set. Same source `ensure_db` runs.
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Default)]
pub struct RepairReport {
    /// Migrations whose recorded checksum was patched.
    pub updated: Vec<i64>,
    /// Migrations whose hash already matched — no action.
    pub unchanged: Vec<i64>,
    /// Migrations recorded in the DB with no matching embedded file.
    /// Surfaced for the operator to investigate; never auto-deleted.
    pub orphaned: Vec<i64>,
    /// Embedded migrations not yet applied. Reported for awareness;
    /// the next `ensure_db` will apply them.
    pub pending: Vec<i64>,
}

/// Recompute and patch checksums for every applied migration whose
/// embedded source is available. Idempotent. Never deletes rows.
///
/// The migration content read here is the same content `sqlx::migrate!`
/// embeds at compile time, so the recomputed checksum matches what a
/// fresh `ensure_db` on a clean DB would produce. After repair, the
/// migrator runs without complaint.
pub async fn repair_migrations(conn: &mut SqliteConnection) -> Result<RepairReport> {
    let mut report = RepairReport::default();

    // Index embedded migrations by version. sqlx already computes the
    // checksum (Vec<u8>) at compile time; reuse it.
    let mut embedded: HashMap<i64, Vec<u8>> = HashMap::new();
    for m in MIGRATOR.iter() {
        embedded.insert(m.version, m.checksum.to_vec());
    }

    let rows = sqlx::query("SELECT version, checksum FROM _sqlx_migrations ORDER BY version")
        .fetch_all(&mut *conn)
        .await
        .context("read _sqlx_migrations")?;

    let mut applied: Vec<i64> = Vec::with_capacity(rows.len());
    for row in rows {
        let version: i64 = row.try_get("version")?;
        let recorded: Vec<u8> = row.try_get("checksum")?;
        applied.push(version);

        let Some(expected) = embedded.get(&version) else {
            report.orphaned.push(version);
            continue;
        };

        if expected == &recorded {
            report.unchanged.push(version);
        } else {
            sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
                .bind(expected)
                .bind(version)
                .execute(&mut *conn)
                .await
                .with_context(|| format!("patch checksum for migration {version}"))?;
            report.updated.push(version);
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

/// Delete bookkeeping rows for the supplied versions from
/// `_sqlx_migrations`. Use only for versions returned in
/// `RepairReport::orphaned` — rows whose embedded migration file no
/// longer exists (typically because the migration was squashed or
/// consolidated upstream). Without pruning, `sqlx::migrate!` refuses to
/// start with `migration N was previously applied but is missing in the
/// resolved migrations`.
///
/// Idempotent: deleting a version that isn't present is a no-op. Returns
/// the number of rows actually removed. No data tables are touched.
pub async fn prune_orphans(
    conn: &mut SqliteConnection,
    versions: &[i64],
) -> Result<u64> {
    if versions.is_empty() {
        return Ok(0);
    }
    let mut total: u64 = 0;
    for &version in versions {
        let result = sqlx::query("DELETE FROM _sqlx_migrations WHERE version = ?")
            .bind(version)
            .execute(&mut *conn)
            .await
            .with_context(|| format!("delete _sqlx_migrations row for version {version}"))?;
        total += result.rows_affected();
    }
    Ok(total)
}

/// Record an embedded migration as applied without executing it. For
/// the case where a migration's effects already landed via an earlier
/// (now-renamed) file — running it again would fail (duplicate column,
/// table-exists, etc.), but the schema is already correct.
///
/// Returns true if the row was inserted, false if the version was
/// already recorded (idempotent).
pub async fn mark_applied(conn: &mut SqliteConnection, version: i64) -> Result<bool> {
    let migration = MIGRATOR
        .iter()
        .find(|m| m.version == version)
        .with_context(|| format!("no embedded migration at version {version}"))?;

    let existing: Option<i64> =
        sqlx::query("SELECT version FROM _sqlx_migrations WHERE version = ?")
            .bind(version)
            .fetch_optional(&mut *conn)
            .await?
            .map(|row| row.try_get::<i64, _>("version").unwrap_or(version));
    if existing.is_some() {
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) \
         VALUES (?, ?, CURRENT_TIMESTAMP, 1, ?, 0)",
    )
    .bind(version)
    .bind(&*migration.description)
    .bind(&*migration.checksum)
    .execute(&mut *conn)
    .await
    .with_context(|| format!("insert _sqlx_migrations row for version {version}"))?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ensure_db_at, open_db_at, platform_db_path};
    use crate::testutil::init_project_minimal;
    use tempfile::tempdir;

    /// Insert a synthetic `_sqlx_migrations` row at a version that doesn't
    /// correspond to any embedded migration — i.e. an orphan from the
    /// perspective of the current source tree.
    async fn insert_orphan(conn: &mut SqliteConnection, version: i64) {
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) \
             VALUES (?, 'orphan', CURRENT_TIMESTAMP, 1, X'00', 0)",
        )
        .bind(version)
        .execute(&mut *conn)
        .await
        .unwrap();
    }

    async fn count_versions(conn: &mut SqliteConnection) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&mut *conn)
            .await
            .unwrap()
    }

    fn fresh_db() -> (tempfile::TempDir, SqliteConnection) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project_minimal(tmp.path()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        let conn = open_db_at(&platform_db_path(&ship_dir)).unwrap();
        (tmp, conn)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn prune_orphans_removes_specified_rows() {
        let (_tmp, mut conn) = fresh_db();
        // 9_000_000_001 has no embedded migration. The current applied set is
        // whatever the source tree actually contains.
        insert_orphan(&mut conn, 9_000_000_001).await;
        insert_orphan(&mut conn, 9_000_000_002).await;

        let report = repair_migrations(&mut conn).await.unwrap();
        assert!(report.orphaned.contains(&9_000_000_001));
        assert!(report.orphaned.contains(&9_000_000_002));

        let before = count_versions(&mut conn).await;
        let pruned = prune_orphans(&mut conn, &report.orphaned).await.unwrap();
        assert_eq!(pruned, report.orphaned.len() as u64);

        let after = count_versions(&mut conn).await;
        assert_eq!(before - after, report.orphaned.len() as i64);

        // Subsequent repair sees no orphans.
        let report = repair_migrations(&mut conn).await.unwrap();
        assert!(report.orphaned.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn prune_orphans_noop_when_empty() {
        let (_tmp, mut conn) = fresh_db();
        let before = count_versions(&mut conn).await;
        let pruned = prune_orphans(&mut conn, &[]).await.unwrap();
        assert_eq!(pruned, 0);
        let after = count_versions(&mut conn).await;
        assert_eq!(before, after);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn prune_orphans_does_not_touch_known_versions() {
        let (_tmp, mut conn) = fresh_db();
        // A pristine DB has zero orphans — every recorded row has a matching
        // embedded migration. Running repair without flags should leave the
        // bookkeeping table untouched, and so should an empty prune.
        let report = repair_migrations(&mut conn).await.unwrap();
        assert!(report.orphaned.is_empty(), "fresh DB has no orphans");

        let before = count_versions(&mut conn).await;
        let pruned = prune_orphans(&mut conn, &report.orphaned).await.unwrap();
        assert_eq!(pruned, 0);
        let after = count_versions(&mut conn).await;
        assert_eq!(before, after);
    }
}
