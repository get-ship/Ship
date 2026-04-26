---
title: "Collections as the Universal Data Layer"
description: "ADR: all user-authored content goes through collections in data.db; platform.db holds only runtime state."
group: ADRs
order: 10
audience: internal
status: accepted
date: 2026-04-19
supersedes: []
tags: [storage, collections, platform-db, data-db]
---

# Collections as the Universal Data Layer

## Context

Ship has accumulated multiple storage mechanisms for user content:

- **`.artifacts/`** — filesystem directory for specs, job outputs, pipeline records, QA reports. No API, no schema, no events. Agents write here as a scratch space.
- **`platform.db` tables** — `note`, `adr`, `target`, `capability` tables store structured user content alongside runtime state (events, sessions, actors, jobs). These are user data mixed into the runtime database.
- **`data.db` collections** — schema-first, queryable, event-emitting. Built for exactly this purpose but adopted by nothing yet.

This creates three problems:

1. User content is scattered across filesystem and two databases with no unified query API.
2. `platform.db` conflates runtime state with user data. Deleting runtime state (reset, migration) risks losing user work. Taking your data to another machine means extracting rows from a database that also contains session heartbeats and actor registrations.
3. `.artifacts/` is a convention, not a system. No events emitted on write, no schema validation, no way for views to query artifacts, no sync story.

Meanwhile, the collections system in `data.db` already provides: JSONC schemas, CRUD, Django-style query filters, auto-indexing, and event emission on mutation. It's the right tool. Nothing uses it.

## Decision

**All user-authored content goes through collections in `data.db`.** Specifically:

### What moves

| Content | From | To | Collection |
|---------|------|----|------------|
| Specs | `.artifacts/specs/` + ad-hoc files | `data.db` | `specs` (built-in schema) |
| ADRs | `platform.db` `adr` table | `data.db` | `adrs` (built-in schema) |
| Notes | `platform.db` `note` table | `data.db` | `notes` (built-in schema) |
| Targets/capabilities | `platform.db` `target` + `capability` | `data.db` | `targets`, `capabilities` (built-in schemas) |
| Feedback/annotations | (new) | `data.db` | `feedback` (built-in schema) |
| File metadata | (new) | `data.db` | `files` (built-in schema, blobs in `.ship/files/`) |

### What stays

| Content | Where | Why |
|---------|-------|-----|
| Events | `platform.db` `events` | Runtime audit trail, not user content |
| Workspaces | `platform.db` `workspace` | Runtime state |
| Sessions | `platform.db` `workspace_session` | Runtime state |
| Actors | `platform.db` `actors` | Runtime state |
| Jobs | `platform.db` `jobs` | Runtime state (job outputs go to collections via hooks) |
| File claims | `platform.db` `file_claim` | Runtime coordination |
| Managed MCP state | `platform.db` `managed_mcp_state` | Runtime state |
| Agent artifact registry | `platform.db` `agent_artifact_registry` | Compiler cache |

### What `.artifacts/` becomes

A **working directory for agents**, not a data store. Agents write intermediate outputs here during pipeline runs and sessions. Final results are committed to collections (via hooks or explicit pipeline steps). `.artifacts/` is gitignored, ephemeral, and not the source of truth for anything.

Existing `.artifacts/specs/` content is migrated to the `specs` collection on first run after upgrade.

## Built-in collection schemas

These ship with the runtime. Users don't create them. They follow the same schema format and query API as user-defined collections.

```jsonc
// Embedded in runtime, not in .ship/schemas/
{
  "id": "specs",
  "name": "Specs",
  "builtin": true,
  "fields": {
    "title":       { "type": "string", "required": true, "filterable": true },
    "status":      { "type": "enum", "values": ["draft", "in-progress", "complete", "abandoned"], "filterable": true },
    "description": { "type": "markdown" },
    "acceptance":  { "type": "markdown" },
    "scope":       { "type": "array" },
    "tags":        { "type": "array", "filterable": true }
  }
}
```

```jsonc
{
  "id": "feedback",
  "name": "Feedback",
  "builtin": true,
  "fields": {
    "target_type": { "type": "enum", "values": ["pipeline_run", "agent_output", "collection_item", "file", "session"], "required": true, "filterable": true },
    "target_id":   { "type": "string", "required": true, "filterable": true },
    "rating":      { "type": "number" },
    "annotation":  { "type": "markdown" },
    "tags":        { "type": "array", "filterable": true },
    "actor":       { "type": "string", "filterable": true }
  }
}
```

```jsonc
{
  "id": "files",
  "name": "Files",
  "builtin": true,
  "fields": {
    "name":          { "type": "string", "required": true },
    "path":          { "type": "string", "required": true },
    "mime":          { "type": "string", "filterable": true },
    "size":          { "type": "number" },
    "collection_id": { "type": "string", "filterable": true },
    "item_id":       { "type": "string" }
  }
}
```

## Consequences

### Positive

- **One query API for all user data.** Views, pipelines, agents, and hooks query collections uniformly. No special cases for "this is a spec" vs "this is a note" vs "this is user-created."
- **Events on all mutations.** Hooks can react to spec changes, new feedback, file uploads — all through the same `collection.item_created` event with different collection IDs.
- **Sync for free.** When event replication ships, all user content syncs because it's all in `data.db` emitting events. No per-table sync logic.
- **Clean database boundary.** `platform.db` is Ship's brain (runtime state). `data.db` is the user's brain (their content). Either can be reset independently.
- **Files become queryable.** File metadata in a collection means "show me all PDFs uploaded this week" is a collection query, not a filesystem walk.

### Negative

- **Migration required.** Existing `platform.db` tables (`note`, `adr`, `target`, `capability`) need one-time migration to `data.db`. Existing `.artifacts/specs/` files need import.
- **Built-in schemas are rigid.** If the `specs` schema needs a new field, it's a runtime code change, not a user config change. Mitigant: built-in schemas are intentionally minimal. Users create their own collections for custom structures.
- **Performance characteristics change.** Filesystem reads (`.artifacts/`) become database queries. For the volumes involved (hundreds of specs, not millions) this is strictly faster, but it's a different failure mode.

## Alternatives Considered

### Keep `.artifacts/` as primary storage, add an API

Add a `/api/artifacts` endpoint that reads/writes the filesystem. Simpler migration but: no schema validation, no events, no query filters, no sync story. Just an HTTP wrapper around `fs::read`/`fs::write`. This pushes the problem forward instead of solving it.

### Merge everything into `platform.db`

Simpler (one database), but violates the "user data vs runtime state" boundary. Makes it impossible to "take my data elsewhere" without also exporting session heartbeats and actor registrations.

### Use a separate document database (e.g., embedded Mongo-like)

Overkill. SQLite with JSON columns already provides document-style storage with relational query capabilities. Adding another database engine increases complexity without proportional benefit.

## Migration Plan

1. Create built-in collection schemas in runtime (code change, no user action).
2. On daemon startup, check for `platform.db` tables that should be in `data.db`.
3. If found: copy rows to `data.db` collections, drop tables from `platform.db`.
4. Scan `.artifacts/specs/` for markdown files with YAML frontmatter.
5. Import each as a `specs` collection item (title from frontmatter, content as markdown field).
6. Log migration results, don't delete originals until confirmed.

## How to Measure Success

- Zero user content in `platform.db` after migration.
- All first-party views that show user content query collections (not filesystem or `platform.db` directly).
- `.artifacts/` contains only ephemeral agent working files.
- New features (feedback, files) built on collections from day one, no special storage.
