# Collections UI — Component & View Brief

**For:** ship-designer + operator (visual/interaction decisions are theirs).
**This doc provides:** what components exist, what each is for, which RPC methods each calls. No layout, no interaction mechanisms, no HTML — those are decided visually.
**Backend:** `CollectionService` (10 RPCs), merged. Contract: `proto/ship/ops/v1/collection.proto`. Shape rationale: `README.md`.
**Demo data:** `fixtures/collections.demo.json` (decoded, designer-readable — build/screenshot against this before backend wiring).

## The pattern (every domain follows it)

Two UI deliverables per domain:
1. **Component pack** — `<sui-collection-*>` components wrapping the domain's features, independently composable into custom views. Lives in `packages/components/ui-components/src/`.
2. **Built-in view** — domains that warrant it ship a view composing the pack, in the binary. Collections warrants one (a data explorer + schema/query editor + query testing).

Collections is the **reference** for this two-layer pattern.

## The one hard requirement: typed-from-schema

The collection's **schema is the single source** every component reads. Table columns, the item form's fields + validation, query autocomplete, and result columns all derive from it. `Item.data_json` and `schema_json` arrive as JSON-bytes on the wire; the client decodes before rendering (fixtures show decoded shapes). `system: true` collections (feedback/annotations) are read-only and never offered schema-edit or delete.

Live data: `Watch`'s first frame is a full snapshot (`set` at root), then incremental patches (`append`/`merge`/`remove`/… mirroring `patch.ts`). Any component showing a collection should be able to apply patches in place.

---

## Component pack — `sui-collection-*`

Each is usable standalone (a user could compose `sui-collection-table` into their own view) and assembles into the built-in view.

### `sui-collection-list`
- **What:** index of the space's collections; flags system collections.
- **Calls:** `List()`.
- **Use:** sidebar/picker of available collections; selection drives everything else.

### `sui-collection-table`
- **What:** the data explorer — rows of a collection with columns derived from its schema; cursor-paginated; live.
- **Calls:** `QueryItems(collection, cursor, limit)` for pages; `Watch(collection)` for live updates; `Get(id)` to know the schema for columns.
- **Use:** primary "browse the data" surface. Consumes `Watch` frames to stay live.

### `sui-item-detail`
- **What:** one item rendered by its typed fields, with a raw-JSON fallback.
- **Calls:** `GetItem(collection, id)`.
- **Use:** drill-in from a table row.

### `sui-item-editor`
- **What:** create or edit one item as a form generated from the schema; validates against the schema client-side (mirrors server rejection).
- **Calls:** `CreateItem(collection, data)` / `UpdateItem(collection, id, data)`; reads schema via `Get`.
- **Use:** new-item and edit-item flows. `json`-typed fields may fall back to a raw editor.

### `sui-schema-editor`
- **What:** author/edit a collection's schema — fields with name/type/required/indexed/unique/default. Supports both a guided field-list and a raw editor surface (operator chose: both, toggle).
- **Calls:** `CreateSchema(id, schema)` / `UpdateSchema(id, schema)`.
- **Use:** new-collection wizard and schema editing. Must surface the **reproject warning** when an `UpdateSchema` change rebuilds the projection from events (data preserved, column dropped) — this is a real consequence of the migration model, not hidden.

### `sui-query-editor`
- **What:** write a query against a collection, with field/op autocomplete from the schema.
- **Calls:** none directly — produces a captured query (AST) handed to the runner. (Rich Kysely-TS autocomplete depends on the later `@ship/db` toolchain; until then it's a plain editor.)
- **Use:** the authoring half of query testing; also "save as named query" (writes `.ship/collections/queries/`, not an RPC in v1).

### `sui-query-runner`
- **What:** run a query and render typed results + row count + elapsed time.
- **Calls:** `QueryItems(inline_ast | named_query, params, limit)`.
- **Use:** the testing half — run, see results, iterate. Pairs with `sui-query-editor`.

### `sui-collection-picker`
- **What:** compact collection selector.
- **Calls:** `List()`.
- **Use:** anywhere another surface needs "pick a collection" (e.g. artifacts upload already wants this).

### `sui-watch-badge`
- **What:** small live / reconnecting / paused indicator.
- **Calls:** reflects `Watch` connection status (no data of its own).
- **Use:** shared affordance on any live surface (table, detail).

### States every data-bearing component handles
loading · empty (first-run) · empty-filtered · error (retryable) · ok · live (applying patches). The `views/artifacts/` view is the reference for how this repo handles the loading/error/empty/ok flow and tone.

---

## Built-in view — `views/collections/` ("Data Studio")

A view that composes the pack to do three jobs. **How they're arranged/navigated is a visual decision** — this only says what it must let the user do and what it composes.

- **Explore** — browse/filter/page/edit a collection's items, live. Composes `sui-collection-table` + `sui-item-detail` + `sui-item-editor`.
- **Schema** — author/edit the selected collection's schema. Composes `sui-schema-editor`. Reproject warning required.
- **Query** — write, run, and save queries against the selected collection. Composes `sui-query-editor` + `sui-query-runner`.
- **Collection selection** — composes `sui-collection-list`; system collections selectable but read-only (Explore only).
- **New collection** — schema-first entry via `sui-schema-editor` → `CreateSchema`.

Rail entry + route metadata (`manifest.jsonc`) are the view author's call; `auth: user`.

---

## RPC ↔ action reference

| Action | RPC |
|---|---|
| list collections | `List` |
| collection schema + count | `Get` |
| browse items (paged) | `QueryItems` |
| live item updates | `Watch` |
| open one item | `GetItem` |
| create / edit / delete item | `CreateItem` / `UpdateItem` / `DeleteItem` |
| create / edit schema | `CreateSchema` / `UpdateSchema` |
| run a query | `QueryItems` (inline_ast or named_query) |

## Fixtures provided (`fixtures/collections.demo.json`)
- `List.response` — 5 collections (3 user, 2 system).
- `schemas` — decoded schemas covering the full type vocab + every constraint.
- `items` — decoded rows for notes/tasks/contacts (rich + sparse + null cases).
- `queries` — named queries with compiled SQL + typed results + timing.
- `watch_stream` — a 5-frame live sequence (snapshot → append → merge → remove → heartbeat).
- `errors` — validation / unsupported-query / reserved-id / reproject-warning, with tones.
- `empty_states` — copy for each empty case.

## Deferred (don't design against yet)
- Rich Kysely-TS authoring/autocomplete inside `sui-query-editor` (waits on `@ship/db` toolchain).
- Per-collection/per-query permission UI (capabilities don't reach collections at launch).
- Reproject progress UI beyond the warning.
