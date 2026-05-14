---
group: Triggers
order: 2
title: Triggers Reference
description: Trigger file shape, source types, action types, template context, lifecycle, schema.
audience: internal
---

A trigger is a JSONC file in `.ship/hooks/*.jsonc` that subscribes to an event type (or an external source) and runs one or more actions when matching events fire. Reactive automation without writing code — declarative JSONC, hot-reloaded when you edit.

**Naming note.** The code, directory, and types are currently called "hooks" (legacy name). The product concept is "triggers" — the rename is planned. This page uses "trigger" for the concept and points at `hooks` paths/types for the code.

## File shape

```jsonc
{
  "name": "log-failed-jobs",
  "description": "Emit a derived notice event when a job fails.",
  "trigger": {
    "event": "job.failed"
  },
  "actions": [
    {
      "type": "event.emit",
      "event_type": "hook.job_failed_notice",
      "payload": {
        "job_id": "{{ event.entity_id }}",
        "triggered_by": "{{ event.id }}"
      }
    }
  ]
}
```

Fields:

| Field | Required | Purpose |
|---|---|---|
| `name` | yes | Unique identifier. |
| `description` | no | Human-readable intent. |
| `trigger.event` | one-of | Internal event type to match. Mutually exclusive with `trigger.source`. |
| `trigger.source` | one-of | External trigger source (webhook, file, or cron). Mutually exclusive with `trigger.event`. |
| `trigger.filter` | no | Dot-path filter on the event payload. Match runs only when the filter resolves to a truthy value. |
| `actions` | yes | Ordered list. Each step has a `type` and type-specific fields. |
| `actions[].when` | no | Jinja expression — action skips if falsy. |

The `actions[].type` vocabulary is a **closed enum at v1**: only
`collection.insert`, `collection.update`, `event.emit`, and
`pipeline.trigger` are accepted. Unknown action types fail schema
validation at load time. Adding a new action type is a coordinated
change across `schemas/trigger.schema.json`, the loader at
`crates/core/ship-platform/src/hooks/`, and this page.

## External sources

Instead of `trigger.event`, a trigger can use `trigger.source` to react to external signals.

### Webhook source

Fires when an HTTP POST arrives at a registered path. The daemon exposes a **public** endpoint at `/api/hooks/webhook/*path` — the trigger's own `secret` is the auth boundary.

```jsonc
{
  "name": "on-github-push",
  "trigger": {
    "source": {
      "type": "webhook",
      "path": "/webhooks/github",
      "secret": "env:GH_WEBHOOK_SECRET"
    }
  },
  "actions": [
    {
      "type": "event.emit",
      "event_type": "hook.github_push",
      "payload": { "ref": "{{ event.payload.body.ref }}" }
    }
  ]
}
```

Source fields:

| Field | Required | Default | Purpose |
|---|---|---|---|
| `type` | yes | — | `"webhook"` |
| `path` | yes | — | Route suffix. Must start with `/webhooks/`. |
| `secret` | no | — | Verified against `X-Ship-Webhook-Secret` header. Use `env:VAR_NAME` to read from the environment. |

Security: Always set a secret. Rate limit is 60 req/min/path. Response: `200 {"ok":true}` · `401` bad secret · `404` unknown path · `429` rate limited.

Emitted event: `external.webhook.<trigger_name>` · payload: `{ "path", "body" }`.

### File source

Fires when a file matching the watch path or glob changes.

```jsonc
{
  "name": "on-src-change",
  "trigger": {
    "source": {
      "type": "file",
      "watch": "src/**/*.rs",
      "events": ["created", "modified"],
      "debounce_ms": 300
    }
  },
  "actions": [
    {
      "type": "event.emit",
      "event_type": "hook.src_changed",
      "payload": { "path": "{{ event.payload.path }}" }
    }
  ]
}
```

Source fields:

| Field | Required | Default | Purpose |
|---|---|---|---|
| `type` | yes | — | `"file"` |
| `watch` | yes | — | Path or glob. Relative paths resolve from the project root. |
| `events` | no | all three | `"created"`, `"modified"`, `"deleted"` — defaults to all. |
| `debounce_ms` | no | `200` | Debounce window in milliseconds. |

