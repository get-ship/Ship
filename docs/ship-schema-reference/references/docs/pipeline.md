---
group: Schema Reference
title: Pipelines
section: reference
order: 8
---

# Pipeline Schema — pipelines/*.yaml

**Location**: `.ship/pipelines/<name>.yaml`
**Schema**: `pipeline.schema.json`

Workflow templates defining a DAG of steps for job execution. Pipelines coordinate multiple agents through a sequence of work, review, and merge steps.

## Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Pipeline identifier. Matches filename without `.yaml`. Pattern: `^[a-z0-9][a-z0-9-]*$` |
| `description` | string | no | What this pipeline does |
| `steps` | array | yes | Ordered list of steps forming a DAG. At least one required. |

## Step fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique step ID within the pipeline. Pattern: `^[a-z0-9][a-z0-9-]*$` |
| `type` | enum | yes | `agent`, `gate`, `merge`, `human` |
| `agent` | string | yes (agent/gate) | Agent profile ID, e.g. `rust-runtime` |
| `after` | string[] | no | Step IDs that must complete first. Empty = root step. |
| `model` | string | no | Model override, e.g. `sonnet`, `opus`, `haiku` |
| `provider` | string | no | Provider override, e.g. `anthropic` |

## Step types

| Type | Description |
|------|-------------|
| `agent` | Runs an agent to do work. Requires `agent`. |
| `gate` | Reviews the output of prior steps. Requires `agent`. Returns pass/fail. |
| `merge` | Merges the working branch. |
| `human` | Pauses and waits for human action. |

## Example

```yaml
name: feature
description: Standard feature pipeline with specialist, review, and merge.

steps:
  - id: implement
    type: agent
    agent: rust-runtime

  - id: review
    type: gate
    agent: reviewer
    after: [implement]
    model: opus

  - id: merge
    type: merge
    after: [review]
```
