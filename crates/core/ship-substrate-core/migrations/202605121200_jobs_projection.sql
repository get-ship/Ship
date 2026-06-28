-- Materialized jobs projection.
--
-- Replaces the prior pattern where `ship_platform::projections::job::load_jobs`
-- scanned every `job.*` and `step.*` event ever emitted and folded them in
-- memory on each call. Seven orchestrator hot-path sites called load_jobs;
-- the cost grew quadratically with event-log size.
--
-- Maintained by `ship_platform::projections::job::JobProjection` via the
-- standard `Projection` trait — `apply()` UPDATEs this table as job/step
-- events flow. A one-shot hydration on first read backfills the table from
-- the existing event log for installs that predate the projection.
--
-- Drop any pre-projection `jobs` table first: `CREATE TABLE IF NOT EXISTS`
-- would silently no-op against an older-shaped table, then the `slug`
-- index below would fail. The table is a pure projection — it rebuilds
-- from the event log on first read — so dropping it loses no data.
DROP TABLE IF EXISTS jobs;

CREATE TABLE jobs (
  job_id              TEXT PRIMARY KEY,
  slug                TEXT NOT NULL,
  agent               TEXT NOT NULL,
  branch              TEXT NOT NULL,
  spec_path           TEXT NOT NULL,
  source_spec_path    TEXT,
  depends_on_json     TEXT,
  status              TEXT NOT NULL,
  worktree            TEXT,
  blocker             TEXT,
  error               TEXT,
  model               TEXT,
  harness            TEXT,
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL,
  harness_meta_json  TEXT,
  pipeline_json       TEXT,
  steps_json          TEXT,
  current_phase       INTEGER,
  completed_steps_json TEXT NOT NULL DEFAULT '[]',
  current_step        TEXT,
  vars_json           TEXT,
  step_retries_json   TEXT NOT NULL DEFAULT '{}',
  headless            INTEGER NOT NULL DEFAULT 0,
  baseline_sha        TEXT
);

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_slug ON jobs(slug);
