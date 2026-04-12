-- Ship specs table — planning/spec management UI backing store.
-- Body lives on disk at body_path (.artifacts/specs/<id>.md), metadata in DB.

CREATE TABLE IF NOT EXISTS spec (
  id            TEXT PRIMARY KEY,
  title         TEXT NOT NULL,
  body_path     TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'draft',
  pipeline      TEXT,
  vars_json     TEXT NOT NULL DEFAULT '{}',
  criteria_json TEXT NOT NULL DEFAULT '[]',
  scope_json    TEXT NOT NULL DEFAULT '[]',
  priority      INTEGER,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS spec_status_idx ON spec(status, updated_at DESC);
