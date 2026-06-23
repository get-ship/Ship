# Known gaps — @ship/collections

The pack components ship against the live `ship.collections.*` surface in
`@ship/sdk-client` (see `packages/sdk-client/src/ship-sdk.ts:78-134`).
A few of the spec's calls (per `.artifacts/design-drop/design/domains/collections/spec.md`)
are not yet on the SDK — the components can still ship using the methods
they have, but the gaps below are the follow-up SDK work that lands the
spec's full contract.

## 1. `ship.collections.watch(collection)` — live item updates

The spec calls for `sui-collection-table` to consume a live `Watch`
channel so it can apply `set / append / merge / remove / heartbeat`
patches in place (see spec.md §"sui-collection-table" and the
`watch_stream` fixture in `collections.demo.json`).

The SDK exposes:
- `list()` — collection summaries
- `state({active})` — schema + total
- `detail({id})` — schema + items + total
- `items({collection})` — paginated rows
- `create` / `update_schema` / `create_item` / `update_item` / `delete_item` / `get_item`

**No `watch` op.** `sui-collection-table` falls back to re-querying
`items()` on mount and does not patch in place. The visual
"applying patches" state the spec lists is not reachable until
`ship.collections.watch` lands on the SDK (the daemon route
`CollectionService.Watch` is the upstream surface).

**Workaround for v1:** the built-in view can poll `items()` on a
short interval (e.g. 5s) to approximate live behaviour. Strict
eventual consistency, not real-time — but it covers the demo path
and lets us ship the rest of the pack now.

**Follow-up:** add `watch(collection, cursor?)` to `ShipSDK.collections`
in `packages/sdk-client/src/ship-sdk.ts`, returning a `LiveSource`
of patch frames. The runtime already supports SSE/patch streams
(`@ship/sui-runtime`'s patch.ts:37-39, in the external sui repo); the controller hooks
up `applyPatch` from `@ship/sui-runtime/patch` to mutate the rows
signal in place.

## 2. `ship.collections.query_items(inline_ast | named_query, params, limit)` — query runner

The spec calls for `sui-query-runner` to dispatch a real query AST
(or named query reference) to the daemon's `QueryItems` RPC.

The SDK surface routes all read access through `state` / `detail` /
`items` — there is no `query_items` op. The runner ships a v0 that
calls `items({collection})` and renders the result set; the `source`
prop is captured by the editor and emitted on `ship-query-change`
for the host to forward, but the runner does not actually execute
arbitrary queries.

**Follow-up:** add `query_items(args: { collection, source, params?, limit? })`
to the SDK and have the runner route through it. The rich Kysely-TS
compilation that would let `sui-query-editor` produce a typed AST
is gated on the `@ship/db` toolchain (spec.md §"Deferred").

## 3. `ship.collections.named_queries.*` — save / list named queries

The spec calls for `sui-query-editor`'s "save as named query" path
to write to `.ship/collections/queries/` (spec.md §"sui-query-editor").
No SDK method is exposed for this in v1; the host owns the file
write through its own surface until the SDK op lands.

## 4. Reproject progress UI

`ship-schema-editor` ships the **warning** the spec requires (see
`collections.demo.json:170-174` and the controller's
`needsReproject` / `reprojectWarning` pair). The "rebuild progress
UI beyond the warning" is on the deferred list per spec.md §"Deferred"
and not in scope for v1.
