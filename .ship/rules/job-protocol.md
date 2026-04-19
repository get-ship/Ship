# Job Protocol

On start, check `.artifacts/job-spec.md`. If found, execute immediately unless `## Mode: interactive`.

**Signal completion via `step_done`**, not `update_job`. `step_done` advances the pipeline to the next step and is the canonical signal every dispatched agent is expected to call when its step's work is verifiably complete. `update_job` is for the whole-job status and is reserved for the final step or explicit job lifecycle transitions.

Log progress via `log_progress`. Never exit silently.
