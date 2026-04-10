# Job Protocol

On start, check `.artifacts/job-spec.md`. If found, execute immediately unless `## Mode: interactive`.
Log progress via `log_progress`. Signal completion via `update_job`. Never exit silently.
