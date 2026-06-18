# Handoff — `collections`

> **Status:** `in-design` — component set built; Data Studio view not yet composed
> **Upstream service:** `CollectionService` (10 RPCs) · **Contract:** `proto/ship/ops/v1/collection.proto`
> **Spec:** [`./spec.md`](./spec.md) · **Fixtures:** [`./fixtures/collections.demo.json`](./fixtures/collections.demo.json)

The **"Data Studio"** — a data explorer + schema editor + query tester. Collections
is the reference for the two-layer pattern (component pack + built-in view).

---

## 1. RPC surface

| Action | RPC |
|---|---|
| list collections | `List` |
| schema + count | `Get` |
| browse items (paged) | `QueryItems` |
| live item updates | `Watch` |
| open one item | `GetItem` |
| create/edit/delete item | `CreateItem` / `UpdateItem` / `DeleteItem` |
| create/edit schema | `CreateSchema` / `UpdateSchema` |
| run a query | `QueryItems` (inline_ast \| named_query) |

**Hard requirement — typed-from-schema:** the collection's schema is the single
source for table columns, item-form fields + validation, query autocomplete, and
result columns. `system: true` collections are read-only (no schema-edit/delete).

## 2. Requirements (the view does three jobs)

- [ ] **Explore** — browse/filter/page/edit items, live. (`sui-collection-table` + `sui-item-detail` + `sui-item-editor`)
- [ ] **Schema** — author/edit schema; **reproject warning** on destructive `UpdateSchema`. (`sui-schema-editor`)
- [ ] **Query** — write/run/save queries; show SQL + row count + elapsed. (`sui-query-editor` + `sui-query-runner`)
- [ ] Collection selection (`sui-collection-list`); system collections read-only.
- [ ] New collection = schema-first → `CreateSchema`.
- [ ] All states incl. live (apply Watch patches in place).

## 3. Components — `sui-collection-*`

`sui-collection-list` · `sui-collection-table` · `sui-item-detail` · `sui-item-editor`
· `sui-schema-editor` · `sui-query-editor` · `sui-query-runner` · `sui-collection-picker`
· `sui-watch-badge` (shared). All **new**. See [`./spec.md`](./spec.md) for each.

## 4. Fixtures

[`./fixtures/collections.demo.json`](./fixtures/collections.demo.json) — 5 collections
(3 user, 2 system), decoded schemas (full type vocab + constraints), items, named
queries (with SQL + timing), 5-frame watch stream, error tones, empty copy.

## 5. Related view

- Route `/collections` → `reference/views/collections/page.sui`. Host scaffold: `shell`.
- **Primary deliverable:** [`./components.html`](./components.html) — the `sui-collection-*` component set (9 components, each isolated, schema-typed, themeable).
- Data Studio composition view: _not built yet._

## Deferred (don't design yet)

Kysely-TS autocomplete (waits on `@ship/db`); per-collection permission UI;
reproject progress beyond the warning.
