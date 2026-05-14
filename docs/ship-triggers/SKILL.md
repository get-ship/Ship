---
name: ship-triggers
stable-id: ship-triggers
description: Reference for Ship's reactive trigger system — JSONC files that match events and run actions. Not to be confused with agent/provider hooks (a compiler concern). Code currently lives under the legacy name `hooks` until rename lands.
tags: [reference, triggers, events, automation]
authors: [ship]
---

# Ship Triggers

Declarative event → action rules. A trigger is a JSONC file. The engine subscribes to the Ship event bus, matches events to triggers, and dispatches registered action handlers.

**Filesystem name:** `.ship/hooks/*.jsonc` (rename to `.ship/triggers/` is pending; see STATUS). "Hooks" in other contexts refers to agent/provider compiler hooks — different system.

## Surface

- **Types:** `HookDef`, `HookTrigger`, `HookAction`, `HookActionType` in `crates/core/ship-platform/src/hooks/def.rs`
- **Actions (4 today):** `collection.insert`, `collection.update`, `event.emit`, `pipeline.trigger`
- **Engine:** `HookEngine`, `HookService`, `spawn_hook_service` in `crates/core/ship-platform/src/hooks/engine.rs`
- **Loader:** `load_hooks_dir`, `parse_hook`, `validate_hook` in `crates/core/ship-platform/src/hooks/loader.rs`
- **Filter semantics:** `crates/core/ship-platform/src/hooks/filter.rs`
- **Jinja rendering:** `crates/core/ship-platform/src/hooks/render.rs` (context: `event`, `workspace`, `actor`, `results`, `now()`)
- **File watcher:** `crates/core/ship-platform/src/hooks/watcher.rs`
- **Event taxonomy:** `crates/core/ship-types/src/event_types.rs` — any declared event type is a valid trigger

## File map

| Path | Role |
|---|---|
| `.ship/hooks/*.jsonc` | User-authored trigger definitions |
| `crates/core/ship-platform/src/hooks/` | Engine, loader, executor, actions, rendering |
| `crates/core/ship-substrate/src/events/` | Event bus the engine subscribes to |

## Guarantees

- Triggers load on daemon start and on `.ship/hooks/` file change.
- Invalid triggers log a warning and are skipped; do not crash the daemon.
- Max recursion depth is 3 (a trigger emitting an event that fires another trigger → third depth is dropped).
- Actions in one trigger execute sequentially. Independent triggers on the same event execute concurrently. No ordering guarantee across triggers.
- Action results are addressable as `{{ results[0] }}` from subsequent actions in the same trigger.

## Related skills

- `trigger-authoring` (in `skills/`) — hands-on authoring guide, agent-consumed when writing a trigger.
- `ship-schema-reference` — JSONC hook schema documentation.
- `ship-collections` — source of `collection.item_*` events + target of `collection.insert`/`collection.update` actions.
