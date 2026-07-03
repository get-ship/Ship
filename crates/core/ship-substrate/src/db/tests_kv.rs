//! kv.db provisioning tests — the KV plane is DDL-on-open, like the per-scope
//! event dbs (`scope_schema`): idempotent `CREATE … IF NOT EXISTS` applied on
//! every open, with no sqlx migration chain and no `_sqlx_migrations` version
//! log in the file.

use tempfile::tempdir;

use crate::db::{block_on, ensure_kv_db_at, kv_db_path, open_kv_db_at};

#[test]
fn fresh_kv_db_round_trips_without_migration_log() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");

    ensure_kv_db_at(&ship_dir)?;
    // Re-ensure: the schema application is idempotent (re-applied on every open).
    ensure_kv_db_at(&ship_dir)?;

    let path = kv_db_path(&ship_dir);
    let mut conn = open_kv_db_at(&path)?;

    // Behavior: a fresh kv.db serves a kv_state write + read.
    block_on(async {
        sqlx::query(
            "INSERT INTO kv_state (namespace, key, value_json, updated_at) \
             VALUES ('ns', 'k', '{\"v\":1}', datetime('now'))",
        )
        .execute(&mut conn)
        .await
    })?;
    let value: String = block_on(async {
        sqlx::query_scalar("SELECT value_json FROM kv_state WHERE namespace = 'ns' AND key = 'k'")
            .fetch_one(&mut conn)
            .await
    })?;
    assert_eq!(value, "{\"v\":1}");

    // Mechanism: DDL-on-open means there is no migration version log to drift.
    let migration_log: Option<String> = block_on(async {
        sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
        )
        .fetch_optional(&mut conn)
        .await
    })?;
    assert_eq!(
        migration_log, None,
        "kv.db must be provisioned by DDL-on-open, not a _sqlx_migrations chain"
    );
    Ok(())
}
