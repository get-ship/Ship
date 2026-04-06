-- Add target_actor_id for directed delivery metadata.
ALTER TABLE events ADD COLUMN target_actor_id TEXT;
