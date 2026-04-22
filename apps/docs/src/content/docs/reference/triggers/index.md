---
title: "Triggers"
description: "Reactive JSONC files that match events and run actions. Authored in .ship/hooks/ (pending rename to .ship/triggers/)."
sidebar:
  label: "Triggers"
  order: 1
---
A trigger is a JSONC file in `.ship/hooks/*.jsonc` that subscribes to an event type and runs one or more actions when matching events fire. Reactive automation without writing code — declarative JSONC, hot-reloaded when you edit.

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
| `trigger.event` | yes | Event type to match (e.g. `workspace.compiled`, `collection.item_created`). |
| `trigger.filter` | no | Dot-path filter on the event payload. Match runs only when the filter resolves to a truthy value. |
| `actions` | yes | Ordered list. Each step has a `type` and type-specific fields. |
| `actions[].when` | no | Jinja expression — action skips if falsy. |

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

Common patterns: `{{ event.entity_id }}`, `{{ event.payload.slug }}`, `{{ workspace.branch }}`.

## Lifecycle

1. `runtime::hooks::loader::load_hooks_dir(ship_dir)` reads `.ship/hooks/*.jsonc` into `Vec<HookDef>`. Returns a `LoadReport` noting any parse errors — one bad file doesn't block the rest.
2. `HookEngine` holds the loaded set and subscribes to the kernel for `trigger.event` types.
3. When an event fires, the engine finds matching triggers, renders actions against the template context, dispatches each in order.
4. `runtime::hooks::watcher::start_watcher(engine, ship_dir)` watches `.ship/hooks/` for file changes and reloads the engine on add/remove/modify. No daemon restart needed.

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

The schema uses `oneOf` with a `type` discriminator to describe the four action variants precisely. Files also validate at load time — unknown action types fail with a clear message.

## Related

- [Events](/reference/runtime/events/) — event types triggers can subscribe to.
- [Collections](/reference/collections/) — target of `collection.insert` / `collection.update` actions.
- [Pipelines](/reference/workflow/pipelines/) — target of `pipeline.trigger` actions.
