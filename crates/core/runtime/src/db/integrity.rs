//! Database integrity check — detect corruption on startup.
//!
//! Runs `PRAGMA integrity_check` against platform.db. If corruption is
//! detected, reports actionable information:
//! - If projection tables are corrupt but event_log is intact → auto-rebuild
//! - If event_log is corrupt → alert the user, refuse to start

use anyhow::{Result, anyhow};

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

/// Run `PRAGMA integrity_check` on platform.db at the given path.
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

    // Check if corruption is limited to projection tables (recoverable).
    let projection_tables = [
        "workspace",
        "workspace_session",
        "workspace_session_record",
        "actors",
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

/// Quick integrity check — runs `PRAGMA integrity_check(1)` for a faster
/// partial check (stops at the first error).
pub fn quick_check(db_path: &std::path::Path) -> Result<bool> {
    let mut conn = open_db_at(db_path)?;
    let row: (String,) = block_on(async {
        sqlx::query_as("PRAGMA quick_check")
            .fetch_one(&mut conn)
            .await
    })?;
    Ok(row.0 == "ok")
}

/// Run integrity check on platform.db at the given ship_dir.
/// Convenience wrapper around `check_integrity`.
pub fn check_platform_integrity(ship_dir: &std::path::Path) -> Result<IntegrityStatus> {
    let db_path = ship_dir.join("platform.db");
    if !db_path.exists() {
        return Err(anyhow!(
            "platform.db not found at {}",
            db_path.display()
        ));
    }
    check_integrity(&db_path)
}
