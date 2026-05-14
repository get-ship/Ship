---
group: Pipelines
order: 2
title: Step I/O — produces, consumes, promotes
description: Declarative artifact handoff between pipeline steps via produces, consumes, and promotes.
audience: internal
---

Declarative artifact handoff between pipeline steps. Steps declare what they produce, what they need from prior steps, and which outputs to promote on success.

## Concepts

### produces

A list of named outputs the step is expected to upload. Each entry becomes a contract between the step and the orchestrator.

```yaml
produces:
  - name: failing_tests
    scope: run       # default
    required: true   # step fails if absent from step_done.outputs
```

Fields:
- `name` — logical handle, unique within the step; used in `consumes` and `promotes`
- `scope` — `run` (default, ephemeral), `project` (persistent), `user` (user-personal)
- `required` — when `true`, missing output fails the step before advancing

### consumes

A list of output names from prior steps. The orchestrator resolves each name before dispatching the step:

1. Finds the producing step (must be earlier in the DAG).
2. Looks up the artifact ID from the producer's `step.done_declared` event.
3. Fetches the artifact record.
4. Writes the blob to `<worktree>/.artifacts/input/<name>` (hardlink or copy).
5. Adds an "Inputs available" section to the dispatch prompt.

If resolution fails, the step is failed with a clear message and downstream steps are not dispatched.

### promotes

A list of name → target_scope mappings. After a step's `step.observed` resolves to Advance, each promote runs:

1. Looks up the artifact ID from the step's `step.done_declared` outputs map.
2. Calls `ArtifactService::promote(id, target_scope)`.
3. Emits `artifact.moved`. Promoted artifacts survive run GC.

Promote failures are non-fatal — a warning is logged but the pipeline continues.

## Scope transition rules

Valid promotes (enforced at load time):

```
run → project    ✓
run → user       ✓
project → user   ✓
run → run        ✗  (rejected)
project → run    ✗  (rejected)
user → anything  ✗  (rejected)
```

## Agent contract

When a step declares `produces`:

1. Upload artifacts via `upload_artifact`:
   ```
   upload_artifact(
     scope: "run",
     name: "failing_tests",
     bytes: <base64>,
     metadata: { produced_name: "failing_tests" }
   )
   ```

2. Report IDs in `step_done.outputs`:
   ```
   step_done({
     summary: "...",
     outputs: { "failing_tests": "01J..." }
   })
   ```

Required outputs missing from `outputs` fail the step.

## Resolution details

### Event store query

The orchestrator queries `step.done_declared` events for the job to find the artifact ID:

```
events WHERE entity_id = job_id
       AND event_type = 'step.done_declared'
       AND payload.step_id = producing_step_id
       → payload.outputs[consumed_name]
```

### Filesystem projection

Resolved inputs are written to `.artifacts/input/<name>` as hardlinks (or copies across device boundaries). These files stay for the worktree lifetime and are cleaned up with the worktree.

### Prompt injection

The dispatch prompt for a consuming step includes:

```
## Inputs available

  - **failing_tests** — id: `01J...`, path: `.artifacts/input/failing_tests`, size: 4.3 KB
```

## Load-time validation

`ensure_pipeline` rejects:
- `consumes` with no producer in the DAG
- A step consuming its own output
- `promotes.name` not matching a `produces.name` on the same step
- `promotes.target_scope` with an invalid transition

## Implementation notes

- Resolution happens in `pipeline.rs` (`advance_pipeline_inner`) before the step is dispatched.
- Required-output check happens in `router.rs` when `step.done_declared` arrives.
- Promotes run in `router.rs` after `step.observed` resolves to Advance, before `advance_pipeline`.
- All artifact I/O helpers live in `orchestrator/step_io.rs`.
