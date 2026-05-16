---
group: Pipelines
order: 1
title: Pipelines
description: Multi-step work as a declarative DAG. Steps can run in parallel, gates pause for review, runs are resumable.
---

A pipeline is a JSONC file that describes multi-step work — who does what, what waits on what, and where humans review. You author it once at `.ship/pipelines/<name>.jsonc`; the runtime walks the steps, dispatches each agent, and routes completion signals downstream.

## When to reach for a pipeline

<sui-card padding="md">
<strong slot="header">Distinct phases</strong>
Write → review → merge. Each phase wants a different agent, possibly a different model.
</sui-card>

<sui-card padding="md">
<strong slot="header">Parallel work</strong>
Two branches that can run independently after some shared setup, then converge.
</sui-card>

<sui-card padding="md">
<strong slot="header">Gates and approvals</strong>
A reviewer agent (or a human) decides whether the work moves on.
</sui-card>

If a single agent run is enough, skip the pipeline. Pipelines pay off when you have orchestration to express.

## A small example

`.ship/pipelines/tdd.jsonc`:

```jsonc
{
  "name": "tdd",
  "description": "Test-driven development",
  "steps": [
    { "id": "test", "agent": "test-writer" },
    { "id": "impl", "agent": "rust-runtime", "after": ["test"] },
    { "id": "review", "type": "gate", "agent": "reviewer", "after": ["impl"] }
  ]
}
```

Run it:

```bash
ship run tdd --spec spec.md
```

The runtime allocates an isolated workspace, runs the test agent, then the implementation agent, then opens a review gate. If the gate passes (and `auto_merge` is set in the pipeline), the work merges back to your parent branch.

## Short form for sequential pipelines

When every step is an agent run and they're strictly sequential, drop the `steps` array and list agents:

```jsonc
{
  "name": "quick",
  "agents": ["planner", "coder", "reviewer"]
}
```

This expands to three sequential steps. Use the explicit `steps` form when you need branches, gates, or parallel work.

## What you get

- **Resumable runs.** A run that crashed at 2am picks up where it stopped — the runtime replays from its event log.
- **Observable.** Every step transition is an event. The dashboard shows live state across the run.
- **Composable.** Pipelines can call other pipelines as nested steps.

## Related

- [Competing approaches](./competing-approaches/) — fan out K agents, judge picks the best result.
- [Triggers](/triggers/) — fire a pipeline when an event matches.
