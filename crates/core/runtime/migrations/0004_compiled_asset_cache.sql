-- Rename agent_artifact_registry → compiled_asset_cache.
-- This table tracks compiled skills, rules, and MCP server configs —
-- calling it an "agent artifact registry" was a misnomer.
CREATE TABLE IF NOT EXISTS compiled_asset_cache (
  uuid         TEXT PRIMARY KEY,
  kind         TEXT NOT NULL,
  external_id  TEXT NOT NULL,
  name         TEXT NOT NULL,
  source_path  TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  UNIQUE(kind, external_id)
);
CREATE INDEX IF NOT EXISTS compiled_asset_cache_kind_idx
  ON compiled_asset_cache(kind);

-- Copy existing rows.
INSERT OR IGNORE INTO compiled_asset_cache
  SELECT uuid, kind, external_id, name, source_path, content_hash, updated_at
  FROM agent_artifact_registry;

DROP TABLE IF EXISTS agent_artifact_registry;
DROP INDEX IF EXISTS agent_artifact_kind_idx;
