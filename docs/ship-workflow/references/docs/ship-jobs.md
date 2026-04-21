---
audience: agent-only
---

Moved to [ship-pipelines/ship-jobs](../../ship-pipelines/references/docs/ship-jobs.md) as part of the pipelines-split. This file is retained only so inbound file-path links don't 404; it does not render on the site.

# Pipeline Internals

`crates/core/ship-jobs/`. The shared types and normalization logic that both the CLI (`ship run`) and the daemon (orchestrator) use to understand pipelines. Pure types + a pipeline parser. No runtime, no kernel — this crate is below both.

```
ship-jobs/src/
  lib.rs           — module tree
  step.rs          — PipelineStep, StepType, infer_step_type
  pipeline.rs      — Pipeline, PipelineMeta, normalize_pipeline, short-form expansion
  spec.rs          — JobSpec (the markdown+frontmatter input)
  payload.rs       — event payloads for job/step events
  record.rs        — job record shapes shared with projections
  event_types.rs   — event type string constants specific to ship-jobs
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

Three variants. `Agent` and `Run` are obvious. `Review` is a step that takes the prior step's output and runs an agent to evaluate it — distinct from the `gate:` attribute which is a diff-rule check on any step. Historical alias: `gate` (as a step type) maps to `Review`; the `gate:` step attribute is different.

## PipelineStep

```rust
pub struct PipelineStep {
    pub id: String,
    pub step_type: StepType,
    pub agent: Option<String>,
    pub goal: Option<String>,
    pub after: Vec<String>,   // step ids this depends on
}
```

`after` is the DAG edge set. An empty `after` means root step. Multiple `after` entries are an AND — the step runs when all dependencies complete.

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

A flat vec + a root-finder. The orchestrator's `advance_pipeline` does its own DAG traversal against the `after` edges; the `Pipeline` type doesn't need to expose a full graph API.

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

Pipeline-level metadata from the YAML/JSONC. Extracted during `normalize_pipeline` separately from the step list.

- `vars` — pipeline-level defaults. Merged under CLI `--var` flags at dispatch time.
- `timeout`, `max_retries` — orchestrator consumes these for step monitoring and gate retries.
- `step_specs` — optional per-step spec bodies (indexed by position). When present, the orchestrator writes each into `.artifacts/pipeline/NN-<step_id>.md` at job launch, replacing the legacy one-spec-shared-across-steps behavior.
- `auto_merge` — when true, the runtime runs `git merge` on the job branch after the final step. Replaces the former `type: merge` step.

## normalize_pipeline

```rust
pub fn normalize_pipeline(input: &str) -> Result<(Value, PipelineMeta), String>
pub fn normalize_pipeline_yaml(input: &str) -> Result<(Value, PipelineMeta), String>   // alias
```

Takes a pipeline definition string and returns `(normalized_steps_as_json, meta)`. Input format detection: JSONC first (primary format), YAML fallback (legacy, still supported). Both routes produce the same normalized output.

Accepts two pipeline shapes:

### Short form (agents list)

```yaml
name: tdd
agents: [planner, implementer, reviewer]
```

Expanded into three sequential steps:

- `planner` at `after: []`
- `implementer` at `after: [planner]`
- `reviewer` at `after: [implementer]`

Handled by `agents_to_steps(&agents)`.

### Long form (explicit steps)

```yaml
name: fan-out
steps:
  - id: plan
    agent: planner
  - id: build-a
    agent: coder
    after: [plan]
  - id: build-b
    agent: coder
    after: [plan]
  - id: review
    agent: reviewer
    after: [build-a, build-b]
