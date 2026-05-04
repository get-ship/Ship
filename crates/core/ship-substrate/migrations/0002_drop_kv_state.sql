-- Drop kv_state from platform.db. KV now lives in kv.db.
-- Existing rows were already migrated to kv.db on startup (see db::kv::migrate_from_platform_db).
DROP TABLE IF EXISTS kv_state;
