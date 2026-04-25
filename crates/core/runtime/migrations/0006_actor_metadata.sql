-- Extend `actors` with rich runtime metadata so the unified dashboard can
-- render every running thing (chat, pipeline, service) without a parallel
-- session registry. New columns are populated from the `actor.created`
-- payload by the actor projection; `last_event_at` is bumped by the two
-- triggers below on every event correlated to the actor.
ALTER TABLE actors ADD COLUMN agent_id TEXT;
ALTER TABLE actors ADD COLUMN provider TEXT;
ALTER TABLE actors ADD COLUMN model TEXT;
ALTER TABLE actors ADD COLUMN transport TEXT;
ALTER TABLE actors ADD COLUMN persistence TEXT;
ALTER TABLE actors ADD COLUMN provider_session_id TEXT;
ALTER TABLE actors ADD COLUMN system_prompt_hash TEXT;
ALTER TABLE actors ADD COLUMN pid INTEGER;
ALTER TABLE actors ADD COLUMN capabilities_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE actors ADD COLUMN exit_code INTEGER;
ALTER TABLE actors ADD COLUMN exit_reason TEXT;
ALTER TABLE actors ADD COLUMN duration_ms INTEGER;
ALTER TABLE actors ADD COLUMN last_event_at TEXT;

CREATE INDEX IF NOT EXISTS idx_actors_provider   ON actors(provider);
CREATE INDEX IF NOT EXISTS idx_actors_agent_id   ON actors(agent_id);
CREATE INDEX IF NOT EXISTS idx_actors_last_event ON actors(last_event_at);

-- Liveness without a heartbeat event: any event correlated to the actor
-- (either emitted by it or directed at it) bumps `last_event_at`. Quiet
-- actors fall stale naturally; the dashboard reads recency from the
-- column. No periodic ticks needed.
CREATE TRIGGER IF NOT EXISTS bump_actor_last_event_on_actor_id
AFTER INSERT ON events
WHEN NEW.actor_id IS NOT NULL
BEGIN
  UPDATE actors SET last_event_at = NEW.created_at WHERE id = NEW.actor_id;
END;

CREATE TRIGGER IF NOT EXISTS bump_actor_last_event_on_target
AFTER INSERT ON events
WHEN NEW.target_actor_id IS NOT NULL
BEGIN
  UPDATE actors SET last_event_at = NEW.created_at WHERE id = NEW.target_actor_id;
END;
