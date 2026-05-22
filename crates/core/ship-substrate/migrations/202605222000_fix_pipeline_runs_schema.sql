-- Reconcile the pipeline_runs projection schema.
--
-- Symptom: "SQLite operation failed: no such column: job_id" on every
-- load_pipeline_runs_at + orchestrator coherence sweep — pipeline data never
-- loads (pipelines surface is empty) and the sweep error-spams every ~20s.
--
-- Cause: dev worktrees share one ~/.ship/machine/db/platform.db (see
-- db::mod). Across diverging branches its `pipeline_runs` table drifted out
-- of sync with the projection's ROW_COLS (missing job_id, the PK). The rename
-- migration (ALTER jobs RENAME TO pipeline_runs) preserves columns, so a clean
-- history is fine — this only repairs already-drifted DBs.
--
-- Safe because pipeline_runs is a *pure projection*: rebuilt from the event
-- log on first read (projections::pipeline_run::hydrate_if_needed). Dropping
-- + recreating loses no data — the events table is the source of truth.
DROP TABLE IF EXISTS pipeline_runs;

CREATE TABLE pipeline_runs (
  job_id               TEXT PRIMARY KEY,
  slug                 TEXT NOT NULL,
  agent                TEXT NOT NULL,
  branch               TEXT NOT NULL,
  spec_path            TEXT NOT NULL,
  source_spec_path     TEXT,
  depends_on_json      TEXT,
  status               TEXT NOT NULL,
  worktree             TEXT,
  blocker              TEXT,
  error                TEXT,
  model                TEXT,
  harness              TEXT,
  created_at           TEXT NOT NULL,
  updated_at           TEXT NOT NULL,
  harness_meta_json    TEXT,
  pipeline_json        TEXT,
  steps_json           TEXT,
  current_phase        INTEGER,
  completed_steps_json TEXT NOT NULL DEFAULT '[]',
  current_step         TEXT,
  vars_json            TEXT,
  step_retries_json    TEXT NOT NULL DEFAULT '{}',
  headless             INTEGER NOT NULL DEFAULT 0,
  baseline_sha         TEXT
);

CREATE INDEX IF NOT EXISTS idx_pipeline_runs_status ON pipeline_runs(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_slug ON pipeline_runs(slug);
