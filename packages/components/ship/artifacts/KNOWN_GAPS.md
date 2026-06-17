# Known gaps — `@ship/artifacts`

The `@ship/artifacts` pack surfaces five components pre-bound to the
`ship.artifacts.*` SDK surface. As of writing, the SDK exposes a
read-only list and a single write (`upload`) — the design handoff
calls for `Watch`, `Patch`, `Promote`, and `Delete` ops, none of
which are in the SDK client yet. This file documents the gap and
how the host view (when it refactors to compose this pack) can work
around it.

## Missing SDK methods

| Method         | SDK status    | Pack impact                                                                                           |
| -------------- | ------------- | ----------------------------------------------------------------------------------------------------- |
| `watch`        | NOT IN SDK    | `ship-artifact-list` subscribes to `ship.artifacts.list({})` only. New artifacts require a manual list reload. |
| `patch`        | NOT IN SDK    | `ship-artifact-rename` emits `ship-artifact-rename { id, name }`; the host must call a `Patch` op when it lands, or implement via upload + delete as a stopgap. |
| `promote`      | NOT IN SDK    | `ship-artifact-promote` emits `ship-artifact-promote { id }`; the host wires a future `Promote` op when it lands. |
| `delete`       | NOT IN SDK    | `ship-artifact-delete` emits `ship-artifact-delete { id }`; the host wires a future `Delete` op when it lands. |

The present design handoff (`.artifacts/design-drop/design/domains/artifacts/handoff.md`)
treats `Watch`, `Patch`, `Promote`, and `Delete` as first-class ops.
Until they land in `@ship/sdk-client`, the host view (the eventual
refactor of `views/artifacts/`) has two options:

1. **Add the ops to the SDK client** (preferred). The runtime's
   `<data live>` + `live()` will pick them up automatically because
   the surface is generated from the client.
2. **Fallback to a list-reload workaround** for now. After any
   `ship-artifact-{promote,delete,rename}` event the host can call
   `ship.artifacts.list({})` again to refresh the projection. This
   is what the current `views/artifacts/controller.ts` does via the
   `refresh()` signal-nonce trick, and what the `ship-artifact-list`
   controller will continue to do until `Watch` lands.

## What this pack already does

- `ship-artifact-list` subscribes to `ship.artifacts.list({})` via
  `live()`. Renders the default six columns (name, mime, viewer
  badge, actor, age, size) and emits `ship-artifact-select` on row
  click.
- `ship-artifact-preview` is a pure presentation component driven by
  `prop:record` and `prop:url`. No SDK calls.
- `ship-artifact-filters` is a pure presentation component that
  emits `ship-artifact-filter { kind, value }`. The host wires the
  filter values to a list reload (or future `Watch` filters).
- `ship-artifact-actions` is a pure presentation component that
  emits `ship-artifact-{promote,delete,rename}`. The host owns the
  SDK call (which is the gap).
- `ship-artifact-upload` is a pure presentation component that
  emits `ship-artifact-upload { files, collection_id }`. The host
  calls `ship.artifacts.upload({ file, collection_id })`, which IS
  in the SDK.

## Decorative helpers

`decorate.ts` and `viewers.ts` (with `viewers.types.ts`) are moved
out of `views/artifacts/` and re-exported from the pack's
`src/index.ts`. They are pure functions — no SDK, no state. The view
will import them from the pack after the refactor lands.
