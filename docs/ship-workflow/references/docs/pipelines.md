---
audience: agent-only
---

Moved to [ship-pipelines](../../ship-pipelines/references/docs/) as part of the pipelines-split. This file is retained only so inbound file-path links don't 404; it does not render on the site.

# Pipelines

Pipelines define how a job executes — a DAG of steps that the runtime walks automatically. When a step completes, the engine finds unblocked steps and dispatches them.

## Defining a Pipeline

Create `.ship/pipelines/<name>.yaml`:

```yaml
name: tdd
description: Test-driven development

steps:
  - id: test
    type: agent
    agent: test-writer
    model: sonnet

  - id: impl
    type: agent
    agent: rust-runtime
    after: [test]

  - id: gate
    type: gate
    agent: reviewer
    model: sonnet
    after: [impl]

  - id: merge
    type: merge
    after: [gate]
```

Reference it in spec frontmatter:

```yaml
---
slug: my-feature
agent: rust-runtime
pipeline: tdd
---
```

## Step Types

### `agent`

Spawns an agent in the worktree. The agent reads the job spec and works.

```yaml
- id: implement
  type: agent
  agent: rust-runtime
  model: opus          # optional
  provider: anthropic  # optional
```

### `gate`

Review step. Same as agent but semantically a read-only review. The gate agent reads the original spec and the diff, then passes or fails.

```yaml
- id: review
  type: gate
  agent: reviewer
  model: sonnet
  after: [implement]
```

### `merge`

Merges the job branch into the current branch. Emits `job.merged` which triggers cleanup (worktree removal, branch deletion, tmux session kill).

```yaml
- id: merge
  type: merge
  after: [review]
```

### `human`

Pauses the pipeline until a human acts. Creates an inbox item.

```yaml
- id: approve
  type: human
  after: [review]
```

## DAG Dependencies

Steps declare dependencies with `after`. Steps with no `after` are root steps and run first. Steps with shared dependencies run in parallel when unblocked.

```yaml
steps:
  - id: setup
    type: agent
    agent: default

  - id: web
    type: agent
    agent: web-lane
    after: [setup]

  - id: api
    type: agent
    agent: rust-runtime
    after: [setup]

  - id: integration
    type: agent
    agent: test-writer
    after: [web, api]
```

## Default Pipeline

If no pipeline is specified, the runtime injects:

```
implement (job's agent) → gate (reviewer, sonnet) → merge
```

## Resolution Order

1. Job payload has `steps` → use them
2. Spec frontmatter has `pipeline: <name>` → load `.ship/pipelines/<name>.yaml`
3. Spec frontmatter has inline `steps:` → use them
4. None → default pipeline

## Runtime Behavior

1. Job created → runtime loads pipeline, dispatches root step
2. Agent works, calls `update_job(status: "completed")`
3. Runtime marks current step completed
4. Finds unblocked steps (all `after` deps resolved)
5. For each: graceful exit current agent → `ship use <next-agent>` → relaunch in same worktree
6. Repeat until all steps done

## Aspirational

- Conditional steps (`when:` expressions)
- Retry loops (`on_failure: retry`, `retry_target`, `max_retries`)
- Pipeline registry (`ship install pipeline:tdd`)
- Parallel agent steps with file scope enforcement
- Webhook/notify step types
- Per-pipeline concurrency config
