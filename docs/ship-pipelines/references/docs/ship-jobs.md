---
group: Pipelines
order: 3
title: ship-jobs crate
description: The ship-jobs crate — Pipeline, PipelineStep, StepType, JobSpec, normalize_pipeline, short/long pipeline forms.
audience: internal
---

# ship-jobs

`crates/core/ship-jobs/`. Shared types + pipeline normalization. Pure — no filesystem, no network, no database. Used by the CLI (`ship run`), the daemon (`runtime_api::create_job`), and the orchestrator. Sits below runtime and kernel in the dependency graph.

```
ship-jobs/src/
  lib.rs           — module tree
  step.rs          — PipelineStep, StepType, infer_step_type
  pipeline.rs      — Pipeline, PipelineMeta, normalize_pipeline, short-form expansion
  spec.rs          — JobSpec (the markdown+frontmatter input)
  payload.rs       — event payloads for job/step events
  record.rs        — job record shapes shared with projections
  event_types.rs   — string constants for pipeline-step events
  pipeline_tests.rs, tests.rs
```

## StepType

```rust
pub enum StepType {
    Agent,
    Review,                      // agent-backed reviewer of the previous step
    Run { command: String },
}
```

Three variants. `Agent` and `Run` are obvious. `Review` is a step that runs an agent to evaluate the prior step's output. Historical alias: `gate` (as a step type) maps to `Review`. Don't confuse with the `gate:` attribute on any step — that's a declarative diff-rule check, separate mechanism.

## PipelineStep

```rust
pub struct PipelineStep {
    pub id: String,
    pub step_type: StepType,
    pub agent: Option<String>,
    pub goal: Option<String>,
    pub after: Vec<String>,
}
```

`after` is the DAG edge set. Empty → root step. Multiple entries = AND (the step runs when all deps complete).

## Pipeline

```rust
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
}

impl Pipeline {
    pub fn root_steps(&self) -> Vec<&PipelineStep> {
        self.steps.iter().filter(|s| s.after.is_empty()).collect()
    }
}
```

Flat vec plus a root-finder. Orchestrator's `advance_pipeline` traverses the DAG against `after` edges — the `Pipeline` type doesn't need to expose a full graph API.

## PipelineMeta

```rust
pub struct PipelineMeta {
    pub name: Option<String>,
    pub description: Option<String>,
    pub vars: Option<HashMap<String, String>>,
    pub timeout: Option<String>,
    pub max_retries: Option<u32>,
    pub step_specs: Option<Vec<String>>,
    pub auto_merge: Option<bool>,
}
```

Extracted during `normalize_pipeline` separately from the step list.

- `vars` — pipeline-level defaults. Merged under CLI `--var` flags at dispatch.
- `timeout`, `max_retries` — consumed by step monitors and gate retries.
- `step_specs` — optional per-step spec bodies (position-indexed). Orchestrator writes each to `.artifacts/pipeline/NN-<step_id>.md` at launch.
- `auto_merge` — when true, runtime runs `git merge` on the job branch after the final step. Replaces the legacy `type: merge` step.

## normalize_pipeline

```rust
pub fn normalize_pipeline(input: &str) -> Result<(Value, PipelineMeta), String>
```

Takes a pipeline definition string (JSONC) and returns `(normalized_steps_as_json, meta)`. JSONC is the only supported format — YAML is not accepted.

Accepts two shapes:

### Short form

```jsonc
{
  "name": "tdd",
  "agents": ["planner", "implementer", "reviewer"]
}
```

Expands to three sequential steps. Handled by `agents_to_steps`.

### Long form

```jsonc
{
  "name": "fan-out",
  "steps": [
    { "id": "plan", "agent": "planner" },
    { "id": "build-a", "agent": "coder", "after": ["plan"] },
    { "id": "build-b", "agent": "coder", "after": ["plan"] },
    { "id": "review", "agent": "reviewer", "after": ["build-a", "build-b"] }
  ]
}
```

