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
