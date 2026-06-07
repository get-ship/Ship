//! Database integrity check — detect corruption on startup.
//!
//! Runs `PRAGMA integrity_check` against each per-scope event db. After the
//! Phase-4 storage partition there is no single platform.db: every scope owns
//! its own db file (`runtime.db`, `account.db`, each mounted
//! `spaces/<id>/space.db`), each carrying its own `events` log and projection
//! tables. The boot path checks each present scope db:
//! - If projection tables are corrupt but `events` is intact → auto-rebuild
//! - If `events` itself is corrupt → alert the user, refuse to start

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

use crate::db::{block_on, open_db_at};

/// Result of an integrity check.
#[derive(Debug)]
pub enum IntegrityStatus {
    /// Database is healthy.
    Ok,
    /// Projection tables are corrupt but events table is intact.
    /// Caller should trigger a projection rebuild.
    ProjectionsCorrupt { details: String },
    /// Events table or core schema is corrupt. Cannot recover.
    EventsCorrupt { details: String },
}

/// Run `PRAGMA integrity_check` on the scope db at the given path.
///
/// Returns an `IntegrityStatus` indicating whether the database is healthy,
/// has recoverable corruption (projections only), or is fatally corrupt.
pub fn check_integrity(db_path: &std::path::Path) -> Result<IntegrityStatus> {
    let mut conn = match open_db_at(db_path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(IntegrityStatus::EventsCorrupt {
                details: format!("cannot open database: {e}"),
            });
        }
    };

    let rows: Vec<(String,)> = block_on(async {
        sqlx::query_as("PRAGMA integrity_check")
            .fetch_all(&mut conn)
            .await
    })?;

    if rows.len() == 1 && rows[0].0 == "ok" {
        return Ok(IntegrityStatus::Ok);
    }

    let details: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    let detail_str = details.join("; ");

    // Corruption confined to projection tables is recoverable by replay; only
    // the `events` log is irreplaceable. Any corruption that does NOT name a
    // projection table is treated as events/core corruption (fatal). The set
    // covers the substrate projections plus the platform-layer projections that
    // share the scope dbs.
    let projection_tables = [
        "sessions",
        "actors",
        "processes",
        "agent_sessions",
        "pipeline_runs",
        "projects",
        "project_compile_state",
        "space_settings",
        "auth_sessions",
        "daemon_owner",
        "provisioned_grants",
        "analytics_sessions",
        "analytics_tool_uses",
        "analytics_turns",
        "cursors",
    ];
    let all_projection_only = details.iter().all(|d| {
        projection_tables
            .iter()
            .any(|t| d.contains(t))
    });

    if all_projection_only {
        Ok(IntegrityStatus::ProjectionsCorrupt {
            details: detail_str,
        })
    } else {
        Ok(IntegrityStatus::EventsCorrupt {
            details: detail_str,
        })
    }
}

/// Quick integrity check — runs `PRAGMA quick_check` for a faster partial
/// check (stops at the first error).
pub fn quick_check(db_path: &std::path::Path) -> Result<bool> {
    let mut conn = open_db_at(db_path)?;
    let row: (String,) = block_on(async {
        sqlx::query_as("PRAGMA quick_check")
            .fetch_one(&mut conn)
            .await
    })?;
    Ok(row.0 == "ok")
}

/// The per-scope event db files that exist under `ship_dir`: `runtime.db`,
/// `account.db` (if present), and every mounted `spaces/<id>/space.db`. Only
/// existing files are returned — a scope whose db has never been written is
/// not a corruption candidate.
pub fn scope_db_paths(ship_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let runtime = crate::db::runtime_db_path(ship_dir);
    if runtime.exists() {
        paths.push(runtime);
    }
    let account = crate::db::account_db_path(ship_dir);
    if account.exists() {
        paths.push(account);
    }
    let spaces_dir = ship_dir.join(crate::db::SPACES_DIR);
    if let Ok(entries) = std::fs::read_dir(&spaces_dir) {
        for entry in entries.flatten() {
            if let Some(id) = entry.file_name().to_str() {
                let space_db = crate::db::space_db_path(ship_dir, id);
                if space_db.exists() {
                    paths.push(space_db);
                }
            }
        }
    }

    paths
}

/// Check integrity across every present per-scope db under `ship_dir`.
///
/// Returns the most severe status observed: any `EventsCorrupt` wins (fatal),
/// else any `ProjectionsCorrupt` (recoverable by rebuild), else `Ok`. The
/// reported `details` are prefixed with the offending db path so an operator
/// knows which scope to inspect. Errors only when no scope db exists at all.
pub fn check_scope_integrity(ship_dir: &Path) -> Result<IntegrityStatus> {
    let paths = scope_db_paths(ship_dir);
    if paths.is_empty() {
        return Err(anyhow!(
            "no scope event db found under {} (expected runtime.db / account.db / spaces/*/space.db)",
            ship_dir.display()
        ));
    }

    let mut projections_corrupt: Option<String> = None;
    for path in &paths {
        match check_integrity(path)? {
            IntegrityStatus::Ok => {}
            IntegrityStatus::ProjectionsCorrupt { details } => {
                projections_corrupt
                    .get_or_insert_with(|| format!("{}: {details}", path.display()));
            }
            IntegrityStatus::EventsCorrupt { details } => {
                // Fatal — report immediately with the offending db named.
                return Ok(IntegrityStatus::EventsCorrupt {
                    details: format!("{}: {details}", path.display()),
                });
            }
        }
    }

    match projections_corrupt {
        Some(details) => Ok(IntegrityStatus::ProjectionsCorrupt { details }),
        None => Ok(IntegrityStatus::Ok),
    }
}

/// Run the integrity check across the per-scope dbs at `ship_dir`.
///
/// Thin alias for [`check_scope_integrity`] — kept so the boot path and tests
/// can name "the integrity check for this ship-home" without restating the
/// scope-iteration rule.
pub fn check_platform_integrity(ship_dir: &std::path::Path) -> Result<IntegrityStatus> {
    check_scope_integrity(ship_dir)
}
