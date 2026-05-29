-- Add space_id column to agent_sessions table.
-- Backfill from events table: most agent.session.started events carry
-- space_id in the envelope. For any orphaned rows (events without
-- space_id), set to NULL; these are safe to drop since agent_sessions
-- is a cache derived from events and can be regenerated.

ALTER TABLE agent_sessions ADD COLUMN space_id TEXT;

-- Backfill: match agent_sessions rows to their agent.session.started events
-- and copy the space_id from the event envelope.
UPDATE agent_sessions
SET space_id = (
  SELECT e.space_id
  FROM events e
  WHERE e.event_type = 'agent.session.started'
    AND json_extract(e.payload_json, '$.session_id') = agent_sessions.session_id
  LIMIT 1
)
WHERE space_id IS NULL;

-- Create index for queries filtering by space_id.
CREATE INDEX IF NOT EXISTS idx_agent_sessions_space_id ON agent_sessions(space_id);