Emitted event: `external.file.<trigger_name>` · payload: `{ "path", "event_kind", "timestamp" }`.

### Cron source

Fires on a schedule.

```jsonc
{
  "name": "nightly-digest",
  "trigger": {
    "source": {
      "type": "cron",
      "expression": "0 2 * * *",
      "timezone": "America/New_York"
    }
  },
  "actions": [
    { "type": "pipeline.trigger", "pipeline": "nightly-digest" }
  ]
}
```

Source fields:

| Field | Required | Default | Purpose |
|---|---|---|---|
| `type` | yes | — | `"cron"` |
| `expression` | yes | — | Standard 5-field cron (`min hour dom month dow`). Optional seconds prefix. |
| `timezone` | no | `"UTC"` | IANA timezone name. |

Restart behavior: last-fire times are persisted; no double-fire on restart; missed runs are not caught up.

Emitted event: `external.cron.<trigger_name>` · payload: `{ "scheduled_for", "fired_at" }`.

## Action types

Four today. All typed in `ship_platform::hooks::def::HookActionType`:

| Type | Effect |
|---|---|
| `collection.insert` | Insert a new item into a collection. Fields: `collection`, `data`, optional `template_file`. |
| `collection.update` | Update items matching a filter. Fields: `collection`, `filter`, `data`. |
| `event.emit` | Emit a derived event through the kernel. Fields: `event_type`, `payload`. |
| `pipeline.trigger` | Dispatch a pipeline run. Fields: `pipeline`, `args`. |

Actions are evaluated in order. An action's failure is logged and the next action proceeds — triggers don't halt a chain on one bad step. Use `when` if a step should be conditional.

## Template context

Actions are rendered with MiniJinja against a context:

| Variable | Meaning |
|---|---|
| `event` | The triggering event envelope — `event.event_type`, `event.entity_id`, `event.payload_json` parsed, `event.actor_id`, etc. |
| `workspace` | The active workspace (if any). |
| `actor` | The actor that emitted the event (if known). |
| `results` | Outputs from preceding actions in the same trigger run. |

Common patterns: `{{ event.entity_id }}`, `{{ event.payload.slug }}`, `{{ event.payload.body.ref }}`.

## Lifecycle

1. `ship_platform::hooks::loader::load_hooks_dir(ship_dir)` reads `.ship/hooks/*.jsonc` into `Vec<HookDef>`. Returns a `LoadReport` noting any parse errors — one bad file doesn't block the rest.
2. `HookEngine` holds the loaded set and subscribes to the kernel for all events.
3. When an event fires, the engine finds matching triggers, renders actions against the template context, dispatches each in order.
4. `ship_platform::hooks::watcher::start_watcher(engine, ship_dir)` watches `.ship/hooks/` for file changes and reloads the engine on add/remove/modify. No daemon restart needed.
5. External source adapters (webhook, file, cron) are also notified on hot-reload and update their registered trigger sets.

## Not to be confused with

- **User hooks in ship.jsonc** — the `hooks` array in `.ship/ship.jsonc` declares shell commands to run on provider lifecycle events (`PreToolUse`, `Stop`, etc.). Those are compiler-level — emitted into each provider's native config and run by the provider, not by Ship.

The name collision is real. See [Hooks](/runtime/hooks/) for both meanings side by side.

## Schema

The official JSON Schema for trigger files is at [`schemas/trigger.schema.json`](https://raw.githubusercontent.com/madvib/ship/main/schemas/trigger.schema.json). Add a `$schema` key to your `.jsonc` file to get editor autocompletion and inline validation:

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/trigger.schema.json",
  "name": "my-trigger",
  ...
}
```

## Related

- [Events](/runtime/events/) — event types triggers can subscribe to.
- [Collections](/collections/) — target of `collection.insert` / `collection.update` actions.
- [Pipelines](/workflow/pipelines/) — target of `pipeline.trigger` actions.
