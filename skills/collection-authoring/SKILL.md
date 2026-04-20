---
name: collection-authoring
stable-id: collection-authoring
description: Hands-on guide to writing a collection schema for Ship. Covers the schema file shape, field-type selection, filterable/required/indexed flags, naming conventions, testing the schema against the API, and common patterns. Pair with ship-collections (reference).
tags: [collections, schema, extension-authoring, jsonc]
authors: [ship]
---

# Collection Authoring

You write one JSONC file at `.ship/schemas/<id>.jsonc`, the daemon picks it up, and you have a typed table with CRUD, query, events, and API access. This skill is the practical companion to `ship-collections` — read that for concept depth and the full query-operator list. Read this when you are sitting down to design a new collection.

## The one-screen schema

```jsonc
// .ship/schemas/todo.jsonc
{
  "id": "todo",
  "name": "Todo",
  "fields": {
    "title":      { "type": "string",   "required": true,  "filterable": true },
    "status":     { "type": "enum",     "values": ["open", "doing", "done"], "filterable": true, "default": "open" },
    "priority":   { "type": "enum",     "values": ["low", "normal", "high"], "filterable": true, "default": "normal" },
    "notes":      { "type": "markdown" },
    "due_date":   { "type": "date",     "filterable": true },
    "tags":       { "type": "array",    "filterable": true },
    "archived":   { "type": "boolean",  "filterable": true, "default": false }
  }
}
```

Drop the file in `.ship/schemas/`, hit `POST /api/collections/todo/items`, done.

## The authoring checklist

1. **`id` is the handle.** Lowercase, underscore-separated, singular (`todo`, not `todos`). It's the URL slug, the event namespace, the Jinja template name. Rename = migration.
2. **Pick field types deliberately** — see §Field types.
3. **Mark what you filter on.** Every field the user sorts, filters, or searches on needs `filterable: true`. The daemon auto-creates an index; queries stay fast under volume.
4. **Declare required early.** `required: true` at author time beats adding it later when half your rows violate it.
5. **Provide `default` for enums and booleans.** It removes a class of `NULL` handling bugs in views and triggers.
6. **No foreign keys.** If `task.project_id` references a project, store the ID string and look up at query time. Collections don't enforce referential integrity — this is deliberate. Use a trigger if cascade behavior matters.
7. **No timestamps in the schema.** `created_at` and `updated_at` are managed automatically. Don't declare them. (They're always returned in item responses.)

## Field types

| Type | When to use | Gotchas |
|---|---|---|
| `string` | Short text — title, slug, external id | Don't use for anything > ~200 chars. Use `markdown`. |
| `markdown` | Long-form content — notes, descriptions, body | Rendered by views via the `markdown` Jinja filter. Not indexed. |
| `number` | Numeric — count, score, price | Stored as double (REAL). No int/float distinction. |
| `boolean` | Flag — `archived`, `starred`, `is_public` | Coerced on read; safe to filter with `true`/`false`. |
| `enum` | Fixed list — status, priority, type | Values enforced on write. Adding a new value is safe; removing is not. |
| `date` | Date-only — due date, published date | ISO 8601 (`YYYY-MM-DD`). Use `datetime` if you need time-of-day. |
| `datetime` | Timestamp — event time, snapshot time | ISO 8601 with timezone. |
| `array` | List of strings — tags, labels, categories | Filterable via `{ "$contains": "x" }`. Items are always strings today. |
| `object` | Structured blob — metadata, config, nested payloads | Not filterable past the top level. If you need to query inside, promote to its own collection. |
| `file` | File reference | Stores a file ID; resolve via the files API. |

For the authoritative storage mapping + full operator list see `docs/ship-collections/references/docs/schemas.md` and `querying.md`.

## Field-level options

```jsonc
"status": {
  "type":       "enum",
  "values":     ["open", "closed"],   // enum only
  "required":   true,                  // must be present on write
  "filterable": true,                  // indexed; usable in filters
  "default":    "open",                // applied if field missing on write
  "description": "Workflow state"      // shows up in schema-editor UI
}
```

`filterable` vs `indexed`: `filterable` is the author-facing switch. The daemon creates the index automatically for every filterable field — you don't pass `indexed: true` separately.

## Naming

- `id`: snake_case, singular (`bug_triage`, `daily_log`).
- Field keys: snake_case, descriptive. Avoid abbreviations (`priority` not `pri`).
- Avoid names that collide with built-ins: `id`, `created_at`, `updated_at`. The daemon refuses writes using these field names.
- Built-in collection ids are reserved: `specs`, `adrs`, `notes`, `feedback`, `files`. Don't shadow them.

## Testing your schema

```bash
# 1. Drop the schema file
$EDITOR .ship/schemas/todo.jsonc

# 2. Seed an item
curl -X POST http://localhost:9315/api/collections/todo/items \
  -H 'Content-Type: application/json' \
  -d '{"title":"ship it","status":"open","tags":["launch"]}'

# 3. Query it
curl http://localhost:9315/api/collections/todo/items?filter=%7B%22status%22%3A%22open%22%7D

# 4. Watch the events
ship events tail --filter 'collection.item_*'
```

If step 2 returns a validation error, read the message — the validator names the field and the problem. If the column isn't filterable but you filtered on it, the query returns an error listing the filterable fields.

## Common patterns

### Kanban-style status tracking
```jsonc
"status": { "type": "enum", "values": ["backlog", "in_progress", "review", "done"], "filterable": true, "default": "backlog" }
```

### Soft delete
```jsonc
"archived": { "type": "boolean", "default": false, "filterable": true }
```
Queries filter `archived: false` to hide without delete. Add a trigger on `collection.item_updated` to cascade effects if needed.

### Tag/category
```jsonc
"tags": { "type": "array", "filterable": true }
```
Filter by `{ "tags": { "$contains": "launch" } }`.

### Assignee / owner
```jsonc
"assignee_id": { "type": "string", "filterable": true }
```
Store the actor id. Resolve to a display name at query time in the view.

### Aggregate source reference
```jsonc
"source_event": { "type": "string" }
```
For items derived by a trigger — store the triggering event id so you can trace back.

## Schema evolution

- Adding a new field: safe. Existing rows return `null` for the new field until written.
- Adding a new enum value: safe.
- Removing a field: the column stays, just stop writing to it. True removal is a migration.
- Changing a field's type: not supported in-place. Create a new field, migrate with a script, delete the old.
- Renaming the collection: migration. Don't.

## Virtual collections (read-only external data)

If the data lives in another system (GitHub issues, Jira tickets) and you don't want Ship to own a copy, see `ship-collections` §Virtual. The schema file has a `virtual: true` flag, a `connection` reference, and field `map:` expressions — the query API translates filters to the external API. Out of scope for local authoring.

## When NOT to make a new collection

- You need to store arbitrary short-lived state for a view → use view preferences (localStorage) instead.
- You need a per-user setting → use `/api/settings`.
- You need to cache an external query result → that's a virtual collection, not a local one.
- You need one row with global config → a kv table fits better; don't spin up a collection for a single row.

## Relation to other systems

- **Views** consume collections via the query API. See `ship-views` for the Jinja helpers (`ship_query`, `ship_items`, `| where`).
- **Triggers** react to `collection.item_created`/`_updated`/`_deleted` events. See `trigger-authoring`.
- **Hooks** (agent/provider layer) have nothing to do with collections.
