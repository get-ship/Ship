-- Add space_id column to agent_sessions table.
-- Backfill from events table: most agent.session.started events carry
-- space_id in the envelope. For any orphaned rows (events without
-- space_id), set to NULL; these are safe to drop since agent_sessions
-- is a cache derived from events and can be regenerated.

ALTER TABLE agent_sessions ADD COLUMN space_id TEXT;

-- Backfill: match agent_sessions rows to their agent.session.started events
-- and copy the space id from the event envelope. The envelope's space now
-- rides the (scope, scope_key) tuple (D0): the space id is `scope_key` on
-- space-scoped events.
UPDATE agent_sessions
SET space_id = (
  SELECT e.scope_key
  FROM events e
  WHERE e.event_type = 'agent.session.started'
    AND e.scope = 'space'
    AND json_extract(e.payload_json, '$.session_id') = agent_sessions.session_id
  LIMIT 1
)
WHERE space_id IS NULL;

-- Create index for queries filtering by space_id.
CREATE INDEX IF NOT EXISTS idx_agent_sessions_space_id ON agent_sessions(space_id);
