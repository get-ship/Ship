-- Git commit drafts — agent-written commit messages awaiting human sign-off.
-- Keyed by branch; one draft per branch at a time. Cleared on successful commit.

CREATE TABLE IF NOT EXISTS git_commit_drafts (
    branch       TEXT PRIMARY KEY,
    subject      TEXT NOT NULL,
    body         TEXT NOT NULL DEFAULT '',
    by_agent     TEXT,
    by_pipeline  TEXT,
    drafted_at   TEXT NOT NULL
);
