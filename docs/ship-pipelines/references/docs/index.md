---
group: Pipelines
order: 1
title: Pipelines Overview
description: Declarative DAGs of steps the runtime walks automatically. When to use one, how dispatch works, where they live.
---

A pipeline is a DAG of steps declared in `.ship/pipelines/<name>.yaml`. When a job uses a pipeline, the runtime walks the DAG: root steps run first, and as each step completes the orchestrator finds newly unblocked dependents and dispatches them. The loop continues until every step is done or the pipeline halts on an error or human gate.

## When a pipeline helps

Reach for a pipeline when:

- The work has more than one distinct phase (write → review → merge).
- Phases need different agents or different models (a fast sonnet reviewer after an opus implementer).
- Steps can run in parallel once shared work finishes (web + api branches after a shared setup).
- You want a gate — an agent that reviews a diff and passes/fails before merge.
- You need a `human:` step that pauses until explicitly unblocked.

Skip the pipeline when a single agent run is enough. A one-off `ship run` against a single agent doesn't need ceremony.

## Minimal example

`.ship/pipelines/tdd.yaml`:

```yaml
name: tdd
description: Test-driven development

steps:
  - id: test
    agent: test-writer

  - id: impl
    agent: rust-runtime
    after: [test]

  - id: review
    type: gate
    agent: reviewer
    after: [impl]
```

Run it:

```bash
ship run tdd --spec-file spec.md
```

The orchestrator creates a job on `job/<slug>`, allocates a git worktree, and dispatches `test` first. When `test` completes, `impl` fires. When `impl` completes, `review` fires as a gate. If the gate passes and `auto_merge` is set (see Reference), the job merges to the parent branch; otherwise it sits Completed.

## What happens when you `ship run`

1. CLI parses the pipeline YAML and normalizes it.
2. POSTs `job.created` to `shipd`.
3. Kernel persists; orchestrator's mailbox receives the event.
4. Orchestrator runs `ensure_pipeline(payload)` (already run by the handler, idempotent here).
5. Orchestrator calls `isolation.prepare` — git worktree created.
6. First step's agent spawns in the worktree.
7. Agent finishes, emits `job.step_completed`.
8. Kernel persists + delivers back to orchestrator.
9. `advance_pipeline` finds the next unblocked step, dispatches.
10. Loop until drained.

See the [architecture overview sequence diagram](/reference/architecture/) for the `ship run` trace in detail, and [Orchestrator](/reference/runtime/orchestrator/) for pipeline advancement internals.

## Short form — agents only

If every step is an agent run and they're strictly sequential, drop the `steps` array and list agents:

```yaml
name: quick
agents: [planner, coder, reviewer]
```

Normalized to three sequential steps: `planner` → `coder` → `reviewer`. Short form is for one-line pipelines; when you need branches, merges, or gates, use the explicit `steps` form.

## Where pipelines live

- Authored: `.ship/pipelines/<name>.yaml`.
- Referenced: `pipeline:` frontmatter field in a job spec (`.artifacts/*-spec.md`), or the `--pipeline` flag on `ship run`.
- Persisted at dispatch: parsed into `JobCreatedPayload.steps` before the event lands in the event log. Replay on daemon startup reads the parsed form, not the YAML — so editing a pipeline file after dispatch doesn't affect in-flight jobs.

## Not covered here

- The YAML schema in detail — see [Reference](/reference/pipelines/reference/).
- Orchestrator internals (pipeline advancement, gate retry, replay) — see [Orchestrator](/reference/runtime/orchestrator/).
- The `ship-jobs` crate that owns the types — see [ship-jobs](/reference/pipelines/ship-jobs/).

## Artifact I/O (phase 2)

Step-to-step artifact passing via `produces`/`consumes`/`promotes` declarations is a **phase-2 deliverable**. In phase 1, agents use the [`artifact_upload`/`artifact_get` MCP tools](/reference/artifacts/) directly to share files across steps — the pipeline orchestrator does not yet manage artifact routing automatically.

## Related

- [Orchestrator](/reference/runtime/orchestrator/) — how pipelines execute at runtime.
- [Workspaces](/reference/runtime/workspaces/) — where pipelines run.
- [Triggers](/reference/triggers/) — a trigger with a `pipeline.trigger` action dispatches a pipeline run.
- [Artifacts](/reference/artifacts/) — the artifact store agents use for step I/O.
