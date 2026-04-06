# Ship Pipelines

Pipelines define how a job executes — a DAG of steps that the runtime walks automatically. When a step completes, the engine finds unblocked steps and dispatches them.

## Quick Start

Create `.ship/pipelines/tdd.yaml`:

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

Reference it in your spec frontmatter:

```yaml
---
slug: my-feature
agent: rust-runtime
pipeline: tdd
---
```

When a job is created from this spec, the runtime loads the pipeline and executes step by step.

## Step Types

### `agent` — Run an agent in the worktree

The workhorse. Spawns an agent with `ship use <agent>`, launches in tmux, waits for completion.

```yaml
- id: implement
  type: agent
  agent: rust-runtime
  model: opus          # optional model override
  provider: anthropic  # optional provider override
```

### `gate` — Review step

Same as agent but semantically a review. The gate agent reads the original spec and the diff, then passes or fails.

```yaml
- id: review
  type: gate
  agent: reviewer
  model: sonnet
  after: [implement]
```

### `merge` — Merge to target branch

Merges the job branch. Cleans up worktree after merge.

```yaml
- id: merge
  type: merge
  after: [review]
```

*Status: planned — currently logs and waits for manual merge.*

### `human` — Human approval gate

Pauses the pipeline until a human acts.

```yaml
- id: approve
  type: human
  after: [review]
```

*Status: planned — creates inbox item, pipeline pauses.*

## DAG Dependencies

Steps declare dependencies with `after`. Steps with no `after` are root steps and run first. Multiple steps can share the same dependency — they run in parallel when unblocked.

```yaml
steps:
  - id: setup
    type: agent
    agent: default

  # These two run concurrently after setup
  - id: web
    type: agent
    agent: web-lane
    after: [setup]

  - id: api
    type: agent
    agent: rust-runtime
    after: [setup]

  # Waits for both
  - id: integration
    type: agent
    agent: test-writer
    after: [web, api]
```

## Default Pipeline

If no pipeline is specified, the runtime injects a default:

```
implement (job's agent) → gate (reviewer) → merge
```

## Pipeline Resolution

When a job is created, the runtime resolves its pipeline in order:

1. Job already has `steps` in payload → use them
2. Spec frontmatter has `pipeline: <name>` → load from `.ship/pipelines/<name>.yaml`
3. Spec frontmatter has inline `steps:` → use them
4. None of the above → inject default pipeline

## How It Works

1. Job created → runtime loads pipeline, dispatches root step
2. Agent works in the worktree, calls `update_job(status: "completed")`
3. Runtime catches the completion event
4. Marks current step as completed
5. Finds unblocked steps (all `after` deps completed)
6. For each unblocked step:
   - `agent`/`gate`: graceful exit current agent → `ship use <next-agent>` → relaunch
   - `merge`: git merge (planned)
   - `human`: create inbox item, pause (planned)
7. Repeat until all steps done

## File Location

Pipelines live in `.ship/pipelines/` in your project root. They're reusable — any spec can reference them by name.

```
.ship/
  pipelines/
    tdd.yaml
    feature.yaml
    hotfix.yaml
  agents/
    ...
```

## Aspirational

- **Conditional steps** — `when: "check.scope == 'small'"` for branching flows
- **Retry loops** — `on_failure: retry, retry_target: implement, max_retries: 2`
- **Named pipeline registry** — `ship install pipeline:tdd` from the registry
- **Parallel agent steps** — multiple agents in the same worktree with file scope enforcement
- **Webhook/notify steps** — `type: webhook` for external integrations
- **Concurrency config** — per-pipeline max concurrent steps
