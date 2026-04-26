//! Schema DDL for workflow tables — the opinionated planning layer.
//!
//! These tables power Ship's job workflow. Canonical DDL lives in
//! `migrations/0001_schema.sql`. These constants are retained as
//! code-level documentation.
//!
//! Notes, ADRs, targets, and capabilities were removed — user content now
//! lives in `data.db` collections. See `collections::builtin` for the
//! replacement schemas and `collections::migration` for the one-time copy.

/// Jobs: agent work queue.
/// Status lifecycle: pending -> running -> complete | failed | done.
pub const JOBS: &str = r#"
CREATE TABLE IF NOT EXISTS jobs (
  id            TEXT PRIMARY KEY,
  kind          TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'pending',
  branch        TEXT,
  payload_json  TEXT NOT NULL DEFAULT '{}',
  created_by    TEXT,
  claimed_by    TEXT,
  touched_files TEXT NOT NULL DEFAULT '[]',
  assigned_to   TEXT,
  priority      INTEGER NOT NULL DEFAULT 0,
  blocked_by    TEXT,
  file_scope    TEXT NOT NULL DEFAULT '[]',
  capability_id TEXT,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS jobs_status_idx ON jobs(status, created_at DESC);
CREATE INDEX IF NOT EXISTS jobs_branch_idx ON jobs(branch, status);
CREATE INDEX IF NOT EXISTS idx_jobs_assigned ON jobs(assigned_to, status);
"#;

/// File claim: batch-atomic file-path claims for concurrent agent coordination.
pub const FILE_CLAIM: &str = r#"
CREATE TABLE IF NOT EXISTS file_claim (
  path         TEXT PRIMARY KEY,
  job_id       TEXT NOT NULL,
  workspace_id TEXT,
  claimed_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_file_claim_job ON file_claim(job_id);
"#;
