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

1. Job created → runtime loads pipeline, dispatches root step.
2. Agent works in the worktree. When its step's work is verifiably done, the
   agent calls `step_done` (MCP tool). `update_job` is reserved for
   whole-job lifecycle; it is **not** the per-step completion signal.
3. `StepMonitor` (per step) emits `step.observed` with a **verdict**:
   - `Advance` — pipeline advances to unblocked dependents
   - `Block` — pipeline halts, job marked failed
   - `Inconclusive` — pipeline still advances but `succeeded: false`
4. For each unblocked step: graceful exit current agent → `ship use <next-agent>`
   → relaunch in the same worktree.
5. Repeat until all steps done.

## Step Monitor

Every dispatched agent step is watched by a `StepMonitor` that decides
when the step has ended and what the verdict is. Source:
`crates/core/runtime/src/services/orchestrator/step_monitor/`.

### Completion paths

Whichever fires first ends the step:

| Trigger | Cause | Verdict default |
|---|---|---|
| `DoneDeclared` | agent called `step_done` | Advance |
| `TurnSettled` | provider Stop hook + settle window elapses | commits decide |
| `ProcessExit` | executor reports exit code | exit code decides |
| `Timeout` | idle heartbeat or hard cap fires | Inconclusive (or evidence-based) |
| `ChannelClosed` | exit channel dropped without signal | Inconclusive |

### Liveness model

Monitor layered signals, not just wall-clock:

- **Idle heartbeat (primary kill)** — samples the dispatched process tree
  every 60s via `sysinfo`. CPU-delta across descendants counts as activity.
  If the tree has been idle for 5 minutes (default), kill the step with
  `reason=idle_timeout`.
- **Hard cap (safety net)** — upper bound from step start (60 min default).
  Fires only when a runaway agent keeps CPU hot with no pipeline progress.
  `reason=hard_cap`.
- **Turn-end activity** — provider Stop hooks reset the idle timer and
  start a 15s settle window; late CPU activity extends the settle so
  mid-flight `cargo test` doesn't get truncated.
- **step_done short-circuit** — always wins; emits immediately.

### Kill on timeout

On any timeout path the monitor calls `executor.stop(handle)` **before**
emitting `step.observed`. Without this, the stalled agent keeps running
and fights the next step for worktree state (cargo locks, git index,
etc.). `process.signal_sent` + `process.exited` observability events fire
alongside.

### Evidence gathering

The monitor scans the worktree on completion: commits since step start
(via `git rev-list`) and uncommitted file count (via `git status
--porcelain`). These feed the verdict rules in
`events::job::compute_verdict`.

## MCP gate

Dispatched agents receive a curated tool surface. The tool gate has
two tiers and a preset:

- **CORE_TOOLS** — always visible (workspace CRUD)
- **PLATFORM_TOOLS** — visible to every dispatched agent (`step_done`,
  `log_progress`, `update_job`, `write_session_file`, `view_*`)
- **ORCHESTRATION_TOOLS** — only with `tools_preset: orchestrator`
  (`create_job`, `mesh_*`)

Presets: `worker` | `orchestrator` in the agent profile's `tools_preset`
field. A profile's `active_tools` (if non-empty) replaces the preset
entirely — the escape hatch.

### Dispatched-context signal

The gate detects whether the current MCP session is running inside a
dispatched agent worktree by walking up from `cwd` looking for
`.artifacts/.dispatched`. The marker file is written by
isolation-prep at dispatch time and contains the job slug. This
survives provider quirks (codex strips env vars from MCP
subprocesses, so env-based signalling is unreliable).

## Aspirational

- Conditional steps (`when:` expressions)
- Retry loops (`on_failure: retry`, `retry_target`, `max_retries`)
- Pipeline registry (`ship install pipeline:tdd`)
- Parallel agent steps with file scope enforcement
- Webhook/notify step types
- Per-pipeline concurrency config
- Per-step `idle_timeout` / `hard_cap` overrides (defaults only today)
- Mesh health-probe on approaching-timeout warn (step-heartbeat phase 2)
