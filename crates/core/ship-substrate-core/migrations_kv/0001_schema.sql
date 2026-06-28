-- kv.db schema — durable key-value store, separated from the event log.
--
-- Previously colocated in platform.db. Moved to its own file in Phase 3
-- so KV writes don't compete with the event log for WAL locks, and so the
-- storage tiers are physically distinct.

CREATE TABLE IF NOT EXISTS kv_state (
  namespace  TEXT NOT NULL,
  key        TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);
CREATE INDEX IF NOT EXISTS idx_kv_namespace ON kv_state(namespace);
