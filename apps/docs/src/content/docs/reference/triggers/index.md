---
title: "Triggers"
description: "Reactive JSONC files that match events and run actions. Authored in .ship/hooks/ (pending rename to .ship/triggers/)."
sidebar:
  label: "Triggers"
  order: 1
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
| `trigger.event` | one-of | Internal event type to match (e.g. `workspace.compiled`). Mutually exclusive with `trigger.source`. |
| `trigger.source` | one-of | External trigger source (webhook, file, or cron). Mutually exclusive with `trigger.event`. |
| `trigger.filter` | no | Dot-path filter on the event payload. Match runs only when the filter resolves to a truthy value. |
| `actions` | yes | Ordered list. Each step has a `type` and type-specific fields. |
| `actions[].when` | no | Jinja expression — action skips if falsy. |

## External sources

Instead of `trigger.event`, a trigger can use `trigger.source` to react to external signals. Three source kinds are supported.

### Webhook source

Fires when an HTTP POST arrives at a registered path. The daemon exposes a **public** (unauthenticated) endpoint at `/api/hooks/webhook/*` — authentication is the trigger's own `secret` field.

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

**Security notes:**
- The path is public — any client can POST to it. The `secret` is the auth boundary. Always set a secret for production webhooks.
- Secrets written as `env:VAR_NAME` are resolved once at trigger-load time and on hot-reload.
- Rate limit: 60 requests/minute per registered path (token bucket).

**Response codes:** `200 {"ok":true}` on accept · `401` on bad/missing secret · `404` on unknown path · `429` on rate limit exceeded.

The emitted event has type `external.webhook.<trigger_name>` with payload `{ "path": "...", "body": <parsed JSON or null> }`.

### File source

Fires when a file or directory matching the watch path/glob changes.

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
| `events` | no | all three | Event kinds to react to: `"created"`, `"modified"`, `"deleted"`. |
| `debounce_ms` | no | `200` | Coalescing debounce window in milliseconds. |

**Trust model:** Watch paths are relative to the project root. Absolute paths are allowed but must be accessible to the daemon process. The trigger fires for any file that matches the path — ensure globs don't accidentally include sensitive files.

The emitted event has type `external.file.<trigger_name>` with payload `{ "path": "...", "event_kind": "created|modified|deleted", "timestamp": "..." }`.

### Cron source

Fires on a schedule defined by a cron expression.

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
    {
      "type": "pipeline.trigger",
      "pipeline": "nightly-digest"
    }
  ]
}
```

Source fields:

| Field | Required | Default | Purpose |
|---|---|---|---|
| `type` | yes | — | `"cron"` |
| `expression` | yes | — | Standard 5-field cron (`min hour dom month dow`). Optionally prefix with a seconds field (6 fields total). |
| `timezone` | no | `"UTC"` | IANA timezone name. |

**Restart behavior:** Last-fire timestamps are persisted so restarts don't double-fire within the same scheduled interval. Missed runs while the daemon was down are not caught up — the next fire is computed from the current time.

The emitted event has type `external.cron.<trigger_name>` with payload `{ "scheduled_for": "...", "fired_at": "..." }`.

## Action types

Four today. All typed in `runtime::hooks::def::HookActionType`:

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

1. `runtime::hooks::loader::load_hooks_dir(ship_dir)` reads `.ship/hooks/*.jsonc` into `Vec<HookDef>`. Returns a `LoadReport` noting any parse errors — one bad file doesn't block the rest.
2. `HookEngine` holds the loaded set and subscribes to the kernel for all events.
3. When an event fires, the engine finds matching triggers, renders actions against the template context, dispatches each in order.
4. `runtime::hooks::watcher::start_watcher(engine, ship_dir)` watches `.ship/hooks/` for file changes and reloads the engine on add/remove/modify. No daemon restart needed.
5. External source adapters (webhook, file, cron) are also notified on hot-reload and update their registered trigger sets.

## Not to be confused with

- **User hooks in ship.jsonc** — the `hooks` array in `.ship/ship.jsonc` declares shell commands to run on provider lifecycle events (`PreToolUse`, `Stop`, etc.). Those are compiler-level — emitted into each provider's native config and run by the provider, not by Ship.
- **Runtime hooks trait** — `runtime::config::hooks::RuntimeHooks` is an embedder extension point for log-sink injection. Different concept, same word.

The name collision is real. See [Hooks](/reference/runtime/hooks/) for the three meanings side by side.

## Schema

The official JSON Schema for trigger files is at [`schemas/trigger.schema.json`](https://raw.githubusercontent.com/madvib/ship/main/schemas/trigger.schema.json). Add a `$schema` key to your `.jsonc` file to get editor autocompletion and inline validation:

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/trigger.schema.json",
  "name": "my-trigger",
  ...
}
```

The schema uses `oneOf` to describe the source kinds and action variants. Files also validate at load time — unknown source types or conflicting `event`+`source` fields fail with a clear message.

## Related

- [Events](/reference/runtime/events/) — event types triggers can subscribe to.
- [Collections](/reference/collections/) — target of `collection.insert` / `collection.update` actions.
- [Pipelines](/reference/workflow/pipelines/) — target of `pipeline.trigger` actions.
