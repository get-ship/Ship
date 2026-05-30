---
title: Collections
description: A collection is a typed dataset — declare its shape in JSONC and get storage, queries, live events, and agent access with no database layer.
section: Data
order: 2
kicker: Data
tags: [data, collections]
related: [storage]
---

A **collection** is a typed dataset. You declare its shape in a small JSONC file; Ship gives you storage, queries, real-time events, and agent access — without writing a database layer.

## When to reach for one

- **Structured records** — tasks, ADRs, interviews, content drafts; anything you'd otherwise put in a Notion database or an Airtable base.
- **Data your views display** — a view's template reads collection items directly: list, filter, edit, no API plumbing.
- **A target for automation** — a trigger or pipeline can insert into or update a collection in response to an event, turning automation into structured records.

## A small example

`.ship/collections/task.jsonc`:

```jsonc
{
  "id": "task",
  "name": "Task",
  "fields": {
    "title":    { "type": "string", "required": true },
    "status":   { "type": "enum", "values": ["todo", "doing", "done"] },
    "priority": { "type": "number" }
  }
}
```

That's enough to get a typed table, a CRUD API, an MCP surface for agents, and live events on every change. No code, no migrations.

## What you get

- **Typed storage** — Ship creates a real table for each collection in your space's local database.
- **Query, sort, filter** — a simple structured query language for finding what you want.
- **Live events** — `collection.item_*` events fire on every mutation; views and triggers can listen.
- **Agent access** — agents read and write collections over MCP without you wiring up custom tools.
