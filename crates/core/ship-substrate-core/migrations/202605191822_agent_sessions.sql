-- agent_sessions: dispatch state for the agent-session primitive.
--
-- Sibling of jobs (which now tracks pipeline COMPOSITION runs only).
-- Every single-agent dispatch — bare `ship use rust-lane`, `ship
-- dispatch`, MCP `dispatch_agent`, a single pipeline step — projects
-- exactly one row here.
--
-- session_id is the primary key (ULID, stable for the session's life).
-- slug is unique and url-safe (used in route paths and CLI output).
-- parent_pipeline_run_id is set when this session is a step of a
-- composing pipeline; null for bare dispatches. The pipeline composer
-- filters agent.session.* events on this field.
--
-- See .artifacts/agents-arc/discussions/agent-primitive-inversion-plan.md
-- for the full schema rationale + transition table.

CREATE TABLE IF NOT EXISTS agent_sessions (
  session_id              TEXT PRIMARY KEY NOT NULL,
  slug                    TEXT NOT NULL UNIQUE,
  status                  TEXT NOT NULL,      -- pending|dispatched|awaiting_push|completed|failed|cancelled
  agent                   TEXT NOT NULL,
  harness                 TEXT,
  model                   TEXT,
  workspace               TEXT NOT NULL,
  branch                  TEXT NOT NULL,
  open_tab                INTEGER NOT NULL DEFAULT 0,
  invocation              TEXT NOT NULL,      -- 'pty'|'print'|'stream'
  keepalive               INTEGER NOT NULL DEFAULT 0,
  resume                  INTEGER NOT NULL DEFAULT 0,
  resume_session_id       TEXT,
  depends_on_json         TEXT,               -- JSON array of slugs
  vars_json               TEXT,               -- JSON object
  baseline_sha            TEXT,
  parent_pipeline_run_id  TEXT,
  worktree                TEXT,               -- absolute path, NULL until dispatched
  pid                     INTEGER,            -- NULL for server-mode or pre-dispatch
  source_spec_path        TEXT,
  prompt                  TEXT,               -- last sent (overwritten on push)
  error                   TEXT,               -- populated on failed
  created_at              TEXT NOT NULL,      -- ISO-8601
  updated_at              TEXT NOT NULL,
  completed_at            TEXT                -- ISO-8601; NULL until terminal
);

CREATE INDEX IF NOT EXISTS idx_agent_sessions_slug      ON agent_sessions(slug);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_status    ON agent_sessions(status);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_parent    ON agent_sessions(parent_pipeline_run_id);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_workspace ON agent_sessions(workspace);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_agent     ON agent_sessions(agent);
