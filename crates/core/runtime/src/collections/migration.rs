//! One-time migration of user content from `platform.db` to `data.db` collections.
//!
//! Historically user-authored content (notes, ADRs, targets/capabilities)
//! lived in dedicated tables on `platform.db`. The collections system now
//! owns all user content, so we copy existing rows into the matching
//! built-in collections and record a completion flag so the job never
//! re-runs.
//!
//! The prune step (`prune_legacy_source_rows`) removes the source rows from
//! `platform.db` once migration has settled. It is gated on the v1 migration
//! completion flag — fresh installs (no `platform.db` or un-migrated
//! projects) are never touched.

use anyhow::{Context, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use super::builtin;
use super::migration_tables as tables;
use crate::data::{DataService, SqlParam};

pub(super) const MIGRATION_KEY: &str = "platform_to_collections_v1";

/// Summary of a migration run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub already_migrated: bool,
    pub source_missing: bool,
    pub notes_migrated: usize,
    pub adrs_migrated: usize,
    pub specs_migrated: usize,
}

/// Summary of a prune run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PruneReport {
    pub skipped_not_migrated: bool,
    pub already_pruned: bool,
    pub notes_deleted: u64,
    pub adrs_deleted: u64,
    pub specs_deleted: u64,
}

/// Run the one-time migration. Idempotent — a completion flag in `data.db`
/// prevents re-execution.
pub fn migrate_platform_to_collections(
    svc: &mut DataService,
    platform_db_path: &Path,
) -> Result<MigrationReport> {
    ensure_migration_state_table(svc)?;

    if is_migration_complete(svc, MIGRATION_KEY)? {
        return Ok(MigrationReport { already_migrated: true, ..Default::default() });
    }

    let Some(mut platform) = open_platform(platform_db_path, true)? else {
        // No platform.db present — nothing to migrate, but mark complete so
        // subsequent startups don't probe the filesystem every time.
        mark_migration_complete(svc, MIGRATION_KEY)?;
        return Ok(MigrationReport { source_missing: true, ..Default::default() });
    };

    let notes_schema = builtin::notes_schema();
    let adrs_schema = builtin::adrs_schema();
    let specs_schema = builtin::specs_schema();

    let notes_migrated = tables::migrate_notes(svc, &notes_schema, &mut platform)?;
    let adrs_migrated = tables::migrate_adrs(svc, &adrs_schema, &mut platform)?;
    let specs_migrated = tables::migrate_specs(svc, &specs_schema, &mut platform)?;

    mark_migration_complete(svc, MIGRATION_KEY)?;

    tracing::info!(
        notes = notes_migrated,
        adrs = adrs_migrated,
        specs = specs_migrated,
        "platform.db → data.db migration complete"
    );

    Ok(MigrationReport {
        already_migrated: false,
        source_missing: false,
        notes_migrated,
        adrs_migrated,
        specs_migrated,
    })
}

/// Prune legacy source rows from `platform.db` after the v1 migration has settled.
///
/// Safety gate: if `platform_to_collections_v1` is not marked complete in
/// `data.db`, this function returns immediately — fresh installs and
/// un-migrated projects are never touched.
///
/// Idempotent: a separate completion flag (`platform_to_collections_prune_v1`)
/// prevents re-running after a successful prune.
pub fn prune_legacy_source_rows(
    svc: &mut DataService,
    platform_db_path: &Path,
) -> Result<PruneReport> {
    ensure_migration_state_table(svc)?;

    if !is_migration_complete(svc, MIGRATION_KEY)? {
        return Ok(PruneReport { skipped_not_migrated: true, ..Default::default() });
    }

    const PRUNE_KEY: &str = "platform_to_collections_prune_v1";
    if is_migration_complete(svc, PRUNE_KEY)? {
        return Ok(PruneReport { already_pruned: true, ..Default::default() });
    }

    if !platform_db_path.exists() {
        mark_migration_complete(svc, PRUNE_KEY)?;
        return Ok(PruneReport::default());
    }

    let mut conn = open_platform(platform_db_path, false)?
        .ok_or_else(|| anyhow::anyhow!("platform.db disappeared between exists() check and open"))?;

    let notes_deleted = tables::delete_if_table_exists(&mut conn, "note")?;
    let adrs_deleted = tables::delete_if_table_exists(&mut conn, "adr")?;
    let specs_deleted = tables::delete_if_table_exists(&mut conn, "target")?;
    // capability rows reference target — prune together.
    tables::delete_if_table_exists(&mut conn, "capability")?;

    crate::db::block_on(async { conn.close().await })?;
    mark_migration_complete(svc, PRUNE_KEY)?;

    tracing::info!(
        notes = notes_deleted,
        adrs = adrs_deleted,
        specs = specs_deleted,
        "collections: pruned legacy source rows from platform.db"
    );

    Ok(PruneReport {
        notes_deleted,
        adrs_deleted,
        specs_deleted,
        ..Default::default()
    })
}

// ── Migration state ──────────────────────────────────────────────────────────

pub(super) fn ensure_migration_state_table(svc: &mut DataService) -> Result<()> {
    svc.execute(
        "CREATE TABLE IF NOT EXISTS _collection_migration_state (
            name TEXT PRIMARY KEY,
            completed_at TEXT NOT NULL
         )",
        &[],
    )?;
    Ok(())
}

pub(super) fn is_migration_complete(svc: &mut DataService, key: &str) -> Result<bool> {
    let rows = svc.query(
        "SELECT name FROM _collection_migration_state WHERE name = ?",
        &[SqlParam::from(key)],
    )?;
    Ok(!rows.is_empty())
}

pub(super) fn mark_migration_complete(svc: &mut DataService, key: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    svc.execute(
        "INSERT OR REPLACE INTO _collection_migration_state (name, completed_at)
         VALUES (?, ?)",
        &[SqlParam::from(key), SqlParam::from(now.as_str())],
    )?;
    Ok(())
}

// ── Platform db access ───────────────────────────────────────────────────────

/// Open `platform.db`. When `read_only` is true, opens in read-only mode.
/// Returns `None` if the file does not exist.
pub(super) fn open_platform(path: &Path, read_only: bool) -> Result<Option<SqliteConnection>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    let url = format!("sqlite://{raw}");
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .busy_timeout(Duration::from_secs(5))
        .read_only(read_only);
    let conn = crate::db::block_on(async { SqliteConnection::connect_with(&opts).await })?;
    Ok(Some(conn))
}

#[cfg(test)]
mod tests;
