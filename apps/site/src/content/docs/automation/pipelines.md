---
title: Pipelines
description: Multi-step work as a declarative DAG — steps run in order or in parallel, gates pause for review, runs are resumable.
section: Automation
order: 1
kicker: Automation
tags: [automation, pipelines]
related: [triggers]
status: ready
---

A **pipeline** is a JSONC file describing multi-step work — who does what, what waits on what, where humans review. You author it once at `.ship/pipelines/<name>.jsonc`; the runtime walks the steps, dispatches each agent, and routes completion downstream.

## When to reach for one

- **Distinct phases** — write → review → merge, each a different agent (maybe a different model).
- **Parallel work** — branches that run independently after shared setup, then converge.
- **Gates and approvals** — a reviewer agent, or a human, decides whether work moves on.

If a single agent run is enough, skip the pipeline. Pipelines pay off when there's orchestration to express.

## A small example

`.ship/pipelines/tdd.jsonc`:

```jsonc
{
  "name": "tdd",
  "description": "Test-driven development",
  "steps": [
    { "id": "test",   "agent": "test-writer" },
    { "id": "impl",   "agent": "rust-runtime", "after": ["test"] },
    { "id": "review", "type": "gate", "agent": "reviewer", "after": ["impl"] }
  ]
}
```

```bash
ship run tdd --spec spec.md
```

The runtime allocates an isolated workspace, runs the test agent, then the implementation agent, then opens a review gate. If the gate passes (and `auto_merge` is set), the work merges back to your parent branch.

Because every step transition is an event on the log, a run that crashes can resume where it left off — see [Why a runtime](/docs/why-a-runtime/). To fire pipelines automatically, pair them with [Triggers](/docs/triggers/).