Each entry is either a string (shorthand for `{ id: <str>, agent: <str>, after: [prev] }`) or a map with explicit fields. Mixed lists work.

### infer_step_type

Decides the `type` field when the user omits it:

- `val.is_string()` → `"agent"`.
- `val.has("run")` → `"run"`.
- `val["type"] == "review" | "gate"` → `"review"`.
- default → `"agent"`.

Keeps common cases terse.

## JobSpec

`spec.rs`. Markdown-with-frontmatter input to a pipeline run, authored in `.artifacts/`:

```yaml
---
title: "Add pipeline retry flag"
status: draft
pipeline: tdd
criteria:
  - CLI accepts --retry flag
  - Job record tracks retry count
scope:
  - apps/ship-cli/src/ship_run.rs
vars:
  branch_prefix: feat
---
<markdown body>
```

```rust
pub struct JobSpec {
    pub title: Option<String>,
    pub status: SpecStatus,
    pub pipeline: Option<String>,
    pub criteria: Vec<String>,
    pub scope: Vec<String>,
    pub vars: HashMap<String, String>,
    pub priority: Option<u32>,
}

pub enum SpecStatus { Draft, Ready, InProgress, Done, Abandoned }
```

Body is opaque markdown — it becomes the agent's spec content via `spec_content` in the `PipelineRunStartedPayload`.

## Payload types

`payload.rs`. Event serialization shapes shared between orchestrator (emitter) and CLI (observer for `ship tail`, `ship jobs`):

- `PipelineRunStartedPayload` — POSTed by `ship run`. Carries slug, agent, branch, spec_path, and (after `ensure_pipeline`) parsed pipeline steps.
- `PipelineStepStartedPayload` — step begins.
- `PipelineStepCompletedPayload` — clean completion.
- Additional payloads for cancellation, failure, merge, gate pass/fail.

## Record types

`record.rs`. Shapes shared with `ship_platform::projections::job`. `JobRecord` lives in projections (computed, no table) but the field shape is defined here since the crate that owns the payloads owns the record shape too. One source of truth for the job projection.

## event_types.rs

String constants for pipeline step events. Historically local to this crate; step event strings have since been lifted into `ship-types::event_types` (`JOB_STEP_COMPLETED`, `JOB_STEP_FAILED`, `PIPELINE_RUN_CANCELLED`, `PIPELINE_RUN_MERGED`, `PIPELINE_STEP_SPAWN_FAILED`) via the db-cleanup-01 refactor. The shared crate is the canonical source; any remaining literals here should be migrated in a follow-up.

## Invariants

- Pure crate. No filesystem, network, or database.
- `normalize_pipeline` accepts short or long form, always emits long form. Orchestrator only sees normalized input.
- Step IDs unique within a pipeline. Duplicates = error at normalize time.
- `after` edges must reference existing step IDs. Dangling refs = error.
- DAG acyclicity checked at dispatch by the orchestrator, not here (the type doesn't prevent a cycle at construction).

## Where the pipeline gets consumed

1. **CLI** (`ship run`): loads the JSONC pipeline file, calls `normalize_pipeline`, POSTs `PipelineRunStartedPayload` with normalized form.
2. **Daemon** (`runtime_api::create_job`): runs `orchestrator::pipeline_loader::ensure_pipeline` (idempotent) to deserialize into the payload's `steps` field, then persists the event.
3. **Orchestrator**: consumes `PipelineRunStartedPayload`, reads `steps` as a `Pipeline`, runs `advance_pipeline` to dispatch.

Same `Pipeline` type, shared via ship-jobs, used by all three.

## Related

- [Orchestrator](/runtime/orchestrator/) — runtime execution of a pipeline.
- [Pipelines Overview](/pipelines/) — user-facing concept.
- [Pipelines Reference](/pipelines/) — file format + schema.
- [Architecture Overview](/architecture/) — where ship-jobs sits.