```

Each step can be:

- A string (just an agent id) — shorthand for `{ id: ..., agent: <string>, after: [prev] }`.
- A map with explicit `id`, `type`, `agent`, `after`, `run`, etc.

Mixed lists work — string shorthand for simple steps, maps for complex ones.

### infer_step_type

Used during normalization to decide the `type` field when the user omits it:

```rust
pub fn infer_step_type(val: &serde_json::Value) -> &'static str {
    if val.is_string()          { return "agent"; }
    if val.has("run")            { return "run"; }
    if val.has("type == review|gate") { return "review"; }
    // ...
    "agent"  // default
}
```

Keeps common cases terse. Users rarely type `type: agent`.

## JobSpec

`spec.rs`. The input to a pipeline run. Authored as markdown with YAML frontmatter in `.artifacts/`:

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
  - crates/core/ship-jobs/src/pipeline.rs
vars:
  branch_prefix: feat
---
<markdown body — free-form>
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

The frontmatter fields are typed here so callers parse once and work with a struct. Body is opaque markdown — it becomes the agent's spec content via `spec_content` in the `JobCreatedPayload`.

## Payload types

`payload.rs`. Serialization shapes for job/step events:

- `JobCreatedPayload` — emitted by the POST handler when `ship run` or equivalent creates a job. Carries slug, agent, branch, spec_path, and (after `ensure_pipeline`) the parsed pipeline steps.
- `StepDispatchedPayload` — when a step begins executing.
- `StepCompletedPayload` — clean completion.
- More payload types for cancellation, failure, merge, gate pass/fail.

These types are shared between the runtime orchestrator (which emits them) and the CLI (which observes them for `ship tail`, `ship jobs` pipeline detail).

## Record types

`record.rs`. Shapes shared between ship-jobs and `runtime::projections::job`. `JobRecord` ultimately lives in projections (computed, not materialized), but the field shape is defined here so the crate that owns the payloads also owns the record shape.

This is why `runtime::projections::job::JobRecord` re-exports or wraps the ship-jobs types — keeps one source of truth for the job projection.

## event_types.rs

String constants for job/step events specific to ship-jobs (not in `ship-types::event_types`):

```rust
pub mod event_types {
    pub const STEP_DISPATCHED: &str = "job.step_dispatched";
    pub const STEP_COMPLETED: &str = "job.step_completed";
    pub const STEP_FAILED: &str = "job.step_failed";
    // ... the pipeline-level step events
}
```

**Note**: the `db-cleanup-01` job (in flight) lifts the string literals for `job.step_completed`, `job.step_failed`, `job.cancelled`, `job.merged`, `job.spawn_failed` into `ship-types::event_types`. Once that lands, these constants here may be deduplicated into the shared crate. Today they exist in both places; the tests enforce consistency.

## Invariants

- Crate is pure. No filesystem, no network, no database.
- `normalize_pipeline` accepts both short and long forms, always emits long form. Orchestrator only sees normalized input.
- Step IDs must be unique within a pipeline. `normalize_pipeline` returns an error on duplicates.
- `after` edges must reference step IDs that exist in the pipeline. Dangling refs = error.
- DAG must be acyclic. Cycle detection happens in the orchestrator's `advance_pipeline`, not here (the type doesn't prevent a cycle at construction time — it just represents a vec of steps).

## Where the pipeline gets consumed

1. **CLI** (`ship run`): loads pipeline YAML, calls `normalize_pipeline`, POSTs to `/api/runtime/jobs` with the normalized form in `JobCreatedPayload`.
2. **Daemon** (`runtime_api::create_job`): enriches the payload via `orchestrator::pipeline_loader::ensure_pipeline` (idempotent, returns the `Pipeline` struct deserialized into the payload's `steps` field). Persists the event.
3. **Orchestrator**: consumes `JobCreatedPayload`, reads the `steps` field (already a `Pipeline`), runs `advance_pipeline` to dispatch each step.

Same `Pipeline` type, shared via ship-jobs, used by all three.

## Related

- [Orchestrator](/reference/runtime/orchestrator/) — how pipelines execute.
- [Pipelines](/reference/workflow/pipelines/) — user-facing authoring reference.
- [Architecture Overview](/reference/architecture/) — where ship-jobs sits (below runtime, above ship-types).
