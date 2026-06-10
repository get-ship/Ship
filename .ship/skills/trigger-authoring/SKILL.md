---
name: trigger-authoring
audience: internal
stable-id: trigger-authoring
description: Author Ship triggers — JSONC files in .ship/triggers/*.jsonc that match a Ship event and run actions. Reactive automation layer. Distinct from agent/provider hooks, which are a compiler concern.
tags: [triggers, extension-authoring, jsonc, jinja, automation]
authors: [ship]
---

# Ship Triggers — The Reactive Automation Layer

A **trigger** is a declarative JSONC file that listens for a Ship event and runs one or more actions when it fires. You write it as `.ship/triggers/<name>.jsonc`, the daemon hot-reloads on file change, and your automation is live.

> **Naming note.** Internal Rust types still carry the `Hook*` prefix (`HookDef`, `HookEngine`, `HookAction`) for historical reasons; the public/filesystem surface is consistently **triggers**. "Hooks" elsewhere in Ship refers to the agent/provider compiler hooks (pre/post-tool), a different system entirely. Keep the two separate in your head.

## Minimal example

`.ship/triggers/log-failed-jobs.jsonc`:

```jsonc
{
  "name": "log-failed-jobs",
  "description": "Emit a follow-up event when a job ends in failure.",
  "trigger": {
    "event": "job.failed"
  },
  "actions": [
    {
      "type": "event.emit",
      "event_type": "trigger.job_failed_notice",
      "payload": {
        "job_id": "{{ event.entity_id }}",
        "triggered_by": "{{ event.id }}"
      }
    }
  ]
}
```

Drop the file in `.ship/triggers/`, the daemon watcher picks it up, and the next `job.failed` event produces a `trigger.job_failed_notice` event with the rendered payload.

## Definition shape

```jsonc
{
  "name": "<unique slug>",          // required — used in logs and dedup
  "description": "<sentence>",      // optional but strongly encouraged
  "trigger": {
    "event": "<event.type>",        // required — see §Events below
    "filter": { ... }               // optional — see §Filters
  },
  "actions": [                      // 1+ actions, run sequentially
    { "type": "<action>", ... }
  ]
}
```

## Events

Every Ship event is a trigger candidate. Pick from these namespaces:

| Namespace | Events | Typical use |
|---|---|---|
| `workspace.*` | `created`, `deleted`, `compiled`, `started`, `branch_switched` | Snapshot state, warm caches |
| `session.*` | `started`, `completed`, `failed`, `cancelled`, `suspended` (+ `session.turn.*`) | Digest daily activity |
| `actor.*` | `created`, `woke`, `slept`, `stopped`, `crashed` | Alert on agent crashes |
| `job.*` | `created`, `claimed`, `completed`, `failed` | Triage on failure, archive completed |
| `pipeline.*` | `started`, `step_completed`, `step_failed`, `blocked`, `completed` | Pipeline notifications |
| `collection.*` | `item_created`, `item_updated`, `item_deleted`, `query_executed` | Derived data, search indexing |
| `skill.*` | `started`, `completed`, `failed` | Skill-usage metrics |
| `gate.*` | `passed`, `failed` | Auto-merge on pass, flag on fail |
| `webhook.received` | inbound from external providers (Telegram, GitHub, etc.) | Integration glue |
| `connection.*` | `created`, `expired` | Re-auth prompts |
| `config.changed` | `.ship/` files changed | Warm reloads |

Canonical event-type constants live in `crates/core/ship-types/src/event_types.rs`.

## Filters

A filter narrows which event instances fire the trigger. Dot-path keys match against the event payload:

```jsonc
"trigger": {
  "event": "job.failed",
  "filter": {
    "payload.pipeline_name": "ship-dev",
    "payload.retry_count": { "$gte": 3 }
  }
}
```

Filter operators:
- Equality: `"key": "value"` or `{ "$eq": "value" }`
- Comparison: `{ "$gt": N }`, `{ "$gte": N }`, `{ "$lt": N }`, `{ "$lte": N }`, `{ "$ne": X }`
- Set: `{ "$in": [a, b, c] }`, `{ "$nin": [...] }`
- Existence: `{ "$exists": true }`
- Array contains: `{ "$contains": "x" }`
- AND is implicit across keys. For OR, define two triggers.

## Actions

Four action types are implemented today. More arrive with Phase 2 (connections + notifications).

### `collection.insert`
```jsonc
{
  "type": "collection.insert",
  "collection": "bug_triage",
  "data": {
    "title": "{{ event.payload.issue.title }}",
    "severity": "{{ 'high' if 'critical' in event.payload.issue.labels else 'normal' }}",
    "source_event": "{{ event.id }}"
  }
}
```
Or, for multiline JSON, reference a `.j2` file:
```jsonc
{
  "type": "collection.insert",
  "collection": "bug_triage",
  "template_file": "triggers/templates/triage-bug.j2"
}
```
The template must render to a valid JSON object.

### `collection.update`
```jsonc
{
  "type": "collection.update",
  "collection": "tasks",
  "filter": { "id": "{{ event.entity_id }}" },
  "data": { "status": "done", "completed_at": "{{ now() }}" }
}
```

### `event.emit`
```jsonc
{
  "type": "event.emit",
  "event_type": "trigger.digest_ready",
  "payload": { "summary": "..." }
}
```
Emitted events can themselves trigger other triggers — see §Recursion guard.

### `pipeline.trigger`
```jsonc
{
  "type": "pipeline.trigger",
  "pipeline": "bug-triage",
  "args": { "issue_url": "{{ event.payload.issue.html_url }}" }
}
```
Use this as the escape hatch for anything that needs multi-step async work or agent involvement. If Jinja + one-step actions aren't enough, trigger a pipeline and let the pipeline handle complexity.

### Conditional actions

Every action accepts an optional `when` Jinja expression. Falsy → skip:

```jsonc
{
  "type": "pipeline.trigger",
  "pipeline": "escalate",
  "when": "{{ 'critical' in event.payload.labels }}",
  "args": { ... }
}
```

## Jinja rendering context

Inside `data`, `payload`, `filter`, `template_file` content, and `when`:

| Variable | Contents |
|---|---|
| `event` | The full event envelope: `id`, `type`, `entity_id`, `payload`, `occurred_at` |
| `workspace` | Current workspace metadata |
| `actor` | Actor that caused the event, when available |
| `results` | Array of prior-action results in this trigger, indexed by position. `{{ results[0].id }}` after a `collection.insert` |
| `now()` | Current UTC timestamp (ISO 8601) |

The renderer is a fresh `minijinja::Environment` per evaluation —
custom filters are not registered today, so `{{ a | b }}` chains
limited to minijinja built-ins (`length`, `upper`, `lower`, `default`,
etc.). The renderer lives at
`crates/core/ship-platform/src/triggers/render.rs` — register a filter
there if you genuinely need one.

## Recursion guard

Triggers can emit events that fire other triggers. To prevent infinite loops the engine enforces **max depth 3**. Beyond depth 3, further trigger invocations are dropped and a `trigger.depth_exceeded` diagnostic event is emitted.

## What triggers cannot do

- **No shell execution.** No `sh`, no `eval`, no arbitrary commands. If you need a shell command, trigger a pipeline that runs it.
- **No raw HTTP.** The `connection.proxy` and `notify` actions (Phase 2) are the only way to hit external services. Until they land, triggers stay local.
- **No custom event types.** Only the canonical event namespaces match. You can *emit* a `trigger.*` event, but agents can't invent new top-level event namespaces.
- **No ordering guarantees across triggers.** Two triggers on the same event run concurrently. If ordering matters, compose one trigger or use a pipeline.

## Validation

The daemon validates every trigger on load and on file change:
- `trigger.event` must be a known event type
- Each `action.type` must be a known action
- Referenced `template_file` paths must exist and parse
- Jinja expressions must compile (actual evaluation is lazy)

Invalid triggers log a warning and are skipped — they do not crash the daemon.

## Authoring workflow

1. Write the JSONC file in `.ship/triggers/<name>.jsonc`
2. Confirm the daemon loaded it: `tail -n 20 ~/.ship/logs/shipd.stderr.log | grep trigger`
3. Fire the event manually for testing — via the CLI (`ship events emit <type>`) or by producing it naturally
4. Inspect the result (collection write, emitted event, pipeline run) via the events view or Studio

## Common patterns

**On job failure, notify and archive:**
```jsonc
{
  "name": "triage-failed-jobs",
  "trigger": { "event": "job.failed" },
  "actions": [
    { "type": "collection.insert", "collection": "failed_jobs",
      "data": { "job_id": "{{ event.entity_id }}", "reason": "{{ event.payload.reason }}" } },
    { "type": "event.emit", "event_type": "trigger.job_failure_archived",
      "payload": { "job_id": "{{ event.entity_id }}" } }
  ]
}
```

**On external webhook, route to a pipeline:**
```jsonc
{
  "name": "github-issue-triage",
  "trigger": {
    "event": "webhook.received",
    "filter": { "payload.provider": "github", "payload.action": "opened" }
  },
  "actions": [
    { "type": "pipeline.trigger", "pipeline": "triage",
      "args": { "issue_url": "{{ event.payload.issue.html_url }}" } }
  ]
}
```

**On collection write, derive a second collection item:**
```jsonc
{
  "name": "index-notes",
  "trigger": {
    "event": "collection.item_created",
    "filter": { "payload.collection": "notes" }
  },
  "actions": [
    { "type": "collection.insert", "collection": "note_search_index",
      "data": {
        "note_id": "{{ event.payload.item_id }}",
        "title": "{{ event.payload.data.title }}",
        "excerpt": "{{ event.payload.data.content[:200] }}"
      } }
  ]
}
```

## Relation to other systems

- **Views** can read a collection that a trigger writes to, giving you live dashboards of derived state. Views subscribe via `<data name="…" source="ship.collection.items(…)" live />` and stay current as triggers mutate the table. See `ship-views`.
- **Pipelines** handle multi-step or agent-involving work. Triggers are the cheap reflex; pipelines are the thoughtful response.
- **Connections** (Phase 2) will add `notify` and `connection.proxy` actions so triggers can send messages and call external APIs through authenticated proxies.

## Do not confuse with

- `**/hooks.toml`, `.claude/hooks/`, provider-level pre/post-tool hooks — those are **agent/provider hooks**, a compiler concern for Claude/Codex/Cursor/Gemini/OpenCode. Completely separate system that happens to share a name.
