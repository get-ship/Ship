---
group: Pipelines
order: 2
title: Pipeline Reference
description: Pipeline JSONC format — every field, every step type, resolution order, and the official schema.
audience: internal
---

Pipelines are JSONC files at `.ship/pipelines/<name>.jsonc`. Two shapes — short (agents list) and long (explicit steps). Both parse into the same normalized form before dispatch.

The official schema is at [`schemas/pipeline.schema.json`](https://raw.githubusercontent.com/madvib/ship/main/schemas/pipeline.schema.json). Add the `$schema` key at the top of your file for editor autocompletion:

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/pipeline.schema.json",
  "name": "my-pipeline",
  // …
}
```

## File-level fields

| Field | Required | Type | Purpose |
|---|---|---|---|
| `name` | yes | string | Pipeline identifier. Shown in logs and `ship jobs`. |
| `description` | no | string | Human-readable purpose. |
| `agents` | mutually exclusive with `steps` | string[] | Short form: sequential agent list. |
| `steps` | mutually exclusive with `agents` | Step[] | Long form: explicit DAG of steps. |
| `vars` | no | object | Pipeline-level default variables. Merged below CLI `--var` flags. |
| `timeout` | no | string | Maximum duration (e.g. `"30m"`). Consumed by step monitors. |
| `max_retries` | no | integer | Gate-retry limit. After this many failures, the gate's diff-rule halts the pipeline. |
| `auto_merge` | no | boolean | When true, the runtime runs `git merge` on the job branch after the final step completes. Replaces the former `type: merge` step. |
| `step_specs` | no | string[] | Optional per-step spec bodies indexed by step position. When present, the orchestrator writes each to `.artifacts/pipeline/NN-<step_id>.md` at launch. |

## Short form

```jsonc
{
  "name": "quick",
  "agents": ["planner", "coder", "reviewer"]
}
```

Expands to three steps sequentially dependent on each other:

```
planner (after: [])
coder   (after: [planner])
reviewer (after: [coder])
```

Short form is for linear agent pipelines. Any branching, non-agent steps, or gates require the long form.

## Long form — `steps`

```jsonc
{
  "name": "fan-out",
  "steps": [
    { "id": "setup", "agent": "default" },
    { "id": "web", "agent": "web-lane", "after": ["setup"] },
    { "id": "api", "agent": "rust-runtime", "after": ["setup"] },
    { "id": "integration", "agent": "test-writer", "after": ["web", "api"] }
  ]
}
```

Each entry is either a shorthand string (just an agent id) or a step object.

### Step object fields

| Field | Required | Type | Purpose |
|---|---|---|---|
| `id` | yes | string | Unique within the pipeline. Referenced by `after`. |
| `type` | no | enum | `agent` (default), `review`, `run`. See "Step types" below. `gate` accepted as alias for `review`. |
| `agent` | depends on type | string | Required for `agent` and `review` types. |
| `model` | no | string | Provider-specific model override (e.g. `opus`, `sonnet`). |
| `provider` | no | string | Provider override (e.g. `anthropic`). |
| `after` | no | string[] | Step IDs this depends on. Empty/absent = root step. |
| `goal` | no | string | Short instruction surfaced to the agent. |
| `run` | required for `run` type | string | Shell command to execute. Replaces `agent`. |
| `gate` | no | object | Diff-rule gate on this step. See "Gates" below. |

Shorthand rule: a string entry in `steps` is equivalent to `{ "id": <string>, "agent": <string>, "after": [<previous step>] }`. Useful for linear mid-pipeline sections.

## Step types

### `agent`

The default. Spawns the named agent in the worktree. The agent reads the job spec and performs work. Emits `job.step_completed` on clean exit, `job.step_failed` on non-zero.

```jsonc
{ "id": "implement", "type": "agent", "agent": "rust-runtime", "model": "opus" }
```

### `review` (alias: `gate`)

Agent-backed reviewer of the previous step. Same mechanics as `agent` but semantically read-only — the review agent sees the diff and passes or fails. Often paired with `auto_merge: true` so a passing review triggers merge.

```jsonc
{
  "id": "review",
  "type": "review",
  "agent": "reviewer",
  "model": "sonnet",
  "after": ["implement"]
}
```

### `run`

Plain shell command. No agent process. Used for build/test/lint steps that don't need an LLM.

```jsonc
{
  "id": "check",
  "type": "run",
  "run": "cargo test --workspace",
  "after": ["implement"]
}
```

Non-zero exit = `job.step_failed`; pipeline halts.

### Human handoff (no `type: human`)

There is no `type: human` step. The legacy variant was removed in the v1 contract freeze; the loader rejects it. For an explicit pause-point, emit `job.blocked { needs_human: true }` from a `run`/`agent` step — that routes through the human gateway service (see [Human Gateway](/reference/runtime/human-gateway/)).

## Gates

Two distinct concepts share the name — be careful:

1. **`type: review` / `type: gate`** — a step that is itself an agent-backed review. The reviewer reads the diff and emits pass/fail.
2. **Diff-rule `gate` attribute on any step** — declarative rules the runtime checks against the step's output (e.g., "no files outside `src/foo/**` changed," "test file count increased"). Authored as a `gate:` block on the step. Failure triggers `review_retry::retry_gated_step` which re-dispatches the step up to `max_retries` times.

The review step type produces an agent-flavored yes/no. The gate attribute does a mechanical check on the diff. Both can appear on the same pipeline — review the agent's output, then mechanically verify constraints.

### Canonical `gate:` shape

`require_changed` and `forbid_changed` are **lists of rule objects**. Each rule's `match`/`not_match` are **lists of glob strings**. The legacy single-object and pipe-separated string shorthands were removed in the v1 contract freeze; the loader rejects them.

```jsonc
{
  "gate": {
    "require_changed": [
      { "match": ["**/*.rs"], "not_match": ["**/tests/**"], "min": 1 }
    ],
    "forbid_changed": [
      { "match": [".artifacts/**", "Cargo.lock"], "min": 0 }
    ]
  }
}
```

## DAG dependencies

Steps with no `after` are root steps — they run first. Multiple root steps run in parallel. A step runs when every id in its `after` has completed.

```jsonc
{
  "steps": [
    { "id": "a", "agent": "planner" },
    { "id": "b", "agent": "researcher" },
    { "id": "c", "agent": "writer", "after": ["a", "b"] }
  ]
}
```

DAG validation happens in the orchestrator, not at parse time. A cycle (`after` refs form a loop) surfaces at dispatch as a runtime error. Dangling refs (after referencing an id that doesn't exist) also fail at dispatch.

## Template variables

Step `goal`, `run`, and prompt text support MiniJinja interpolation. Variables come from three layers, merged in order (last wins):

1. Pipeline `vars` defaults.
2. CLI flags (`ship run --var k=v`).
3. Job-level vars set in the spec's frontmatter.

```jsonc
{
  "name": "per-feature",
  "vars": { "branch_prefix": "feat" },
  "steps": [
    {
      "id": "impl",
      "agent": "rust-runtime",
      "goal": "Implement feature on {{ branch_prefix }}/{{ feature_slug }}"
    }
  ]
}
```

## auto_merge

With `auto_merge: true`, the runtime runs `git merge --no-ff <job-branch>` on the parent branch after the final step completes cleanly. Emits `job.merged`; cleanup follows (worktree removal, branch deletion, tmux session kill). Replaces the former `type: merge` step.

Without `auto_merge`, the job sits in Completed after the last step. A human or another pipeline can merge manually.

## Resolution order

When `ship run <pipeline-or-spec>` fires, the runtime resolves the pipeline in this order:

1. Job payload has inline `steps` → use them (MCP dispatch path).
2. Spec frontmatter has `pipeline: <name>` → load `.ship/pipelines/<name>.jsonc`.
3. None → default pipeline: `implement (job's agent) → review (reviewer, sonnet) → merge`.

The default pipeline exists so `ship run` works without authoring any pipeline file. Spec frontmatter cannot define inline pipelines; put them in `.ship/pipelines/`.

## Step specs

For pipelines where each step needs its own tailored spec body (not a shared one), author `step_specs:` as a list indexed by step position:

```jsonc
{
  "name": "multi-spec",
  "steps": [
    { "id": "plan", "agent": "planner" },
    { "id": "build", "agent": "coder", "after": ["plan"] }
  ],
  "step_specs": [
    "# Plan\n\nResearch the problem space, write options.",
    "# Build\n\nImplement option 1 from the plan."
  ]
}
```

At dispatch, the orchestrator writes each entry to `.artifacts/pipeline/00-plan.md`, `01-build.md`, and so on inside the worktree. Each agent's dispatch prompt points at its own step file rather than a shared one.

## Runtime behavior in brief

1. `job.created` lands, orchestrator's `handle_job_created` runs.
2. `isolation.prepare` creates the git worktree on `job/<slug>`.
3. First step (or all root steps) dispatched.
4. Agent exits → `job.step_completed` → `advance_pipeline` → next step(s).
5. On completion: `auto_merge` runs if set, otherwise the job sits Completed.
6. `job.merged` or terminal failure → cleanup (worktree removed, branch deleted, tmux session killed).

For the full trace including replay on daemon startup, see [Orchestrator](/reference/runtime/orchestrator/).

## Not yet implemented

Documented in the source as aspirations but not available today:

- Conditional steps via `when:` expressions.
- `on_failure: retry` + `retry_target` + `max_retries` at the step level (pipeline-level `max_retries` exists for gate retries).
- Pipeline registry (`ship install pipeline:tdd`).
- Parallel agent steps with file-scope enforcement beyond file claims.
- Webhook / notify step types.
- Per-pipeline concurrency config.

## Related

- [Overview](/reference/pipelines/) — concept + example.
- [ship-jobs crate](/reference/pipelines/ship-jobs/) — types backing the pipeline file.
- [Orchestrator](/reference/runtime/orchestrator/) — runtime execution.
- [Triggers](/reference/triggers/) — a trigger can dispatch a pipeline via `pipeline.trigger` action.
