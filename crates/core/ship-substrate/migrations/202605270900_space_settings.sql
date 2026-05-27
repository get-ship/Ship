-- space_settings: per-space settings overrides, layered OVER the global daemon
-- settings (daemon_settings KV). Space overrides win per-key; global stays the
-- base. Materialized from `space.setting.changed` events (the space scope rides
-- the event envelope's `space_id`; the key is `entity_id`). Latest write wins
-- per (space_id, key).
--
-- See .artifacts/specs/space/129-per-space-settings.md.

CREATE TABLE IF NOT EXISTS space_settings (
  space_id    TEXT NOT NULL,
  key         TEXT NOT NULL,
  value_json  TEXT NOT NULL,
  updated_at  TEXT NOT NULL,
  PRIMARY KEY (space_id, key)
);
