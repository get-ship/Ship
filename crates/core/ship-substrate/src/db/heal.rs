//! Scope-safe, corruption-tolerant projection-table clearing for rebuild.
//!
//! `Projection::truncate` implementations call [`truncate_or_heal`] instead of
//! a raw `DELETE FROM <table>` so a single mechanism makes rebuild:
//!   - **scope-safe**: a table this scope's schema never provisions (e.g.
//!     `actors` in a Space-scope db) is already-empty, not an error.
//!   - **corruption-tolerant**: a table whose own root page is damaged is
//!     healed by schema surgery that never reads that page, instead of
//!     failing the same way `DELETE`/`DROP` would.

use anyhow::{Context, Result};
use sqlx::{Row, SqliteConnection};

use crate::db::{block_on, block_on_anyhow};

/// Clear `table` so replay can rebuild it from scratch.
///
/// - `DELETE FROM <table>` succeeds: the common case, done.
/// - fails with "no such table": this scope's schema never provisions
///   `table` — already empty from this scope db's point of view.
/// - fails any other way (page-level corruption): healed via
///   [`heal_corrupt_table`].
pub fn truncate_or_heal(conn: &mut SqliteConnection, table: &str) -> Result<()> {
    let stmt = format!("DELETE FROM \"{table}\"");
    let result = block_on(async { sqlx::query(&stmt).execute(&mut *conn).await });

    match result {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("no such table") => Ok(()),
        Err(e) => heal_corrupt_table(conn, table)
            .with_context(|| format!("truncate '{table}' failed ({e}); heal also failed")),
    }
}

/// Heal a projection table whose own b-tree page(s) are corrupted. Both
/// `DELETE` and `DROP TABLE` must walk the table's page tree to do their
/// job, so both fail the same way against a damaged root page. Instead:
/// capture the table's (and its indexes'/triggers') `CREATE` statements
/// from `sqlite_master` — a read of the schema page, never the damaged
/// table page — remove those schema rows via `PRAGMA writable_schema`,
/// `VACUUM` to reclaim the now-unreferenced pages (VACUUM only walks the
/// live schema reachable from `sqlite_master`, so it never touches the
/// removed page either), then replay the captured DDL to recreate an
/// empty table with the identical shape.
fn heal_corrupt_table(conn: &mut SqliteConnection, table: &str) -> Result<()> {
    block_on_anyhow(async {
        let rows = sqlx::query(
            "SELECT sql FROM sqlite_master \
             WHERE tbl_name = ? AND sql IS NOT NULL \
             ORDER BY (type = 'table') DESC",
        )
        .bind(table)
        .fetch_all(&mut *conn)
        .await?;
        let ddl: Vec<String> = rows.iter().map(|row| row.get::<String, _>("sql")).collect();

        if ddl.is_empty() {
            anyhow::bail!("table '{table}' has no schema entry to heal from");
        }

        sqlx::query("PRAGMA writable_schema = ON")
            .execute(&mut *conn)
            .await?;
        sqlx::query("DELETE FROM sqlite_master WHERE tbl_name = ?")
            .bind(table)
            .execute(&mut *conn)
            .await?;
        sqlx::query("PRAGMA writable_schema = OFF")
            .execute(&mut *conn)
            .await?;
        sqlx::query("VACUUM").execute(&mut *conn).await?;

        for stmt in ddl {
            sqlx::query(&stmt).execute(&mut *conn).await?;
        }
        Ok(())
    })
}
