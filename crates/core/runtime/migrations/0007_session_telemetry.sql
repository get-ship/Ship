-- Session telemetry: enriched metrics on workspace_session_record
ALTER TABLE workspace_session_record ADD COLUMN lines_added INTEGER;
ALTER TABLE workspace_session_record ADD COLUMN lines_removed INTEGER;
ALTER TABLE workspace_session_record ADD COLUMN tool_calls INTEGER;
ALTER TABLE workspace_session_record ADD COLUMN exit_reason TEXT;
