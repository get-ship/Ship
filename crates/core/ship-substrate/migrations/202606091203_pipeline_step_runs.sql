-- pipeline_step_runs: normalized per-step read-model for pipeline runs.
--
-- PIPELINES P0.1 — the first decoupled step contract. One row per
-- (run_id, step_id). Replaces the opaque steps_json / completed_steps_json /
-- current_step blobs carried on the pipeline_runs row with a relational,
-- per-step table that records the step's real lifecycle timestamps and the
-- DAG edge. Topology-invariant: survives the R13 step_monitor dissolution
-- verbatim because it never anchors on step_monitor evidence.
--
-- Anchored on the DURABLE step events (pipeline.step.started/completed/failed,
-- plus retry/done_declared/output) AND on the CHILD SESSION TERMINAL
-- (canonical session.completed/failed/cancelled whose parent ==
-- pipeline_run_id). A composed step's agent session reaching a terminal IS the
-- step's real completion signal (and the post-dissolve resume trigger). It is
-- deliberately NOT anchored on pipeline.step.observed — that is step_monitor
-- evidence the R13 dissolution deletes.
--
-- Observed-derived evidence (trigger / commits / uncommitted / log_tail) is
-- demoted: it never lands on this row. The row stays clean + durable-anchored.
--
-- DB placement: pipeline.* is Runtime-scoped, so the table lives in runtime.db
-- (same file as pipeline_runs). The canonical session.* coarse family is
-- Space-scoped, but the projection funnels both families into runtime.db (same
-- pattern as canonical_sessions). CREATE TABLE IF NOT EXISTS means the table is
-- created in every scoped db at migration time; only runtime.db receives
-- writes from PipelineStepRunProjection::apply().

CREATE TABLE IF NOT EXISTS pipeline_step_runs (
  run_id          TEXT NOT NULL,
  step_id         TEXT NOT NULL,
  step_type       TEXT NOT NULL DEFAULT 'agent',   -- agent|review|run
  status          TEXT NOT NULL DEFAULT 'pending',  -- pending|running|completed|failed
  started_at      TEXT,                             -- ISO-8601; NULL until running
  completed_at    TEXT,                             -- ISO-8601; NULL until terminal
  depends_on_json TEXT,                             -- JSON array of upstream step ids (DAG edge)
  attempt         INTEGER NOT NULL DEFAULT 1,
  session_id      TEXT,                             -- composed agent session id (child-terminal anchor)
  note            TEXT,                             -- last done summary / run-update message / failure reason
  updated_at      TEXT NOT NULL,                    -- ISO-8601; last event that touched the row
  PRIMARY KEY (run_id, step_id)
);

CREATE INDEX IF NOT EXISTS idx_pipeline_step_runs_run     ON pipeline_step_runs(run_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_step_runs_status  ON pipeline_step_runs(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_step_runs_session ON pipeline_step_runs(session_id);
