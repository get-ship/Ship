//! kv.db schema — the durable key-value plane (L2+ user-app state), kept in
//! its own file so KV writes never compete with the event log for WAL locks.
//!
//! Same mechanism as the per-scope event dbs ([`super::scope_schema`]): the
//! DDL is idempotent (`CREATE … IF NOT EXISTS`) and applied directly on every
//! open, with no `_sqlx_migrations` version log — so there is nothing to
//! drift. kv.db is not an event db; it carries exactly this one table.

/// Idempotent kv.db DDL, applied on every open.
pub const KV_DDL: &[&str] = &[
    r#"CREATE TABLE IF NOT EXISTS kv_state (
  namespace  TEXT NOT NULL,
  key        TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
)"#,
    "CREATE INDEX IF NOT EXISTS idx_kv_namespace ON kv_state(namespace)",
];
