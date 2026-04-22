---
title: Step I/O — produces, consumes, promotes
description: Declarative artifact handoff between pipeline steps.
---

# Step I/O

Pipeline steps can declare what artifacts they produce, what artifacts they consume from prior steps, and which outputs to promote to a wider scope on success.

## Overview

```
step 1 (write-tests)
  produces: failing_tests (run scope, required)
       ↓
step 2 (implement)
  consumes: failing_tests
  produces: patch (run scope)
       ↓
step 3 (gate)
  consumes: failing_tests, patch
  promotes: patch → project
```

The orchestrator resolves `consumes` before dispatching each step and writes the resolved blobs to `.artifacts/input/<name>` in the worktree.

## Declaring produces

```yaml
steps:
  - id: write-tests
    type: agent
    agent: rust-runtime
    produces:
      - name: failing_tests
        scope: run       # default — ephemeral for this run
        required: true   # step fails if the agent doesn't produce this
```

### StepOutput fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | — | Logical handle. Unique within the step. Used in `consumes` and `promotes` references. |
| `scope` | `run` \| `project` \| `user` | `run` | Storage scope for the artifact. |
| `required` | boolean | `false` | When `true`, the step fails if the output is absent from `step_done.outputs`. |

## Declaring consumes

```yaml
  - id: implement
    type: agent
    agent: rust-runtime
    consumes:
      - failing_tests   # produced by an earlier step
```

The orchestrator:
1. Finds the step that declares `produces.name: failing_tests`.
2. Looks up the artifact ID from that step's `step.done_declared` event.
3. Fetches the artifact record and writes the bytes to `.artifacts/input/failing_tests` in the worktree.
4. Includes the resolved inputs in the dispatch prompt.

If resolution fails (artifact not found, producer didn't report it), the step is failed with a clear message and downstream steps are not dispatched.

**Cross-scope consumes are not supported in phase 2.** Only run-scope artifacts produced in the same run can be consumed.

## Declaring promotes

```yaml
  - id: gate
    type: agent
    agent: reviewer
    consumes:
      - patch
    promotes:
      - name: patch
        target_scope: project   # survives run GC
```

### Valid scope transitions

| From | To | Allowed |
|------|----|---------|
| `run` | `project` | ✓ |
| `run` | `user` | ✓ |
| `project` | `user` | ✓ |
| Any | `run` | ✗ |
| Higher | Lower | ✗ |

Promotes run after the step advances and are non-fatal — a failed promote logs a warning but does not halt the pipeline.

## Agent contract

When a step declares `produces`, the agent must:

1. Upload each artifact using `artifact_upload`:
   ```
   artifact_upload(
     scope: "run",
     name: "failing_tests",
     bytes: <base64>,
     metadata: { produced_name: "failing_tests" }
   )
   ```
   The tool returns `{ "id": "01J..." }`.

2. Include the IDs in `step_done.outputs`:
   ```
   step_done({
     summary: "wrote 4 failing tests",
     outputs: { "failing_tests": "01J..." }
   })
   ```

If a `required: true` output is missing from `outputs`, the orchestrator fails the step before the pipeline advances.

## Resolved inputs in the dispatch prompt

When a step has `consumes`, the dispatch prompt includes:

```
## Inputs available

  - **failing_tests** — id: `01J...`, path: `.artifacts/input/failing_tests`, size: 4.3 KB
```

The blob is also available on the filesystem at `.artifacts/input/failing_tests`. Agents can read it directly or use `artifact_get` with the ID.

## Load-time validation

The orchestrator rejects a pipeline at load time if:

- A `consumes` name has no producer in the pipeline.
- A step consumes its own output.
- A `promotes.name` doesn't match a `produces.name` on the same step.
- A `promotes.target_scope` is an invalid transition.

## Example: 3-step TDD pipeline

```yaml
name: tdd-io
steps:
  - id: write-tests
    type: agent
    agent: rust-runtime
    produces:
      - name: failing_tests
        required: true

  - id: implement
    type: agent
    agent: rust-runtime
    after: [write-tests]
    consumes:
      - failing_tests
    produces:
      - name: patch

  - id: gate
    type: agent
    agent: reviewer
    after: [implement]
    consumes:
      - failing_tests
      - patch
    promotes:
      - name: patch
        target_scope: project
```

## Related

- [Artifacts](/reference/artifacts/) — the underlying blob store.
- [Pipelines schema](/reference/schema-reference/pipeline/) — full field reference.
- [Pipelines overview](/reference/pipelines/) — DAG, steps, gates.
