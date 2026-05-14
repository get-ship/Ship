---
group: Schema Reference
title: Pipelines
section: reference
order: 8
---

# Pipeline Schema — pipelines/*.jsonc

**Location**: `.ship/pipelines/<name>.jsonc`
**Schema**: `pipeline.schema.json`

Workflow templates defining a DAG of steps for job execution. Pipelines coordinate multiple agents through a sequence of work, review, and merge steps.

## Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Pipeline identifier. Matches filename without `.jsonc`. Pattern: `^[a-z0-9][a-z0-9-]*$` |
| `description` | string | no | What this pipeline does |
| `agents` | string[] | mutually exclusive with `steps` | Short form: sequential agent list |
| `steps` | array | mutually exclusive with `agents` | Ordered list of steps forming a DAG. At least one required. |
| `auto_merge` | boolean | no | When true, the runtime runs `git merge` after the final step completes. Replaces the former `type: merge` step. |

## Step fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique step ID within the pipeline. Pattern: `^[a-z0-9][a-z0-9-]*$` |
| `type` | enum | no | `agent` (default), `review`, `run`. `gate` is accepted as alias for `review`. |
| `agent` | string | yes (agent/review) | Agent profile ID, e.g. `rust-runtime` |
| `run` | string | yes (run type) | Shell command to execute |
| `after` | string[] | no | Step IDs that must complete first. Empty = root step. |
| `model` | string | no | Model override, e.g. `sonnet`, `opus`, `haiku` |
| `provider` | string | no | Provider override, e.g. `anthropic` |

## Step types

| Type | Description |
|------|-------------|
| `agent` | Runs an agent to do work. Default type. Requires `agent`. |
| `review` / `gate` | Agent-backed review of prior steps. Requires `agent`. Returns pass/fail. |
| `run` | Executes a shell command. Non-zero exit = pipeline halts. |

**Removed step types:** `merge` is replaced by `auto_merge: true` at the pipeline level. `human` type is recognized by the parser but the primary human-in-the-loop mechanism routes through the human gateway service via `job.blocked` events.

## Example

```jsonc
{
  "name": "feature",
  "description": "Standard feature pipeline with specialist, review, and auto-merge.",
  "auto_merge": true,
  "steps": [
    { "id": "implement", "type": "agent", "agent": "rust-runtime" },
    {
      "id": "review",
      "type": "review",
      "agent": "reviewer",
      "after": ["implement"],
      "model": "opus"
    }
  ]
}
```
